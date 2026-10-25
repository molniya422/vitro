//! Release-boundary guard: forbidden legacy and competitor sources, content
//! markers, and prototype files must never reach runtime or staging trees.
//!
//! The fixtures are staged in a temporary directory so the suite never scans
//! the whole repository and never depends on untracked checkouts.

use std::fs;
use std::path::{Path, PathBuf};

use dock_app::core::provenance::scan_release;
use tempfile::TempDir;

fn write(root: &Path, relative: &str, contents: &str) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file has a parent"))
        .unwrap_or_else(|error| panic!("cannot create {}: {error}", path.display()));
    fs::write(&path, contents)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
    path
}

/// Build a release fixture containing every forbidden shape. The temporary
/// directory is returned so it outlives the scan.
fn stage_release_fixture() -> (TempDir, Vec<PathBuf>) {
    let temp = TempDir::new().expect("temporary release fixture directory");
    let files = vec![
        write(temp.path(), "vitro/src/main.rs", "fn main() {}\n"),
        write(temp.path(), "vitro/src-tauri/tauri.conf.json", "{}\n"),
        write(
            temp.path(),
            "Glassboard-main/apps/dock/src/lib.rs",
            "pub fn dock() {}\n",
        ),
        write(
            temp.path(),
            "Seelen/apps/settings/lib.rs",
            "pub fn x() {}\n",
        ),
        write(
            temp.path(),
            "runtime/notes.txt",
            "behavior ported from GlassDesk; reference-only\n",
        ),
        write(
            temp.path(),
            "runtime/staging/panel.html",
            "<!doctype html><html><body></body></html>\n",
        ),
        write(temp.path(), "runtime/motion.js", "export const x = 1;\n"),
    ];
    (temp, files)
}

#[test]
fn release_scan_rejects_reference_sources_and_old_runtime_paths() {
    let (_temp, files) = stage_release_fixture();
    let violations = scan_release(&files);

    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("vitro/src")),
        "old runtime path was not rejected: {violations:#?}"
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("vitro/src-tauri")),
        "Tauri shell path was not rejected: {violations:#?}"
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("Seelen")),
        "Seelen path was not rejected: {violations:#?}"
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("Glassboard-main")),
        "competitor checkout was not rejected: {violations:#?}"
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("GlassDesk")),
        "competitor content marker was not rejected: {violations:#?}"
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("reference-only")),
        "reference-only content marker was not rejected: {violations:#?}"
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains(".html")),
        "prototype file extension was not rejected: {violations:#?}"
    );
    assert!(
        violations.iter().any(|violation| violation.contains(".js")),
        "script prototype file extension was not rejected: {violations:#?}"
    );
}

#[test]
fn release_scan_accepts_a_clean_runtime_tree() {
    let temp = TempDir::new().expect("temporary clean fixture directory");
    write(temp.path(), "runtime/panel.rs", "pub fn render() {}\n");
    write(
        temp.path(),
        "runtime/README.txt",
        "Native Dock App runtime. No legacy or competitor code is used.\n",
    );

    let files = vec![temp.path().to_path_buf()];
    let violations = scan_release(&files);
    assert!(
        violations.is_empty(),
        "clean tree produced violations: {violations:#?}"
    );
}

#[test]
fn release_scan_skips_build_output_directories() {
    let temp = TempDir::new().expect("temporary build output fixture directory");
    write(
        temp.path(),
        "target/debug/vitro/src/generated.rs",
        "fn generated() {}\n",
    );
    write(
        temp.path(),
        "target/debug/prototype.html",
        "<!doctype html><html></html>\n",
    );

    let violations = scan_release(&[temp.path().to_path_buf()]);
    assert!(
        violations.is_empty(),
        "build output must be skipped: {violations:#?}"
    );
}

#[test]
fn release_scan_ignores_similar_names_that_are_not_forbidden_segments() {
    let temp = TempDir::new().expect("temporary lookalike fixture directory");
    write(temp.path(), "myvitro/src/lib.rs", "pub fn x() {}\n");
    write(temp.path(), "runtime/glassdesk_notes.rs", "pub fn y() {}\n");

    let violations = scan_release(&[temp.path().to_path_buf()]);
    assert!(
        violations.is_empty(),
        "lookalike names must not be rejected: {violations:#?}"
    );
}

#[test]
fn runtime_and_staging_trees_contain_no_forbidden_references() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut roots = vec![crate_root.join("src"), crate_root.join("assets")];
    for staging in ["staging", "dist", "release-staging"] {
        let candidate = crate_root.join(staging);
        if candidate.is_dir() {
            roots.push(candidate);
        }
    }

    let violations = scan_release(&roots);
    assert!(
        violations.is_empty(),
        "runtime or staging tree references forbidden resources: {violations:#?}"
    );
}
