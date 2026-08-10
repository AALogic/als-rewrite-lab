import type { AssistantPresentationModel } from "../assistant/contracts";

export type QuickWindowFootprint = "compact" | "expanded";

export function footprintForPresentation(
  presentation: Pick<AssistantPresentationModel, "bubbleVisible">,
): QuickWindowFootprint {
  return presentation.bubbleVisible ? "expanded" : "compact";
}

export function shouldApplyHoverChange(next: boolean, suppressLeave: boolean): boolean {
  return next || !suppressLeave;
}
