use dock_tauri_lib::apps::{catalog_from_dirs, reconcile, AppEntry, WindowRef};

fn write_lnk(dir: &std::path::Path, name: &str) {
    std::fs::write(dir.join(format!("{name}.lnk")), b"L\0\0\0fake").unwrap();
}

#[test]
fn scans_lnk_files_into_entries() {
    let dir = tempfile::tempdir().unwrap();
    write_lnk(dir.path(), "Notepad");
    write_lnk(dir.path(), "Paint");

    let entries = catalog_from_dirs(&[dir.path().to_path_buf()]);
    assert_eq!(entries.len(), 2);
    let names: Vec<_> = entries.iter().map(|e| e.display_name.clone()).collect();
    assert!(names.contains(&"Notepad".to_string()));
    assert!(names.contains(&"Paint".to_string()));
}

#[test]
fn identity_is_the_absolute_shortcut_path() {
    let dir = tempfile::tempdir().unwrap();
    write_lnk(dir.path(), "Notepad");
    let entries = catalog_from_dirs(&[dir.path().to_path_buf()]);
    assert!(entries[0].id.ends_with("notepad.lnk"));
    assert!(!entries[0].id.is_empty());
}

#[test]
fn entries_are_sorted_by_display_name_case_insensitively() {
    let dir = tempfile::tempdir().unwrap();
    write_lnk(dir.path(), "Zebra");
    write_lnk(dir.path(), "alpha");
    let entries = catalog_from_dirs(&[dir.path().to_path_buf()]);
    assert_eq!(entries[0].display_name, "alpha");
    assert_eq!(entries[1].display_name, "Zebra");
}

#[test]
fn duplicates_across_directories_are_collapsed() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    write_lnk(a.path(), "Notepad");
    write_lnk(b.path(), "Notepad");
    let entries = catalog_from_dirs(&[a.path().to_path_buf(), b.path().to_path_buf()]);
    assert_eq!(entries.len(), 1);
}

#[test]
fn missing_directory_is_not_an_error() {
    let entries = catalog_from_dirs(&[std::path::PathBuf::from("Z:/does/not/exist")]);
    assert!(entries.is_empty());
}

#[test]
fn reconciliation_matches_windows_to_apps_by_process_path() {
    let entries = vec![AppEntry {
        id: "c:/x/notepad.lnk".into(),
        display_name: "Notepad".into(),
        target: Some("C:/Windows/notepad.exe".into()),
    }];
    let windows = vec![WindowRef {
        process_path: "C:/Windows/notepad.exe".into(),
        minimized: false,
    }];
    let running = reconcile(&entries, &windows);
    assert!(running[0].running);
    assert_eq!(running[0].minimized, 0);
}

#[test]
fn reconciliation_counts_minimized_windows_case_insensitively() {
    let entries = vec![AppEntry {
        id: "c:/x/notepad.lnk".into(),
        display_name: "Notepad".into(),
        target: Some("C:/Windows/notepad.exe".into()),
    }];
    let windows = vec![
        WindowRef {
            process_path: "C:/Windows/notepad.exe".into(),
            minimized: true,
        },
        WindowRef {
            process_path: "c:/windows/NOTEPAD.exe".into(),
            minimized: true,
        },
    ];
    let running = reconcile(&entries, &windows);
    assert!(running[0].running);
    assert_eq!(running[0].minimized, 2);
}

#[test]
fn app_without_a_target_is_never_running() {
    let entries = vec![AppEntry {
        id: "c:/x/uwp.lnk".into(),
        display_name: "Uwp".into(),
        target: None,
    }];
    let windows = vec![WindowRef {
        process_path: "C:/Windows/notepad.exe".into(),
        minimized: false,
    }];
    let running = reconcile(&entries, &windows);
    assert!(!running[0].running);
}
