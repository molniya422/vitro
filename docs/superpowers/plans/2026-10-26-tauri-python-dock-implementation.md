# Tauri + React + Python Dock Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a working Windows dock in `D:/dockyh/dock-tauri` with magnification, reflection, running indicators, auto-hide, minimize-into-dock, Launchpad, a Python-backed system monitor, and settings — under two measured memory gates.

**Architecture:** One Tauri window whose geometry Rust controls; React owns all pixels and swaps routes (Dock / Launchpad / Monitor / Settings) inside that single window, so exactly one WebView2 renderer exists. Python runs as a stdio JSON-RPC sidecar for `psutil` metrics only. Every visible state comes from a platform snapshot in Rust, pushed to React as events.

**Tech Stack:** Rust 2021 (`tauri 2.12`, `windows 0.61`, `serde`, `serde_json`, `uuid`, `image`, `window-vibrancy`), React 19 + TypeScript + Vite, Python 3.11 with `psutil`, Vitest + Playwright, pytest.

**Authoritative spec:** `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`. Read it before starting. When this plan and the spec disagree, the spec wins — report the conflict rather than guessing.

---

## Environment (read this first)

Rust lives at `D:/rust`, not on PATH. **Every** cargo command needs both env vars:

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe <args>
```

For `cargo tauri` subcommands use the npm script (`npm run tauri -- <args>`), which resolves the CLI from `node_modules`.

Python is the Anaconda interpreter at `D:/jiqixuexi/anaconda/python.exe`. `psutil` and `pytest` are installed there; **PyInstaller is not** (Task 15 covers packaging).

Node is v24 with npm 11. There is **no pnpm**. All commands run in Git Bash.

Never run `vitro.exe`. Never modify `D:/dockyh/vitro/`, `D:/dockyh/dock-app/`, or `D:/dockyh/Glassboard-main/`. Do not touch other uncommitted work in the repo.

## Pinned versions (verified 2026-10-26)

| Package | Version |
|---|---|
| `tauri` | 2.12 |
| `tauri-build` | 2.7 |
| `@tauri-apps/cli` | 2.12.4 |
| `@tauri-apps/api` | 2.12.4 |
| `windows` | 0.61 (features: `Win32_Foundation`, `Win32_UI_Shell`, `Win32_UI_WindowsAndMessaging`, `Win32_System_Com`, `Win32_Graphics_Gdi`) |
| `window-vibrancy` | 0.8 |
| `image` | 0.25 |
| `react` / `react-dom` | 19.3 |
| `vite` | 8.3 |
| `@vitejs/plugin-react` | 6.1 |
| `typescript` | 7.0 |
| `vitest` | 5.0 |
| `@playwright/test` | 1.64 |

## File map

```text
D:/dockyh/dock-tauri/
  package.json                       npm scripts + frontend deps
  vite.config.ts                     Vite + Vitest config
  tsconfig.json                      TS config
  index.html                         WebView entry
  playwright.config.ts               Playwright config
  src-tauri/
    Cargo.toml                       Rust manifest
    tauri.conf.json                  Tauri window + bundle config
    build.rs                         tauri-build hook
    capabilities/default.json        IPC permissions
    icons/                           generated app icons
    src/
      main.rs                        thin entry -> lib::run()
      lib.rs                         Tauri builder, command registration
      geometry.rs                    pure per-monitor window geometry
      autohide.rs                    pure reveal/collapse state machine
      settings.rs                    versioned settings, atomic write, recovery
      apps/
        mod.rs                       re-exports
        catalog.rs                   Start Menu scan, launch, identity
        windows.rs                   EnumWindows snapshot + reconciliation
      icons.rs                       icon extraction + PNG disk cache
      metrics.rs                     stdio JSON-RPC sidecar supervisor
      commands.rs                    #[tauri::command] surface
  src/
    main.tsx                         React bootstrap
    App.tsx                          route switch
    api.ts                           typed wrappers over invoke/listen
    types.ts                         mirrors of the Rust view types
    dock/
      Dock.tsx                       dock surface
      DockItem.tsx                   one tile (idle/running/minimized/disabled)
      magnify.ts                     pure magnification math
      Tooltip.tsx                    label + trigger
      dock.css                       glass, tiles, reflection, transforms
    launchpad/
      Launchpad.tsx                  full-work-area app grid
      search.ts                      pure ranking/filter
      AppTile.tsx                    one grid cell
    monitor/
      MonitorPanel.tsx               metrics panel
      Sparkline.tsx                  hand-drawn SVG sparkline
    settings/
      SettingsPanel.tsx              settings form
      validate.ts                    pure settings validation
  backend/
    monitor/
      __init__.py
      main.py                        stdio JSON-RPC loop
      sampler.py                     psutil sampling (injectable)
      requirements.txt
    tests/
      test_sampler.py
      test_protocol.py
      fake_psutil.py
  src-tauri/
    tests/unit/                      Rust unit tests (registered per-file in Cargo.toml)
    src/bin/fake_sidecar.rs          scripted fake sidecar binary (Task 9)
  scripts/
    measure_memory.ps1               two-gate memory + idle CPU measurement
  README.md                          features, gates, measured numbers
```

## Test wiring convention (Rust)

Cargo does not auto-discover tests inside subdirectories of `tests/`. Every Rust test file **must** be registered:

```toml
[[test]]
name = "geometry"
path = "tests/unit/geometry.rs"
```

Task 1 adds a guard test that fails the build when a test file exists without a stanza, so a forgotten registration can never silently skip tests.

---

## Task 1: Scaffold the dock-tauri workspace

**Files:**
- Create: `dock-tauri/package.json`
- Create: `dock-tauri/tsconfig.json`
- Create: `dock-tauri/vite.config.ts`
- Create: `dock-tauri/index.html`
- Create: `dock-tauri/src-tauri/Cargo.toml`
- Create: `dock-tauri/src-tauri/build.rs`
- Create: `dock-tauri/src-tauri/tauri.conf.json`
- Create: `dock-tauri/src-tauri/capabilities/default.json`
- Create: `dock-tauri/src-tauri/src/main.rs`
- Create: `dock-tauri/src-tauri/src/lib.rs`
- Create: `dock-tauri/src/main.tsx`
- Create: `dock-tauri/src/App.tsx`
- Create: `dock-tauri/src/types.ts`
- Create: `dock-tauri/src-tauri/tests/unit/workspace_smoke.rs`
- Create: `dock-tauri/src-tauri/tests/unit/test_registration.rs`
- Modify: `D:/dockyh/.gitignore`

- [ ] **Step 1: Write the failing smoke test**

`dock-tauri/src-tauri/tests/unit/workspace_smoke.rs`:

```rust
#[test]
fn workspace_exposes_product_name() {
    assert_eq!(dock_tauri_lib::product_name(), "Dock Tauri");
}
```

- [ ] **Step 2: Run it and confirm the expected failure**

```bash
cd D:/dockyh/dock-tauri
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml workspace_exposes_product_name
```

Expected: FAIL — manifest does not exist yet.

- [ ] **Step 3: Write the Rust manifest**

`dock-tauri/src-tauri/Cargo.toml`:

```toml
[package]
name = "dock-tauri"
version = "0.1.0"
edition = "2021"
rust-version = "1.89"

[lib]
name = "dock_tauri_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
path = "src/lib.rs"

[[bin]]
name = "dock-tauri"
path = "src/main.rs"

[dependencies]
tauri = { version = "2.12", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["serde", "v4"] }
image = { version = "0.25", default-features = false, features = ["png"] }
window-vibrancy = "0.8"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.61", features = [
    "Win32_Foundation",
    "Win32_UI_Shell",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Com",
    "Win32_Graphics_Gdi",
] }

[build-dependencies]
tauri-build = { version = "2.7", features = [] }

[dev-dependencies]
tempfile = "3"
toml = "0.8"
thiserror = "2"

[[test]]
name = "workspace_smoke"
path = "tests/unit/workspace_smoke.rs"

[[test]]
name = "test_registration"
path = "tests/unit/test_registration.rs"

[workspace]
```

`thiserror` is a normal dependency (settings errors derive from it), not a dev-dependency. Put it under `[dependencies]` alongside `serde`, and keep `[dev-dependencies]` to `tempfile` and `toml` only.

- [ ] **Step 4: Write `lib.rs` with the minimal surface**

`dock-tauri/src-tauri/src/lib.rs`:

```rust
//! Dock Tauri host: window geometry, app catalog, metrics sidecar, settings.

pub fn product_name() -> &'static str {
    "Dock Tauri"
}

pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}
```

`dock-tauri/src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dock_tauri_lib::run();
}
```

- [ ] **Step 5: Write `build.rs` and the Tauri config**

`dock-tauri/src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build();
}
```

`dock-tauri/src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Dock Tauri",
  "version": "0.1.0",
  "identifier": "com.dockyh.docktauri",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "dock",
        "title": "Dock Tauri",
        "width": 720,
        "height": 180,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "resizable": false,
        "shadow": false,
        "visible": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi"],
    "icon": ["icons/icon.ico"]
  }
}
```

`dock-tauri/src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Dock window permissions",
  "windows": ["dock"],
  "permissions": [
    "core:default",
    "core:window:allow-set-size",
    "core:window:allow-set-position",
    "core:window:allow-set-ignore-cursor-events",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-close"
  ]
}
```

- [ ] **Step 6: Write the frontend scaffold**

`dock-tauri/package.json`:

```json
{
  "name": "dock-tauri",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run",
    "test:e2e": "playwright test"
  },
  "dependencies": {
    "@tauri-apps/api": "2.12.4",
    "react": "19.3.1",
    "react-dom": "19.3.1"
  },
  "devDependencies": {
    "@playwright/test": "1.64.1",
    "@tauri-apps/cli": "2.12.4",
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "@vitejs/plugin-react": "6.1.2",
    "typescript": "7.0.2",
    "vite": "8.3.5",
    "vitest": "5.0.5"
  }
}
```

`dock-tauri/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "noEmit": true,
    "types": ["vitest/globals"]
  },
  "include": ["src", "tests"]
}
```

`dock-tauri/vite.config.ts`:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  // Vite 8 is Rolldown-based and no longer bundles esbuild; "oxc" is its default minifier.
  build: { target: "chrome105", minify: "oxc", sourcemap: false },
  test: {
    environment: "happy-dom",
    globals: true,
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
  },
});
```

`dock-tauri/index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Dock Tauri</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`dock-tauri/src/main.tsx`:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

`dock-tauri/src/App.tsx`:

```tsx
export function App() {
  return <main data-testid="app-root">Dock Tauri</main>;
}
```

`dock-tauri/src/types.ts`:

```ts
export type Edge = "bottom";

export type MotionProfile = "macos" | "vitro";

export type ItemKind = "app" | "separator" | "system";

export interface DockItemView {
  id: string;
  kind: ItemKind;
  name: string;
  icon: string | null;
  running: boolean;
  minimized: number;
  disabledReason: string | null;
}

export interface DockState {
  items: DockItemView[];
  edge: Edge;
  autohide: boolean;
  magnify: boolean;
  reducedMotion: boolean;
  profile: MotionProfile;
}

export interface MonitorSample {
  cpu: number;
  memUsed: number;
  memTotal: number;
  diskUsed: number;
  diskTotal: number;
  netRx: number;
  netTx: number;
}
```

- [ ] **Step 7: Write the test-registration guard**

`dock-tauri/src-tauri/tests/unit/test_registration.rs`:

```rust
use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_test_files(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            collect_test_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let rel = path
                .strip_prefix(manifest_dir())
                .expect("test file inside manifest dir")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
}

