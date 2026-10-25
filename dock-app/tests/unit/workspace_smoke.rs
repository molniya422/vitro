#[test]
fn workspace_exposes_product_modules() {
    assert_eq!(dock_app::product_name(), "Dock App");
}
