# Dock Tauri — Phase 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Context:** Phase 1 (T1–T15) is complete: the dock runs, 118 tests pass, both memory gates are measured. This phase closes the three gaps the Phase 1 plan deferred. See `docs/superpowers/specs/2026-10-26-tauri-python-dock-design.md`.

**Deferred to a later phase (agreed with the user):** drag-to-reorder, drag-from-Launchpad, Shelf, tray mirroring, Docklet host, MSIX/AUMID identity.

**Environment reminder:**
```bash
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
export PATH="/d/rust/cargo/bin:$PATH"     # the Tauri CLI shells out to cargo
```
Frontend at `D:/dockyh/dock-tauri`. Python at `D:/jiqixuexi/anaconda/python.exe`.

---

## Task 16: Playwright end-to-end tests against a mocked Tauri IPC

The design requires "Playwright + mocked IPC". The React app cannot run inside a real Tauri webview under Playwright without `tauri-driver`, so the tests drive the **built frontend in a normal browser** with the Tauri bridge stubbed. This exercises real React rendering, real routing, and real event handling; only the native side is mocked.

**Files:**
- Create: `dock-tauri/playwright.config.ts`
- Create: `dock-tauri/e2e/tauri-mock.ts` — installs a fake `window.__TAURI_INTERNALS__`
- Create: `dock-tauri/e2e/dock.spec.ts`
- Create: `dock-tauri/e2e/panels.spec.ts`
- Modify: `dock-tauri/package.json` (scripts: `test:e2e`, `test:e2e:ui`)

- [ ] **Step 1: Understand the bridge Tauri v2 actually calls**

Read `node_modules/@tauri-apps/api/core.js`. In v2, `invoke(cmd, args)` calls
`window.__TAURI_INTERNALS__.invoke(cmd, args)` and `listen(event, handler)` calls
`window.__TAURI_INTERNALS__.invoke("plugin:event|listen", ...)` plus a transform callback
registered through `window.__TAURI_INTERNALS__.transformCallback`. Confirm the exact shape
before writing the mock; if the installed version differs, follow the installed source, not
this paragraph.

- [ ] **Step 2: Write the mock**

`dock-tauri/e2e/tauri-mock.ts` exports a string of JS to be injected with
`page.addInitScript(...)`. It must:

- define `window.__TAURI_INTERNALS__` with `invoke`, `transformCallback`, and `metadata`;
- answer `get_dock_state` with a fixture dock of **10 items** covering all four tile states
  (idle, running, minimized with `minimized: 2`, disabled with a `disabledReason`), plus
  `edge: "bottom"`, `magnify: true`, `reducedMotion: false`, `profile: "vitro"`;
- answer `launch_item`, `set_route`, `save_settings`, `set_pointer_inside`, `autohide_tick`
  with `null` and record the calls on `window.__MOCK_CALLS__`;
- answer `sample_monitor` with a fixed sample;
- answer `plugin:event|listen` by storing handlers and returning an id; expose
  `window.__MOCK_EMIT__(event, payload)` so a test can push `dock://state`.
- expose `window.__MOCK_FAIL__ = (cmd) => ...` so a test can make one command reject, to
  exercise the error paths.

- [ ] **Step 3: Configure Playwright**

`playwright.config.ts`:
- `testDir: "./e2e"`, `fullyParallel: false`, `workers: 1`
- `webServer: { command: "npm run dev", url: "http://localhost:1420", reuseExistingServer: true, timeout: 120_000 }`
- single project: chromium, viewport 1280×720
- `use: { baseURL: "http://localhost:1420" }`
- reporter list

Add scripts: `"test:e2e": "playwright test"`.

Install the browser once: `npx playwright install chromium`.

- [ ] **Step 4: Write `dock.spec.ts`**

Before each test: `page.addInitScript(TAURI_MOCK)`, then `page.goto("/")`, then
`await expect(page.getByRole("toolbar", { name: "Dock" })).toBeVisible()`.

Tests (each asserts observable behaviour, not implementation):
1. **All tile states render** — idle tile by name; running tile has an accessible name
   containing "running"; the minimized tile shows a badge "2"; the disabled tile is disabled.
2. **Click launches** — click a tile, then assert `__MOCK_CALLS__` contains
   `{cmd: "launch_item", args: {id: <that id>}}`.
