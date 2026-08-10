import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  AlertTriangle,
  CalendarClock,
  CheckSquare,
  FileAudio,
  FolderPlus,
  HardDrive,
  LoaderCircle,
  RefreshCw,
  Search,
} from "lucide-react";
import type { DesktopApplicationError } from "../contracts/desktop";
import type {
  ProjectCatalogListResult,
  ProjectCatalogRefreshResult,
  ProjectSelection,
  ProjectSelectionResult,
} from "./contracts";
import {
  formatFileSize,
  formatProjectDate,
  selectedCatalogIds,
  visibleProjectGroups,
} from "./projectCatalogView";

type CatalogBusyStage = "catalog_load" | "catalog_scan" | "selection";

type Props = {
  disabled: boolean;
  onBusyChange: (stage: CatalogBusyStage | null) => void;
  onError: (message: string | null) => void;
  onSelectionResolved: (selections: ProjectSelection[]) => void;
};

function messageFromError(error: unknown, fallback: string) {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as DesktopApplicationError).message;
    if (typeof message === "string" && message) return message;
  }
  return fallback;
}

export function ProjectCatalogPanel({
  disabled,
  onBusyChange,
  onError,
  onSelectionResolved,
}: Props) {
  const [catalog, setCatalog] = useState<ProjectCatalogListResult | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [query, setQuery] = useState("");
  const [includeBackups, setIncludeBackups] = useState(false);
  const [busyStage, setBusyStage] = useState<CatalogBusyStage | null>(null);
  const groups = useMemo(
    () => catalog ? visibleProjectGroups(catalog, query) : [],
    [catalog, query],
  );
  const orderedSelectedIds = useMemo(
    () => catalog ? selectedCatalogIds(catalog, selectedIds) : [],
    [catalog, selectedIds],
  );
  const busy = disabled || busyStage !== null;

  useEffect(() => {
    void loadCatalog(includeBackups);
  }, [includeBackups]);

  async function withBusy<T>(stage: CatalogBusyStage, operation: () => Promise<T>) {
    setBusyStage(stage);
    onBusyChange(stage);
    try {
      return await operation();
    } finally {
      setBusyStage(null);
      onBusyChange(null);
    }
  }

  async function loadCatalog(showBackups: boolean) {
    onError(null);
    try {
      const next = await withBusy("catalog_load", () =>
        invoke<ProjectCatalogListResult>("list_project_catalog", {
          request: { include_backups: showBackups },
        }),
      );
      if (next.errors.some((error) => error.error_code !== "PROJECT_CATALOG_NOT_FOUND")) {
        onError(next.errors[0]?.message ?? "Nie udało się wczytać listy projektów.");
      }
      setCatalog(next.errors.length ? null : next);
      retainKnownSelections(next);
    } catch (error) {
      onError(messageFromError(error, "Nie udało się wczytać listy projektów."));
    }
  }

  function retainKnownSelections(next: ProjectCatalogListResult) {
    const knownIds = new Set(next.items.map((item) => item.live_set_id));
    setSelectedIds((current) => new Set([...current].filter((id) => knownIds.has(id))));
  }

  async function scanProjects() {
    const consent = window.confirm(
      "ALS Rescue przeskanuje lokalne foldery użytkowników i podłączone dyski w poszukiwaniu plików .als. Kontynuować?",
    );
    if (!consent) return;
    onError(null);
    try {
      const requestId = globalThis.crypto?.randomUUID?.() ?? `catalog-${Date.now()}`;
      const result = await withBusy("catalog_scan", () =>
        invoke<ProjectCatalogRefreshResult>("refresh_project_catalog", {
          request: {
            request_id: requestId,
            full_scan_consent: true,
            include_backups: includeBackups,
          },
        }),
      );
      if (result.errors.length) {
        onError(result.errors[0]?.message ?? "Skan projektów nie został zakończony.");
        return;
      }
      setCatalog(result.catalog);
      retainKnownSelections(result.catalog);
    } catch (error) {
      onError(messageFromError(error, "Skan projektów nie został zakończony."));
    }
  }

  async function addManualAls() {
    onError(null);
    try {
      const paths = await open({
        multiple: true,
        directory: false,
        filters: [{ name: "Ableton Live Set", extensions: ["als"] }],
      });
      const selectedPaths = typeof paths === "string" ? [paths] : paths;
      if (!selectedPaths?.length) return;
      await resolveSelection([], selectedPaths, null);
    } catch (error) {
      onError(messageFromError(error, "Nie udało się dodać plików ALS."));
    }
  }

  async function continueWithSelection() {
    if (!catalog || orderedSelectedIds.length === 0) return;
    await resolveSelection(orderedSelectedIds, [], catalog.metadata.catalog_revision);
  }

  async function resolveSelection(
    catalogIds: string[],
    manualPaths: string[],
    catalogRevision: number | null,
  ) {
    const requestId = globalThis.crypto?.randomUUID?.() ?? `selection-${Date.now()}`;
    const result = await withBusy("selection", () =>
      invoke<ProjectSelectionResult>("resolve_project_selection", {
        request: {
          request_id: requestId,
          catalog_revision: catalogRevision,
          catalog_live_set_ids: catalogIds,
          manual_als_paths: manualPaths,
        },
      }),
    );
    if (result.errors.length) {
      onError(result.errors[0]?.message ?? "Nie udało się zatwierdzić wyboru projektów.");
      return;
    }
    onSelectionResolved(result.selections);
  }

  function toggleSelection(id: string) {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return (
    <section className="catalog" aria-label="Katalog projektów Ableton">
      <div className="catalog-heading">
        <div>
          <h2>Twoje projekty</h2>
          <span>{catalog ? `${catalog.metadata.visible_item_count} dostępnych` : "Katalog nieutworzony"}</span>
        </div>
        <div className="catalog-actions">
          <button className="button secondary" onClick={addManualAls} disabled={busy}>
            <FolderPlus size={16} />Dodaj ALS
          </button>
          <button className="button secondary" onClick={scanProjects} disabled={busy}>
            {busyStage === "catalog_scan" ? <LoaderCircle className="spin" size={16} /> : <RefreshCw size={16} />}
            {catalog ? "Odśwież" : "Skanuj projekty"}
          </button>
        </div>
      </div>

      {catalog ? (
        <>
          <div className="catalog-toolbar">
            <label className="catalog-search">
              <Search size={16} aria-hidden="true" />
              <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Szukaj projektu" />
            </label>
            <label className="catalog-toggle">
              <input type="checkbox" checked={includeBackups} onChange={(event) => setIncludeBackups(event.target.checked)} />
              <span>Pokaż backupy</span>
            </label>
          </div>

          {catalog.warnings.length ? (
            <div className="catalog-warning"><AlertTriangle size={16} />Lista może być niepełna po ostatnim skanie.</div>
          ) : null}

          <div className="catalog-groups">
            {groups.map(({ group, items }) => (
              <section className="catalog-group" key={group.group_id}>
                <header><strong>{group.display_name}</strong><span>{items.length} {items.length === 1 ? "projekt" : "projekty"}</span></header>
                {items.map((item) => (
                  <label className="catalog-row" key={item.item_id}>
                    <input
                      type="checkbox"
                      checked={selectedIds.has(item.live_set_id)}
                      disabled={item.selection_status === "stale_unavailable" || busy}
                      onChange={() => toggleSelection(item.live_set_id)}
                    />
                    <span className="catalog-file-icon"><FileAudio size={17} /></span>
                    <span className="catalog-row-main">
                      <strong>{item.display_name}</strong>
                      <span>{item.location_kind.startsWith("backup") ? "Backup" : "Główny Set"}</span>
                    </span>
                    <span className="catalog-row-fact"><CalendarClock size={14} />{formatProjectDate(item.modified_time_unix_ms)}</span>
                    <span className="catalog-row-fact"><HardDrive size={14} />{formatFileSize(item.file_size)}</span>
                    {item.warning_codes.length ? <AlertTriangle className="catalog-row-warning" size={16} /> : null}
                  </label>
                ))}
              </section>
            ))}
            {!groups.length ? <div className="catalog-empty">Brak projektów pasujących do wyszukiwania.</div> : null}
          </div>

          <div className="catalog-selection-bar">
            <span><CheckSquare size={16} />Wybrano: <b>{orderedSelectedIds.length}</b></span>
            <button className="button primary" disabled={orderedSelectedIds.length === 0 || busy} onClick={continueWithSelection}>
              {busyStage === "selection" ? <LoaderCircle className="spin" size={16} /> : null}
              {orderedSelectedIds.length > 1 ? "Przygotuj wybrane" : "Analizuj wybrany"}
            </button>
          </div>
        </>
      ) : (
        <div className="catalog-empty catalog-empty-start">
          {busyStage === "catalog_load" ? <LoaderCircle className="spin" size={20} /> : <FileAudio size={20} />}
          <span>{busyStage === "catalog_load" ? "Wczytuję katalog" : "Uruchom skan lub dodaj plik ALS"}</span>
        </div>
      )}
    </section>
  );
}
