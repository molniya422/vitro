//! The serialisable snapshot React renders. Pure: no Win32, no IO.

use crate::apps::{AppEntry, RunningInfo};
use serde::Serialize;

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

/// Build the view. `icons` is indexed parallel to `apps`; a missing entry or a
/// `None` value means "render the fallback glyph".
pub fn build_dock_state(
    apps: &[AppEntry],
    running: &[RunningInfo],
    icons: &[Option<String>],
    config: &DockConfig,
) -> DockStateView {
    let items = apps
        .iter()
        .enumerate()
        .map(|(index, app)| {
            let info = running.get(index);
            let running_now = info.is_some_and(|i| i.running);
            let minimized = info.map_or(0, |i| i.minimized);
            DockItemView {
                id: app.id.clone(),
                kind: "app".to_string(),
                name: app.display_name.clone(),
                icon: icons.get(index).cloned().flatten(),
                running: running_now,
                minimized,
                disabled_reason: if app.target.is_none() {
                    Some("This shortcut has no resolvable target".to_string())
                } else {
                    None
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
