//! The serialisable snapshot React renders. Pure: no Win32, no IO.

use crate::apps::{self, AppEntry, RunningInfo};
use serde::Serialize;

/// The dock never shows more app tiles than this. First-run seeding spends the
/// budget on two system tiles plus up to eight catalog apps.
pub const DOCK_ITEM_CAP: usize = 10;

pub struct DockConfig {
    pub autohide: bool,
    pub magnify: bool,
    pub reduced_motion: bool,
    pub profile: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DockItemView {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub icon: Option<String>,
    pub running: bool,
    pub minimized: u32,
    pub disabled_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DockStateView {
    pub items: Vec<DockItemView>,
    pub edge: String,
    pub autohide: bool,
    pub magnify: bool,
    pub reduced_motion: bool,
    pub profile: String,
}

/// First-run pins: Explorer and Settings as system tiles, then catalog apps in
/// display-name order until the dock is full.
pub fn seed_pins(catalog: &[AppEntry]) -> Vec<String> {
    let mut pinned = vec![
        apps::SYSTEM_EXPLORER_ID.to_string(),
        apps::SYSTEM_SETTINGS_ID.to_string(),
    ];

    // Prefer apps with a resolvable executable: those are the ones the icon
    // extractor can render and the launcher can start, so the seeded dock shows
    // real artwork instead of letter placeholders or disabled tiles.
    let mut sorted: Vec<&AppEntry> = catalog.iter().collect();
    sorted.sort_by(|a, b| {
        b.target.is_some().cmp(&a.target.is_some()).then_with(|| {
            a.display_name
                .to_ascii_lowercase()
                .cmp(&b.display_name.to_ascii_lowercase())
        })
    });
    for app in sorted {
        if pinned.len() >= DOCK_ITEM_CAP {
            break;
        }
        if !pinned.contains(&app.id) {
            pinned.push(app.id.clone());
        }
    }
    pinned
}

fn view_for(
    app: &AppEntry,
    info: Option<&RunningInfo>,
    icon: Option<String>,
    kind: &str,
) -> DockItemView {
    DockItemView {
        id: app.id.clone(),
        kind: kind.to_string(),
        name: app.display_name.clone(),
        icon,
        running: info.is_some_and(|i| i.running),
        minimized: info.map_or(0, |i| i.minimized),
        disabled_reason: if app.target.is_none() {
            Some("This shortcut has no resolvable target".to_string())
        } else {
            None
        },
    }
}

fn name_from_id(id: &str) -> String {
    id.rsplit('/')
        .next()
        .unwrap_or(id)
        .trim_end_matches(".lnk")
        .to_string()
}

/// Build the dock view from the pinned ids only. `running` and `icons` are
/// indexed parallel to `catalog`; a missing entry means "no indicator" and
/// "render the fallback glyph" respectively.
pub fn build_dock_state(
    catalog: &[AppEntry],
    pinned: &[String],
    running: &[RunningInfo],
    icons: &[Option<String>],
    config: &DockConfig,
) -> DockStateView {
    let items = pinned
        .iter()
        .take(DOCK_ITEM_CAP)
        .map(|id| {
            if let Some(app) = apps::system_entry(id) {
                return view_for(&app, None, None, "system");
            }
            match catalog.iter().position(|app| &app.id == id) {
                Some(index) => view_for(
                    &catalog[index],
                    running.get(index),
                    icons.get(index).cloned().flatten(),
                    "app",
                ),
                // A pinned app that left the Start Menu stays visible as a
                // disabled tile so the user can remove it.
                None => DockItemView {
                    id: id.clone(),
                    kind: "app".to_string(),
                    name: name_from_id(id),
                    icon: None,
                    running: false,
                    minimized: 0,
                    disabled_reason: Some("This app is no longer installed".to_string()),
                },
            }
        })
        .collect();

    DockStateView {
        items,
        edge: "bottom".to_string(),
        autohide: config.autohide,
        magnify: config.magnify,
        reduced_motion: config.reduced_motion,
        profile: config.profile.clone(),
    }
}

/// Build views for every catalog app. Launchpad receives this list, so it is
/// deliberately not capped by [`DOCK_ITEM_CAP`].
pub fn build_catalog(
    catalog: &[AppEntry],
    running: &[RunningInfo],
    icons: &[Option<String>],
) -> Vec<DockItemView> {
    catalog
        .iter()
        .enumerate()
        .map(|(index, app)| {
            view_for(
                app,
                running.get(index),
                icons.get(index).cloned().flatten(),
                "app",
            )
        })
        .collect()
}
