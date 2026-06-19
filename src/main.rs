use crate::wheel::event_stream;
use crate::x11_dpi_set::set_xrdb_dpi;
use log::info;
use crate::prepare_log::prepare_log4rs;
use std::fs::create_dir;
use std::path::Path;

mod wheel;
mod minimize;
mod xdg_term_bell;
mod x11_dpi_set;
mod prepare_log;

fn main() {
    //env_logger::Builder::from_env(Env::default().default_filter_or("info")).format_timestamp_secs().init();
    if let Err(e) = prepare_log4rs() {
        eprintln!("failed to ring system bell, reason: {e}");
    }

    let work_dir = Path::new("/tmp/hypr_handled");
    if work_dir.metadata().is_err() {
        match create_dir(work_dir) {
            Ok(o) => o,
            Err(e) => eprintln!("Failed to create a directory: {e}"),
        };
    }

    info!("Initialize hypr_handled now");
    set_xrdb_dpi();
    event_stream();
}
