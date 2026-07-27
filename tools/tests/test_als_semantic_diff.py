import gzip
import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS_DIR))

import als_semantic_diff as semantic_diff  # noqa: E402


def document(path_value: str, relative_value: str = "../old.wav") -> bytes:
    xml = f"""<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43">
  <LiveSet>
    <SampleRef>
      <FileRef>
        <RelativePathType Value="1" />
        <RelativePath Value="{relative_value}" />
        <Path Value="{path_value}" />
        <Type Value="1" />
        <OriginalFileSize Value="100" />
        <OriginalCrc Value="42" />
      </FileRef>
    </SampleRef>
  </LiveSet>
</Ableton>
"""
    return gzip.compress(xml.encode("utf-8"), mtime=0)


class SemanticDiffTest(unittest.TestCase):
    def test_path_only_change_passes_path_allowlist(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            before = root / "before.als"
            after = root / "after.als"
            before.write_bytes(document("/old/sample.wav"))
            after.write_bytes(document("/new/sample.wav"))

            report = semantic_diff.compare_documents(before, after, ["Path"])

            self.assertEqual(report["active_changed_fields"], {"Path": 1})
            self.assertTrue(report["active_locator_sequences_equal"])
            self.assertTrue(report["equal_after_allowed_active_redaction"])

    def test_unallowed_relative_path_change_fails_path_only_allowlist(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            before = root / "before.als"
            after = root / "after.als"
            before.write_bytes(document("/old/sample.wav"))
            after.write_bytes(document("/new/sample.wav", "Samples/Imported/sample.wav"))

            report = semantic_diff.compare_documents(before, after, ["Path"])

            self.assertEqual(
                report["active_changed_fields"],
                {"Path": 1, "RelativePath": 1},
            )
            self.assertFalse(report["equal_after_allowed_active_redaction"])


if __name__ == "__main__":
    unittest.main()
