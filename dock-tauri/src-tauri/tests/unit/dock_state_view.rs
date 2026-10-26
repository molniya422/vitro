use dock_tauri_lib::apps::{AppEntry, RunningInfo};
use dock_tauri_lib::state::{build_dock_state, DockConfig};

fn config() -> DockConfig {
    DockConfig {
        autohide: false,
        magnify: true,
        reduced_motion: false,
        profile: "vitro".into(),
    }
}

#[test]
fn running_and_minimized_flow_into_the_view() {
    let apps = vec![
        AppEntry {
            id: "c:/x/notepad.lnk".into(),
            display_name: "Notepad".into(),
            target: Some("C:/Windows/notepad.exe".into()),
        },
        AppEntry {
            id: "c:/x/paint.lnk".into(),
            display_name: "Paint".into(),
            target: Some("C:/Windows/mspaint.exe".into()),
        },
    ];
    let running = vec![
        RunningInfo {
            running: true,
            minimized: 2,
        },
        RunningInfo {
            running: false,
            minimized: 0,
        },
    ];

    let state = build_dock_state(&apps, &running, &[], &config());
    assert_eq!(state.items.len(), 2);
    assert!(state.items[0].running);
    assert_eq!(state.items[0].minimized, 2);
    assert!(!state.items[1].running);
}

#[test]
fn missing_icon_produces_a_null_not_a_placeholder_path() {
    let apps = vec![AppEntry {
        id: "c:/x/notepad.lnk".into(),
        display_name: "Notepad".into(),
        target: Some("C:/Windows/notepad.exe".into()),
    }];
    let running = vec![RunningInfo {
        running: false,
        minimized: 0,
    }];
    let state = build_dock_state(&apps, &running, &[], &config());
    assert!(state.items[0].icon.is_none());
    assert!(state.items[0].disabled_reason.is_none());
}

#[test]
fn item_with_no_target_is_marked_unavailable() {
    let apps = vec![AppEntry {
        id: "c:/x/broken.lnk".into(),
        display_name: "Broken".into(),
        target: None,
    }];
    let running = vec![RunningInfo {
        running: false,
        minimized: 0,
    }];
    let state = build_dock_state(&apps, &running, &[], &config());
    assert!(state.items[0].disabled_reason.is_some());
}

#[test]
fn fewer_running_entries_than_apps_does_not_panic() {
    let apps = vec![
        AppEntry {
            id: "a".into(),
            display_name: "A".into(),
            target: None,
        },
        AppEntry {
            id: "b".into(),
            display_name: "B".into(),
            target: None,
        },
    ];
    let running = vec![RunningInfo {
        running: true,
        minimized: 0,
    }];
    let state = build_dock_state(&apps, &running, &[], &config());
    assert_eq!(state.items.len(), 2);
    assert!(state.items[1].disabled_reason.is_some());
}
