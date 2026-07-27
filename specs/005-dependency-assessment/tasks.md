# Tasks 005: DependencyAssessment

Status: ready
Date: 2026-07-27

## Readiness

- [x] DOCUMENTED_ONLY: grouping represents equal recorded claims, not content identity
- [x] DOCUMENTED_ONLY: OriginalCrc is weak evidence only
- [x] REVIEW_ONLY: module remains a pure transformation
- [x] REVIEW_ONLY: no candidate path is selected

## Contract

- [x] ENFORCED_BY_TEST: identical_complete_claims_group
- [x] ENFORCED_BY_TEST: same_filename_different_path_does_not_group
- [x] ENFORCED_BY_TEST: same_crc_different_path_does_not_group
- [x] ENFORCED_BY_TEST: incomplete_occurrences_remain_separate
- [x] ENFORCED_BY_TEST: observed_file_candidate_does_not_resolve_asset
- [x] ENFORCED_BY_TEST: missing_candidates_are_assessed_conservatively
- [x] ENFORCED_BY_TEST: snapshot_mismatch_fails_closed
- [x] ENFORCED_BY_TEST: occurrence_handoff_mismatch_fails_closed
- [x] ENFORCED_BY_TEST: assessment_output_is_deterministic
- [x] ENFORCED_BY_TEST: fake_preflight_consumer_uses_assessment_contract
- [x] ENFORCED_BY_TYPE: DependencyAssessmentResult
- [x] ENFORCED_BY_TYPE: DependencyAssessmentMetadata
- [x] ENFORCED_BY_TYPE: RequiredAsset
- [x] ENFORCED_BY_TYPE: RequiredAssetCandidateObservation
- [x] ENFORCED_BY_TYPE: DependencyAssessmentWarning
- [x] ENFORCED_BY_TYPE: DependencyAssessmentError

## Build

- [x] Implement dependency_assessment models.
- [x] Implement handoff validation and grouping.
- [x] Implement conservative status and risk aggregation.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 005-dependency-assessment.
