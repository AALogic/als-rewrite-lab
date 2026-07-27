import sys
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS_DIR))

import private_path_guard  # noqa: E402


class PrivatePathGuardTest(unittest.TestCase):
    def test_detects_private_home_paths(self) -> None:
        macos_path = "/" + "Users/" + "actual-person/Project/Set.als"
        linux_path = "/" + "home/" + "actual-person/audio.wav"
        windows_path = "C:" + "\\Users\\" + "actual-person\\Set.als"

        violations = private_path_guard.text_violations(
            "\n".join((macos_path, linux_path, windows_path))
        )

        self.assertEqual(
            [violation.category for violation in violations],
            ["macos_home", "linux_home", "windows_home"],
        )

    def test_allows_documented_placeholders(self) -> None:
        text = "\n".join(
            (
                "/Users/me/example.wav",
                "/home/user/example.wav",
                "C:\\Users\\private\\example.als",
            )
        )

        self.assertEqual(private_path_guard.text_violations(text), [])

    def test_repository_tree_contains_no_private_home_paths(self) -> None:
        repository = TOOLS_DIR.parent

        self.assertEqual(private_path_guard.scan_repository(repository), [])


if __name__ == "__main__":
    unittest.main()
