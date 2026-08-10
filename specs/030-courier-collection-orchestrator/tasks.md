# Tasks 030: CourierCollectionOrchestrator

Status: accepted, v0.2
Date: 2026-08-06

## Readiness

- [x] DOCUMENTED_ONLY: active module-024 requests are immutable
- [x] DOCUMENTED_ONLY: new intake during processing becomes a later wave
- [x] REVIEW_ONLY: orchestrator owns no copy rewrite validation or target naming policy
- [x] REVIEW_ONLY: collection snapshots remain backend private
- [x] DOCUMENTED_ONLY: a completed handoff is terminal for the active order
- [x] REVIEW_ONLY: provider adapters report outcomes but do not own collection lifecycle

## Required Tests

- [x] ENFORCED_BY_TEST: play_freezes_all_pending_items_into_one_wave
- [x] ENFORCED_BY_TEST: intake_during_active_wave_enters_next_wave
- [x] ENFORCED_BY_TEST: drain_boundary_never_loses_or_duplicates_item
- [x] ENFORCED_BY_TEST: active_wave_is_never_mutated
- [x] ENFORCED_BY_TEST: successful_waves_accumulate_one_collection
- [x] ENFORCED_BY_TEST: failed_and_blocked_jobs_do_not_enter_collection
- [x] ENFORCED_BY_TEST: ready_collection_returns_to_collecting_after_new_intake
- [x] ENFORCED_BY_TEST: delivery_snapshot_is_revision_bound_and_immutable
- [x] ENFORCED_BY_TEST: wave_result_identity_mismatch_fails_closed
- [x] ENFORCED_BY_TEST: orchestrator_errors_are_path_free
- [x] ENFORCED_BY_TEST: only_one_handoff_can_be_active
- [x] ENFORCED_BY_TEST: handoff_identity_and_revision_must_match
- [x] ENFORCED_BY_TEST: completed_handoff_is_terminal_for_direct_intake
- [x] ENFORCED_BY_TEST: cancelled_and_retryable_handoffs_keep_payload_available
- [x] ENFORCED_BY_TEST: explicit_reopen_preserves_collection_revision
- [x] ENFORCED_BY_TEST: completed_handoff_new_intake_starts_fresh_collection
- [x] ENFORCED_BY_TEST: intake_during_handoff_starts_next_order_exactly_once
- [x] ENFORCED_BY_TEST: cancelled_handoff_merges_deferred_intake_into_current_order
- [x] ENFORCED_BY_TYPE: CourierCollectionOrchestrator
- [x] ENFORCED_BY_TYPE: CourierBatchWave
- [x] ENFORCED_BY_TYPE: CourierCollectionItem
- [x] ENFORCED_BY_TYPE: CourierCollectionSnapshot
- [x] ENFORCED_BY_TYPE: DeliveryCollectionSnapshot
- [x] ENFORCED_BY_TYPE: CourierCollectionError
- [x] ENFORCED_BY_TYPE: CourierHandoffSnapshot

## Build

- [x] Add pure orchestrator state and tests.
- [x] Add Tauri runtime coordinator and path-free events.
- [x] Reuse module 024 without source edits.
- [x] Run readiness and verification guards.
- [x] Add handoff lifecycle and runtime fresh-order rotation.
- [x] Connect native drag and local delivery outcomes.
- [ ] OWNER_TEST: verify packaged native-drop terminal and retry behavior.
