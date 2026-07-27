# Fixture Contract 005: DependencyAssessment

Status: ready
Date: 2026-07-27

Synthetic producer models are authoritative for module tests because the
assessment is a pure handoff transformation.

Fixtures must cover:

```text
two identical complete reference claims
same filename with different paths
same OriginalCrc with different paths
incomplete occurrences
observed regular-file, missing and unknown candidates
mismatched snapshot hashes
missing and duplicate occurrence observations
```

No fixture may require a real user path or private ALS file. A downstream fake
report consumer must prove that requirement counts and statuses are sufficient
without depending on diagnostic-only extractor fields.
