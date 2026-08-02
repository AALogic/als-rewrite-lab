# Tasks: 017 CurrentPathCopyPipeline

- [x] DOCUMENTED_ONLY: missing files are reported and never searched in this flow
- [x] REVIEW_ONLY: the pipeline delegates domain behavior to existing modules
- [x] REVIEW_ONLY: current-path audio does not invoke content-addressed inventory
- [x] ENFORCED_BY_TEST: complete_current_path_copy_is_promoted
- [x] ENFORCED_BY_TEST: current_path_audio_uses_metadata_only_verification_end_to_end
- [x] ENFORCED_BY_TEST: missing_asset_creates_incomplete_copy
- [x] ENFORCED_BY_TEST: missing_reference_remains_unchanged
- [x] ENFORCED_BY_TEST: available_assets_are_relinked_when_another_asset_is_missing
- [x] ENFORCED_BY_TEST: unsafe_output_still_blocks_before_write
- [x] ENFORCED_BY_TEST: current_path_copy_keeps_sources_read_only
- [x] ENFORCED_BY_TEST: project_local_type3_copy_preserves_structure_and_rewrites_path_only
- [x] ENFORCED_BY_TYPE: CurrentPathCopyRequest
- [x] ENFORCED_BY_TYPE: CurrentPathCopyResult
- [x] ENFORCED_BY_TEST: confirmed_core_library_dependency_is_left_system_managed_without_blocking
- [x] ENFORCED_BY_TEST: changed_plan_fingerprint_blocks_before_write
- [x] ENFORCED_BY_TEST: matching_plan_fingerprint_allows_execution
