use log::{LevelFilter, SetLoggerError};
use std::fmt;
use log4rs::{Handle,
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::policy::compound::{
            roll::fixed_window::FixedWindowRoller,
            trigger::size::SizeTrigger,
            CompoundPolicy,
        }
    },
    config::{Appender, Config, Root, runtime::ConfigErrors},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter
};

// Trigger logging file slice size in byte. 2*1024 means 2kb
const TRIGGER_FILE_SIZE: u64 = 1024 * 1024;

const LOG_FILE_COUNT: u32 = 3;
// Path of logging file
const FILE_PATH: &str = "/tmp/hypr_handled/logs/latest.log";
//
const ARCHIVE_PATTERN: &str = "/tmp/hypr_handled/logs/hypr_handle.{}.log";

#[derive(Debug)]
pub enum StartLoggerError {
    ConfigFailed(ConfigErrors),
    InvokeLoggerError(SetLoggerError),
    IoError(std::io::Error),
    BuildFixedWindowRollerError(String)
}

impl fmt::Display for StartLoggerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StartLoggerError::ConfigFailed(why) => write!(f, "Failed to prepare config: {why}"),
            StartLoggerError::InvokeLoggerError(why) => write!(f, "Failed to set logger: {why}"),
            StartLoggerError::IoError(why) => write!(f, "IO error: {why}"),
            StartLoggerError::BuildFixedWindowRollerError(why) => write!(f,"Failed to build fixed window roller: {why}"),
        }
    }
}

impl std::error::Error for StartLoggerError {}

/// Prepare log4rs as logging backend
pub fn prepare_log4rs() -> Result<Handle, StartLoggerError>{
    let level = log::LevelFilter::Info;

    // Build a stderr logger.
    let stderr = ConsoleAppender::builder()
    .target(Target::Stderr)
    .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d_%H:%M:%S)} [{l}] {t} - {m}{n}")))
    .build();

    let trigger = SizeTrigger::new(TRIGGER_FILE_SIZE);
    let roller = match FixedWindowRoller::builder()
    .base(0) // Default Value (line not needed unless you want to change from 0 (only here for demo purposes)
    .build(ARCHIVE_PATTERN, LOG_FILE_COUNT) {// Roll based on pattern and max 3 archive files
        Ok(roller) => roller,
        Err(e) => return Err(StartLoggerError::BuildFixedWindowRollerError(e.to_string()))
    };

    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));

    // Logging to log file
    let logfile_roll = match log4rs::append::rolling_file::RollingFileAppender::builder()
    // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
    .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d_%H:%M:%S)} [{l}] {t} - {m}{n}")))
    .build(FILE_PATH, Box::new(policy)) {
        Ok(o) => o,
        Err(e) => return Err(StartLoggerError::IoError(e)),
    };

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
    .appender(Appender::builder()
    .filter(Box::new(ThresholdFilter::new(level)))
    .build("logfile_roll", Box::new(logfile_roll)))
    .appender(
        Appender::builder()
        .filter(Box::new(ThresholdFilter::new(level)))
        .build("stderr", Box::new(stderr)),
    )
    .build(
        Root::builder()
        .appender("logfile_roll")
        .appender("stderr")
        .build(LevelFilter::Info),
    );

    let unwrap_config = match config {
        Ok(conf) => conf,
        Err(e) => return Err(StartLoggerError::ConfigFailed(e))
    };

    let handle = match log4rs::init_config(unwrap_config) {
        Ok(handle) => handle,
        Err(e) => return Err(StartLoggerError::InvokeLoggerError(e)),
    };

    Ok(handle)
}
