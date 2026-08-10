# Tasks 020: ALSProjectScanner

Status: complete
Date: 2026-08-03

## Readiness

- [x] DOCUMENTED_ONLY: complete means complete only within approved roots and exclusions
- [x] DOCUMENTED_ONLY: Windows reparse behavior needs native evidence
- [x] REVIEW_ONLY: scanner does not parse ALS or inspect audio
- [x] REVIEW_ONLY: scanner does not group Project folders or infer version families
- [x] REVIEW_ONLY: scanner performs no filesystem writes

## Contract

- [x] ENFORCED_BY_TEST: standard_project_entries_are_observed
- [x] ENFORCED_BY_TEST: backup_and_standalone_als_are_retained
- [x] ENFORCED_BY_TEST: invalid_gzip_als_is_not_opened
- [x] ENFORCED_BY_TEST: uppercase_als_is_recognized
- [x] ENFORCED_BY_TEST: non_als_files_are_ignored
- [x] ENFORCED_BY_TEST: multiple_main_sets_are_not_collapsed
- [x] ENFORCED_BY_TEST: same_names_in_different_roots_stay_distinct
- [x] ENFORCED_BY_TEST: overlapping_roots_do_not_duplicate_observations
- [x] ENFORCED_BY_TEST: explicit_exclusions_are_honored
- [x] ENFORCED_BY_TEST: symlinks_are_not_followed
- [x] ENFORCED_BY_TEST: entry_limit_marks_project_scan_partial
- [x] ENFORCED_BY_TEST: depth_limit_marks_project_scan_partial
- [x] ENFORCED_BY_TEST: inaccessible_directory_preserves_partial_results
- [x] ENFORCED_BY_TEST: cancellation_returns_collected_observations
- [x] ENFORCED_BY_TEST: progress_events_contain_no_private_paths
- [x] ENFORCED_BY_TEST: project_scan_is_read_only
- [x] ENFORCED_BY_TEST: project_scan_output_is_deterministic
- [x] ENFORCED_BY_TEST: fake_catalog_builder_uses_project_scan_contract
- [x] ENFORCED_BY_TYPE: ProjectScanRequest
- [x] ENFORCED_BY_TYPE: ProjectScanResult
- [x] ENFORCED_BY_TYPE: ProjectScanMetadata
- [x] ENFORCED_BY_TYPE: ALSFileObservation
- [x] ENFORCED_BY_TYPE: ProjectMarkerObservation
- [x] ENFORCED_BY_TYPE: ProjectScanProgress
- [x] ENFORCED_BY_TYPE: ProjectScanWarning
- [x] ENFORCED_BY_TYPE: ProjectScanError

## Build

- [x] Add project scan public contracts.
- [x] Add cancellation/progress observer boundary.
- [x] Implement request/root validation.
- [x] Implement bounded read-only traversal.
- [x] Implement deterministic observation normalization.
- [x] Add all required tests before implementation acceptance.
- [x] Run workspace quality commands.
- [x] Run workflow_guard verify-module 020-als-project-scanner.
- [x] Record native macOS experiment evidence.
- [x] Leave Windows reparse verification as an explicit native follow-up if
      physical Windows is unavailable in this environment.
