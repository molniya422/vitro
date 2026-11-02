use std::ptr::NonNull;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWLP_USERDATA,
};

use crate::state::State;
use crate::core::model::BehaviorState;

pub(crate) fn cleanup_after_startup_error<T, E>(
    result: std::result::Result<T, E>,
    cleanup: impl FnOnce(),
) -> std::result::Result<T, E> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            cleanup();
            Err(error)
        }
    }
}

#[derive(Default)]
pub(crate) struct ShutdownGate {
    started: bool,
}

impl ShutdownGate {
    pub(crate) fn allows_commands(&self) -> bool {
        !self.started
    }

    pub(crate) fn begin_shutdown(&mut self) -> bool {
        if self.started {
            false
        } else {
            self.started = true;
            true
        }
    }
}

pub(crate) struct AppRuntime {
    pub(crate) state: State,
    pub(crate) behavior: BehaviorState,
    pub(crate) launcher: Option<crate::lpad::LpadState>,
    pub(crate) dock_drag: Option<DockDrag>,
    pub(crate) subdock_hover: Option<(crate::core::model::TileId, usize)>,
    shutdown_gate: ShutdownGate,
}

pub(crate) struct DockDrag {
    pub(crate) tile_id: crate::core::model::TileId,
    pub(crate) start: (i32, i32),
    pub(crate) moved: bool,
    pub(crate) target: Option<crate::core::model::TileId>,
}

impl AppRuntime {
    pub(crate) fn new(state: State, mut behavior: BehaviorState) -> Self {
        behavior.next_tile_id = state.tiles.iter()
            .map(|tile| tile.id.0)
            .max()
            .map(|last| last.wrapping_add(1))
            .unwrap_or(0);
        Self {
            state,
            behavior,
            launcher: None,
            dock_drag: None,
            subdock_hover: None,
            shutdown_gate: ShutdownGate::default(),
        }
    }

    pub(crate) fn begin_shutdown(&mut self) -> bool {
        self.shutdown_gate.begin_shutdown()
    }

    pub(crate) fn is_shutting_down(&self) -> bool {
        !self.shutdown_gate.allows_commands()
    }
}

pub(crate) fn runtime_pointer_from_userdata(value: isize) -> Option<NonNull<AppRuntime>> {
    NonNull::new(value as *mut AppRuntime)
}

pub(crate) unsafe fn bind_window(hwnd: HWND, runtime: NonNull<AppRuntime>) {
    let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, runtime.as_ptr() as isize);
}

pub(crate) unsafe fn from_window(hwnd: HWND) -> Option<NonNull<AppRuntime>> {
    runtime_pointer_from_userdata(GetWindowLongPtrW(hwnd, GWLP_USERDATA))
}

#[cfg(test)]
mod tests {
    use super::{cleanup_after_startup_error, runtime_pointer_from_userdata, ShutdownGate};

    #[test]
    fn shutdown_gate_allows_only_the_first_shutdown() {
        let mut gate = ShutdownGate::default();
        assert!(gate.begin_shutdown());
        assert!(!gate.begin_shutdown());
        assert!(!gate.begin_shutdown());
    }

    #[test]
    fn shutdown_gate_rejects_commands_after_shutdown_begins() {
        let mut gate = ShutdownGate::default();
        assert!(gate.allows_commands());
        assert!(gate.begin_shutdown());
        assert!(!gate.allows_commands());
    }

    #[test]
    fn failed_startup_runs_cleanup_and_preserves_the_error() {
        let mut cleanup_calls = 0;
        let result = cleanup_after_startup_error::<(), _>(Err("startup failed"), || {
            cleanup_calls += 1;
        });

        assert_eq!(result, Err("startup failed"));
        assert_eq!(cleanup_calls, 1);
    }

    #[test]
    fn successful_startup_does_not_run_cleanup() {
        let mut cleanup_calls = 0;
        let result = cleanup_after_startup_error::<i32, ()>(Ok(42), || {
            cleanup_calls += 1;
        });

        assert_eq!(result, Ok(42));
        assert_eq!(cleanup_calls, 0);
    }

    #[test]
    fn null_window_userdata_does_not_produce_a_runtime_pointer() {
        assert!(runtime_pointer_from_userdata(0).is_none());
    }
}