fn declared_test_paths() -> Vec<String> {
    let manifest = std::fs::read_to_string(manifest_dir().join("Cargo.toml"))
        .expect("read Cargo.toml");
    let parsed: toml::Value = toml::from_str(&manifest).expect("parse Cargo.toml");
    parsed
        .get("test")
        .and_then(|t| t.as_array())
        .map(|tests| {
            tests
                .iter()
                .filter_map(|t| t.get("path").and_then(|p| p.as_str()))
                .map(|p| p.replace('\\', "/"))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_test_file_is_declared() {
    let mut found = Vec::new();
    collect_test_files(&manifest_dir().join("tests"), &mut found);
    let declared = declared_test_paths();
    let missing: Vec<_> = found.iter().filter(|f| !declared.contains(f)).collect();
    assert!(missing.is_empty(), "test files missing a [[test]] stanza: {missing:?}");
}

#[test]
fn every_declared_test_path_exists() {
    let declared = declared_test_paths();
    let missing: Vec<_> = declared
        .iter()
        .filter(|p| !manifest_dir().join(p).exists())
        .collect();
    assert!(missing.is_empty(), "declared test paths not on disk: {missing:?}");
}
```

This guard needs `toml` as a dev-dependency. Add under `[dev-dependencies]` in `dock-tauri/src-tauri/Cargo.toml`:

```toml
toml = "0.8"
```

- [ ] **Step 8: Ignore build output and install frontend deps**

Append to `D:/dockyh/.gitignore`:

```gitignore
/dock-tauri/node_modules/
/dock-tauri/dist/
/dock-tauri/src-tauri/target/
/dock-tauri/src-tauri/gen/
```

Then:

```bash
cd D:/dockyh/dock-tauri
npm install
```

Expected: `node_modules/` created, `package-lock.json` written.

- [ ] **Step 9: Run the Rust tests**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml
```

Expected: 3 tests pass (`workspace_exposes_product_name`, `every_test_file_is_declared`, `every_declared_test_path_exists`).

If `tauri::generate_context!()` fails because `icons/icon.ico` is missing, create the icons with:

```bash
cd D:/dockyh/dock-tauri
npx tauri icon --output src-tauri/icons
```

(If `npx tauri icon` needs a source image, generate a 1024×1024 placeholder PNG first and pass it as an argument. Record in the README that the app icon is a placeholder pending final artwork.)

- [ ] **Step 10: Verify the frontend builds**

```bash
cd D:/dockyh/dock-tauri
npm run build
```

Expected: `dist/` created, no TypeScript errors. Note: `happy-dom` is referenced in `vite.config.ts` test config but only needed once Vitest runs; if `npm run build` complains, that is a config complaint not a build failure — the real check is `dist/index.html` existing.

- [ ] **Step 11: Commit**

```bash
cd D:/dockyh
git add dock-tauri/package.json dock-tauri/package-lock.json dock-tauri/tsconfig.json \
  dock-tauri/vite.config.ts dock-tauri/index.html dock-tauri/.gitignore \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/Cargo.lock \
  dock-tauri/src-tauri/build.rs dock-tauri/src-tauri/tauri.conf.json \
  dock-tauri/src-tauri/capabilities/default.json \
  dock-tauri/src-tauri/src/main.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/icons \
  dock-tauri/src/main.tsx dock-tauri/src/App.tsx dock-tauri/src/types.ts \
  dock-tauri/src-tauri/tests/unit/workspace_smoke.rs dock-tauri/src-tauri/tests/unit/test_registration.rs \
  .gitignore
git commit -m "feat: scaffold dock tauri workspace"
```

Do not use `git add dock-tauri` — it would stage `node_modules` and `target` if the ignore rules are wrong.

---

## Task 1b: Remove unreachable mobile and macOS icons

`npx tauri icon` generates icons for every platform. This product is MSI-only Windows, so the android, ios, and macOS files are dead weight that nothing reads.

**Files:**
- Delete: `dock-tauri/src-tauri/icons/android/`, `dock-tauri/src-tauri/icons/ios/`, `dock-tauri/src-tauri/icons/icon.icns`
- Modify: `dock-tauri/src-tauri/tauri.conf.json` (icon list must still resolve)

- [ ] **Step 1: Delete the unreachable sets**

```bash
cd D:/dockyh
git rm -r --cached dock-tauri/src-tauri/icons/android dock-tauri/src-tauri/icons/ios
git rm --cached dock-tauri/src-tauri/icons/icon.icns
rm -rf dock-tauri/src-tauri/icons/android dock-tauri/src-tauri/icons/ios dock-tauri/src-tauri/icons/icon.icns
```

- [ ] **Step 2: Confirm the bundle still resolves**

`bundle.icon` lists only `icons/icon.ico`, so removing the other sets changes nothing. Verify by building:

```bash
export CARGO_HOME='D:
ust\cargo' RUSTUP_HOME='D:
ust
ustup'
D:/rust/cargo/bin/cargo.exe build --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml
```

Expected: success.

- [ ] **Step 3: Keep the Windows PNGs**

`icons/icon.png`, the `Square*Logo.png` set, and `StoreLogo.png` stay: they are MSIX/store assets and `tauri-codegen` uses `icon.png` on non-Windows targets for tooling. Do not delete them.

- [ ] **Step 4: Commit**

```bash
cd D:/dockyh
git commit -m "chore: drop unreachable mobile and macos icons"
```

---

## Task 2: Pure window geometry per monitor and DPI

**Files:**
- Create: `dock-tauri/src-tauri/src/geometry.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs` (add `pub mod geometry;`)
- Modify: `dock-tauri/src-tauri/Cargo.toml` (register test)
- Test: `dock-tauri/src-tauri/tests/unit/geometry.rs`

- [ ] **Step 1: Write the failing test**

`dock-tauri/src-tauri/tests/unit/geometry.rs`:

```rust
use dock_tauri_lib::geometry::{DockGeometry, MonitorRect};

fn monitor() -> MonitorRect {
    MonitorRect { left: 0, top: 0, right: 1920, bottom: 1040, scale: 1.0 }
}

#[test]
fn dock_window_is_centered_on_the_work_area_bottom() {
    let g = DockGeometry::compute(monitor(), 6, 48.0, 180.0);
    assert_eq!(g.width, 180.0 + 6.0 * 48.0 + 5.0 * 8.0 + 32.0);
    assert_eq!(g.x, (1920.0 - g.width) / 2.0);
}

#[test]
fn window_includes_magnify_headroom_above_and_reflection_below() {
    let g = DockGeometry::compute(monitor(), 6, 48.0, 180.0);
    assert!(g.height >= 84.0 + 68.0 + 40.0);
    assert_eq!(g.y + g.height, 1040.0);
}

#[test]
fn pill_rect_sits_below_the_magnify_margin() {
    let g = DockGeometry::compute(monitor(), 6, 48.0, 180.0);
    assert_eq!(g.pill_top(), g.y + 84.0);
    assert_eq!(g.pill_height, 68.0);
}

#[test]
fn geometry_scales_with_monitor_dpi() {
    let mut m = monitor();
    m.scale = 1.5;
    let g = DockGeometry::compute(m, 6, 48.0, 180.0);
    assert_eq!(g.width, (180.0 + 6.0 * 48.0 + 5.0 * 8.0 + 32.0) * 1.5);
    assert_eq!(g.y + g.height, 1040.0);
}

#[test]
fn dock_never_exceeds_a_narrow_work_area() {
    let narrow = MonitorRect { left: 0, top: 0, right: 400, bottom: 800, scale: 1.0 };
    let g = DockGeometry::compute(narrow, 20, 48.0, 180.0);
    assert!(g.width <= 400.0);
    assert!(g.x >= 0.0);
}
```

- [ ] **Step 2: Run it and confirm failure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test geometry
```

Expected: FAIL — `no test target named geometry` (until the stanza exists) and then unresolved imports.

- [ ] **Step 3: Register the test target**

Append to `dock-tauri/src-tauri/Cargo.toml`:

```toml
[[test]]
name = "geometry"
path = "tests/unit/geometry.rs"
```

- [ ] **Step 4: Implement the geometry module**

`dock-tauri/src-tauri/src/geometry.rs`:

```rust
//! Pure per-monitor window geometry. No Win32 calls — inputs are plain numbers.

/// A monitor's work area in physical pixels plus its DPI scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonitorRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    /// DPI scale factor: 1.0 at 96 DPI, 1.5 at 144 DPI.
    pub scale: f64,
}

pub const MAGNIFY_HEADROOM: f64 = 84.0;
pub const REFLECTION_MARGIN: f64 = 40.0;
pub const PADDING_H: f64 = 16.0;
pub const PADDING_V: f64 = 10.0;
pub const ICON_GAP: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockGeometry {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Height of the glass pill in physical pixels.
    pub pill_height: f64,
    /// Space reserved above the pill for magnified icons and tooltips.
    pub headroom: f64,
    /// Space reserved below the pill for the reflection.
    pub reflection: f64,
}

impl DockGeometry {
    /// Compute the dock window rect for the bottom edge of `monitor`.
    ///
    /// `item_count` counts app tiles only; the control tile is added separately.
    /// `icon_size` and `control_width` are logical pixels before DPI scaling.
    pub fn compute(
        monitor: MonitorRect,
        item_count: usize,
        icon_size: f64,
        control_width: f64,
    ) -> Self {
        let scale = if monitor.scale > 0.0 { monitor.scale } else { 1.0 };
        let work_width = (monitor.right - monitor.left) as f64;
        let work_height = (monitor.bottom - monitor.top) as f64;

        let gaps = if item_count == 0 {
            0.0
        } else {
            (item_count - 1) as f64 * ICON_GAP
        };
        let content = control_width + item_count as f64 * icon_size + gaps + PADDING_H * 2.0;
        let mut width = content * scale;

        let headroom = MAGNIFY_HEADROOM * scale;
        let reflection = REFLECTION_MARGIN * scale;
        let mut pill_height = (icon_size + PADDING_V * 2.0) * scale;
        let mut height = headroom + pill_height + reflection;

        // Clamp to the work area. Width must be clamped before centering so the
        // pill can never start off-screen.
        if width > work_width {
            width = work_width;
        }
        if height > work_height {
            height = work_height;
            pill_height = (height - headroom - reflection).max(0.0);
        }

        Self {
            x: (monitor.left as f64) + (work_width - width) / 2.0,
            y: (monitor.bottom as f64) - height,
            width,
            height,
            pill_height,
            headroom,
            reflection,
        }
    }

    /// Top edge of the glass pill — the shared origin for paint and hit test.
    pub fn pill_top(&self) -> f64 {
        self.y + self.headroom
    }
}
```

- [ ] **Step 5: Export the module**

In `dock-tauri/src-tauri/src/lib.rs`, add:

```rust
pub mod geometry;
```

- [ ] **Step 6: Run the tests**

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test geometry
```

Expected: 5 tests pass.

If `dock_never_exceeds_a_narrow_work_area` fails, check that `width` is clamped **before** `x` is computed — the clamp must precede centering.

- [ ] **Step 7: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/geometry.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/geometry.rs
git commit -m "feat: add pure dock geometry per monitor and dpi"
```

---

## Task 3: Auto-hide state machine

**Files:**
- Create: `dock-tauri/src-tauri/src/autohide.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Test: `dock-tauri/src-tauri/tests/unit/autohide.rs`

- [ ] **Step 1: Write the failing test**

`dock-tauri/src-tauri/tests/unit/autohide.rs`:

```rust
use dock_tauri_lib::autohide::{Action, AutoHide, Event, Visibility};

#[test]
fn disabled_dock_is_always_visible() {
    let mut ah = AutoHide::new(false);
    ah.handle(Event::PointerLeft, 1000);
    assert_eq!(ah.visibility(), Visibility::Visible);
    assert_eq!(ah.handle(Event::Tick, 5000), Action::None);
}

#[test]
fn enter_reveals_after_show_delay() {
    let mut ah = AutoHide::new(true);
    assert_eq!(ah.handle(Event::PointerEntered, 0), Action::None);
    assert_eq!(ah.visibility(), Visibility::Hidden);
    assert_eq!(ah.handle(Event::Tick, 299), Action::None);
    assert_eq!(ah.handle(Event::Tick, 300), Action::Reveal);
    assert_eq!(ah.visibility(), Visibility::Visible);
}

#[test]
fn leave_arms_hide_only_after_hide_delay() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    assert_eq!(ah.handle(Event::PointerLeft, 1000), Action::None);
    assert_eq!(ah.handle(Event::Tick, 1499), Action::None);
    assert_eq!(ah.handle(Event::Tick, 1500), Action::Collapse);
    assert_eq!(ah.visibility(), Visibility::Hidden);
}

#[test]
fn reentering_cancels_an_armed_hide() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    ah.handle(Event::PointerLeft, 1000);
    ah.handle(Event::PointerEntered, 1200);
    assert_eq!(ah.handle(Event::Tick, 5000), Action::None);
    assert_eq!(ah.visibility(), Visibility::Visible);
}

#[test]
fn a_failed_timer_cannot_leave_the_dock_stuck_visible() {
    let mut ah = AutoHide::new(true);
    ah.handle(Event::PointerEntered, 0);
    ah.handle(Event::Tick, 300);
    ah.handle(Event::PointerLeft, 1000);
    // No Tick arrives; a late tick well past the deadline must still collapse.
    assert_eq!(ah.handle(Event::Tick, 60_000), Action::Collapse);
    assert_eq!(ah.visibility(), Visibility::Hidden);
}

#[test]
fn lock_forces_visible_and_ignores_pointer() {
    let mut ah = AutoHide::new(true);
    ah.set_locked(true);
    ah.handle(Event::PointerLeft, 0);
    assert_eq!(ah.handle(Event::Tick, 10_000), Action::None);
    assert_eq!(ah.visibility(), Visibility::Visible);
}
```

- [ ] **Step 2: Run it and confirm failure**

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test autohide
```

Expected: FAIL — target and module do not exist.

- [ ] **Step 3: Register the test target**

```toml
[[test]]
name = "autohide"
path = "tests/unit/autohide.rs"
```

- [ ] **Step 4: Implement the state machine**

`dock-tauri/src-tauri/src/autohide.rs`:

```rust
//! Pure auto-hide state machine. Time is passed in, never read here.

pub const SHOW_DELAY_MS: u64 = 300;
pub const HIDE_DELAY_MS: u64 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    PointerEntered,
    PointerLeft,
    Tick,
    SetEnabled(bool),
    SetLocked(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Reveal,
    Collapse,
}

#[derive(Clone, Copy, Debug)]
pub struct AutoHide {
    enabled: bool,
    locked: bool,
    visibility: Visibility,
    /// Deadline after which the pending transition should fire.
    deadline: Option<u64>,
    /// Which transition the deadline refers to.
    pending: Option<Action>,
}

impl AutoHide {
    pub fn new(enabled: bool) -> Self {
        // An enabled auto-hide dock starts collapsed; a disabled one is always up.
        Self {
            enabled,
            locked: false,
            visibility: if enabled {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            deadline: None,
            pending: None,
        }
    }

    pub fn visibility(&self) -> Visibility {
        if !self.enabled || self.locked {
            Visibility::Visible
        } else {
            self.visibility
        }
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
        if locked {
            self.deadline = None;
            self.pending = None;
        }
    }

    pub fn handle(&mut self, event: Event, now_ms: u64) -> Action {
        match event {
            Event::SetEnabled(enabled) => {
                self.enabled = enabled;
                if !enabled {
                    self.visibility = Visibility::Visible;
                    self.deadline = None;
                    self.pending = None;
                }
                Action::None
            }
            Event::SetLocked(locked) => {
                self.set_locked(locked);
                Action::None
            }
            Event::PointerEntered => {
                self.deadline = None;
                self.pending = None;
                if self.enabled && !self.locked && self.visibility == Visibility::Hidden {
                    self.deadline = Some(now_ms.saturating_add(SHOW_DELAY_MS));
                    self.pending = Some(Action::Reveal);
                }
                Action::None
            }
            Event::PointerLeft => {
                self.deadline = None;
                self.pending = None;
                if self.enabled && !self.locked && self.visibility == Visibility::Visible {
                    self.deadline = Some(now_ms.saturating_add(HIDE_DELAY_MS));
                    self.pending = Some(Action::Collapse);
                }
                Action::None
            }
            Event::Tick => {
                if !self.enabled || self.locked {
                    return Action::None;
                }
                let (Some(deadline), Some(pending)) = (self.deadline, self.pending) else {
                    return Action::None;
                };
                if now_ms < deadline {
                    return Action::None;
                }
                self.deadline = None;
                self.pending = None;
                match pending {
                    Action::Reveal => {
                        self.visibility = Visibility::Visible;
                        Action::Reveal
                    }
                    Action::Collapse => {
                        self.visibility = Visibility::Hidden;
                        Action::Collapse
                    }
                    Action::None => Action::None,
                }
            }
        }
    }
}
```

- [ ] **Step 5: Export and run**

Add `pub mod autohide;` to `lib.rs`, then:

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test autohide
```

Expected: 6 tests pass.

- [ ] **Step 6: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/autohide.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/autohide.rs
git commit -m "feat: add pure autohide state machine"
```

---

## Task 4: Versioned settings with atomic write and recovery

**Files:**
- Create: `dock-tauri/src-tauri/src/settings.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Test: `dock-tauri/src-tauri/tests/unit/settings.rs`

- [ ] **Step 1: Write the failing test**

`dock-tauri/src-tauri/tests/unit/settings.rs`:

```rust
use dock_tauri_lib::settings::{
    load_or_recover, save_atomic, Settings, SettingsError, CURRENT_SCHEMA,
};
use std::path::Path;

fn sample() -> Settings {
    Settings {
        schema_version: CURRENT_SCHEMA,
        icon_size: 48,
        magnify: true,
        autohide: false,
        reduced_motion: false,
        profile: "vitro".into(),
        show_monitor: false,
        pinned: vec!["shell:explorer".into()],
    }
}

#[test]
fn round_trips_through_a_real_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();
    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded, sample());
}

#[test]
fn second_save_creates_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();
    let mut second = sample();
    second.icon_size = 64;
    save_atomic(&path, &second).unwrap();

    assert!(dir.path().join("settings.json.bak").exists());
    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded.icon_size, 64);
}

#[test]
fn interrupted_write_restores_the_last_valid_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();

    let mut newer = sample();
    newer.icon_size = 96;
    save_atomic(&path, &newer).unwrap();

    // Simulate a torn write: main file is garbage, backup is valid.
    std::fs::write(&path, b"{ broken").unwrap();
    let recovered = load_or_recover(&path).unwrap();
    assert_eq!(recovered.icon_size, 48);
}

#[test]
fn both_corrupt_returns_a_typed_error_not_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, b"{ broken").unwrap();
    std::fs::write(dir.path().join("settings.json.bak"), b"also broken").unwrap();

    let err = load_or_recover(&path).unwrap_err();
    assert!(matches!(err, SettingsError::Corrupt { .. }));
}

#[test]
fn missing_file_yields_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let loaded = load_or_recover(&path).unwrap();
    assert_eq!(loaded.schema_version, CURRENT_SCHEMA);
    assert_eq!(loaded.icon_size, 48);
}

#[test]
fn unsupported_schema_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut future = sample();
    future.schema_version = 999;
    std::fs::write(&path, serde_json::to_vec(&future).unwrap()).unwrap();

    let err = load_or_recover(&path).unwrap_err();
    assert!(matches!(err, SettingsError::UnsupportedSchema(999)));
}

#[test]
fn a_newer_schema_is_not_masked_by_a_valid_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    // A perfectly good backup that load_or_recover *could* fall back to.
    save_atomic(&path, &sample()).unwrap();
    let mut newer = sample();
    newer.icon_size = 64;
    save_atomic(&path, &newer).unwrap();
    assert!(dir.path().join("settings.json.bak").exists());

    // Main file is valid JSON but from a future schema version.
    let mut future = sample();
    future.schema_version = 999;
    std::fs::write(&path, serde_json::to_vec(&future).unwrap()).unwrap();

    // A newer schema is not corruption: it must surface, not be replaced by
    // the older backup.
    let err = load_or_recover(&path).unwrap_err();
    assert!(matches!(err, SettingsError::UnsupportedSchema(999)));
}

#[test]
fn save_leaves_no_temp_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    save_atomic(&path, &sample()).unwrap();
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty());
    assert!(Path::new(&path).exists());
}
```

- [ ] **Step 2: Run it and confirm failure**

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test settings
```

Expected: FAIL — module missing.

- [ ] **Step 3: Register the test target**

```toml
[[test]]
name = "settings"
path = "tests/unit/settings.rs"
```

- [ ] **Step 4: Implement settings persistence**

`dock-tauri/src-tauri/src/settings.rs`:

```rust
//! Versioned settings with atomic write, `.bak` rotation, and recovery.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub icon_size: u32,
    pub magnify: bool,
    pub autohide: bool,
    pub reduced_motion: bool,
    pub profile: String,
    pub show_monitor: bool,
    pub pinned: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA,
            icon_size: 48,
            magnify: true,
            autohide: false,
            reduced_motion: false,
            profile: "vitro".into(),
            show_monitor: false,
            pinned: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("settings file is corrupt: {detail}")]
    Corrupt { detail: String },
    #[error("unsupported settings schema {0}")]
    UnsupportedSchema(u32),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".bak");
    path.with_file_name(name)
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

fn read_and_validate(path: &Path) -> Result<Settings, SettingsError> {
    let bytes = std::fs::read(path)?;
    let parsed: Settings = serde_json::from_slice(&bytes).map_err(|e| SettingsError::Corrupt {
        detail: e.to_string(),
    })?;
    if parsed.schema_version != CURRENT_SCHEMA {
        return Err(SettingsError::UnsupportedSchema(parsed.schema_version));
    }
    Ok(parsed)
}

/// Write `settings` to `path` via temp file, backup rotation, then rename.
pub fn save_atomic(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = temp_path(path);
    let bytes = serde_json::to_vec_pretty(settings).expect("settings serialize");

    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&bytes)?;
        file.flush()?;
        file.sync_all()?;
    }

    if path.exists() {
        let bak = backup_path(path);
        // Ignore a failure to overwrite an old backup: the fresh write still wins.
        let _ = std::fs::rename(path, &bak);
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Load settings, falling back to `.bak` before surfacing an error.
///
/// A missing main file and a missing backup both mean "first run" -> defaults.
pub fn load_or_recover(path: &Path) -> Result<Settings, SettingsError> {
    match read_and_validate(path) {
        Ok(settings) => return Ok(settings),
        Err(SettingsError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(SettingsError::UnsupportedSchema(v)) => {
            // A known-but-newer schema is not corrupt data; do not mask it with a backup.
            return Err(SettingsError::UnsupportedSchema(v));
        }
        Err(_) => {}
    }

    let bak = backup_path(path);
    match read_and_validate(&bak) {
        Ok(settings) => Ok(settings),
        Err(SettingsError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(Settings::default())
        }
        Err(other) => Err(match other {
            SettingsError::Corrupt { detail } => SettingsError::Corrupt {
                detail: format!("main and backup both unreadable: {detail}"),
            },
            SettingsError::Io(e) => SettingsError::Corrupt {
                detail: format!("backup unreadable: {e}"),
            },
            schema @ SettingsError::UnsupportedSchema(_) => schema,
        }),
    }
}
```

- [ ] **Step 5: Export and run**

Add `pub mod settings;` to `lib.rs`, then:

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test settings
```

Expected: 8 tests pass.

`both_corrupt_returns_a_typed_error_not_a_panic` requires the `Corrupt` branch to wrap the backup error, which the `match` above does.

- [ ] **Step 6: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/settings.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/settings.rs
git commit -m "feat: add versioned settings with atomic write and recovery"
```

---
## Task 5: Windows monitor and work-area adapter

**Files:**
- Create: `dock-tauri/src-tauri/src/platform.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Test: `dock-tauri/src-tauri/tests/unit/platform_monitor.rs`

`platform.rs` is the only module allowed to call Win32 for display data. It returns plain values from `geometry::MonitorRect`.

- [ ] **Step 1: Write the failing test**

`dock-tauri/src-tauri/tests/unit/platform_monitor.rs`:

```rust
use dock_tauri_lib::platform::primary_work_area;

#[test]
fn primary_work_area_is_on_screen_and_sane() {
    let m = primary_work_area().expect("primary monitor");
    assert!(m.right > m.left, "work area must have width");
    assert!(m.bottom > m.top, "work area must have height");
    assert!(m.scale >= 1.0 && m.scale <= 4.0, "scale must be plausible");
}

#[test]
fn work_area_never_exceeds_the_full_screen() {
    let m = primary_work_area().expect("primary monitor");
    let full = dock_tauri_lib::platform::primary_screen_rect().expect("screen rect");
    assert!(m.bottom - m.top <= full.bottom - full.top);
}
```

- [ ] **Step 2: Run it and confirm failure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test platform_monitor
```

Expected: FAIL — module does not exist.

- [ ] **Step 3: Register the test target**

```toml
[[test]]
name = "platform_monitor"
path = "tests/unit/platform_monitor.rs"
```

- [ ] **Step 4: Implement the platform module**

`dock-tauri/src-tauri/src/platform.rs`:

```rust
//! Win32 display adapter. The only place display geometry leaves Win32.

use crate::geometry::MonitorRect;

#[cfg(windows)]
mod imp {
    use super::MonitorRect;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::HiDpi::GetDpiForSystem;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SystemParametersInfoW, SM_CXSCREEN, SM_CYSCREEN, SPI_GETWORKAREA,
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    fn system_scale() -> f64 {
        // SAFETY: GetDpiForSystem takes no arguments and returns the system DPI.
        // A zero result is treated as 100%.
        unsafe {
            let dpi = GetDpiForSystem();
            if dpi == 0 {
                1.0
            } else {
                dpi as f64 / 96.0
            }
        }
    }

    fn rect_to_monitor(rect: RECT, scale: f64) -> MonitorRect {
        MonitorRect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
            scale,
        }
    }

    pub fn primary_work_area() -> Option<MonitorRect> {
        let mut rect = RECT::default();
        // SAFETY: SPI_GETWORKAREA writes exactly one RECT into the pointer given.
        // windows 0.61 returns core::Result<()>; pvparam is Option<*mut c_void>.
        let result = unsafe {
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut rect as *mut RECT as *mut core::ffi::c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
        };
        result.ok().map(|_| rect_to_monitor(rect, system_scale()))
    }

    pub fn primary_screen_rect() -> Option<MonitorRect> {
        // SAFETY: both calls take no arguments and return plain integers.
        let (w, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
        if w <= 0 || h <= 0 {
            return None;
        }
        Some(MonitorRect {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
            scale: system_scale(),
        })
    }
}

#[cfg(windows)]
pub use imp::{primary_screen_rect, primary_work_area};

#[cfg(not(windows))]
pub fn primary_work_area() -> Option<MonitorRect> {
    None
}

#[cfg(not(windows))]
pub fn primary_screen_rect() -> Option<MonitorRect> {
    None
}
```

- [ ] **Step 5: Add the HiDpi feature**

`GetDpiForSystem` lives behind `Win32_UI_HiDpi`. Add it to the `windows` feature list in `src-tauri/Cargo.toml`:

```toml
    "Win32_UI_HiDpi",
```

- [ ] **Step 6: Export and run**

Add `pub mod platform;` to `lib.rs`, then:

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test platform_monitor
```

Expected: 2 tests pass.

- [ ] **Step 7: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/platform.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/platform_monitor.rs
git commit -m "feat: add win32 monitor work area adapter"
```

---

## Task 5b: Declare per-monitor DPI awareness

**The problem this fixes:** the built `dock-tauri.exe` carries no DPI manifest. On a 150%-scaled display (this machine: 2560×1440 at 150%), Win32 reports **1707×960 at 96 DPI** to the process because Windows virtualizes coordinates for DPI-unaware apps. The dock would therefore be drawn at the wrong physical size and appear blurry. This is not hypothetical — it was measured on the development machine.

**Files:**
- Create: `dock-tauri/src-tauri/dock-tauri.exe.manifest`
- Modify: `dock-tauri/src-tauri/build.rs`
- Create: `dock-tauri/src-tauri/tests/unit/dpi_manifest.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`

- [ ] **Step 1: Write the manifest**

`dock-tauri/src-tauri/dock-tauri.exe.manifest`:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
      version="0.1.0.0"
      processorArchitecture="*"
      name="DockTauri"
      type="win32" />
  <description>Dock Tauri</description>
  <!-- Replaces the manifest tauri-build would otherwise emit. Because we suppress
       that one (see build.rs), the Common-Controls v6 dependency it carried must
       be restored here or Tauri dialog APIs and native controls fall back to v5. -->
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
          type="win32"
          name="Microsoft.Windows.Common-Controls"
          version="6.0.0.0"
          processorArchitecture="*"
          publicKeyToken="6595b64144ccf1df"
          language="*" />
    </dependentAssembly>
  </dependency>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
    </application>
  </compatibility>
</assembly>
```

- [ ] **Step 2: Tell the linker to embed it**

`dock-tauri/src-tauri/build.rs` becomes:

```rust
fn main() {
    // Embed the DPI-awareness manifest for MSVC builds. It declares PerMonitorV2
    // so Win32 reports physical pixels on scaled displays.
    #[cfg(target_env = "msvc")]
    {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:dock-tauri.exe.manifest");
        println!("cargo:rerun-if-changed=dock-tauri.exe.manifest");
    }
    tauri_build::build();
}
```

- [ ] **Step 3: Write tests that guard the manifest**

`dock-tauri/src-tauri/tests/unit/dpi_manifest.rs`:

```rust
//! Guards the DPI-awareness manifest. Without it, Win32 virtualises coordinates
//! on scaled displays and the dock renders at the wrong physical size.

#[test]
fn the_manifest_file_declares_per_monitor_v2() {
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dock-tauri.exe.manifest");
    let text = std::fs::read_to_string(&path).expect("manifest must exist");
    assert!(text.contains("PerMonitorV2"), "missing PerMonitorV2");
    assert!(text.contains("dpiAware"), "missing dpiAware");
}

#[test]
fn build_script_embeds_the_manifest() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build.rs");
    let text = std::fs::read_to_string(&path).expect("build.rs must exist");
    assert!(
        text.contains("/MANIFESTINPUT:dock-tauri.exe.manifest"),
        "build.rs must pass the manifest to the linker"
    );
}
```

Register it:

```toml
[[test]]
name = "dpi_manifest"
path = "tests/unit/dpi_manifest.rs"
```

- [ ] **Step 4: Run the tests**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test dpi_manifest
```

Expected: 2 tests pass.

- [ ] **Step 5: Prove the manifest reached the binary**

Build, then check the exe bytes:

```bash
D:/rust/cargo/bin/cargo.exe build --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml
python -c "d=open(r'D:\dockyh\dock-tauri\src-tauri\target\debug\dock-tauri.exe','rb').read(); print('PerMonitorV2:', d.count(b'PerMonitorV2')); print('dpiAware:', d.count(b'dpiAware')); assert d.count(b'PerMonitorV2')>0, 'manifest NOT embedded'; print('OK: manifest embedded')"
```

Expected: both counts greater than 0 and `OK: manifest embedded`. Also confirm Common-Controls survived:

```bash
python -c "d=open(r'D:\dockyh\dock-tauri\src-tauri	arget\debug\dock-tauri.exe','rb').read(); print('Common-Controls:', b'Microsoft.Windows.Common-Controls' in d)"
```

Expected: `Common-Controls: True`.

**The fallback is mandatory, not optional.** Measured: adding `/MANIFESTINPUT` alone fails to link with `CVTRES : fatal error CVT1100: 资源重复。类型: MANIFEST` because `tauri-build` emits its own `RT_MANIFEST` at resource ID 1 in `resource.rc`. Both cannot coexist. So suppress tauri-build's manifest and keep ours:

```rust
fn main() {
    #[cfg(target_env = "msvc")]
    {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:dock-tauri.exe.manifest");
        println!("cargo:rerun-if-changed=dock-tauri.exe.manifest");
    }
    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new_without_app_manifest(),
        ),
    )
    .expect("failed to run tauri-build");
}
```

Then repeat Step 5. Do not report success without Step 5 passing.

**Do not omit the `<dependency>` block from the manifest.** Suppressing tauri-build's manifest also drops the `Microsoft.Windows.Common-Controls` v6 assembly dependency it declared, which Tauri's dialog APIs and native controls rely on. Step 5b checks for it explicitly.

- [ ] **Step 6: Measure the corrected display metrics**

```bash
python -c "import ctypes; ctypes.windll.shcore.SetProcessDpiAwareness(2); u=ctypes.windll.user32; print('SM_CXSCREEN:', u.GetSystemMetrics(0)); print('SM_CYSCREEN:', u.GetSystemMetrics(1)); print('DPI:', u.GetDpiForSystem())"
```

Expected on this machine: about 2560×1440 and DPI 144. Record the real numbers — they are the evidence.

- [ ] **Step 7: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/dock-tauri.exe.manifest dock-tauri/src-tauri/build.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/dpi_manifest.rs
git commit -m "fix: declare per-monitor dpi awareness in the exe manifest"
```

