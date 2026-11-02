use dock_tauri_lib::metrics::{RpcError, SampleRpc, SidecarSpec, Supervisor};
use std::path::PathBuf;
use std::time::Duration;

fn fake(mode: &str) -> SampleRpc {
    // cargo sets CARGO_BIN_EXE_<name> for integration tests.
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_fake_sidecar"));
    SampleRpc::spawn(exe, vec![mode.to_string()]).expect("spawn fake sidecar")
}

fn spec(mode: &str) -> SidecarSpec {
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_fake_sidecar"));
    SidecarSpec::new(exe, vec![mode.to_string()])
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
