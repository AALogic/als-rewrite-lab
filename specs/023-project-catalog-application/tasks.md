# Tasks 023: ProjectCatalogApplicationService

Status: completed
Date: 2026-08-03

## Readiness

- [x] DOCUMENTED_ONLY: one visible item represents one concrete LiveSet
- [x] DOCUMENTED_ONLY: platform roots and private store path belong to Tauri adapters
- [x] REVIEW_ONLY: UI contains no grouping, freshness or selection policy
- [x] REVIEW_ONLY: manual and catalog input converge on ProjectSelection
- [x] REVIEW_ONLY: existing one-project analysis/copy policy remains unchanged

## Contract

- [x] ENFORCED_BY_TEST: refresh_composes_scan_build_store_and_list
- [x] ENFORCED_BY_TEST: default_list_hides_backups_and_stale_records
- [x] ENFORCED_BY_TEST: retained_partial_records_remain_visible_with_warning
- [x] ENFORCED_BY_TEST: multiple_main_sets_remain_separate_items
- [x] ENFORCED_BY_TEST: duplicate_display_names_never_merge
- [x] ENFORCED_BY_TEST: ambiguous_and_ungrouped_sets_remain_explicit
- [x] ENFORCED_BY_TEST: project_list_contains_no_native_paths
- [x] ENFORCED_BY_TEST: catalog_selection_requires_matching_revision
- [x] ENFORCED_BY_TEST: stale_catalog_item_cannot_be_selected
- [x] ENFORCED_BY_TEST: manual_and_catalog_inputs_share_selection_contract
- [x] ENFORCED_BY_TEST: duplicate_selection_fails_closed
- [x] ENFORCED_BY_TEST: catalog_application_diagnostics_are_path_free
- [x] ENFORCED_BY_TEST: fake_batch_consumer_receives_ordered_selections
- [x] ENFORCED_BY_TEST: project_catalog_wire_contract_is_stable
- [x] ENFORCED_BY_TYPE: ProjectCatalogRefreshRequest
- [x] ENFORCED_BY_TYPE: ProjectCatalogRefreshResult
- [x] ENFORCED_BY_TYPE: ProjectCatalogListRequest
- [x] ENFORCED_BY_TYPE: ProjectCatalogListResult
- [x] ENFORCED_BY_TYPE: ProjectCatalogListMetadata
- [x] ENFORCED_BY_TYPE: ProjectListGroup
- [x] ENFORCED_BY_TYPE: ProjectListItem
- [x] ENFORCED_BY_TYPE: ProjectSelectionRequest
- [x] ENFORCED_BY_TYPE: ProjectSelectionResult
- [x] ENFORCED_BY_TYPE: ProjectSelection
- [x] ENFORCED_BY_TYPE: ProjectCatalogApplicationWarning
- [x] ENFORCED_BY_TYPE: ProjectCatalogApplicationError

## Build

- [x] Add public application contracts.
- [x] Add refresh orchestration.
- [x] Add path-redacted list projection.
- [x] Add explicit selection resolution.
- [x] Add Rust contract and behavior tests.
- [x] Add private Tauri store/root adapters and commands.
- [x] Add TypeScript contracts and contract tests.
- [x] Add Project catalog UI without changing domain policy.
- [x] Run backend, frontend and workflow verification.
