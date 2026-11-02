//! Sidecar supervisor: owns the Python child and speaks newline JSON-RPC.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
// The sidecar speaks snake_case; the webview model is camelCase.
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Sample {
    pub cpu: f64,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub net_rx: u64,
    pub net_tx: u64,
}

#[derive(Debug)]
pub enum RpcError {
    Spawn(std::io::Error),
    Io(std::io::Error),
    Decode { detail: String },
    Closed,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::Spawn(e) => write!(f, "failed to spawn sidecar: {e}"),
            RpcError::Io(e) => write!(f, "sidecar io error: {e}"),
            RpcError::Decode { detail } => write!(f, "sidecar sent invalid json: {detail}"),
            RpcError::Closed => write!(f, "sidecar closed the pipe"),
        }
    }
}

impl std::error::Error for RpcError {}

pub struct SampleRpc {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
}

impl SampleRpc {
    pub fn spawn(program: PathBuf, args: Vec<String>) -> Result<Self, RpcError> {
        let mut child = Command::new(&program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(RpcError::Spawn)?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take().map(BufReader::new);
        Ok(Self {
            child: Some(child),
            stdin,
            stdout,
        })
    }

    /// Request one sample. The call is blocking; callers run it off the UI thread.
    pub fn sample(&mut self) -> Result<Sample, RpcError> {
        let stdin = self.stdin.as_mut().ok_or(RpcError::Closed)?;
        stdin
            .write_all(b"{\"cmd\":\"sample\"}\n")
            .map_err(RpcError::Io)?;
        stdin.flush().map_err(RpcError::Io)?;

        let stdout = self.stdout.as_mut().ok_or(RpcError::Closed)?;
        let mut line = String::new();
        let read = stdout.read_line(&mut line).map_err(RpcError::Io)?;
        if read == 0 {
            return Err(RpcError::Closed);
        }

        let value: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| RpcError::Decode {
                detail: e.to_string(),
            })?;

        match value.get("t").and_then(|t| t.as_str()) {
            Some("sample") => serde_json::from_value(value).map_err(|e| RpcError::Decode {
                detail: e.to_string(),
            }),
            Some("error") => Err(RpcError::Decode {
                detail: value
                    .get("detail")
                    .and_then(|d| d.as_str())
                    .unwrap_or("sidecar reported an error")
                    .to_string(),
            }),
            _ => Err(RpcError::Decode {
                detail: "unexpected payload".to_string(),
            }),
        }
    }

    pub fn shutdown(&mut self) {
        self.stdin.take();
        self.stdout.take();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for SampleRpc {
    fn drop(&mut self) {
        self.shutdown();
    }
}

use std::time::{Duration, Instant};

/// How to launch the sidecar, plus the minimum interval between respawns.
pub struct SidecarSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cooldown: Duration,
}

impl SidecarSpec {
    pub fn new(program: PathBuf, args: Vec<String>) -> Self {
        Self {
            program,
            args,
            // Backoff floor: never spawn more than once per two seconds.
            cooldown: Duration::from_secs(2),
        }
    }
}

/// The sidecar is stopped after this long without a sample. The monitor route
/// re-warms it on open, so the first sample stays fast without paying for a
/// Python process while the dock is idle.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(45);

/// Owns at most one sidecar and respawns it on demand after a failure.
pub struct Supervisor {
    spec: SidecarSpec,
    rpc: Option<SampleRpc>,
    last_attempt: Option<Instant>,
    last_used: Option<Instant>,
}

impl Supervisor {
    pub fn new(spec: SidecarSpec) -> Self {
        Self {
            spec,
            rpc: None,
            last_attempt: None,
            last_used: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.rpc.is_some()
    }

    /// Start the child without sampling, so the first monitor open does not
    /// pay the PyInstaller boot cost.
    pub fn warm(&mut self) {
        if self.ensure_started().is_ok() {
            self.last_used = Some(Instant::now());
        }
    }

    fn ensure_started(&mut self) -> Result<(), RpcError> {
        if self.rpc.is_some() {
            return Ok(());
        }
        if let Some(last) = self.last_attempt {
            if last.elapsed() < self.spec.cooldown {
                return Err(RpcError::Closed);
            }
        }
        self.last_attempt = Some(Instant::now());
        self.rpc = Some(SampleRpc::spawn(
            self.spec.program.clone(),
            self.spec.args.clone(),
        )?);
        Ok(())
    }

    /// Request a sample, starting the sidecar first if needed.
    ///
    /// On failure the broken child is dropped so the next call can respawn it,
    /// subject to `cooldown`.
    pub fn sample(&mut self) -> Result<Sample, RpcError> {
        self.ensure_started()?;
        self.last_used = Some(Instant::now());

        let rpc = self.rpc.as_mut().expect("just started");
        match rpc.sample() {
            Ok(sample) => Ok(sample),
            Err(error) => {
                rpc.shutdown();
                self.rpc = None;
                Err(error)
            }
        }
    }

    /// Stop the child after `IDLE_TIMEOUT` without use. Returns whether a
    /// process was stopped.
    pub fn reap_if_idle(&mut self) -> bool {
        if self.rpc.is_none() {
            return false;
        }
        let idle = self
            .last_used
            .map(|last| last.elapsed())
            .unwrap_or_default();
        if idle < IDLE_TIMEOUT {
            return false;
        }
        self.shutdown();
        true
    }

    pub fn shutdown(&mut self) {
        if let Some(mut rpc) = self.rpc.take() {
            rpc.shutdown();
        }
    }
}
