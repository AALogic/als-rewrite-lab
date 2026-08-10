# Tasks: 016 DesktopApplicationService

- [x] DOCUMENTED_ONLY: Desktop Alpha is local-only and one-project-at-a-time
- [x] REVIEW_ONLY: service contains no ALS parsing, matching, copy or rewrite policy
- [x] REVIEW_ONLY: UI and CLI depend on the service rather than domain internals for preflight
- [x] ENFORCED_BY_TEST: valid_project_returns_desktop_preflight
- [x] ENFORCED_BY_TEST: diagnostic_report_omits_private_names_and_paths
- [x] ENFORCED_BY_TEST: missing_als_returns_structured_failure
- [x] ENFORCED_BY_TEST: desktop_analysis_is_read_only
- [x] ENFORCED_BY_TEST: desktop_analysis_is_deterministic
- [x] ENFORCED_BY_TEST: diagnostic_preserves_system_dependency_classification
- [x] ENFORCED_BY_TEST: desktop_ipc_wire_contract_is_stable
- [x] ENFORCED_BY_TYPE: DesktopAnalyzeRequest
- [x] ENFORCED_BY_TYPE: DesktopAnalyzeResult
- [x] ENFORCED_BY_TYPE: DesktopDiagnosticReport
- [x] ENFORCED_BY_TYPE: DiagnosticRequirement
- [x] ENFORCED_BY_TYPE: DesktopApplicationError
