# Tasks 031: UniversalPayloadDrag

Status: accepted, v0.2
Date: 2026-08-06

## Readiness

- [x] DOCUMENTED_ONLY: provider opening and payload drag are independent
- [x] DOCUMENTED_ONLY: native drop never claims remote upload completion
- [x] REVIEW_ONLY: React cannot nominate payload paths
- [x] REVIEW_ONLY: one invalid folder blocks the complete snapshot
- [x] REVIEW_ONLY: native outcomes report exact collection identity to module 030

## Required Tests

- [x] ENFORCED_BY_TEST: multi_directory_attempt_preserves_snapshot_order
- [x] ENFORCED_BY_TEST: invalid_member_blocks_entire_drag_attempt
- [x] ENFORCED_BY_TEST: universal_drag_operation_is_copy_only
- [x] ENFORCED_BY_TEST: universal_drag_wire_contract_is_path_free
- [x] ENFORCED_BY_TEST: parcel_surface_rect_is_validated
- [x] ENFORCED_BY_TEST: terminal_drag_preserves_candidate_for_retry
- [x] ENFORCED_BY_TEST: unsupported_platform_fails_closed
- [x] ENFORCED_BY_TEST: native_drag_outcomes_drive_handoff_lifecycle
- [x] ENFORCED_BY_TEST: reset_after_terminal_attempt_has_no_attempt_identity
- [x] ENFORCED_BY_TEST: reset_releases_all_native_surfaces_without_attempt_identity
- [x] ENFORCED_BY_TYPE: UniversalPayloadDragRequest
- [x] ENFORCED_BY_TYPE: UniversalPayloadDragPrepared
- [x] ENFORCED_BY_TYPE: UniversalPayloadDragFinished
- [x] ENFORCED_BY_TYPE: UniversalPayloadDragError

## Manual Acceptance

- [ ] Finder receives every selected Project directory.
- [ ] The parcel-only drag image follows the pointer.
- [ ] Cancelled drag returns the ready parcel presentation.
- [ ] WeTransfer native hover/drop behavior is recorded without an upload claim.

## Reusable Intake Surface Correction

- [x] REVIEW_ONLY: a new order cannot inherit an outbound native parcel surface
- [x] Remove every native parcel surface during reset, independent of attempt identity.
- [ ] RUNTIME_TEST: after one successful handoff and New Order, Finder ALS drop is accepted again.
