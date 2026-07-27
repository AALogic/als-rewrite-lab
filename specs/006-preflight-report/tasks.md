# Tasks 006: PreflightReport

Status: ready
Date: 2026-07-27

## Readiness

- [x] DOCUMENTED_ONLY: candidate availability is not asset resolution
- [x] DOCUMENTED_ONLY: preflight is a private local report
- [x] REVIEW_ONLY: report projection remains read-only
- [x] REVIEW_ONLY: CLI only orchestrates domain modules

## Contract

- [x] ENFORCED_BY_TEST: all_candidates_observed_remain_unresolved
- [x] ENFORCED_BY_TEST: missing_requirement_requests_asset_search
- [x] ENFORCED_BY_TEST: unknown_requirement_requests_review
- [x] ENFORCED_BY_TEST: untrusted_input_blocks_report
- [x] ENFORCED_BY_TEST: source_mismatch_blocks_report
- [x] ENFORCED_BY_TEST: local_candidate_paths_are_explainable
- [x] ENFORCED_BY_TEST: preflight_output_is_deterministic
- [x] ENFORCED_BY_TEST: preflight_cli_command_is_available
- [x] ENFORCED_BY_TYPE: PreflightReport
- [x] ENFORCED_BY_TYPE: PreflightReportMetadata
- [x] ENFORCED_BY_TYPE: PreflightProjectContext
- [x] ENFORCED_BY_TYPE: PreflightSummary
- [x] ENFORCED_BY_TYPE: PreflightRequirement
- [x] ENFORCED_BY_TYPE: PreflightNotice
- [x] ENFORCED_BY_TYPE: PreflightReportError

## Build

- [x] Implement preflight report contracts and projection.
- [x] Add read-only CLI orchestration.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 006-preflight-report.
