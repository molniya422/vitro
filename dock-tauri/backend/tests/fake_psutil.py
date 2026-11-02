"""Deterministic stand-in for psutil, injected by tests."""


class _Value:
    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)


class FakePsutil:
    """Minimal psutil surface used by sampler.py."""

    def __init__(self, cpu=12.5, recv=2000, sent=900):
        self._cpu = cpu
        self._recv = recv
        self._sent = sent

    def cpu_percent(self, interval=None):
        _ = interval
        return self._cpu

    def virtual_memory(self):
        return _Value(used=8 * 1024**3, total=16 * 1024**3)

    def disk_usage(self, path):
        _ = path
        return _Value(used=100 * 1024**3, total=500 * 1024**3)

    def net_io_counters(self):
        return _Value(bytes_recv=self._recv, bytes_sent=self._sent)
