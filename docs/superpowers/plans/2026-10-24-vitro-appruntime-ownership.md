# Vitro AppRuntime Ownership Implementation Plan

> **For agentic workers:** Follow this plan task by task. Keep Dock stopped; do not run `vitro.exe` for verification.

**Goal:** Move the current Win32 `State` into a UI-thread-owned `AppRuntime`, remove `static mut STATE` and the unsafe `s()` accessor, and preserve existing input/render behavior.

**Architecture:** `main.rs` owns a stable `Box<AppRuntime>` for the life of the message loop and attaches its pointer to the Dock and tooltip HWNDs through `GWLP_USERDATA`. Window procedures translate messages and pass the raw runtime context to short-lived handlers; the low-level hook uses a small atomic route snapshot plus a pure, tested pointer-gating policy because the hook callback has no HWND userdata. The existing D2D/COM-rich `State` stays intact in this milestone; domain-state and renderer extraction are later milestones.

**Tech Stack:** Rust 2021, `windows` 0.58, `cargo test`, `cargo build --release`.

---

## File map

- `vitro/src/core/input.rs`: pure mouse-neighborhood, bottom-edge, throttle, and leave-transition policy.
- `vitro/src/core/mod.rs`: export the pure input policy.
- `vitro/src/runtime.rs`: UI-thread runtime owner and HWND userdata lookup.
- `vitro/src/main.rs`: assemble `State`, allocate the stable runtime box, bind both HWNDs, and keep the box alive through the message loop.
- `vitro/src/state.rs`: retain the existing UI state shape but remove `static mut STATE` and `s()`.
- `vitro/src/dockwin.rs`: route window messages to runtime-aware helpers; keep references out of nested menu and destruction calls.
- `vitro/src/input.rs`: publish/read atomic hook routing values; move auto-hide and animation decisions to the Dock window message handler.
- `vitro/src/tray.rs`: return a menu command from the nested popup loop, then apply it after the loop without a live mutable runtime borrow.

### Task 1: Lock mouse routing behavior in pure tests

**Files:**
- Create: `vitro/src/core/input.rs`
- Modify: `vitro/src/core/mod.rs`

- [x] **Step 1: Add failing tests for neighborhood entry, throttle, and leave notification**

```rust
#[test]
fn posts_nearby_samples_at_most_every_eight_ms_and_posts_the_leave_edge() {
    let snapshot = PointerSnapshot {
        pill: ScreenRect { left: 400, top: 800, right: 600, bottom: 900 },
        screen_height: 1000,
        autohide: false,
    };
    let (state, entered) = route_pointer(snapshot, PointerRouteState::default(), 400, 800, 100);
    assert!(entered.post);
    assert!(entered.near);
    let (state, throttled) = route_pointer(snapshot, state, 401, 801, 103);
    assert!(!throttled.post);
    let (state, resumed) = route_pointer(snapshot, state, 402, 802, 108);
    assert!(resumed.post);
    let (_, left) = route_pointer(snapshot, state, 100, 100, 109);
    assert!(left.post);
    assert!(!left.near);
}
```

- [x] **Step 2: Run the new test and confirm the expected missing-symbol failure**

Run: `cargo test --manifest-path D:\dockyh\vitro\Cargo.toml posts_nearby_samples_at_most_every_eight_ms_and_posts_the_leave_edge`

Expected: FAIL because `core::input` and its routing types do not exist yet.

- [x] **Step 3: Add the pure route types and function**

Define copyable `ScreenRect`, `PointerSnapshot`, `PointerRouteState { was_near, last_post_tick: Option<u32> }`, and `PointerDecision { near, post }`. The pure function computes the existing ±200 px rectangle and optional 3 px bottom activation zone, uses `wrapping_sub` for the 8 ms throttle, emits one final post when leaving, and returns the next route state. It must not import `windows` or perform side effects.

- [x] **Step 4: Verify the tests**

Run: `cargo test --manifest-path D:\dockyh\vitro\Cargo.toml core::input`

