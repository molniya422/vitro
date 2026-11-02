"""psutil sampling. The psutil module is injected so tests stay deterministic."""

import os


class Sampler:
    def __init__(self, psutil_module, disk_path=None):
        self.psutil = psutil_module
        self.disk_path = disk_path or self._default_disk_path()

    @staticmethod
    def _default_disk_path():
        drive = os.environ.get("SystemDrive", "C:")
        return drive + os.sep

    def sample(self):
        cpu = float(self.psutil.cpu_percent(interval=None))
        memory = self.psutil.virtual_memory()
        disk = self.psutil.disk_usage(self.disk_path)
        net = self.psutil.net_io_counters()
        return {
            "cpu": cpu,
            "mem_used": int(memory.used),
            "mem_total": int(memory.total),
            "disk_used": int(disk.used),
            "disk_total": int(disk.total),
            "net_rx": int(net.bytes_recv),
            "net_tx": int(net.bytes_sent),
        }
