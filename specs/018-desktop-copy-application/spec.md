# Module Specification: 018 DesktopCopyApplicationService

Status: implementation update approved, output v0.3
Date: 2026-08-03

## Purpose

DesktopCopyApplicationService is the write-capable application boundary used by
the Tauri frontend. It prepares a read-only copy preview and executes only an
explicitly confirmed current-path copy request.

## Contracts

`DesktopPrepareCopyRequest v0.1` contains request ID, source ALS path and an
absolute absent target Project root.

`DesktopCopyPreview v0.3` contains the source snapshot hash, target root,
available copy count, system dependency count, rewrite count, omitted count,
expected complete/incomplete status, `PlanFingerprint v0.1` and structured
blockers. Preparing a preview writes nothing.

`DesktopExecuteCopyRequest v0.2` contains the accepted preview plus explicit
write consent. Execution verifies the source and exact reviewed plan; stale or
changed previews fail closed before staging.

`DesktopCopyResult v0.2` contains complete/incomplete/failure status, final
target path, copied/system-dependency/rewrite/omitted counts and structured
errors. A confirmed system dependency is visible in both outputs but is not a
copy or rewrite operation.

## Safety

- UI never calls planner, staging or rewriter directly.
- Preview is read-only.
- Execution requires explicit consent and an absent target.
- Staging and private ledger paths are derived beside the chosen target with
  run-specific names and must be absent.
- Source snapshot drift blocks execution.
- Package-plan drift blocks execution and requires a fresh preview.
- A platform without implemented atomic staged-ALS replacement returns
  `write_pipeline_failed`, preserves every source file and does not publish a
  final target. This is a safe refusal, not declared write support.
- UI transports the fingerprint but never computes or interprets it.
- Private ledger content is not exposed by the desktop result.

## Acceptance

- preview reports complete and incomplete outcomes without writing;
- execution refuses missing consent;
- execution refuses stale source evidence;
- execution refuses changed copy/rewrite semantics without changing the ALS;
- an unsupported atomic-replacement platform fails closed without publishing a
  target;
- successful execution returns the final target and correct outcome;
- UI needs no domain-policy implementation.
