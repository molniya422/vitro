use super::model::TileId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BehaviorEvent {
    PointerMoved {
        x: i32,
        y: i32,
        near: bool,
        hit: Option<TileId>,
        now_ms: u32,
    },
    DwellElapsed {
        now_ms: u32,
    },
    HideElapsed {
        slide_progress: f32,
    },
    TileRemoved(TileId),
    ShowDock,
    ToggleAutohide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Timer {
    TooltipDwell,
    AutoHide,
    Animation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    SetTimer { timer: Timer, interval_ms: u32 },
    KillTimer(Timer),
    ShowTooltip(TileId),
    HideTooltip,
    SaveAutohide(bool),
    PublishHookSnapshot,
    Render,
}
