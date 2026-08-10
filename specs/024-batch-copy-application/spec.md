# Module Spec 024: BatchCopyApplicationService

Status: implemented and verified, v0.1
Date: 2026-08-03
Parent capability: PC-024 / multi-project portable copy
Upstream contracts: module 023 `ProjectSelection`, existing Desktop one-project copy service
Durable decision: `docs/architecture/adr/ADR-011-project-catalog-identity-and-boundaries.md`

## 1. Responsibility

BatchCopyApplicationService prepares and executes the unchanged one-project
copy flow for an ordered list of explicit Project selections.

```text
BatchPrepareCopyRequest v0.1
-> prepare_batch_copy
-> BatchCopyPreview v0.1

BatchExecuteCopyRequest v0.1
-> execute_batch_copy
-> BatchCopyResult v0.1
```

It owns batch ordering, target collision detection, per-job isolation,
cancellation between jobs, aggregate status and a path-redacted diagnostic
report. It does not own ALS reading, dependency policy, package planning,
rewrite, validation, manifests or promotion.

## 2. Flow Position

```text
ordered ProjectSelection values
-> selected destination parent
-> prepare every one-project preview without writes
-> detect target collisions
-> show aggregate and per-project plan
-> explicit write consent
-> execute ready previews sequentially
-> keep blocked/failed jobs isolated
-> aggregate result and diagnostic report
```

All previews must finish before the first write. Previewing never invokes the
one-project execute operation.

## 3. Prepare Contract

`BatchPrepareCopyRequest`:

```text
request_id
selections
destination_parent
experimental_compatibility_consent
```

The selection list is ordered and non-empty. Every selection is processed as
one separate job. `destination_parent` must be absolute.

`BatchCopyPreview`:

```text
service_version
request_id
preview_status
destination_parent
jobs
summary
diagnostic_report
errors
```

Preview statuses:

```text
ready
partially_ready
blocked
```

`BatchPreviewJob`:

```text
job_id
selection_id
source_als_path
target_project_root
job_status
preview
errors
```

Job preview statuses:

```text
ready
blocked
target_collision
```

The target path is proposed only through the existing
`default_target_project_root` function. Batch code must not invent a second
naming policy.

If two jobs resolve to the same target path, all jobs in that collision are
`target_collision`. They remain visible but cannot execute. On Windows the
comparison is case-insensitive. No suffix is silently invented in v0.1.

## 4. Execute Contract

`BatchExecuteCopyRequest`:

```text
request_id
preview
write_consent
```

`BatchCopyResult`:

```text
service_version
request_id
run_status
destination_parent
jobs
summary
diagnostic_report
errors
```

Run statuses:

```text
completed
completed_with_issues
cancelled
blocked
```

`BatchCopyJobResult`:

```text
job_id
selection_id
source_als_path
target_project_root
job_status
result
errors
```

Job result statuses:

```text
completed
completed_incomplete
failed
skipped_blocked
cancelled
```

Only a `ready` job with a successful source-bound `DesktopCopyPreview` may be
passed to the existing `execute_copy`. Batch execution does not rebuild or
weaken a preview.

Before the first write, the service recomputes the aggregate preview status and
summary from the job list. It rejects unsupported contract versions, unknown
job statuses, duplicate identities, duplicate targets and any ready target that
is not a direct child of the selected destination parent. Values returned from
the UI are evidence to validate, not authority to trust.

## 5. Summary And Diagnostics

`BatchCopySummary`:

```text
total_job_count
ready_job_count
blocked_job_count
completed_job_count
incomplete_job_count
failed_job_count
cancelled_job_count
```

`BatchCopyDiagnosticReport`:

```text
diagnostic_schema_version
request_id
service_version
host_os
host_arch
run_status
elapsed_ms
summary
error_codes
```

The diagnostic report contains no native paths, project names, hashes or
audio metadata. Per-project manifests remain owned by the existing pipeline.

## 6. Isolation And Continuation Policy

```text
one selection -> one job ID -> one preview -> one staging transaction -> one result
```

A blocked preview does not block ready previews. A failed execution does not
stop later ready jobs. Completed projects are never rolled back because a
different job failed.

Batch result order always equals selection order, including blocked and
cancelled entries.

## 7. Cancellation

`BatchCopyObserver` receives `BatchProgressEvent` and may request cancellation.

```text
request_id
stage
job_id
job_index
total_job_count
completed_job_count
failed_job_count
```

Cancellation is checked before each project execution. It never interrupts the
currently running one-project transaction. The current job finishes through
its existing validation/promotion rules; remaining ready jobs become
`cancelled` without writes.

## 8. Desktop Adapter And UI

Tauri runs batch preview and execution in blocking workers. It owns an
in-memory cancellation flag keyed by request ID and emits path-free progress
events. React sends explicit selections, one destination parent and consent.

The UI shows:

```text
selected Project count
one destination-folder chooser
aggregate preview summary
one status row per Project
explicit Create copies action
Cancel remaining action during execution
aggregate final result
copyable path-redacted diagnostic report
```

The existing single-project screen and service remain unchanged.

## 9. Errors

Codes include:

```text
BATCH_REQUEST_EMPTY
BATCH_SELECTION_EMPTY
BATCH_DESTINATION_NOT_ABSOLUTE
BATCH_DUPLICATE_SELECTION
BATCH_TARGET_SUGGESTION_FAILED
BATCH_TARGET_COLLISION
BATCH_PREVIEW_BLOCKED
BATCH_WRITE_CONSENT_REQUIRED
BATCH_PREVIEW_INVALID
BATCH_EXECUTION_FAILED
```

No aggregate error message includes paths or project filenames.

## 10. Safety And Architecture

```text
no second package/rewrite/validation policy
no write during batch preview
all previews before first write
one-project execute called sequentially
no silent target rename or overwrite
no target shared by two jobs
no execute without explicit consent
recompute and validate batch summary before execution
ready targets remain direct children of the selected destination
no interruption inside a one-project transaction
one failure does not stop later independent jobs
path-free aggregate diagnostics
```

## 11. Acceptance Criteria

```text
all jobs are previewed before any write
one-project prepare/execute contracts are reused unchanged
target collisions are blocked before write
existing or invalid target blocks only its job
partial preview can execute ready jobs after consent
writes are strictly sequential
one failed job does not stop a later job
cancellation stops before the next job
no write occurs without consent
tampered batch summary blocks before write
ready target outside the selected destination blocks before write
result order equals selection order
aggregate diagnostics contain no private paths
Tauri and TypeScript wire contracts match Rust
Desktop supports one or many resolved selections
tests, clippy, frontend checks and workflow_guard pass
```

## 12. Known Limits

```text
no parallel analysis or copy
no persisted/resumable batch queue
no automatic collision renaming
no per-project custom destination name in v0.1
no rollback across already completed projects
no whole-computer audio index or missing-file matching
```

## 13. Gate Status Summary

```text
Vision Gate: CLEAR
Use Case Gate: CLEAR
Flow Gate: CLEAR
Data Gate: CLEAR
Behavior Gate: CLEAR
Decision Gate: CLEAR with block-on-collision policy
Safety Gate: CLEAR because one-project transactions remain authoritative
Test Gate: CLEAR with injected fake one-project operations and real contract fixtures
Evidence Gate: CLEAR for orchestration; Ableton manual verification remains per project
Module Gate: CLEAR
```
