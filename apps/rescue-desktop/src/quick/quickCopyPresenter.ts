import type { AssistantPresentationModel } from "../assistant/contracts";
import type { CourierDisplaySnapshot } from "./contracts";
import { canExecuteCollection, phaseForSnapshot } from "./copyJobState";
import type { TransferPayloadState } from "./transferPayloadState";
import { isTransferPayloadBusy } from "./transferPayloadState";

export type QuickCopyControls = {
  chooseDestination: boolean;
  play: boolean;
  openProvider: boolean;
  localDelivery: boolean;
  close: boolean;
};

export function controlsForState(
  snapshot: CourierDisplaySnapshot,
  payload: TransferPayloadState,
  deliveryWorking = false,
): QuickCopyControls {
  const processing = snapshot.work_phase === "processing" || deliveryWorking;
  const payloadBusy = isTransferPayloadBusy(payload);
  const deliveryReady = !processing
    && snapshot.work_phase === "ready"
    && snapshot.pending_item_count === 0
    && snapshot.delivery_available
    && matchesAvailableHandoff(snapshot.handoff_status);
  return {
    chooseDestination: !processing
      && snapshot.handoff_status !== "in_progress"
      && snapshot.handoff_status !== "completed"
      && !snapshot.delivery_available,
    play: !processing && canExecuteCollection(snapshot),
    openProvider: deliveryReady,
    localDelivery: deliveryReady,
    close: !payloadBusy,
  };
}

export function statusCopy(
  snapshot: CourierDisplaySnapshot,
  payload: TransferPayloadState,
  arriving = false,
  deliveryWorking = false,
  unable = false,
): string {
  const phase = phaseForSnapshot(snapshot, arriving, deliveryWorking, unable);
  if (phase === "ready" && payload.phase === "dragging") return "Trzymaj mocno.";
  if (phase === "completed" && snapshot.handoff_channel === "native_drag") {
    return "Dostarczone pod drzwi";
  }
  if (phase === "completed" && snapshot.handoff_channel === "local_google_drive") {
    return "Gotowe. Zlecenie wysłane do folderu Google Drive.";
  }
  switch (phase) {
    case "arriving": return "";
    case "working": return "";
    case "delivering": return "Już niosę.";
    case "unable": return "Nie mogę dla ciebie tego zrobić.";
    case "ready":
      if (snapshot.handoff_status === "retryable_issue") {
        return "Nie mogę dla ciebie tego zrobić.";
      }
      return snapshot.omitted_asset_count > 0
        ? `Gotowe! Brakuje ${snapshot.omitted_asset_count} plików.`
        : "Gotowe! Co z tym robimy?";
    case "waiting": return "Przenosimy tam gdzie zawsze, kierowniku?";
    case "completed": return "Dostarczone pod drzwi";
  }
}

export function buildAssistantPresentation(
  snapshot: CourierDisplaySnapshot,
  payload: TransferPayloadState,
  options: { arriving: boolean; deliveryWorking: boolean; unable: boolean; hovered: boolean },
): AssistantPresentationModel {
  const phase = phaseForSnapshot(
    snapshot,
    options.arriving,
    options.deliveryWorking,
    options.unable,
  );
  const bubbleAlwaysVisible = phase === "ready"
    || phase === "unable"
    || phase === "delivering"
    || phase === "completed";
  const nativeDragging = payload.phase === "dragging";
  const animationClass = nativeDragging
    ? "quick-worker--payload-released"
    : phase === "completed"
    ? "quick-worker--complete"
    : phase === "ready"
    ? "quick-worker--payload-held"
    : `quick-worker--${phase}`;
  return {
    statusText: statusCopy(
      snapshot,
      payload,
      options.arriving,
      options.deliveryWorking,
      options.unable,
    ),
    destinationDisplayPath: phase === "completed"
      ? null
      : compactPath(snapshot.destination_display_path),
    destinationTitle: phase === "completed" ? null : snapshot.destination_display_path || null,
    animationClass,
    characterAccessibleLabel: "Kurier projektów Ableton",
    windowDragEnabled: payload.phase !== "dragging",
    payloadDragEnabled: payload.phase === "armed" || payload.phase === "dragging",
    bubbleVisible: !options.arriving
      && snapshot.items.length > 0
      && (options.hovered || bubbleAlwaysVisible),
    arrivalActive: options.arriving,
    payloadHitSurfaceVisible: phase === "ready"
      && (payload.phase === "idle" || payload.phase === "arming" || payload.phase === "armed"),
  };
}

export function collectionSummaryLabel(snapshot: CourierDisplaySnapshot): string {
  if (snapshot.items.length === 1) return snapshot.items[0].source_display_name;
  if (snapshot.work_phase === "ready") {
    const packaged = snapshot.completed_item_count + snapshot.incomplete_item_count;
    return `${packaged} z ${snapshot.items.length} projektów`;
  }
  return `${snapshot.items.length} ${projectNoun(snapshot.items.length)}`;
}

function matchesAvailableHandoff(status: CourierDisplaySnapshot["handoff_status"]): boolean {
  return status === "not_started" || status === "retryable_issue";
}

function projectNoun(count: number): string {
  const lastTwo = count % 100;
  const last = count % 10;
  return last >= 2 && last <= 4 && (lastTwo < 12 || lastTwo > 14)
    ? "projekty"
    : "projektów";
}

function compactPath(path: string): string | null {
  if (!path) return null;
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.slice(-2).join("/");
}
