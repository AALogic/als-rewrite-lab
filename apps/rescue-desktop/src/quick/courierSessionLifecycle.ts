import type { CourierDisplaySnapshot } from "./contracts";
import { shouldDiscardSessionOnClose } from "./copyJobState";

export type CourierSessionCloseRequest = {
  snapshot: CourierDisplaySnapshot;
  deliveryWorking: boolean;
  armedPayloadAttemptId: string | null;
};

export type CourierSessionCloseDependencies = {
  cancelPayload: (attemptId: string) => Promise<void>;
  resetCollection: () => Promise<void>;
  closeWindow: () => Promise<void>;
};

export async function closeCourierSession(
  request: CourierSessionCloseRequest,
  dependencies: CourierSessionCloseDependencies,
): Promise<void> {
  if (request.armedPayloadAttemptId) {
    await dependencies.cancelPayload(request.armedPayloadAttemptId);
  }
  if (shouldDiscardSessionOnClose(request.snapshot, request.deliveryWorking)) {
    await dependencies.resetCollection();
  }
  await dependencies.closeWindow();
}
