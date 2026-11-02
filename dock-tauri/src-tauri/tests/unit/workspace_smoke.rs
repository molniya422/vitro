#[test]
fn workspace_exposes_product_name() {
    assert_eq!(dock_tauri_lib::product_name(), "Dock Tauri");
}
