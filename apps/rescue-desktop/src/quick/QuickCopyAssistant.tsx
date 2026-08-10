import { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { ChevronDown, ChevronUp, CloudUpload, FolderOpen, Play, X } from "lucide-react";
import { AssistantHost } from "../assistant/AssistantHost";
import type {
  CollectionDeliveryResult,
  CourierDisplaySnapshot,
  CourierPreferences,
} from "./contracts";
import { emptyCourierSnapshot } from "./contracts";
import { snapshotRequiresPayloadReset } from "./copyJobState";
import { closeCourierSession } from "./courierSessionLifecycle";
import type { ExternalProviderOpenRequest } from "./externalFolderHandoff";
import { WETRANSFER_PROVIDER_ID } from "./externalFolderHandoff";
import {
  buildAssistantPresentation,
  collectionSummaryLabel,
  controlsForState,
} from "./quickCopyPresenter";
import { footprintForPresentation, shouldApplyHoverChange } from "./quickWindowFootprint";
import {
  initialTransferPayloadState,
  transferPayloadReducer,
} from "./transferPayloadState";
import type {
  FolderDragSurfaceRect,
  UniversalPayloadDragFinished,
  UniversalPayloadDragPrepared,
  UniversalPayloadDragRequest,
  UniversalPayloadDragStarted,
} from "./universalPayloadDrag";
import {
  PAYLOAD_DRAG_FINISHED_EVENT,
  PAYLOAD_DRAG_STARTED_EVENT,
} from "./universalPayloadDrag";
import "./QuickCopyAssistant.css";

const COURIER_SNAPSHOT_EVENT = "courier-snapshot-changed";
const ARRIVAL_DURATION_MS = 1300;

function requestId(prefix: string): string {
  return globalThis.crypto?.randomUUID?.() ?? `${prefix}-${Date.now()}`;
}

export function QuickCopyAssistant() {
  const [snapshot, setSnapshot] = useState<CourierDisplaySnapshot>(emptyCourierSnapshot);
  const [payload, dispatchPayload] = useReducer(
    transferPayloadReducer,
    initialTransferPayloadState,
  );
  const [arriving, setArriving] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [queueExpanded, setQueueExpanded] = useState(false);
  const [deliveryWorking, setDeliveryWorking] = useState(false);
  const [unable, setUnable] = useState(false);
  const characterRef = useRef<HTMLDivElement>(null);
  const payloadRef = useRef<HTMLSpanElement>(null);
  const arrivalShown = useRef(false);
  const armingKey = useRef<string | null>(null);
  const suppressHoverLeave = useRef(false);

  const controls = controlsForState(snapshot, payload, deliveryWorking);
  const presentation = buildAssistantPresentation(snapshot, payload, {
    arriving,
    deliveryWorking,
    unable,
    hovered,
  });
  const windowFootprint = footprintForPresentation(presentation);

  useEffect(() => {
    if (windowFootprint === "expanded") suppressHoverLeave.current = true;
    void invoke("set_quick_window_footprint", { footprint: windowFootprint })
      .catch(() => console.warn("Quick window footprint update failed."))
      .finally(() => {
        window.setTimeout(() => { suppressHoverLeave.current = false; }, 80);
      });
  }, [windowFootprint]);

  const updateHovered = useCallback((next: boolean) => {
    if (!shouldApplyHoverChange(next, suppressHoverLeave.current)) return;
    setHovered(next);
  }, []);

  const acceptSnapshot = useCallback((next: CourierDisplaySnapshot) => {
    setSnapshot(next);
    setUnable(false);
    if (snapshotRequiresPayloadReset(next)) {
      armingKey.current = null;
      dispatchPayload({ type: "reset" });
    }
    if (!arrivalShown.current && next.items.length > 0) {
      arrivalShown.current = true;
      setArriving(true);
      window.setTimeout(() => setArriving(false), ARRIVAL_DURATION_MS);
    }
  }, []);

  useEffect(() => {
    let active = true;
    let stopListening: (() => void) | undefined;
    void listen<CourierDisplaySnapshot>(COURIER_SNAPSHOT_EVENT, (event) => {
      if (active) acceptSnapshot(event.payload);
    }).then((unlisten) => {
      if (!active) {
        unlisten();
        return;
      }
      stopListening = unlisten;
      void invoke<CourierDisplaySnapshot>("get_courier_snapshot")
        .then((current) => { if (active) acceptSnapshot(current); })
        .catch(() => { if (active) setUnable(true); });
    });
    return () => {
      active = false;
      stopListening?.();
    };
  }, [acceptSnapshot]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let active = true;
    void listen<UniversalPayloadDragStarted>(PAYLOAD_DRAG_STARTED_EVENT, (event) => {
      if (active) dispatchPayload({ type: "drag_started", attemptId: event.payload.attempt_id });
    }).then((unlisten) => active ? unlisteners.push(unlisten) : unlisten());
    void listen<UniversalPayloadDragFinished>(PAYLOAD_DRAG_FINISHED_EVENT, (event) => {
      if (!active) return;
      dispatchPayload({ type: "finished", result: event.payload });
      if (event.payload.outcome !== "dropped") armingKey.current = null;
    }).then((unlisten) => active ? unlisteners.push(unlisten) : unlisten());
    return () => {
      active = false;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  const armPayload = useCallback(async () => {
    if (!snapshot.delivery_available || !snapshot.collection_id || payload.phase !== "idle") return;
    const bounds = payloadRef.current?.getBoundingClientRect();
    if (!bounds || bounds.width <= 0 || bounds.height <= 0) return;
    const key = `${snapshot.collection_id}:${snapshot.revision}`;
    if (armingKey.current === key) return;
    armingKey.current = key;
    const surfaceRect: FolderDragSurfaceRect = {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    };
    const request: UniversalPayloadDragRequest = {
      request_id: requestId("payload-drag"),
      collection_id: snapshot.collection_id,
      collection_revision: snapshot.revision,
    };
    dispatchPayload({ type: "arm_started" });
    try {
      const attempt = await invoke<UniversalPayloadDragPrepared>(
        "prepare_universal_payload_drag",
        { request, surface_rect: surfaceRect },
      );
      dispatchPayload({ type: "prepared", attempt });
    } catch {
      armingKey.current = null;
      dispatchPayload({ type: "arm_failed" });
    }
  }, [payload.phase, snapshot.collection_id, snapshot.delivery_available, snapshot.revision]);

  useEffect(() => {
    if (!arriving && presentation.payloadHitSurfaceVisible && payload.phase === "idle") {
      const timer = window.setTimeout(() => void armPayload(), 50);
      return () => window.clearTimeout(timer);
    }
  }, [armPayload, arriving, payload.phase, presentation.payloadHitSurfaceVisible]);

  async function chooseDestination() {
    if (!controls.chooseDestination) return;
    const selected = await open({ multiple: false, directory: true });
    if (typeof selected !== "string") return;
    try {
      const next = await invoke<CourierDisplaySnapshot>("set_courier_destination", {
        destination_parent: selected,
      });
      acceptSnapshot(next);
    } catch {
      setUnable(true);
    }
  }

  async function startProcessing() {
    if (!controls.play) return;
    try {
      const next = await invoke<CourierDisplaySnapshot>("start_courier_processing");
      acceptSnapshot(next);
    } catch {
      setUnable(true);
    }
  }

  async function removeItem(workItemId: string) {
    try {
      const next = await invoke<CourierDisplaySnapshot>("remove_courier_item", {
        work_item_id: workItemId,
      });
      acceptSnapshot(next);
    } catch {
      setUnable(true);
    }
  }

  async function openProvider() {
    if (!controls.openProvider) return;
    const request: ExternalProviderOpenRequest = {
      request_id: requestId("provider-open"),
      provider_id: WETRANSFER_PROVIDER_ID,
    };
    try {
      await invoke("open_external_provider", { request });
    } catch {
      setUnable(true);
    }
  }

  async function deliverLocally() {
    if (!controls.localDelivery) return;
    setDeliveryWorking(true);
    try {
      let preferences = await invoke<CourierPreferences>("get_courier_preferences");
      if (!preferences.local_delivery_root) {
        const selected = await open({ multiple: false, directory: true });
        if (typeof selected !== "string") return;
        preferences = await invoke<CourierPreferences>("set_courier_delivery_root", {
          local_delivery_root: selected,
        });
      }
      const result = await invoke<CollectionDeliveryResult>("deliver_courier_collection");
      const current = await invoke<CourierDisplaySnapshot>("get_courier_snapshot");
      acceptSnapshot({ ...current, local_delivery_configured: Boolean(preferences.local_delivery_root) });
      if (result.run_status !== "completed") setUnable(true);
    } catch {
      setUnable(true);
    } finally {
      setDeliveryWorking(false);
    }
  }

  async function startNewOrder() {
    if (snapshot.work_phase === "processing") return;
    try {
      dispatchPayload({ type: "reset" });
      armingKey.current = null;
      const next = await invoke<CourierDisplaySnapshot>("reset_courier_collection");
      setQueueExpanded(false);
      acceptSnapshot(next);
    } catch {
      setUnable(true);
    }
  }

  async function resendPayload() {
    armingKey.current = null;
    setUnable(false);
    try {
      dispatchPayload({ type: "reset" });
      const next = await invoke<CourierDisplaySnapshot>("reopen_courier_handoff");
      acceptSnapshot(next);
    } catch {
      setUnable(true);
    }
  }

  async function closeWindow() {
    if (!controls.close) return;
    try {
      await closeCourierSession({
        snapshot,
        deliveryWorking,
        armedPayloadAttemptId: payload.phase === "armed" && payload.attempt
          ? payload.attempt.attempt_id
          : null,
      }, {
        cancelPayload: async (attemptId) => {
          await invoke("cancel_universal_payload_drag", { attempt_id: attemptId });
        },
        resetCollection: async () => { await invoke("reset_courier_collection"); },
        closeWindow: async () => { await getCurrentWindow().close(); },
      });
    } catch {
      setUnable(true);
    }
  }

  const visibleItems = useMemo(
    () => snapshot.items.filter((item) => item.status !== "failed" || snapshot.failed_item_count > 0),
    [snapshot.failed_item_count, snapshot.items],
  );

  const bubbleDetails = snapshot.items.length > 0 ? (
    <div className="quick-queue-summary">
      <button
        type="button"
        className="quick-queue-toggle"
        disabled={snapshot.items.length < 2}
        onClick={() => setQueueExpanded((value) => !value)}
        aria-expanded={queueExpanded}
      >
        <span title={snapshot.items.length === 1 ? snapshot.items[0].source_display_name : undefined}>
          {collectionSummaryLabel(snapshot)}
        </span>
        {snapshot.items.length >= 2
          ? queueExpanded ? <ChevronUp aria-hidden="true" size={14} /> : <ChevronDown aria-hidden="true" size={14} />
          : null}
      </button>
      {queueExpanded && snapshot.items.length >= 2 ? (
        <ul className="quick-queue-list">
          {visibleItems.map((item) => (
            <li key={item.work_item_id}>
              <span title={item.source_display_name}>{item.source_display_name}</span>
              {item.status === "queued" ? (
                <button type="button" onClick={() => void removeItem(item.work_item_id)}
                  aria-label={`Usuń ${item.source_display_name}`} title="Usuń z kolejki">
                  <X aria-hidden="true" size={13} />
                </button>
              ) : <small>{statusMark(item.status)}</small>}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  ) : null;

  const renderedControls = (
    <>
      {controls.chooseDestination ? (
        <button type="button" className="quick-icon-button quick-stack-button"
          onClick={() => void chooseDestination()} aria-label="Zmień folder docelowy" title="Zmień folder docelowy">
          <FolderOpen aria-hidden="true" size={20} />
        </button>
      ) : null}
      {controls.play ? (
        <button type="button" className="quick-icon-button quick-icon-button--primary quick-stack-button"
          onClick={() => void startProcessing()} aria-label="Rozpocznij pracę" title="Rozpocznij pracę">
          <Play aria-hidden="true" size={21} fill="currentColor" />
        </button>
      ) : null}
      {controls.openProvider ? (
        <button type="button" className="quick-icon-button quick-transfer-button"
          onClick={() => void openProvider()} aria-label="Otwórz WeTransfer" title="Otwórz WeTransfer">
          <span className="quick-transfer-mark" aria-hidden="true">W</span>
        </button>
      ) : null}
      {controls.localDelivery ? (
        <button type="button" className="quick-icon-button quick-drive-button"
          onClick={() => void deliverLocally()} aria-label="Skopiuj do folderu dostawy" title="Skopiuj do folderu dostawy">
          <CloudUpload aria-hidden="true" size={20} />
        </button>
      ) : null}
    </>
  );

  return (
    <AssistantHost
      model={presentation}
      closeEnabled={controls.close}
      resendEnabled={snapshot.handoff_status === "completed"}
      onClose={() => void closeWindow()}
      onNewOrder={() => void startNewOrder()}
      onResendPayload={() => void resendPayload()}
      onHoverChange={updateHovered}
      characterRef={characterRef}
      payloadRef={payloadRef}
      bubbleDetails={bubbleDetails}
      controls={renderedControls}
    />
  );
}

function statusMark(status: CourierDisplaySnapshot["items"][number]["status"]): string {
  switch (status) {
    case "processing": return "w toku";
    case "completed": return "gotowe";
    case "completed_incomplete": return "niepełne";
    case "blocked": return "pominięte";
    case "failed": return "błąd";
    case "queued": return "kolejka";
  }
}
