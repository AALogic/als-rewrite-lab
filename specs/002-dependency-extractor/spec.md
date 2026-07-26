# Module Spec 002: DependencyExtractor

Status: draft v0.1 specification  
Date: 2026-06-09  
Owner: Product Office / Codex  
Implementation target: Rust core  
Product source: `PRODUCT_SPEC.md`  
Upstream module: `specs/001-als-reader/spec.md`

## 1. Responsibility

`DependencyExtractor` converts trusted ALSReader output into normalized project
dependency records.

Primary responsibility:

```text
Turn ALSReadModel v0.2 active_audio_references into DependencyExtractionResult
v0.1 without touching the filesystem and without making package decisions.
```

It is the second technical module in the product pipeline.

It exists between:

```text
ALSReader
-> DependencyExtractor
-> PathObservation
-> DependencyAssessment / read-only report
-> later matching/planning/copy/rewrite/validation/manifest
```

## 2. Product Traceability

Product promise:

```text
Help a music producer understand, recover and safely package Ableton projects
whose audio dependencies are spread across messy local folders.
```

Supported use case:

```text
Take raw active sample facts from an ALS file and turn them into a stable list
of audio dependencies that later modules can verify, classify and report.
```

MVP classification:

```text
MVP
```

## 3. Scope Decision

`DependencyExtractor v0.1` handles only active audio samples.

In scope:

```text
ALSReadModel.active_audio_references
audio sample dependencies
raw ALS path fields
ALS audio evidence fields
warnings that affect trust in extracted active audio references
```

Out of scope:

```text
historical_refs as active dependencies
non_audio_dependency_signals as dependencies
VST dependencies
plugin presets
Ableton device presets
Max for Live devices
Core Library / User Library classification
path existence checks
filesystem scanning
sample matching
copy decisions
rewrite decisions
```

Reason:

```text
This module is a normalizer, not a verifier, classifier, matcher, planner or
rewriter.
```

## 4. Inputs

Primary input:

```text
ALSReadModel v0.2
```

Required upstream contract:

```text
ALSReadModel.set_metadata.als_read_model_version must be 0.2.
ALSReadModel must be trusted: no fatal ALSReader errors.
active_audio_references must be available as a list.
```

Allowed core handoff fields from ALSReader:

```text
From set_metadata:
  source_als_path
  source_file_hash
  reader_version
  als_read_model_version

From active_audio_references:
  ref_id
  source_kind
  raw_path
  raw_relative_path
  relative_path_type
  file_type
  filename
  extension
  original_file_size
  original_crc
  default_duration
  default_sample_rate
  usage_context
  xml_context
  rewrite_support_status
  warnings

From warnings/errors:
  warning/error code and severity when they affect trust in extracted
  active_audio_references.
```

Important rule:

```text
DependencyExtractor may not require non-core_handoff ALSReader fields unless
this module spec and its contract tests are updated first.
```

## 5. Outputs

Primary output:

```text
DependencyExtractionResult v0.1
```

`DependencyExtractionResult` contains:

```text
extraction_metadata
dependencies: list<DependencyRef>
ignored_input_summary
warnings: list<DependencyExtractionWarning>
errors: list<DependencyExtractionError>
```

Why not return only `list<DependencyRef>`:

```text
The module should be auditable. It must explicitly record what it extracted,
what it ignored by scope, and whether incomplete references were preserved with
warnings.
```

## 6. Data Contracts

### 6.1 DependencyExtractionResult v0.1

Fields:

```text
extraction_metadata: DependencyExtractionMetadata
dependencies: list<DependencyRef>
ignored_input_summary: IgnoredInputSummary
warnings: list<DependencyExtractionWarning>
errors: list<DependencyExtractionError>
```

Rules:

```text
dependencies are ordered exactly like ALSReadModel.active_audio_references.
0 dependencies is a valid result.
errors mean the extraction result is not trusted.
warnings mean extraction completed but some dependency records are incomplete
or downstream should be cautious.
```

### 6.2 DependencyExtractionMetadata v0.1

Fields:

```text
extractor_version
dependency_ref_version
input_als_read_model_version
source_als_path
source_project_root
source_file_hash
dependency_count
warning_count
error_count
```

Rules:

```text
Metadata records extraction facts only.
It does not classify dependency risk or readiness.
source_project_root remains a nullable compatibility field and must not be
interpreted as a discovered or confirmed Ableton Project root.
```

### 6.3 DependencyRef v0.1

One `DependencyRef` represents one ReferenceOccurrence compatibility record
created from one active audio sample reference in
`ALSReadModel.active_audio_references`.

Fields:

```text
dependency_id
dependency_kind
als_ref_id
source_kind
raw_path
raw_relative_path
relative_path_type
file_type
filename
extension
original_file_size
original_crc
default_duration
default_sample_rate
usage_context
xml_context
rewrite_support_status
extraction_status
path_basis
evidence_status
evidence_notes
warnings
```

