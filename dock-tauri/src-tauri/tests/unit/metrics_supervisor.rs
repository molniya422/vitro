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
