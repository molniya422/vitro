use dock_tauri_lib::settings::{Settings, SettingsView, CURRENT_SCHEMA};

#[test]
fn settings_view_round_trips_every_field() {
    let original = Settings::default();
    let view = SettingsView::from_settings(&original);
    let restored = view.into_settings(vec!["shell:explorer".into()]);

    let mut expected = original;
    expected.pinned = vec!["shell:explorer".into()];
    assert_eq!(restored, expected);
}

#[test]
fn into_settings_preserves_backend_owned_pins() {
    let mut settings = Settings::default();
    settings.pinned = vec!["real-pin".into()];
    let view = SettingsView::from_settings(&settings);

    let restored = view.into_settings(vec!["backend-pin".into()]);
    assert_eq!(restored.pinned, ["backend-pin"]);
}

#[test]
fn into_settings_forces_the_current_schema() {
    let mut old = Settings::default();
    old.schema_version = 1;
    let view = SettingsView::from_settings(&old);

    let restored = view.into_settings(Vec::new());
    assert_eq!(restored.schema_version, CURRENT_SCHEMA);
}

#[test]
fn into_settings_sanitizes_untrusted_values() {
    let mut view = SettingsView::from_settings(&Settings::default());
    view.icon_size = 999;
    view.fill_opacity = 9.0;
    view.tint = "not-a-colour".into();

    let restored = view.into_settings(Vec::new());
    assert_eq!(restored.icon_size, 80);
    assert_eq!(restored.fill_opacity, 1.0);
    assert_eq!(restored.tint, "#161a26");
}

#[test]
fn settings_view_is_camel_case_and_omits_backend_owned_fields() {
    let view = SettingsView::from_settings(&Settings::default());
    let json = serde_json::to_value(&view).unwrap();

    assert_eq!(json["iconSize"], 48);
    assert_eq!(json["fillOpacity"], 0.62);
    assert_eq!(json["showMonitorEntry"], true);
    assert!(json.get("icon_size").is_none());
    assert!(json.get("pinned").is_none());
    assert!(json.get("schemaVersion").is_none());

    let parsed: SettingsView = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, view);
}
