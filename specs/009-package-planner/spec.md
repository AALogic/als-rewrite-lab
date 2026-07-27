# Module Spec 009: PackagePlanner

Status: ready for implementation, v0.1
Date: 2026-07-27
Implementation target: new `rescue_packaging` crate

## Responsibility

PackagePlanner is a pure function that converts one trusted ALS snapshot,
assessment, inventory snapshot, resolution decisions and an explicit target
into an immutable, inspectable package plan.

```text
PackagePlanningRequest
+ ALSReadModel v0.2
+ DependencyAssessmentResult v0.1
+ AssetInventoryResult v0.1
+ AssetResolutionResult v0.1
-> PackagePlan v0.1
```

It plans but never creates directories, copies files or rewrites ALS.

## Modes

```text
copy_only
laboratory_rescue_rewrite
```

`copy_only` plans the copied ALS and selected audio but no relink.

`laboratory_rescue_rewrite` is allowed only by E-03's experimental profile:

```text
rule: live11_3_external_to_imported_v0.1-experimental
MajorVersion: 5
MinorVersion: 11.0_11300
Creator prefix: Ableton Live 11.3.
active direct SampleRef/FileRef
old RelativePathType: 1
new destination: Samples/Imported/<safe filename>
changed fields: Path, RelativePath, RelativePathType
new RelativePathType: 3
original metadata and historical refs unchanged
```

The plan remains laboratory-only until semantic validation and a manual Live
open promote the E-03 rule.

## Input Validation

All producer results must be trusted and mutually consistent. The ALS source
hash must equal the assessment snapshot hash. Resolution and inventory scan IDs
must agree. The target Project root must be absolute, and the target ALS path
must not equal the original ALS path.

Only `auto_accepted` decisions can become executable copy/rewrite operations.
Every selected occurrence and active reference must exist exactly once.

## Copy Planning

The first operation copies the source ALS to the target Project root. Audio is
planned under `Samples/Imported`.

Copy operations are deduplicated by target relative path and content ID. Two
different content IDs targeting the same relative path are a blocking
`PACKAGE_TARGET_COLLISION`; no automatic rename policy is invented.

All operations use `fail_if_exists` and record expected SHA-256, byte size and
preconditions.

## Rewrite Planning

Every active occurrence gets its own snapshot-bound `RewriteOperation`.
Operations include locator, source ALS hash, expected old values, exact new
values, changed-field allowlist and ruleset.

Locator and reference values come from the explicit ADR-005 ALSReader rewrite
handoff, not DependencyRef.

## Status

```text
ready_copy_only
ready_for_laboratory_execution
blocked
```

Any unresolved requirement, unsupported reference, target collision or invalid
handoff blocks execution.

## Safety

```text
pure and deterministic
no filesystem access
no overwrite/merge policy
no silent collision rename
no invented user confirmation
no production rewrite claim
source and target paths remain explicit
```

## Acceptance

```text
target equal to source is rejected
unresolved decisions block the plan
one selected content produces one copy operation
repeated occurrences produce separate rewrite operations
same-name different-content collision blocks
unsupported Live/version/type/locator blocks rewrite
copy-only mode contains no rewrite operations
same input produces the same plan
tests, clippy and module guard pass
```
