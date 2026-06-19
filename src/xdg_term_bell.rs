use std::process::Command;
use std::fs::File;
use std::path::{PathBuf, Path};
use log::error;
use rodio::decoder::DecoderError;
use rodio::stream::StreamError;
use rodio::Decoder;
use std::{thread, fmt};
use crate::wheel::get_var;

#[derive(Debug)]
pub enum AudioError {
    GsettingsFailed(String),
    NoSoundFileFound,
    DecoderError(DecoderError),
    StreamError(StreamError),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AudioError::GsettingsFailed(why) => write!(f, "GsettingsFailed: {why}"),
            AudioError::NoSoundFileFound => write!(f, "NoSoundFileFound"),
            AudioError::DecoderError(d) => write!(f, "{d}"),
            AudioError::StreamError(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(target_os  = "linux")]
fn get_current_sound_theme() -> Result<String, AudioError> {
    let output = Command::new("gsettings")
    .arg("get")
    .arg("org.gnome.desktop.sound")
    .arg("theme-name")
    .output()
    .map_err( |e| AudioError::GsettingsFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AudioError::GsettingsFailed(stderr.to_string()))
    }

    let theme = String::from_utf8(output.stdout)
    .map_err(|_| AudioError::GsettingsFailed("Invaild UTF-8".to_string()))?
    .trim()
    .trim_start_matches('\'')
    .trim_end_matches('\'')
    .to_string();

    Ok(if theme.is_empty() { "freedesktop".to_string() } else { theme })
}

fn find_sound_file() -> Result<PathBuf, AudioError> {
    let theme = get_current_sound_theme()?;
    if let Some(path) = find_sound_file_in_user_dir(&theme) {
        return Ok(path);
    }
    
    find_sound_file_in_sys_dir(&theme)
    .ok_or(AudioError::NoSoundFileFound )
}

fn find_sound_file_in_dir (base_dir: &Path, theme: &str) -> Option<PathBuf> {
    let stereo_dir = base_dir.join(theme).join("stereo");

    if stereo_dir.metadata().is_err() {
        return None;
    }

    for ext in &["oga", "ogg", "wav"] {
        let path = stereo_dir.join(format!("bell.{ext}"));
        if path.metadata().is_ok() {
            return Some(path);
        }
    }
    None
}

fn find_sound_file_in_user_dir(theme: &str) -> Option<PathBuf> {
    let home = match get_var("HOME") {
        Some(s) => s,
        None => String::from("null"),
    };

    let user_dir = PathBuf::from(home).join(".local/share/sounds");
    find_sound_file_in_dir(&user_dir, theme)
}

fn find_sound_file_in_sys_dir(theme: &str) -> Option<PathBuf> {
    let system_dir = PathBuf::from("/usr/share/sounds");
    find_sound_file_in_dir(&system_dir, theme)
}

fn play_sound_file(path: &PathBuf) -> Result<(), AudioError> {
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
    .map_err( AudioError::StreamError)?;

    let file = File::open(path)
    .map_err(|_| AudioError::NoSoundFileFound)?;

    let source = Decoder::try_from(file)
    .map_err( AudioError::DecoderError)?;

    let sink = rodio::Sink::connect_new(stream_handle.mixer());
    sink.append(source);
    sink.sleep_until_end();
    sink.clear();

    Ok(())
}

pub fn play_audio() -> Result<thread::JoinHandle<()>, AudioError> {
    let sound_path = find_sound_file()?;

    let handle = thread::spawn( move || {
        if let Err(e) = play_sound_file(&sound_path) {
            error!("[bell] Failed to play sound {e}");
        }
    });
    Ok(handle)
}
