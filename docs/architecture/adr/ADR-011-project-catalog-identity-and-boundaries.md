# ADR-011: Project Catalog Identity And Boundaries

Status: accepted
Date: 2026-08-03

## Context

The one-project Desktop flow starts from one ALS chosen by the user. The next
product increment must discover many Ableton Projects, present a simple list,
accept one or many selections and reuse the existing safe copy pipeline.

Local evidence shows that raw ALS files cannot be presented as one flat list:
Backups are numerous, several physical Project folders contain multiple main
ALS files, identical folder or Set names occur in different locations, and
some ALS files have no structural Project marker.

A physical Project folder is therefore not the same thing as one ALS file or a
logical musical work. Treating every main ALS in one folder as a version family
would create false relationships.

## Decision

The Project catalog uses four separate concepts:

```text
ProjectFolder
  one physical filesystem container supported by structural Project evidence

LiveSet
  one concrete ALS file observation at one native path and scan time

BackupSet
  one LiveSet observed under an Ableton Backup directory

ProjectWork
  a future logical musical work or version family supported by separate
  relationship evidence
```

`ALSProjectScanner` observes ALS files, exact Project markers and scan coverage
under explicit approved roots. It does not parse ALS, group projects, hash
audio, persist data or write.

`ProjectCatalogBuilder` is pure. It builds physical ProjectFolder, LiveSet,
BackupSet and ungrouped-Set records from scanner observations. It does not infer
ProjectWork relationships, select a primary Set or access the filesystem.

`ProjectCatalogStore` persists snapshots, coverage and freshness behind a
private storage adapter. Partial scans cannot mark unseen records missing,
delete history or claim complete coverage.

The Desktop receives a small display projection and opaque identifiers. Manual
`Add ALS` and catalog selection converge on one versioned `ProjectSelection`
contract. React does not group paths or choose a Set from a multi-Set folder.

Batch copy is an application-level orchestrator. It prepares per-project
previews, resolves target-name conflicts before write consent, and then invokes
the unchanged one-project application flow sequentially. Every selected Set is
an isolated job with its own staging, validation, manifests and result. One
blocked or failed job does not invalidate other jobs.

## Platform Boundary

The core scanner receives explicit native roots and uses `PathBuf`. Root
selection and platform permission discovery remain adapter responsibilities.
Symlinks and Windows reparse-point traversals are not followed by default.
Denied and skipped locations produce partial coverage instead of silent
success.

The first scanner does not need SQLite, a filesystem watcher, ALS parsing,
sample hashes, parallel copy or version-family inference.

## Consequences

- The scanner, catalog, UI and batch remain independently testable.
- Folder and list views can be added as projections without changing scanner
  truth.
- Version-family and semantic-diff work can use catalog Set snapshots later
  without contaminating physical discovery.
- Existing module 004 `ProjectDiscovery` remains responsible only for
  establishing Project-root context for one selected ALS.
- Existing one-project copy, rewrite and validation policy remains the sole
  policy used by batch mode.
- The first Project catalog pass remains lightweight and read-only.

## Rejected Alternatives

```text
Expand module 004 into a whole-computer scanner
  rejected because it would mix one-selected-ALS context with broad discovery

Treat each ALS as an independent project row
  rejected because Backups and same-folder Sets would overwhelm the user

Treat each physical Project folder as one musical work
  rejected because one folder may contain unrelated main Sets

Parse every ALS or hash audio during discovery
  rejected because neither is needed to build the selection catalog

Create a second batch-specific package pipeline
  rejected because it would duplicate and drift from proven safety policy
```
