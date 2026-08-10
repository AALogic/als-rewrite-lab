import { describe, expect, test } from "vitest";
import type { CourierDisplaySnapshot } from "./contracts";
import { emptyCourierSnapshot } from "./contracts";
import { closeCourierSession } from "./courierSessionLifecycle";
import {
  canExecuteCollection,
  phaseForSnapshot,
  shouldDiscardSessionOnClose,
  snapshotRequiresPayloadReset,
} from "./copyJobState";
import {
  buildAssistantPresentation,
  collectionSummaryLabel,
  controlsForState,
  statusCopy,
} from "./quickCopyPresenter";
import {
  initialTransferPayloadState,
  transferPayloadReducer,
} from "./transferPayloadState";
import type {
  UniversalPayloadDragFinished,
  UniversalPayloadDragPrepared,
} from "./universalPayloadDrag";

function snapshot(overrides: Partial<CourierDisplaySnapshot> = {}): CourierDisplaySnapshot {
  return {
    ...emptyCourierSnapshot,
    collection_id: "collection-1",
    revision: 3,
    work_phase: "collecting",
    destination_display_path: "/Rescued",
    items: [{ work_item_id: "item-1", source_display_name: "Set.als", status: "queued" }],
    pending_item_count: 1,
    ...overrides,
  };
}

const attempt: UniversalPayloadDragPrepared = {
  schema_version: "0.1",
  attempt_id: "attempt-1",
  state: "armed",
};

function finished(outcome: UniversalPayloadDragFinished["outcome"]): UniversalPayloadDragFinished {
  return {
    schema_version: "0.1",
    attempt_id: attempt.attempt_id,
    outcome,
    error_code: outcome === "failed" ? "PAYLOAD_NATIVE_DRAG_FAILED" : null,
  };
}

function armedPayload() {
  const arming = transferPayloadReducer(initialTransferPayloadState, { type: "arm_started" });
  return transferPayloadReducer(arming, { type: "prepared", attempt });
}

