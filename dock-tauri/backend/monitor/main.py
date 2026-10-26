"""Newline-delimited JSON-RPC over stdio. No sockets, no threads."""

import json
import sys


def serve(stdin, stdout, psutil_module):
    from monitor.sampler import Sampler

    sampler = Sampler(psutil_module)
    for line in stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError as error:
            _write(stdout, {"t": "error", "detail": f"malformed request: {error}"})
            continue

        command = request.get("cmd") if isinstance(request, dict) else None
        if command == "sample":
            try:
                payload = sampler.sample()
            except Exception as error:  # noqa: BLE001 - report, never break the loop
                _write(stdout, {"t": "error", "detail": f"sample failed: {error}"})
                continue
            payload["t"] = "sample"
            _write(stdout, payload)
        else:
            _write(stdout, {"t": "error", "detail": f"unknown command: {command}"})


def _write(stdout, payload):
    stdout.write(json.dumps(payload) + "\n")
    stdout.flush()


def main():
    import psutil

    serve(sys.stdin, sys.stdout, psutil_module=psutil)


if __name__ == "__main__":
    main()
