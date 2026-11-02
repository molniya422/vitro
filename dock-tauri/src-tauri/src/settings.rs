//! Versioned settings with atomic write, `.bak` rotation, and recovery.
//!
//! Schema 2 turned the dock into a configurable product; schema 3 added the
//! shelf entry. Unknown fields from older files are ignored and every missing
//! field falls back to the documented default, so an older file loads without a
//! migration pass and is rewritten as the current schema on the next save.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA: u32 = 3;

/// Glass tint used when a stored value is not a valid `#rrggbb` colour.
pub const DEFAULT_TINT: &str = "#161a26";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub icon_size: u32,
    pub icon_gap: u32,
    pub padding_h: u32,
    pub padding_v: u32,
    pub pill_radius: u32,
    pub tint: String,
    pub fill_opacity: f64,
    pub icon_radius: u32,
    pub reflection: bool,
    pub reflection_strength: f64,
    pub magnify: bool,
    pub magnify_peak: f64,
    pub magnify_radius_icons: f64,
    pub profile: String,
    pub reduced_motion: bool,
    pub tooltips: bool,
    pub tooltip_delay_ms: u32,
    pub show_running_dots: bool,
    pub show_badges: bool,
    pub autohide: bool,
    pub show_delay_ms: u32,
    pub hide_delay_ms: u32,
    pub yield_fullscreen: bool,
    pub show_launchpad_entry: bool,
    pub show_monitor_entry: bool,
    pub show_shelf_entry: bool,
    /// Ids of the apps shown in the dock, in display order.
    pub pinned: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA,
            icon_size: 48,
            icon_gap: 8,
            padding_h: 16,
            padding_v: 10,
            pill_radius: 34,
            tint: DEFAULT_TINT.into(),
            fill_opacity: 0.62,
            icon_radius: 22,
            reflection: true,
            reflection_strength: 0.25,
            magnify: true,
            magnify_peak: 1.6,
            magnify_radius_icons: 2.2,
            profile: "vitro".into(),
            reduced_motion: false,
            tooltips: true,
            tooltip_delay_ms: 500,
            show_running_dots: true,
            show_badges: true,
            // Auto-hide is on for new installs. A schema-1 file keeps its own
            // stored value; migration never overrides an explicit choice.
            autohide: true,
            show_delay_ms: 300,
            hide_delay_ms: 500,
            yield_fullscreen: true,
            show_launchpad_entry: true,
            show_monitor_entry: true,
            show_shelf_entry: true,
            pinned: Vec::new(),
        }
    }
}

fn clamp_finite(value: f64, min: f64, max: f64) -> f64 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        min
    }
}

fn normalize_tint(value: &str) -> String {
    let trimmed = value.trim().to_ascii_lowercase();
    match trimmed.strip_prefix('#') {
        Some(digits) if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) => {
            trimmed
        }
        _ => DEFAULT_TINT.to_string(),
    }
}

impl Settings {
    /// Clamp every value into its documented range.
    ///
    /// The UI cannot produce out-of-range values, but the settings file is
    /// plain JSON and may be edited by hand. Sanitising on load and before save
    /// keeps geometry and CSS from receiving values they cannot render.
    pub fn sanitize(&mut self) {
        self.schema_version = CURRENT_SCHEMA;
        self.icon_size = self.icon_size.clamp(32, 80);
        self.icon_gap = self.icon_gap.clamp(0, 24);
        self.padding_h = self.padding_h.clamp(4, 32);
        self.padding_v = self.padding_v.clamp(2, 24);
        self.pill_radius = self.pill_radius.clamp(0, 64);
        self.fill_opacity = clamp_finite(self.fill_opacity, 0.15, 1.0);
        self.icon_radius = self.icon_radius.clamp(0, 50);
        self.reflection_strength = clamp_finite(self.reflection_strength, 0.0, 1.0);
        self.magnify_peak = clamp_finite(self.magnify_peak, 1.0, 2.0);
        self.magnify_radius_icons = clamp_finite(self.magnify_radius_icons, 1.0, 5.0);
        self.tooltip_delay_ms = self.tooltip_delay_ms.clamp(0, 2_000);
        self.show_delay_ms = self.show_delay_ms.clamp(0, 1_000);
        self.hide_delay_ms = self.hide_delay_ms.clamp(100, 3_000);
        self.tint = normalize_tint(&self.tint);
        self.profile = match self.profile.trim().to_ascii_lowercase().as_str() {
            "macos" => "macos".to_string(),
            _ => "vitro".to_string(),
        };
    }
}

