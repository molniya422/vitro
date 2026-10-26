"""Entry point used by the Rust supervisor.

Keeps the ``monitor`` package importable regardless of the working directory the
parent process happens to use.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from monitor.main import main  # noqa: E402

if __name__ == "__main__":
    main()
