use dock_tauri_lib::apps::{AppEntry, RunningInfo, SYSTEM_EXPLORER_ID, SYSTEM_SETTINGS_ID};
use dock_tauri_lib::state::{
    build_catalog, build_dock_state, seed_pins, DockConfig, DOCK_ITEM_CAP,
};

fn config() -> DockConfig {
    DockConfig {
        autohide: false,
        magnify: true,
        reduced_motion: false,
        profile: "vitro".into(),
    }
}

fn app(id: &str, name: &str) -> AppEntry {
    AppEntry {
        id: id.into(),
        display_name: name.into(),
        target: Some(format!("C:/Windows/{name}.exe")),
    }
}

fn ids(apps: &[AppEntry]) -> Vec<String> {
    apps.iter().map(|app| app.id.clone()).collect()
}

#[test]
fn running_and_minimized_flow_into_the_view() {
    let apps = vec![
        app("c:/x/notepad.lnk", "Notepad"),
        app("c:/x/paint.lnk", "Paint"),
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

    let state = build_dock_state(&apps, &ids(&apps), &running, &[], &config());
    assert_eq!(state.items.len(), 2);
    assert!(state.items[0].running);
    assert_eq!(state.items[0].minimized, 2);
    assert!(!state.items[1].running);
}

#[test]
fn missing_icon_produces_a_null_not_a_placeholder_path() {
    let apps = vec![app("c:/x/notepad.lnk", "Notepad")];
    let running = vec![RunningInfo {
        running: false,
        minimized: 0,
    }];
    let state = build_dock_state(&apps, &ids(&apps), &running, &[], &config());
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
    let state = build_dock_state(&apps, &ids(&apps), &running, &[], &config());
    assert!(state.items[0].disabled_reason.is_some());
}

#[test]
fn fewer_running_entries_than_apps_does_not_panic() {
    let apps = vec![app("a", "A"), app("b", "B")];
    let running = vec![RunningInfo {
        running: true,
        minimized: 0,
    }];
    let state = build_dock_state(&apps, &ids(&apps), &running, &[], &config());
    assert_eq!(state.items.len(), 2);
    assert!(state.items[1].disabled_reason.is_none());
}

#[test]
fn only_pinned_ids_reach_the_dock() {
    let apps = vec![
        app("c:/x/a.lnk", "Alpha"),
        app("c:/x/b.lnk", "Beta"),
        app("c:/x/c.lnk", "Gamma"),
    ];
    let running = vec![
        RunningInfo {
            running: false,
            minimized: 0,
        };
        3
    ];
    let pinned = vec!["c:/x/c.lnk".to_string(), "c:/x/a.lnk".to_string()];

    let state = build_dock_state(&apps, &pinned, &running, &[], &config());
    let names: Vec<_> = state.items.iter().map(|item| item.name.as_str()).collect();
    assert_eq!(names, ["Gamma", "Alpha"]);
}

#[test]
fn dock_never_shows_more_than_the_cap_even_if_pinned_is_longer() {
    let apps: Vec<_> = (0..12)
        .map(|index| app(&format!("c:/x/{index}.lnk"), &format!("App {index:02}")))
        .collect();
    let running = vec![
        RunningInfo {
            running: false,
            minimized: 0,
        };
        12
    ];
    let pinned = ids(&apps);
    let state = build_dock_state(&apps, &pinned, &running, &[], &config());
    assert_eq!(state.items.len(), DOCK_ITEM_CAP);
}

#[test]
fn first_run_seed_is_system_tiles_plus_eight_catalog_apps() {
    let apps = vec![
        app("c:/x/zebra.lnk", "Zebra"),
        app("c:/x/alpha.lnk", "Alpha"),
        app("c:/x/mike.lnk", "Mike"),
        app("c:/x/bravo.lnk", "Bravo"),
        app("c:/x/yankee.lnk", "Yankee"),
        app("c:/x/charlie.lnk", "Charlie"),
        app("c:/x/xray.lnk", "Xray"),
        app("c:/x/delta.lnk", "Delta"),
        app("c:/x/echo.lnk", "Echo"),
        app("c:/x/foxtrot.lnk", "Foxtrot"),
    ];

    let pinned = seed_pins(&apps);
    assert_eq!(pinned.len(), DOCK_ITEM_CAP);
    assert_eq!(pinned[0], SYSTEM_EXPLORER_ID);
    assert_eq!(pinned[1], SYSTEM_SETTINGS_ID);
    assert_eq!(
        &pinned[2..],
        [
            "c:/x/alpha.lnk",
            "c:/x/bravo.lnk",
            "c:/x/charlie.lnk",
            "c:/x/delta.lnk",
            "c:/x/echo.lnk",
            "c:/x/foxtrot.lnk",
            "c:/x/mike.lnk",
            "c:/x/xray.lnk",
        ]
    );

    let state = build_dock_state(&apps, &pinned, &[], &[], &config());
    assert_eq!(state.items.len(), DOCK_ITEM_CAP);
    assert_eq!(state.items[0].kind, "system");
    assert_eq!(state.items[0].name, "Explorer");
    assert_eq!(state.items[1].name, "Settings");
}

#[test]
fn seeding_a_short_catalog_keeps_the_system_tiles() {
    let apps = vec![app("c:/x/a.lnk", "Alpha")];
    let pinned = seed_pins(&apps);
    assert_eq!(pinned.len(), 3);
    assert_eq!(pinned[0], SYSTEM_EXPLORER_ID);
    assert_eq!(pinned[1], SYSTEM_SETTINGS_ID);
    assert_eq!(pinned[2], "c:/x/a.lnk");
}

#[test]
fn pinned_app_that_left_the_catalog_is_disabled_not_dropped() {
    let pinned = vec!["c:/gone/removed.lnk".to_string()];
    let state = build_dock_state(&[], &pinned, &[], &[], &config());
    assert_eq!(state.items.len(), 1);
    assert_eq!(state.items[0].name, "removed");
    assert!(state.items[0].disabled_reason.is_some());
}

#[test]
fn catalog_returns_every_app_without_the_dock_cap() {
    let apps: Vec<_> = (0..12)
        .map(|index| app(&format!("c:/x/{index}.lnk"), &format!("App {index:02}")))
        .collect();
    let running = vec![
        RunningInfo {
            running: false,
            minimized: 0,
        };
        12
    ];
    let catalog = build_catalog(&apps, &running, &[]);
    assert_eq!(catalog.len(), 12);
}
