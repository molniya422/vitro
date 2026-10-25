//! Provenance guard: every inventoried input has a real source, license, and
//! hash, and the icon inventory is frozen at the 25 reference-only PNGs that
//! exist today.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use dock_app::core::provenance::{asset_manifest_path, load_asset_manifest, sha256_file};

const ICON_NAMES: [&str; 25] = [
    "battery", "browser", "calendar", "clip", "clock", "code", "control", "cpu", "doc", "drawer",
    "folder", "gamepad", "mail", "menulist", "music", "net", "notes", "ram", "settings",
    "shelfico", "tasks", "terminal", "trash", "tray", "weather",
];

const SPEC_PATHS: [&str; 3] = [
    "docs/specs/effects-catalog.toml",
    "docs/specs/motion-tokens.toml",
    "docs/specs/macos-parity.toml",
];

const PROTOTYPE_PATHS: [&str; 4] = [
    "docs/prototypes/vitro-effects-gallery.html",
    "docs/prototypes/vitro-m05.html",
    "docs/prototypes/vitro-macos-parity.html",
    "docs/prototypes/vitro-nexus-full.html",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("dock-app lives inside the repository root")
        .to_path_buf()
}

fn manifest_value() -> serde_json::Value {
    let path = asset_manifest_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&source)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

#[test]
fn every_runtime_asset_has_source_license_hash_and_allowed_use() {
    let assets = load_asset_manifest();
    assert!(!assets.is_empty(), "asset manifest must not be empty");
    for asset in assets {
        assert!(!asset.path.is_empty(), "asset path must not be empty");
        assert!(!asset.source.is_empty(), "{} has no source", asset.path);
        assert!(!asset.author.is_empty(), "{} has no author", asset.path);
        assert!(!asset.license.is_empty(), "{} has no license", asset.path);
        assert_eq!(
            asset.sha256.len(),
            64,
            "{} has a non-SHA-256 hash",
            asset.path
        );
        assert!(
            asset
                .sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
            "{} has a non-hex hash",
            asset.path
        );
        assert!(
            matches!(
                asset.allowed_use.as_str(),
                "reference" | "internal" | "release"
            ),
            "{} has unknown allowed_use {:?}",
            asset.path,
            asset.allowed_use
        );
        assert_eq!(
            asset.verified_at, "2026-10-25",
            "{} is not frozen",
            asset.path
        );
    }
}

#[test]
fn every_manifest_entry_matches_the_real_file_on_disk() {
    let root = repo_root();
    for asset in load_asset_manifest() {
        let file = root.join(&asset.path);
        assert!(
            file.is_file(),
            "manifest lists a missing file: {}",
            file.display()
        );
        let actual = sha256_file(&file)
            .unwrap_or_else(|error| panic!("cannot hash {}: {error}", file.display()));
        assert_eq!(actual, asset.sha256, "hash mismatch for {}", asset.path);
    }
}

#[test]
fn icon_inventory_is_frozen_at_25_reference_only_96px_pngs() {
    let assets = load_asset_manifest();
    let icons: Vec<_> = assets
        .iter()
        .filter(|asset| asset.path.starts_with("vitro/assets/icons/"))
        .collect();

    let expected: BTreeSet<String> = ICON_NAMES
        .iter()
        .map(|name| format!("vitro/assets/icons/{name}.png"))
        .collect();
    let actual: BTreeSet<String> = icons.iter().map(|asset| asset.path.clone()).collect();
    assert_eq!(
        actual, expected,
        "icon inventory drifted from the frozen set"
    );

    for icon in icons {
        assert_eq!(icon.allowed_use, "reference", "{} must not ship", icon.path);
        assert_eq!(icon.width, Some(96), "{} has the wrong width", icon.path);
        assert_eq!(icon.height, Some(96), "{} has the wrong height", icon.path);
    }
}

#[test]
fn specifications_are_release_inputs_and_prototypes_are_reference_only() {
    let assets = load_asset_manifest();

    for path in SPEC_PATHS {
        let asset = assets
            .iter()
            .find(|asset| asset.path == path)
            .unwrap_or_else(|| panic!("missing specification record: {path}"));
        assert_eq!(
            asset.allowed_use, "release",
            "{path} must be a release input"
        );
    }

    for path in PROTOTYPE_PATHS {
        let asset = assets
            .iter()
            .find(|asset| asset.path == path)
            .unwrap_or_else(|| panic!("missing prototype record: {path}"));
        assert_eq!(asset.allowed_use, "reference", "{path} must never ship");
    }
}

#[test]
fn research_is_reference_only_and_restricted_sources_never_ship() {
    for asset in load_asset_manifest()
        .iter()
        .filter(|asset| asset.path.starts_with("docs/research/"))
    {
        assert_eq!(
            asset.allowed_use, "reference",
            "{} must never ship",
            asset.path
        );
    }

    let manifest = manifest_value();

    let restricted = manifest["restricted_sources"]
        .as_array()
        .expect("manifest must declare restricted_sources");
    assert!(
        !restricted.is_empty(),
        "restricted_sources must not be empty"
    );
    for source in restricted {
        assert_eq!(
            source["allowed_use"], "reference",
            "restricted source {} must be reference-only",
            source["name"]
        );
        assert_eq!(
            source["shipped"], false,
            "restricted source {} must not ship",
            source["name"]
        );
    }

    let apis = manifest["platform_apis"]
        .as_array()
        .expect("manifest must declare platform_apis");
    assert!(!apis.is_empty(), "platform_apis must not be empty");
    for api in apis {
        assert_eq!(
            api["allowed_use"], "release",
            "platform API {} must be usable in a release",
            api["component"]
        );
    }
}
