import type {
  UniversalPayloadDragFinished,
  UniversalPayloadDragPrepared,
} from "./universalPayloadDrag";

export type TransferPayloadPhase = "idle" | "arming" | "armed" | "dragging" | "delivered";

export type TransferPayloadState = {
  phase: TransferPayloadPhase;
  attempt: UniversalPayloadDragPrepared | null;
};

export type TransferPayloadEvent =
  | { type: "arm_started" }
  | { type: "prepared"; attempt: UniversalPayloadDragPrepared }
  | { type: "arm_failed" }
  | { type: "drag_started"; attemptId: string }
  | { type: "finished"; result: UniversalPayloadDragFinished }
  | { type: "reload" }
  | { type: "reset" };

export const initialTransferPayloadState: TransferPayloadState = {
  phase: "idle",
  attempt: null,
};

export function transferPayloadReducer(
  state: TransferPayloadState,
  event: TransferPayloadEvent,
): TransferPayloadState {
  switch (event.type) {
    case "arm_started":
      return state.phase === "idle" ? { phase: "arming", attempt: null } : state;
    case "prepared":
      return state.phase === "arming" ? { phase: "armed", attempt: event.attempt } : state;
    case "arm_failed":
      return state.phase === "arming" ? initialTransferPayloadState : state;
    case "drag_started":
      return state.phase === "armed" && event.attemptId === state.attempt?.attempt_id
        ? { ...state, phase: "dragging" }
        : state;
    case "finished":
      if (!state.attempt || event.result.attempt_id !== state.attempt.attempt_id) return state;
      return event.result.outcome === "dropped"
        ? { phase: "delivered", attempt: null }
        : initialTransferPayloadState;
    case "reload":
      return state.phase === "delivered" ? initialTransferPayloadState : state;
    case "reset":
      return initialTransferPayloadState;
  }
}

export function isTransferPayloadBusy(state: TransferPayloadState): boolean {
  return state.phase === "arming" || state.phase === "dragging";
}
