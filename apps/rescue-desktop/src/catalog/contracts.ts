import type { DesktopApplicationError } from "../contracts/desktop";

export type ProjectCatalogWarning = {
  warning_code: string;
  message: string;
};

export type ProjectCatalogListMetadata = {
  service_version: string;
  catalog_revision: number;
  coverage_status: string;
  total_group_count: number;
  total_item_count: number;
  visible_item_count: number;
  hidden_backup_count: number;
  hidden_stale_count: number;
};

export type ProjectListGroup = {
  group_id: string;
  display_name: string;
  group_kind: string;
  item_ids: string[];
  main_set_count: number;
  backup_set_count: number;
  freshness_status: string;
};

export type ProjectListItem = {
  item_id: string;
  live_set_id: string;
  group_id: string;
  display_name: string;
  project_display_name: string | null;
  location_kind: string;
  freshness_status: string;
  selection_status: string;
  modified_time_unix_ms: number | null;
  file_size: number;
  duplicate_display_name_count: number;
  warning_codes: string[];
};

export type ProjectCatalogListResult = {
  metadata: ProjectCatalogListMetadata;
  groups: ProjectListGroup[];
  items: ProjectListItem[];
  warnings: ProjectCatalogWarning[];
  errors: DesktopApplicationError[];
};

export type ProjectCatalogRefreshResult = {
  service_version: string;
  request_id: string;
  refresh_status: string;
  scan_status: string;
  store_status: string;
  catalog: ProjectCatalogListResult;
  warnings: ProjectCatalogWarning[];
  errors: DesktopApplicationError[];
};

export type ProjectSelection = {
  selection_id: string;
  selection_source: string;
  live_set_id: string | null;
  native_als_path: string;
  catalog_revision: number | null;
  observation_fingerprint: string | null;
  freshness_status: string;
};

export type ProjectSelectionResult = {
  service_version: string;
  request_id: string;
  selection_status: string;
  selections: ProjectSelection[];
  warnings: ProjectCatalogWarning[];
  errors: DesktopApplicationError[];
};
