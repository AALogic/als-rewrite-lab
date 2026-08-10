import { describe, expect, it } from "vitest";
import previewFixture from "../../../../contracts/desktop-ipc/v1/batch-preview.json";
import resultFixture from "../../../../contracts/desktop-ipc/v1/batch-result.json";
import type {
  BatchCopyDiagnosticReport,
  BatchCopyJobResult,
  BatchCopyPreview,
  BatchCopyResult,
  BatchCopySummary,
  BatchPreviewJob,
} from "./contracts";

const previewKeys = ["service_version", "request_id", "preview_status", "destination_parent", "jobs", "summary", "diagnostic_report", "errors"] as const satisfies readonly (keyof BatchCopyPreview)[];
const resultKeys = ["service_version", "request_id", "run_status", "destination_parent", "jobs", "summary", "diagnostic_report", "errors"] as const satisfies readonly (keyof BatchCopyResult)[];
const previewJobKeys = ["job_id", "selection_id", "source_als_path", "target_project_root", "job_status", "preview", "errors"] as const satisfies readonly (keyof BatchPreviewJob)[];
const resultJobKeys = ["job_id", "selection_id", "source_als_path", "target_project_root", "job_status", "result", "errors"] as const satisfies readonly (keyof BatchCopyJobResult)[];
const summaryKeys = ["total_job_count", "ready_job_count", "blocked_job_count", "completed_job_count", "incomplete_job_count", "failed_job_count", "cancelled_job_count"] as const satisfies readonly (keyof BatchCopySummary)[];
const diagnosticKeys = ["diagnostic_schema_version", "request_id", "service_version", "host_os", "host_arch", "run_status", "elapsed_ms", "summary", "error_codes"] as const satisfies readonly (keyof BatchCopyDiagnosticReport)[];

function sorted(values: readonly string[]) { return [...values].sort(); }

describe("Batch desktop wire contract", () => {
  it("matches shared Rust and TypeScript fixtures", () => {
    expect(sorted(Object.keys(previewFixture))).toEqual(sorted(previewKeys));
    expect(sorted(Object.keys(resultFixture))).toEqual(sorted(resultKeys));
    expect(sorted(Object.keys(previewFixture.jobs[0]))).toEqual(sorted(previewJobKeys));
    expect(sorted(Object.keys(resultFixture.jobs[0]))).toEqual(sorted(resultJobKeys));
    expect(sorted(Object.keys(previewFixture.summary))).toEqual(sorted(summaryKeys));
    expect(sorted(Object.keys(resultFixture.diagnostic_report))).toEqual(sorted(diagnosticKeys));
  });

  it("keeps the aggregate diagnostic path-free", () => {
    const diagnostic = JSON.stringify(resultFixture.diagnostic_report);
    expect(diagnostic).not.toContain("/fixture");
    expect(diagnostic).not.toContain("Set.als");
  });
});
