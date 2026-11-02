use dock_tauri_lib::commands::resolve_sidecar;
use std::path::{Path, PathBuf};

fn bundled(dir: &Path) -> PathBuf {
    dir.join("monitor.exe")
}

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

#[test]
fn a_directory_named_monitor_exe_is_not_the_bundled_binary() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(bundled(tmp.path())).unwrap();
    let spec = resolve_sidecar(tmp.path(), Path::new("Z:/py.exe"), Path::new("Z:/run.py"));
    assert_eq!(spec.program, PathBuf::from("Z:/py.exe"));
    assert_eq!(spec.args, vec!["Z:/run.py".to_string()]);
}
