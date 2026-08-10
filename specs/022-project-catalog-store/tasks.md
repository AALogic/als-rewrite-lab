# Tasks 022: ProjectCatalogStore

Status: completed
Date: 2026-08-03

## Readiness

- [x] DOCUMENTED_ONLY: v0.1 uses private JSON behind a replaceable adapter
- [x] DOCUMENTED_ONLY: no concurrent-process writer is supported in v0.1
- [x] REVIEW_ONLY: partial coverage cannot create absence or deletion claims
- [x] REVIEW_ONLY: Store owns persistence and freshness, not grouping or UI policy
- [x] REVIEW_ONLY: stored native paths remain private local data

## Contract

- [x] ENFORCED_BY_TEST: first_snapshot_persists_and_round_trips
- [x] ENFORCED_BY_TEST: repeated_identical_snapshot_is_idempotent
- [x] ENFORCED_BY_TEST: complete_scan_marks_unseen_records_stale
- [x] ENFORCED_BY_TEST: partial_scan_retains_unseen_records_without_missing_claim
- [x] ENFORCED_BY_TEST: changed_complete_scope_retains_unseen_records_without_stale_claim
- [x] ENFORCED_BY_TEST: partial_scan_does_not_shrink_known_folder_membership
- [x] ENFORCED_BY_TEST: changed_observation_updates_same_occurrence
- [x] ENFORCED_BY_TEST: corrupt_or_unsupported_store_fails_without_overwrite
- [x] ENFORCED_BY_TEST: failed_snapshot_is_rejected_without_write
- [x] ENFORCED_BY_TEST: atomic_write_leaves_no_owned_temporary_files
- [x] ENFORCED_BY_TEST: stored_bytes_are_deterministic
- [x] ENFORCED_BY_TEST: store_diagnostics_contain_no_private_paths
- [x] ENFORCED_BY_TEST: fake_application_service_uses_stored_catalog_contract
- [x] ENFORCED_BY_TYPE: ProjectCatalogStoreRequest
- [x] ENFORCED_BY_TYPE: ProjectCatalogStoreResult
- [x] ENFORCED_BY_TYPE: ProjectCatalogLoadResult
- [x] ENFORCED_BY_TYPE: StoredProjectCatalog
- [x] ENFORCED_BY_TYPE: ProjectCatalogStoreMetadata
- [x] ENFORCED_BY_TYPE: StoredProjectFolderRecord
- [x] ENFORCED_BY_TYPE: StoredLiveSetRecord
- [x] ENFORCED_BY_TYPE: ProjectCatalogStoreWarning
- [x] ENFORCED_BY_TYPE: ProjectCatalogStoreError

## Build

- [x] Add public contracts.
- [x] Add input and state validation.
- [x] Add complete/partial merge policy.
- [x] Add deterministic JSON adapter.
- [x] Add atomic create/replace implementation.
- [x] Add round-trip and idempotency behavior.
- [x] Add tests before implementation acceptance.
- [x] Run workspace quality commands.
- [x] Run workflow_guard verify-module 022-project-catalog-store.
