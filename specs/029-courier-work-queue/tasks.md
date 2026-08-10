# Tasks 029: CourierWorkQueue

Status: accepted
Date: 2026-08-05

## Readiness

- [x] DOCUMENTED_ONLY: queue accepts resolved selections and owns no copy policy
- [x] REVIEW_ONLY: removal never deletes or moves a source file
- [x] REVIEW_ONLY: native paths do not enter exported queue errors

## Required Tests

- [x] ENFORCED_BY_TEST: courier_queue_preserves_acceptance_order
- [x] ENFORCED_BY_TEST: courier_queue_rejects_active_duplicate_source
- [x] ENFORCED_BY_TEST: removed_item_can_be_added_again
- [x] ENFORCED_BY_TEST: processing_item_cannot_be_removed
- [x] ENFORCED_BY_TEST: non_als_directory_and_symlink_are_rejected
- [x] ENFORCED_BY_TEST: queue_errors_are_path_free
- [x] ENFORCED_BY_TEST: queue_never_mutates_source_files
- [x] ENFORCED_BY_TYPE: CourierWorkQueue
- [x] ENFORCED_BY_TYPE: CourierWorkItem
- [x] ENFORCED_BY_TYPE: CourierQueueSnapshot
- [x] ENFORCED_BY_TYPE: CourierQueueError

## Build

- [x] Add `courier_work_queue.rs`.
- [x] Export the module-030-facing API.
- [x] Add focused tests.
- [x] Run `module-ready` before code and `verify-module` after code.