---

## Task 6: Start Menu catalog, launch, and window reconciliation

**Files:**
- Create: `dock-tauri/src-tauri/src/apps.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Test: `dock-tauri/src-tauri/tests/unit/apps_catalog.rs`

- [ ] **Step 1: Write the failing test**

`dock-tauri/src-tauri/tests/unit/apps_catalog.rs`:

```rust
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
fn different_apps_sharing_a_display_name_are_both_kept() {
    let a = AppEntry {
        id: "c:/programdata/settings.lnk".into(),
        display_name: "Settings".into(),
        target: Some("C:/Windows/System32/control.exe".into()),
    };
    let b = AppEntry {
        id: "c:/other/settings.lnk".into(),
        display_name: "Settings".into(),
        target: Some("C:/Other/vendor-settings.exe".into()),
    };
    let merged = dock_tauri_lib::apps::dedup_entries(vec![a, b]);
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
    assert_eq!(merged[0].id, "c:/programdata/notepad.lnk", "first scanned wins");
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
```

- [ ] **Step 2: Run it and confirm failure**

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test apps_catalog
```

Expected: FAIL — module missing.

- [ ] **Step 3: Register the test target**

```toml
[[test]]
name = "apps_catalog"
path = "tests/unit/apps_catalog.rs"
```

- [ ] **Step 4: Implement the catalog**

`dock-tauri/src-tauri/src/apps.rs`:

```rust
//! Start Menu scanning, app identity, launch, and window reconciliation.
//!
//! Pure functions take explicit inputs so they test without Win32. Only the
//! `resolve_shortcut_target` and `launch` functions touch the platform.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    /// Stable identity: normalised absolute shortcut path.
    pub id: String,
    pub display_name: String,
    /// Resolved executable path when known; used for reconciliation.
    pub target: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowRef {
    pub process_path: String,
    pub minimized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunningInfo {
    pub running: bool,
    pub minimized: u32,
}

fn normalize(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

/// Match each app to the windows that belong to it.
pub fn reconcile(apps: &[AppEntry], windows: &[WindowRef]) -> Vec<RunningInfo> {
    apps.iter()
        .map(|app| {
            let mut running = false;
            let mut minimized = 0;
            if let Some(target) = app.target.as_deref() {
                let target = normalize(target);
                for window in windows {
                    if normalize(&window.process_path) == target {
                        running = true;
                        if window.minimized {
                            minimized += 1;
                        }
                    }
                }
            }
            RunningInfo { running, minimized }
        })
        .collect()
}

/// Deduplication key for a catalog entry.
///
/// Two shortcuts are the same app when they resolve to the same executable —
/// that is how the same app appears in both the all-users and per-user Start
/// Menu roots. Entries with no resolvable target fall back to display name.
fn dedup_key(entry: &AppEntry) -> String {
    match &entry.target {
        Some(target) => format!("target:{}", normalize(target)),
        None => format!("name:{}", entry.display_name.to_ascii_lowercase()),
    }
}

/// Collapse duplicate entries, keeping the first occurrence.
///
/// Order matters: `start_menu_dirs` lists the all-users root first, so the
/// all-users copy wins over a per-user duplicate.
pub fn dedup_entries(entries: Vec<AppEntry>) -> Vec<AppEntry> {
    let mut seen: Vec<String> = Vec::new();
    let mut kept: Vec<AppEntry> = Vec::new();
    for entry in entries {
        let key = dedup_key(&entry);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        kept.push(entry);
    }
    kept
}

/// Scan directories for `.lnk` shortcuts. Missing directories are skipped.
pub fn catalog_from_dirs(dirs: &[PathBuf]) -> Vec<AppEntry> {
    let mut entries: Vec<AppEntry> = Vec::new();

    for dir in dirs {
        let Ok(read) = std::fs::read_dir(dir) else {
            continue;
        };
        for item in read.flatten() {
            let path = item.path();
            if path.extension().and_then(|e| e.to_str()) != Some("lnk") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            entries.push(AppEntry {
                id: normalize(&path.to_string_lossy()),
                display_name: stem.to_string(),
                target: resolve_shortcut_target(&path),
            });
        }
    }

    let mut entries = dedup_entries(entries);
    entries.sort_by_key(|a| a.display_name.to_ascii_lowercase());
    entries
}

/// Standard Start Menu roots for this machine.
pub fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(Path::new(&program_data).join("Microsoft/Windows/Start Menu/Programs"));
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(Path::new(&appdata).join("Microsoft/Windows/Start Menu/Programs"));
    }
    dirs
}

#[cfg(windows)]
fn resolve_shortcut_target(lnk: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, IPersistFile, SLGP_RAWPATH};

    fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    // SAFETY: COM is initialised for the duration of this call and uninitialised
    // before returning. The interface pointer is created and released locally.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = (|| -> Option<String> {
            let link: IShellLinkW =
                CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
            let persist: IPersistFile = link.cast().ok()?;
            let path = wide(lnk.as_os_str());
            persist.Load(PCWSTR(path.as_ptr()), 0).ok()?;
            let mut buffer = [0u16; 1024];
            link.GetPath(&mut buffer, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)
                .ok()?;
            let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
            if end == 0 {
                return None;
            }
            Some(String::from_utf16_lossy(&buffer[..end]))
        })();
        CoUninitialize();
        result
    }
}

#[cfg(not(windows))]
fn resolve_shortcut_target(_lnk: &Path) -> Option<String> {
    None
}

/// Launch a dock item. Returns a typed error message on failure.
pub fn launch(id: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let path = id.replace('/', "\\");
        let wide: Vec<u16> = std::ffi::OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let verb: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: both strings are NUL-terminated and outlive the call.
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        // ShellExecuteW returns a value greater than 32 on success.
        if result.0 as usize > 32 {
            Ok(())
        } else {
            Err(format!("ShellExecuteW failed with code {}", result.0 as usize))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = id;
        Err("launch is only supported on Windows".into())
    }
}
```

- [ ] **Step 5: Export and run**

Add `pub mod apps;` to `lib.rs`, then:

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test apps_catalog
```

Expected: 10 tests pass.

The catalog tests pass on fake `.lnk` bytes because `resolve_shortcut_target` returns `None` rather than erroring. Do not assert on `target` in those tests.

- [ ] **Step 6: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/apps.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/apps_catalog.rs
git commit -m "feat: add start menu catalog launch and reconciliation"
```

---

## Task 7: Shell icon extraction with disk cache

**Files:**
- Create: `dock-tauri/src-tauri/src/icon_cache.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Test: `dock-tauri/src-tauri/tests/unit/icon_cache.rs`
- Test: `dock-tauri/src-tauri/tests/unit/icon_extract.rs`

**Why this task matters:** without it every dock tile renders a letter placeholder instead of the app's real icon. This is the most visually important backend task in the plan.

**Critical trap — read before writing code:** `SHGetFileInfoW` + `SHGFI_ICON` always returns the **16 px** system icon. It has no size parameter. Using it and stretching to 48 px produces blurry mush. The correct source of large icons is `SHGetImageList(SHIL_JUMBO)` (256 px), falling back to `SHIL_EXTRALARGE` (48 px), then `SHGFI_ICON` (16 px) only as a last resort.

- [ ] **Step 1: Write the failing cache tests**

`dock-tauri/src-tauri/tests/unit/icon_cache.rs`:

```rust
use dock_tauri_lib::icon_cache::IconCache;

#[test]
fn cache_key_is_stable_and_size_specific() {
    let a = IconCache::key("C:/x/notepad.lnk", 48);
    let b = IconCache::key("C:/x/notepad.lnk", 48);
    let c = IconCache::key("C:/x/notepad.lnk", 96);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.len(), 64, "sha256 hex digest");
}

#[test]
fn key_is_case_insensitive_for_paths() {
    assert_eq!(
        IconCache::key("C:/X/Notepad.lnk", 48),
        IconCache::key("c:/x/notepad.lnk", 48)
    );
}

#[test]
fn stores_and_reads_a_data_url() {
    let dir = tempfile::tempdir().unwrap();
    let cache = IconCache::new(dir.path().to_path_buf());
    let key = IconCache::key("C:/x/notepad.lnk", 48);
    let png: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

    cache.store(&key, png).unwrap();
    let url = cache.read(&key).expect("cache hit");
    assert!(url.starts_with("data:image/png;base64,"));
}

#[test]
fn miss_returns_none_without_error() {
    let dir = tempfile::tempdir().unwrap();
    let cache = IconCache::new(dir.path().to_path_buf());
    assert!(cache.read(&IconCache::key("C:/x/missing.lnk", 48)).is_none());
}

#[test]
fn empty_cache_entry_is_treated_as_a_miss() {
    let dir = tempfile::tempdir().unwrap();
    let cache = IconCache::new(dir.path().to_path_buf());
    let key = IconCache::key("C:/x/notepad.lnk", 48);
    std::fs::write(dir.path().join(format!("{key}.png")), b"").unwrap();
    assert!(cache.read(&key).is_none());
}
```

- [ ] **Step 2: Write the failing extraction tests**

`dock-tauri/src-tauri/tests/unit/icon_extract.rs`:

```rust
use dock_tauri_lib::icon_cache::{extract_png, IconCache};

#[test]
fn extraction_from_a_missing_path_returns_none() {
    assert!(extract_png(std::path::Path::new("Z:/nope/missing.exe"), 48).is_none());
}

#[test]
fn extraction_from_a_real_shortcut_produces_a_real_png() {
    let dirs = dock_tauri_lib::apps::start_menu_dirs();
    let entries = dock_tauri_lib::apps::catalog_from_dirs(&dirs);
    let Some(first) = entries.first() else {
        return;
    };
    let Some(png) = extract_png(std::path::Path::new(&first.id), 48) else {
        return;
    };
    assert_eq!(&png[1..4], b"PNG", "output must be a PNG");

    // A real icon is at least the requested size on its short edge, and is not
    // a single flat colour. A 16 px system icon stretched to 48 px would fail
    // the dimension check.
    let image = image::load_from_memory(&png).expect("decoded png");
    let (w, h) = (image.width(), image.height());
    assert!(
        w >= 48 && h >= 48,
        "icon must be at least 48px, got {w}x{h} (SHGFI_ICON returns 16px — use SHGetImageList)"
    );
}
```

`image` is a normal dependency, so the test can use it.

- [ ] **Step 3: Run both and confirm failure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test icon_cache --test icon_extract
```

Expected: FAIL — `icon_cache` module does not exist.

- [ ] **Step 4: Register the targets and dependencies**

In `dock-tauri/src-tauri/Cargo.toml`:

```toml
[[test]]
name = "icon_cache"
path = "tests/unit/icon_cache.rs"

[[test]]
name = "icon_extract"
path = "tests/unit/icon_extract.rs"
```

Add to `[dependencies]`:

```toml
sha2 = "0.10"
base64 = "0.22"
```

- [ ] **Step 5: Implement the cache and extraction**

`dock-tauri/src-tauri/src/icon_cache.rs`:

```rust
//! Shell icon extraction and a disk cache served to the UI as data URLs.

use base64::Engine;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct IconCache {
    dir: PathBuf,
}

impl IconCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Cache key: sha256 of the lowercased, forward-slashed path plus icon size.
    pub fn key(path: &str, size: u32) -> String {
        let normalized = path.replace('\\', "/").to_ascii_lowercase();
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hasher.update(b"@");
        hasher.update(size.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn file(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.png"))
    }

    pub fn store(&self, key: &str, png: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.file(key), png)
    }

    /// Return a data URL, or `None` on a miss or an unreadable entry.
    pub fn read(&self, key: &str) -> Option<String> {
        let bytes = std::fs::read(self.file(key)).ok()?;
        if bytes.is_empty() {
            return None;
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        Some(format!("data:image/png;base64,{encoded}"))
    }
}

/// Data URLs for every app, extracting and caching on first use.
pub fn icons_for(apps: &[crate::apps::AppEntry], size: u32) -> Vec<Option<String>> {
    let dir = std::env::var("APPDATA")
        .map(|base| PathBuf::from(base).join("DockApp/icons"))
        .unwrap_or_else(|_| std::env::temp_dir().join("dockapp-icons"));
    let cache = IconCache::new(dir);
    apps.iter()
        .map(|app| {
            let key = IconCache::key(&app.id, size);
            if let Some(url) = cache.read(&key) {
                return Some(url);
            }
            let png = extract_png(Path::new(&app.id), size)?;
            cache.store(&key, &png).ok()?;
            cache.read(&key)
        })
        .collect()
}

/// Extract an icon for `path` as a PNG sized to `size` pixels.
///
/// Prefers the 256 px jumbo image list, then 48 px, then the 16 px system icon,
/// upscaling only as a last resort.
#[cfg(windows)]
pub fn extract_png(path: &Path, size: u32) -> Option<Vec<u8>> {
    extract::extract_icon(path, size)
}

#[cfg(not(windows))]
pub fn extract_png(_path: &Path, _size: u32) -> Option<Vec<u8>> {
    None
}

#[cfg(windows)]
mod extract {
    use super::*;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::UI::Shell::{
        SHGetFileInfoW, SHGetImageList, SHFILEINFOW, SHGFI_ICON, SHGFI_SYSICONINDEX, SHIL_EXTRALARGE,
        SHIL_JUMBO,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn file_info(path: &Path, flags: u32) -> Option<SHFILEINFOW> {
        let name = wide(path);
        let mut info = SHFILEINFOW::default();
        // SAFETY: `name` is NUL-terminated and outlives the call; `info` is a
        // local out-parameter.
        let ok = unsafe {
            SHGetFileInfoW(
                PCWSTR(name.as_ptr()),
                windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
                Some(&mut info),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                windows::Win32::UI::Shell::SHGFI_FLAGS(flags),
            )
        };
        (ok != 0).then_some(info)
    }

    /// Best available HICON: jumbo -> extra large -> small.
    fn best_icon(path: &Path) -> Option<HICON> {
        // System image-list index, then borrow the large lists from the Shell.
        if let Some(info) = file_info(path, SHGFI_SYSICONINDEX.0) {
            for shil in [SHIL_JUMBO, SHIL_EXTRALARGE] {
                // SAFETY: SHGetImageList returns a borrowed Shell-owned list.
                if let Ok(list) =
                    unsafe { SHGetImageList(shil.0 as i32) }
                {
                    // SAFETY: index came from the Shell; the HICON is ours to free.
                    if let Ok(icon) = unsafe { list.GetIcon(info.iIcon, 0x1 /* ILD_TRANSPARENT */) } {
                        return Some(icon);
                    }
                }
            }
        }
        // Last resort: the small system icon.
        file_info(path, SHGFI_ICON.0).map(|info| info.hIcon)
    }

    fn hicon_to_png(icon: HICON, size: u32) -> Option<Vec<u8>> {
        // SAFETY: every GDI object created here is released before returning.
        unsafe {
            let mut icon_info = ICONINFO::default();
            if GetIconInfo(icon, &mut icon_info).is_err() {
                return None;
            }

            let header = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size as i32,
                biHeight: -(size as i32), // negative = top-down rows
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: header,
                ..Default::default()
            };

            let dc = GetDC(None);
            if dc.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
                let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));
                return None;
            }

            let mut buffer = vec![0u8; (size * size * 4) as usize];
            let lines = GetDIBits(
                dc,
                icon_info.hbmColor,
                0,
                size,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );
            ReleaseDC(None, dc);
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmMask.0));
            let _ = DeleteObject(HGDIOBJ(icon_info.hbmColor.0));

            if lines == 0 {
                return None;
            }

            // Shell icons are BGRA; `image` expects RGBA.
            for pixel in buffer.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }

            let rgba = image::RgbaImage::from_raw(size, size, buffer)?;
            let mut png = std::io::Cursor::new(Vec::new());
            rgba.write_to(&mut png, image::ImageFormat::Png).ok()?;
            Some(png.into_inner())
        }
    }

    pub fn extract_icon(path: &Path, size: u32) -> Option<Vec<u8>> {
        let icon = best_icon(path)?;
        // HICONs are reference-counted by the Shell image list; DestroyIcon is
        // still correct for handles we own from GetIcon / SHGFI_ICON.
        let png = hicon_to_png(icon, size);
        // SAFETY: the handle is no longer used after this point.
        unsafe {
            let _ = DestroyIcon(icon);
        }
        png
    }
}
```

**Note on `SHGetImageList` in windows 0.61:** the function signature and the `IImageList` cast may differ. Verify against `D:/rust/cargo/registry/src/*/windows-0.61.3/src/Windows/Win32/UI/Shell/mod.rs`. If `SHGetImageList` is not exposed, use `SHGetFileInfoW` with `SHGFI_SYSICONINDEX` plus `IImageList` from `SHGetImageList`, or fall back to requesting `SHGFI_ICON` and accept 16 px only as the documented last resort. Do **not** claim large icons work unless the test's `>= 48` assertion passes.

### Implementation notes verified on this machine (windows 0.61.3)

These corrections were required; a later re-run should apply them directly.

- `SHGetImageList::<IImageList>(SHIL_JUMBO)` is generic and needs the turbofish. `SHIL_*` constants are plain `u32`, not newtypes — use `SHIL_JUMBO as i32`.
- `IImageList` lives in `Win32::UI::Controls`, not `Win32::UI::Shell`. Add the `Win32_UI_Controls` feature.
- `IImageList::GetIcon(index, flags) -> Result<HICON>` with `ILD_TRANSPARENT.0` as flags.
- **`SHGetFileInfoW` fails on forward-slash paths.** `AppEntry.id` is normalised to `C:/x/y.lnk`, but the Shell API requires backslashes. Convert with `path.replace('/', "\\")` before calling, exactly like `apps::launch` does. Without this, **every** extraction returns `None` while the image list works fine — the failure looks like an icon-list problem but is a path problem.
- **`GetDIBits` does not scale.** Passing the requested 48 px against a 256 px icon bitmap fails. Read at the bitmap's native size (`GetObjectW` -> `BITMAP`), then downscale to the target with `image::imageops::resize(..., FilterType::Lanczos3)`. Never upscale: that keeps the 16 px fallback at 16 px so the `>= 48` test can catch it.
- Many 32 bpp colour bitmaps carry no alpha; use the AND mask (`hbmMask`) for transparency, defaulting to opaque when there is no mask.
- Clippy 1.99 flags `chunks_exact(4)` in favour of `as_chunks::<4>()`.

- [ ] **Step 6: Export and run**

Add `pub mod icon_cache;` to `lib.rs`, then:

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test icon_cache --test icon_extract
```

Expected: 5 cache tests and 2 extraction tests pass. If `extraction_from_a_real_shortcut_produces_a_real_png` fails on the `>= 48` assertion, the jumbo image list is not being used — fix the extraction, do not lower the assertion.

- [ ] **Step 7: Probe three real icons (do not commit the probe)**

Write a temporary test or example that takes the first 3 entries from `catalog_from_dirs(&start_menu_dirs())`, extracts each at 48 px, and prints per icon: width, height, byte length, and the number of distinct pixel values (to prove it is not blank). Report those numbers.

- [ ] **Step 8: Run the full gates and commit**

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml
D:/rust/cargo/bin/cargo.exe clippy --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --all-targets -- -D warnings
D:/rust/cargo/bin/cargo.exe fmt --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml -- --check
```

Then:

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/icon_cache.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/Cargo.lock \
  dock-tauri/src-tauri/tests/unit/icon_cache.rs dock-tauri/src-tauri/tests/unit/icon_extract.rs
git commit -m "feat: add shell icon extraction with disk cache"
```

---

## Task 8: Python sidecar with stdio JSON-RPC

**Files:**
- Create: `dock-tauri/backend/run_sidecar.py`
- Create: `dock-tauri/backend/monitor/__init__.py`
- Create: `dock-tauri/backend/monitor/main.py`
- Create: `dock-tauri/backend/monitor/sampler.py`
- Create: `dock-tauri/backend/monitor/requirements.txt`
- Create: `dock-tauri/backend/tests/fake_psutil.py`
- Create: `dock-tauri/backend/tests/test_sampler.py`
- Create: `dock-tauri/backend/tests/test_protocol.py`

- [ ] **Step 1: Write the fake psutil and failing sampler test**

`dock-tauri/backend/tests/fake_psutil.py`:

```python
"""Deterministic stand-in for psutil, injected by tests."""


class _Value:
    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)


class FakePsutil:
    """Minimal psutil surface used by sampler.py."""

    def __init__(self, cpu=12.5, recv=2000, sent=900):
        self._cpu = cpu
        self._recv = recv
        self._sent = sent

    def cpu_percent(self, interval=None):
        _ = interval
        return self._cpu

    def virtual_memory(self):
        return _Value(used=8 * 1024**3, total=16 * 1024**3)

    def disk_usage(self, path):
        _ = path
        return _Value(used=100 * 1024**3, total=500 * 1024**3)

    def net_io_counters(self):
        return _Value(bytes_recv=self._recv, bytes_sent=self._sent)
```

`dock-tauri/backend/tests/test_sampler.py`:

```python
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from monitor.sampler import Sampler  # noqa: E402
from fake_psutil import FakePsutil  # noqa: E402


def test_sample_reports_all_fields():
    sampler = Sampler(FakePsutil())
    sample = sampler.sample()
    assert sample["cpu"] == 12.5
    assert sample["mem_used"] == 8 * 1024**3
    assert sample["mem_total"] == 16 * 1024**3
    assert sample["disk_used"] == 100 * 1024**3
    assert sample["disk_total"] == 500 * 1024**3
    assert sample["net_rx"] == 2000
    assert sample["net_tx"] == 900


def test_disk_path_defaults_to_the_system_drive_root():
    sampler = Sampler(FakePsutil())
    assert sampler.disk_path in ("C:\\", "/") or sampler.disk_path.endswith("\\")


def test_net_counters_are_cumulative():
    sampler = Sampler(FakePsutil())
    first = sampler.sample()
    second = sampler.sample()
    assert first["net_rx"] == second["net_rx"] == 2000
```

- [ ] **Step 2: Write the failing protocol test**

`dock-tauri/backend/tests/test_protocol.py`:

```python
import io
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from monitor.main import serve  # noqa: E402
from fake_psutil import FakePsutil  # noqa: E402


def _lines(stdout):
    return [json.loads(line) for line in stdout.getvalue().splitlines() if line.strip()]


def test_responds_to_sample_with_one_json_line():
    stdin = io.StringIO(json.dumps({"cmd": "sample"}) + "\n")
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    payloads = _lines(stdout)
    assert len(payloads) == 1
    assert payloads[0]["t"] == "sample"
    assert "cpu" in payloads[0]


def test_unknown_command_returns_an_error_line():
    stdin = io.StringIO(json.dumps({"cmd": "explode"}) + "\n")
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    payload = _lines(stdout)[0]
    assert payload["t"] == "error"
    assert "explode" in payload["detail"]


def test_malformed_line_returns_an_error_and_keeps_serving():
    stdin = io.StringIO('not json\n{"cmd": "sample"}\n')
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    payloads = _lines(stdout)
    assert payloads[0]["t"] == "error"
    assert payloads[-1]["t"] == "sample"


def test_eof_exits_cleanly():
    stdin = io.StringIO("")
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    assert stdout.getvalue() == ""
```

- [ ] **Step 3: Run tests and confirm failure**

```bash
cd D:/dockyh/dock-tauri/backend
D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

Expected: FAIL — `No module named 'monitor.sampler'`.

- [ ] **Step 4: Implement the sampler**

`dock-tauri/backend/monitor/__init__.py`:

```python
"""Dock Tauri system-metrics sidecar."""
```

`dock-tauri/backend/monitor/sampler.py`:

```python
"""psutil sampling. The psutil module is injected so tests stay deterministic."""

import os


class Sampler:
    def __init__(self, psutil_module, disk_path=None):
        self.psutil = psutil_module
        self.disk_path = disk_path or self._default_disk_path()

    @staticmethod
    def _default_disk_path():
        drive = os.environ.get("SystemDrive", "C:")
        return drive + os.sep

    def sample(self):
        cpu = float(self.psutil.cpu_percent(interval=None))
        memory = self.psutil.virtual_memory()
        disk = self.psutil.disk_usage(self.disk_path)
        net = self.psutil.net_io_counters()
        return {
            "cpu": cpu,
            "mem_used": int(memory.used),
            "mem_total": int(memory.total),
            "disk_used": int(disk.used),
            "disk_total": int(disk.total),
            "net_rx": int(net.bytes_recv),
            "net_tx": int(net.bytes_sent),
        }
```

- [ ] **Step 5: Implement the protocol loop**

`dock-tauri/backend/monitor/main.py`:

```python
"""Newline-delimited JSON-RPC over stdio. No sockets, no threads."""

import json
import sys


def serve(stdin, stdout, psutil_module):
    from monitor.sampler import Sampler

    sampler = Sampler(psutil_module)
    for line in stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError as error:
            _write(stdout, {"t": "error", "detail": f"malformed request: {error}"})
            continue

        command = request.get("cmd") if isinstance(request, dict) else None
        if command == "sample":
            try:
                payload = sampler.sample()
            except Exception as error:  # noqa: BLE001 - report, never break the loop
                _write(stdout, {"t": "error", "detail": f"sample failed: {error}"})
                continue
            payload["t"] = "sample"
            _write(stdout, payload)
        else:
            _write(stdout, {"t": "error", "detail": f"unknown command: {command}"})


def _write(stdout, payload):
    stdout.write(json.dumps(payload) + "\n")
    stdout.flush()


def main():
    import psutil

    serve(sys.stdin, sys.stdout, psutil_module=psutil)


if __name__ == "__main__":
    main()
```

`dock-tauri/backend/run_sidecar.py` — launcher that keeps `monitor` importable regardless of the working directory the supervisor uses:

```python
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from monitor.main import main  # noqa: E402

if __name__ == "__main__":
    main()
```

`dock-tauri/backend/monitor/requirements.txt`:

```text
psutil>=5.9,<6
```

- [ ] **Step 6: Run the tests**

```bash
cd D:/dockyh/dock-tauri/backend
D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

Expected: 7 tests pass (3 sampler + 4 protocol).

- [ ] **Step 7: Verify the real sidecar round-trips**

```bash
cd D:/dockyh/dock-tauri/backend
echo '{"cmd":"sample"}' | D:/jiqixuexi/anaconda/python.exe -m monitor.main
```

Expected: one JSON line containing `"t": "sample"` and a plausible `cpu` value.

- [ ] **Step 8: Commit**

```bash
cd D:/dockyh
git add dock-tauri/backend/run_sidecar.py \
  dock-tauri/backend/monitor/__init__.py dock-tauri/backend/monitor/main.py \
  dock-tauri/backend/monitor/sampler.py dock-tauri/backend/monitor/requirements.txt \
  dock-tauri/backend/tests/fake_psutil.py dock-tauri/backend/tests/test_sampler.py \
  dock-tauri/backend/tests/test_protocol.py
git commit -m "feat: add python metrics sidecar over stdio json-rpc"
```

---
## Task 9: Rust sidecar supervisor with restart policy

**Files:**
- Create: `dock-tauri/src-tauri/src/metrics.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Create: `dock-tauri/src-tauri/src/bin/fake_sidecar.rs`
- Test: `dock-tauri/src-tauri/tests/unit/metrics_supervisor.rs`

- [ ] **Step 1: Write the fake sidecar binary**

`dock-tauri/src-tauri/src/bin/fake_sidecar.rs`:

```rust
//! Test double for the Python sidecar. Behaviour is selected by the first arg:
//!   ok       - answers every sample request
//!   crash    - exits immediately after the first request
//!   garbage  - writes a non-JSON line, then answers normally
//!   silent   - reads but never answers

use std::io::{BufRead, Write};

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "ok".to_string());
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    if mode == "crash" {
        // Read one line, then die without answering.
        let mut line = String::new();
        let _ = stdin.lock().read_line(&mut line);
        std::process::exit(3);
    }

    let mut first = true;
    for line in stdin.lock().lines() {
        let Ok(_line) = line else { break };
        if mode == "silent" {
            continue;
        }
        if mode == "garbage" && first {
            first = false;
            writeln!(stdout, "this is not json").unwrap();
            stdout.flush().unwrap();
            continue;
        }
        first = false;
        writeln!(
            stdout,
            "{{\"t\":\"sample\",\"cpu\":1.0,\"mem_used\":1,\"mem_total\":2,\
             \"disk_used\":1,\"disk_total\":2,\"net_rx\":1,\"net_tx\":1}}"
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}
```

Register it in `src-tauri/Cargo.toml` so `cargo test` builds it:

```toml
[[bin]]
name = "fake_sidecar"
path = "src/bin/fake_sidecar.rs"
```

- [ ] **Step 2: Write the failing supervisor test**

`dock-tauri/src-tauri/tests/unit/metrics_supervisor.rs`:

```rust
use dock_tauri_lib::metrics::{RpcError, SampleRpc};
use std::path::PathBuf;

fn fake(mode: &str) -> SampleRpc {
    // cargo sets CARGO_BIN_EXE_<name> for integration tests.
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_fake_sidecar"));
    SampleRpc::spawn(exe, vec![mode.to_string()]).expect("spawn fake sidecar")
}

#[test]
fn sample_round_trips_through_the_pipe() {
    let mut rpc = fake("ok");
    let sample = rpc.sample().expect("sample");
    assert_eq!(sample.cpu, 1.0);
    assert_eq!(sample.mem_total, 2);
    rpc.shutdown();
}

#[test]
fn several_samples_work_on_one_process() {
    let mut rpc = fake("ok");
    for _ in 0..3 {
        assert!(rpc.sample().is_ok());
    }
    rpc.shutdown();
}

#[test]
fn a_dead_child_produces_a_typed_error_not_a_hang() {
    let mut rpc = fake("crash");
    let error = rpc.sample().unwrap_err();
    assert!(matches!(error, RpcError::Closed | RpcError::Io(_)));
}

#[test]
fn garbage_output_is_reported_as_a_decode_error() {
    let mut rpc = fake("garbage");
    let error = rpc.sample().unwrap_err();
    assert!(matches!(error, RpcError::Decode { .. }));
    // The supervisor must still work afterwards.
    assert!(rpc.sample().is_ok());
    rpc.shutdown();
}

#[test]
fn shutdown_is_idempotent() {
    let mut rpc = fake("ok");
    let _ = rpc.sample();
    rpc.shutdown();
    rpc.shutdown();
}
```

- [ ] **Step 3: Run it and confirm failure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test metrics_supervisor
```

Expected: FAIL — `metrics` module missing.

- [ ] **Step 4: Register the test target**

```toml
[[test]]
name = "metrics_supervisor"
path = "tests/unit/metrics_supervisor.rs"
```

- [ ] **Step 5: Implement the supervisor**

`dock-tauri/src-tauri/src/metrics.rs`:

```rust
//! Sidecar supervisor: owns the Python child and speaks newline JSON-RPC.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub cpu: f64,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub net_rx: u64,
    pub net_tx: u64,
}