3. **Hover magnifies** — hover an icon, read its `transform` via `evaluate`, assert a
   `scale(...)` greater than 1; move away, assert it returns to ~1 within a second. Poll,
   because the transition is animated.
4. **Tooltip after dwell** — hover, assert no `[role=tooltip]`; wait ~600 ms; assert the
   tooltip with the app name is visible.
5. **State push re-renders** — `__MOCK_EMIT__("dock://state", {...one item running})` and
   assert the running indicator appears without a reload.

- [ ] **Step 5: Write `panels.spec.ts`**

1. **Settings opens on single click** — click the control tile, wait past the 250 ms
   disambiguation delay, assert the Settings panel is visible.
2. **Successful save calls the backend** — change the icon-size range, click Save, assert
   `save_settings` was invoked with `icon_size` matching the new value.
3. **Validation blocks an invalid save** — force the range to an invalid value (dispatch an
   input event with `value = 8`; jsdom is not involved here, so the browser will clamp —
   instead set the value through the DOM and dispatch, or type into a number input if the
   component exposes one). If the control cannot produce an invalid value through the UI,
   assert the client-side validator directly by importing `validateSettings` in a unit test
   instead, and say so in the report. Do not fake a passing test.
4. **Launchpad search filters** — open Launchpad, type a query that matches one fixture app,
   assert only that app remains and the others are gone.
5. **Launchpad empty state** — type a query that matches nothing, assert the "no applications
   match" message.
6. **Monitor renders a sample** — open the monitor, assert CPU percentage text and memory
   text from the fixture sample are visible.
7. **Monitor error path** — set `__MOCK_FAIL__("sample_monitor")`, open the monitor, assert
   an alert is visible and the panel is still on screen.

- [ ] **Step 6: Run and fix**

```bash
cd D:/dockyh/dock-tauri
npm run test:e2e
```
Expected: 12 tests pass. Failures usually mean the mock's `invoke` shape does not match the
installed `@tauri-apps/api`; fix the mock, not the product, unless the product is genuinely
broken (in which case report it).

- [ ] **Step 7: Commit**

```bash
git add dock-tauri/playwright.config.ts dock-tauri/e2e dock-tauri/package.json dock-tauri/package-lock.json
git commit -m "test: add playwright e2e against mocked tauri ipc"
```

---

## Task 17: Freeze the sidecar and ship an MSI

**Files:**
- Modify: `dock-tauri/backend/build_sidecar.ps1`
- Modify: `dock-tauri/src-tauri/tauri.conf.json`
- Modify: `dock-tauri/src-tauri/src/commands.rs` (resolve the bundled sidecar first)
- Modify: `dock-tauri/README.md`
- Create: `dock-tauri/src-tauri/tests/unit/sidecar_path.rs`

- [ ] **Step 1: Install PyInstaller and freeze the sidecar**

```bash
D:/jiqixuexi/anaconda/python.exe -m pip install pyinstaller
cd D:/dockyh/dock-tauri
powershell -ExecutionPolicy Bypass -File backend/build_sidecar.ps1
```
Expected: `src-tauri/binaries/monitor.exe` exists and is a few MB. Run it to prove it works:
```bash
printf '{"cmd":"sample"}\n' | ./src-tauri/binaries/monitor.exe
```
Expected: one JSON line with `"t": "sample"`.

- [ ] **Step 2: Give the binary the name Tauri expects**

Tauri requires `externalBin` entries to be suffixed with the target triple at build time.
Rename/copy to `src-tauri/binaries/monitor-x86_64-pc-windows-msvc.exe` (the triple is what
`rustc -vV` reports for this host — verify with
`D:/rust/cargo/bin/rustc.exe -vV | grep host`). Update `build_sidecar.ps1` so it produces
that name directly instead of requiring a manual rename.

- [ ] **Step 3: Declare it in the bundle**

In `tauri.conf.json`:
```json
"bundle": {
  "active": true,
  "targets": ["msi"],
  "icon": ["icons/icon.ico"],
  "externalBin": ["binaries/monitor"],
  "resources": []
}
```
(Do not add `resources` if unused.)

- [ ] **Step 4: Prefer the bundled sidecar at runtime**

In `commands.rs`, change `sidecar_spec()` to:
1. resolve `current_exe()`'s directory, look for `monitor.exe` next to it (Tauri copies
   `externalBin` beside the executable);
2. if present, return `SidecarSpec::new(that path, vec![])`;
3. otherwise fall back to the existing dev path (Python interpreter + `run_sidecar.py`).

