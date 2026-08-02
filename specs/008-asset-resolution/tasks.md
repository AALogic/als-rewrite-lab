# Tasks 008: AssetResolution

Status: policy v0.3 implementation approved
Date: 2026-08-02

## Readiness

- [x] DOCUMENTED_ONLY: policy threshold remains 95 for candidate ranking
- [x] REVIEW_ONLY: recorded-path binding does not claim historical identity
- [x] REVIEW_ONLY: moved candidates require explicit source-bound user selection
- [x] DOCUMENTED_ONLY: OriginalCrc is not scored
- [x] REVIEW_ONLY: resolution remains a pure function
- [x] REVIEW_ONLY: ambiguous candidates never auto-select

## Contract

- [x] ENFORCED_BY_TEST: exact_recorded_path_is_accepted_as_current_binding
- [x] ENFORCED_BY_TEST: zero_original_file_size_is_unknown_not_conflict
- [x] ENFORCED_BY_TEST: path_observer_status_v0_2_is_consumed_without_translation
- [x] ENFORCED_BY_TEST: name_and_size_only_requires_confirmation
- [x] ENFORCED_BY_TEST: same_name_different_content_remains_distinct
- [x] ENFORCED_BY_TEST: high_score_tie_blocks_automatic_resolution
- [x] ENFORCED_BY_TEST: no_candidate_remains_unresolved
- [x] ENFORCED_BY_TEST: original_crc_alone_does_not_create_candidate
- [x] ENFORCED_BY_TEST: partial_inventory_blocks_auto_acceptance
- [x] ENFORCED_BY_TEST: untrusted_upstream_fails_closed
- [x] ENFORCED_BY_TEST: resolution_output_is_deterministic
- [x] ENFORCED_BY_TEST: fake_package_planner_receives_no_unconfirmed_selection
- [x] ENFORCED_BY_TEST: explicit_path_and_hash_selection_accepts_moved_candidate
- [x] ENFORCED_BY_TEST: changed_file_invalidates_user_selection
- [x] ENFORCED_BY_TEST: selection_for_different_als_is_rejected
- [x] ENFORCED_BY_TEST: exact_current_path_becomes_metadata_binding_without_content_identity
- [x] ENFORCED_BY_TEST: missing_current_path_is_a_non_blocking_omission
- [x] ENFORCED_BY_TEST: conflicting_size_blocks_metadata_binding
- [x] ENFORCED_BY_TEST: multiple_current_paths_block_without_content_identity
- [x] ENFORCED_BY_TEST: untrusted_assessment_fails_closed
- [x] ENFORCED_BY_TYPE: UserSelectionSet
- [x] ENFORCED_BY_TYPE: UserAssetSelection
- [x] ENFORCED_BY_TYPE: AssetResolutionResult
- [x] ENFORCED_BY_TYPE: AssetResolutionMetadata
- [x] ENFORCED_BY_TYPE: ResolutionProposal
- [x] ENFORCED_BY_TYPE: ResolutionCandidate
- [x] ENFORCED_BY_TYPE: ResolutionEvidence
- [x] ENFORCED_BY_TYPE: ResolutionDecision
- [x] ENFORCED_BY_TYPE: AssetResolutionWarning
- [x] ENFORCED_BY_TYPE: AssetResolutionError
- [x] ENFORCED_BY_TYPE: CurrentPathBindingResult
- [x] ENFORCED_BY_TYPE: CurrentPathBindingMetadata
- [x] ENFORCED_BY_TYPE: CurrentPathBinding
- [x] ENFORCED_BY_TYPE: CurrentPathBindingOmission
- [x] ENFORCED_BY_TYPE: CurrentPathBindingError

## Build

- [x] Create rescue_resolution crate.
- [x] Implement candidate generation and scoring.
- [x] Implement policy v0.3 decision gates.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 008-asset-resolution.