/// The camelCase settings payload React reads and writes.
///
/// The on-disk format stays snake_case so existing files keep loading; this DTO
/// exists only at the IPC boundary. Pins are managed by `pin_app`/`unpin_app`
/// and never by the form, and the schema version is owned by Rust.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub icon_size: u32,
    pub icon_gap: u32,
    pub padding_h: u32,
    pub padding_v: u32,
    pub pill_radius: u32,
    pub tint: String,
    pub fill_opacity: f64,
    pub icon_radius: u32,
    pub reflection: bool,
    pub reflection_strength: f64,
    pub magnify: bool,
    pub magnify_peak: f64,
    pub magnify_radius_icons: f64,
    pub profile: String,
    pub reduced_motion: bool,
    pub tooltips: bool,
    pub tooltip_delay_ms: u32,
    pub show_running_dots: bool,
    pub show_badges: bool,
    pub autohide: bool,
    pub show_delay_ms: u32,
    pub hide_delay_ms: u32,
    pub yield_fullscreen: bool,
    pub show_launchpad_entry: bool,
    pub show_monitor_entry: bool,
    pub show_shelf_entry: bool,
}

impl SettingsView {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            icon_size: settings.icon_size,
            icon_gap: settings.icon_gap,
            padding_h: settings.padding_h,
            padding_v: settings.padding_v,
            pill_radius: settings.pill_radius,
            tint: settings.tint.clone(),
            fill_opacity: settings.fill_opacity,
            icon_radius: settings.icon_radius,
            reflection: settings.reflection,
            reflection_strength: settings.reflection_strength,
            magnify: settings.magnify,
            magnify_peak: settings.magnify_peak,
            magnify_radius_icons: settings.magnify_radius_icons,
            profile: settings.profile.clone(),
            reduced_motion: settings.reduced_motion,
            tooltips: settings.tooltips,
            tooltip_delay_ms: settings.tooltip_delay_ms,
            show_running_dots: settings.show_running_dots,
            show_badges: settings.show_badges,
            autohide: settings.autohide,
            show_delay_ms: settings.show_delay_ms,
            hide_delay_ms: settings.hide_delay_ms,
            yield_fullscreen: settings.yield_fullscreen,
            show_launchpad_entry: settings.show_launchpad_entry,
            show_monitor_entry: settings.show_monitor_entry,
            show_shelf_entry: settings.show_shelf_entry,
        }
    }

    /// Build a persistable `Settings`, preserving the backend-owned pins and
    /// forcing the current schema. Values are sanitised, not trusted.
    pub fn into_settings(self, pinned: Vec<String>) -> Settings {
        let mut settings = Settings {
            schema_version: CURRENT_SCHEMA,
            icon_size: self.icon_size,
            icon_gap: self.icon_gap,
            padding_h: self.padding_h,
            padding_v: self.padding_v,
            pill_radius: self.pill_radius,
            tint: self.tint,
            fill_opacity: self.fill_opacity,
            icon_radius: self.icon_radius,
            reflection: self.reflection,
            reflection_strength: self.reflection_strength,
            magnify: self.magnify,
            magnify_peak: self.magnify_peak,
            magnify_radius_icons: self.magnify_radius_icons,
            profile: self.profile,
            reduced_motion: self.reduced_motion,
            tooltips: self.tooltips,
            tooltip_delay_ms: self.tooltip_delay_ms,
            show_running_dots: self.show_running_dots,
            show_badges: self.show_badges,
            autohide: self.autohide,
            show_delay_ms: self.show_delay_ms,
            hide_delay_ms: self.hide_delay_ms,
            yield_fullscreen: self.yield_fullscreen,
            show_launchpad_entry: self.show_launchpad_entry,
            show_monitor_entry: self.show_monitor_entry,
            show_shelf_entry: self.show_shelf_entry,
            pinned,
        };
        settings.sanitize();
        settings
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
    let mut parsed: Settings =
        serde_json::from_slice(&bytes).map_err(|e| SettingsError::Corrupt {
            detail: e.to_string(),
        })?;
    // Every older schema's fields either still exist or have a documented
    // default, so an old file upgrades in memory and is written back as the
    // current schema on the next save.
    if !(1..=CURRENT_SCHEMA).contains(&parsed.schema_version) {
        return Err(SettingsError::UnsupportedSchema(parsed.schema_version));
    }
    parsed.sanitize();
    Ok(parsed)
}

/// Write `settings` to `path` via temp file, backup rotation, then rename.
pub fn save_atomic(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = temp_path(path);
    // A caller that did not come through the command layer still cannot write
    // an out-of-range file.
    let mut settings = settings.clone();
    settings.sanitize();
    let bytes = serde_json::to_vec_pretty(&settings).expect("settings serialize");

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
