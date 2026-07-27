# Module Spec 003: PathObservation

Status: ready for implementation, v0.2
Date: 2026-07-27
Implementation target: Rust core  
Product source: `PRODUCT_SPINE.md`  
Upstream contract: `DependencyExtractionResult v0.1`

The directory keeps the historical id `003-path-verifier` so links and guard
history do not break. The public responsibility is now PathObservation, not
asset verification.

## 1. Responsibility

PathObservation records what the local filesystem says about every safe path
candidate derived from an audio `DependencyRef v0.1`.

Primary responsibility:

```text
DependencyExtractionResult v0.1
+ explicit PathObservationContext
-> PathObservationResult v0.2
```

The module answers:

```text
Which candidate paths can be derived without guessing?
Can each candidate be checked on this platform?
Does an entry exist?
Is it a regular file, directory or symlink?
What size was observed?
Which checks were blocked, unsafe, unsupported or inconclusive?
```

It does not answer:

```text
Which candidate is the correct sample?
Which candidate should be selected?
Do two paths represent the same audio content?
Is the whole Ableton project complete or portable?
```

## 2. Product Traceability

Parent capability:

```text
Inspect Project Dependencies
```

Supported use case:

```text
For one selected Live Set, show whether its recognized recorded audio path
candidates are available, unavailable, unknown or unsupported without changing
the project.
```

Flow position:

```text
ALSReader
-> DependencyExtractor
-> PathObservation
-> DependencyAssessment
-> read-only project report
```

MVP classification:

```text
MVP, narrowed read-only scope
```

## 3. Domain Meaning

`DependencyRef v0.1` represents a `ReferenceOccurrence` compatibility record.
One input record produces one `DependencyPathObservation`. Duplicate references
remain separate.

A `CandidatePathObservation` describes local filesystem evidence. It is not a
`FileOccurrence` with confirmed content identity and is never a
`ResolutionDecision`.

Required invariant:

```text
path availability != expected asset identity
```

## 4. Inputs

### 4.1 DependencyExtractionResult v0.1

Requirements:

```text
dependency_ref_version is 0.1
fatal extractor errors are absent
dependencies are processed one-to-one and in order
raw_path and raw_relative_path remain unchanged
```

### 4.2 PathObservationContext v0.1

Context is supplied by the application workflow. ALSReader must not construct
it from `parent(ALS)`.

Fields:

```text
host_platform
confirmed_project_root: optional native path
project_root_basis: optional, required when confirmed_project_root exists
```

`host_platform` is `macos` or `windows` in the product. `posix` is accepted
only so the same core safety contract can run in non-product Unix CI.

Allowed `project_root_basis` values for the first implementation:

```text
user_selected
controlled_fixture
confirmed_ableton_project_structure
```

Candidate project roots inferred only from a filename or from `parent(ALS)` are
not accepted as confirmed context.

### 4.3 Filesystem Boundary

The module needs a read-only filesystem metadata port supporting:

```text
lexical path construction
symlink_metadata without following the link
regular-file / directory / symlink classification
file size for regular files
structured inaccessible / not-found / I/O outcomes
```

Native paths are not normalized back into raw ALS strings.

## 5. Outputs

### 5.1 PathObservationResult v0.2

Fields:

```text
observation_metadata: PathObservationMetadata
dependency_observations: list<DependencyPathObservation>
warnings: list<PathObservationWarning>
errors: list<PathObservationError>
```

### 5.2 PathObservationMetadata v0.2

Fields:

```text
observer_version
path_observation_model_version
input_dependency_ref_version
source_als_path
source_file_hash
project_root_basis
dependency_count
candidate_count
regular_file_count
missing_count
unknown_count
warning_count
error_count
```

The metadata does not expose an inferred `source_project_root`.

### 5.3 DependencyPathObservation v0.2

Fields:

```text
dependency_id
als_ref_id
raw_path
raw_relative_path
parsed_raw_path
parsed_raw_relative_path
candidates: list<CandidatePathObservation>
availability_summary
identity_status
warnings
```

Required values:

```text
identity_status: not_evaluated
```

`availability_summary` is derived from observations and may be:

```text
regular_file_observed
no_regular_file_observed
unknown
unsupported_on_current_platform
```

It may not be named `verified`, `matched` or `resolved`.

### 5.4 CandidatePathObservation v0.2

Fields:

```text
candidate_id
candidate_basis
candidate_path
platform_status
safety_status
availability_status
entry_kind
size_evidence_status
observed_file_size
expected_file_size
evidence_notes
warnings
```

Candidate basis:

```text
confirmed_project_root_plus_raw_path
confirmed_project_root_plus_raw_relative_path
recorded_raw_absolute_path
```

