# Provenance

Machine-readable source of truth: [`dock-app/assets/manifest.json`](../dock-app/assets/manifest.json)
(schema version 1). This page mirrors that manifest for humans; if the two ever
disagree, the manifest wins and this page is wrong.

Frozen on 2026-10-25. Every record carries `path`, `source`, `author`,
`license`, `sha256`, `width`/`height` (bitmaps only), `allowed_use`, and
`verified_at`. `dock-app/tests/unit/provenance.rs` re-hashes every listed file
on every test run, so a stale hash fails CI.

## Allowed-use vocabulary

| Value | Meaning |
|---|---|
| `release` | May be compiled, copied, or packaged into the shipped product. |
| `internal` | Usable during development, never packaged in a release. |
| `reference` | Read for design/behavior input only. Never copied, compiled, or packaged. |

## Decision summary

| Resource group | Count | Allowed use | Decision |
|---|---:|---|---|
| `vitro/assets/icons/*.png` (96x96) | 25 | `reference` | Authorship and trademark review is incomplete, so these are prototype and internal-comparison artwork only. The release build needs a newly drawn vector master set. |
| `docs/specs/*.toml` | 3 | `release` | These are this project's own frozen specifications (effect slots, motion tokens, macOS parity parameters). They drive the native renderer directly. |
| `docs/prototypes/*.html` | 4 | `reference` | Visual and interaction acceptance baselines. Never packaged, never hosted in a WebView, never a source of DOM/JS runtime code. |
| `docs/research/*.md` | 12 | `reference` | Requirements, behavior notes, and competitor observations. Each requirement must trace to a test or acceptance item. |
| Windows SDK APIs | 9 components | `release` | Used only through typed Rust adapters in `dock-app/src/platform`. HWNDs never leave the platform layer. |
| Restricted competitor/legacy sources | 8 sources | `reference` | Behavior and architecture notes only, never shipped. See below. |

## Icons (96x96 PNG)

Origin: the `SQ` SVG library inside `docs/prototypes/vitro-nexus-full.html`,
rasterized to 96 px RGBA PNGs by `vitro/tools/gen_sq_icons.py`. All 25 are
`allowed_use = "reference"` until authorship and trademark review completes.

