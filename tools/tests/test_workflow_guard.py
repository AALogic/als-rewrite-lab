import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS_DIR))

import workflow_guard as guard  # noqa: E402


class WorkspaceTestDiscoveryTest(unittest.TestCase):
    def test_rust_tests_are_discovered_across_workspace_crates(self) -> None:
        original_root = guard.ROOT
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                for crate, test_name in (
                    ("rescue_core", "core_contract_is_kept"),
                    ("rescue_analyzer", "analyzer_contract_is_kept"),
                ):
                    tests_dir = root / "crates" / crate / "tests"
                    tests_dir.mkdir(parents=True)
                    (tests_dir / "contract.rs").write_text(
                        f"#[test]\nfn {test_name}() {{}}\n",
                        encoding="utf-8",
                    )

                guard.ROOT = root

                self.assertEqual(
                    set(guard.rust_test_cases()),
                    {"core_contract_is_kept", "analyzer_contract_is_kept"},
                )
        finally:
            guard.ROOT = original_root


class QualitySourceCoverageTest(unittest.TestCase):
    def test_private_implementation_file_is_checked_by_quality_glob(self) -> None:
        original_root = guard.ROOT
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                root = Path(temp_dir)
                source_dir = root / "crates" / "core" / "src"
                source_dir.mkdir(parents=True)
                (source_dir / "api.rs").write_text("pub fn api() {}\n", encoding="utf-8")
                (source_dir / "implementation.rs").write_text(
                    "\n".join(["fn detail() {", *["    let _value = 1;" for _ in range(8)], "}"]),
                    encoding="utf-8",
                )
                guard.ROOT = root
                result = guard.GuardResult("verify-module", "fixture")
                contract = {
                    "expected_source_files": ["crates/core/src/api.rs"],
                    "quality_source_globs": ["crates/core/src/implementation.rs"],
                    "quality_limits": {"max_source_file_lines": 5},
                }

                guard.check_quality_limits(result, contract)

                self.assertFalse(result.passed)
                self.assertTrue(
                    any(
                        "implementation.rs" in failure and "source file too long" in failure
                        for failure in result.failures
                    )
                )
                self.assertTrue(all("api.rs" not in failure for failure in result.failures))
        finally:
            guard.ROOT = original_root


if __name__ == "__main__":
    unittest.main()
