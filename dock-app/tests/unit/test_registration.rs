//! Registration guard: every test file under `tests/` must be declared as an
//! explicit `[[test]]` target. Cargo only auto-discovers `tests/*.rs`, so a
//! file in a subdirectory without a stanza would silently run zero tests.

use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn manifest_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR)
}

/// Test source files on disk, relative to the crate root with forward slashes
/// (`[[test]] path` values use that form).
fn discovered_test_files() -> Vec<String> {
    let mut files = Vec::new();
    collect_rs_files(&manifest_dir().join("tests"), &mut files);
    files.sort();
    files
}

fn collect_rs_files(dir: &Path, files: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(relative_slash_path(&path));
        }
    }
}

fn relative_slash_path(path: &Path) -> String {
    path.strip_prefix(manifest_dir())
        .unwrap_or_else(|_| panic!("{} is outside the crate root", path.display()))
        .to_string_lossy()
        .replace('\\', "/")
}

/// `path` values declared by `[[test]]` stanzas in `Cargo.toml`.
fn declared_test_paths() -> Vec<String> {
    let manifest_path = manifest_dir().join("Cargo.toml");
    let manifest_source = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest_path.display()));
    let manifest: Value = toml::from_str(&manifest_source)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", manifest_path.display()));
    manifest
        .get("test")
        .and_then(Value::as_array)
        .map(|targets| {
            targets
                .iter()
                .filter_map(|target| target.get("path").and_then(Value::as_str))
                .map(|path| path.replace('\\', "/"))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_test_file_is_declared_as_a_cargo_test_target() {
    let declared = declared_test_paths();
    let undeclared: Vec<String> = discovered_test_files()
        .into_iter()
        .filter(|file| !declared.contains(file))
        .collect();
    assert!(
        undeclared.is_empty(),
        "test files missing a [[test]] stanza in Cargo.toml: {undeclared:?}"
    );
}

#[test]
fn every_declared_test_target_exists_on_disk() {
    let missing: Vec<String> = declared_test_paths()
        .into_iter()
        .filter(|path| !manifest_dir().join(path).is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "[[test]] paths in Cargo.toml that do not exist: {missing:?}"
    );
}
