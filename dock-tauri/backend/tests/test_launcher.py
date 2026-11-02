"""The launcher is what the Rust supervisor actually spawns, so exercise it as a
subprocess rather than importing it — that is the only way to catch
working-directory or sys.path problems."""

import json
import subprocess
import sys
from pathlib import Path

BACKEND = Path(__file__).resolve().parent.parent
LAUNCHER = BACKEND / "run_sidecar.py"


def test_launcher_answers_a_sample_from_an_unrelated_working_directory(tmp_path):
    completed = subprocess.run(
        [sys.executable, str(LAUNCHER)],
        input='{"cmd":"sample"}\n',
        capture_output=True,
        text=True,
        timeout=30,
        cwd=tmp_path,
    )
    assert completed.returncode == 0, completed.stderr
    lines = [line for line in completed.stdout.splitlines() if line.strip()]
    assert len(lines) == 1
    payload = json.loads(lines[0])
    assert payload["t"] == "sample"
    assert payload["mem_total"] > 0


def test_launcher_exits_cleanly_on_eof(tmp_path):
    completed = subprocess.run(
        [sys.executable, str(LAUNCHER)],
        input="",
        capture_output=True,
        text=True,
        timeout=30,
        cwd=tmp_path,
    )
    assert completed.returncode == 0
    assert completed.stdout.strip() == ""
