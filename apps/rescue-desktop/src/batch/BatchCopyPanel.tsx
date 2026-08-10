import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import {
  AlertTriangle,
  Check,
  Clipboard,
  ExternalLink,
  FolderOutput,
  LoaderCircle,
  ShieldCheck,
  Square,
  X,
} from "lucide-react";
import type { ProjectSelection } from "../catalog/contracts";
import type {
  BatchCopyPreview,
  BatchCopyResult,
  BatchProgressEvent,
} from "./contracts";

type Props = {
  selections: ProjectSelection[];
  experimentalCompatibilityAvailable: boolean;
  onBusyChange: (stage: "batch_preview" | "batch_copy" | null) => void;
  onError: (message: string | null) => void;
  onClose: () => void;
};

function fileName(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

function statusLabel(status: string) {
  const labels: Record<string, string> = {
    ready: "Gotowy",
    blocked: "Zablokowany",
    target_collision: "Kolizja nazwy",
    completed: "Kopia gotowa",
    completed_incomplete: "Kopia niepełna",
    failed: "Błąd",
    skipped_blocked: "Pominięty",
    cancelled: "Anulowany",
  };
  return labels[status] ?? status;
}

function messageFromError(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message) return message;
  }
  return fallback;
}

function openResultFolder(path: string | null | undefined) {
  if (path) void openPath(path);
}

