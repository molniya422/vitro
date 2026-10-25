# Nexus + BitDock Desktop Enhancement Suite Design

**Status:** Draft for user review  
**Date:** 2026-10-25  
**Scope:** A new, independent Windows desktop enhancement suite. The existing Glassboard annotation application remains in the repository but is excluded from the Dock suite release package.

## Product definition

The product is a standalone Windows desktop enhancement suite whose primary surface is a real Dock. It provides a persistent entry point for launching applications, switching and controlling windows, browsing files, and hosting small system extensions. It coexists with the Windows taskbar and may hide or reveal its own surface without replacing or injecting into the Windows shell.

The suite is not a decorative icon strip. Every visible item maps to a real application, window, file, folder, system target, or permissioned Docklet. Visual effects are subordinate to those actions and must degrade cleanly when reduced motion or performance mode is enabled.

## Product boundaries

- `Glassboard-main` remains the source of the original annotation product: transparent drawing, capture, undo/redo, tutorials, tray, startup, and update flows. Its code is not folded into Dock state or Docklet state.
- The old `vitro` implementation is not an implementation dependency. Its research and art assets may be inspected, but its runtime, state model, and window code are not reused.
- GlassDesk 1.4.0 is the primary architecture reference for layering and acceptance coverage: Core, Platform, AppServices, Dock, Launcher, Settings, and UI are separate responsibilities. Its implementation, brand, icons, and PolyForm-restricted additions are not copied into this product.
- The new product lives in a new top-level product domain, initially `dock-app/`, with its own runtime, tests, configuration namespace, and release packaging.
- The suite uses existing research and artwork only where the license and asset inventory permit it. The HTML prototypes are behavioral and visual references, not production runtime code.
- Windows Explorer remains the shell owner. There is no Explorer/DWM injection, shell replacement, or system-taskbar takeover.

## User-facing modules

### Dock Shell

The Dock Shell owns the edge surface and its item layout. It supports bottom, left, right, and top placement; monitor-specific placement; size and scale; optional magnification; running indicators; badges; launch animation; context menus; drag reorder; external file drag-in; and recycle-bin targets.

The Dock is divided into application, content, and system regions. Separators are data items, not CSS-only gaps, so they can be moved, persisted, and addressed by menus. A control tile opens preferences on single click and collapses or expands the Dock on double click.

### Application and window runtime

This module maps Dock application items to real Windows applications and top-level windows. It provides:

- application discovery and stable identity;
- icon extraction at the requested DPI;
- launch through a validated executable or registered app identity;
- running-state reconciliation;
- activation, minimize, restore, close, and force-close actions where Windows permits them;
- per-application window menus and previews;
- workspace and monitor association;
- notifications when an application or window disappears.

Polling is bounded and event-driven where Windows APIs allow it. The renderer never invents running state from a click.

### Launchpad

Launchpad exposes installed applications, recently used applications, search, categories, and drag-to-Dock. Search results are backed by discovered application identities and remain usable when an icon cannot be loaded. Launchpad has an explicit empty, loading, and error state.

### Shelf

Shelf manages files, folders, downloads, recent documents, and stacks. A stack can open as fan, grid, or list. Dragging a file over an application invokes the platform open-with action; dragging to Recycle Bin requests deletion and reports failure instead of silently discarding the item.

### System surface

The suite presents a controlled view of system tray items, a clock, keyboard shortcuts, notifications, and taskbar coexistence controls. It does not clone every Windows shell feature. Unsupported tray behavior is displayed as unavailable with an explicit action rather than a fake button.

### Docklet runtime

Docklets are extension modules with a manifest, version, declared permissions, lifecycle, refresh policy, visual form, and error state. The first release supports four visual forms:

- icon plus badge;
- sparkline with a bounded sample history;
- mini-dial with a normalized value;
- flyout panel with a compact summary and actions.

Docklets answer one small question on the Dock surface. Multi-step workflows belong in the flyout or a settings page. A Docklet cannot access process control, filesystem paths, window handles, network, or clipboard data without a manifest permission and a user-visible permission decision.

The host exposes a versioned API through a constrained QuickJS runtime or an isolated helper process. No Docklet receives arbitrary Rust, Win32, or DOM access. A crashed Docklet is disabled and reported without taking down the Dock.

### Settings and theme system

Settings use global defaults plus per-item overrides. They are stored in a versioned text format with atomic writes, backups, schema migration, and reset-to-default support. Settings cover placement, monitor, icon size, magnification, auto-hide, hot zones, keyboard shortcuts, motion profile, reduced-motion behavior, performance mode, Docklet permissions, theme, and import/export.

The material system follows the research tokens: blur and fill are theme parameters; edges use inner highlights rather than external outlines; the corner radius follows the Dock height; and all geometry snaps to the four-pixel grid. Material rendering is independent from interaction state so a theme cannot change hit testing.

## Architecture

### `shell-runtime`

