import { createRef } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, test } from "vitest";
import {
  AssistantHost,
  COURIER_CHARACTER_FRAME,
  PAYLOAD_HELD_SPRITE_STYLE,
  RESEND_ACTION_LABEL,
} from "./AssistantHost";
import type { AssistantPresentationModel } from "./contracts";

function model(overrides: Partial<AssistantPresentationModel> = {}): AssistantPresentationModel {
  return {
    statusText: "Gotowe! Co z tym robimy?",
    destinationDisplayPath: "Rescued/Set Rescue Project",
    destinationTitle: "/Rescued/Set Rescue Project",
    animationClass: "quick-worker--payload-held",
    characterAccessibleLabel: "Kurier projektów Ableton",
    windowDragEnabled: true,
    payloadDragEnabled: true,
    bubbleVisible: true,
    arrivalActive: false,
    payloadHitSurfaceVisible: true,
    ...overrides,
  };
}

function renderHost(presentation = model(), controls = <button type="button">Akcja</button>) {
  return renderToStaticMarkup(
    <AssistantHost
      model={presentation}
      closeEnabled
      resendEnabled
      onClose={() => undefined}
      onNewOrder={() => undefined}
      onResendPayload={() => undefined}
      onHoverChange={() => undefined}
      characterRef={createRef()}
      payloadRef={createRef()}
      bubbleDetails={<span>2 projekty</span>}
      controls={controls}
    />,
  );
}

describe("AssistantHost presentation boundary", () => {
  test("assistant_host_renders_supplied_status_without_domain_interpretation", () => {
    const html = renderHost(model({ statusText: "Własny status." }));
    expect(html).toContain("Własny status.");
    expect(html).toContain("2 projekty");
    expect(html).not.toContain("prepare_copy");
  });

  test("assistant_host_window_drag_region_follows_model", () => {
    expect(renderHost()).toContain("data-tauri-drag-region=\"true\"");
    expect(renderHost(model({ windowDragEnabled: false })))
      .not.toContain("data-tauri-drag-region=\"true\"");
  });

  test("assistant_host_keeps_payload_and_window_drag_geometry_separate", () => {
    const html = renderHost(model({ payloadDragEnabled: true, windowDragEnabled: false }));
    expect(html).toContain("data-assistant-character-surface=\"true\"");
    expect(html).toContain("data-assistant-payload-surface=\"true\"");
    expect(html).toContain("data-payload-drag-enabled=\"true\"");
  });

  test("assistant_host_hides_bubble_and_payload_when_presentation_requests_it", () => {
    const html = renderHost(model({ bubbleVisible: false, payloadHitSurfaceVisible: false }));
    expect(html).not.toContain("quick-copy-bubble");
    expect(html).not.toContain("data-assistant-payload-surface");
  });

  test("assistant_host_exposes_supplied_controls_without_owning_actions", () => {
    const html = renderHost(model(), <button type="button">Kontrola testowa</button>);
    expect(html).toContain("Kontrola testowa");
    expect(html).not.toContain("quick-copy-close");
  });

  test("assistant_host_names_completed_retry_as_send_again", () => {
    expect(RESEND_ACTION_LABEL).toBe("Wyślij ponownie");
  });

  test("ready_character_and_parcel_share_one_sprite", () => {
    const html = renderHost(model({ animationClass: "quick-worker--payload-held" }));
    expect(html).toContain("quick-worker--payload-held");
    expect(html).toContain("data-assistant-payload-surface=\"true\"");
    expect(html).toContain("data-payload-visual=\"native-drag-only\"");
  });

  test("courier_canvas_scale_remains_stable_across_payload_states", () => {
    expect(COURIER_CHARACTER_FRAME).toEqual({ width: 96, height: 128 });
    const held = renderHost(model({ animationClass: "quick-worker--payload-held" }));
    const released = renderHost(model({ animationClass: "quick-worker--payload-released" }));
    for (const html of [held, released]) {
      expect(html).toContain("width:96px");
      expect(html).toContain("height:128px");
    }
  });

  test("payload_held_sprite_normalizes_silhouette_without_resizing_character_frame", () => {
    expect(PAYLOAD_HELD_SPRITE_STYLE).toEqual({
      transform: "translateY(4px) scale(1.18)",
      transformOrigin: "50% 100%",
    });
    const held = renderHost(model({ animationClass: "quick-worker--payload-held" }));
    expect(held).toContain("overflow:hidden");
    expect(held).toContain("transform:translateY(4px) scale(1.18)");
  });
});
