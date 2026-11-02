# Native Dock Redesign

## Goal

Build a new Windows dock and plugin beautification application from an empty application layer. The existing Vitro implementation is excluded from the design and implementation inputs. The approved visual references are the research documents in `docs/research` and the existing PNG artwork under `vitro/assets/icons`.

## Visual direction

The dock is a Tahoe-inspired dark glass pill. It uses a low-alpha dark fill, backdrop blur, a small saturation lift, subtle monochrome grain, a one-pixel top inner highlight, and a one-pixel bottom inner shadow. There is no outer stroke and no heavy outer shadow. The default dock is 86 px tall with a 43 px radius, 16 px horizontal padding, 10 px vertical padding, and 8 px icon gaps.

Icons are rendered from a rebuilt asset map using the existing PNG artwork. Every icon is presented in a consistent 56 px square treatment with a common corner profile, restrained shadow, and a reflection beginning 12 to 14 px below the icon. Reflections fade with a mask and are omitted for plugin badges and running indicators. A running application uses a 4 px indicator dot six pixels below the icon.

## Interaction

- Hover uses a Gaussian-like neighbor falloff with a 1.28x maximum scale and an upward offset that keeps the icon bottoms aligned.
- Tooltip appears after a 500 ms dwell, moves upward by 6 px, and fades in over 150 ms.
- Launch feedback is two small upward bounces followed by a short settle.
- Drag reorder is supported inside the dock. A dragged item can be moved over the trash artwork to remove the shortcut.
- Reduced motion replaces bounce and scale transitions with a 200 ms fade.
- The dock remains an always-on-top, transparent desktop window and does not take over Explorer or the system taskbar.

## Plugin surface

The first release includes four compact read-only docklets: local time, weather placeholder artwork, CPU, and memory. CPU and memory use small sparkline or histogram treatments derived from the research specifications. The visual shell is shared with the dock icons and uses the same typography, spacing, and contrast rules. Detailed flyouts remain a later extension point.

## Architecture

The new application is a standalone Rust crate under `dock-rebuild/`. Its native window and input loop are implemented with Win32 APIs. Direct2D and DirectWrite provide the compositor, bitmap loading, text, paths, and animation drawing. A small, explicit state model owns icon order, hover position, drag state, plugin samples, and reduced-motion settings. Rendering is split into material, icon, plugin, and tooltip passes. The asset catalog is copied into the new crate as data; no old application source is imported.

## Performance and safety

- Poll cursor position at a bounded cadence and invalidate only while animation or plugin sampling requires it.
- Sample CPU, memory, and clock data at low frequency; stop plugin sampling while the dock is hidden.
- Cache decoded PNGs and DirectWrite layouts.
- Keep the window click-through outside painted icon and panel regions.
- Never launch, modify, or delete the existing `vitro.exe` during development.

## Verification

- Unit tests cover hover falloff, icon layout, reduced-motion timing, and plugin sample normalization.
- `cargo test` and `cargo build --release` run for the new crate.
- `git diff --check` passes.
- The old Dock process is absent after every build check.
- A local visual preview is produced before the native executable is built. Runtime visual claims are limited to what the preview and build evidence prove.
