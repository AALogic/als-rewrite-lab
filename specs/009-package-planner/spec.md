# Module Spec 009: PackagePlanner

Status: implementation update approved, PackagePlan v0.5 and PlanFingerprint v0.1
Date: 2026-08-03
Implementation target: new `rescue_packaging` crate

## Responsibility

PackagePlanner is a pure function that converts one trusted ALS snapshot,
assessment, inventory snapshot, resolution decisions and an explicit target
into an immutable, inspectable package plan.

```text
PackagePlanningRequest
+ ALSReadModel v0.2
+ DependencyAssessmentResult v0.2
+ AssetInventoryResult v0.1
+ AssetResolutionResult v0.1 using policy v0.3
-> PackagePlan v0.5
```

It plans but never creates directories, copies files or rewrites ALS.

PackagePlanner also owns the semantic identity of an immutable plan.
`PlanFingerprint v0.1` is a versioned SHA-256 digest of a canonical projection
of `PackagePlan v0.5`. It lets an application prove that a freshly rebuilt
plan has the same write semantics as the plan previously reviewed by the user.
The fingerprint is small plan metadata; it never hashes audio bytes.

The plan explicitly includes `CreateDirectoryOperation` entries for
`Ableton Project Info`, `Samples`, and `Samples/Imported`. The project
marker is a required part of the Ableton project structure, not an implicit
side effect of execution.

## Modes

```text
copy_only
laboratory_rescue_rewrite
current_paths_copy
```

`copy_only` plans the copied ALS and selected audio but no relink.

Rewrite-capable modes use the bounded Live 11.3 current-path ruleset:

```text
ruleset: live11_3_current_paths_v0.2-lab
MajorVersion: 5
MinorVersion: 11.0_11300
Creator prefix: Ableton Live 11.3.
active direct SampleRef/FileRef

external rule:
  old RelativePathType: 1
  destination: Samples/Imported/<safe filename>
  changed fields: Path, RelativePath, RelativePathType
  new RelativePathType: 3

project-local rule:
  old RelativePathType: 3
  safe RelativePath starts with Samples/
  destination: preserve the same Samples/... relative path
  changed fields: Path only
  RelativePath and RelativePathType remain unchanged

original metadata and historical refs remain unchanged
```

The external Type 1 rule remains experimental. The Type 3 Path-only rule is
confirmed for laboratory Live 11.3 use and still requires semantic validation
plus a manual Live open for every generated package.

## Input Validation

All producer results must be trusted and mutually consistent. The ALS source
hash must equal the assessment snapshot hash. Resolution and inventory scan IDs
must agree, and every resolution decision must use the current safety policy.
The target Project root must be absolute, and the target ALS path must not equal
the original ALS path.

Only `auto_accepted` decisions with a supported policy basis can become
executable copy/rewrite operations. Policy v0.3 supports
`current_recorded_path_binding` and `explicit_user_selection`. The former
copies the file currently bound to the project and does not claim historical
identity; the latter is bound to an explicit candidate path and SHA-256.
Every selected occurrence and active reference must exist exactly once.
Laboratory rewrite mode requires at least one approved rewrite operation; a
zero-reference or otherwise operation-free plan is blocked before execution.

## Copy Planning

Directory operations precede file copies, use safe relative paths and
`fail_if_exists`, and carry a purpose recorded for audit.

The first operation copies the source ALS to the target Project root. External
Type 1 audio is planned under `Samples/Imported`. Safe project-local Type 3
audio preserves its existing `Samples/...` location. Every required parent
directory is an explicit deterministic directory operation.

Copy operations are deduplicated by target relative path and content ID. Two
different content IDs targeting the same relative path are a blocking
`PACKAGE_TARGET_COLLISION`; no automatic rename policy is invented.

All operations use `fail_if_exists` and record expected SHA-256, byte size and
preconditions.

Every copy operation also declares a verification policy. Hash-backed
laboratory assets and the source ALS use `sha256_and_size`. Desktop current-path
audio uses `stable_source_and_size`, carries an observed byte size and current
source binding ID, and has `expected_source_sha256 = None` plus
`content_id = None`. The planner never represents an unknown hash as an empty
string.

`plan_current_path_package` consumes `CurrentPathBindingResult v0.1` directly.
It does not require or fabricate an AssetInventory content snapshot.

## System Dependency Planning

A confirmed `ableton_core_library/system_dependency` is represented by a
`SystemDependencyRequirement` with `package_action = leave_system_managed` and
`portability_status = portable_risk`. It creates no audio copy, no rewrite and
no unresolved requirement. Full observed paths remain private plan evidence;
the portable manifest receives no absolute path.

Unclassified type 5 references keep the existing fail-closed behavior. The
planner must not infer Core Library ownership from type 5 alone.

## Rewrite Planning

Every active occurrence gets its own snapshot-bound `RewriteOperation`.
Operations include locator, source ALS hash, expected old values, exact new
values, changed-field allowlist and ruleset.

Locator and reference values come from the explicit ADR-005 ALSReader rewrite
handoff, not DependencyRef.

The handoff must explicitly mark the reference as a rewrite candidate with
`rewrite_support_status = supported` and a known usage context. `unknown`,
`requires_test`, or a false candidate flag blocks rewrite planning even when the
locator and RelativePathType match the laboratory profile.

## Status

```text
ready_copy_only
ready_for_laboratory_execution
ready_current_paths_complete
ready_current_paths_incomplete
blocked
```

Any unresolved requirement, unsupported non-system reference, target collision
or invalid handoff blocks execution. Laboratory mode also blocks when no
approved rewrite operation exists.

`current_paths_copy` never searches for a missing asset. A requirement with no
existing recorded-path binding is retained as a non-blocking omission and its
ALS reference remains unchanged. Supported existing references are copied and
rewritten. Contract inconsistencies, unsafe paths, collisions, stale source
evidence and invalid rewrite locators remain blocking.

`UnresolvedPackageRequirement` records `blocks_execution`. Only a genuinely
missing recorded-path asset may be non-blocking in `current_paths_copy`.
Existing audio with an unsupported or unsafe reference always blocks; it must
never be copied into a package without a resolving output reference.

## Safety

```text
pure and deterministic
no filesystem access
no overwrite/merge policy
no silent collision rename
no invented user confirmation
no production rewrite claim
source and target paths remain explicit
plan fingerprint excludes run IDs, timestamps and generated staging paths
plan fingerprint includes every field that changes copy/rewrite behavior
equivalent operation ordering produces the same plan fingerprint
```

## Acceptance

```text
target equal to source is rejected
unresolved decisions block the plan
one selected content produces one copy operation
repeated occurrences produce separate rewrite operations
same-name different-content collision blocks
unsupported Live/version/type/locator blocks rewrite
unknown or unapproved rewrite support evidence blocks rewrite
outdated resolution policy blocks planning
zero-reference laboratory plans block before execution
copy-only mode contains no rewrite operations
same input produces the same plan
metadata-only current-path audio has no hash or content identity
same semantic plan produces the same PlanFingerprint v0.1
changed copy or rewrite semantics change the fingerprint
tests, clippy and module guard pass
```
