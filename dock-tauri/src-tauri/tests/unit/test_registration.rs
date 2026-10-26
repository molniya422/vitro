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
    let manifest =
        std::fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("read Cargo.toml");
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
    assert!(
        missing.is_empty(),
        "test files missing a [[test]] stanza: {missing:?}"
    );
}

#[test]
fn every_declared_test_path_exists() {
    let declared = declared_test_paths();
    let missing: Vec<_> = declared
        .iter()
        .filter(|p| !manifest_dir().join(p).exists())
        .collect();
    assert!(
        missing.is_empty(),
        "declared test paths not on disk: {missing:?}"
    );
}
