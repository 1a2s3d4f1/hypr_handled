use log::error;
use serde::Deserialize;
use std::path::PathBuf;
use std::{fmt::Debug, process::Command,fmt};
use std::os::unix::net::UnixStream;
use std::io::Read;
use std::env;
use crate::x11_dpi_set::set_xrdb_dpi;
use crate::xdg_term_bell::play_audio;
use crate::minimize::{handle_active_window,handle_minimize_event,handle_urgent_window};

#[derive(Debug)]
pub enum ExecuteError {
    //Failed to execute a command (app name, reason)
    FailedToExecute(String, String),
    JsonReadError(serde_json::Error),
    FailedToFind(String),
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
        ExecuteError::FailedToExecute(cmd, why) => write!(f, "Failed to execute \'{cmd}\': {why}"),
        ExecuteError::JsonReadError(info) => write!(f, "Failed to read json: {info}"),
        ExecuteError::FailedToFind(target) => write!(f, "Failed to find {target}"),
        }
    }
}

impl std::error::Error for ExecuteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExecuteError::JsonReadError(e) => Some(e),
            _ => None,
        }
}
}

#[derive(Deserialize, Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WindowInfo {
    pub address: String,
    pub workspace: Workspace,
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i32,
}

#[cfg(target_os  = "linux")]
pub fn hyprctl<T: for<'de> Deserialize<'de>>(command: &str) -> Result<T, ExecuteError> {
    let output = Command::new("hyprctl")
    .arg("-j")
    .arg(command)
    .output()
    .map_err(|e| ExecuteError::FailedToExecute("hyprctl".to_string(), e.to_string()))?;

    if !output.status.success() {
        // Hyprland may fix this problem in future
        let error_msg = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        };
        // I can't get error output from hyprctl via stderr, maybe hyprctl print error message to stdout
        return Err(ExecuteError::FailedToExecute("hyprctl".to_string(), error_msg.to_string()));
    }
    match serde_json::from_slice(&output.stdout) {
        Ok(o) => Ok(o),
        Err(e) => Err(ExecuteError::JsonReadError(e)),
    }
}

pub fn get_var (name: &str) -> Option<String> {
    match env::var(name) {
        Ok(val) => Some(val),
        Err(e) => {error!("get_var failed: {e}"); None},
    }
}

fn get_socket_dir () -> Option<PathBuf> {
    let mut path_buf = PathBuf::from(get_var("XDG_RUNTIME_DIR")?);
    path_buf.push("hypr");
    path_buf.push(get_var("HYPRLAND_INSTANCE_SIGNATURE")?);
    path_buf.push(".socket2.sock");

    Some(path_buf)
}

pub fn event_stream()  {
    let Some(socket_path) = get_socket_dir() else {
        error!("Failed to determine socket path (missing environment variables)"); return;
    };
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(stre) => stre,
        Err(e) => {error!("Failed to connect unix socket: {e}"); return;},
    };
    let mut buffer = vec![0; 8192];
    loop {
    let byte = stream.read(&mut buffer);
    let b = match byte {
        Ok(b) => {
            b
        },
            Err(e) =>  {error!("Can't get information from socket {e}");break;},
        };

        let buf = &buffer[..b];
        let string = String::from_utf8_lossy(buf).to_string();
        handle_event(&string);
    }
}

pub fn split_event(event: &str) -> Option<(&str, &str)> {
    let trimmed = event.trim();
    if trimmed.is_empty() {
        None
    } else {
    trimmed.split_once(">>")
    }
}

pub fn handle_event(event: &str) {
    for event in event.lines() {
        if let Some((name, message)) = split_event(event) {
            wait_events_for_handle((name, message));
        }
    }
}

pub fn wait_events_for_handle(event_line: (&str,&str)) {
    let name =event_line.0;
    let message = event_line.1;
    match name {
        "urgent" => handle_urgent_window(message),
        "minimized" => handle_minimize_event(message),
        "activewindowv2" => handle_active_window(message),
        "bell" => {
            if let Err(e) = play_audio() {
                error!("failed to ring system bell, reason: {e}");
            }
        },
        "configreloaded" => set_xrdb_dpi(),
        _=> {},
    }
}
