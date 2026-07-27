import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS_DIR))

import private_path_guard  # noqa: E402


class PrivatePathGuardTest(unittest.TestCase):
    def initialize_repository(self, repository: Path) -> None:
        subprocess.run(
            ["git", "init", "-q"],
            cwd=repository,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

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

    def test_detects_private_experiment_provenance(self) -> None:
        text = "experiments/" + "2026-01-01_private/copies/" + "sensitive-set.als"

        violations = private_path_guard.text_violations(text)

        self.assertTrue(
            any(violation.category == "private_provenance" for violation in violations)
        )

    def test_detects_configured_private_identifier_in_tracked_path(self) -> None:
        with tempfile.TemporaryDirectory() as raw_repository:
            repository = Path(raw_repository)
            self.initialize_repository(repository)
            tracked = repository / "sensitive_corpus.als"
            tracked.write_text("synthetic", encoding="utf-8")
            subprocess.run(["git", "add", "--", tracked.name], cwd=repository, check=True)
            hashes = frozenset(
                {private_path_guard.identifier_hash("sensitive corpus")}
            )

            violations = private_path_guard.scan_repository(repository, hashes)

        self.assertTrue(
            any(violation.category == "private_identifier" for violation in violations)
        )

    def test_rejects_tracked_binary_media_case_insensitively(self) -> None:
        with tempfile.TemporaryDirectory() as raw_repository:
            repository = Path(raw_repository)
            self.initialize_repository(repository)
            tracked_names = (
                "fixture.ALS",
                "fixture.OgG",
                "fixture.AaC",
                "fixture-upper.SD2",
                "fixture-lower.sd2",
            )
            for tracked_name in tracked_names:
                (repository / tracked_name).write_bytes(b"media\0payload")
            subprocess.run(
                ["git", "add", "--", *tracked_names],
                cwd=repository,
                check=True,
            )

            violations = private_path_guard.scan_repository(repository)

        self.assertEqual(
            {
                violation.path.as_posix()
                for violation in violations
                if violation.category == "tracked_private_media"
            },
            set(tracked_names),
        )

    @unittest.skipIf(sys.platform == "win32", "tracked symlink fixture requires Unix")
    def test_scans_tracked_symlink_target_payload(self) -> None:
        with tempfile.TemporaryDirectory() as raw_repository:
            repository = Path(raw_repository)
            self.initialize_repository(repository)
            tracked = repository / "private-link"
            target = "/" + "Users/" + "actual-person/private.als"
            os.symlink(target, tracked)
            subprocess.run(["git", "add", "--", tracked.name], cwd=repository, check=True)

            violations = private_path_guard.scan_repository(repository)

        self.assertTrue(
            any(violation.category == "macos_home" for violation in violations)
        )

    def test_repository_tree_passes_private_data_policy(self) -> None:
        repository = TOOLS_DIR.parent

        self.assertEqual(private_path_guard.ALLOWED_TRACKED_MEDIA_FIXTURES, frozenset())
        self.assertEqual(private_path_guard.scan_repository(repository), [])


if __name__ == "__main__":
    unittest.main()
