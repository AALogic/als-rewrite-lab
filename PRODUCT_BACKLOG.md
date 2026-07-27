# Product Backlog

Status: working backlog
Date: 2026-07-27

## 1. Cel

Ten plik przechowuje rzeczy, ktore sa wazne, ale nie zawsze sa gotowe na spec albo kod.

To jest miejsce na:

```text
pomysly
hipotezy
funkcje pozniejsze
ryzyka
tematy researchowe
rzeczy odlozone poza MVP
```

## 2. Aktualna Kolejnosc

`CURRENT_STATE.md` jest jedynym wlascicielem aktualnego stanu, blockerow i
nastepnego kroku. Ten backlog ich nie kopiuje.

## 3. Kandydaci Na Pozniej

Pozycje w tym pliku sa kandydatami, a nie aktywna kolejnoscia implementacji.
Promocja do biezacej pracy wymaga aktualizacji `CURRENT_STATE.md` i aktywnej
specyfikacji modulu.

## 4. Pozniej

Rzeczy wazne, ale nie na MVP:

```text
Tauri desktop UI
SQLite asset index
redirect ledger
multi-project scan
safe cleanup
quarantine
Windows migration
Max for Live bridge
JUCE/VST bridge
```

## 5. Hipotezy do sprawdzenia

```text
Source classification should be separate from ALSReader.
Preflight report should appear before first rewrite/package UI.
Redirect ledger becomes necessary when user reorganizes a central sample library.
Max for Live may be a better first Ableton bridge than VST.
RelativePathType 0 may represent project-relative paths stored directly in Path.
ALSReader may need two layers: raw/internal dependency model and diagnostic JSON.
ALSReader may need to preserve xml_context / usage_context for downstream modules.
```

## 6. Ryzyka

```text
Starting UI before stable core can create fake product progress.
Putting too much into ALSReader can blur module boundaries.
Treating hypotheses as facts can make rewrite unsafe.
Long chat history cannot be the only project memory.
Accepting a locally tested module without Product Spine traceability can
produce code that works technically but does not support package/rewrite.
Treating ALS Structure Corpus 20 observations as final rewrite rules would be unsafe.
```

## 6A. Deferred Technical Hardening From Code Review

Status:

```text
accepted but intentionally deferred unless noted otherwise
```

Implemented immediately on 2026-06-30:

```text
ALSReader no longer uses host OS path parsing for filename/extension.
ALSReader uses first non-empty raw path candidate for derived filename/extension.
ALSReader enforces compressed ALS and decompressed XML size limits.
CLI has an extract command that runs ALSReader -> DependencyExtractor.
Added typed ALS path parsing helper:
  RawAlsPath
  ParsedAlsPath
  AlsPathKind
  parse_als_path
Added minimal architecture docs:
  docs/architecture/README.md
  docs/architecture/traceability.md
  docs/architecture/adr/
Prepared specs/003-path-verifier as the next module boundary.
```

Deferred until the right module boundary:

```text
Enum migration:
  Replace decision/status strings with Rust enums gradually.
  First candidates: PathBasis, ExtractionStatus, EvidenceStatus, Severity.
  Reassess only inside an active module; do not retrofit 001/002 merely to
  satisfy this backlog note.

Activity status:
  active_audio_references currently means SampleRef/FileRef extracted from
  reader scope with usage_context unknown.
  Before widening ALS rewrite readiness, add activity_status or rename
  semantics if tests show SampleRef can represent unsupported/non-active
  contexts.

Stable dependency identity:
  dep_audio_000000 is valid for the current 002 contract.
  Before promising cross-run identity, add a stable_dependency_key derived from
  ALS hash, xml locator, raw path and supporting evidence.

Parsed numeric evidence:
  Keep ALSReader raw fields as strings.
  Reassess parsed_original_file_size ownership in the active observation or
  resolution contract instead of repeatedly parsing raw strings independently.

Storage model:
  Current Rust structs are in-memory/domain/JSON models.
  A future SQLite schema must not treat storage IDs as domain IDs.

```

## 6B. Deferred Workflow Recommendations From Architecture Review

Status:

```text
remaining deferred recommendations
```

Deferred until the matching module boundary:

```text
cargo-deny / dependency policy:
  Add after repository structure is tracked cleanly in Git.
  Purpose: license/advisory/supply-chain checks in CI.

SQLite index:
  Add only when AssetIndexer needs persistent reuse across runs.

Tauri desktop UI:
  Add after core pipeline can produce trustworthy preflight/package plans.
```

## 6C. Agent Workflow Readiness

