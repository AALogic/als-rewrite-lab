# Tasks 008: AssetResolution

Status: ready
Date: 2026-07-27

## Readiness

- [x] DOCUMENTED_ONLY: policy threshold is 95 for v0.2 ranking
- [x] REVIEW_ONLY: expected content identity is required for auto-acceptance
- [x] DOCUMENTED_ONLY: OriginalCrc is not scored
- [x] REVIEW_ONLY: resolution remains a pure function
- [x] REVIEW_ONLY: ambiguous candidates never auto-select

## Contract

- [x] ENFORCED_BY_TEST: exact_path_name_and_size_requires_confirmation_without_expected_hash
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
- [x] ENFORCED_BY_TYPE: AssetResolutionResult
- [x] ENFORCED_BY_TYPE: AssetResolutionMetadata
- [x] ENFORCED_BY_TYPE: ResolutionProposal
- [x] ENFORCED_BY_TYPE: ResolutionCandidate
- [x] ENFORCED_BY_TYPE: ResolutionEvidence
- [x] ENFORCED_BY_TYPE: ResolutionDecision
- [x] ENFORCED_BY_TYPE: AssetResolutionWarning
- [x] ENFORCED_BY_TYPE: AssetResolutionError

## Build

- [x] Create rescue_resolution crate.
- [x] Implement candidate generation and scoring.
- [x] Implement decision gates.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 008-asset-resolution.
