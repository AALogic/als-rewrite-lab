export type AssistantPresentationModel = {
  statusText: string;
  destinationDisplayPath: string | null;
  destinationTitle: string | null;
  animationClass: string;
  characterAccessibleLabel: string;
  windowDragEnabled: boolean;
  payloadDragEnabled: boolean;
  bubbleVisible: boolean;
  arrivalActive: boolean;
  payloadHitSurfaceVisible: boolean;
};