Expected: PASS for entry, throttle, leave-edge, bottom-edge activation, and far-away silence.

### Task 2: Add the runtime owner and bind window userdata

**Files:**
- Create: `vitro/src/runtime.rs`
- Modify: `vitro/src/main.rs`

- [x] **Step 1: Add a runtime owner that holds the legacy state**

```rust
pub(crate) struct AppRuntime {
    pub(crate) state: crate::state::State,
    shutdown_started: bool,
}
```

Provide a constructor and an idempotent shutdown gate. Do not put D2D objects behind `Arc<Mutex<_>>`; all runtime use stays on the UI thread.

- [x] **Step 2: Build, bind, and retain the runtime**

After assembling the existing `State`, move it into `Box<AppRuntime>`. Store the stable pointer in `GWLP_USERDATA` for both Dock and tooltip windows before showing either window or entering `GetMessageW`. Keep the box in `main` until the loop exits. Remove direct initialization of `STATE` and later `s()` calls from `main.rs`.

- [x] **Step 3: Verify pointer lookup rejects unbound windows**

Test the pure shutdown gate for `begin_shutdown()` returning true once and false thereafter. Window pointer lookup must return `None` for null userdata rather than dereferencing it.

### Task 3: Route UI callbacks through the owned runtime

**Files:**
- Modify: `vitro/src/state.rs`
- Modify: `vitro/src/dockwin.rs`
- Modify: `vitro/src/input.rs`
- Modify: `vitro/src/tray.rs`

- [x] **Step 1: Convert state-dependent helpers to take a runtime context**

Replace global `s()` lookups with explicit state/runtime parameters. Pass the context down `wndproc -> handler -> render/save` and through nested draw helpers so `render_frame` and `draw_tile_body` do not reacquire mutable global state.

- [x] **Step 2: Keep nested Win32 loops outside mutable state access**

Before `TrackPopupMenu`, copy the HWND, coordinates, and item-hit result; after it returns, reacquire the runtime to apply the chosen command. Before shutdown calls `DestroyWindow`, set the idempotence gate and copy handles, then perform synchronous Win32 destruction without holding `&mut AppRuntime`.

- [x] **Step 3: Route the low-level hook through the tested policy**

Keep only atomic HWND/geometry/autohide routing values in the hook adapter. The hook calls `route_pointer`, stores its returned route state, and posts only when requested. The Dock WndProc receives the latest coordinates and handles slide, hide timers, cursor state, and rendering through the runtime.

- [x] **Step 4: Remove the old global state entry points**

Delete `static mut STATE` and `s()` from `state.rs`; ensure `rg -n 'static mut STATE|\bs\(\)' vitro/src` returns no matches.

### Task 4: Verify the ownership milestone without starting Dock

**Files:** no source changes unless verification exposes a defect.

- [x] **Step 1: Run all Rust tests**

Run: `cargo test --manifest-path D:\dockyh\vitro\Cargo.toml`

Expected: all unit tests pass.

- [x] **Step 2: Build the Windows release executable**

Run: `cargo build --release --manifest-path D:\dockyh\vitro\Cargo.toml`

Expected: success; warnings are recorded for later cleanup, not hidden by widening lint allowances.

- [x] **Step 3: Check the patch and process state**

Run: `git diff --check`; confirm `vitro.exe` is not running and do not launch it. Confirm core modules have no `windows` dependency and the existing uncommitted Sonoma appearance changes remain present.

---

## Self-review and later milestones

This milestone removes only the Dock's `STATE` global. The launcher `LP`, plugin state/text, and lazy DirectWrite format statics remain explicit follow-up work, as do pure `AppState`/reducer integration, persistence ports, shell/process monitoring, and D2D `RenderSnapshot` extraction. The visual prototypes are references: use macOS parity for material/geometric relationships and NeXuS for interaction coverage, while following `docs/research/00-SUMMARY.md` for native API choices, frozen geometry, reduced motion, and idle-resource constraints.