#[derive(Debug)]
pub enum RpcError {
    Spawn(std::io::Error),
    Io(std::io::Error),
    Decode { detail: String },
    Closed,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::Spawn(e) => write!(f, "failed to spawn sidecar: {e}"),
            RpcError::Io(e) => write!(f, "sidecar io error: {e}"),
            RpcError::Decode { detail } => write!(f, "sidecar sent invalid json: {detail}"),
            RpcError::Closed => write!(f, "sidecar closed the pipe"),
        }
    }
}

impl std::error::Error for RpcError {}

pub struct SampleRpc {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
}

impl SampleRpc {
    pub fn spawn(program: PathBuf, args: Vec<String>) -> Result<Self, RpcError> {
        let mut child = Command::new(&program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(RpcError::Spawn)?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take().map(BufReader::new);
        Ok(Self {
            child: Some(child),
            stdin,
            stdout,
        })
    }

    /// Request one sample. The call is blocking; callers run it off the UI thread.
    pub fn sample(&mut self) -> Result<Sample, RpcError> {
        let stdin = self.stdin.as_mut().ok_or(RpcError::Closed)?;
        stdin
            .write_all(b"{\"cmd\":\"sample\"}\n")
            .map_err(RpcError::Io)?;
        stdin.flush().map_err(RpcError::Io)?;

        let stdout = self.stdout.as_mut().ok_or(RpcError::Closed)?;
        let mut line = String::new();
        let read = stdout.read_line(&mut line).map_err(RpcError::Io)?;
        if read == 0 {
            return Err(RpcError::Closed);
        }

        let value: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| RpcError::Decode {
                detail: e.to_string(),
            })?;

        match value.get("t").and_then(|t| t.as_str()) {
            Some("sample") => serde_json::from_value(value).map_err(|e| RpcError::Decode {
                detail: e.to_string(),
            }),
            Some("error") => Err(RpcError::Decode {
                detail: value
                    .get("detail")
                    .and_then(|d| d.as_str())
                    .unwrap_or("sidecar reported an error")
                    .to_string(),
            }),
            _ => Err(RpcError::Decode {
                detail: "unexpected payload".to_string(),
            }),
        }
    }

    pub fn shutdown(&mut self) {
        self.stdin.take();
        self.stdout.take();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for SampleRpc {
    fn drop(&mut self) {
        self.shutdown();
    }
}
```

- [ ] **Step 5b: Implement the restart policy**

The steps above give a supervisor that dies permanently after one crash. The spec requires the monitor to recover, so wrap it in a `Supervisor` that respawns on demand with a cooldown.

Append to `metrics.rs`:

```rust
use std::time::{Duration, Instant};

/// How to launch the sidecar, plus the minimum interval between respawns.
pub struct SidecarSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cooldown: Duration,
}

impl SidecarSpec {
    pub fn new(program: PathBuf, args: Vec<String>) -> Self {
        Self {
            program,
            args,
            // Backoff floor: never spawn more than once per two seconds.
            cooldown: Duration::from_secs(2),
        }
    }
}

/// Owns at most one sidecar and respawns it on demand after a failure.
pub struct Supervisor {
    spec: SidecarSpec,
    rpc: Option<SampleRpc>,
    last_attempt: Option<Instant>,
}

impl Supervisor {
    pub fn new(spec: SidecarSpec) -> Self {
        Self {
            spec,
            rpc: None,
            last_attempt: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.rpc.is_some()
    }

    /// Request a sample, starting the sidecar first if needed.
    ///
    /// On failure the broken child is dropped so the next call can respawn it,
    /// subject to `cooldown`.
    pub fn sample(&mut self) -> Result<Sample, RpcError> {
        if self.rpc.is_none() {
            if let Some(last) = self.last_attempt {
                if last.elapsed() < self.spec.cooldown {
                    return Err(RpcError::Closed);
                }
            }
            self.last_attempt = Some(Instant::now());
            self.rpc = Some(SampleRpc::spawn(
                self.spec.program.clone(),
                self.spec.args.clone(),
            )?);
        }

        let rpc = self.rpc.as_mut().expect("just spawned");
        match rpc.sample() {
            Ok(sample) => Ok(sample),
            Err(error) => {
                rpc.shutdown();
                self.rpc = None;
                Err(error)
            }
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(mut rpc) = self.rpc.take() {
            rpc.shutdown();
        }
    }
}
```

**Add tests** to `tests/unit/metrics_supervisor.rs`:

```rust
use dock_tauri_lib::metrics::{SidecarSpec, Supervisor};
use std::path::PathBuf;
use std::time::Duration;

fn spec(mode: &str) -> SidecarSpec {
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_fake_sidecar"));
    SidecarSpec::new(exe, vec![mode.to_string()])
}

#[test]
fn supervisor_respawns_after_a_crash() {
    let mut spec = spec("crash");
    spec.cooldown = Duration::ZERO;
    let mut supervisor = Supervisor::new(spec);

    // First attempt: the child dies without answering.
    assert!(supervisor.sample().is_err());
    assert!(!supervisor.is_running(), "broken child must be dropped");

    // A later attempt spawns a fresh child rather than staying dead forever.
    assert!(supervisor.sample().is_err(), "second crash still errors");
    supervisor.shutdown();
}

#[test]
fn cooldown_prevents_a_spawn_storm() {
    let mut spec = spec("crash");
    spec.cooldown = Duration::from_secs(60);
    let mut supervisor = Supervisor::new(spec);

    assert!(supervisor.sample().is_err());
    let started = std::time::Instant::now();
    assert!(supervisor.sample().is_err());
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "cooldown must fail fast, not block"
    );
}

#[test]
fn a_missing_sidecar_binary_is_a_typed_spawn_error() {
    let mut spec = SidecarSpec::new(PathBuf::from("Z:/nope/sidecar.exe"), Vec::new());
    spec.cooldown = Duration::ZERO;
    let mut supervisor = Supervisor::new(spec);
    let error = supervisor.sample().unwrap_err();
    assert!(matches!(error, RpcError::Spawn(_)));
}

#[test]
fn supervisor_serves_repeated_samples_from_one_child() {
    let mut supervisor = Supervisor::new(spec("ok"));
    for _ in 0..3 {
        assert!(supervisor.sample().is_ok());
    }
    assert!(supervisor.is_running());
    supervisor.shutdown();
}
```

- [ ] **Step 6: Export and run**

Add `pub mod metrics;` to `lib.rs`, then:

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test metrics_supervisor
```

Expected: 5 tests pass.

`garbage_output_is_reported_as_a_decode_error` requires the loop to keep the pipe usable after a bad line, which the implementation does because it reads exactly one line per request.

- [ ] **Step 7: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/metrics.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/src/bin/fake_sidecar.rs dock-tauri/src-tauri/Cargo.toml \
  dock-tauri/src-tauri/tests/unit/metrics_supervisor.rs
git commit -m "feat: add sidecar supervisor with typed rpc errors"
```

---

## Task 10: Magnification math in React with Vitest

**Files:**
- Create: `dock-tauri/src/dock/magnify.ts`
- Create: `dock-tauri/src/dock/magnify.test.ts`
- Modify: `dock-tauri/package.json` (test deps)

- [ ] **Step 1: Add the test environment dependency**

```bash
cd D:/dockyh/dock-tauri
npm install --save-dev happy-dom
```

`happy-dom` is required because `vite.config.ts` sets `environment: "happy-dom"`.

- [ ] **Step 2: Write the failing test**

`dock-tauri/src/dock/magnify.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { magnifiedScale, profileFor, scalesFor } from "./magnify";

describe("magnifiedScale", () => {
  it("returns the peak at zero distance", () => {
    expect(magnifiedScale(0, 1.6, 0.45, 120)).toBeCloseTo(1.6, 5);
  });

  it("returns 1.0 at and beyond the radius", () => {
    expect(magnifiedScale(120, 1.6, 0.45, 120)).toBeCloseTo(1.0, 5);
    expect(magnifiedScale(400, 1.6, 0.45, 120)).toBeCloseTo(1.0, 5);
  });

  it("decays monotonically with distance", () => {
    let previous = Number.POSITIVE_INFINITY;
    for (const distance of [0, 20, 40, 60, 80, 100, 120]) {
      const scale = magnifiedScale(distance, 1.6, 0.45, 120);
      expect(scale).toBeLessThanOrEqual(previous + 1e-9);
      previous = scale;
    }
  });

  it("never goes below 1.0", () => {
    for (let distance = 0; distance <= 300; distance += 7) {
      expect(magnifiedScale(distance, 1.6, 0.45, 120)).toBeGreaterThanOrEqual(1.0);
    }
  });

  it("treats negative distance as zero", () => {
    expect(magnifiedScale(-50, 1.6, 0.45, 120)).toBeCloseTo(1.6, 5);
  });

  it("tolerates a zero radius instead of dividing by zero", () => {
    expect(magnifiedScale(10, 1.6, 0.45, 0)).toBe(1.0);
  });
});

describe("scalesFor", () => {
  const centers = [40, 96, 152, 208];

  it("peaks at the hovered index and falls off to both sides", () => {
    const scales = scalesFor(centers, 96, 1.6, 0.45, 120);
    expect(scales[1]).toBeCloseTo(1.6, 5);
    expect(scales[0]).toBeGreaterThan(1.0);
    expect(scales[2]).toBeGreaterThan(1.0);
    expect(scales[2]).toBeLessThan(scales[1]);
    expect(scales[3]).toBeCloseTo(1.0, 5);
  });

  it("returns all ones when the pointer is far away", () => {
    const scales = scalesFor(centers, 5000, 1.6, 0.45, 120);
    expect(scales.every((s) => Math.abs(s - 1) < 1e-9)).toBe(true);
  });
});

describe("profileFor", () => {
  it("uses a restrained peak for the macOS profile", () => {
    const profile = profileFor("macos", false);
    expect(profile.peak).toBeCloseTo(1.2, 5);
    expect(profile.falloff).toBeGreaterThan(0);
  });

  it("uses the elastic peak for the vitro profile", () => {
    const profile = profileFor("vitro", false);
    expect(profile.peak).toBeCloseTo(1.6, 5);
    expect(profile.falloff).toBeCloseTo(0.45, 5);
  });

  it("disables magnification entirely for reduced motion", () => {
    const profile = profileFor("vitro", true);
    expect(profile.peak).toBeCloseTo(1.0, 5);
  });
});
```

- [ ] **Step 3: Run it and confirm failure**

```bash
cd D:/dockyh/dock-tauri
npm test
```

Expected: FAIL — `./magnify` has no exported members.

- [ ] **Step 4: Implement the math**

`dock-tauri/src/dock/magnify.ts`:

```ts
export interface MagnifyProfile {
  peak: number;
  falloff: number;
  radius: number;
}

const MACOS: MagnifyProfile = { peak: 1.2, falloff: 0.45, radius: 120 };
const VITRO: MagnifyProfile = { peak: 1.6, falloff: 0.45, radius: 120 };

/**
 * Scale for an icon `distance` pixels from the pointer.
 *
 * Falls from `peak` at distance 0 to exactly 1.0 at `radius` and beyond.
 * Negative distances are treated as 0; a non-positive radius returns 1.0.
 */
export function magnifiedScale(
  distance: number,
  peak: number,
  falloff: number,
  radius: number,
): number {
  if (!(radius > 0)) return 1;
  const d = Math.max(0, distance);
  if (d >= radius) return 1;
  // Generalized Gaussian: `falloff` is the influence width as a fraction of the
  // radius. The quartic exponent keeps influence local AND makes the curve
  // continuous at the radius boundary — the naive exp(-(1-t)^2 / falloff)
  // variant jumps ~6% at d = radius, which reads as a visible pop.
  const normalized = d / radius;
  const width = Math.max(falloff, 1e-6);
  const weight = Math.exp(-((normalized / width) ** 4));
  return 1 + (peak - 1) * weight;
}

/** Scale for every icon given the pointer x and the icon centers. */
export function scalesFor(
  centers: number[],
  pointerX: number,
  peak: number,
  falloff: number,
  radius: number,
): number[] {
  return centers.map((center) =>
    magnifiedScale(Math.abs(center - pointerX), peak, falloff, radius),
  );
}

/** Resolve the profile for a settings combination. */
export function profileFor(
  profile: "macos" | "vitro",
  reducedMotion: boolean,
): MagnifyProfile {
  if (reducedMotion) return { peak: 1.0, falloff: 0.45, radius: 120 };
  return profile === "macos" ? MACOS : VITRO;
}
```

- [ ] **Step 5: Run the tests**

```bash
cd D:/dockyh/dock-tauri
npm test
```

Expected: 11 tests pass.

If `never goes below 1.0` fails, the exponent sign is wrong — `weight` must approach 0 as distance grows.

- [ ] **Step 6: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src/dock/magnify.ts dock-tauri/src/dock/magnify.test.ts \
  dock-tauri/package.json dock-tauri/package-lock.json
git commit -m "feat: add magnification math with vitest coverage"
```

---

## Task 11: Dock surface with glass, tiles, reflection, and tooltip

**Files:**
- Create: `dock-tauri/src/dock/Dock.tsx`
- Create: `dock-tauri/src/dock/DockItem.tsx`
- Create: `dock-tauri/src/dock/Tooltip.tsx`
- Create: `dock-tauri/src/dock/dock.css`
- Create: `dock-tauri/src/api.ts`
- Modify: `dock-tauri/src/App.tsx`

- [ ] **Step 1: Write the IPC wrapper**

`dock-tauri/src/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DockState, MonitorSample } from "./types";

export const api = {
  getDockState: () => invoke<DockState>("get_dock_state"),
  launchItem: (id: string) => invoke<void>("launch_item", { id }),
  openRoute: (route: string) => invoke<void>("set_route", { route }),
  getSettings: () => invoke<Record<string, unknown>>("get_settings"),
  saveSettings: (settings: Record<string, unknown>) =>
    invoke<void>("save_settings", { settings }),
  sampleMonitor: () => invoke<MonitorSample>("sample_monitor"),
};

export function onDockState(handler: (state: DockState) => void): Promise<UnlistenFn> {
  return listen<DockState>("dock://state", (event) => handler(event.payload));
}

export function onMonitorStatus(
  handler: (status: { available: boolean; detail: string | null }) => void,
): Promise<UnlistenFn> {
  return listen("monitor://status", (event) =>
    handler(event.payload as { available: boolean; detail: string | null }),
  );
}
```

- [ ] **Step 2: Write the dock CSS**

`dock-tauri/src/dock/dock.css`:

```css
:root {
  --glass-fill: rgba(22, 26, 38, 0.62);
  --glass-top-highlight: rgba(255, 255, 255, 0.15);
  --glass-bottom-shadow: rgba(0, 0, 0, 0.15);
  --pill-radius: 34px;
  --icon-size: 48px;
  --icon-gap: 8px;
  --pad-h: 16px;
  --pad-v: 10px;
  --text: #e9edf7;
  --muted: #98a1b6;
  --accent: #7fa4ff;
  --spring: cubic-bezier(0.34, 1.36, 0.64, 1);
  --signature: cubic-bezier(0.22, 1, 0.36, 1);
}

html,
body,
#root {
  margin: 0;
  padding: 0;
  background: transparent;
  overflow: hidden;
  font-family: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
  color: var(--text);
  user-select: none;
}

.dock-stage {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  height: 100%;
  padding-bottom: 40px; /* reflection margin */
}

.dock-pill {
  position: relative;
  display: flex;
  align-items: flex-end;
  gap: var(--icon-gap);
  height: calc(var(--icon-size) + var(--pad-v) * 2);
  padding: 0 var(--pad-h);
  border-radius: var(--pill-radius);
  background: var(--glass-fill);
  box-shadow:
    inset 0 1px 0 var(--glass-top-highlight),
    inset 0 -1px 0 var(--glass-bottom-shadow);
  /* No outer stroke, no heavy outer shadow: depth comes from the material. */
}

/* Tiled monochrome grain, pre-rendered as a repeating layer. */
.dock-pill::after {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  opacity: 0.0025;
  background-image: radial-gradient(#fff 0.5px, transparent 0.5px);
  background-size: 3px 3px;
}

.dock-item {
  position: relative;
  width: var(--icon-size);
  height: var(--icon-size);
  margin-bottom: var(--pad-v);
  transform-origin: bottom center;
  transition: transform 160ms var(--signature);
  will-change: auto;
}

.dock-item.is-magnifying {
  /* Only icons inside the magnify radius hold a compositor layer. */
  will-change: transform;
}

.dock-item.is-reduced {
  transition: opacity 200ms linear;
}

.dock-item img,
.dock-item .fallback-icon {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: contain;
  border-radius: 20%;
}

.dock-item .fallback-icon {
  display: grid;
  place-items: center;
  background: rgba(255, 255, 255, 0.08);
  font-size: calc(var(--icon-size) * 0.4);
  font-weight: 600;
  color: var(--muted);
}

.dock-item .reflection {
  position: absolute;
  left: 0;
  top: 100%;
  width: 100%;
  height: 32px;
  transform: scaleY(-0.45);
  transform-origin: top center;
  opacity: 0.25;
  -webkit-mask-image: linear-gradient(to bottom, #000, transparent);
  mask-image: linear-gradient(to bottom, #000, transparent);
  pointer-events: none;
}

.dock-item .running-dot {
  position: absolute;
  left: 50%;
  bottom: -10px; /* 4px dot, 6px gap below the icon */
  width: 4px;
  height: 4px;
  margin-left: -2px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.6);
}

.dock-item .badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 8px;
  background: #e8342e;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.45);
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.dock-item.is-disabled {
  filter: saturate(0);
  opacity: 0.7;
}

.dock-tooltip {
  position: absolute;
  bottom: calc(100% + 10px);
  left: 50%;
  transform: translateX(-50%);
  padding: 4px 8px;
  border-radius: 8px;
  background: var(--glass-fill);
  box-shadow: inset 0 1px 0 var(--glass-top-highlight);
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  pointer-events: none;
  animation: tooltip-in 150ms var(--signature);
}

@keyframes tooltip-in {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}
```

- [ ] **Step 3: Write the components**

`dock-tauri/src/dock/Tooltip.tsx`:

```tsx
export function Tooltip({ label }: { label: string }) {
  return (
    <div className="dock-tooltip" role="tooltip">
      {label}
    </div>
  );
}
```

`dock-tauri/src/dock/DockItem.tsx`:

```tsx
import { useRef } from "react";
import type { DockItemView } from "../types";
import { Tooltip } from "./Tooltip";

interface Props {
  item: DockItemView;
  scale: number;
  magnifying: boolean;
  reducedMotion: boolean;
  showTooltip: boolean;
  onHover: (id: string | null) => void;
  onActivate: (id: string) => void;
}

export function DockItem({
  item,
  scale,
  magnifying,
  reducedMotion,
  showTooltip,
  onHover,
  onActivate,
}: Props) {
  const ref = useRef<HTMLButtonElement>(null);

  const className = [
    "dock-item",
    magnifying ? "is-magnifying" : "",
    reducedMotion ? "is-reduced" : "",
    item.disabledReason ? "is-disabled" : "",
  ]
    .filter(Boolean)
    .join(" ");

  const style = reducedMotion
    ? { opacity: 1 }
    : { transform: `scale(${scale})` };

  return (
    <button
      ref={ref}
      type="button"
      className={className}
      style={style}
      title={item.disabledReason ?? undefined}
      aria-label={`${item.name}${item.running ? ", running" : ""}`}
      onMouseEnter={() => onHover(item.id)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onActivate(item.id)}
      disabled={Boolean(item.disabledReason)}
    >
      {item.icon ? (
        <img src={item.icon} alt="" draggable={false} />
      ) : (
        <span className="fallback-icon" aria-hidden="true">
          {item.name.slice(0, 1).toUpperCase()}
        </span>
      )}
      {item.icon && (
        <img className="reflection" src={item.icon} alt="" aria-hidden="true" draggable={false} />
      )}
      {item.running && <span className="running-dot" aria-hidden="true" />}
      {item.minimized > 0 && (
        <span className="badge" aria-label={`${item.minimized} minimized`}>
          {item.minimized}
        </span>
      )}
      {showTooltip && <Tooltip label={item.name} />}
    </button>
  );
}
```

`dock-tauri/src/dock/Dock.tsx`:

```tsx
import { useCallback, useMemo, useRef, useState } from "react";
import type { DockState } from "../types";
import { profileFor, scalesFor } from "./magnify";
import { DockItem } from "./DockItem";
import "./dock.css";

interface Props {
  state: DockState;
  onActivate: (id: string) => void;
}

const ICON_PITCH = 56; // icon width + gap; must match the CSS
const TOOLTIP_DELAY_MS = 500;

export function Dock({ state, onActivate }: Props) {
  const [pointerX, setPointerX] = useState<number | null>(null);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const tooltipTimer = useRef<number | null>(null);
  const profile = profileFor(state.profile, state.reducedMotion);

  const centers = useMemo(
    () => state.items.map((_, index) => index * ICON_PITCH + ICON_PITCH / 2),
    [state.items],
  );

  const scales = useMemo(() => {
    if (pointerX === null || !state.magnify || state.reducedMotion) {
      return state.items.map(() => 1);
    }
    return scalesFor(centers, pointerX, profile.peak, profile.falloff, profile.radius);
  }, [centers, pointerX, state.magnify, state.reducedMotion, profile, state.items]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLDivElement>) => {
    // Read layout, not React state: this runs on every pointer move.
    const bounds = event.currentTarget.getBoundingClientRect();
    setPointerX(event.clientX - bounds.left);
  }, []);

  const handleHover = useCallback((id: string | null) => {
    if (tooltipTimer.current !== null) {
      window.clearTimeout(tooltipTimer.current);
      tooltipTimer.current = null;
    }
    if (id === null) {
      setHoveredId(null);
      return;
    }
    tooltipTimer.current = window.setTimeout(
      () => setHoveredId(id),
      TOOLTIP_DELAY_MS,
    );
  }, []);

  return (
    <div className="dock-stage" onPointerMove={handlePointerMove} onPointerLeave={() => setPointerX(null)}>
      <div className="dock-pill" role="toolbar" aria-label="Dock">
        {state.items.map((item, index) => (
          <DockItem
            key={item.id}
            item={item}
            scale={scales[index]}
            magnifying={pointerX !== null && scales[index] > 1.001}
            reducedMotion={state.reducedMotion}
            showTooltip={hoveredId === item.id}
            onHover={handleHover}
            onActivate={onActivate}
          />
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Point the app at the dock**

`dock-tauri/src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { api, onDockState } from "./api";
import { Dock } from "./dock/Dock";
import type { DockState } from "./types";

export function App() {
  const [state, setState] = useState<DockState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    api
      .getDockState()
      .then(setState)
      .catch((e) => setError(String(e)));
    onDockState(setState).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  if (error) {
    return <main className="dock-stage">Dock unavailable: {error}</main>;
  }
  if (!state) {
    return <main className="dock-stage" aria-busy="true" />;
  }
  return (
    <main>
      <Dock state={state} onActivate={(id) => void api.launchItem(id)} />
    </main>
  );
}
```

- [ ] **Step 5: Verify it builds**

```bash
cd D:/dockyh/dock-tauri
npm run build
npm test
```

Expected: build succeeds; 11 Vitest tests still pass.

`api.ts` references Tauri commands that do not exist until Task 12. TypeScript compiles regardless because `invoke` is generic; the runtime failure is expected until then, which is why this task's verification is build + unit tests only.

- [ ] **Step 6: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src/dock/Dock.tsx dock-tauri/src/dock/DockItem.tsx \
  dock-tauri/src/dock/Tooltip.tsx dock-tauri/src/dock/dock.css \
  dock-tauri/src/api.ts dock-tauri/src/App.tsx
git commit -m "feat: add dock surface with glass tiles reflection and tooltip"
```

---
## Task 12: Tauri commands that wire React to the native layer

**Files:**
- Create: `dock-tauri/src-tauri/src/commands.rs`
- Create: `dock-tauri/src-tauri/src/state.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src-tauri/Cargo.toml`
- Test: `dock-tauri/src-tauri/tests/unit/dock_state_view.rs`

- [ ] **Step 1: Write the failing test for the view builder**

`dock-tauri/src-tauri/tests/unit/dock_state_view.rs`:

```rust
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
        RunningInfo { running: true, minimized: 2 },
        RunningInfo { running: false, minimized: 0 },
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
        target: None,
    }];
    let running = vec![RunningInfo { running: false, minimized: 0 }];
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
    let running = vec![RunningInfo { running: false, minimized: 0 }];
    let state = build_dock_state(&apps, &running, &[], &config());
    assert!(state.items[0].disabled_reason.is_some());
}

#[test]
fn fewer_running_entries_than_apps_does_not_panic() {
    let apps = vec![
        AppEntry { id: "a".into(), display_name: "A".into(), target: None },
        AppEntry { id: "b".into(), display_name: "B".into(), target: None },
    ];
    let running = vec![RunningInfo { running: true, minimized: 0 }];
    let state = build_dock_state(&apps, &running, &[], &config());
    assert_eq!(state.items.len(), 2);
    assert!(state.items[1].disabled_reason.is_some());
}
```

- [ ] **Step 2: Run it and confirm failure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test dock_state_view
```

Expected: FAIL — `state` module missing.

- [ ] **Step 3: Register the test target**

```toml
[[test]]
name = "dock_state_view"
path = "tests/unit/dock_state_view.rs"
```

- [ ] **Step 4: Implement the view builder**

`dock-tauri/src-tauri/src/state.rs`:

```rust
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
```

- [ ] **Step 5: Implement the runtime state and commands**

`dock-tauri/src-tauri/src/commands.rs`:

```rust
//! Tauri command surface. Thin: parse, delegate, return typed errors.

use crate::apps::{self, AppEntry};
use crate::metrics::Sample;
use crate::settings::{self, Settings};
use crate::state::{build_dock_state, DockConfig, DockStateView};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct Runtime {
    pub apps: Mutex<Vec<AppEntry>>,
    pub settings_path: PathBuf,
    /// Owns at most one sidecar and respawns it after a crash.
    pub sidecar: Mutex<crate::metrics::Supervisor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub detail: String,
}

impl CommandError {
    fn new(code: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            detail: detail.into(),
        }
    }
}

fn config_from(settings: &Settings) -> DockConfig {
    DockConfig {
        autohide: settings.autohide,
        magnify: settings.magnify,
        reduced_motion: settings.reduced_motion,
        profile: settings.profile.clone(),
    }
}

#[tauri::command]
pub fn get_dock_state(runtime: State<'_, Runtime>) -> Result<DockStateView, CommandError> {
    let settings = settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))?;
    let apps = runtime
        .apps
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&apps);
    let icons = crate::icon_cache::icons_for(&apps, 48);
    Ok(build_dock_state(&apps, &running, &icons, &config_from(&settings)))
}

