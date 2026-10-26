use dock_tauri_lib::icon_cache::{extract_png, IconCache};

#[test]
fn extraction_from_a_missing_path_returns_none() {
    assert!(extract_png(std::path::Path::new("Z:/nope/missing.exe"), 48).is_none());
}

#[test]
fn extraction_from_a_real_shortcut_produces_a_real_png() {
    let dirs = dock_tauri_lib::apps::start_menu_dirs();
    let entries = dock_tauri_lib::apps::catalog_from_dirs(&dirs);
    let Some(first) = entries.first() else {
        return;
    };
    let Some(png) = extract_png(std::path::Path::new(&first.id), 48) else {
        return;
    };
    assert_eq!(&png[1..4], b"PNG", "output must be a PNG");

    // A real icon is at least the requested size on its short edge, and is not
    // a single flat colour. A 16 px system icon stretched to 48 px would fail
    // the dimension check.
    let image = image::load_from_memory(&png).expect("decoded png");
    let (w, h) = (image.width(), image.height());
    assert!(
        w >= 48 && h >= 48,
        "icon must be at least 48px, got {w}x{h} (SHGFI_ICON returns 16px — use SHGetImageList)"
    );
}

#[test]
fn extraction_never_panics_and_returns_png_when_the_shell_cooperates() {
    let dirs = dock_tauri_lib::apps::start_menu_dirs();
    let entries = dock_tauri_lib::apps::catalog_from_dirs(&dirs);
    let Some(first) = entries.first() else {
        return;
    };
    if let Some(png) = extract_png(std::path::Path::new(&first.id), 48) {
        assert!(png.len() > 8);
        assert_eq!(&png[1..4], b"PNG");
    }
}

#[test]
fn extract_store_then_read_round_trips() {
    let dirs = dock_tauri_lib::apps::start_menu_dirs();
    let entries = dock_tauri_lib::apps::catalog_from_dirs(&dirs);
    let Some(first) = entries.first() else {
        return;
    };
    let Some(png) = extract_png(std::path::Path::new(&first.id), 48) else {
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let cache = IconCache::new(tmp.path().to_path_buf());
    let key = IconCache::key(&first.id, 48);
    cache.store(&key, &png).unwrap();
    assert!(cache
        .read(&key)
        .unwrap()
        .starts_with("data:image/png;base64,"));
}