Owns single-instance behavior, startup and shutdown, crash recovery, logging, configuration paths, update boundaries, and service wiring. It does not own Dock layout rules or Win32 window handles.

### `dock-core`

Owns pure domain types and state transitions:

- `ItemId`, `AppItem`, `WindowItem`, `FolderItem`, `StackItem`, `DockletItem`, `SeparatorItem`, `SystemItem`;
- `DockLayout` and monitor-specific layout;
- running-state and window snapshots;
- drag/reorder commands;
- menu models;
- Docklet manifests, permissions, and lifecycle events;
- versioned persistence models.

This layer is testable without Windows and contains no renderer, process launch, or DOM code.

### `windows-platform`

Owns Win32 bindings and adapters for monitor/work-area queries, DPI, top-level window enumeration, shell application identity, icon extraction, process launch, window activation, drag/drop, recycle-bin operations, taskbar detection, tray enumeration, and hot-zone input. Platform failures are typed and surfaced as domain events.

### `native-ui`

Owns borderless top-level windows, hit testing, Direct2D drawing, DirectComposition surfaces, DWM-backed material, text and icon layout, animation scheduling, menus, flyouts, sub-Docks, and window preview composition. The UI consumes snapshots and commands; it does not mutate domain state directly.

### `docklet-host`

Owns manifest validation, installation, signature/source metadata, permissions, sandbox lifecycle, refresh scheduling, resource limits, and crash isolation. Official Docklets use the same contract as third-party Docklets.

## Interaction contract

The first release must satisfy these behavior rules:

1. Clicking an uninstalled or unavailable target reports a recoverable error.
2. Clicking an installed application launches it; clicking a running item activates the correct window or opens its window list.
3. Running indicators come from platform snapshots and disappear when the process/window exits.
4. Right-click or long-press opens a data-driven menu with application actions, window entries, and Dock item actions.
5. Dragging rearranges items and persists the result. External Explorer drops are accepted only by targets that declare support.
6. Auto-hide has a visible edge trigger, a configurable delay, a manual lock, and reduced-motion behavior. It never becomes permanently visible because a timer failed.
7. Multi-monitor placement uses the selected monitor's work area and DPI. Moving a monitor or changing scaling cannot strand the Dock off-screen.
8. Every async platform action has loading, success, and failure states. No button exists only for visual demonstration.
9. Disabled/reduced-motion mode removes magnification, bounce, genie, and elastic overshoot while preserving the same actions and hit targets.
10. Settings and Docklet state survive restart and recover from an interrupted write.

## Visual acceptance

`vitro-nexus-full.html` is the primary interaction checklist. `vitro-macos-parity.html` is the material, geometry, tooltip, preview, and minimization reference. `vitro-effects-gallery.html` is the effect-slot inventory. The implementation must reproduce the behavior with native windows and real platform data rather than copying DOM structure.

Required visual behaviors include parabolic magnification, stable edge anchoring, a four-pixel running point, readable badges, controlled tooltip delay, launch feedback, menu/flyout hierarchy, sub-Dock entry/exit, and consistent light/dark material tokens. Reduced motion and battery/performance modes are first-class profiles.

## Performance and reliability targets

- Idle CPU target below one percent on a modern desktop with no active animation.
- Idle private memory target below 30 MB for the host before optional Docklets.
- No unbounded polling loop, per-frame process enumeration, or dynamic swap-chain resizing during ordinary hover.
- A failing Docklet, malformed configuration, missing icon, or unavailable application must not terminate the host.
- Configuration writes are atomic and recoverable; logs contain actionable error codes without leaking file contents or secrets.

## Delivery sequence

1. Create the isolated `dock-app` product domain and define the core types, persistence schema, and platform interfaces.
2. Build the native window and real application/window loop: launch, enumerate, activate, running state, multi-monitor placement, and taskbar coexistence.
3. Add Dock layout, drag/reorder, Shelf, Launchpad search, menus, auto-hide, and persistence.
4. Add system surface, tray integration, window previews, stacks, sub-Docks, and hot zones.
5. Add the Docklet manifest, installer, permissions, sandbox, built-in Docklets, and failure isolation.
6. Apply the Nexus/parity/effects visual tokens, motion profiles, accessibility behavior, and performance budgets.
7. Package only the independent suite for release while keeping Glassboard as a separate repository product.

Each stage is accepted through behavior tests and a native preview. A feature is not complete when its icon renders; it is complete when the platform action, state update, persistence, error path, and accessible interaction all work.

## Explicit non-goals

- Reusing the old Vitro Dock runtime or its fixed demo actions.
- Making the existing Glassboard drawing session responsible for Dock state.
- Injecting Explorer or DWM, replacing the taskbar, or taking ownership of the desktop shell.
- Shipping arbitrary JavaScript evaluation, unsigned code execution, or DOM-based fake windows.
- Adding a general desktop widget/wallpaper engine to this product boundary before the Dock, Launchpad, Shelf, window runtime, tray, and Docklet contracts are stable.
