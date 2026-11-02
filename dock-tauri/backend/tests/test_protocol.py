import io
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from monitor.main import serve  # noqa: E402
from fake_psutil import FakePsutil  # noqa: E402


def _lines(stdout):
    return [json.loads(line) for line in stdout.getvalue().splitlines() if line.strip()]


def test_responds_to_sample_with_one_json_line():
    stdin = io.StringIO(json.dumps({"cmd": "sample"}) + "\n")
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    payloads = _lines(stdout)
    assert len(payloads) == 1
    assert payloads[0]["t"] == "sample"
    assert "cpu" in payloads[0]


def test_unknown_command_returns_an_error_line():
    stdin = io.StringIO(json.dumps({"cmd": "explode"}) + "\n")
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    payload = _lines(stdout)[0]
    assert payload["t"] == "error"
    assert "explode" in payload["detail"]


def test_malformed_line_returns_an_error_and_keeps_serving():
    stdin = io.StringIO('not json\n{"cmd": "sample"}\n')
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    payloads = _lines(stdout)
    assert payloads[0]["t"] == "error"
    assert payloads[-1]["t"] == "sample"


def test_eof_exits_cleanly():
    stdin = io.StringIO("")
    stdout = io.StringIO()
    serve(stdin, stdout, psutil_module=FakePsutil())
    assert stdout.getvalue() == ""
