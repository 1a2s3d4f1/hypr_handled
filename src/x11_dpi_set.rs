use std::process::Command;
use crate::wheel::{ExecuteError, hyprctl};
use log::{error, warn};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
struct MonitorInfo {
    scale: f32,
    focused: bool,
}

fn xrdb (flag: &str) -> Result<(),ExecuteError>{
    let output = Command::new("xrdb")
    .arg("-merge")
    .arg(flag)
    .output()
    .map_err(|e| ExecuteError::FailedToExecute("xrdb".to_string(), e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ExecuteError::FailedToExecute("xrdp".to_string(), stderr.to_string()));
    }
    Ok(())
}

fn get_monitor_info () -> Result<MonitorInfo, ExecuteError> {
    let clients: Vec<MonitorInfo>  = hyprctl("monitors")?;
    clients
    .into_iter()
    .find(|c| c.focused)
    .ok_or(ExecuteError::FailedToFind("active monitor".to_string()))
}

fn get_monitor_scale () -> f32 {
    match get_monitor_info() {
        Ok(o) => o.scale,
        Err(e) => { warn!("[font dpi set] Can't get scale of screen: {e}"); 1.0},
    }
}

pub fn set_xrdb_dpi () {
    const X_DPI :f32 = 96.0;
    let xft_dpi_f = get_monitor_scale() * X_DPI;
    let xft_dpi_i = xft_dpi_f as i32;
    let xft_dpi_s = format!("Xft.dpi: {xft_dpi_i}");
    let path = Path::new("/tmp/hypr_handled/hypr_x_dpi");
    let display = path.display();
    let mut file = match File::create(path) {
        Err(why) => {error!("[font dpi set] Failed to creat {display}: {why}"); return;},
        Ok(file) => file,
    };
    if let Err(e) = file.write_all(xft_dpi_s.as_bytes()) {
        error!("[font dip set] Failed to write {display}: {e}");
    }
    if let Err(e) = xrdb("/tmp/hypr_handled/hypr_x_dpi") {
        error!("{e}");
    }
}
