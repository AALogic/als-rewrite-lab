# Tasks 009: PackagePlanner

Status: ready
Date: 2026-07-27

## Readiness

- [x] DOCUMENTED_ONLY: rewrite handoff is snapshot-bound under ADR-005
- [x] DOCUMENTED_ONLY: rescue rewrite profile remains laboratory-only
- [x] REVIEW_ONLY: planner has no filesystem effects
- [x] REVIEW_ONLY: target collisions and ambiguity block

## Contract

- [x] ENFORCED_BY_TEST: valid_lab_input_builds_copy_and_rewrite_plan
- [x] ENFORCED_BY_TEST: duplicate_occurrences_share_audio_copy
- [x] ENFORCED_BY_TEST: unresolved_decision_blocks_plan
- [x] ENFORCED_BY_TEST: target_equal_to_source_is_rejected
- [x] ENFORCED_BY_TEST: different_content_same_target_name_blocks
- [x] ENFORCED_BY_TEST: unsupported_live_version_blocks_rewrite
- [x] ENFORCED_BY_TEST: unsupported_relative_path_type_blocks_rewrite
- [x] ENFORCED_BY_TEST: unknown_rewrite_support_blocks_rewrite
- [x] ENFORCED_BY_TEST: outdated_resolution_policy_blocks_plan
- [x] ENFORCED_BY_TEST: zero_reference_laboratory_plan_is_blocked
- [x] ENFORCED_BY_TEST: locator_mismatch_blocks_rewrite
- [x] ENFORCED_BY_TEST: copy_only_mode_has_no_rewrite_operations
- [x] ENFORCED_BY_TEST: missing_current_path_produces_non_blocking_incomplete_plan
- [x] ENFORCED_BY_TEST: available_current_path_produces_copy_and_rewrite
- [x] ENFORCED_BY_TEST: safety_failure_still_blocks_current_paths_copy
- [x] ENFORCED_BY_TEST: package_plan_is_deterministic
- [x] ENFORCED_BY_TEST: package_planner_is_pure
- [x] ENFORCED_BY_TEST: project_local_type3_preserves_target_and_changes_path_only
- [x] ENFORCED_BY_TEST: mixed_type1_and_type3_plan_has_no_orphan_audio_copy
- [x] ENFORCED_BY_TEST: unsupported_existing_reference_blocks_without_audio_copy
- [x] ENFORCED_BY_TEST: unsafe_type3_relative_path_blocks_without_audio_copy
- [x] ENFORCED_BY_TEST: metadata_only_current_path_plan_has_no_audio_hash_or_content_id
- [x] ENFORCED_BY_TYPE: PackagePlanningRequest
- [x] ENFORCED_BY_TYPE: PackagePlan
- [x] ENFORCED_BY_TYPE: PackagePlanMetadata
- [x] ENFORCED_BY_TYPE: PlannedSourceAls
- [x] ENFORCED_BY_TYPE: CreateDirectoryOperation
- [x] ENFORCED_BY_TYPE: CopyOperation
- [x] ENFORCED_BY_TYPE: RewriteOperation
- [x] ENFORCED_BY_TYPE: UnresolvedPackageRequirement
- [x] ENFORCED_BY_TYPE: SystemDependencyRequirement
- [x] ENFORCED_BY_TEST: confirmed_core_library_dependency_is_left_system_managed_without_blocking
- [x] ENFORCED_BY_TEST: plan_fingerprint_is_order_independent
- [x] ENFORCED_BY_TEST: plan_fingerprint_changes_with_semantic_plan
- [x] ENFORCED_BY_TYPE: PackagePlanWarning
- [x] ENFORCED_BY_TYPE: PackagePlanError
- [x] ENFORCED_BY_TYPE: PlanFingerprint

## Build

- [x] Create rescue_packaging crate.
- [x] Implement handoff and target validation.
- [x] Implement deterministic copy/rewrite operation planning.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 009-package-planner.
