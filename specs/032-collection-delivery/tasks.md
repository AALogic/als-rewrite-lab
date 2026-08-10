# Tasks 032: CollectionDelivery

Status: accepted, v0.2
Date: 2026-08-06

## Readiness

- [x] DOCUMENTED_ONLY: delivery is a validated local copy not a cloud-sync claim
- [x] DOCUMENTED_ONLY: no content hash is computed in v0.1
- [x] REVIEW_ONLY: cleanup is restricted to staging owned by the current request
- [x] REVIEW_ONLY: user-specific paths are local settings not source constants
- [x] REVIEW_ONLY: only full validated local delivery completes the active order

## Required Tests

- [x] ENFORCED_BY_TEST: delivery_plan_is_read_only_and_required
- [x] ENFORCED_BY_TEST: delivery_requires_explicit_write_consent
- [x] ENFORCED_BY_TEST: delivery_never_overwrites_existing_target
- [x] ENFORCED_BY_TEST: collision_suffix_is_deterministic_and_recorded
- [x] ENFORCED_BY_TEST: validated_delivery_preserves_project_tree
- [x] ENFORCED_BY_TEST: delivery_never_mutates_source_collection
- [x] ENFORCED_BY_TEST: failed_item_does_not_remove_completed_delivery
- [x] ENFORCED_BY_TEST: previously_delivered_items_are_skipped
- [x] ENFORCED_BY_TEST: symlink_and_special_file_sources_fail_closed
- [x] ENFORCED_BY_TEST: preferences_write_is_atomic_and_versioned
- [x] ENFORCED_BY_TEST: delivery_diagnostics_are_path_free
- [x] ENFORCED_BY_TEST: local_delivery_outcomes_drive_handoff_lifecycle
- [x] ENFORCED_BY_TYPE: CollectionDeliveryRequest
- [x] ENFORCED_BY_TYPE: CollectionDeliveryPlan
- [x] ENFORCED_BY_TYPE: CollectionDeliveryOperation
- [x] ENFORCED_BY_TYPE: CollectionDeliveryResult
- [x] ENFORCED_BY_TYPE: CollectionDeliveryItemResult
- [x] ENFORCED_BY_TYPE: CollectionDeliveryError
- [x] ENFORCED_BY_TYPE: CourierPreferences

## Build

- [x] Implement delivery service and tests.
- [x] Add Tauri settings and delivery commands.
- [x] Connect Google Drive control through opaque collection identity.
- [x] Run guards, workspace tests and manual local-folder smoke test.
