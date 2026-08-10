# Tasks 028: TransferPayload

Status: accepted
Date: 2026-08-05

## Readiness

- [x] DOCUMENTED_ONLY: TransferPayload owns no provider or browser behavior
- [x] DOCUMENTED_ONLY: React no longer supplies an authoritative payload path
- [x] DOCUMENTED_ONLY: terminal attempts preserve the candidate for explicit retry
- [x] REVIEW_ONLY: module 018 behavior and contracts remain unchanged
- [x] REVIEW_ONLY: payload validation never enumerates or mutates directory contents

## Required Tests

- [x] ENFORCED_BY_TEST: latest_successful_result_is_the_only_payload_candidate
- [x] ENFORCED_BY_TEST: new_copy_attempt_invalidates_payload_and_attempt
- [x] ENFORCED_BY_TEST: mismatched_copy_result_identity_is_rejected
- [x] ENFORCED_BY_TEST: missing_or_symlink_payload_target_is_rejected
- [x] ENFORCED_BY_TEST: concurrent_payload_attempt_is_rejected
- [x] ENFORCED_BY_TEST: payload_attempt_is_one_shot
- [x] ENFORCED_BY_TEST: terminal_drag_preserves_candidate_for_retry
- [x] ENFORCED_BY_TEST: explicit_retry_reset_clears_only_the_attempt
- [x] ENFORCED_BY_TEST: reset_clears_hung_attempt_and_candidate
- [x] ENFORCED_BY_TEST: payload_errors_are_path_free
- [x] ENFORCED_BY_TYPE: TransferPayloadAttempt

## Build

- [x] Extract TransferPayloadState and contracts.
- [x] Migrate module 026 to the payload boundary.
- [x] Advance handoff IPC request to v0.2.
- [x] Split payload and provider/native tests.
- [x] Run module guard, Rust, frontend and package verification.

## Collection Payload v0.2

- [x] DOCUMENTED_ONLY: candidate binds one immutable collection revision
- [x] REVIEW_ONLY: one invalid collection member blocks the whole attempt
- [x] ENFORCED_BY_TEST: latest_collection_revision_is_the_only_payload_candidate
- [x] ENFORCED_BY_TEST: multi_directory_attempt_preserves_snapshot_order
- [x] ENFORCED_BY_TEST: stale_collection_revision_is_rejected
- [x] ENFORCED_BY_TEST: invalid_member_blocks_entire_drag_attempt
- [x] ENFORCED_BY_TYPE: TransferPayloadAttempt

- [x] Replace single-result candidate with collection snapshot candidate.
- [x] Preserve one-shot and retry behavior.
- [x] Preserve the collection candidate while explicitly clearing a stale retry attempt.
