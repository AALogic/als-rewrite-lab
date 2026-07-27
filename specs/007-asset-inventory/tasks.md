# Tasks 007: AssetInventory

Status: ready
Date: 2026-07-27

## Readiness

- [x] DOCUMENTED_ONLY: selected roots are not a full-disk scan
- [x] DOCUMENTED_ONLY: content identity is separate from file occurrence
- [x] REVIEW_ONLY: scanner never follows symlinks
- [x] REVIEW_ONLY: scanner has no resolution or package policy

## Contract

- [x] ENFORCED_BY_TEST: recognized_audio_files_are_hashed
- [x] ENFORCED_BY_TEST: non_audio_files_are_ignored
- [x] ENFORCED_BY_TEST: identical_content_keeps_distinct_occurrences
- [x] ENFORCED_BY_TEST: same_name_different_content_stays_distinct
- [x] ENFORCED_BY_TEST: symlinks_are_not_followed
- [x] ENFORCED_BY_TEST: invalid_roots_fail_closed
- [x] ENFORCED_BY_TEST: entry_limit_marks_scan_partial
- [x] ENFORCED_BY_TEST: overlapping_roots_do_not_duplicate_paths
- [x] ENFORCED_BY_TEST: inventory_is_read_only
- [x] ENFORCED_BY_TEST: inventory_output_is_deterministic
- [x] ENFORCED_BY_TEST: fake_resolution_consumer_uses_inventory_contract
- [x] ENFORCED_BY_TYPE: AssetInventoryRequest
- [x] ENFORCED_BY_TYPE: AssetInventoryResult
- [x] ENFORCED_BY_TYPE: AssetInventoryMetadata
- [x] ENFORCED_BY_TYPE: FileOccurrence
- [x] ENFORCED_BY_TYPE: ContentRecord
- [x] ENFORCED_BY_TYPE: AssetInventoryWarning
- [x] ENFORCED_BY_TYPE: AssetInventoryError

## Build

- [x] Create rescue_catalog crate.
- [x] Implement bounded traversal and hash adapter.
- [x] Implement occurrence/content snapshot builder.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 007-asset-inventory.