Extract the search into a pure function so it is testable:
```rust
/// `exe_dir` is the directory containing dock-tauri.exe. `dev_python` and
/// `dev_launcher` are the development fallbacks.
pub fn resolve_sidecar(
    exe_dir: &Path,
    dev_python: &Path,
    dev_launcher: &Path,
) -> SidecarSpec
```
Rules: bundled `monitor.exe` wins; otherwise Python + launcher; otherwise Python + launcher
even if the launcher is missing (the supervisor will surface a typed spawn error, which is
better than a panic).

- [ ] **Step 5: Test the resolution order**

`src-tauri/tests/unit/sidecar_path.rs` (register the `[[test]]` stanza):
```rust
use dock_tauri_lib::commands::resolve_sidecar;
use std::path::{Path, PathBuf};

fn bundled(dir: &Path) -> PathBuf { dir.join("monitor.exe") }

#[test]
fn prefers_a_bundled_sidecar_next_to_the_exe() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(bundled(tmp.path()), b"stub").unwrap();
    let spec = resolve_sidecar(tmp.path(), Path::new("Z:/py.exe"), Path::new("Z:/run.py"));
    assert_eq!(spec.program, bundled(tmp.path()));
    assert!(spec.args.is_empty());
}

#[test]
fn falls_back_to_the_dev_python_launcher() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = resolve_sidecar(tmp.path(), Path::new("Z:/py.exe"), Path::new("Z:/run.py"));
    assert_eq!(spec.program, PathBuf::from("Z:/py.exe"));
    assert_eq!(spec.args, vec!["Z:/run.py".to_string()]);
}
```
`resolve_sidecar` and `SidecarSpec.program/args` must be public for this. Add a third test
that the dev fallback still wins when a *directory* named `monitor.exe` exists (must not be
mistaken for the binary).

- [ ] **Step 6: Build the MSI**

```bash
cd D:/dockyh/dock-tauri
npm run tauri build
```
Expected: an `.msi` under `src-tauri/target/release/bundle/msi/`, and the build log shows the
sidecar being bundled.

**Do not install the MSI.** Installing changes the user's machine and is theirs to decide.
Verify the package instead:
```bash
powershell -Command "Get-ChildItem src-tauri/target/release/bundle/msi/*.msi | Select-Object Name,Length"
```

- [ ] **Step 7: Verify the bundled binary still runs and meets the memory gates**

Run the release exe from `src-tauri/target/release/` (the sidecar resolution will look for
`monitor.exe` beside it — copy it there if the build did not), open the monitor panel, and
confirm live values. Then re-run `scripts/measure_memory.ps1` and record the numbers.

- [ ] **Step 8: Update the README and commit**

Replace the "Packaging the sidecar" limitation with the real story, and add the MSI path.

```bash
git add dock-tauri/backend/build_sidecar.ps1 dock-tauri/src-tauri/tauri.conf.json \
  dock-tauri/src-tauri/src/commands.rs dock-tauri/src-tauri/Cargo.toml \
  dock-tauri/src-tauri/tests/unit/sidecar_path.rs dock-tauri/README.md
git commit -m "feat: bundle the frozen sidecar and produce an msi"
```

---

## Task 18: Acrylic — blocked pending a decision

**Do not implement until the user decides.** The finding:

`window-vibrancy::apply_acrylic` applies blur to the **entire window rectangle**, and CSS
transparency inside a webview cannot punch holes in it. The dock window is 1128×288 with the
pill occupying only the middle band (84 px of magnification headroom above, 40 px of
reflection margin below). Applying acrylic would therefore blur a large rectangle around the
pill — visibly wrong, exactly the failure the spec's "probe-gated fallback" was meant to
catch.

Three ways forward, for the user to choose:

1. **Keep painted glass** (status quo). Zero risk, no real desktop blur.
2. **Native glass window behind the webview.** A second, non-webview Tauri window with acrylic
   and a pill-shaped `SetWindowRgn`, positioned 1 px behind the content window. Real blur,
   confined to the pill. Costs one native window and region/monitor-resize handling.
3. **Acrylic only when magnification is off**, where the window can be sized to the pill
   exactly (plus reflection). Simple, but the material changes when the user toggles
   magnification, which is odd.

Recommendation: **2** if real blur matters; otherwise **1** and delete the acrylic claim from
the spec so documentation matches reality.
