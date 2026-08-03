# Module Spec 008: AssetResolution

Status: ready for implementation, model v0.1, policy v0.3
Date: 2026-08-02
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

## Metadata-Only Current-Path Binding

The desktop MVP uses a separate `CurrentPathBindingResult v0.1`. It consumes
only `DependencyAssessmentResult v0.1` and binds a requirement when exactly one
safe regular non-symlink candidate exists at a recorded path with an observed
non-conflicting size.

`CurrentPathBinding` records the required asset, candidate, current source
path, filename and observed size. It contains no SHA-256 and no content ID.
This is a current location binding, not `ContentIdentity` and not a historical
match claim. Missing paths become non-blocking omissions. Size conflicts,
multiple distinct current paths and unsafe evidence block execution.
Equivalent spellings of one local Windows drive path, including slash style
and the extended-length `\\?\` prefix, form one binding. This is lexical path
normalization only; it does not merge different paths or claim content identity.

The content-addressed `AssetResolutionResult` path below remains available for
laboratory matching, explicit selections and the future persistent index.

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

## Scoring And Decision Policy v0.3

```text
exact safe-for-metadata-read existing native path: +50
exact case-sensitive filename:       +20
ASCII case-insensitive filename:     +10
exact extension:                     +5
exact expected byte size:            +25
```

Size mismatch is a conflict when ALS provides a positive expected byte size.
`OriginalFileSize = 0` is preserved as raw evidence but means that expected
size is unavailable; it creates neither match evidence nor a mismatch conflict.
`OriginalCrc` is preserved as upstream evidence but is not scored because no
candidate-side equivalent or confirmed Ableton algorithm exists.

An exact existing recorded-path candidate may become executable with:

```text
complete inventory snapshot
exactly one candidate at the recorded active path
no candidate conflicts
decision basis = current_recorded_path_binding
```

This preserves the file that the current project would read at its recorded
path. It is an availability/binding decision, not a claim that the file is the
historical original sample.

A candidate away from the recorded path may become executable only through a
valid `UserSelectionSet v0.1`. The set is bound to the source ALS SHA-256 and
contains, per selected asset, its required-asset ID, native candidate path and
expected full SHA-256. Resolution accepts the choice only when the current
inventory contains exactly that candidate with exactly that content hash and
no conflicts. A stale, changed, unknown or duplicate selection blocks.

Without a valid explicit selection, filename, extension, size, score and a
newly observed candidate hash only rank recovery candidates. They do not choose
one. `OriginalCrc` remains weak evidence and is not scored.

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

`auto_accepted` is executable only when `decision_basis` is either
`current_recorded_path_binding`, `explicit_user_selection`, or a future basis
explicitly added to the versioned policy. The first basis does not assert
historical content identity.

## Validation And Errors

```text
RESOLUTION_UNSUPPORTED_ASSESSMENT_MODEL
RESOLUTION_UNSUPPORTED_INVENTORY_MODEL
RESOLUTION_UNTRUSTED_ASSESSMENT
RESOLUTION_UNTRUSTED_INVENTORY
RESOLUTION_SELECTION_SCHEMA_UNSUPPORTED
RESOLUTION_SELECTION_SOURCE_MISMATCH
RESOLUTION_SELECTION_DUPLICATE_ASSET
RESOLUTION_SELECTION_UNKNOWN_ASSET
```

Partial inventory is trusted as positive evidence but blocks automatic
acceptance because candidate search was incomplete.

## Safety And Boundaries

```text
no filesystem access
no hidden recovery-candidate selection
no user decision fabrication
no copy/rewrite/delete
recorded-path binding is kept distinct from historical content identity
path, name, size, newly observed hash or CRC alone never select a moved candidate
ambiguous high-scoring candidates block automation
```

## Acceptance

```text
one exact current-path candidate is accepted as the current project binding
zero OriginalFileSize means unavailable evidence, not a zero-byte expectation
name/size-only candidates require confirmation
valid explicit path-and-hash selection accepts one moved candidate
changed or stale explicit selection blocks
same-name different-content candidates remain distinct
high-score ties remain ambiguous
no candidate remains unresolved
partial scan blocks auto acceptance
all scores have evidence
output is deterministic
tests, clippy and module guard pass
metadata-only current-path binding never invents a content hash or content ID
```
