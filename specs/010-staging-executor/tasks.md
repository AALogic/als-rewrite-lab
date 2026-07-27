# Tasks: 010 StagingExecutor

- [x] Define StagingExecutionRequest and StagingExecutionResult v0.1.
- [x] ENFORCED_BY_TEST: ready_plan_is_copied_and_hash_verified_in_new_staging_root
- [x] ENFORCED_BY_TEST: source_hash_mismatch_fails_before_target_promotion
- [x] ENFORCED_BY_TEST: existing_staging_root_is_rejected_without_writes
- [x] ENFORCED_BY_TEST: unsafe_relative_target_is_rejected
- [x] ENFORCED_BY_TEST: blocked_plan_is_rejected
- [x] ENFORCED_BY_TEST: repeated_execution_reports_existing_staging_instead_of_overwriting
- [x] ENFORCED_BY_TEST: final_target_and_sources_remain_untouched
- [x] ENFORCED_BY_TEST: symlink_source_is_rejected
- [x] ENFORCED_BY_TYPE: StagingExecutionRequest
- [x] ENFORCED_BY_TYPE: StagingExecutionResult
- [x] ENFORCED_BY_TYPE: StagingExecutionMetadata
- [x] ENFORCED_BY_TYPE: CopyExecutionRecord
- [x] ENFORCED_BY_TYPE: StagingExecutionWarning
- [x] ENFORCED_BY_TYPE: StagingExecutionError
- [x] REVIEW_ONLY: executor does not choose assets or rewrite ALS
- [x] REVIEW_ONLY: only an absent staging root may be created
- [x] DOCUMENTED_ONLY: malicious local path-swap hardening remains pre-release work

