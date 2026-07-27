# Module Spec 008: AssetResolution

Status: ready for implementation, v0.1
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
its native path exactly equals an accepted existing path observation
its filename equals the recorded filename
its filename is ASCII-case-insensitively equal to the recorded filename
```

All candidates remain visible and deterministically ordered.

## Scoring Policy v0.1

```text
exact accepted observed native path: +50
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
```

Otherwise the decision is `needs_user_confirmation` or `unresolved`.

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
same name, size or CRC alone never proves identity
ambiguous high-scoring candidates block automation
```

## Acceptance

```text
one exact path/name/size candidate may auto-accept at >=95
name/size-only candidates require confirmation
same-name different-content candidates remain distinct
high-score ties remain ambiguous
no candidate remains unresolved
partial scan blocks auto acceptance
all scores have evidence
output is deterministic
tests, clippy and module guard pass
```
