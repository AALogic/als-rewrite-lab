export const PAYLOAD_DRAG_STARTED_EVENT = "universal-payload-drag-started";
export const PAYLOAD_DRAG_FINISHED_EVENT = "universal-payload-drag-finished";

export type FolderDragSurfaceRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type UniversalPayloadDragRequest = {
  request_id: string;
  collection_id: string;
  collection_revision: number;
};

export type UniversalPayloadDragPrepared = {
  schema_version: "0.1";
  attempt_id: string;
  state: "armed";
};

export type UniversalPayloadDragStarted = {
  schema_version: "0.1";
  attempt_id: string;
  state: "dragging";
};

export type UniversalPayloadDragFinished = {
  schema_version: "0.1";
  attempt_id: string;
  outcome: "dropped" | "cancelled" | "failed";
  error_code: string | null;
};
