use dock_tauri_lib::icon_cache::IconCache;

#[test]
fn cache_key_is_stable_and_size_specific() {
    let a = IconCache::key("C:/x/notepad.lnk", 48);
    let b = IconCache::key("C:/x/notepad.lnk", 48);
    let c = IconCache::key("C:/x/notepad.lnk", 96);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.len(), 64, "sha256 hex digest");
}

#[test]
fn key_is_case_insensitive_for_paths() {
    let a = IconCache::key("C:/X/Notepad.lnk", 48);
    let b = IconCache::key("c:/x/notepad.lnk", 48);
    assert_eq!(a, b);
}

#[test]
fn stores_and_reads_a_data_url() {
    let dir = tempfile::tempdir().unwrap();
    let cache = IconCache::new(dir.path().to_path_buf());
    let key = IconCache::key("C:/x/notepad.lnk", 48);
    let png: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

    cache.store(&key, png).unwrap();
    let url = cache.read(&key).expect("cache hit");
    assert!(url.starts_with("data:image/png;base64,"));
}

#[test]
fn miss_returns_none_without_error() {
    let dir = tempfile::tempdir().unwrap();
    let cache = IconCache::new(dir.path().to_path_buf());
    let key = IconCache::key("C:/x/missing.lnk", 48);
    assert!(cache.read(&key).is_none());
}

#[test]
fn empty_cache_entry_is_treated_as_a_miss() {
    let dir = tempfile::tempdir().unwrap();
    let cache = IconCache::new(dir.path().to_path_buf());
    let key = IconCache::key("C:/x/notepad.lnk", 48);
    std::fs::write(dir.path().join(format!("{key}.png")), b"").unwrap();
    assert!(cache.read(&key).is_none());
}
