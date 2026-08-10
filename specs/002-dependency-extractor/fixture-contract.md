# Fixture Contract 002: DependencyExtractor

Status: draft v0.1  
Date: 2026-06-09  
Scope: expected behavior for DependencyExtractor tests

## 1. Purpose

Define concrete fixture expectations for turning ALSReader output into
DependencyExtractionResult v0.1.

This contract exists to prevent implementation drift:

```text
DependencyExtractor normalizes active audio refs.
DependencyExtractor does not verify disk existence.
DependencyExtractor does not classify source category.
DependencyExtractor does not deduplicate.
```

## 2. Fixture Sources

Core automated tests generate minimal gzip/XML ALS fixtures through ALSReader.
They cover active refs, zero-active projects, historical refs, duplicate paths
and raw RelativePathType preservation without requiring private projects.

The following files remain an optional private evidence corpus:

```text
tests/fixtures/als/private_fixture_001_before_collect.als
tests/fixtures/als/private_fixture_001_after_collect.als
tests/fixtures/als/synthetic_fixture_zero_active.als
tests/fixtures/als/private_fixture_002_external_refs.als
<private-corpus-root>/private_corpus_fixture_014.als
```

Synthetic ALSReadModel fixtures may be constructed in Rust tests for:

```text
incomplete active refs
unsupported model version
input model with fatal errors
duplicate ref edge cases if needed
```

## 3. Expected Fixture Behavior

### 3.1 `private_fixture_002_external_refs.als`

Purpose:

```text
Compact fixture with active audio refs and repeated paths.
Good for one-ref-to-one-dependency and duplicate-not-deduped tests.
```

Expected:

```text
ALSReader active_audio_ref_count: 11
DependencyExtractor dependency_count: 11
DependencyRef order follows active_audio_references order
dependency_id values:
  dep_audio_000000
  dep_audio_000001
  ...
  dep_audio_000010
```

Required assertions:

```text
duplicate raw paths are not deduplicated
raw_path is preserved
raw_relative_path is preserved
relative_path_type is preserved
source_kind is preserved
original_file_size is preserved
original_crc is preserved as value, not identity proof
path_basis is raw_path_and_raw_relative_path for refs with both path fields
```

### 3.2 `synthetic_fixture_zero_active.als`

Purpose:

```text
Zero active audio dependency case.
```

Expected:

```text
ALSReader active_audio_ref_count: 0
DependencyExtractor dependencies: []
DependencyExtractor errors: []
ignored_input_summary.historical_refs_ignored_count > 0
```

Rule:

```text
Zero dependencies is valid.
```

### 3.3 `private_fixture_001_before_collect.als`

Purpose:

```text
Larger fixture with active refs and historical refs.
Good for count preservation and ignored summary.
```

Expected:

```text
ALSReader active_audio_ref_count: 153
DependencyExtractor dependency_count: 153
ALSReader historical_ref_count: 71
ignored_input_summary.historical_refs_ignored_count: 71
```

Required assertions:

```text
historical_refs do not become DependencyRef records
DependencyExtractor does not check path existence
DependencyExtractor does not add source_category
DependencyExtractor does not add existence_status
```

### 3.4 `private_fixture_001_after_collect.als`

Purpose:

```text
Compare before/after CAS-like paths without path existence checks.
```

Expected:

```text
DependencyExtractor preserves raw fields only.
No path verification.
No source classification.
No package readiness decision.
```

### 3.5 Corpus Copy With `RelativePathType 0`

Fixture:

```text
<private-corpus-root>/private_corpus_fixture_014.als
```

Expected:

```text
ALSReader active_audio_ref_count: 1800
DependencyExtractor dependency_count: 1800
RelativePathType 0 is preserved exactly
RelativePathType 0 is not classified or interpreted here
```

## 4. Synthetic Test Models

### 4.1 Incomplete Reference

Construct an ALSReadModel v0.2 with one active audio ref:

```text
raw_path: null
raw_relative_path: null
filename: null
relative_path_type: null
```

Expected DependencyRef:

```text
extraction_status: incomplete
path_basis: path_unavailable
warnings include DEPENDENCY_PATH_UNAVAILABLE
dependency is still returned
```

### 4.2 Raw Path Only

Expected:

```text
extraction_status: extracted
path_basis: raw_path
warnings include DEPENDENCY_RAW_RELATIVE_PATH_MISSING
```

### 4.3 Raw Relative Path Only

Expected:

```text
extraction_status: extracted
path_basis: raw_relative_path
warnings include DEPENDENCY_RAW_PATH_MISSING
```

### 4.4 Unsupported ALSReadModel Version

Construct:

```text
set_metadata.als_read_model_version: 999
```

Expected:

```text
errors include DEPENDENCY_UNSUPPORTED_READ_MODEL_VERSION
dependencies: []
extraction_metadata.error_count > 0
```

### 4.5 Input With Fatal Reader Errors

Construct:

```text
errors: [ALSReadError]
```

Expected:

```text
errors include DEPENDENCY_NO_TRUSTED_ALS_MODEL
dependencies: []
extraction_metadata.error_count > 0
```

## 5. Forbidden Output Fields In v0.1

DependencyExtractor v0.1 output must not claim:

```text
exists_on_disk
resolved_path
existence_status
source_category
risk_flags
match_candidates
copy_decision
rewrite_decision
```

If these appear in a v0.1 test output, the test should fail unless the module
spec has been intentionally updated.

Ownership:

```text
candidate path observations belong to PathObservation.
grouping and project-level status belong to DependencyAssessment.
match_candidates and resolution decisions belong to AssetResolution.
copy_decision belongs to PackagePlanner.
rewrite_decision belongs to ALSRewriter / PackagePlanner contracts.
```

## 6. Determinism Expectations

For the same ALSReadModel:

```text
two runs produce identical dependencies
two runs produce identical dependency_id values
two runs produce identical warnings/errors except intentionally excluded
run metadata
```

`dependency_id` format:

```text
dep_audio_000000
dep_audio_000001
dep_audio_000002
...
```

## 7. Safety Expectations

DependencyExtractor tests must not require real sample files.

Rules:

```text
No sample folders copied for this module.
No filesystem existence checks.
No writes except normal test output/temp handled by test framework.
No modification of ALS fixtures.
```

## 8. Fixture Contract Closeout

This fixture contract is accepted when tests prove:

```text
active refs become dependencies
zero active refs is valid
duplicates are not deduplicated
incomplete refs are preserved with warnings
historical refs are ignored with summary
non-audio signals are ignored with summary
unsupported input version is rejected
output is deterministic
forbidden downstream fields are absent
```
