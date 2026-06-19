use std::fmt::Display;
use std::process::Command;
use log::{error, warn, debug};

use crate::wheel::{ExecuteError, WindowInfo, Workspace, hyprctl};

fn hyprctl_dispatch(command: &str) -> Result<String, ExecuteError>{
    let output = Command::new("hyprctl")
    .arg("dispatch")
    .arg(command)
    .output()
    .map_err(|e| ExecuteError::FailedToExecute("hyprctl".to_string(), e.to_string()))?;

    if !output.status.success() {
        // CLI tool hyprctl output error message to stdout, but error message may be printed to stderr in future.
        // So read stdout first and read sdterr as failback
        let error_msg = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        };
        // I can't get error output from hyprctl via stderr, it means hyprctl print error message to stdout.
        return Err(ExecuteError::FailedToExecute(
            "hyprctl dispatch".to_string(),
            error_msg.trim().to_string()
        ));
    }

    String::from_utf8(output.stdout)
    .map_err(|e| ExecuteError::FailedToExecute(
        "hyprctl dispatch".to_string(),
        format!("Invalid UTF-8: {e}")
    ))
}

fn get_window_by_address(address: &str ) -> Result<WindowInfo, ExecuteError> {
    let clients: Vec<WindowInfo>  = hyprctl("clients")?;
    clients
    .into_iter()
    .find(|c| c.address == address)
    .ok_or(ExecuteError::FailedToFind(format!("window with address: {address}")))
}

fn get_active_workspace() -> Option<Workspace> {
    match hyprctl("activeworkspace") {
        Ok(o) => Some(o),
        Err(e) => {error!("Can not find an active workspace: {e}"); None},
    }
}

fn format_win_addr (raw_addr: &str) -> String {
    if raw_addr.starts_with("0x") {
        raw_addr.to_string()
    } else {
        format!("0x{raw_addr}" )
    }
}

fn move_window_to_workspace<T:Display>(workspace_id: T, follow: bool,win_addr: &str) {
    match hyprctl_dispatch(&format!("hl.dsp.window.move({{ workspace = \"{workspace_id}\", follow = {follow}, window = \"address:{win_addr}\" }})")) {
        Ok(o) => debug!("Successfully moved window: {win_addr} to workspace: {workspace_id}, result: {o}"),
        Err(e) => error!("{e}"),
    }
}

fn set_window_on_top(win_addr: &str) {
    match hyprctl_dispatch(&format!("hl.dsp.window.alter_zorder({{ mode = \"top\", window = \"address:{win_addr}\" }})")) {
        Ok(o) => debug!("Successfully moved window: {win_addr} to the top, result: {o}"),
        Err(e) => error!("{e}"),
    }
}

fn change_focus(win_addr: &str) {
    match hyprctl_dispatch(&format!("hl.dsp.focus({{ window = \"address:{win_addr}\"}})")) {
        Ok(o) => debug!("Successfully changed focus of window: {win_addr}, result: {o}"),
        Err(e) => error!("{e}"),
    }
}

fn is_window_in_minimized_workspace(current_window: &WindowInfo) -> bool {
    current_window.workspace.name == "special:minimized"
}

fn restore_window_from_workspace(ws: &Workspace, win_addr: &str) {
    move_window_to_workspace(ws.id, true, win_addr);
    set_window_on_top(win_addr);
}

/// Handle minimize events from the compositor.
pub fn handle_minimize_event(addr: &str) {
    debug!("Handling minimize event: {addr}");
    let Some((cut_addr, _)) = addr.split_once(',') else {
            warn!("Invalid address format: {addr}");
            return;
    };
    let win_addr = format_win_addr(cut_addr);
    let Some(active_workspace) = get_active_workspace() else {
        return;
    };
    let current_window = match get_window_by_address(&win_addr) {
        Ok(addr) => addr,
        Err(e) => {warn!("{e}"); return;},
    };
    let is_active = current_window.focus_history_id == 0;
    let is_minimize = is_window_in_minimized_workspace(&current_window);
    match (is_active, is_minimize) {
        (true, false) => move_window_to_workspace("special:minimized",false,&win_addr),
        (true, true) | (false, true) => restore_window_from_workspace(&active_workspace, &win_addr),
        (false, false) => {change_focus(&win_addr); set_window_on_top(&win_addr)},
    }
}

/// Move an active window to the current workspace if window stay on a special workspace
pub fn handle_active_window(addr: &str) {
    debug!("Handling active window event: {addr}");
    let Some(active_workspace) = get_active_workspace() else {
        return;
    };
    let win_addr = format_win_addr(addr);
    let this_window = match get_window_by_address(&win_addr) {
        Ok(addr) => addr,
        Err(e) => {warn!("{e}"); return;},
    };
    if is_window_in_minimized_workspace(&this_window)
        && this_window.focus_history_id == 0 {
            restore_window_from_workspace(&active_workspace, &win_addr);
        }
}

/// Move a window to the current workspace when receive a urgent action.
pub fn handle_urgent_window(addr: &str) {
    debug!("Handling urgent window event: {addr}");
    let Some(active_workspace) = get_active_workspace() else {
        return;
    };
    let win_addr = format_win_addr(addr);
    let target_window = match get_window_by_address(&win_addr) {
        Ok(addr) => addr,
        Err(e) => {warn!("{e}"); return;},
    };

    if is_window_in_minimized_workspace(&target_window) {
        restore_window_from_workspace(&active_workspace, &win_addr);
    }
}
