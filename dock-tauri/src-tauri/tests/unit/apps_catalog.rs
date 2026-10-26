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

#[test]
fn different_apps_sharing_a_display_name_are_both_kept() {
    let a = tempfile::tempdir().unwrap();
    write_lnk(a.path(), "Settings");

    // Build the first entry by hand so it has a distinct target: a fake `.lnk`
    // resolves to no target, and two target-less entries would collapse by name.
    let mut scanned = dock_tauri_lib::apps::catalog_from_dirs(&[a.path().to_path_buf()]);
    scanned[0].target = Some("C:/Windows/System32/control.exe".into());
    let other = AppEntry {
        id: "c:/other/settings.lnk".into(),
        display_name: "Settings".into(),
        target: Some("C:/Other/settings.exe".into()),
    };

    let merged = dock_tauri_lib::apps::dedup_entries(vec![scanned[0].clone(), other]);
    assert_eq!(merged.len(), 2, "distinct targets must not collapse");
}

#[test]
fn the_same_target_from_two_roots_collapses() {
    let shared = "C:/Windows/notepad.exe";
    let first = AppEntry {
        id: "c:/programdata/notepad.lnk".into(),
        display_name: "Notepad".into(),
        target: Some(shared.into()),
    };
    let second = AppEntry {
        id: "c:/appdata/notepad.lnk".into(),
        display_name: "Notepad".into(),
        target: Some(shared.into()),
    };
    let merged = dock_tauri_lib::apps::dedup_entries(vec![first, second]);
    assert_eq!(merged.len(), 1);
    assert_eq!(
        merged[0].id, "c:/programdata/notepad.lnk",
        "first scanned wins"
    );
}

#[test]
fn running_window_in_a_different_directory_matches_by_file_name() {
    // The Doubao shape: shortcut target is D:\doubao\Doubao.exe, but the app
    // actually runs its window from D:\doubao\app\Doubao.exe.
    let entries = vec![AppEntry {
        id: "c:/programdata/doubao.lnk".into(),
        display_name: "Doubao".into(),
        target: Some("D:/doubao/Doubao.exe".into()),
    }];
    let windows = vec![WindowRef {
        process_path: "D:/doubao/app/Doubao.exe".into(),
        minimized: false,
    }];
    let running = reconcile(&entries, &windows);
    assert!(running[0].running, "file-name fallback should match");
}

#[test]
fn exact_path_wins_over_a_same_named_binary_elsewhere() {
    let entries = vec![AppEntry {
        id: "c:/a/editor.lnk".into(),
        display_name: "Editor".into(),
        target: Some("C:/A/editor.exe".into()),
    }];
    // A different program that happens to share the file name.
    let windows = vec![
        WindowRef {
            process_path: "D:/B/editor.exe".into(),
            minimized: true,
        },
        WindowRef {
            process_path: "C:/A/editor.exe".into(),
            minimized: false,
        },
    ];
    let running = reconcile(&entries, &windows);
    assert!(running[0].running);
    assert_eq!(
        running[0].minimized, 0,
        "only the exact-path window belongs to this app"
    );
}

#[test]
fn a_different_file_name_never_matches() {
    let entries = vec![AppEntry {
        id: "c:/a/notes.lnk".into(),
        display_name: "Notes".into(),
        target: Some("C:/A/notes.exe".into()),
    }];
    let windows = vec![WindowRef {
        process_path: "C:/A/other.exe".into(),
        minimized: false,
    }];
    assert!(!reconcile(&entries, &windows)[0].running);
}

#[test]
fn file_name_fallback_still_counts_minimized_windows() {
    let entries = vec![AppEntry {
        id: "c:/a/browser.lnk".into(),
        display_name: "Browser".into(),
        target: Some("C:/A/browser.exe".into()),
    }];
    let windows = vec![
        WindowRef {
            process_path: "C:/A/app/browser.exe".into(),
            minimized: true,
        },
        WindowRef {
            process_path: "C:/A/app/browser.exe".into(),
            minimized: true,
        },
    ];
    let running = reconcile(&entries, &windows);
    assert!(running[0].running);
    assert_eq!(running[0].minimized, 2);
}