Fields intentionally not present in `DependencyRef v0.1`:

```text
resolved_path
existence_status
source_category
risk_flags
match_candidates
copy_decision
rewrite_decision
```

Ownership:

```text
PathObservation owns candidate-path availability observations, not resolved_path.
DependencyAssessment owns grouping and project-level availability/risk assessment.
match_candidates and resolution decisions belong to AssetResolution.
copy_decision belongs to PackagePlanner.
rewrite_decision belongs to ALSRewriter / PackagePlanner contracts.
```

Allowed `dependency_kind` values for v0.1:

```text
audio_sample
```

Allowed `extraction_status` values:

```text
extracted
incomplete
```

Allowed `path_basis` values:

```text
raw_path
raw_relative_path
raw_path_and_raw_relative_path
path_unavailable
```

Allowed `evidence_status` values:

```text
extracted_from_als
extracted_with_warnings
```

Required identity evidence policy:

```text
original_file_size is supporting evidence.
original_crc is weak supporting evidence only.
original_crc must not be treated as proof of file identity.
original_file_size, original_crc, default_duration and default_sample_rate must
be preserved as raw string values from ALSReader.
DependencyExtractor v0.1 must not parse or coerce those values into numeric
types.
```

Recommended `evidence_notes` when fields exist:

```text
original_file_size_supporting_signal
original_crc_weak_identity_signal
```

### 6.4 IgnoredInputSummary v0.1

Fields:

```text
historical_refs_ignored_count
non_audio_dependency_signals_ignored_count
input_warnings_seen_count
input_errors_seen_count
ignored_scope_notes
```

Required notes:

```text
historical_refs_ignored_by_v0_1_scope
non_audio_signals_ignored_by_v0_1_scope
```

Rules:

```text
Ignoring historical_refs and non_audio_dependency_signals is intentional in
v0.1. It must be visible in the output summary, not silent.
```

### 6.5 DependencyExtractionWarning v0.1

Fields:

```text
warning_id
warning_code
severity
message
dependency_id
als_ref_id
evidence_status
```

Optionality:

```text
dependency_id is optional because some warnings can apply to the whole input.
als_ref_id is optional because some warnings can apply before a DependencyRef is
created.
```

Suggested warning codes:

```text
DEPENDENCY_PATH_UNAVAILABLE
DEPENDENCY_RAW_PATH_MISSING
DEPENDENCY_RAW_RELATIVE_PATH_MISSING
DEPENDENCY_RELATIVE_PATH_TYPE_MISSING
DEPENDENCY_FILENAME_MISSING
DEPENDENCY_UPSTREAM_REF_WARNING_PROPAGATED
DEPENDENCY_ORIGINAL_CRC_WEAK_SIGNAL
```

Rules:

```text
Warnings must be structured.
Warnings must not become filesystem claims.
Warnings may mark a dependency incomplete, but incomplete dependencies are still
returned.
```

### 6.6 DependencyExtractionError v0.1

Fields:

```text
error_code
message
input_model_version
```

Error codes:

```text
DEPENDENCY_INPUT_INVALID
DEPENDENCY_UNSUPPORTED_READ_MODEL_VERSION
DEPENDENCY_REQUIRED_FIELD_MISSING
DEPENDENCY_NO_TRUSTED_ALS_MODEL
```

Rules:

```text
0 dependencies is not an error.
Missing sample files are not an error here because this module does not touch
the filesystem.
Fatal ALSReader errors mean no trusted ALSReadModel should be consumed.
Unsupported ALSReadModel version returns dependencies: [] and a structured
DEPENDENCY_UNSUPPORTED_READ_MODEL_VERSION error.
Fatal ALSReader errors return dependencies: [] and a structured
DEPENDENCY_NO_TRUSTED_ALS_MODEL error.
```

## 7. Algorithm

The algorithm must be deterministic.

Steps:

```text
1. Validate ALSReadModel version.
2. Validate model trust state.
3. Initialize DependencyExtractionResult metadata.
4. For every active_audio_reference in input order:
   a. Create one DependencyRef.
   b. Set dependency_id deterministically from active reference position:
      dep_audio_000000, dep_audio_000001, ...
   c. Copy core handoff fields.
   d. Compute path_basis from raw_path/raw_relative_path presence.
   e. Set extraction_status.
   f. Preserve upstream warnings and add extraction warnings when needed.
   g. Add evidence notes for OriginalFileSize and OriginalCrc when present.
5. Record ignored_input_summary.
6. Return result.
```

Path basis rules:

```text
raw_path present and raw_relative_path present:
  path_basis = raw_path_and_raw_relative_path
  extraction_status = extracted

raw_path present and raw_relative_path missing:
  path_basis = raw_path
  extraction_status = extracted
  warning = DEPENDENCY_RAW_RELATIVE_PATH_MISSING

raw_path missing and raw_relative_path present:
  path_basis = raw_relative_path
  extraction_status = extracted
  warning = DEPENDENCY_RAW_PATH_MISSING

raw_path missing and raw_relative_path missing:
  path_basis = path_unavailable
  extraction_status = incomplete
  warning = DEPENDENCY_PATH_UNAVAILABLE
```

