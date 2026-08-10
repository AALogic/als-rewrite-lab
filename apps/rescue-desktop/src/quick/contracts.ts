export type CourierDisplayItem = {
  work_item_id: string;
  source_display_name: string;
  status: "queued" | "processing" | "completed" | "completed_incomplete" | "blocked" | "failed";
};

export type CourierDisplaySnapshot = {
  schema_version: "0.2";
  collection_id: string | null;
  revision: number;
  work_phase: "empty" | "collecting" | "processing" | "ready";
  handoff_status: "not_started" | "in_progress" | "completed" | "retryable_issue";
  handoff_channel: "native_drag" | "local_google_drive" | null;
  destination_display_path: string;
  items: CourierDisplayItem[];
  pending_item_count: number;
  completed_item_count: number;
  incomplete_item_count: number;
  failed_item_count: number;
  omitted_asset_count: number;
  delivery_available: boolean;
  local_delivery_configured: boolean;
};

export type CourierPreferences = {
  schema_version: "0.1";
  output_parent: string;
  local_delivery_root: string | null;
};

export type CollectionDeliveryResult = {
  schema_version: "0.1";
  request_id: string;
  run_status: "completed" | "completed_with_issues" | "failed";
  collection_id: string;
  collection_revision: number;
  items: Array<{
    item_id: string;
    status: "completed" | "failed";
    final_target_root: string | null;
    file_count: number;
    total_bytes: number;
    error_code: string | null;
  }>;
  skipped_item_ids: string[];
  error_codes: string[];
};

export const emptyCourierSnapshot: CourierDisplaySnapshot = {
  schema_version: "0.2",
  collection_id: null,
  revision: 0,
  work_phase: "empty",
  handoff_status: "not_started",
  handoff_channel: null,
  destination_display_path: "",
  items: [],
  pending_item_count: 0,
  completed_item_count: 0,
  incomplete_item_count: 0,
  failed_item_count: 0,
  omitted_asset_count: 0,
  delivery_available: false,
  local_delivery_configured: false,
};
