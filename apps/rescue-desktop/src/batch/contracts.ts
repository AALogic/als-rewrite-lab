import type {
  DesktopApplicationError,
  DesktopCopyPreview,
  DesktopCopyResult,
} from "../contracts/desktop";
import type { ProjectSelection } from "../catalog/contracts";

export type BatchPrepareCopyRequest = {
  request_id: string;
  selections: ProjectSelection[];
  destination_parent: string;
  experimental_compatibility_consent: boolean;
};

export type BatchCopySummary = {
  total_job_count: number;
  ready_job_count: number;
  blocked_job_count: number;
  completed_job_count: number;
  incomplete_job_count: number;
  failed_job_count: number;
  cancelled_job_count: number;
};

export type BatchCopyDiagnosticReport = {
  diagnostic_schema_version: string;
  request_id: string;
  service_version: string;
  host_os: string;
  host_arch: string;
  run_status: string;
  elapsed_ms: number;
  summary: BatchCopySummary;
  error_codes: string[];
};

export type BatchPreviewJob = {
  job_id: string;
  selection_id: string;
  source_als_path: string;
  target_project_root: string;
  job_status: string;
  preview: DesktopCopyPreview | null;
  errors: DesktopApplicationError[];
};

export type BatchCopyPreview = {
  service_version: string;
  request_id: string;
  preview_status: string;
  destination_parent: string;
  jobs: BatchPreviewJob[];
  summary: BatchCopySummary;
  diagnostic_report: BatchCopyDiagnosticReport;
  errors: DesktopApplicationError[];
};

export type BatchExecuteCopyRequest = {
  request_id: string;
  preview: BatchCopyPreview;
  write_consent: boolean;
};

export type BatchCopyJobResult = {
  job_id: string;
  selection_id: string;
  source_als_path: string;
  target_project_root: string;
  job_status: string;
  result: DesktopCopyResult | null;
  errors: DesktopApplicationError[];
};

export type BatchCopyResult = {
  service_version: string;
  request_id: string;
  run_status: string;
  destination_parent: string;
  jobs: BatchCopyJobResult[];
  summary: BatchCopySummary;
  diagnostic_report: BatchCopyDiagnosticReport;
  errors: DesktopApplicationError[];
};

export type BatchProgressEvent = {
  request_id: string;
  stage: string;
  job_id: string | null;
  job_index: number;
  total_job_count: number;
  completed_job_count: number;
  failed_job_count: number;
};
