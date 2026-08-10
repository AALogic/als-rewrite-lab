# Tasks 024: BatchCopyApplicationService

Status: completed
Date: 2026-08-03

## Readiness

- [x] DOCUMENTED_ONLY: batch reuses the unchanged one-project copy policy
- [x] DOCUMENTED_ONLY: all previews finish before any write
- [x] DOCUMENTED_ONLY: collisions block instead of silently renaming
- [x] REVIEW_ONLY: React owns no package, rewrite or continuation policy
- [x] REVIEW_ONLY: cancellation occurs only between one-project transactions

## Contract

- [x] ENFORCED_BY_TEST: all_previews_finish_before_first_write
- [x] ENFORCED_BY_TEST: unchanged_one_project_contract_is_reused
- [x] ENFORCED_BY_TEST: target_collision_blocks_all_colliding_jobs
- [x] ENFORCED_BY_TEST: blocked_preview_does_not_block_ready_job
- [x] ENFORCED_BY_TEST: execution_is_strictly_sequential
- [x] ENFORCED_BY_TEST: failed_job_does_not_stop_later_job
- [x] ENFORCED_BY_TEST: cancellation_stops_before_next_job
- [x] ENFORCED_BY_TEST: no_batch_write_without_consent
- [x] ENFORCED_BY_TEST: tampered_batch_summary_is_rejected_without_write
- [x] ENFORCED_BY_TEST: ready_target_outside_destination_is_rejected_without_write
- [x] ENFORCED_BY_TEST: batch_result_preserves_selection_order
- [x] ENFORCED_BY_TEST: batch_diagnostic_is_path_free
- [x] ENFORCED_BY_TEST: batch_wire_contract_is_stable
- [x] ENFORCED_BY_TYPE: BatchPrepareCopyRequest
- [x] ENFORCED_BY_TYPE: BatchCopyPreview
- [x] ENFORCED_BY_TYPE: BatchPreviewJob
- [x] ENFORCED_BY_TYPE: BatchExecuteCopyRequest
- [x] ENFORCED_BY_TYPE: BatchCopyResult
- [x] ENFORCED_BY_TYPE: BatchCopyJobResult
- [x] ENFORCED_BY_TYPE: BatchCopySummary
- [x] ENFORCED_BY_TYPE: BatchCopyDiagnosticReport
- [x] ENFORCED_BY_TYPE: BatchProgressEvent

## Build

- [x] Add public batch contracts.
- [x] Add target assignment and collision detection.
- [x] Add preview orchestration.
- [x] Add sequential controlled execution.
- [x] Add Rust behavior and contract tests.
- [x] Add Tauri progress/cancellation adapter.
- [x] Add TypeScript contracts and shared fixtures.
- [x] Add batch preview/result UI.
- [x] Run backend, frontend and workflow verification.
