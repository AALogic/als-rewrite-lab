# Fixture Contract 001: ALSReader

Status: draft; v0.1 fixture counts with v0.2 contract addendum  
Date: 2026-06-02  
Scope: expected fixture behavior for ALSReader tests

Update note 2026-06-09:

```text
This file keeps the original v0.1 fixture counts.
Path existence is not an ALSReader v0.2 expectation.
Any legacy all-active-paths-exist expectation belongs to PathVerifier, not to
ALSReader.
```

## 1. Purpose

This file freezes the first known `.als` expectations before product code is
written.

Why:

```text
If we know the expected counts before implementation, the parser cannot silently
define its own meaning of "works".
```

## 2. Required Fixtures

### cziki_after_cas

Path:

```text
tests/fixtures/als/cziki_after_cas.als
```

Source provenance:

```text
/Users/tru.siak/Documents/TRWAJA GRUBE TESTY/cziki Project/cziki.als
```

Expected:

```text
sample_ref_count: 153
active_file_ref_count: 153
unique_active_paths: 126
relative_path_type_counts: 3 -> 33, 5 -> 120
sample_ref_historical_original_file_ref_count: 32
total_original_file_ref_count: 71
legacy_v0_1_all_active_paths_exist: true
```

Why this fixture matters:

```text
It represents a Collect All and Save project where imported samples are
project-local with RelativePathType 3 and Ableton Core Library refs remain type 5.
```

### cziki_before_cas_copy

Path:

```text
tests/fixtures/als/cziki_before_cas.als
```

Source provenance:

```text
experiments/2026-05-30_blabla_stemiki_2_before_cas/project_copy/cziki.als
```

Expected:

```text
sample_ref_count: 153
active_file_ref_count: 153
unique_active_paths: 126
relative_path_type_counts: 1 -> 33, 5 -> 120
sample_ref_historical_original_file_ref_count: 32
total_original_file_ref_count: 71
legacy_v0_1_all_active_paths_exist: true
```

Why this fixture matters:

```text
It represents the same project family before Collect All and Save, with external
active sample paths preserved as RelativePathType 1.
```

### template_zero_active

Path:

```text
tests/fixtures/als/template_zero_active.als
```

Source provenance:

```text
/Users/tru.siak/REMIX/TEMPLATE 1.0_TRUSIAK Project/TEMPLATE 1.0_TRUSIAK.als
```

Expected:

```text
sample_ref_count: 0
active_file_ref_count: 0
unique_active_paths: 0
relative_path_type_counts: none
sample_ref_historical_original_file_ref_count: 0
total_original_file_ref_count: 6
legacy_v0_1_all_active_paths_exist: true
```

Why this fixture matters:

```text
It proves that a project can contain historical/provenance OriginalFileRef nodes
without having active audio SampleRef dependencies.
```

## 3. Optional Fixture

### kombinacja_piejo

Path:

```text
tests/fixtures/als/kombinacja_piejo.als
```

Source provenance:

```text
/Users/tru.siak/first new Project/kombinacja piejo.als
```

Expected:

```text
sample_ref_count: 11
active_file_ref_count: 11
unique_active_paths: 4
relative_path_type_counts: 1 -> 11
sample_ref_historical_original_file_ref_count: 0
total_original_file_ref_count: 7
legacy_v0_1_all_active_paths_exist: true
```

Why this fixture matters:

```text
It is a small project with active external files from Downloads, useful as a
compact external-reference test.
```

## 4. Fixture Copy Decision

Decision:

```text
Copy only the minimum .als fixtures into tests/fixtures/als.
```

Do not copy:

```text
audio folders
Ableton project folders
sample libraries
large generated reports
```

Reason:

```text
ALSReader only needs the .als file to test parsing. Path-existence checks can be
tested separately or treated as environment-dependent.
```

## 5. v0.2 ALSReadModel Contract Expectations

Every valid fixture must produce an `ALSReadModel v0.2` with these top-level
groups:

```text
set_metadata
active_audio_references
historical_references
non_audio_dependency_signals
warnings
errors
```

Required fixture invariants:

```text
active_audio_references count matches the legacy active_file_ref_count
historical references are separated from active audio references
historical references are never rewrite candidates in MVP
raw path fields are preserved when present
raw RelativePathType values are preserved when present
ALSReader output does not contain exists_on_disk
ALSReader output does not classify source_category or storage_state
ALSReader output does not decide matching, copying or rewriting
warnings are allowed for unknown but non-fatal ALS structures
fatal errors produce no trusted ALSReadModel
```

Fixture-specific v0.2 expectations:

```text
cziki_after_cas:
  active_audio_references: 153
  relative_path_type_counts: 3 -> 33, 5 -> 120
  sample_ref_historical_original_file_ref_count: 32
  total_original_file_ref_count: 71

cziki_before_cas_copy:
  active_audio_references: 153
  relative_path_type_counts: 1 -> 33, 5 -> 120
  sample_ref_historical_original_file_ref_count: 32
  total_original_file_ref_count: 71

template_zero_active:
  active_audio_references: 0
  relative_path_type_counts: none
  sample_ref_historical_original_file_ref_count: 0
  total_original_file_ref_count: 6

kombinacja_piejo:
  active_audio_references: 11
  relative_path_type_counts: 1 -> 11
  sample_ref_historical_original_file_ref_count: 0
  total_original_file_ref_count: 7
```

Known missing v0.2 fixtures:

```text
RelativePathType 0 fixture
non-audio dependency signal fixture
plugin/preset evidence fixture
Max for Live dependency signal fixture
ambiguous or unknown ALS structure fixture
```

Boundary rule:

```text
PathVerifier will later test whether active audio paths exist on disk.
ALSReader must only report what the ALS file says.
```
