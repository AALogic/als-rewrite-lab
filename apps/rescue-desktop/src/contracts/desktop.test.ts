import { describe, expect, it } from "vitest";
import analysisFixture from "../../../../contracts/desktop-ipc/v1/analysis-result.json";
import previewFixture from "../../../../contracts/desktop-ipc/v1/copy-preview.json";
import executeRequestFixture from "../../../../contracts/desktop-ipc/v1/copy-execute-request.json";
import copyResultFixture from "../../../../contracts/desktop-ipc/v1/copy-result.json";
import copyErrorResultFixture from "../../../../contracts/desktop-ipc/v1/copy-error-result.json";
import type {
  DesktopAnalyzeResult,
  DesktopCopyPreview,
  DesktopCopyResult,
  DesktopCopyDiagnosticReport,
  DesktopDiagnosticReport,
  DesktopExecuteCopyRequest,
  PlanFingerprint,
} from "./desktop";

const analysisKeys = [
  "service_version",
  "request_id",
  "run_status",
  "preflight_report",
  "diagnostic_report",
  "errors",
] as const satisfies readonly (keyof DesktopAnalyzeResult)[];

const diagnosticKeys = [
  "diagnostic_schema_version",
  "request_id",
  "service_version",
  "build_commit",
  "host_os",
  "host_arch",
  "run_status",
  "elapsed_ms",
  "source_als_sha256",
  "summary",
  "requirements",
  "rewrite_compatibility",
  "error_codes",
] as const satisfies readonly (keyof DesktopDiagnosticReport)[];

const previewKeys = [
  "service_version",
  "request_id",
  "preview_status",
  "rewrite_policy",
  "source_als_path",
  "source_als_sha256",
  "plan_fingerprint",
  "target_project_root",
  "required_asset_count",
  "system_dependency_count",
  "copy_asset_count",
  "rewrite_reference_count",
  "omitted_asset_count",
  "expected_result_status",
  "diagnostic_report",
  "errors",
] as const satisfies readonly (keyof DesktopCopyPreview)[];

const copyDiagnosticKeys = [
  "diagnostic_schema_version",
  "request_id",
  "operation_kind",
  "service_version",
  "pipeline_version",
  "build_commit",
  "host_os",
  "host_arch",
  "rewrite_policy",
  "ableton_document_version",
  "ableton_creator_version",
  "ableton_minor_version",
  "compatibility_status",
  "run_status",
  "completed_stage",
  "elapsed_ms",
  "required_asset_count",
  "system_dependency_count",
  "copied_asset_count",
  "rewritten_reference_count",
  "omitted_asset_count",
  "errors",
] as const satisfies readonly (keyof DesktopCopyDiagnosticReport)[];

const fingerprintKeys = [
  "schema_version",
  "sha256",
] as const satisfies readonly (keyof PlanFingerprint)[];

const executeRequestKeys = [
  "request_id",
  "preview",
  "write_consent",
] as const satisfies readonly (keyof DesktopExecuteCopyRequest)[];

const copyResultKeys = [
  "service_version",
  "request_id",
  "run_status",
  "final_target_root",
  "system_dependency_count",
  "copied_asset_count",
  "rewritten_reference_count",
  "omitted_asset_count",
  "diagnostic_report",
  "errors",
] as const satisfies readonly (keyof DesktopCopyResult)[];

function assertAllContractKeysCovered<T extends never>(_proof?: T) {}

assertAllContractKeysCovered<
  Exclude<keyof DesktopAnalyzeResult, (typeof analysisKeys)[number]>
>();
assertAllContractKeysCovered<
  Exclude<keyof DesktopDiagnosticReport, (typeof diagnosticKeys)[number]>
>();
assertAllContractKeysCovered<
  Exclude<keyof DesktopCopyPreview, (typeof previewKeys)[number]>
>();
assertAllContractKeysCovered<
  Exclude<keyof DesktopCopyDiagnosticReport, (typeof copyDiagnosticKeys)[number]>
>();
assertAllContractKeysCovered<
  Exclude<keyof PlanFingerprint, (typeof fingerprintKeys)[number]>
>();
assertAllContractKeysCovered<
  Exclude<keyof DesktopExecuteCopyRequest, (typeof executeRequestKeys)[number]>
>();
assertAllContractKeysCovered<
  Exclude<keyof DesktopCopyResult, (typeof copyResultKeys)[number]>
>();

function sorted(values: readonly string[]) {
  return [...values].sort();
}

describe("desktop IPC wire contract", () => {
  it("desktop_ipc_wire_contract_is_stable", () => {
    expect(sorted(Object.keys(analysisFixture))).toEqual(sorted(analysisKeys));
    expect(sorted(Object.keys(analysisFixture.diagnostic_report))).toEqual(
      sorted(diagnosticKeys),
    );
    expect(sorted(Object.keys(previewFixture))).toEqual(sorted(previewKeys));
    expect(sorted(Object.keys(previewFixture.plan_fingerprint))).toEqual(
      sorted(fingerprintKeys),
    );
    expect(sorted(Object.keys(previewFixture.diagnostic_report))).toEqual(
      sorted(copyDiagnosticKeys),
    );
    expect(sorted(Object.keys(executeRequestFixture))).toEqual(
      sorted(executeRequestKeys),
    );
    expect(sorted(Object.keys(executeRequestFixture.preview))).toEqual(
      sorted(previewKeys),
    );
    expect(sorted(Object.keys(copyResultFixture))).toEqual(sorted(copyResultKeys));
    expect(sorted(Object.keys(copyErrorResultFixture))).toEqual(
      sorted(copyResultKeys),
    );
    expect(sorted(Object.keys(copyResultFixture.diagnostic_report))).toEqual(
      sorted(copyDiagnosticKeys),
    );
  });

  it("keeps safety-critical wire values explicit", () => {
    expect(previewFixture.preview_status).toBe("complete_copy_preview_ready");
    expect(previewFixture.plan_fingerprint.schema_version).toBe("0.1");
    expect(executeRequestFixture.write_consent).toBe(true);
    expect(copyResultFixture.run_status).toBe(
      "complete_copy_ready_for_manual_check",
    );
    expect(copyErrorResultFixture.run_status).toBe("preview_plan_changed");
    expect(copyErrorResultFixture.final_target_root).toBeNull();
    expect(copyErrorResultFixture.errors[0]?.error_code).toBe(
      "CURRENT_PATH_PREVIEW_PLAN_CHANGED",
    );
  });

  it("desktop_copy_error_report_is_available_on_wire", () => {
    expect(copyErrorResultFixture.diagnostic_report.operation_kind).toBe(
      "copy_execution",
    );
    expect(copyErrorResultFixture.diagnostic_report.errors[0]?.error_code).toBe(
      "CURRENT_PATH_PREVIEW_PLAN_CHANGED",
    );
  });
});
