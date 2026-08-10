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
fine-grained desktop progress, cancellation and recovery
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

## 5A. Local Project And File Dependency Catalog

Classification:

```text
Later product capability
Build after multi-project discovery and the persistent local asset index
Not part of the current one-project MVP
```

Product idea:

```text
Give the user a local CMDB-like view of Ableton projects, Live Set versions,
audio files and the relationships between them.

The user should be able to start from either side:

project -> all files and system dependencies used by that project
file -> every project and Set snapshot that references that file
```

Questions the capability should answer:

```text
Which projects use this sample?
How many Set versions depend on it?
Where are all known occurrences of the same content?
What projects are at risk if this path disappears or the file is moved?
Is a relationship current, historical, missing, ambiguous or only inferred?
Which files appear unreferenced within the latest complete scan evidence?
```

Expected views:

```text
searchable project list
searchable file/content list
project detail with dependency table
file detail with reverse project references
interactive relationship graph for exploration
impact preview before a future move, cleanup or library reorganization
```

The graph is a presentation and query capability, not a reason to introduce a
graph database now. SQLite join tables should be sufficient for the expected
many-to-many relationships. The domain model should distinguish at least:

```text
ProjectWork
LiveSet
SetSnapshot
ReferenceOccurrence
RequiredAsset
FileOccurrence
ContentRecord
Volume
ScanRun
```

Every relationship must retain evidence and freshness, including the source
ALS snapshot, observation time, scan coverage and current/historical status.
Partial or stale scans must be visible to the user and must never authorize a
destructive cleanup. This catalog remains local-only and read-only until a
separate planned operation explicitly requests a move, relink or cleanup.

Likely implementation order:

```text
multi-project discovery
-> versioned Set snapshots
-> persistent incremental file index
-> relationship builder and reverse-reference queries
-> table/search UI
-> graph visualization
-> impact analysis for future move/cleanup workflows
```

## 5B. Live Set Version Families And Semantic Diff

Classification:

```text
Later product capability
Build after lightweight multi-project discovery and versioned Set snapshots
Not part of the current project scanner or batch-copy MVP
```

Product idea:

```text
Group ALS snapshots that are confirmed or likely to be versions of the same
musical work. Show a simple timeline or family view and let the user compare
two selected versions using a semantic, producer-friendly diff.
```

The product must distinguish:

```text
confirmed relationship
  official Ableton Backup sequence, explicit user confirmation, or a change
  directly observed and recorded by this application

probable relationship
  multiple independent similarity signals support the same Set family

unknown relationship
  evidence is insufficient or conflicting
```

Static ALS research on 2026-08-03 did not find a reliable embedded parent or
project-lineage identifier. In the local corpus:

```text
Revision was shared by hundreds of unrelated ALS files created by the same
Ableton build and must be treated as build metadata, not project identity.

OverwriteProtectionNumber was also massively reused and must not be treated as
project identity or inheritance evidence.
```

Candidate evidence for probable families may include:

```text
same confirmed Ableton Project Folder
normalized Set-name similarity
official Backup filename stem and embedded timestamp
filesystem chronology, with copy/move caveats
overlap of referenced sample paths or names
overlap of track, clip, device and pointee identifiers
track names and structural layout
plugin/device-chain similarity
tempo, scenes, locators and arrangement characteristics
Ableton creator/document compatibility
```

No single similarity signal may silently create a confirmed parent-child edge.
Templates can preserve object IDs, track layouts and devices across unrelated
songs. Same-folder placement is also evidence, not proof. The relationship
record should retain every supporting and conflicting observation, confidence,
ruleset version and any user confirmation.

Preferred presentation:

```text
confirmed Ableton backups -> chronological history
probable Save As variants -> probable family/timeline
uncertain ordering -> flat family ordered by observed time, without fake arrows
```

The diff should be semantic rather than a raw XML diff. Candidate user-facing
changes include:

```text
tempo and Ableton version changes
tracks, scenes and locators added or removed
track renames and structural changes
clips added, removed or materially changed
sample dependencies added, removed or made missing
plugins and devices added, removed or changed
project completeness changes
```

Likely future module boundaries:

```text
SetFingerprintExtractor
  produces a versioned, normalized structural fingerprint for one ALS snapshot

VersionRelationshipAnalyzer
  ranks family and chronology hypotheses while preserving evidence status

SemanticSetDiffer
  compares two snapshots and returns producer-facing changes
```

Do not enlarge ALSReader into a version-history engine. Fingerprints and diffs
should be computed lazily when project details are opened or as background
catalog work, then cached by ALS snapshot hash and extractor version. This does
not require hashing referenced audio files.

Domain evidence to retain:

- [Ableton: Backup Sets](https://help.ableton.com/hc/en-us/articles/360000377870-Backup-Sets)
- [Ableton: Saving Projects](https://help.ableton.com/hc/en-us/articles/115000915804-Saving-Projects)

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

Desktop distribution:
  Add signing, notarization and release provenance after the manual runtime gate.
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

### ALP Inspection And Optional Export Research

Status:

```text
Later / Research
Not part of the current MVP
No ALP writer implementation approved
```

Purpose:

```text
Investigate Ableton Live Pack (.alp) as an optional transport/archive format
after Rescue has already created and validated a self-contained Ableton Project.
Potential future value includes read-only Pack inspection, Pack dependency
reporting and an optional final export or Live-assisted handoff.
```

Known evidence:

```text
.alp is an undocumented proprietary binary container, not a renamed ZIP.
Observed official Pack structure includes a pl-a header, FolderConfigData,
concatenated payload data and a trailing serialized file/directory index with
offsets, sizes, names, versions and package metadata.
Official licensed Packs may contain .eflac / Encrypted FLAC assets and must not
be treated as equivalent to user-created project Packs without experiments.
```

Guardrails:

```text
Rescue continues to produce a normal validated Ableton Project folder.
Do not make ALP generation a requirement for portable project creation.
Do not implement licensed Pack decryption, authorization bypass or repackaging.
Do not write ALP until a read-only parser, version matrix and round-trip corpus
have established the supported user-created Pack format.
Prefer Ableton Live's own Create Pack operation when Live-assisted export is
sufficient.
```

Required experiments:

```text
1. Create minimal user-owned Projects and Packs in available Live 10/11/12 versions.
2. Vary one input at a time: ALS, WAV, AIF, preset, ASD and nested directories.
3. Compare Pack headers, payloads, trailing indexes and version metadata.
4. Unpack each Pack through Live and compare restored files with source hashes.
5. Separate user-created Pack behavior from official licensed Pack behavior.
6. Build a bounded read-only ALP inspector before considering extraction or writing.
7. Validate every experimental writer output by installing it in matching Live versions.
```

Promotion gate:

```text
Return to this topic only after the normal Project-folder package flow is stable
and there is a concrete product need for ALP inspection or one-file export.
Any writer requires a dedicated module spec, security limits for archive input,
cross-version fixtures and manual Ableton round-trip verification.
```
