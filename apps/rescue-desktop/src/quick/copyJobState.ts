import type { CourierDisplaySnapshot } from "./contracts";

export type QuickCopyJobPhase =
  | "arriving"
  | "waiting"
  | "working"
  | "ready"
  | "delivering"
  | "completed"
  | "unable";

export function phaseForSnapshot(
  snapshot: CourierDisplaySnapshot,
  arriving: boolean,
  deliveryWorking: boolean,
  unable: boolean,
): QuickCopyJobPhase {
  if (unable) return "unable";
  if (arriving) return "arriving";
  if (deliveryWorking || snapshot.handoff_status === "in_progress") return "delivering";
  if (snapshot.handoff_status === "completed") return "completed";
  if (snapshot.work_phase === "processing") return "working";
  if (snapshot.pending_item_count > 0 || snapshot.work_phase === "collecting") return "waiting";
  if (snapshot.work_phase === "ready" && snapshot.delivery_available) return "ready";
  return "waiting";
}

export function canExecuteCollection(snapshot: CourierDisplaySnapshot): boolean {
  return snapshot.work_phase !== "processing"
    && snapshot.handoff_status !== "in_progress"
    && snapshot.handoff_status !== "completed"
    && snapshot.pending_item_count > 0;
}

export function snapshotRequiresPayloadReset(snapshot: CourierDisplaySnapshot): boolean {
  if (snapshot.handoff_status === "in_progress" && snapshot.handoff_channel === "native_drag") {
    return false;
  }
  return snapshot.work_phase !== "ready"
    || snapshot.pending_item_count > 0
    || !snapshot.delivery_available
    || snapshot.handoff_status === "completed";
}

export function shouldDiscardSessionOnClose(
  snapshot: CourierDisplaySnapshot,
  deliveryWorking = false,
): boolean {
  return !deliveryWorking
    && snapshot.work_phase !== "processing"
    && snapshot.handoff_status !== "in_progress";
}
