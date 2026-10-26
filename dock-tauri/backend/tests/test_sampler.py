import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from monitor.sampler import Sampler  # noqa: E402
from fake_psutil import FakePsutil  # noqa: E402


def test_sample_reports_all_fields():
    sampler = Sampler(FakePsutil())
    sample = sampler.sample()
    assert sample["cpu"] == 12.5
    assert sample["mem_used"] == 8 * 1024**3
    assert sample["mem_total"] == 16 * 1024**3
    assert sample["disk_used"] == 100 * 1024**3
    assert sample["disk_total"] == 500 * 1024**3
    assert sample["net_rx"] == 2000
    assert sample["net_tx"] == 900


def test_disk_path_defaults_to_the_system_drive_root():
    sampler = Sampler(FakePsutil())
    assert sampler.disk_path in ("C:\\", "/") or sampler.disk_path.endswith("\\")


def test_net_counters_are_cumulative():
    sampler = Sampler(FakePsutil())
    first = sampler.sample()
    second = sampler.sample()
    assert first["net_rx"] == second["net_rx"] == 2000
