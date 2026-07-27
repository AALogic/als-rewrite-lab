# Module Spec 008: AssetResolution

Status: ready for implementation, model v0.1, policy v0.2
Date: 2026-07-27
Implementation target: new `rescue_resolution` crate

## Responsibility

AssetResolution compares grouped audio requirements with one immutable asset
inventory snapshot and creates explainable candidates plus explicit decisions.

```text
DependencyAssessmentResult v0.1 + AssetInventoryResult v0.1
-> AssetResolutionResult v0.1
```

The module is a pure function. It never reads files, scans directories, copies
data or rewrites ALS.

## Candidate Generation

An inventory occurrence is a candidate only when at least one is true:

```text
its native path exactly equals an existing observation whose PathObservation
v0.2 statuses are `safety_status = safe_for_metadata_read` and
`availability_status = existing_regular_file`
its filename equals the recorded filename
its filename is ASCII-case-insensitively equal to the recorded filename
```

All candidates remain visible and deterministically ordered.

## Scoring Policy v0.1

```text
exact safe-for-metadata-read existing native path: +50
exact case-sensitive filename:       +20
ASCII case-insensitive filename:     +10
exact extension:                     +5
exact expected byte size:            +25
```

Size mismatch is a conflict. `OriginalCrc` is preserved as upstream evidence
but is not scored because no candidate-side equivalent or confirmed Ableton
algorithm exists.

Automatic acceptance requires:

```text
score >= 95
exactly one qualifying candidate
no candidate conflicts
complete inventory snapshot
full SHA-256 match against expected content identity supplied upstream
```

The current `RequiredAsset v0.1` input contains no expected content hash and no
explicit user-selection decision. Therefore path, filename, extension, size,
and a newly observed candidate hash can rank candidates but cannot auto-accept
one under policy v0.2. Such candidates remain `needs_user_confirmation` until a
separate supported identity or user-decision contract exists.

## Output

The result contains one proposal and one decision per required asset. Candidate
evidence and conflicts explain every score. A decision identifies both the file
occurrence and its content ID; a path alone is never treated as durable content
identity.

Decision statuses:

```text
auto_accepted
needs_user_confirmation
unresolved
```

## Validation And Errors

```text
RESOLUTION_UNSUPPORTED_ASSESSMENT_MODEL
RESOLUTION_UNSUPPORTED_INVENTORY_MODEL
RESOLUTION_UNTRUSTED_ASSESSMENT
RESOLUTION_UNTRUSTED_INVENTORY
```

Partial inventory is trusted as positive evidence but blocks automatic
acceptance because candidate search was incomplete.

## Safety And Boundaries

```text
no filesystem access
no hidden candidate selection
no user decision fabrication
no copy/rewrite/delete
path, name, size, newly observed hash or CRC alone never proves expected identity
ambiguous high-scoring candidates block automation
```

## Acceptance

```text
one exact path/name/size candidate requires confirmation without an expected hash
name/size-only candidates require confirmation
same-name different-content candidates remain distinct
high-score ties remain ambiguous
no candidate remains unresolved
partial scan blocks auto acceptance
all scores have evidence
output is deterministic
tests, clippy and module guard pass
```
