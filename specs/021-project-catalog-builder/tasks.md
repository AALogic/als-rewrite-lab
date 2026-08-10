# Tasks 021: ProjectCatalogBuilder

Status: completed
Date: 2026-08-03

## Readiness

- [x] DOCUMENTED_ONLY: catalog IDs are local path occurrences, not musical-work identity
- [x] DOCUMENTED_ONLY: exact Backup component is the only automatic backup rule in v0.1
- [x] REVIEW_ONLY: builder has no filesystem access
- [x] REVIEW_ONLY: builder does not select a primary Set or infer version families
- [x] REVIEW_ONLY: builder has no dependency, package, copy or rewrite policy

## Contract

- [x] ENFORCED_BY_TEST: standard_project_builds_physical_catalog
- [x] ENFORCED_BY_TEST: multiple_main_sets_are_retained_without_primary_selection
- [x] ENFORCED_BY_TEST: backup_set_is_retained_separately
- [x] ENFORCED_BY_TEST: standalone_set_remains_ungrouped
- [x] ENFORCED_BY_TEST: same_display_names_at_different_paths_do_not_merge
- [x] ENFORCED_BY_TEST: nested_marker_candidates_remain_ambiguous
- [x] ENFORCED_BY_TEST: marker_without_set_is_retained
- [x] ENFORCED_BY_TEST: partial_and_cancelled_scan_coverage_propagates
- [x] ENFORCED_BY_TEST: failed_or_inconsistent_scan_is_rejected
- [x] ENFORCED_BY_TEST: catalog_output_is_deterministic
- [x] ENFORCED_BY_TEST: path_ids_change_when_native_paths_change
- [x] ENFORCED_BY_TEST: catalog_warnings_contain_no_native_paths
- [x] ENFORCED_BY_TEST: fake_catalog_store_uses_snapshot_contract
- [x] ENFORCED_BY_TYPE: ProjectCatalogBuildRequest
- [x] ENFORCED_BY_TYPE: ProjectCatalogSnapshot
- [x] ENFORCED_BY_TYPE: ProjectCatalogMetadata
- [x] ENFORCED_BY_TYPE: ProjectFolderRecord
- [x] ENFORCED_BY_TYPE: LiveSetRecord
- [x] ENFORCED_BY_TYPE: ProjectCatalogWarning
- [x] ENFORCED_BY_TYPE: ProjectCatalogError

## Build

- [x] Add catalog public contracts.
- [x] Add upstream validation.
- [x] Add deterministic path identity policy.
- [x] Add pure physical grouping and association.
- [x] Add location and Backup classification.
- [x] Add path-free warning projection.
- [x] Add required tests before implementation acceptance.
- [x] Run workspace quality commands.
- [x] Run workflow_guard verify-module 021-project-catalog-builder.
