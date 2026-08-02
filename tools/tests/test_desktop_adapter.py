import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TAURI_COMMANDS = ROOT / "apps" / "rescue-desktop" / "src-tauri" / "src" / "lib.rs"


class DesktopAdapterTest(unittest.TestCase):
    def test_blocking_desktop_services_are_offloaded(self) -> None:
        source = TAURI_COMMANDS.read_text(encoding="utf-8")

        for command in ("analyze_project", "prepare_copy", "execute_copy"):
            self.assertRegex(source, rf"async\s+fn\s+{command}\s*\(")

        self.assertGreaterEqual(source.count("spawn_blocking"), 3)


if __name__ == "__main__":
    unittest.main()
