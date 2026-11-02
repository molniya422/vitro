# Third-Party Notices

License strings below were read from `dock-app/Cargo.lock` through
`cargo metadata` and from each crate's own `Cargo.toml` in the local registry
cache. Versions are the versions actually resolved for `dock-app`.

## Direct dependencies

| Crate | Version | License | Role |
|---|---|---|---|
| `windows` | 0.61.3 | MIT OR Apache-2.0 | Win32, DWM, Direct2D/DirectWrite, DirectComposition, DXGI, Shell, UIA bindings |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | Manifest and settings serialization |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | JSON manifest and settings format |
| `toml` | 0.8.23 | MIT OR Apache-2.0 | TOML settings and spec-input parsing |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | Typed error enums |
| `uuid` | 1.29.0 | Apache-2.0 OR MIT | Stable entry and workspace identifiers |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | SHA-256 for the provenance manifest and release scan |

Development-only:

| Crate | Version | License | Role |
|---|---|---|---|
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | Temporary release-scan fixtures and atomic-write tests |

## Transitive dependencies

`dock-app/Cargo.lock` resolves 68 dependency packages besides `dock-app`
itself. Every one of them declares MIT and/or Apache-2.0, with these documented
variations:

| Crate | Version | License |
|---|---|---|
| `generic-array` | 0.14.7 | MIT |
| `memchr` | 2.8.3 | Unlicense OR MIT |
| `winnow` | 0.7.15 | MIT |
| `zmij` | 1.0.23 | MIT |
| `slab` | 0.4.12 | MIT |
| `version_check` | 0.9.5 | MIT/Apache-2.0 (non-SPDX string; both permissive) |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `rustix`, `linux-raw-sys` | 1.1.5, 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |

The non-MIT/Apache notices are standard permissive or weak-copyleft options on
crates whose permissive alternative is available and used. `rustix`,
`linux-raw-sys`, and `r-efi` are Unix/firmware-only dependencies of `tempfile`
and `getrandom`; they are not compiled into a Windows build.

`thiserror` 2.x carries `thiserror-impl` 2.0.21, `toml` 0.8 carries
`toml_edit` 0.22.27 / `toml_datetime` 0.6.11 / `serde_spanned` 0.6.9, and
`windows` 0.61 carries `windows-core` 0.61.2 and the `windows-*` family. All
declare MIT OR Apache-2.0.

## Verification

```
export CARGO_HOME='D:\rust\cargo' RUSTUP_HOME='D:\rust\rustup'
D:/rust/cargo/bin/cargo.exe metadata --manifest-path D:/dockyh/dock-app/Cargo.toml \
  --format-version 1 --locked
```

Re-run that command and diff this table after any dependency change. There is
no `cargo-deny` gate yet; this file is the manual record.

## Product assets

No third-party UI kit, icon set, font file, theme, or sound is shipped. The
only artwork currently inventoried is the project's own 25 reference-only PNGs,
which are **not** part of a release. See [PROVENANCE.md](PROVENANCE.md).

Competitor and legacy sources (Glassboard, GlassDesk, Seelen, Nexus, BitDock,
MyDockFinder, transitions.dev) are behavior and parameter references only. No
code or asset from them is copied into `dock-app`.

## Project license

The repository is MIT (see [`LICENSE`](../LICENSE)).
