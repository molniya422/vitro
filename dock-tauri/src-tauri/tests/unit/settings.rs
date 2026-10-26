use dock_tauri_lib::settings::{
    load_or_recover, save_atomic, Settings, SettingsError, CURRENT_SCHEMA,
};
use std::path::Path;

fn sample() -> Settings {
    Settings {
        schema_version: CURRENT_SCHEMA,
        icon_size: 48,
        magnify: true,
        autohide: false,
        reduced_motion: false,
        profile: "vitro".into(),
        show_monitor: false,
        pinned: vec!["shell:explorer".into()],
    }
}

#[test]
fn round_trips_through_a_real_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();
    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded, sample());
}

#[test]
fn second_save_creates_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();
    let mut second = sample();
    second.icon_size = 64;
    save_atomic(&path, &second).unwrap();

    assert!(dir.path().join("settings.json.bak").exists());
    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded.icon_size, 64);
}

#[test]
fn interrupted_write_restores_the_last_valid_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();

    let mut newer = sample();
    newer.icon_size = 96;
    save_atomic(&path, &newer).unwrap();

    // Simulate a torn write: main file is garbage, backup is valid.
    std::fs::write(&path, b"{ broken").unwrap();
    let recovered = load_or_recover(&path).unwrap();
    assert_eq!(recovered.icon_size, 48);
}

#[test]
fn both_corrupt_returns_a_typed_error_not_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, b"{ broken").unwrap();
    std::fs::write(dir.path().join("settings.json.bak"), b"also broken").unwrap();

    let err = load_or_recover(&path).unwrap_err();
    assert!(matches!(err, SettingsError::Corrupt { .. }));
}

#[test]
fn missing_file_yields_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded.schema_version, CURRENT_SCHEMA);
    assert_eq!(loaded.icon_size, 48);
}

#[test]
fn unsupported_schema_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut future = sample();
    future.schema_version = 999;
    std::fs::write(&path, serde_json::to_vec(&future).unwrap()).unwrap();

    let err = load_or_recover(&path).unwrap_err();
    assert!(matches!(err, SettingsError::UnsupportedSchema(999)));
}

#[test]
fn save_leaves_no_temp_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty());
    assert!(Path::new(&path).exists());
}