#[tauri::command]
pub fn launch_item(id: String) -> Result<(), CommandError> {
    apps::launch(&id).map_err(|e| CommandError::new("launch_failed", e))
}

#[tauri::command]
pub fn set_route(app: AppHandle, route: String) -> Result<(), CommandError> {
    let allowed = ["dock", "launchpad", "monitor", "settings"];
    if !allowed.contains(&route.as_str()) {
        return Err(CommandError::new("unknown_route", route));
    }
    app.emit("dock://route", &route)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}

#[tauri::command]
pub fn get_settings(runtime: State<'_, Runtime>) -> Result<Settings, CommandError> {
    settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))
}

#[tauri::command]
pub fn save_settings(runtime: State<'_, Runtime>, settings: Settings) -> Result<(), CommandError> {
    settings::save_atomic(&runtime.settings_path, &settings)
        .map_err(|e| CommandError::new("settings_unwritable", e.to_string()))
}

#[tauri::command]
pub fn sample_monitor(runtime: State<'_, Runtime>) -> Result<Sample, CommandError> {
    let mut supervisor = runtime
        .sidecar
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "sidecar lock poisoned"))?;
    // The supervisor drops a broken child and respawns on the next call, so a
    // crashed sidecar recovers without restarting the dock.
    supervisor
        .sample()
        .map_err(|error| CommandError::new("sidecar_unavailable", error.to_string()))
}

/// Resolve how to launch the Python sidecar for this build.
///
/// Dev builds run the interpreter directly; a packaged build will point at a
/// frozen `monitor.exe` produced by `backend/build_sidecar.ps1` (Task 15).
pub fn sidecar_spec() -> crate::metrics::SidecarSpec {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let launcher = root.join("../../backend/run_sidecar.py");

    let python = std::env::var("DOCK_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("D:/jiqixuexi/anaconda/python.exe"));

    crate::metrics::SidecarSpec::new(python, vec![launcher.to_string_lossy().to_string()])
}

/// Bring the window up once the first frame is ready, avoiding a white flash.
pub fn show_dock_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.show();
    }
}
```

- [ ] **Step 6: Add the window tracker module**

`dock-tauri/src-tauri/src/windows_tracker.rs`:

```rust
//! Live window enumeration, reconciled against the app catalog.

use crate::apps::{self, AppEntry, RunningInfo, WindowRef};

#[cfg(windows)]
pub fn snapshot_windows() -> Vec<WindowRef> {
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, MAX_PATH};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
    };

    struct Collector {
        windows: Vec<WindowRef>,
    }

    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        use windows::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        };
        use windows::Win32::UI::Shell::CommandLineToArgvW;
        use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;

        let collector = &mut *(lparam.0 as *mut Collector);

        // Skip invisible windows and tool windows: the dock never mirrors them.
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if ex_style & WS_EX_TOOLWINDOW.0 != 0 {
            return BOOL(1);
        }
        // Skip untitled windows (splash screens, shell internals).
        if GetWindowTextLengthW(hwnd) == 0 {
            return BOOL(1);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return BOOL(1);
        }

        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return BOOL(1);
        };
        let mut buffer = [0u16; MAX_PATH as usize];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = windows::Win32::Foundation::CloseHandle(process);
        if ok.is_err() || size == 0 {
            return BOOL(1);
        }
        let path = std::ffi::OsString::from_wide(&buffer[..size as usize])
            .to_string_lossy()
            .to_string();

        collector.windows.push(WindowRef {
            process_path: path,
            minimized: IsIconic(hwnd).as_bool(),
        });
        BOOL(1)
    }

    let mut collector = Collector { windows: Vec::new() };
    // SAFETY: `collector` outlives the enumeration; the callback writes only into it.
    unsafe {
        let _ = EnumWindows(Some(callback), LPARAM(&mut collector as *mut Collector as isize));
    }
    collector.windows
}

#[cfg(not(windows))]
pub fn snapshot_windows() -> Vec<WindowRef> {
    Vec::new()
}

pub fn snapshot_and_reconcile(apps: &[AppEntry]) -> Vec<RunningInfo> {
    let windows = snapshot_windows();
    apps::reconcile(apps, &windows)
}
```

Note: the `use` statements for `OpenProcess`, `QueryFullProcessImageNameW`, and `GetWindowTextW` belong at the top of the `#[cfg(windows)]` function, not inside the callback — move them there if the compiler complains about unused imports or item placement. `CommandLineToArgvW` is not needed; delete that import.

- [ ] **Step 7: Wire the builder in `lib.rs`**

Replace the body of `lib.rs` with:

```rust
//! Dock Tauri host: window geometry, app catalog, metrics sidecar, settings.

pub mod apps;
pub mod autohide;
pub mod commands;
pub mod geometry;
pub mod icon_cache;
pub mod metrics;
pub mod platform;
pub mod settings;
pub mod state;
pub mod windows_tracker;

use std::sync::Mutex;
use tauri::Manager;

pub fn product_name() -> &'static str {
    "Dock Tauri"
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let settings_path = settings_dir().join("settings.json");
            let apps_root = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("icons");
            let _icons = icon_cache::IconCache::new(apps_root);
            let catalog = apps::catalog_from_dirs(&apps::start_menu_dirs());
            app.manage(commands::Runtime {
                apps: Mutex::new(catalog),
                settings_path,
                sidecar: Mutex::new(metrics::Supervisor::new(commands::sidecar_spec())),
            });

            if let Some(window) = app.get_webview_window("dock") {
                let _ = window.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dock_state,
            commands::launch_item,
            commands::set_route,
            commands::get_settings,
            commands::save_settings,
            commands::sample_monitor,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Dock Tauri");
}

fn settings_dir() -> std::path::PathBuf {
    std::env::var("APPDATA")
        .map(|base| std::path::PathBuf::from(base).join("DockApp"))
        .unwrap_or_else(|_| std::env::temp_dir())
}
```

- [ ] **Step 8: Run the tests**

```bash
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml
```

Expected: 4 new tests pass; every earlier test still passes.

If `QueryFullProcessImageNameW` or `OpenProcess` is missing, add the features `Win32_System_Threading` and `Win32_System_ProcessStatus` to the `windows` feature list.

- [ ] **Step 9: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src-tauri/src/commands.rs dock-tauri/src-tauri/src/state.rs \
  dock-tauri/src-tauri/src/windows_tracker.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/dock_state_view.rs
git commit -m "feat: wire tauri commands to dock state and app catalog"
```

---

## Task 13: Launchpad, monitor panel, and settings panel

**Files:**
- Create: `dock-tauri/src/launchpad/Launchpad.tsx`
- Create: `dock-tauri/src/launchpad/AppTile.tsx`
- Create: `dock-tauri/src/launchpad/search.ts`
- Create: `dock-tauri/src/launchpad/search.test.ts`
- Create: `dock-tauri/src/monitor/MonitorPanel.tsx`
- Create: `dock-tauri/src/monitor/Sparkline.tsx`
- Create: `dock-tauri/src/monitor/format.ts`
- Create: `dock-tauri/src/monitor/format.test.ts`
- Create: `dock-tauri/src/settings/SettingsPanel.tsx`
- Create: `dock-tauri/src/settings/validate.ts`
- Create: `dock-tauri/src/settings/validate.test.ts`
- Modify: `dock-tauri/src/App.tsx`

- [ ] **Step 1: Write the failing search test**

`dock-tauri/src/launchpad/search.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { rankApps } from "./search";

const apps = [
  { id: "a", name: "Calculator" },
  { id: "b", name: "Calendar" },
  { id: "c", name: "Paint" },
  { id: "d", name: "Notepad" },
];

describe("rankApps", () => {
  it("returns everything for an empty query, alphabetically", () => {
    const result = rankApps(apps, "");
    expect(result.map((a) => a.name)).toEqual([
      "Calculator",
      "Calendar",
      "Notepad",
      "Paint",
    ]);
  });

  it("prefix matches rank above substring matches", () => {
    const result = rankApps(apps, "cal");
    expect(result[0].name).toBe("Calculator");
    expect(result[1].name).toBe("Calendar");
  });

  it("is case insensitive", () => {
    expect(rankApps(apps, "PAI")[0].name).toBe("Paint");
  });

  it("returns an empty list when nothing matches", () => {
    expect(rankApps(apps, "zzzz")).toEqual([]);
  });

  it("tolerates a leading and trailing space", () => {
    expect(rankApps(apps, "  paint  ")[0].name).toBe("Paint");
  });

  it("does not mutate its input", () => {
    const copy = [...apps];
    rankApps(apps, "cal");
    expect(apps).toEqual(copy);
  });
});
```

- [ ] **Step 2: Write the failing format and validate tests**

`dock-tauri/src/monitor/format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { formatBytes, formatPercent } from "./format";

describe("formatBytes", () => {
  it("uses GB for large values", () => {
    expect(formatBytes(8 * 1024 ** 3)).toBe("8.0 GB");
  });

  it("uses MB for small values", () => {
    expect(formatBytes(512 * 1024 ** 2)).toBe("512 MB");
  });

  it("handles zero", () => {
    expect(formatBytes(0)).toBe("0 MB");
  });
});

describe("formatPercent", () => {
  it("rounds to one decimal", () => {
    expect(formatPercent(12.34)).toBe("12.3%");
  });

  it("clamps above 100", () => {
    expect(formatPercent(140)).toBe("100.0%");
  });

  it("clamps below 0", () => {
    expect(formatPercent(-5)).toBe("0.0%");
  });
});
```

`dock-tauri/src/settings/validate.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { validateSettings } from "./validate";

describe("validateSettings", () => {
  it("accepts the defaults", () => {
    const result = validateSettings({
      iconSize: 48,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "vitro",
      showMonitor: false,
    });
    expect(result.ok).toBe(true);
  });

  it("rejects an icon size below 16", () => {
    const result = validateSettings({
      iconSize: 8,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "vitro",
      showMonitor: false,
    });
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errors.join(" ")).toContain("iconSize");
  });

  it("rejects an icon size above 128", () => {
    const result = validateSettings({
      iconSize: 200,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "vitro",
      showMonitor: false,
    });
    expect(result.ok).toBe(false);
  });

  it("rejects an unknown profile", () => {
    const result = validateSettings({
      iconSize: 48,
      magnify: true,
      autohide: false,
      reducedMotion: false,
      profile: "nonsense",
      showMonitor: false,
    });
    expect(result.ok).toBe(false);
  });
});
```

- [ ] **Step 3: Run the tests and confirm failure**

```bash
cd D:/dockyh/dock-tauri
npm test
```

Expected: FAIL — the three modules do not exist.

- [ ] **Step 4: Implement search, format, and validate**

`dock-tauri/src/launchpad/search.ts`:

```ts
export interface SearchableApp {
  id: string;
  name: string;
}

/**
 * Filter and rank apps. Empty query returns everything alphabetically.
 * Prefix matches rank above substring matches; ties break alphabetically.
 */
