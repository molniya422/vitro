//! Test double for the Python sidecar. Behaviour is selected by the first arg:
//!   ok       - answers every sample request
//!   crash    - exits immediately after the first request
//!   garbage  - writes a non-JSON line, then answers normally
//!   silent   - reads but never answers

use std::io::{BufRead, Write};

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "ok".to_string());
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    if mode == "crash" {
        // Read one line, then die without answering.
        let mut line = String::new();
        let _ = stdin.lock().read_line(&mut line);
        std::process::exit(3);
    }

    let mut first = true;
    for line in stdin.lock().lines() {
        let Ok(_line) = line else { break };
        if mode == "silent" {
            continue;
        }
        if mode == "garbage" && first {
            first = false;
            writeln!(stdout, "this is not json").unwrap();
            stdout.flush().unwrap();
            continue;
        }
        first = false;
        writeln!(
            stdout,
            "{{\"t\":\"sample\",\"cpu\":1.0,\"mem_used\":1,\"mem_total\":2,\
             \"disk_used\":1,\"disk_total\":2,\"net_rx\":1,\"net_tx\":1}}"
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}
