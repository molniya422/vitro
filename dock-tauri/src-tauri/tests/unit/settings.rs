use dock_tauri_lib::settings::{
    load_or_recover, save_atomic, Settings, SettingsError, CURRENT_SCHEMA, DEFAULT_TINT,
};
use std::path::Path;

fn sample() -> Settings {
    Settings {
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
        // `sample` predates the default flip; keeping it false proves that a
        // stored value survives a round trip.
        autohide: false,
        show_delay_ms: 300,
        hide_delay_ms: 500,
        yield_fullscreen: true,
        show_launchpad_entry: true,
        show_monitor_entry: true,
        show_shelf_entry: true,
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
    assert!(loaded.autohide);
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
fn a_newer_schema_is_not_masked_by_a_valid_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // A perfectly good backup that load_or_recover could fall back to.
    save_atomic(&path, &sample()).unwrap();
    let mut newer = sample();
    newer.icon_size = 64;
    save_atomic(&path, &newer).unwrap();
    assert!(dir.path().join("settings.json.bak").exists());

    // Main file is valid JSON but from a future schema version.
    let mut future = sample();
    future.schema_version = 999;
    std::fs::write(&path, serde_json::to_vec(&future).unwrap()).unwrap();

    // A newer schema is not corruption: it must surface, not be silently
    // replaced by the (older) backup.
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

#[test]
fn new_defaults_match_the_settings_table() {
    let d = Settings::default();
    assert_eq!(d.schema_version, CURRENT_SCHEMA);
    assert_eq!(d.icon_size, 48);
    assert_eq!(d.icon_gap, 8);
    assert_eq!(d.padding_h, 16);
    assert_eq!(d.padding_v, 10);
    assert_eq!(d.pill_radius, 34);
    assert_eq!(d.tint, DEFAULT_TINT);
    assert_eq!(d.fill_opacity, 0.62);
    assert_eq!(d.icon_radius, 22);
    assert!(d.reflection);
    assert_eq!(d.reflection_strength, 0.25);
    assert!(d.magnify);
    assert_eq!(d.magnify_peak, 1.6);
    assert_eq!(d.magnify_radius_icons, 2.2);
    assert_eq!(d.profile, "vitro");
    assert!(!d.reduced_motion);
    assert!(d.tooltips);
    assert_eq!(d.tooltip_delay_ms, 500);
    assert!(d.show_running_dots);
    assert!(d.show_badges);
    assert!(d.autohide);
    assert_eq!(d.show_delay_ms, 300);
    assert_eq!(d.hide_delay_ms, 500);
    assert!(d.yield_fullscreen);
    assert!(d.show_launchpad_entry);
    assert!(d.show_monitor_entry);
    assert!(d.show_shelf_entry);
    assert!(d.pinned.is_empty());
}

#[test]
fn v1_file_migrates_with_defaults_for_new_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(
        &path,
        r#"{
            "schema_version": 1,
            "icon_size": 40,
            "magnify": false,
            "autohide": false,
            "reduced_motion": true,
            "profile": "macos",
            "show_monitor": true,
            "pinned": ["shell:explorer"]
        }"#,
    )
    .unwrap();

    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded.schema_version, CURRENT_SCHEMA);
    assert_eq!(loaded.icon_size, 40);
    assert!(!loaded.magnify);
    // An explicit stored choice is never overridden by migration.
    assert!(!loaded.autohide);
    assert!(loaded.reduced_motion);
    assert_eq!(loaded.profile, "macos");
    assert_eq!(loaded.pinned, ["shell:explorer"]);

    assert_eq!(loaded.icon_gap, 8);
    assert_eq!(loaded.fill_opacity, 0.62);
    assert_eq!(loaded.tooltip_delay_ms, 500);
    assert!(loaded.yield_fullscreen);
    // The shelf entry is a schema-3 field, so a v1 file gets it by default.
    assert!(loaded.show_shelf_entry);
    // The unused v1 key is ignored; the dock keeps both route entries.
    assert!(loaded.show_launchpad_entry);
    assert!(loaded.show_monitor_entry);
}

#[test]
fn sanitize_clamps_every_numeric_field() {
    let mut s = sample();
    s.icon_size = 999;
    s.icon_gap = 99;
    s.padding_h = 0;
    s.padding_v = 99;
    s.pill_radius = 999;
    s.fill_opacity = 5.0;
    s.icon_radius = 99;
    s.reflection_strength = -1.0;
    s.magnify_peak = 9.0;
    s.magnify_radius_icons = 0.1;
    s.tooltip_delay_ms = 9_999;
    s.show_delay_ms = 9_999;
    s.hide_delay_ms = 0;
    s.tint = "#ABCDEF".into();
    s.profile = "NOPE".into();
    s.schema_version = 1;

    s.sanitize();

    assert_eq!(s.schema_version, CURRENT_SCHEMA);
    assert_eq!(s.icon_size, 80);
    assert_eq!(s.icon_gap, 24);
    assert_eq!(s.padding_h, 4);
    assert_eq!(s.padding_v, 24);
    assert_eq!(s.pill_radius, 64);
    assert_eq!(s.fill_opacity, 1.0);
    assert_eq!(s.icon_radius, 50);
    assert_eq!(s.reflection_strength, 0.0);
    assert_eq!(s.magnify_peak, 2.0);
    assert_eq!(s.magnify_radius_icons, 1.0);
    assert_eq!(s.tooltip_delay_ms, 2_000);
    assert_eq!(s.show_delay_ms, 1_000);
    assert_eq!(s.hide_delay_ms, 100);
    assert_eq!(s.tint, "#abcdef");
    assert_eq!(s.profile, "vitro");
}

#[test]
fn sanitize_replaces_an_invalid_tint_with_the_default() {
    let mut s = sample();
    s.tint = "blue".into();
    s.sanitize();
    assert_eq!(s.tint, DEFAULT_TINT);
}

#[test]
fn save_upgrades_a_v1_file_to_the_current_schema() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut old = sample();
    old.schema_version = 1;
    save_atomic(&path, &old).unwrap();

    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(raw["schema_version"], CURRENT_SCHEMA);
}

#[test]
fn v2_file_without_the_shelf_field_loads_with_it_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(
        &path,
        r#"{
            "schema_version": 2,
            "icon_size": 44,
            "show_launchpad_entry": true,
            "show_monitor_entry": false,
            "pinned": []
        }"#,
    )
    .unwrap();

    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded.schema_version, CURRENT_SCHEMA);
    assert_eq!(loaded.icon_size, 44);
    assert!(!loaded.show_monitor_entry);
    assert!(loaded.show_shelf_entry);
}
