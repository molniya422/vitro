//! Versioned settings with atomic write, `.bak` rotation, and recovery.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub icon_size: u32,
    pub magnify: bool,
    pub autohide: bool,
    pub reduced_motion: bool,
    pub profile: String,
    pub show_monitor: bool,
    pub pinned: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA,
            icon_size: 48,
            magnify: true,
            autohide: false,
            reduced_motion: false,
            profile: "vitro".into(),
            show_monitor: false,
            pinned: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("settings file is corrupt: {detail}")]
    Corrupt { detail: String },
    #[error("unsupported settings schema {0}")]
    UnsupportedSchema(u32),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".bak");
    path.with_file_name(name)
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

fn read_and_validate(path: &Path) -> Result<Settings, SettingsError> {
    let bytes = std::fs::read(path)?;
    let parsed: Settings = serde_json::from_slice(&bytes).map_err(|e| SettingsError::Corrupt {
        detail: e.to_string(),
    })?;
    if parsed.schema_version != CURRENT_SCHEMA {
        return Err(SettingsError::UnsupportedSchema(parsed.schema_version));
    }
    Ok(parsed)
}

/// Write `settings` to `path` via temp file, backup rotation, then rename.
pub fn save_atomic(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = temp_path(path);
    let bytes = serde_json::to_vec_pretty(settings).expect("settings serialize");

    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&bytes)?;
        file.flush()?;
        file.sync_all()?;
    }

    if path.exists() {
        let bak = backup_path(path);
        // Ignore a failure to overwrite an old backup: the fresh write still wins.
        let _ = std::fs::rename(path, &bak);
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Load settings, falling back to `.bak` before surfacing an error.
///
/// A missing main file and a missing backup both mean "first run" -> defaults.
pub fn load_or_recover(path: &Path) -> Result<Settings, SettingsError> {
    match read_and_validate(path) {
        Ok(settings) => return Ok(settings),
        Err(SettingsError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(SettingsError::UnsupportedSchema(v)) => {
            // A known-but-newer schema is not corrupt data; do not mask it with a backup.
            return Err(SettingsError::UnsupportedSchema(v));
        }
        Err(_) => {}
    }

    let bak = backup_path(path);
    match read_and_validate(&bak) {
        Ok(settings) => Ok(settings),
        Err(SettingsError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(Settings::default())
        }
        Err(other) => Err(match other {
            SettingsError::Corrupt { detail } => SettingsError::Corrupt {
                detail: format!("main and backup both unreadable: {detail}"),
            },
            SettingsError::Io(e) => SettingsError::Corrupt {
                detail: format!("backup unreadable: {e}"),
            },
            schema @ SettingsError::UnsupportedSchema(_) => schema,
        }),
    }
}
