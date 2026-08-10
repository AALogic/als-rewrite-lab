import { describe, expect, it } from "vitest";
import type { ProjectCatalogListResult } from "./contracts";
import { selectedCatalogIds, visibleProjectGroups } from "./projectCatalogView";

const catalog: ProjectCatalogListResult = {
  metadata: {
    service_version: "0.1.0",
    catalog_revision: 7,
    coverage_status: "complete",
    total_group_count: 2,
    total_item_count: 2,
    visible_item_count: 2,
    hidden_backup_count: 0,
    hidden_stale_count: 0,
  },
  groups: [
    {
      group_id: "g1",
      display_name: "Album",
      group_kind: "project_folder",
      item_ids: ["a"],
      main_set_count: 1,
      backup_set_count: 0,
      freshness_status: "observed_in_latest_scan",
    },
    {
      group_id: "g2",
      display_name: "Remixes",
      group_kind: "project_folder",
      item_ids: ["b"],
      main_set_count: 1,
      backup_set_count: 0,
      freshness_status: "observed_in_latest_scan",
    },
  ],
  items: [
    {
      item_id: "a",
      live_set_id: "a",
      group_id: "g1",
      display_name: "Intro.als",
      project_display_name: "Album",
      location_kind: "main_set",
      freshness_status: "observed_in_latest_scan",
      selection_status: "selectable",
      modified_time_unix_ms: 1,
      file_size: 10,
      duplicate_display_name_count: 1,
      warning_codes: [],
    },
    {
      item_id: "b",
      live_set_id: "b",
      group_id: "g2",
      display_name: "Club Mix.als",
      project_display_name: "Remixes",
      location_kind: "main_set",
      freshness_status: "observed_in_latest_scan",
      selection_status: "selectable",
      modified_time_unix_ms: 2,
      file_size: 20,
      duplicate_display_name_count: 1,
      warning_codes: [],
    },
  ],
  warnings: [],
  errors: [],
};

describe("Project catalog view", () => {
  it("filters by group and concrete Set name without changing identity", () => {
    expect(visibleProjectGroups(catalog, "album")[0]?.items[0]?.live_set_id).toBe("a");
    expect(visibleProjectGroups(catalog, "club")[0]?.items[0]?.live_set_id).toBe("b");
  });

  it("preserves catalog display order for selected IDs", () => {
    expect(selectedCatalogIds(catalog, new Set(["b", "a"]))).toEqual(["a", "b"]);
  });

  it("does not drop a selected Set when search hides its row", () => {
    expect(visibleProjectGroups(catalog, "club")[0]?.items[0]?.live_set_id).toBe("b");
    expect(selectedCatalogIds(catalog, new Set(["a", "b"]))).toEqual(["a", "b"]);
  });
});
