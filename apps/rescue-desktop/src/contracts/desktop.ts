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

export type DesktopDiagnosticReport = {
  diagnostic_schema_version: string;
  request_id: string;
  service_version: string;
  run_status: string;
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
  error_codes: string[];
};

export type PlanFingerprint = {
  schema_version: string;
  sha256: string;
};

export type DesktopCopyPreview = {
  service_version: string;
  request_id: string;
  preview_status: string;
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
  errors: DesktopApplicationError[];
};
