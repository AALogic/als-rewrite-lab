import { describe, expect, it } from "vitest";
import catalogFixture from "../../../../contracts/desktop-ipc/v1/project-catalog-list.json";
import selectionFixture from "../../../../contracts/desktop-ipc/v1/project-selection-result.json";
import type {
  ProjectCatalogListMetadata,
  ProjectCatalogListResult,
  ProjectListGroup,
  ProjectListItem,
  ProjectSelection,
  ProjectSelectionResult,
} from "./contracts";

const catalogKeys = ["metadata", "groups", "items", "warnings", "errors"] as const satisfies readonly (keyof ProjectCatalogListResult)[];
const metadataKeys = [
  "service_version", "catalog_revision", "coverage_status", "total_group_count",
  "total_item_count", "visible_item_count", "hidden_backup_count", "hidden_stale_count",
] as const satisfies readonly (keyof ProjectCatalogListMetadata)[];
const groupKeys = [
  "group_id", "display_name", "group_kind", "item_ids", "main_set_count",
  "backup_set_count", "freshness_status",
] as const satisfies readonly (keyof ProjectListGroup)[];
const itemKeys = [
  "item_id", "live_set_id", "group_id", "display_name", "project_display_name",
  "location_kind", "freshness_status", "selection_status", "modified_time_unix_ms",
  "file_size", "duplicate_display_name_count", "warning_codes",
] as const satisfies readonly (keyof ProjectListItem)[];
const selectionResultKeys = [
  "service_version", "request_id", "selection_status", "selections", "warnings", "errors",
] as const satisfies readonly (keyof ProjectSelectionResult)[];
const selectionKeys = [
  "selection_id", "selection_source", "live_set_id", "native_als_path",
  "catalog_revision", "observation_fingerprint", "freshness_status",
] as const satisfies readonly (keyof ProjectSelection)[];

function sorted(values: readonly string[]) {
  return [...values].sort();
}

describe("Project catalog desktop wire contract", () => {
  it("matches the shared Rust and TypeScript fixture", () => {
    expect(sorted(Object.keys(catalogFixture))).toEqual(sorted(catalogKeys));
    expect(sorted(Object.keys(catalogFixture.metadata))).toEqual(sorted(metadataKeys));
    expect(sorted(Object.keys(catalogFixture.groups[0]))).toEqual(sorted(groupKeys));
    expect(sorted(Object.keys(catalogFixture.items[0]))).toEqual(sorted(itemKeys));
    expect(sorted(Object.keys(selectionFixture))).toEqual(sorted(selectionResultKeys));
    expect(sorted(Object.keys(selectionFixture.selections[0]))).toEqual(sorted(selectionKeys));
  });

  it("keeps private paths out of the Project list", () => {
    expect(JSON.stringify(catalogFixture)).not.toContain("native_als_path");
    expect(selectionFixture.selections[0].native_als_path).toContain("Test.als");
  });
});