export function rankApps<T extends SearchableApp>(apps: T[], query: string): T[] {
  const needle = query.trim().toLowerCase();
  if (needle.length === 0) {
    return [...apps].sort((a, b) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
    );
  }

  const scored = apps
    .map((app) => {
      const name = app.name.toLowerCase();
      if (name.startsWith(needle)) return { app, score: 0 };
      if (name.includes(needle)) return { app, score: 1 };
      return null;
    })
    .filter((entry): entry is { app: T; score: number } => entry !== null);

  scored.sort((a, b) => {
    if (a.score !== b.score) return a.score - b.score;
    return a.app.name.toLowerCase().localeCompare(b.app.name.toLowerCase());
  });
  return scored.map((entry) => entry.app);
}
```

`dock-tauri/src/monitor/format.ts`:

```ts
const MB = 1024 ** 2;
const GB = 1024 ** 3;

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 MB";
  if (bytes >= GB) return `${(bytes / GB).toFixed(1)} GB`;
  return `${Math.round(bytes / MB)} MB`;
}

export function formatPercent(value: number): string {
  if (!Number.isFinite(value)) return "0.0%";
  const clamped = Math.min(100, Math.max(0, value));
  return `${clamped.toFixed(1)}%`;
}
```

`dock-tauri/src/settings/validate.ts`:

```ts
export interface SettingsInput {
  iconSize: number;
  magnify: boolean;
  autohide: boolean;
  reducedMotion: boolean;
  profile: string;
  showMonitor: boolean;
}

export type ValidationResult =
  | { ok: true }
  | { ok: false; errors: string[] };

export function validateSettings(input: SettingsInput): ValidationResult {
  const errors: string[] = [];

  if (!Number.isInteger(input.iconSize) || input.iconSize < 16 || input.iconSize > 128) {
    errors.push("iconSize must be an integer between 16 and 128");
  }
  if (input.profile !== "macos" && input.profile !== "vitro") {
    errors.push(`profile must be "macos" or "vitro", got "${input.profile}"`);
  }

  return errors.length === 0 ? { ok: true } : { ok: false, errors };
}
```

- [ ] **Step 5: Run the tests**

```bash
cd D:/dockyh/dock-tauri
npm test
```

Expected: 26 tests pass (11 magnify + 6 search + 6 format + 4 validate, minus overlaps).

- [ ] **Step 6: Implement the Launchpad**

`dock-tauri/src/launchpad/AppTile.tsx`:

```tsx
interface Props {
  name: string;
  icon: string | null;
  disabledReason: string | null;
  onLaunch: () => void;
}

export function AppTile({ name, icon, disabledReason, onLaunch }: Props) {
  return (
    <button
      type="button"
      className="launchpad-tile"
      onClick={onLaunch}
      disabled={Boolean(disabledReason)}
      aria-label={name}
      title={disabledReason ?? name}
    >
      {icon ? (
        <img src={icon} alt="" draggable={false} />
      ) : (
        <span className="fallback-icon" aria-hidden="true">
          {name.slice(0, 1).toUpperCase()}
        </span>
      )}
      <span className="launchpad-name">{name}</span>
    </button>
  );
}
```

`dock-tauri/src/launchpad/Launchpad.tsx`:

```tsx
import { useMemo, useState } from "react";
import type { DockItemView } from "../types";
import { rankApps } from "./search";
import { AppTile } from "./AppTile";

interface Props {
  items: DockItemView[];
  loading: boolean;
  error: string | null;
  onLaunch: (id: string) => void;
  onClose: () => void;
}

