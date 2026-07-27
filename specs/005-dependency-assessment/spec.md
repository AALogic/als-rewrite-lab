# Module Spec 005: DependencyAssessment

Status: ready for implementation, v0.1
Date: 2026-07-27
Implementation target: `rescue_analyzer`

## Responsibility

DependencyAssessment converts one-to-one reference occurrences and their
read-only path observations into conservative logical audio requirements.

```text
DependencyExtractionResult v0.1 + PathObservationResult v0.2
-> DependencyAssessmentResult v0.1
```

It groups only occurrences carrying the same complete recorded reference
claim. It assesses whether local file candidates were observed, but it never
claims that a candidate is the correct sample.

## Product Traceability

```text
ALSReader -> DependencyExtractor -> PathObservation
-> DependencyAssessment -> PreflightReport
```

This module supplies product capability PC-004: grouped audio requirements and
an explainable availability assessment.

## Input Contract

The inputs must describe the same ALS snapshot and contain exactly one path
observation for every dependency occurrence.

Required producer fields:

```text
DependencyExtractionMetadata.source_file_hash
DependencyExtractionMetadata.dependency_ref_version
DependencyRef.dependency_id and recorded reference claim fields
PathObservationMetadata.source_file_hash
PathObservationMetadata.path_observation_model_version
DependencyPathObservation.dependency_id and candidates
```

Mismatched snapshots, duplicate occurrence IDs, missing observations, extra
observations, upstream fatal errors, or unsupported versions produce a fatal
assessment error and no trusted requirements.

## Grouping Rule

An occurrence can share a `RequiredAsset` only when all of these recorded
values are exactly equal:

```text
dependency_kind
source_kind
raw_path
raw_relative_path
relative_path_type
file_type
filename
extension
original_file_size
original_crc
```

At least one non-empty recorded path and a filename are required for grouping.
An incomplete occurrence remains its own requirement. Case, Unicode and raw
path text are preserved. The rule groups equal claims, not proven content.

`OriginalCrc`, filename, size and path are never independent identity proofs.

## Output Contract

`DependencyAssessmentResult v0.1` contains:

```text
assessment_metadata
required_assets
warnings
errors
```

Each `RequiredAsset` contains a deterministic ID, grouping basis, occurrence
IDs, copied recorded hints, all path observations relevant to the group,
availability status, unresolved resolution status, risk flags and evidence
status.

Availability statuses:

```text
regular_file_candidate_observed
no_regular_file_candidate_observed
unknown
```

Resolution status is always `unresolved` in v0.1.

## Safety And Boundaries

```text
pure transformation only
no filesystem access
no hashing or audio decoding
no search
no path selection
no asset identity claim
no copy, rewrite, delete or manifest write
```

## Errors

```text
ASSESSMENT_UNSUPPORTED_EXTRACTION_MODEL
ASSESSMENT_UNSUPPORTED_OBSERVATION_MODEL
ASSESSMENT_UNTRUSTED_EXTRACTION
ASSESSMENT_UNTRUSTED_OBSERVATIONS
ASSESSMENT_SNAPSHOT_MISMATCH
ASSESSMENT_OCCURRENCE_CONTRACT_MISMATCH
```

## Acceptance

```text
exact complete claims group deterministically
same filename or CRC alone never groups records
incomplete occurrences remain separate
candidate existence remains observation, not resolution
all occurrence/candidate evidence remains traceable
invalid handoffs fail closed with structured errors
output is deterministic and filesystem-independent
tests, clippy and module guard pass
```

## Known Unknowns

Content-based grouping, cross-snapshot identity, symlink policy, library source
classification and user-approved resolution belong to later modules.