Platform status:

```text
checkable_on_current_platform
foreign_platform_path
unsupported_path_shape
```

Safety status:

```text
safe_for_metadata_read
rejected_parent_escape
rejected_invalid_path
not_checked
```

Availability status:

```text
existing_regular_file
existing_directory
existing_symlink
missing
inaccessible
not_checked
```

Entry kind:

```text
regular_file
directory
symlink
other
unknown
```

Size evidence:

```text
matches_expected_size
differs_from_expected_size
expected_size_unavailable
observed_size_unavailable
not_applicable
```

Size agreement remains supporting evidence and never changes
`identity_status: not_evaluated`.

## 6. Candidate Generation Rules

The module may produce at most:

```text
one project-relative candidate according to the admitted RelativePathType rule
one direct candidate from raw_path when it is an absolute path checkable on the host
```

Rules:

1. No project-relative candidate exists without `confirmed_project_root`.
2. Type `0` may use a safe relative `raw_path` as
   `confirmed_project_root_plus_raw_path`.
3. Type `3` may use a safe `raw_relative_path` as
   `confirmed_project_root_plus_raw_relative_path`.
4. Types `1` and `5` do not produce a project-relative candidate in v0.2.
5. Unknown or missing types do not produce a project-relative candidate.
6. An absolute native `raw_path` may produce `recorded_raw_absolute_path` for
   any type.
7. Absolute, prefixed or parent-escaping relative values are rejected before a
   project-relative join.
8. A direct candidate is not created from a bare filename.
9. A macOS path is not checked using Windows semantics and vice versa.
10. Both admitted candidates are retained if both can be checked.
11. Candidate order is deterministic but has no preference or selection meaning.
12. Missing, inaccessible and unsupported are distinct outcomes.
13. `symlink_metadata` is used for the first version; a symlink target is not
    followed.
14. Raw ALS values are never rewritten or normalized.

Evidence:

```text
docs/experiments/E-01-E-02-path-and-coverage-2026-07-27.md
```

## 7. Explicit Non-Responsibilities

PathObservation must not:

```text
select a candidate path
return selected_path_candidate
return verified_exact_path
deduplicate reference occurrences
hash or decode audio
interpret OriginalCrc as identity
search directories or disks
classify source libraries
match missing samples
plan or copy files
rewrite ALS
write a manifest
```

## 8. Errors And Uncertainty

Fatal result errors:

```text
PATH_OBSERVATION_UNSUPPORTED_INPUT_MODEL
PATH_OBSERVATION_UNTRUSTED_INPUT
PATH_OBSERVATION_INVALID_CONTEXT
```

Per-candidate failures are observations or warnings, not fatal errors:

```text
PATH_CANDIDATE_NOT_FOUND
PATH_CANDIDATE_INACCESSIBLE
PATH_CANDIDATE_FOREIGN_PLATFORM
PATH_CANDIDATE_PARENT_ESCAPE
PATH_CANDIDATE_IS_SYMLINK
PATH_CANDIDATE_SIZE_DIFFERS
```

## 9. Safety Rules

```text
read metadata only
never follow symlinks in v0.2
never scan a directory
never open or hash an audio file
never create, delete, rename, copy or write a file
never mutate the input model
never infer Project root
never select asset identity
```

## 10. Downstream Consumers

Direct downstream:

```text
DependencyAssessment
read-only project report
```

Later consumers may use the observations as evidence, but they must not treat
`existing_regular_file` as a resolution decision.

## 11. Gate Status Summary

Clear:

```text
read-only metadata scope
no candidate selection
one input occurrence produces one dependency observation
raw path preservation
size and existence are not identity
```

Resolved for v0.2:

```text
E-01 confirms type 0 raw Path and type 3 raw RelativePath as the only admitted
Project-relative joins.
Synthetic fixtures preserve Unicode and case exactly, reject parent escape,
and treat foreign-platform path text as not checkable on the current host.
```

Non-blocking future work:

```text
following symlinks for later copy/package operations
cloud placeholder hydration
file identity and content hashing
project-root discovery automation
```

## 12. Acceptance Criteria

The implementation is accepted when:

```text
module-ready guard passes after blocking markers are removed by evidence
one DependencyRef produces one DependencyPathObservation
all safe candidates remain visible
no selected or verified identity field exists
parent escape is rejected
type 0 uses relative raw Path under the confirmed Project root
type 3 uses raw RelativePath under the confirmed Project root
type 1 and type 5 relative fields are not joined to the Project root
symlinks are reported and not followed
foreign-platform paths are preserved but not checked
raw ALS paths are preserved
all filesystem operations are metadata-only
workflow_guard verify-module passes
cargo fmt/check/test/clippy pass
```
