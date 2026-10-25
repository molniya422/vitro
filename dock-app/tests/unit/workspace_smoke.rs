#[test]
fn workspace_exposes_product_modules() {
    assert_eq!(dock_app::product_name(), "Dock App");
}

// Compile-time contract: Tasks 2-12 fill these roots in.
#[allow(unused_imports)]
mod module_tree_contract {
    use dock_app::{core, docklets, platform::windows, runtime, ui};
}
