export type DesktopApplicationError = {
  error_code: string;
  stage: string;
  message: string;
};

export type PreflightSummary = {
  overall_status: string;
  reference_occurrence_count: number;
  required_asset_count: number;
  system_dependency_count: number;
  candidate_observed_count: number;
  needs_search_count: number;
  unknown_count: number;
  unresolved_count: number;
};

export type PreflightRequirement = {
  required_asset_id: string;
  filename: string | null;
  occurrence_count: number;
  source_category: string;
  management_class: string;
  portability_status: string;
  availability_status: string;
  resolution_status: string;
  candidate_paths: string[];
  risk_flags: string[];
};

export type PreflightReport = {
  report_metadata: { source_als_path: string; source_file_hash: string };
  project: {
    discovery_status: string;
    confirmed_project_root: string | null;
    set_location: string;
  };
  summary: PreflightSummary;
  requirements: PreflightRequirement[];
  notices: Array<{ notice_code: string; message: string }>;
  errors: Array<{ error_code: string; message: string }>;
};

export type DesktopAnalyzeResult = {
  service_version: string;
  request_id: string;
  run_status: "analysis_complete" | "analysis_failed";
  preflight_report: PreflightReport | null;
  diagnostic_report: DesktopDiagnosticReport;
  errors: DesktopApplicationError[];
};

export type RewriteReferenceCompatibility = {
  ref_id: number;
  usage_context: string;
  relative_path_type: string | null;
  strict_supported: boolean;
  compatibility_lab_supported: boolean;
  structure_profile: string;
  reason_codes: string[];
};

export type RewriteCompatibilityAssessment = {
  schema_version: string;
  document_profile: string;
  evidence_status: string;
  ableton_document_version: string | null;
  ableton_creator_version: string | null;
  ableton_minor_version: string | null;
  ableton_schema_change_count: string | null;
  active_reference_count: number;
  strict_supported_count: number;
  lab_compatible_count: number;
  unsupported_shape_count: number;
  references: RewriteReferenceCompatibility[];
};

export type DesktopDiagnosticReport = {
  diagnostic_schema_version: string;
  request_id: string;
  service_version: string;
  build_commit: string;
  host_os: string;
  host_arch: string;
  run_status: string;
  elapsed_ms: number;
  source_als_sha256: string | null;
  summary: PreflightSummary | null;
  requirements: Array<{
    required_asset_id: string;
    occurrence_count: number;
    source_category: string;
    management_class: string;
    portability_status: string;
    availability_status: string;
    resolution_status: string;
    risk_flags: string[];
  }>;
  rewrite_compatibility: RewriteCompatibilityAssessment | null;
  error_codes: string[];
};

export type DesktopApplicationProfile = {
  schema_version: string;
  application_profile: "strict_alpha" | "compatibility_lab";
  experimental_compatibility_available: boolean;
};

export type PlanFingerprint = {
  schema_version: string;
  sha256: string;
};

export type DesktopDiagnosticError = {
  error_code: string;
  stage: string;
  message: string;
};

export type DesktopCopyDiagnosticReport = {
  diagnostic_schema_version: string;
  request_id: string;
  operation_kind: string;
  service_version: string;
  pipeline_version: string;
  build_commit: string;
  host_os: string;
  host_arch: string;
  rewrite_policy: string;
  ableton_document_version: string | null;
  ableton_creator_version: string | null;
  ableton_minor_version: string | null;
  compatibility_status: string;
  run_status: string;
  completed_stage: string;
  elapsed_ms: number;
  required_asset_count: number;
  system_dependency_count: number;
  copied_asset_count: number;
  rewritten_reference_count: number;
  omitted_asset_count: number;
  errors: DesktopDiagnosticError[];
};

export type DesktopCopyPreview = {
  service_version: string;
  request_id: string;
  preview_status: string;
  rewrite_policy: string;
  source_als_path: string;
  source_als_sha256: string;
  plan_fingerprint: PlanFingerprint | null;
  target_project_root: string;
  required_asset_count: number;
  system_dependency_count: number;
  copy_asset_count: number;
  rewrite_reference_count: number;
  omitted_asset_count: number;
  expected_result_status: string;
  diagnostic_report: DesktopCopyDiagnosticReport;
  errors: DesktopApplicationError[];
};

export type DesktopExecuteCopyRequest = {
  request_id: string;
  preview: DesktopCopyPreview;
  write_consent: boolean;
};

export type DesktopCopyResult = {
  service_version: string;
  request_id: string;
  run_status: string;
  final_target_root: string | null;
  system_dependency_count: number;
  copied_asset_count: number;
  rewritten_reference_count: number;
  omitted_asset_count: number;
  diagnostic_report: DesktopCopyDiagnosticReport;
  errors: DesktopApplicationError[];
};

export type CompatibilityTestReportRequest = {
  analysis_report: DesktopDiagnosticReport;
  copy_report: DesktopCopyDiagnosticReport | null;
  manual_verification_outcome:
    | "not_checked"
    | "opened_without_missing_files"
    | "opened_with_missing_files"
    | "failed_to_open";
  tested_ableton_version: string | null;
};

export type CompatibilityTestReport = {
  report_schema_version: string;
  application_profile: string;
  build_commit: string;
  host_os: string;
  host_arch: string;
  analysis_report: DesktopDiagnosticReport;
  copy_report: DesktopCopyDiagnosticReport | null;
  manual_verification_outcome: string;
  tested_ableton_version: string | null;
  provisional_conclusion: string;
};