| Path | SHA-256 | Dimensions | Author | License | Use |
|---|---|---|---:|---|---|
| `vitro/assets/icons/battery.png` | `c10e6cad4dd40d14e0ad6b0707aab037cda543f7e8f681111b7b4ce6485d44c8` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/browser.png` | `c5c5f4a7d1f7aaa280dd598013a65cd7dfa0b65279a8b3655e41c5ad9e96bc92` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/calendar.png` | `4ece25f02f26c328bb8c0cac3b867fe70e8c021996ee5252a935adf1db9a2091` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/clip.png` | `83df8a14429284f1f2a811fbc13ff31c7074852a16cca815bd38118c042c913a` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/clock.png` | `2202eb9dce3fdb37db199b4c2289e3e1702c1dfa008d7b01a608b845f40af90d` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/code.png` | `eede6fd5993ac485bed8d15906ba406990aaf93a66c271c35a639c8aa0ca21b5` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/control.png` | `18fd1758e194213d06b579a8fa545a384670bff0e2b67624b083328f08d3a0fd` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/cpu.png` | `84fab5d8f0940779e199a9de27f930a4d52b888366d4c923df8bb8dc15d0f46c` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/doc.png` | `0f1d7fc4ae78841adcb9e88886469555e312cb226d9db443b1b98500baa3a060` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/drawer.png` | `b25a56c9c01d886a07b99fccd5d8f328dd56e85421987967f5d7924d777d5f85` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/folder.png` | `bae0337a17f474326f4a3ffa85c85ff335f639c9caea775b7552377c22ea0b73` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/gamepad.png` | `debb76064979fbaf636b98005a9dcb0f07e884b5f5406a62222b3dd0dc940f0e` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/mail.png` | `67ee2784e6792efb80b3324b8fd93ad2e566cdbd390f85232db56309d52343fb` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/menulist.png` | `ad7841bc3d09a2c910af41ca76ee3b5fb6a1405fb0d4a297244d820e4de25d59` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/music.png` | `67a8777640c80ff42fbf2eaf7bd7e30a7e1eda9037afe3d7078e06c14f8e68c4` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/net.png` | `2a0d51a45bfe5dfd9a0b8279c757bbfc3923bea2dad28d05a892ad917e1f53bf` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/notes.png` | `586588c4bc896ec032080c07ebdf46bbe687ec2b713cd11f26852625371f6cba` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/ram.png` | `193e0627a4ef4579b7d547356e13ab5b0e683eb5955b3dc0c765ef54bc9df6f4` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/settings.png` | `61b3761bf3f374bcf89bcba21b9ed69a0b31b28651fa14e18c51253b39b8af2c` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/shelfico.png` | `b9a5e2944e0fa3fe0d20915ae2121ee301ad555113b71a00f655ce68ac02e2d9` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/tasks.png` | `ff566eefc4329022ed2fefa2a484c7c01274b579b3307f25591bb19f38925696` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/terminal.png` | `005654ecba3239fa19ef8a22faa83cbe2a051834ae96479f25c791720bd9c486` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/trash.png` | `690a1f0e3aa00c902a4577d21e3f5d188e6e2562806d5422560d48ba514e311c` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/tray.png` | `1c2ec29733e8317fa8e3deb9066081b37f35d613908009dc151b1df1880c55cb` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |
| `vitro/assets/icons/weather.png` | `308bcfd2a7a0701b62c75959244d4ff000c2d5bbf82ebce29063b11cb142ba3c` | 96x96 | Dock App project (authorship unverified) | Project MIT claim, unverified for artwork | reference |

## Specifications (TOML) — release inputs

| Path | SHA-256 | Use |
|---|---|---|
| `docs/specs/effects-catalog.toml` | `ec3dc6a4544142753686d94c61c5d55730af25bb30fe3ef1e87ebc25439edbd2` | release |
| `docs/specs/motion-tokens.toml` | `97ce9f3bcdc8235e3197e8c395ccbcfc00713747020f1dfaf4cac422688037f1` | release |
| `docs/specs/macos-parity.toml` | `39f1a0e0be4048f39a2b4dd98d83cc347e3783cd75c16099af6615d7ddbe3537` | release |

The parameter values come from transitions.dev snippets and macOS observation;
the files themselves are this project's own records, so the TOML is a release
input. The HTML that demonstrated those values is not.

## HTML prototypes — reference only

| Path | SHA-256 | Use |
|---|---|---|
| `docs/prototypes/vitro-effects-gallery.html` | `f07a4cd834951a6e2b71f0d4b244b0217a33dd00e6318c0761fb862026b48c31` | reference |
| `docs/prototypes/vitro-m05.html` | `79c5e353619a6285da38dd2c44d3c8f1f15df074db5eb3c0ad7595c6641cdcbe` | reference |
| `docs/prototypes/vitro-macos-parity.html` | `87acb26492013fe74c4fa8f77731bb8c3c9eadfb39e5ab885e2f9dd19123e505` | reference |
| `docs/prototypes/vitro-nexus-full.html` | `68c5da97fa5e46a8f517c7d5b38a1b53e3de3e326f6f7321c46b7349317cb108` | reference |

## Research notes — reference only

| Path | SHA-256 | Use |
|---|---|---|
| `docs/research/00-SUMMARY.md` | `241b52d36c3699a80d2cde92355377492614ac7e2db985eb11da515fab4515df` | reference |
| `docs/research/competitive-research-2026-10.md` | `fd02cf4c37581f9a3da7b3f0620a2486445ac100dd7a91ed0ee7717ad9aa0b35` | reference |
| `docs/research/design-language-research.md` | `97a364286bfa071d667d85153ab5f3657ac4e47e7a58ff9222cf4be2ec0dfbb7` | reference |
| `docs/research/design-skills-mcp.md` | `b49251c09f83265f2e0f5f11e91752136b71ce359554faad0c333beef80ac19a` | reference |
| `docs/research/frontend-design-research.md` | `0a777c112b093851e7b06846b8d44e71c443473d3cc833f85630c56510f822c8` | reference |
| `docs/research/glassdesk-vitro-audit.md` | `1c47cfcb8582eefff1b1e718025c14f61814e89deb2560f33a2928a78959dea1` | reference |
| `docs/research/macos-dock-spec.md` | `d6259689c2acb47604dd1e1549132e1c24a72bf9d30708acf3cfd8c1d8b60e66` | reference |
| `docs/research/mydockfinder-bitdock-deepdive.md` | `02fc2a4f3975df0b4a51e92fa20498beb9fc7ddfaba0748e071e21ec4ade5d5a` | reference |
| `docs/research/nexus-dock-behavior-spec.md` | `be6f34e5a2926f13bb044ee4b636042768788f6aa9a9db1fd63a0383660dc718` | reference |
| `docs/research/plugin-design-language.md` | `b79eb590011f1a6149678adbf9d889a0c239373c463a8dcd6f9257dd58f0a7ca` | reference |
| `docs/research/skillry-premium-design-research.md` | `953ee2e37050969a94a87854d737d0e930ca248457b5760b96441216c72788ab` | reference |
| `docs/research/tech-stack-research.md` | `67132e8ef026426439097f327f8aae72c6dc81f52708ff30723af756b77eaf58` | reference |

## Windows SDK APIs — release

Usable in a shipped build, but only behind the typed adapters in
`dock-app/src/platform/windows`. Docklets never receive an HWND.

| Component | Provider | Use |
|---|---|---|
| Win32 windowing and messaging (user32) | Windows SDK | release |
| DWM backdrop and composition attributes | Windows SDK | release |
| Direct2D and DirectWrite | Windows SDK | release |
| DirectComposition | Windows SDK | release |
| DXGI and WIC | Windows SDK | release |
| Shell32 (icon extraction, IFileOperation, IShellItem) | Windows SDK | release |
| UI Automation | Windows SDK | release |
| OLE drag and drop | Windows SDK | release |
| Per-monitor DPI awareness | Windows SDK | release |

## Restricted sources — reference only, never shipped

Recorded in the manifest's `restricted_sources` array with `shipped: false`.

| Source | Location | License | Decision |
|---|---|---|---|
| Vitro (legacy app) | `vitro/` | MIT | Inspect for behavior only. Never compiled, referenced, or packaged by the new suite. Its icon artwork is inventoried above and stays reference-only until authorship and trademark review. |
| Glassboard | `Glassboard-main/` | MIT | MIT would legally permit code reuse, but the product boundary forbids merging it into the dock release. Behavior and error-handling background only. The checkout is git-ignored and not in the repository. |
| GlassDesk | external (`everlosttime/glassdesk-open-source`) | PolyForm Noncommercial 1.0.0 for new work; Apache-2.0 for historical parts | Layering and test-case ideas only. No code, brand, icons, or material reuse. |
| Seelen UI | external (`eythaann/seelen-ui`) | AGPL-3.0 | Public behavior observation only. No AGPL code is read for implementation. |
| Winstep Nexus | `docs/research/nexus-dock-behavior-spec.md` | Proprietary; black-box notes | R1-R12 acceptance behavior documented from observation. |
| BitDock | `docs/research/mydockfinder-bitdock-deepdive.md` | Proprietary; public notes | Feature inventory only. |
| MyDockFinder | `docs/research/mydockfinder-bitdock-deepdive.md` | Proprietary; public notes | Feature inventory only. |
| transitions.dev | `transitions.dev-main/` (parameter source only) | Transitions.dev license | Motion parameter values are frozen into `docs/specs/motion-tokens.toml`; no site code or assets are packaged. |

## Enforcement

`dock-app/src/core/provenance.rs` exposes:

- `load_asset_manifest()` — reads `dock-app/assets/manifest.json` relative to
  `CARGO_MANIFEST_DIR`, so the working directory does not matter.
- `sha256_file(path)` — used to prove every recorded hash is real.
- `scan_release(paths)` — returns sorted violations for forbidden path
  patterns, forbidden file extensions, and forbidden content markers.

The deny list lives in `dock-app/assets/forbidden_patterns.json`, not in Rust
source, so the runtime tree never spells out the names it must reject.

Tests:

- `dock-app/tests/unit/provenance.rs` — schema completeness, real hashes, the
  frozen 25-icon inventory, spec/prototype/research classification, and
  restricted-source shipping rules.
- `dock-app/tests/desktop/release_boundary.rs` — temp-fixture scanner behavior
  plus a scan of `dock-app/src`, which must stay clean.

Run the boundary check locally:

```
D:/rust/cargo/bin/cargo.exe test --manifest-path D:/dockyh/dock-app/Cargo.toml --test release_boundary
```

## Not yet covered

- A newly drawn 512x512 vector icon master set for release (icons stay
  `reference` until this exists and passes the three-part icon review).
- Trademark review of icon subject matter.
- Font licensing for the release bundling decision (Segoe UI ships with
  Windows; no custom font is currently packaged).
- A `cargo deny` license gate. The current check is `docs/THIRD_PARTY_NOTICES.md`
  plus the metadata in `Cargo.lock`.