Live repository, gate, and CI readiness belongs to `CURRENT_STATE.md`,
`README.md`, and `.github/workflows/quality.yml`. This backlog does not mirror
that state.

Operating recommendation:

```text
Use one Codex directly for small sequential tasks.
Use No-Mistakes for safety-sensitive or contract-sensitive changes.
Use Firstmate only when independent research/review or genuine parallel work
repays its larger time and token cost.
```

## 7. Research Evidence To Reuse

```text
<private-corpus-root>/
  20 copied ALS files
  findings.md
  structure_report.md
  reports/corpus_summary.json

session-digests/2026-06-02_als-structure-corpus-20.md
  preserved conversation/product context
```

## 8. Product Decisions To Promote Into Full Spec

```text
Full disk scan:
  Product should request user permission and scan the local computer broadly,
  so the user gets a full project/dependency overview.

Project list:
  Show grouped main Ableton projects by default.
  Hide Backup/backup ALS files from the main list, but record them for later use.

Dependency handling:
  Audio samples are managed/copied by the product.
  VST/plugin dependencies are reported, not copied.
  User Library dependencies should be copied for portable packages by default,
  unless a later mode explicitly treats them as shared dependencies.
  Core Library / Factory Packs are treated as system dependencies by default.

Execution model:
  Always create a plan before copy/rewrite.
  Manifest is required for package/rewrite operations.
  Do not auto-open Ableton after package creation; user verifies manually.

Matching:
  Candidate matches with score >= 95 may be auto-selected by the system.
  The full package plan is still shown to the user before execution.

Target package:
  Accepted project dependencies should be copied into an Ableton-like project
  structure.
  Alternative rescue candidates should be kept outside active ALS references,
  in a separate rescue area/folder for later user review.
```

## 9. Ableton Domain Tests To Run

### Duplicate Filename Collect All and Save Test

Purpose:

```text
Discover how Ableton handles two different audio files with the same filename
when both are used in one Live Set and Collect All and Save is executed.
```

Steps:

```text
1. Create two different audio files with the same filename in two different folders.
2. Use both files in one Ableton Live Set.
3. Run Collect All and Save in Ableton.
4. Inspect the collected project folder.
5. Inspect the rewritten ALS FileRef data.
```

Questions:

```text
Where does Ableton copy each file?
Does Ableton rename either file?
Does Ableton preserve source folder structure or flatten into one folder?
How does Ableton rewrite FileRef / Path / RelativePath values?
Do copied files preserve hash/size/content exactly?
```

Spec consequence:

```text
Until tested, Rescue must never overwrite files on filename collision.
It must generate unique target paths and record old -> new mapping in manifest.
```

### Core Library / Factory Packs Portability Test

Purpose:

```text
Decide when Core Library / Factory Packs can be treated as system dependencies
and when they become portable package risks.
```

Current working rule:

```text
Default:
  report as system_dependency
  do not copy by default
  mark as portable_risk if target computer/version/packs are unknown

Future strict portable mode:
  may collect factory pack audio if user explicitly wants a package that does
  not rely on the target computer having the same Packs installed.
```

Questions:

```text
Are Core Library references stable between Ableton versions?
Which Factory Pack files are safe to treat as system dependencies?
When does Collect All and Save copy Factory Pack files?
How should Rescue classify missing Factory Pack/Core files?
```

Suggested tests:

```text
Run CAS on a project using Core/Factory Pack audio with:
  same Ableton version
  different Ableton version if available
  missing Factory Pack scenario if safely reproducible
Compare collected files and ALS FileRef behavior.
```

### Plugin / VST Preset Portability Test

Purpose:

```text
Discover what Ableton stores inside ALS for plugin state and what external
preset files, if any, Collect All and Save copies.
```

Current working rule:

```text
VST/AU plugin:
  report dependency
  do not copy plugin
  do not assume external plugin preset file is portable

Max for Live device:
  investigate separately; Ableton documentation says used Max for Live devices
  are copied to Presets by Collect All and Save.
```

Questions:

```text
Is the active plugin state stored inside the ALS?
Are VST2/VST3/AU preset files referenced as external files?
Can Rescue detect where those preset files live?
Does Collect All and Save copy any third-party plugin preset files?
How does behavior differ for VST2, VST3, AU, Ableton Rack presets and Max for Live?
```

Suggested tests:

```text
Create small Live Sets using:
  one VST2 preset
  one VST3 preset
  one AU preset on macOS
  one Ableton Rack preset
  one Max for Live device
Run Collect All and Save.
Compare project folder contents and ALS FileRef/device references before/after.
```