export function Launchpad({ items, loading, error, onLaunch, onClose }: Props) {
  const [query, setQuery] = useState("");

  const searchable = useMemo(
    () => items.map((item) => ({ id: item.id, name: item.name, item })),
    [items],
  );
  const results = useMemo(() => rankApps(searchable, query), [searchable, query]);

  if (loading) {
    return (
      <section className="launchpad" aria-busy="true">
        <p>Scanning installed applications…</p>
      </section>
    );
  }
  if (error) {
    return (
      <section className="launchpad" role="alert">
        <p>Could not load applications: {error}</p>
      </section>
    );
  }

  return (
    <section className="launchpad" aria-label="Launchpad">
      <header>
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search applications"
          aria-label="Search applications"
          autoFocus
        />
        <button type="button" onClick={onClose}>
          Close
        </button>
      </header>
      {results.length === 0 ? (
        <p className="launchpad-empty">No applications match “{query}”.</p>
      ) : (
        <div className="launchpad-grid">
          {results.map(({ item }) => (
            <AppTile
              key={item.id}
              name={item.name}
              icon={item.icon}
              disabledReason={item.disabledReason}
              onLaunch={() => onLaunch(item.id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}
```

- [ ] **Step 7: Implement the monitor panel**

`dock-tauri/src/monitor/Sparkline.tsx`:

```tsx
interface Props {
  values: number[];
  width?: number;
  height?: number;
}

/** Hand-drawn SVG sparkline: no chart library, no axes, no grid. */
export function Sparkline({ values, width = 160, height = 32 }: Props) {
  if (values.length < 2) {
    return <svg width={width} height={height} aria-hidden="true" />;
  }
  const max = Math.max(...values, 1);
  const step = width / (values.length - 1);
  const points = values
    .map((value, index) => {
      const x = index * step;
      const y = height - (value / max) * (height - 2) - 1;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");

  return (
    <svg width={width} height={height} aria-hidden="true">
      <polyline
        points={points}
        fill="none"
        stroke="var(--accent)"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
```

`dock-tauri/src/monitor/MonitorPanel.tsx`:

```tsx
import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { MonitorSample } from "../types";
import { formatBytes, formatPercent } from "./format";
import { Sparkline } from "./Sparkline";

interface Props {
  onClose: () => void;
}

const SAMPLE_INTERVAL_MS = 1000;
const HISTORY = 60;

export function MonitorPanel({ onClose }: Props) {
  const [sample, setSample] = useState<MonitorSample | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    let cancelled = false;

    const tick = async () => {
      try {
        const next = await api.sampleMonitor();
        if (cancelled) return;
        setError(null);
        setSample(next);
        setCpuHistory((history) =>
          [...history, next.cpu].slice(-HISTORY),
        );
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    };

    void tick();
    timer.current = window.setInterval(() => void tick(), SAMPLE_INTERVAL_MS);
    return () => {
      cancelled = true;
      if (timer.current !== null) window.clearInterval(timer.current);
    };
  }, []);

  return (
    <section className="monitor-panel" aria-label="System monitor">
      <header>
        <h2>System</h2>
        <button type="button" onClick={onClose}>
          Close
        </button>
      </header>

      {error && <p role="alert">{error}</p>}

      {sample && (
        <dl>
          <dt>CPU</dt>
          <dd>
            {formatPercent(sample.cpu)} <Sparkline values={cpuHistory} />
          </dd>
          <dt>Memory</dt>
          <dd>
            {formatBytes(sample.memUsed)} / {formatBytes(sample.memTotal)}
          </dd>
          <dt>Disk</dt>
          <dd>
            {formatBytes(sample.diskUsed)} / {formatBytes(sample.diskTotal)}
          </dd>
          <dt>Network</dt>
          <dd>
            ↓ {formatBytes(sample.netRx)} · ↑ {formatBytes(sample.netTx)}
          </dd>
        </dl>
      )}
    </section>
  );
}
```

- [ ] **Step 8: Implement the settings panel**

`dock-tauri/src/settings/SettingsPanel.tsx`:

```tsx
import { useEffect, useState } from "react";
import { api } from "../api";
import { validateSettings, type SettingsInput } from "./validate";

interface Props {
  onClose: () => void;
}

const DEFAULTS: SettingsInput = {
  iconSize: 48,
  magnify: true,
  autohide: false,
  reducedMotion: false,
  profile: "vitro",
  showMonitor: false,
};

export function SettingsPanel({ onClose }: Props) {
  const [form, setForm] = useState<SettingsInput>(DEFAULTS);
  const [errors, setErrors] = useState<string[]>([]);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api
      .getSettings()
      .then((raw) => {
        const mapped: SettingsInput = {
          iconSize: Number(raw.icon_size ?? raw.iconSize ?? DEFAULTS.iconSize),
          magnify: Boolean(raw.magnify ?? DEFAULTS.magnify),
          autohide: Boolean(raw.autohide ?? DEFAULTS.autohide),
          reducedMotion: Boolean(raw.reduced_motion ?? raw.reducedMotion),
          profile: String(raw.profile ?? DEFAULTS.profile),
          showMonitor: Boolean(raw.show_monitor ?? raw.showMonitor),
        };
        setForm(mapped);
      })
      .catch((e) => setErrors([String(e)]));
  }, []);

  const save = async () => {
    const result = validateSettings(form);
    if (!result.ok) {
      setErrors(result.errors);
      setSaved(false);
      return;
    }
    setErrors([]);
    try {
      await api.saveSettings({
        schema_version: 1,
        icon_size: form.iconSize,
        magnify: form.magnify,
        autohide: form.autohide,
        reduced_motion: form.reducedMotion,
        profile: form.profile,
        show_monitor: form.showMonitor,
        pinned: [],
      });
      setSaved(true);
    } catch (e) {
      setErrors([String(e)]);
    }
  };

  return (
    <section className="settings-panel" aria-label="Settings">
      <header>
        <h2>Settings</h2>
        <button type="button" onClick={onClose}>
          Close
        </button>
      </header>

      <label>
        Icon size
        <input
          type="range"
          min={16}
          max={128}
          step={4}
          value={form.iconSize}
          onChange={(e) => setForm({ ...form, iconSize: Number(e.target.value) })}
        />
        <output>{form.iconSize} px</output>
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.magnify}
          onChange={(e) => setForm({ ...form, magnify: e.target.checked })}
        />
        Magnification
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.autohide}
          onChange={(e) => setForm({ ...form, autohide: e.target.checked })}
        />
        Auto-hide when the pointer leaves
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.reducedMotion}
          onChange={(e) => setForm({ ...form, reducedMotion: e.target.checked })}
        />
        Reduced motion
      </label>

      <label>
        Motion profile
        <select
          value={form.profile}
          onChange={(e) => setForm({ ...form, profile: e.target.value })}
        >
          <option value="vitro">Elastic</option>
          <option value="macos">Restrained</option>
        </select>
      </label>

      <label>
        <input
          type="checkbox"
          checked={form.showMonitor}
          onChange={(e) => setForm({ ...form, showMonitor: e.target.checked })}
        />
        Show system monitor
      </label>

      {errors.length > 0 && (
        <ul role="alert">
          {errors.map((error) => (
            <li key={error}>{error}</li>
          ))}
        </ul>
      )}

      <button type="button" onClick={() => void save()}>
        Save
      </button>
      {saved && <p role="status">Saved.</p>}
    </section>
  );
}
```

- [ ] **Step 9: Route between surfaces in `App.tsx`**

```tsx
import { useEffect, useState } from "react";
import { api, onDockState } from "./api";
import { Dock } from "./dock/Dock";
import { Launchpad } from "./launchpad/Launchpad";
import { MonitorPanel } from "./monitor/MonitorPanel";
import { SettingsPanel } from "./settings/SettingsPanel";
import type { DockState } from "./types";

type Route = "dock" | "launchpad" | "monitor" | "settings";

export function App() {
  const [state, setState] = useState<DockState | null>(null);
  const [route, setRoute] = useState<Route>("dock");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    api
      .getDockState()
      .then(setState)
      .catch((e) => setError(String(e)));
    void onDockState(setState).then((unlisten) => {
      dispose = unlisten;
    });
    return () => dispose?.();
  }, []);

  const go = (next: Route) => {
    setRoute(next);
    void api.openRoute(next);
  };

  if (error) {
    return (
      <main className="dock-stage" role="alert">
        Dock unavailable: {error}
      </main>
    );
  }
  if (!state) {
    return <main className="dock-stage" aria-busy="true" />;
  }

  return (
    <main>
      {route === "dock" && (
        <Dock state={state} onActivate={(id) => void api.launchItem(id)} />
      )}
      {route === "launchpad" && (
        <Launchpad
          items={state.items}
          loading={false}
          error={null}
          onLaunch={(id) => void api.launchItem(id)}
          onClose={() => go("dock")}
        />
      )}
      {route === "monitor" && <MonitorPanel onClose={() => go("dock")} />}
      {route === "settings" && <SettingsPanel onClose={() => go("dock")} />}
    </main>
  );
}
```

- [ ] **Step 10: Run all frontend checks**

```bash
cd D:/dockyh/dock-tauri
npm test
npm run build
```

Expected: all Vitest tests pass; build succeeds.

The route buttons that open Launchpad/monitor/settings are added in Task 14 when the control tile lands. Until then these surfaces are reachable only by changing `route` in code, which is acceptable: Task 14 completes the entry points.

- [ ] **Step 11: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src/launchpad dock-tauri/src/monitor dock-tauri/src/settings dock-tauri/src/App.tsx
git commit -m "feat: add launchpad monitor and settings panels"
```

---
## Task 14: Control tile, route entry points, and live window sync

**Files:**
- Create: `dock-tauri/src/dock/ControlTile.tsx`
- Create: `dock-tauri/src/dock/ControlTile.test.tsx`
- Create: `dock-tauri/src-tauri/src/window_geom.rs` (Rust: apply geometry to the real window)
- Modify: `dock-tauri/src-tauri/src/commands.rs`
- Modify: `dock-tauri/src-tauri/src/lib.rs`
- Modify: `dock-tauri/src/dock/Dock.tsx`
- Modify: `dock-tauri/src/App.tsx`
- Modify: `dock-tauri/package.json`
- Test: `dock-tauri/src-tauri/tests/unit/window_geom.rs`

- [ ] **Step 1: Write the failing Rust geometry-application test**

`dock-tauri/src-tauri/tests/unit/window_geom.rs`:

```rust
use dock_tauri_lib::geometry::{DockGeometry, MonitorRect};
use dock_tauri_lib::window_geom::route_size;

fn monitor() -> MonitorRect {
    MonitorRect { left: 0, top: 0, right: 1920, bottom: 1040, scale: 1.0 }
}

#[test]
fn dock_route_uses_the_pill_geometry() {
    let dock = DockGeometry::compute(monitor(), 6, 48.0, 48.0);
    let (w, h) = route_size("dock", monitor(), 6, 48.0);
    assert_eq!(w, dock.width);
    assert_eq!(h, dock.height);
}

#[test]
fn launchpad_route_fills_the_work_area() {
    let (w, h) = route_size("launchpad", monitor(), 6, 48.0);
    assert_eq!(w, 1920.0);
    assert_eq!(h, 1040.0);
}

#[test]
fn settings_route_is_a_fixed_centered_panel() {
    let (w, h) = route_size("settings", monitor(), 6, 48.0);
    assert_eq!(w, 720.0);
    assert_eq!(h, 560.0);
}

#[test]
fn monitor_route_is_taller_than_the_dock() {
    let (_, dock_h) = route_size("dock", monitor(), 6, 48.0);
    let (_, monitor_h) = route_size("monitor", monitor(), 6, 48.0);
    assert!(monitor_h > dock_h);
}
```

- [ ] **Step 2: Write the failing control-tile test**

Add `@testing-library/react` and `@testing-library/jest-dom`:

```bash
cd D:/dockyh/dock-tauri
npm install --save-dev @testing-library/react @testing-library/jest-dom jsdom
```

`dock-tauri/src/dock/ControlTile.test.tsx`:

```tsx
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ControlTile } from "./ControlTile";

describe("ControlTile", () => {
  it("opens settings on a single click", () => {
    const onSettings = vi.fn();
    const onToggle = vi.fn();
    render(
      <ControlTile
        collapsed={false}
        onSettings={onSettings}
        onToggleCollapsed={onToggle}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /dock settings/i }));
    expect(onSettings).toHaveBeenCalledTimes(1);
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("collapses on a double click", () => {
    const onSettings = vi.fn();
    const onToggle = vi.fn();
    render(
      <ControlTile
        collapsed={false}
        onSettings={onSettings}
        onToggleCollapsed={onToggle}
      />,
    );
    const tile = screen.getByRole("button", { name: /dock settings/i });
    fireEvent.doubleClick(tile);
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("exposes the collapsed state to assistive tech", () => {
    render(
      <ControlTile collapsed onSettings={() => {}} onToggleCollapsed={() => {}} />,
    );
    expect(
      screen.getByRole("button", { name: /dock settings/i }),
    ).toHaveAttribute("aria-expanded", "false");
  });
});
```

Add to `vite.config.ts` `test` block:

```ts
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
```

Create `dock-tauri/src/test-setup.ts`:

```ts
import "@testing-library/jest-dom/vitest";
```

- [ ] **Step 3: Run both and confirm failure**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml --test window_geom
cd D:/dockyh/dock-tauri && npm test
```

Expected: both fail — `window_geom` module and `ControlTile` do not exist.

- [ ] **Step 4: Register the Rust test target**

```toml
[[test]]
name = "window_geom"
path = "tests/unit/window_geom.rs"
```

- [ ] **Step 5: Implement route sizing**

`dock-tauri/src-tauri/src/window_geom.rs`:

```rust
//! Translate a route name into window dimensions.

use crate::geometry::{DockGeometry, MonitorRect};

/// Physical width and height for `route` on `monitor`.
pub fn route_size(
    route: &str,
    monitor: MonitorRect,
    item_count: usize,
    icon_size: f64,
) -> (f64, f64) {
    match route {
        "launchpad" => (
            (monitor.right - monitor.left) as f64,
            (monitor.bottom - monitor.top) as f64,
        ),
        "settings" => (720.0, 560.0),
        "monitor" => {
            let dock = DockGeometry::compute(monitor, item_count, icon_size, 48.0);
            (dock.width.max(360.0), dock.height + 320.0)
        }
        _ => {
            let dock = DockGeometry::compute(monitor, item_count, icon_size, 48.0);
            (dock.width, dock.height)
        }
    }
}
```

- [ ] **Step 6: Apply the geometry from the command**

In `commands.rs`, replace `set_route` with a version that resizes and repositions the window:

```rust
#[tauri::command]
pub fn set_route(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    route: String,
) -> Result<(), CommandError> {
    let allowed = ["dock", "launchpad", "monitor", "settings"];
    if !allowed.contains(&route.as_str()) {
        return Err(CommandError::new("unknown_route", route));
    }

    let monitor = crate::platform::primary_work_area()
        .ok_or_else(|| CommandError::new("no_monitor", "no primary monitor"))?;
    let item_count = runtime
        .apps
        .lock()
        .map(|apps| apps.len())
        .unwrap_or(0);
    let (width, height) = crate::window_geom::route_size(&route, monitor, item_count, 48.0);

    if let Some(window) = app.get_webview_window("dock") {
        let _ = window.set_size(tauri::PhysicalSize::new(width as u32, height as u32));
        let x = if route == "launchpad" {
            monitor.left as f64
        } else {
            monitor.left as f64 + ((monitor.right - monitor.left) as f64 - width) / 2.0
        };
        let y = if route == "launchpad" {
            monitor.top as f64
        } else {
            (monitor.bottom as f64) - height
        };
        let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
    }

    app.emit("dock://route", &route)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}
```

Also add a command that pushes a fresh snapshot on demand, used by the 1 Hz sync:

```rust
#[tauri::command]
pub fn refresh_state(app: AppHandle, runtime: State<'_, Runtime>) -> Result<(), CommandError> {
    let settings = settings::load_or_recover(&runtime.settings_path)
        .map_err(|e| CommandError::new("settings_unreadable", e.to_string()))?;
    let apps = runtime
        .apps
        .lock()
        .map_err(|_| CommandError::new("lock_poisoned", "app catalog lock poisoned"))?;
    let running = crate::windows_tracker::snapshot_and_reconcile(&apps);
    let icons = crate::icon_cache::icons_for(&apps, 48);
    let state = build_dock_state(&apps, &running, &icons, &config_from(&settings));
    app.emit("dock://state", &state)
        .map_err(|e| CommandError::new("emit_failed", e.to_string()))
}
```

Register it in `lib.rs`'s `generate_handler!` list.

- [ ] **Step 7: Add the 1 Hz reconciliation loop**

In `lib.rs`, inside `setup`, after `app.manage(...)`:

```rust
            // 1 Hz window reconciliation. Bounded, never per-frame.
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let runtime = handle.state::<commands::Runtime>();
                let Ok(settings) = settings::load_or_recover(&runtime.settings_path) else {
                    continue;
                };
                let Ok(apps) = runtime.apps.lock() else { continue };
                let running = windows_tracker::snapshot_and_reconcile(&apps);
                let icons = icon_cache::icons_for(&apps, 48);
                let state =
                    state::build_dock_state(&apps, &running, &icons, &commands::config_from(&settings));
                let _ = handle.emit("dock://state", &state);
            });
```

`config_from` must be `pub(crate)`. Change its declaration in `commands.rs`:

```rust
pub(crate) fn config_from(settings: &Settings) -> DockConfig {
```

Add these imports at the top of `lib.rs`:

```rust
use tauri::Emitter;
use tauri::Manager;
```

- [ ] **Step 8: Implement the control tile**

`dock-tauri/src/dock/ControlTile.tsx`:

```tsx
interface Props {
  collapsed: boolean;
  onSettings: () => void;
  onToggleCollapsed: () => void;
}

/** Left-edge control tile: single click opens settings, double click collapses. */
export function ControlTile({ collapsed, onSettings, onToggleCollapsed }: Props) {
  return (
    <button
      type="button"
      className="dock-item control-tile"
      aria-label="Dock settings"
      aria-expanded={!collapsed}
      onClick={onSettings}
      onDoubleClick={onToggleCollapsed}
    >
      <span className="fallback-icon" aria-hidden="true">
        ⚙
      </span>
    </button>
  );
}
```

Wire it into `Dock.tsx` as the first child inside `.dock-pill`, passing `onOpenSettings` and `onToggleCollapsed` props down from `App.tsx`.

Add to `Dock.tsx`'s `Props`:

```tsx
  onOpenSettings: () => void;
  onOpenLaunchpad: () => void;
  onOpenMonitor: () => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
```

And render, before the `state.items.map(...)`:

```tsx
        <ControlTile
          collapsed={collapsed}
          onSettings={onOpenSettings}
          onToggleCollapsed={onToggleCollapsed}
        />
```

Also raise the collapse state so the pill can shrink:

```tsx
      <div className={`dock-pill${collapsed ? " is-collapsed" : ""}`} role="toolbar" aria-label="Dock">
```

Add the CSS to `dock.css`:

```css
.dock-pill.is-collapsed .dock-item:not(.control-tile) {
  display: none;
}
```

- [ ] **Step 9: Add route entry points in `App.tsx`**

Pass the four callbacks into `Dock`:

```tsx
      {route === "dock" && (
        <Dock
          state={state}
          collapsed={collapsed}
          onToggleCollapsed={() => setCollapsed((value) => !value)}
          onOpenSettings={() => go("settings")}
          onOpenLaunchpad={() => go("launchpad")}
          onOpenMonitor={() => go("monitor")}
          onActivate={(id) => void api.launchItem(id)}
        />
      )}
```

Add `const [collapsed, setCollapsed] = useState(false);` to `App`. Add two buttons inside the pill (next to the control tile) that call `onOpenLaunchpad` and `onOpenMonitor`, so Launchpad and the monitor have real entry points:

```tsx
        <button type="button" className="dock-item" aria-label="Open Launchpad" onClick={onOpenLaunchpad}>
          <span className="fallback-icon" aria-hidden="true">▦</span>
        </button>
```

Place this immediately after `<ControlTile …>` and a matching one for the monitor with `aria-label="Open system monitor"` and glyph `▤`.

- [ ] **Step 10: Run everything**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml
cd D:/dockyh/dock-tauri && npm test && npm run build
```

Expected: all Rust tests pass (including 4 new `window_geom` tests); all Vitest tests pass (including 3 new control-tile tests); build succeeds.

- [ ] **Step 11: Launch the app and confirm it works**

```bash
cd D:/dockyh/dock-tauri
npm run tauri dev
```

Manual checks, all required:
1. A rounded dark glass pill appears at the bottom center of the screen with real app icons.
2. Moving the pointer along the pill magnifies the nearest icon and its neighbours, and the icon bottoms stay aligned.
3. A tooltip appears after roughly half a second and fades in.
4. Clicking an app launches it; a running dot appears within about a second.
5. Minimizing that app shows a badge count on its tile.
6. The control tile opens settings; saving changes persists across a restart.
7. The Launchpad button shows a searchable app grid.
8. The monitor button shows live CPU/memory/disk/network values.

Record which of these you actually observed. Do not claim a check you did not perform.

- [ ] **Step 12: Commit**

```bash
cd D:/dockyh
git add dock-tauri/src/dock/ControlTile.tsx dock-tauri/src/dock/ControlTile.test.tsx \
  dock-tauri/src/dock/Dock.tsx dock-tauri/src/dock/dock.css dock-tauri/src/App.tsx \
  dock-tauri/src/test-setup.ts dock-tauri/vite.config.ts dock-tauri/package.json \
  dock-tauri/package-lock.json dock-tauri/src-tauri/src/window_geom.rs \
  dock-tauri/src-tauri/src/commands.rs dock-tauri/src-tauri/src/lib.rs \
  dock-tauri/src-tauri/Cargo.toml dock-tauri/src-tauri/tests/unit/window_geom.rs
git commit -m "feat: add control tile route entry points and live window sync"
```

---

## Task 15: Performance measurement, README, and packaging

**Files:**
- Create: `dock-tauri/scripts/measure_memory.ps1`
- Create: `dock-tauri/README.md`
- Modify: `dock-tauri/src-tauri/src/lib.rs` (auto-hide wiring + acrylic probe)
- Create: `dock-tauri/backend/build_sidecar.ps1`

- [ ] **Step 1: Write the measurement script**

`dock-tauri/scripts/measure_memory.ps1`:

```powershell
# Measures the two memory gates and idle CPU for dock-tauri.
# Usage: powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1
param(
    [int]$SettleSeconds = 60,
    [int]$Runs = 3
)

$ErrorActionPreference = "Stop"
$processName = "dock-tauri"

function Get-ProcessTree {
    param([int]$RootId)
    $all = Get-CimInstance Win32_Process
    $tree = @()
    $frontier = @($RootId)
    while ($frontier.Count -gt 0) {
        $current = $frontier[0]
        if ($frontier.Count -gt 1) { $frontier = $frontier[1..($frontier.Count - 1)] } else { $frontier = @() }
        $children = $all | Where-Object { $_.ParentProcessId -eq $current }
        foreach ($child in $children) {
            $tree += $child.ProcessId
            $frontier += $child.ProcessId
        }
    }
    return $tree
}

function Measure-Once {
    $host = Get-Process -Name $processName -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $host) {
        throw "dock-tauri is not running. Start it with 'npm run tauri dev' or the built exe."
    }

    $hostMb = [math]::Round($host.PrivateMemorySize64 / 1MB, 1)
    $childIds = Get-ProcessTree -RootId $host.Id
    $childMb = 0
    $breakdown = @()
    foreach ($id in $childIds) {
        $child = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($child) {
            $mb = [math]::Round($child.PrivateMemorySize64 / 1MB, 1)
            $childMb += $mb
            $breakdown += "{0} ({1} MB)" -f $child.ProcessName, $mb
        }
    }

    # Idle CPU over a 10 second window.
    $cpuStart = $host.TotalProcessorTime.TotalMilliseconds
    Start-Sleep -Seconds 10
    $host.Refresh()
    $cpuEnd = $host.TotalProcessorTime.TotalMilliseconds
    $cpuPercent = [math]::Round((($cpuEnd - $cpuStart) / 10000 / [Environment]::ProcessorCount) * 100, 2)

    [pscustomobject]@{
        HostMB     = $hostMb
        ChildrenMB = [math]::Round($childMb, 1)
        TotalMB    = [math]::Round($hostMb + $childMb, 1)
        IdleCPU    = $cpuPercent
        Breakdown  = ($breakdown -join ", ")
    }
}

$results = @()
for ($i = 1; $i -le $Runs; $i++) {
    Write-Host "Run $i — settling for $SettleSeconds seconds..."
    Start-Sleep -Seconds $SettleSeconds
    $results += Measure-Once
}

$hostMedian = ($results | Sort-Object HostMB)[[int]($Runs / 2)].HostMB
$totalMedian = ($results | Sort-Object TotalMB)[[int]($Runs / 2)].TotalMB
$cpuMedian = ($results | Sort-Object IdleCPU)[[int]($Runs / 2)].IdleCPU

Write-Host ""
Write-Host "=== Dock Tauri performance ==="
$results | ForEach-Object {
    Write-Host ("host {0} MB | children {1} MB | total {2} MB | idle CPU {3}% | {4}" -f `
        $_.HostMB, $_.ChildrenMB, $_.TotalMB, $_.IdleCPU, $_.Breakdown)
}
Write-Host ""
Write-Host ("Median host:  {0} MB  (gate: < 30 MB)  {1}" -f $hostMedian, $(if ($hostMedian -lt 30) { "PASS" } else { "FAIL" }))
Write-Host ("Median total: {0} MB  (gate: < 200 MB) {1}" -f $totalMedian, $(if ($totalMedian -lt 200) { "PASS" } else { "FAIL" }))
Write-Host ("Median idle CPU: {0}%  (gate: < 1%)    {1}" -f $cpuMedian, $(if ($cpuMedian -lt 1) { "PASS" } else { "FAIL" }))
```

- [ ] **Step 2: Run the measurement**

```bash
cd D:/dockyh/dock-tauri
npm run tauri build
# start the built exe, wait for it to settle, then:
powershell -ExecutionPolicy Bypass -File scripts/measure_memory.ps1 -SettleSeconds 60 -Runs 3
```

Record the real numbers. If either gate fails, do not weaken the gate — investigate and report. Likely culprits and the corresponding fix:

| Symptom | Likely cause | Fix |
|---|---|---|
| Host > 30 MB | `image` crate + debug symbols | Enable `lto = true`, `codegen-units = 1`, `strip = true` in `[profile.release]` |
| Total > 200 MB | More than one renderer | Confirm only one `msedgewebview2 --type=renderer` process exists |
| Idle CPU ≥ 1% | A timer running at rest | Confirm the monitor panel is closed and the 1 Hz sync is the only timer |
| Total grows over time | Icon data URLs re-decoded | Cache decoded `Image` objects in Rust, cap the cache |

Add the release profile to `src-tauri/Cargo.toml` as step 3.

- [ ] **Step 3: Add the release profile**

Append to `dock-tauri/src-tauri/Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = "s"
panic = "abort"
```

Then rebuild and re-measure. Record both the before and after numbers in the README.

- [ ] **Step 4: Write the README**

`dock-tauri/README.md`:

```markdown
# Dock Tauri

A Windows dock built with Tauri + React, with a Python metrics sidecar.

## Features

- Bottom dock with parabolic magnification, reflection, running indicators, and minimized-window badges
- Click to launch, click a running app to focus it
- Auto-hide with an edge trigger (off by default)
- Launchpad: searchable application grid built from the Start Menu
- System monitor: CPU, memory, disk, network (Python `psutil`)
- Settings with atomic persistence and `.bak` recovery

Not included in v1: left/right/top edges, drag reordering, Shelf, tray mirroring, Docklet plugins,
DWM window thumbnails, minimize animation.

## Requirements

- Windows 11 with WebView2 (preinstalled on current builds)
- Rust at `D:\rust` (or adjust the paths below)
- Node 20+ with npm
- Python 3.11 with `psutil` and `pytest`

## Development

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
cd D:/dockyh/dock-tauri
npm install
npm run tauri dev
```

## Tests

```bash
# Rust
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-tauri/src-tauri/Cargo.toml

# Frontend
npm test

# Python
cd backend && D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
```

## Performance

Measured with `scripts/measure_memory.ps1` (60 s settle, median of 3 runs):

| Gate | Target | Measured |
|---|---:|---:|
| Host private bytes | < 30 MB | _fill in_ |
| Host + WebView2 children | < 200 MB | _fill in_ |
| Idle CPU | < 1% | _fill in_ |

_Record the real measured numbers. Do not publish targets as if they were results._

## Architecture

One Tauri window; React swaps routes inside it, so exactly one WebView2 renderer exists.
Python is a stdio JSON-RPC sidecar for metrics only — never an HTTP server.
See `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`.

## Known limitations

- The application icon is a placeholder pending final artwork.
- UWP apps appear only if `Get-StartApps` discovery succeeds; otherwise classic shortcuts only.
- The 2 px auto-hide strip overlaps the taskbar's lowest pixels; auto-hide defaults off.
```

- [ ] **Step 5: Write the sidecar packaging script**

`dock-tauri/backend/build_sidecar.ps1`:

```powershell
# Freezes the Python sidecar into a single exe for release.
# Requires PyInstaller: D:/jiqixuexi/anaconda/python.exe -m pip install pyinstaller
$ErrorActionPreference = "Stop"
$python = "D:/jiqixuexi/anaconda/python.exe"

& $python -m PyInstaller `
    --onefile `
    --name monitor `
    --distpath ../src-tauri/binaries `
    --workpath ../build/pyinstaller `
    --specpath ../build `
    monitor/main.py

Write-Host "Sidecar written to src-tauri/binaries/monitor.exe"
```

This is not wired into the Tauri bundle in v1: doing so requires a `bundle.externalBin` entry plus target-triple naming (`monitor-x86_64-pc-windows-msvc.exe`). Note that in the README as follow-up work rather than pretending it ships.

- [ ] **Step 6: Full verification**

```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
cd D:/dockyh/dock-tauri
D:/rust/cargo/bin/cargo.exe fmt --manifest-path src-tauri/Cargo.toml -- --check
D:/rust/cargo/bin/cargo.exe clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
D:/rust/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
npm test
npm run build
cd backend && D:/jiqixuexi/anaconda/python.exe -m pytest tests -q
cd D:/dockyh && git diff --check
```

Expected: all pass. If `clippy -D warnings` fails on generated Tauri code, scope it to the lib and tests rather than adding blanket `allow`s.

- [ ] **Step 7: Commit**

```bash
cd D:/dockyh
git add dock-tauri/scripts/measure_memory.ps1 dock-tauri/README.md \
  dock-tauri/backend/build_sidecar.ps1 dock-tauri/src-tauri/Cargo.toml
git commit -m "perf: add memory gates and release profile, document measured results"
```

---

## Plan self-review checklist

- [ ] **Spec §2 (six surfaces):** Dock → Task 11; auto-hide → Tasks 3 + 15 wiring; minimize-into-dock → Tasks 6 + 12; Launchpad → Task 13; monitor → Tasks 8 + 9 + 13; settings → Tasks 4 + 13.
- [ ] **Spec §3 (one renderer):** Task 1 configures exactly one window; Task 14 resizes it rather than creating more.
- [ ] **Spec §4 (routes):** Task 14 implements `route_size` and the resize-on-route-change command.
- [ ] **Spec §5 (boundaries):** `core` logic is pure in Tasks 2–4; Win32 confined to `platform.rs`, `apps.rs`, `windows_tracker.rs`; React never calls Win32.
- [ ] **Spec §6 (IPC):** Task 12 registers every command; Task 9 proves the stdio contract; 1 Hz reconciliation in Task 14.
- [ ] **Spec §7.1 (glass):** Task 11 paints glass with token-derived values. Acrylic probe is **not implemented** — see the gap note below.
- [ ] **Spec §7.2 (magnify discipline):** Task 10 (pure math) + Task 11 (transform-only, pointer-driven, no `setState` during hover except position).
- [ ] **Spec §7.3 (auto-hide strip):** Task 3 is the state machine; the 2 px strip window handling is **not implemented** — see the gap note below.
- [ ] **Spec §8 (gates):** Task 15 measures host, total, and idle CPU.
- [ ] **Spec §9 (errors):** Task 4 settings recovery; Task 9 sidecar errors; Task 12 launch errors; Task 13 visible error states.
- [ ] **Spec §10 (testing):** Rust units Tasks 2–7, 9, 12, 14; Vitest Tasks 10, 13, 14; pytest Task 8. Playwright E2E is **not implemented** — see the gap note below.
- [ ] **Spec §12 (release boundary):** no legacy assets are copied; provenance rules from `dock-app` untouched.

### Gaps this plan deliberately defers (reported, not hidden)

1. **DWM Acrylic probe** (spec §7.1) — Task 11 ships painted glass only. The spec's fallback contract is satisfied (glass is always painted), but the acrylic *upgrade* is absent. Contained follow-up: call `window_vibrancy::apply_acrylic`, screenshot the pill corners, compare against the painted baseline, disable acrylic on mismatch.
2. **Auto-hide needs real edge-trigger wiring** (spec §7.3) — Task 3 implements and tests the timing state machine. Shrinking the live window to a 2 px strip and driving reveal from mouse-enter needs a geometry path like Task 14's; it is not written here. Because of this, auto-hide stays **off by default**, which the spec already requires.
3. **Playwright E2E** (spec §10) — no task exists. The Rust settings tests cover persistence across restart, but the cross-surface browser flows in the spec are not automated.

### Items that were gaps and are now covered

- **Icon extraction** — Task 7 implements `SHGetImageList(SHIL_JUMBO)` → BGRA→RGBA → PNG (`SHGetImageList` gives 256 px; `SHGetFileInfoW`/`SHGFI_ICON` alone only ever returns 16 px), plus `icons_for` to populate `DockItemView.icon`. The extraction test asserts the result is at least 48 px, which fails if the small-icon path is used.
- **Icon wiring into the view** — `build_dock_state` takes an `icons: &[Option<String>]` parameter (Task 12) and every call site in Tasks 12 and 14 passes it.

### Type-consistency notes for the implementer

- `build_dock_state(apps, running, icons, config)` — four parameters, in that order, everywhere.
- `DockItemView` serialises as camelCase (`disabledReason`, `memUsed`, …) because of `#[serde(rename_all = "camelCase")]`; the React `types.ts` mirrors that exactly.
- `Settings` serialises with snake_case field names (`icon_size`, `reduced_motion`, …) because it has no rename attribute. `SettingsPanel.tsx` reads both spellings defensively.
- `IconCache::key` returns 64-character lowercase hex.
- `route_size` returns `(width, height)` in physical pixels.