describe("QuickCopyAssistant state contract", () => {
  test("unable_preview_never_executes_copy", () => {
    const blocked = snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      failed_item_count: 1,
      delivery_available: false,
    });
    expect(canExecuteCollection(blocked)).toBe(false);
    expect(controlsForState(blocked, initialTransferPayloadState).play).toBe(false);
  });

  test("one_play_executes_incomplete_preview_without_second_prompt", () => {
    const queued = snapshot();
    expect(canExecuteCollection(queued)).toBe(true);
    const processing = snapshot({ work_phase: "processing" });
    expect(canExecuteCollection(processing)).toBe(false);
    const incomplete = snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      completed_item_count: 1,
      incomplete_item_count: 1,
      omitted_asset_count: 4,
      delivery_available: true,
      items: [{ work_item_id: "item-1", source_display_name: "Set.als", status: "completed_incomplete" }],
    });
    expect(statusCopy(incomplete, initialTransferPayloadState)).toBe("Gotowe! Brakuje 4 plików.");
    expect(controlsForState(incomplete, initialTransferPayloadState).localDelivery).toBe(true);
  });

  test("quick_state_controls_follow_contract", () => {
    expect(controlsForState(emptyCourierSnapshot, initialTransferPayloadState)).toEqual({
      chooseDestination: true,
      play: false,
      openProvider: false,
      localDelivery: false,
      close: true,
    });
    expect(controlsForState(snapshot(), initialTransferPayloadState).play).toBe(true);
    const processing = snapshot({ work_phase: "processing" });
    expect(controlsForState(processing, initialTransferPayloadState).play).toBe(false);
    const ready = snapshot({ work_phase: "ready", pending_item_count: 0, delivery_available: true });
    expect(controlsForState(ready, initialTransferPayloadState).openProvider).toBe(true);
    expect(controlsForState(ready, armedPayload()).close).toBe(true);
  });

  test("quick_status_copy_is_deterministic", () => {
    expect(statusCopy(snapshot(), initialTransferPayloadState))
      .toBe("Przenosimy tam gdzie zawsze, kierowniku?");
    const ready = snapshot({ work_phase: "ready", pending_item_count: 0, delivery_available: true });
    expect(statusCopy(ready, initialTransferPayloadState)).toBe("Gotowe! Co z tym robimy?");
    expect(statusCopy(ready, initialTransferPayloadState)).toBe("Gotowe! Co z tym robimy?");
    expect(statusCopy(ready, initialTransferPayloadState, false, false, true))
      .toBe("Nie mogę dla ciebie tego zrobić.");
  });

  test("ready_queue_summary_counts_packaged_projects_against_requested_projects", () => {
    const items = [
      { work_item_id: "1", source_display_name: "A.als", status: "blocked" as const },
      { work_item_id: "2", source_display_name: "B.als", status: "blocked" as const },
      { work_item_id: "3", source_display_name: "C.als", status: "completed" as const },
      { work_item_id: "4", source_display_name: "D.als", status: "completed" as const },
    ];
    const requested = snapshot({ items, pending_item_count: 4 });
    const ready = snapshot({
      items,
      work_phase: "ready",
      pending_item_count: 0,
      completed_item_count: 2,
      failed_item_count: 2,
      delivery_available: true,
    });
    expect(collectionSummaryLabel(requested)).toBe("4 projekty");
    expect(collectionSummaryLabel(ready)).toBe("2 z 4 projektów");
  });

  test("new_intake_overrides_a_stale_delivered_presentation_state", () => {
    const collectingAgain = snapshot({
      collection_id: "collection-2",
      revision: 0,
      completed_item_count: 0,
      pending_item_count: 1,
      items: [{ work_item_id: "new", source_display_name: "New.als", status: "queued" }],
    });
    expect(canExecuteCollection(collectingAgain)).toBe(true);
    expect(phaseForSnapshot(collectingAgain, false, false, false)).toBe("waiting");
    expect(statusCopy(collectingAgain, { phase: "delivered", attempt: null }))
      .toBe("Przenosimy tam gdzie zawsze, kierowniku?");
    expect(snapshotRequiresPayloadReset(collectingAgain)).toBe(true);
    expect(controlsForState(collectingAgain, initialTransferPayloadState).play).toBe(true);
  });

  test("closing_discards_idle_session_but_preserves_active_work", () => {
    expect(shouldDiscardSessionOnClose(snapshot())).toBe(true);
    expect(shouldDiscardSessionOnClose(snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      delivery_available: true,
    }))).toBe(true);
    expect(shouldDiscardSessionOnClose(snapshot({ work_phase: "processing" }))).toBe(false);
    expect(shouldDiscardSessionOnClose(snapshot({
      handoff_status: "in_progress",
      handoff_channel: "local_google_drive",
    }))).toBe(false);
    expect(shouldDiscardSessionOnClose(snapshot(), true)).toBe(false);
  });

  test("closing_idle_courier_resets_collection_before_window_close", async () => {
    const calls: string[] = [];
    await closeCourierSession({
      snapshot: snapshot(),
      deliveryWorking: false,
      armedPayloadAttemptId: "attempt-1",
    }, {
      cancelPayload: async (attemptId) => { calls.push(`cancel:${attemptId}`); },
      resetCollection: async () => { calls.push("reset"); },
      closeWindow: async () => { calls.push("close"); },
    });
    expect(calls).toEqual(["cancel:attempt-1", "reset", "close"]);
  });

  test("closing_active_courier_preserves_work_before_window_close", async () => {
    const calls: string[] = [];
    await closeCourierSession({
      snapshot: snapshot({ work_phase: "processing" }),
      deliveryWorking: false,
      armedPayloadAttemptId: null,
    }, {
      cancelPayload: async () => { calls.push("cancel"); },
      resetCollection: async () => { calls.push("reset"); },
      closeWindow: async () => { calls.push("close"); },
    });
    expect(calls).toEqual(["close"]);
  });

  test("ready_character_and_parcel_share_one_sprite", () => {
    const ready = snapshot({ work_phase: "ready", pending_item_count: 0, delivery_available: true });
    const model = buildAssistantPresentation(ready, armedPayload(), {
      arriving: false,
      deliveryWorking: false,
      unable: false,
      hovered: false,
    });
    expect(model.windowDragEnabled).toBe(true);
    expect(model.payloadDragEnabled).toBe(true);
    expect(model.animationClass).toBe("quick-worker--payload-held");
    expect(model.payloadHitSurfaceVisible).toBe(true);
  });

  test("native_drag_switches_to_empty_hands_without_duplicate_parcel", () => {
    const ready = snapshot({ work_phase: "ready", pending_item_count: 0, delivery_available: true });
    const dragging = transferPayloadReducer(armedPayload(), {
      type: "drag_started",
      attemptId: attempt.attempt_id,
    });
    const model = buildAssistantPresentation(ready, dragging, {
      arriving: false,
      deliveryWorking: false,
      unable: false,
      hovered: false,
    });
    expect(model.animationClass).toBe("quick-worker--payload-released");
    expect(model.payloadHitSurfaceVisible).toBe(false);
    expect(model.windowDragEnabled).toBe(false);
  });

  test("cancelled_and_failed_drag_restore_integrated_parcel", () => {
    for (const outcome of ["cancelled", "failed"] as const) {
      const armed = armedPayload();
      const dragging = transferPayloadReducer(armed, {
        type: "drag_started",
        attemptId: attempt.attempt_id,
      });
      const restored = transferPayloadReducer(dragging, {
        type: "finished",
        result: finished(outcome),
      });
      expect(restored).toEqual(initialTransferPayloadState);
      const model = buildAssistantPresentation(
        snapshot({ work_phase: "ready", pending_item_count: 0, delivery_available: true }),
        restored,
        { arriving: false, deliveryWorking: false, unable: false, hovered: false },
      );
      expect(model.animationClass).toBe("quick-worker--payload-held");
      expect(model.payloadHitSurfaceVisible).toBe(true);
    }
  });

  test("accepted_native_drop_requires_explicit_reload", () => {
    const dragging = transferPayloadReducer(armedPayload(), {
      type: "drag_started",
      attemptId: attempt.attempt_id,
    });
    const delivered = transferPayloadReducer(dragging, {
      type: "finished",
      result: finished("dropped"),
    });
    expect(delivered.phase).toBe("delivered");
    expect(transferPayloadReducer(delivered, { type: "arm_started" })).toBe(delivered);
    expect(transferPayloadReducer(delivered, { type: "reload" }))
      .toEqual(initialTransferPayloadState);
  });

  test("completed_handoff_messages_match_the_delivery_channel", () => {
    const native = snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      delivery_available: false,
      handoff_status: "completed",
      handoff_channel: "native_drag",
    });
    const drive = snapshot({
      ...native,
      handoff_channel: "local_google_drive",
    });
    expect(statusCopy(native, initialTransferPayloadState)).toBe("Dostarczone pod drzwi");
    expect(statusCopy(drive, initialTransferPayloadState))
      .toBe("Gotowe. Zlecenie wysłane do folderu Google Drive.");
  });

  test("completed_handoff_hides_parcel_and_delivery_controls", () => {
    const completed = snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      delivery_available: false,
      handoff_status: "completed",
      handoff_channel: "native_drag",
    });
    const model = buildAssistantPresentation(completed, initialTransferPayloadState, {
      arriving: false,
      deliveryWorking: false,
      unable: false,
      hovered: false,
    });
    expect(model.payloadHitSurfaceVisible).toBe(false);
    expect(model.bubbleVisible).toBe(true);
    expect(controlsForState(completed, initialTransferPayloadState)).toEqual({
      chooseDestination: false,
      play: false,
      openProvider: false,
      localDelivery: false,
      close: true,
    });
  });

  test("retryable_handoff_keeps_the_parcel_and_delivery_controls", () => {
    const retryable = snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      delivery_available: true,
      handoff_status: "retryable_issue",
      handoff_channel: "local_google_drive",
    });
    const model = buildAssistantPresentation(retryable, initialTransferPayloadState, {
      arriving: false,
      deliveryWorking: false,
      unable: false,
      hovered: false,
    });
    expect(statusCopy(retryable, initialTransferPayloadState))
      .toBe("Nie mogę dla ciebie tego zrobić.");
    expect(model.animationClass).toBe("quick-worker--payload-held");
    expect(model.payloadHitSurfaceVisible).toBe(true);
    expect(controlsForState(retryable, initialTransferPayloadState).localDelivery).toBe(true);
  });

  test("native_handoff_keeps_attempt_until_terminal_event", () => {
    const dragging = snapshot({
      work_phase: "ready",
      pending_item_count: 0,
      delivery_available: false,
      handoff_status: "in_progress",
      handoff_channel: "native_drag",
    });
    const local = snapshot({ ...dragging, handoff_channel: "local_google_drive" });
    expect(snapshotRequiresPayloadReset(dragging)).toBe(false);
    expect(snapshotRequiresPayloadReset(local)).toBe(true);
  });

  test("ready_delivery_controls_do_not_repeat_the_destination_folder_action", () => {
    const ready = snapshot({ work_phase: "ready", pending_item_count: 0, delivery_available: true });
    expect(controlsForState(ready, initialTransferPayloadState)).toEqual({
      chooseDestination: false,
      play: false,
      openProvider: true,
      localDelivery: true,
      close: true,
    });
  });

  test("native_drag_event_must_match_the_armed_attempt", () => {
    const armed = armedPayload();
    expect(transferPayloadReducer(armed, { type: "drag_started", attemptId: "stale" }))
      .toBe(armed);
    expect(transferPayloadReducer(armed, {
      type: "drag_started",
      attemptId: attempt.attempt_id,
    }).phase).toBe("dragging");
  });

  test("arrival_and_working_phases_hide_status_copy", () => {
    expect(phaseForSnapshot(snapshot(), true, false, false)).toBe("arriving");
    expect(statusCopy(snapshot(), initialTransferPayloadState, true)).toBe("");
    expect(statusCopy(snapshot({ work_phase: "processing" }), initialTransferPayloadState)).toBe("");
  });
});