export function BatchCopyPanel({
  selections,
  experimentalCompatibilityAvailable,
  onBusyChange,
  onError,
  onClose,
}: Props) {
  const [preview, setPreview] = useState<BatchCopyPreview | null>(null);
  const [result, setResult] = useState<BatchCopyResult | null>(null);
  const [progress, setProgress] = useState<BatchProgressEvent | null>(null);
  const [activeRequestId, setActiveRequestId] = useState<string | null>(null);
  const activeRequestIdRef = useRef<string | null>(null);
  const [experimentalConsent, setExperimentalConsent] = useState(false);
  const [reportCopied, setReportCopied] = useState(false);
  const busy = activeRequestId !== null;

  useEffect(() => {
    let active = true;
    let stopListening: (() => void) | undefined;
    void listen<BatchProgressEvent>("batch-copy-progress", (event) => {
      if (event.payload.request_id === activeRequestIdRef.current) setProgress(event.payload);
    }).then((unlisten) => {
      if (active) stopListening = unlisten;
      else unlisten();
    });
    return () => {
      active = false;
      stopListening?.();
    };
  }, []);

  function setActiveRequest(requestId: string | null) {
    activeRequestIdRef.current = requestId;
    setActiveRequestId(requestId);
  }

  async function prepareBatch() {
    onError(null);
    try {
      const destination = await open({ multiple: false, directory: true });
      if (typeof destination !== "string") return;
      const requestId = globalThis.crypto?.randomUUID?.() ?? `batch-preview-${Date.now()}`;
      setActiveRequest(requestId);
      onBusyChange("batch_preview");
      const next = await invoke<BatchCopyPreview>("prepare_batch_copy", {
        request: {
          request_id: requestId,
          selections,
          destination_parent: destination,
          experimental_compatibility_consent: experimentalConsent,
        },
      });
      setPreview(next);
      setResult(null);
      setReportCopied(false);
      if (next.errors.length) onError(next.errors[0]?.message ?? "Nie udało się przygotować projektów.");
    } catch (error) {
      onError(messageFromError(error, "Nie udało się przygotować projektów."));
    } finally {
      setActiveRequest(null);
      onBusyChange(null);
    }
  }

  async function executeBatch() {
    if (!preview || preview.summary.ready_job_count === 0) return;
    const requestId = globalThis.crypto?.randomUUID?.() ?? `batch-execute-${Date.now()}`;
    setActiveRequest(requestId);
    setProgress(null);
    setReportCopied(false);
    onError(null);
    onBusyChange("batch_copy");
    try {
      const next = await invoke<BatchCopyResult>("execute_batch_copy", {
        request: { request_id: requestId, preview, write_consent: true },
      });
      setResult(next);
      if (next.errors.length) onError(next.errors[0]?.message ?? "Kopie nie zostały utworzone.");
    } catch (error) {
      onError(messageFromError(error, "Kopie nie zostały utworzone."));
    } finally {
      setActiveRequest(null);
      onBusyChange(null);
    }
  }

  async function cancelRemaining() {
    if (!activeRequestId) return;
    try {
      await invoke<boolean>("cancel_batch_copy", { request_id: activeRequestId });
    } catch (error) {
      onError(messageFromError(error, "Nie udało się anulować pozostałych projektów."));
    }
  }

  async function copyDiagnostic() {
    const diagnostic = result?.diagnostic_report ?? preview?.diagnostic_report;
    if (!diagnostic) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(diagnostic, null, 2));
      setReportCopied(true);
    } catch {
      onError("Nie udało się skopiować raportu zbiorczego.");
    }
  }

  const jobs = result?.jobs ?? preview?.jobs ?? [];
  const summary = result?.summary ?? preview?.summary;

  return (
    <section className="batch-panel" aria-label="Kopiowanie wielu projektów">
      <div className="batch-heading">
        <div><h2>Wybrane projekty</h2><span>{selections.length} do przygotowania</span></div>
        <button className="icon-text-button" onClick={onClose} disabled={busy}><X size={15} />Zmień wybór</button>
      </div>

      {!preview ? (
        <div className="batch-start">
          <div className="batch-selection-list">
            {selections.map((selection) => <span key={selection.selection_id}>{fileName(selection.native_als_path)}</span>)}
          </div>
          {experimentalCompatibilityAvailable ? (
            <label className="experimental-consent">
              <input type="checkbox" checked={experimentalConsent} onChange={(event) => setExperimentalConsent(event.target.checked)} />
              <span>Zezwalam na eksperymentalne kopie niepotwierdzonych wersji Live</span>
            </label>
          ) : null}
          <button className="button primary" onClick={prepareBatch} disabled={busy}>
            {busy ? <LoaderCircle className="spin" size={16} /> : <FolderOutput size={16} />}
            Wybierz folder docelowy
          </button>
        </div>
      ) : (
        <>
          <div className="batch-summary">
            <span><b>{summary?.ready_job_count ?? 0}</b> gotowych</span>
            <span className={(summary?.blocked_job_count ?? 0) > 0 ? "warning" : ""}><b>{summary?.blocked_job_count ?? 0}</b> zablokowanych</span>
            {result ? <span><b>{(summary?.completed_job_count ?? 0) + (summary?.incomplete_job_count ?? 0)}</b> utworzonych</span> : null}
          </div>

          <div className="batch-jobs">
            {jobs.map((job) => (
              <div className="batch-job" key={job.job_id}>
                <span className={`batch-job-status ${job.job_status}`}>
                  {job.job_status.includes("complete") ? <Check size={15} /> : job.job_status === "ready" ? <ShieldCheck size={15} /> : <AlertTriangle size={15} />}
                </span>
                <span className="batch-job-name"><strong>{fileName(job.source_als_path)}</strong><small>{statusLabel(job.job_status)}</small></span>
                {"preview" in job && job.preview ? <span className="batch-job-fact">{job.preview.copy_asset_count} plików</span> : null}
                {"result" in job && job.result?.final_target_root ? (
                  <button className="icon-button" title="Otwórz folder" onClick={() => openResultFolder(job.result?.final_target_root)}><ExternalLink size={16} /></button>
                ) : null}
              </div>
            ))}
          </div>

          {busy && progress ? (
            <div className="batch-progress"><LoaderCircle className="spin" size={17} /><span>Ukończono {progress.completed_job_count} z {progress.total_job_count}</span></div>
          ) : null}

          <div className="batch-actions">
            <button className="button secondary" onClick={copyDiagnostic} disabled={busy}>
              {reportCopied ? <Check size={15} /> : <Clipboard size={15} />}{reportCopied ? "Raport skopiowany" : "Kopiuj raport"}
            </button>
            {busy ? (
              <button className="button secondary" onClick={cancelRemaining}><Square size={14} />Anuluj pozostałe</button>
            ) : !result ? (
              <>
                <button className="button secondary" onClick={prepareBatch}>Zmień folder</button>
                <button className="button primary" onClick={executeBatch} disabled={preview.summary.ready_job_count === 0}><ShieldCheck size={16} />Utwórz kopie</button>
              </>
            ) : null}
          </div>
        </>
      )}
    </section>
  );
}
