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
- [x] ENFORCED_BY_TEST: locator_mismatch_blocks_rewrite
- [x] ENFORCED_BY_TEST: copy_only_mode_has_no_rewrite_operations
- [x] ENFORCED_BY_TEST: package_plan_is_deterministic
- [x] ENFORCED_BY_TEST: package_planner_is_pure
- [x] ENFORCED_BY_TYPE: PackagePlanningRequest
- [x] ENFORCED_BY_TYPE: PackagePlan
- [x] ENFORCED_BY_TYPE: PackagePlanMetadata
- [x] ENFORCED_BY_TYPE: PlannedSourceAls
- [x] ENFORCED_BY_TYPE: CopyOperation
- [x] ENFORCED_BY_TYPE: RewriteOperation
- [x] ENFORCED_BY_TYPE: UnresolvedPackageRequirement
- [x] ENFORCED_BY_TYPE: PackagePlanWarning
- [x] ENFORCED_BY_TYPE: PackagePlanError

## Build

- [x] Create rescue_packaging crate.
- [x] Implement handoff and target validation.
- [x] Implement deterministic copy/rewrite operation planning.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 009-package-planner.
