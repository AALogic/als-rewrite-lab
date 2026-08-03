import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import {
  AlertTriangle,
  Check,
  CircleAlert,
  Clipboard,
  Copy,
  ExternalLink,
  FileAudio,
  FolderOutput,
  FolderOpen,
  LoaderCircle,
  Play,
  Search,
  ShieldCheck,
} from "lucide-react";
import type {
  CompatibilityTestReport,
  CompatibilityTestReportRequest,
  DesktopAnalyzeResult,
  DesktopApplicationProfile,
  DesktopApplicationError,
  DesktopCopyPreview,
  DesktopCopyResult,
  PreflightRequirement,
} from "./contracts/desktop";
import "./App.css";

function fileName(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function availabilityLabel(status: string) {
  if (status === "regular_file_candidate_observed") return "Plik znaleziony";
  if (status === "no_regular_file_candidate_observed") return "Wymaga wyszukania";
  return "Do sprawdzenia";
}

function availabilityTone(status: string) {
  if (status === "regular_file_candidate_observed") return "success";
  if (status === "no_regular_file_candidate_observed") return "danger";
  return "warning";
}

function requirementLabel(requirement: PreflightRequirement) {
  if (requirement.management_class === "system_dependency") return "System Abletona";
  return availabilityLabel(requirement.availability_status);
}

function requirementTone(requirement: PreflightRequirement) {
  if (requirement.management_class === "system_dependency") return "system";
  return availabilityTone(requirement.availability_status);
}

function errorMessage(error: unknown, fallback: string) {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as DesktopApplicationError).message;
    if (typeof message === "string" && message.length > 0) return message;
  }
  return fallback;
}

