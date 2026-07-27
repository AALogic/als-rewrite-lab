import gzip
import json
import tempfile
import unittest
from pathlib import Path

from tools.als_evidence import build_report


class AlsEvidenceTest(unittest.TestCase):
    def test_report_contains_aggregates_without_filenames_or_raw_paths(self):
        xml = b"""<Ableton MajorVersion="5" Creator="Ableton Live">
  <LiveSet>
    <SampleRef>
      <FileRef>
        <Path Value="/Users/private/Secret Project/Kick.wav" />
        <RelativePath Value="../Secret Project/Kick.wav" />
        <RelativePathType Value="1" />
      </FileRef>
    </SampleRef>
  </LiveSet>
</Ableton>"""
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory) / "private-project-name.als"
            fixture.write_bytes(gzip.compress(xml))
            report = build_report(Path(directory))

        serialized = json.dumps(report)
        self.assertEqual(report["document_count"], 1)
        self.assertEqual(
            report["aggregate"]["file_ref_classes"]["active_direct_sample_ref"],
            1,
        )
        self.assertNotIn("private-project-name", serialized)
        self.assertNotIn("/Users/private", serialized)
        self.assertNotIn("Secret Project", serialized)


if __name__ == "__main__":
    unittest.main()
