import { useState } from "react";
import type { ReactNode, RefObject } from "react";
import { RotateCcw, SquarePlus, X } from "lucide-react";
import type { AssistantPresentationModel } from "./contracts";

export const RESEND_ACTION_LABEL = "Wyślij ponownie";
export const COURIER_CHARACTER_FRAME = { width: 96, height: 128 } as const;
export const PAYLOAD_HELD_SPRITE_STYLE = {
  transform: "translateY(4px) scale(1.18)",
  transformOrigin: "50% 100%",
} as const;

type AssistantHostProps = {
  model: AssistantPresentationModel;
  closeEnabled: boolean;
  resendEnabled: boolean;
  onClose: () => void;
  onNewOrder: () => void;
  onResendPayload: () => void;
  onHoverChange: (hovered: boolean) => void;
  characterRef: RefObject<HTMLDivElement | null>;
  payloadRef: RefObject<HTMLSpanElement | null>;
  bubbleDetails: ReactNode;
  controls: ReactNode;
};

export function AssistantHost({
  model,
  closeEnabled,
  resendEnabled,
  onClose,
  onNewOrder,
  onResendPayload,
  onHoverChange,
  characterRef,
  payloadRef,
  bubbleDetails,
  controls,
}: AssistantHostProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const dragRegion = model.windowDragEnabled
    ? { "data-tauri-drag-region": "true" }
    : {};
  const payloadHeld = model.animationClass === "quick-worker--payload-held";

  function showMenu(event: React.MouseEvent) {
    event.preventDefault();
    setMenuOpen(true);
  }

  return (
    <main
      className="quick-copy-shell"
      onPointerDown={() => setMenuOpen(false)}
      onPointerLeave={() => onHoverChange(false)}
    >
      {model.arrivalActive ? (
        <div className="quick-arrival" aria-hidden="true">
          <span className="quick-arrival-van" />
        </div>
      ) : null}

      <section className="quick-copy-stage" aria-live="polite">
        {model.bubbleVisible ? (
          <div className="quick-copy-bubble">
            <strong>{model.statusText}</strong>
            {bubbleDetails}
            {model.destinationDisplayPath ? (
              <span className="quick-destination" title={model.destinationTitle ?? undefined}>
                {model.destinationDisplayPath}
              </span>
            ) : null}
          </div>
        ) : null}

        <div className="quick-worker-area">
          <div
            ref={characterRef}
            className={`quick-worker ${model.animationClass}`}
            data-assistant-character-surface="true"
            aria-label={model.characterAccessibleLabel}
            role="img"
            style={{
              width: `${COURIER_CHARACTER_FRAME.width}px`,
              height: `${COURIER_CHARACTER_FRAME.height}px`,
              overflow: payloadHeld ? "hidden" : "visible",
            }}
            onMouseEnter={() => onHoverChange(true)}
            onContextMenu={showMenu}
            onDoubleClick={showMenu}
            {...dragRegion}
          >
            <span
              className="quick-worker-sprite"
              style={payloadHeld ? PAYLOAD_HELD_SPRITE_STYLE : undefined}
              {...dragRegion}
            />
          </div>

          {model.payloadHitSurfaceVisible ? (
            <span
              ref={payloadRef}
              className={`quick-payload-surface quick-payload-surface--${model.payloadDragEnabled ? "armed" : "idle"}`}
              data-assistant-payload-surface="true"
              data-payload-drag-enabled={model.payloadDragEnabled ? "true" : "false"}
              data-payload-visual="native-drag-only"
              aria-label="Paczka gotowych projektów"
              role="img"
            />
          ) : null}

          {menuOpen ? (
            <div className="quick-context-menu" role="menu" onPointerDown={(event) => event.stopPropagation()}>
              <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); onNewOrder(); }}>
                <SquarePlus aria-hidden="true" size={15} />
                Nowe zlecenie
              </button>
              {resendEnabled ? (
                <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); onResendPayload(); }}>
                  <RotateCcw aria-hidden="true" size={15} />
                  {RESEND_ACTION_LABEL}
                </button>
              ) : null}
              <button type="button" role="menuitem" disabled={!closeEnabled}
                onClick={() => { setMenuOpen(false); onClose(); }}>
                <X aria-hidden="true" size={15} />
                Zamknij kuriera
              </button>
            </div>
          ) : null}
        </div>
      </section>

      {!model.arrivalActive ? (
        <nav className="quick-copy-controls" aria-label="Działania kuriera">
          {controls}
        </nav>
      ) : null}
    </main>
  );
}