## 8. Determinism

Required behavior:

```text
same ALSReadModel
  -> same DependencyExtractionResult
  -> same dependency order
  -> same dependency_id values
  -> same warnings/errors except run-independent metadata
```

Rules:

```text
Do not use random IDs.
Do not use timestamps in dependency IDs.
Do not sort by filename or path.
Do not deduplicate dependencies in v0.1.
```

## 9. Does Not Do

`DependencyExtractor v0.1` must not:

```text
read sample files from disk
check whether raw_path exists
resolve paths to canonical filesystem locations
classify source as Downloads/User Library/Core Library/project-local
deduplicate repeated sample refs
match missing files
score candidates
decide copy actions
decide rewrite actions
rewrite ALS
copy files
delete files
promote OriginalCrc into identity proof
turn historical_refs into active dependencies
turn non_audio_dependency_signals into dependencies
```

## 10. Boundaries

Boundary with `ALSReader`:

```text
ALSReader reads ALS and preserves raw facts.
DependencyExtractor consumes the core handoff subset and normalizes active audio
refs into dependency records.
```

Boundary with `PathObservation`:

```text
PathObservation owns read-only candidate availability observations.
It does not resolve asset identity or select a path.
DependencyExtractor only reports what ALS provided.
```

Boundary with `DependencyAssessment`:

```text
DependencyAssessment groups ReferenceOccurrence records into RequiredAsset
requirements and owns project-level availability/risk assessment.
DependencyExtractor may not classify dependency risk.
```

Boundary with `AssetResolution`:

```text
AssetResolution owns candidate evidence and explicit resolution decisions.
DependencyExtractor does not search for alternatives.
```

Boundary with `ALSRewriter`:

```text
ALSRewriter owns rewrite operations.
DependencyExtractor does not mark refs as safe to rewrite.
```

## 11. Test Strategy

Minimum tests:

```text
valid_model_extracts_audio_dependencies
zero_active_refs_is_valid
one_active_ref_becomes_one_dependency_ref
duplicate_refs_are_not_deduplicated
incomplete_ref_is_preserved_with_warning
historical_refs_are_ignored_with_summary
non_audio_signals_are_ignored_with_summary
unsupported_als_read_model_version_returns_error
output_is_deterministic
```

Fixture tests:

```text
Use ALSReader fixture outputs where practical.
Use synthetic ALSReadModel for impossible or rare malformed cases.
```

Safety tests:

```text
No filesystem reads beyond already-created ALSReadModel input.
No file writes.
No copy/delete/rewrite operations.
```

Contract tests:

```text
DependencyExtractionResult v0.1 shape is stable.
DependencyRef v0.1 fields are present.
PathObservation-needed raw fields are present without filesystem claims.
Diagnostic/future ALSReader fields are not required unless spec changes.
```

## 12. Acceptance Criteria

`DependencyExtractor v0.1` is accepted when:

```text
1. It accepts ALSReadModel v0.2.
2. It rejects unsupported ALSReadModel versions with structured error.
3. It returns DependencyExtractionResult v0.1.
4. It extracts only active audio references.
5. It returns one DependencyRef per active audio reference.
6. It does not deduplicate duplicate refs.
7. It preserves incomplete refs with warnings.
8. It marks path_basis and extraction_status correctly.
9. It records ignored historical_refs and non_audio_dependency_signals.
10. It treats OriginalCrc as weak evidence only.
11. It does not touch the filesystem.
12. It does not classify, match, copy, rewrite or delete.
13. It is deterministic.
14. It has fixture/contract tests for accepted behavior.
15. It has safety tests proving no filesystem mutation.
```

## 13. Known Unknowns

Known but not blocking v0.1:

```text
Exact long-term shape of non-audio dependency contracts.
Whether future DependencyRef versions include plugin/preset dependencies.
Whether xml_locator becomes required by ALSRewriter contract later.
Whether usage_context can be improved beyond unknown from confirmed ALS evidence.
How DependencyAssessment will group occurrences into RequiredAsset records.
How later AssetResolution will represent evidence and user decisions.
```

Rule:

```text
Known unknowns must not be solved inside DependencyExtractor v0.1 unless this
spec is intentionally revised.
```

## 14. Next Step After Spec Approval

Implementation order:

```text
1. Create fixture-contract.md.
2. Create tasks.md.
3. Add data models to rescue_core.
4. Add pure function extract_dependencies(model).
5. Add unit/fixture/contract tests.
6. Run cargo fmt --check.
7. Run cargo test --workspace.
8. Update CURRENT_STATE.md.
```