function App() {
  const [applicationProfile, setApplicationProfile] = useState<DesktopApplicationProfile>({
    schema_version: "0.1",
    application_profile: "strict_alpha",
    experimental_compatibility_available: false,
  });
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [result, setResult] = useState<DesktopAnalyzeResult | null>(null);
  const [busyStage, setBusyStage] = useState<"analysis" | "preview" | "copy" | null>(null);
  const [uiError, setUiError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [copyReportCopied, setCopyReportCopied] = useState(false);
  const [copyPreview, setCopyPreview] = useState<DesktopCopyPreview | null>(null);
  const [copyResult, setCopyResult] = useState<DesktopCopyResult | null>(null);
  const [experimentalConsent, setExperimentalConsent] = useState(false);
  const [manualOutcome, setManualOutcome] = useState<CompatibilityTestReportRequest["manual_verification_outcome"]>("not_checked");
  const [testedAbletonVersion, setTestedAbletonVersion] = useState("");
  const [compatibilityReportCopied, setCompatibilityReportCopied] = useState(false);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const report = result?.preflight_report ?? null;
  const busy = busyStage !== null;
  const operationError = copyResult?.errors[0] ?? copyPreview?.errors[0] ?? null;
  const operationDiagnostic = copyResult?.errors.length
    ? copyResult.diagnostic_report
    : copyPreview?.errors.length
      ? copyPreview.diagnostic_report
      : null;
  const compatibility = result?.diagnostic_report.rewrite_compatibility ?? null;
  const unconfirmedDocument = compatibility?.document_profile === "unconfirmed_ableton_version";
  const latestCopyDiagnostic = copyResult?.diagnostic_report ?? copyPreview?.diagnostic_report ?? null;

  useEffect(() => {
    invoke<DesktopApplicationProfile>("get_application_profile")
      .then(setApplicationProfile)
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!busyStage) {
      setElapsedSeconds(0);
      return;
    }
    const startedAt = Date.now();
    const timer = window.setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startedAt) / 1000));
    }, 1000);
    return () => window.clearInterval(timer);
  }, [busyStage]);

  const status = useMemo(() => {
    if (busyStage === "analysis") return { label: "Analiza trwa", tone: "working" };
    if (busyStage === "preview") return { label: "Przygotowuję plan", tone: "working" };
    if (busyStage === "copy") return { label: "Tworzę kopię", tone: "working" };
    if (copyResult?.run_status.includes("ready_for_manual_check")) {
      return { label: "Kopia gotowa", tone: "complete" };
    }
    if (result?.run_status === "analysis_complete") {
      return { label: "Analiza gotowa", tone: "complete" };
    }
    if (result?.run_status === "analysis_failed" || uiError) {
      return { label: "Analiza zatrzymana", tone: "failed" };
    }
    return { label: "Gotowy", tone: "idle" };
  }, [busyStage, copyResult, result, uiError]);

  async function chooseAls() {
    setUiError(null);
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Ableton Live Set", extensions: ["als"] }],
      });
      if (typeof path === "string") {
        setSelectedPath(path);
        setResult(null);
        setCopied(false);
        setCopyReportCopied(false);
        setCopyPreview(null);
        setCopyResult(null);
        setExperimentalConsent(false);
        setManualOutcome("not_checked");
        setTestedAbletonVersion("");
        setCompatibilityReportCopied(false);
      }
    } catch {
      setUiError("Nie udało się otworzyć wyboru pliku.");
    }
  }

  async function runAnalysis() {
    if (!selectedPath) return;
    setBusyStage("analysis");
    setUiError(null);
    setCopied(false);
    setCopyReportCopied(false);
    setCopyPreview(null);
    setCopyResult(null);
    setExperimentalConsent(false);
    setManualOutcome("not_checked");
    setTestedAbletonVersion("");
    setCompatibilityReportCopied(false);
    try {
      const requestId = globalThis.crypto?.randomUUID?.() ?? `desktop-${Date.now()}`;
      const next = await invoke<DesktopAnalyzeResult>("analyze_project", {
        request: { request_id: requestId, source_als_path: selectedPath },
      });
      setResult(next);
    } catch (error) {
      setUiError(errorMessage(error, "Analiza nie została wykonana."));
    } finally {
      setBusyStage(null);
    }
  }

  async function prepareProjectCopy() {
    if (!selectedPath || !report) return;
    setUiError(null);
    setCopyReportCopied(false);
    setCopyResult(null);
    try {
      const destinationParent = await open({ multiple: false, directory: true });
      if (typeof destinationParent !== "string") return;
      setBusyStage("preview");
      const targetProjectRoot = await invoke<string>("suggest_target_project_root", {
        source_als_path: selectedPath,
        destination_parent: destinationParent,
      });
      const requestId = globalThis.crypto?.randomUUID?.() ?? `copy-${Date.now()}`;
      const preview = await invoke<DesktopCopyPreview>("prepare_copy", {
        request: {
          request_id: requestId,
          source_als_path: selectedPath,
          target_project_root: targetProjectRoot,
          experimental_compatibility_consent:
            applicationProfile.experimental_compatibility_available
            && unconfirmedDocument
            && experimentalConsent,
        },
      });
      setCopyPreview(preview);
      if (preview.errors.length) setUiError(preview.errors[0].message);
    } catch (error) {
      setUiError(errorMessage(error, "Nie udało się przygotować planu kopii."));
    } finally {
      setBusyStage(null);
    }
  }

  async function executeProjectCopy() {
    if (!copyPreview) return;
    setBusyStage("copy");
    setUiError(null);
    setCopyReportCopied(false);
    try {
      const requestId = globalThis.crypto?.randomUUID?.() ?? `execute-${Date.now()}`;
      const next = await invoke<DesktopCopyResult>("execute_copy", {
        request: { request_id: requestId, preview: copyPreview, write_consent: true },
      });
      setCopyResult(next);
      if (next.errors.length) setUiError(next.errors[0].message);
      if (next.final_target_root) setCopyPreview(null);
    } catch (error) {
      setUiError(errorMessage(error, "Kopia projektu nie została utworzona."));
    } finally {
      setBusyStage(null);
    }
  }

  async function copyDiagnostic() {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(result.diagnostic_report, null, 2));
      setCopied(true);
    } catch {
      setUiError("Nie udało się skopiować raportu diagnostycznego.");
    }
  }

  async function copyOperationDiagnostic() {
    if (!operationDiagnostic) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(operationDiagnostic, null, 2));
      setCopyReportCopied(true);
    } catch {
      setUiError("Nie udało się skopiować raportu błędu.");
    }
  }

  async function copyCompatibilityReport() {
    if (!result) return;
    try {
      const report = await invoke<CompatibilityTestReport>("finalize_compatibility_report", {
        request: {
          analysis_report: result.diagnostic_report,
          copy_report: latestCopyDiagnostic,
          manual_verification_outcome: manualOutcome,
          tested_ableton_version: testedAbletonVersion.trim() || null,
        } satisfies CompatibilityTestReportRequest,
      });
      await navigator.clipboard.writeText(JSON.stringify(report, null, 2));
      setCompatibilityReportCopied(true);
    } catch (error) {
      setUiError(errorMessage(error, "Nie udało się przygotować raportu zgodności."));
    }
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-block">
          <div className="brand-mark" aria-hidden="true">AR</div>
          <div>
            <h1>ALS Rescue</h1>
            <p>{applicationProfile.application_profile === "compatibility_lab" ? "Compatibility Lab" : "Desktop Alpha"}</p>
          </div>
        </div>
        <div className={`app-status ${status.tone}`}>
          <span aria-hidden="true" />{status.label}
        </div>
      </header>

      <section className="project-bar" aria-label="Wybrany projekt">
        <div className="project-file">
          <div className="file-icon"><FileAudio size={20} /></div>
          <div className="file-copy">
            <strong>{selectedPath ? fileName(selectedPath) : "Nie wybrano projektu"}</strong>
            <span title={selectedPath ?? undefined}>
              {selectedPath ?? "Plik Ableton Live Set (.als)"}
            </span>
          </div>
        </div>
        <div className="project-actions">
          <button className="button secondary" onClick={chooseAls} disabled={busy}>
            <FolderOpen size={17} />Wybierz ALS
          </button>
          <button className="button primary" onClick={runAnalysis} disabled={!selectedPath || busy}>
            {busy ? <LoaderCircle className="spin" size={17} /> : <Play size={17} />}
            {busy ? "Analizuję" : "Analizuj"}
          </button>
        </div>
      </section>

      {busyStage ? (
        <section className="operation-progress" role="status" aria-live="polite">
          <LoaderCircle className="spin" size={20} aria-hidden="true" />
          <div>
            <strong>
              {busyStage === "analysis"
                ? "Analizuję projekt"
                : busyStage === "preview"
                  ? "Przygotowuję bezpieczny plan kopii"
                  : "Tworzę i sprawdzam kopię projektu"}
            </strong>
            <span>Program pracuje. Upłynęło {elapsedSeconds} s.</span>
          </div>
        </section>
      ) : null}

      {(uiError || result?.errors.length) ? (
        <section className="error-banner" role="alert">
          <CircleAlert size={19} />
          <div>
            <strong>Operacja nie może być zakończona</strong>
            <span>{uiError ?? result?.errors[0]?.message}</span>
            {operationError ? (
              <small>Kod: {operationError.error_code} · Etap: {operationError.stage}</small>
            ) : null}
            {operationDiagnostic ? (
              <button className="error-report-button" onClick={copyOperationDiagnostic}>
                {copyReportCopied ? <Check size={15} /> : <Clipboard size={15} />}
                {copyReportCopied ? "Raport skopiowany" : "Kopiuj raport błędu"}
              </button>
            ) : null}
          </div>
        </section>
      ) : null}

      {report ? (
        <>
          {compatibility ? (
            <section className={`compatibility-panel ${unconfirmedDocument ? "experimental" : "confirmed"}`}>
              <div>
                <strong>
                  {unconfirmedDocument ? "Niepotwierdzona wersja Abletona" : "Potwierdzony profil Live 11.3"}
                </strong>
                <span>
                  {compatibility.ableton_creator_version ?? "Wersja nieznana"} · {compatibility.lab_compatible_count} znanych struktur · {compatibility.unsupported_shape_count} nieznanych
                </span>
              </div>
              {applicationProfile.experimental_compatibility_available && unconfirmedDocument ? (
                <label className="experimental-consent">
                  <input
                    type="checkbox"
                    checked={experimentalConsent}
                    onChange={(event) => {
                      setExperimentalConsent(event.target.checked);
                      setCompatibilityReportCopied(false);
                    }}
                  />
                  <span>Zezwalam na eksperymentalną kopię do ręcznego sprawdzenia</span>
                </label>
              ) : null}
            </section>
          ) : null}

          <section className="summary-grid" aria-label="Podsumowanie analizy">
            <Metric label="Referencje w ALS" value={report.summary.reference_occurrence_count} icon={<FileAudio size={18} />} />
            <Metric label="Wymagane pliki" value={report.summary.required_asset_count} icon={<ShieldCheck size={18} />} />
            <Metric label="System Abletona" value={report.summary.system_dependency_count} icon={<ShieldCheck size={18} />} />
            <Metric label="Znalezione" value={report.summary.candidate_observed_count} icon={<Check size={18} />} tone="success" />
            <Metric label="Do odnalezienia" value={report.summary.needs_search_count} icon={<Search size={18} />} tone="danger" />
          </section>

          <section className="report-section">
            <div className="section-heading">
              <div><h2>Zależności audio</h2><p>{fileName(report.report_metadata.source_als_path)}</p></div>
              <button className="icon-text-button" onClick={copyDiagnostic}>
                {copied ? <Check size={16} /> : <Clipboard size={16} />}
                {copied ? "Skopiowano" : "Kopiuj diagnostykę"}
              </button>
            </div>

            <div className="table-wrap">
              <table>
                <thead><tr><th>Plik</th><th>Status</th><th>Użycia</th><th>Obserwowana lokalizacja</th></tr></thead>
                <tbody>
                  {report.requirements.map((requirement) => (
                    <tr key={requirement.required_asset_id}>
                      <td><div className="dependency-name"><FileAudio size={16} /><span>{requirement.filename ?? "Nazwa niedostępna"}</span></div></td>
                      <td>
                        <span className={`status-pill ${requirementTone(requirement)}`}>
                          {requirement.availability_status === "no_regular_file_candidate_observed" ? <AlertTriangle size={14} /> : <Check size={14} />}
                          {requirementLabel(requirement)}
                        </span>
                      </td>
                      <td>{requirement.occurrence_count}</td>
                      <td className="path-cell" title={requirement.candidate_paths[0]}>
                        {requirement.candidate_paths[0] ?? "Brak lokalizacji"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {report.requirements.length === 0 ? <div className="empty-table">Brak aktywnych zależności audio.</div> : null}
            </div>
          </section>

          <section className="copy-section" aria-label="Kopia projektu">
            <div>
              <h2>Bezpieczna kopia projektu</h2>
              <p>Dostępne pliki zostaną zebrane, a brakujące pozostaną oznaczone w projekcie.</p>
            </div>
            <button
              className="button primary"
              onClick={prepareProjectCopy}
              disabled={busy || Boolean(
                applicationProfile.experimental_compatibility_available
                && unconfirmedDocument
                && !experimentalConsent
              )}
            >
              {busyStage === "preview" ? <LoaderCircle className="spin" size={17} /> : <FolderOutput size={17} />}
              Wybierz miejsce kopii
            </button>
          </section>

          {copyPreview && copyPreview.errors.length === 0 ? (
            <section className="copy-confirmation" aria-label="Potwierdzenie kopii">
              <div className="copy-confirmation-heading">
                <div className="copy-icon"><Copy size={19} /></div>
                <div>
                  <strong>{copyPreview.omitted_asset_count ? "Powstanie kopia niepełna" : "Projekt jest gotowy do skopiowania"}</strong>
                  <span title={copyPreview.target_project_root}>{copyPreview.target_project_root}</span>
                </div>
              </div>
              <div className="copy-facts">
                <span><b>{copyPreview.copy_asset_count}</b> plików do skopiowania</span>
                <span><b>{copyPreview.rewrite_reference_count}</b> odwołań do zmiany</span>
                <span><b>{copyPreview.system_dependency_count}</b> zależności systemowych bez kopiowania</span>
                <span className={copyPreview.omitted_asset_count ? "fact-warning" : ""}><b>{copyPreview.omitted_asset_count}</b> brakujących</span>
              </div>
              <div className="copy-confirmation-actions">
                <button className="button secondary" onClick={() => setCopyPreview(null)} disabled={busy}>Anuluj</button>
                <button className="button primary" onClick={executeProjectCopy} disabled={busy}>
                  {busyStage === "copy" ? <LoaderCircle className="spin" size={17} /> : <ShieldCheck size={17} />}
                  Utwórz kopię
                </button>
              </div>
            </section>
          ) : null}

          {copyResult?.final_target_root ? (
            <section className={`copy-result ${copyResult.omitted_asset_count ? "incomplete" : "complete"}`}>
              <div>
                {copyResult.omitted_asset_count ? <AlertTriangle size={20} /> : <Check size={20} />}
                <div>
                  <strong>{copyResult.omitted_asset_count ? "Kopia niepełna jest gotowa" : "Kopia projektu jest gotowa"}</strong>
                  <span>{copyResult.copied_asset_count} plików skopiowanych, {copyResult.system_dependency_count} systemowych bez kopiowania, {copyResult.omitted_asset_count} brakujących.</span>
                </div>
              </div>
              <button className="icon-text-button" onClick={() => openPath(copyResult.final_target_root!)}>
                <ExternalLink size={16} />Otwórz folder
              </button>
            </section>
          ) : null}

          {applicationProfile.experimental_compatibility_available ? (
            <section className="compatibility-report-section" aria-label="Raport testu zgodności">
              <div>
                <h2>Raport testu zgodności</h2>
                <p>Po sprawdzeniu kopii w Abletonie wybierz wynik i skopiuj raport.</p>
              </div>
              <div className="compatibility-report-controls">
                <label>
                  <span>Wynik ręcznego sprawdzenia</span>
                  <select
                    value={manualOutcome}
                    onChange={(event) => {
                      setManualOutcome(event.target.value as CompatibilityTestReportRequest["manual_verification_outcome"]);
                      setCompatibilityReportCopied(false);
                    }}
                  >
                    <option value="not_checked">Jeszcze nie sprawdzono</option>
                    <option value="opened_without_missing_files">Otwiera się bez missing files</option>
                    <option value="opened_with_missing_files">Otwiera się z missing files</option>
                    <option value="failed_to_open">Nie otwiera się</option>
                  </select>
                </label>
                <label>
                  <span>Wersja Abletona użyta do testu</span>
                  <input
                    value={testedAbletonVersion}
                    maxLength={80}
                    placeholder="np. Live 10.1.43"
                    onChange={(event) => {
                      setTestedAbletonVersion(event.target.value);
                      setCompatibilityReportCopied(false);
                    }}
                  />
                </label>
                <button className="button secondary" onClick={copyCompatibilityReport}>
                  {compatibilityReportCopied ? <Check size={16} /> : <Clipboard size={16} />}
                  {compatibilityReportCopied ? "Raport skopiowany" : "Kopiuj raport dla Codexa"}
                </button>
              </div>
            </section>
          ) : null}
        </>
      ) : (
        <section className="empty-state">
          <div className="empty-symbol"><FileAudio size={28} /></div>
          <strong>Oczekiwanie na projekt</strong>
          <span>Wybierz plik .als, aby rozpocząć analizę.</span>
        </section>
      )}

      <footer><span>Praca lokalna</span><span>Oryginalne pliki pozostają bez zmian</span></footer>
    </main>
  );
}

function Metric({ label, value, icon, tone = "" }: { label: string; value: number; icon: React.ReactNode; tone?: string }) {
  return <article className={`metric ${tone}`}><span>{label}</span><strong>{value}</strong>{icon}</article>;
}

export default App;
