# Product Spine

Status: active product spine  
Date: 2026-08-05
Scope: global product promise, use cases, capability map and gates from vision to code

## 1. Purpose

This file is the product spine.

It exists to prevent this failure:

```text
a module is technically correct,
has tests,
but does not clearly support the larger product.
```

Specs do not own the product vision.

Specs inherit from this spine.

## 2. Ownership

```text
Product Owner:
  user

Product / Tech Steward:
  Codex

Product Office:
  turns conversations into structured product facts, questions and decisions

Specs:
  local module contracts derived from this spine
```

Rule:

```text
If Codex would need to guess a product decision,
Codex must stop and request clarification.
```

## 3. Product Promise

The product helps Ableton users safely understand and manage project audio
dependencies across messy local folders.

The product should help users:

```text
discover Ableton Project folders and Live Sets across approved local roots
choose one or many concrete Sets without exposing filesystem complexity
inspect which audio files Ableton projects depend on
understand where those files live
detect external, missing, risky or shared dependencies
create safe self-contained project copies/packages
copy needed samples into planned locations
rewrite only supported references in ALS copies
validate changes before trusting them
keep manifests of operations
eventually maintain a safer sample library across many projects
```

The product must not:

```text
silently damage original projects
delete original samples
guess ambiguous matches
pretend a local parser result is a complete product guarantee
```

### 3.1 Core Domain

The core domain is:

```text
evidence-based dependency resolution and safe project recovery
```

The product advantage is not gzip/XML parsing. It is the ability to keep raw
facts, filesystem observations, identity evidence and user/system decisions
separate, then turn them into an explainable and safe recovery plan.

### 3.2 Required Domain Distinctions

```text
ReferenceOccurrence
  one occurrence extracted from an ALS document

RequiredAsset
  one logical audio requirement; many references may require the same asset

FileOccurrence
  one file observed at one native path, volume and scan time

ContentIdentity
  content supported by strong identity evidence
```

Rules:

```text
DependencyRef v0.1 is currently a ReferenceOccurrence compatibility contract.
It is not a RequiredAsset, FileOccurrence or ContentIdentity.

Path is location evidence, not identity.
File existence is availability evidence, not proof of the expected sample.
For packaging current project state, an exact existing active path may be
preserved as the current binding without claiming historical content identity.
Moved recovery candidates still require strong expected identity or explicit
source-bound user selection.
The parent of an ALS file is not automatically the Ableton Project root.
Facts, observations, evidence and decisions must not share one catch-all model.
```

Project catalog concepts are also distinct:

```text
ProjectFolder
  one physical filesystem container supported by Ableton Project structure
  evidence

LiveSet
  one concrete .als file observed at one native path and time

BackupSet
  one LiveSet observed under an Ableton Backup directory; hidden by default
  in the ordinary list but never discarded

ProjectWork
  a future logical musical work or version family supported by separate
  relationship evidence; it is not inferred by the scanner or physical catalog
```

## 4. Primary Use Cases

### UC0: Discover And Select Ableton Projects

User asks:

```text
Show me the Ableton projects on this computer so I can choose one or many Sets
to inspect or copy.
```

Required flow:

```text
obtain explicit local scan scope
-> observe ALS files and Project markers without parsing every ALS
-> build a physical ProjectFolder / LiveSet catalog
-> hide but retain Backup Sets
-> present a simple selectable list
-> create an explicit ProjectSelection for every selected LiveSet
```

The catalog must not silently infer that several Live Sets in one folder are
versions of the same musical work. Version-family analysis is a separate later
capability.

### UC1: Inspect Project Dependencies

User asks:

```text
What audio files does this Ableton project use?
Where are they?
Which ones are external, missing, local, Core Library or risky?
```

First module involved:

```text
ALSReader
```

Later modules involved:

```text
DependencyAssessment
PreflightReport
```

### UC2: Make Project Self-Contained / Collect Package

User asks:

```text
Create a safe copy/package of this project where used audio files live inside
the package/project folder.
```

Required flow:

```text
scan/read
-> classify dependencies
-> build plan
-> user reviews plan
-> copy/stage files
-> rewrite ALS copy
-> validate
-> write manifest
-> user verifies in Ableton
```

The same capability may be entered through more than one presentation surface.
The first compact entry is one explicit `.als` opened with ALS Rescue on macOS:

```text
Open With ALS Rescue
-> compact assistant near the cursor
-> choose destination parent
-> reuse the ordinary one-project preview and execute contracts
-> show complete, incomplete or unable outcome
```

This surface does not create a second copy policy, watch Finder selections or
replace Ableton as the default `.als` opener.

### UC3: Relocate Self-Contained Project

User asks:

```text
Move or duplicate an already self-contained project and make the copied ALS
point to the new location.
```

Required flow:

```text
read ALS
-> verify project-local files exist under new root
-> rewrite supported active paths in copy
-> semantic diff
-> manifest
-> Ableton-open verification
```

### UC4: Find Missing Or Moved Samples

User asks:

```text
Ableton cannot find files. Help me find the right files elsewhere on disk.
```

Required flow:

```text
read ALS refs
-> scan candidate folders
-> compare strong identifiers when available
-> use weak hints only as warnings
-> block ambiguous matches
-> ask user for explicit choice when needed
```

### UC5: Maintain A Safer Audio Library Later

User asks:

```text
Help me organize samples and know what can be moved, copied, kept or cleaned.
```

The product should also let the user browse a local CMDB-like catalog from both
directions: project to all referenced files, and file/content to every project
and Set snapshot that depends on it. A graph is one visualization of this
catalog; searchable tables, reverse references, evidence freshness and impact
preview are equally important.

Required flow:

```text
multi-project scan
-> content hashes
-> dependency/reachability graph
-> redirect/operation ledger
-> dry-run cleanup
-> quarantine/restore
```

## 5. Capability Map

```text
Observe ALS candidates and Project markers under approved roots
  -> ALSProjectScanner

Build a physical ProjectFolder / LiveSet / BackupSet catalog
  -> ProjectCatalogBuilder

Persist catalog snapshots and freshness
  -> ProjectCatalogStore

Expose catalog selection to the desktop
  -> ProjectCatalogApplicationService

Read ALS facts
  -> ALSReader

Extract reference occurrences
  -> DependencyExtractor

Discover Project root candidates
  -> ProjectDiscovery

Observe recorded path candidates
  -> PathObservation

Group occurrences into logical requirements
  -> DependencyAssessment

Bind audio still present at one recorded path without claiming identity
  -> CurrentPathBinding

Scan selected filesystem scopes
  -> AssetInventory

Resolve missing or moved assets
  -> AssetResolution

Plan package/rewrite operation
  -> PackagePlanner

Stage/copy files safely
  -> StagingExecutor

Rewrite supported ALS copies
  -> ALSRewriter

Validate semantic changes
  -> Validator

Record private operation history
  -> PrivateLedger

Export portable evidence
  -> PackageManifest

Promote a validated fresh package
  -> PackagePromoter

Compose the bounded laboratory flow
  -> LaboratoryPipeline

Run the existing one-project application flow for many explicit selections
  -> BatchCopyApplicationService

Hand one completed Project folder to a trusted browser destination
  -> ExternalFolderHandoff

Render a compact character without giving presentation ownership of work
  -> AssistantHost

Bind the latest completed Project folder to private native drag attempts
  -> TransferPayload
```

### 5.1 Current MVP Boundary

The current implementation target is one selected project and audio-only
recovery:

```text
read and report dependencies
scan explicitly selected local scopes
rank candidates from a complete inventory and select only with expected content
identity or an explicit user decision
build and inspect an immutable plan
create a fresh staged package
rewrite only a confirmed laboratory ALS profile
validate files and semantic XML difference
write private and portable manifests
promote to an absent target
require manual Ableton verification
```

Active next MVP increment:

```text
lightweight read-only discovery of ALS files under approved roots
physical ProjectFolder / LiveSet / BackupSet catalog
local persistent catalog with explicit scan coverage and freshness
simple project list and manual Add ALS fallback using one ProjectSelection
sequential multi-project preview and copy using the existing one-project flow
per-project isolation, manifests and aggregate batch reporting
```

Still outside the current MVP:

```text
whole-computer audio sample indexing and moved-file matching
continuous filesystem watching and automatic incremental refresh
parallel project copy execution
automatic Live Set version-family inference and semantic version diff
plugin, preset, Pack and Max for Live portability
cleanup or deletion
Live 9/10/12 and Windows rewrite support claims
```

Implemented write modules remain laboratory-only until the real Ableton runtime
gate and release hardening are complete.

Desktop Alpha is now inside the current MVP. It covers one selected project,
shows preflight facts, collects explicit user choices, invokes approved
application-service workflows and presents final evidence. It does not add new
matching, copy or rewrite policy.

### 5.2 Desktop Alpha Boundary

Required first vertical slice:

```text
choose one ALS
-> run read-only analysis through DesktopApplicationService
-> show project/dependency summary
-> save a local run report
```

Implemented write-capable desktop slice:

```text
use only audio that still exists at paths recorded by the selected ALS
-> report missing audio without searching for replacements
-> choose an absent output target
-> preview a complete or incomplete immutable copy plan
-> create and validate the copy after explicit confirmation
-> show whether the result is complete or still has missing files
```

Project catalog discovery and sequential batch copy are implemented Desktop
increments. They remain separate from whole-computer audio indexing: the first
pass observes ALS files and Project markers only, and performs deep ALS analysis
only for explicit user selections. One or many concrete Sets can be selected;
many Sets reuse the unchanged one-project pipeline sequentially. Moved-file
matching and whole-computer audio candidate indexing remain a later product
stage.

The compact quick surface may also expose the exact latest successful Project
folder as a one-shot native copy drag after an explicit provider action. This
handoff does not upload data itself, automate the browser, create archives or
claim that a remote transfer completed.

The compact surface is being refactored into three explicit responsibilities
without changing the one-project copy policy:

```text
QuickCopy job
  owns destination, preview, execution and copy outcome

TransferPayload
  privately owns the eligible completed Project folder and drag attempts

AssistantHost
  renders speech, character, controls and interaction geometry
```

This foundation still has one courier and one `wetransfer_web` action. Provider
catalogs, inbound ALS drop, visible delivered/reload behavior, character
catalogs, multiple visible characters and licensing are later increments.

Diagnostic exports are local and user-initiated. A shareable export must omit
ALS bytes, audio bytes and private absolute paths by default. The unredacted
private ledger remains local unless the user deliberately chooses otherwise.

## 6. Gate Statuses

Each gate must be marked:

```text
CLEAR
  Enough information exists to continue.

PARTIAL
  Enough information exists only for a narrowed scope.

AMBIGUOUS
  Multiple meanings are plausible. AI would need to guess.

CONFLICTING
  Existing docs or decisions disagree.

UNKNOWN
  Required information is missing.
```

Rule:

```text
AMBIGUOUS, CONFLICTING or important UNKNOWN blocks spec/build.
```

Allowed exception:

```text
The module may continue only if its scope is explicitly narrowed,
and the limitation is written in the spec.
```

## 7. Product Gates

### Vision Gate

Question:

```text
Which product promise does this work support?
```

Blocks if:

```text
the module is useful locally but has no clear product promise parent
```

### Use Case Gate

Question:

```text
Which user use case does this support?
```

Blocks if:

```text
the feature could mean different things to the user
and Codex would need to guess the intended workflow
```

### Flow Gate

Question:

```text
Where does this module sit in the end-to-end operation?
```

Blocks if:

```text
the module output cannot be placed in scan -> plan -> apply -> validate -> manifest
or another named flow
```

### Data Gate

Question:

```text
Which data must this module produce, preserve or explicitly not own?
```

Blocks if:

```text
downstream modules need data that the spec ignores or discards
```

### Behavior Gate

Question:

```text
What does the module do, refuse, warn about and return on failure?
```

Blocks if:

```text
successful behavior is clear but refusal/error behavior is vague
```

### Decision Gate

Question:

```text
Where does the user need to approve, choose, reject or resolve ambiguity?
```

Blocks if:

```text
the AI or code would silently choose between product-significant alternatives
```

### Safety Gate

Question:

```text
What must never be touched, changed, deleted or inferred?
```

Blocks if:

```text
the module could enable unsafe later operations without marking risk
```

### Audit / Manifest Gate

Question:

```text
What must be recorded so the user can later understand what happened?
```

Blocks if:

```text
the operation affects files or references and no manifest/audit plan exists
```

### Reversibility Gate

Question:

```text
Can this operation be safely retried, rolled back, or verified before trust?
```

Blocks if:

```text
the operation is destructive or hard to inspect and has no rollback/validation story
```

### Batch Gate

Question:

```text
Would this logic still make sense across many projects?
```

Blocks if:

```text
a one-project shortcut would corrupt or confuse multi-project behavior later
```

### Evidence Gate

Question:

```text
How do we prove this works?
```

Blocks if:

```text
there are no fixtures, experiments, semantic diffs, tests or manual verification plan
```

### Module Gate

Question:

```text
Is this small module a traceable part of a larger capability?
```

Blocks if:

```text
input/output/tests are clear,
but the module is not traceable to a product capability and use case
```

## 8. Clarification Request

When a gate is AMBIGUOUS, CONFLICTING or important UNKNOWN, Codex must return to
the Product Owner with a specific request.

Format:

```text
Clarification Request

Gate:
  <gate name>

Status:
  AMBIGUOUS / CONFLICTING / UNKNOWN

Problem:
  What is unclear?

Risk:
  What could go wrong if Codex guesses?

Needed from Product Owner:
  1. Specific question
  2. Specific question
  3. Specific question
```

Do not ask:

```text
Please clarify the product.
```

Ask:

```text
For Mass Collect, should the output be a new project folder, a central library
entry, or both?
```

## 9. Module Spec Requirements

Every product module spec must include:

```text
Parent Product Capability
Supported Use Case
Operation Flow Position
Downstream Consumers
Contract Layers
Gate Status Summary
Non-Guessing Questions
Traceability
```

Minimum traceability format:

```text
Product Promise
-> Use Case
-> Capability
-> Flow Step
-> Module Responsibility
-> Output / Behavior
-> Test / Evidence
```

## 10. Definition Of Ready For Module Build

A module may enter Build Mode only when:

```text
spec.md exists
plan.md exists
tasks.md exists
fixture/test/evidence plan exists
module.contract.json exists
module.contract.json references active ENGINEERING_RULES.md version
workflow_guard module-ready passes
Gate Status Summary has no blocking AMBIGUOUS / CONFLICTING / important UNKNOWN
scope limitations are explicit
the module is traceable to a product capability
```

## 11. Definition Of Done For Module Acceptance

A module is accepted only when:

```text
tests pass
implemented behavior matches spec
engineering quality obligations are satisfied
workflow_guard verify-module passes
scope limitations are still true
result can be explained in product terms
downstream contract status is known
CURRENT_STATE is updated
```

If downstream contract is only PARTIAL, the module can be accepted only as a
limited slice, not as a full foundation for later operations.

## 12. Retrospective: ALSReader v0.1

ALSReader v0.1 may be accepted as:

```text
read-only diagnostic reader
```

It must not yet be accepted as:

```text
complete data contract for ProjectAnalyzer / PackagePlanner / ALSRewriter
```

Retrospective gate result:

```text
Vision Gate: PARTIAL
Use Case Gate: AMBIGUOUS
Flow Gate: AMBIGUOUS
Data Gate: PARTIAL
Behavior Gate: CLEAR
Safety Gate: CLEAR
Decision Gate: PARTIAL / N/A
Audit / Manifest Gate: UNKNOWN
Reversibility Gate: N/A for read-only
Batch Gate: UNKNOWN
Evidence Gate: CLEAR locally
Module Gate: PARTIAL
```

Required next review:

```text
ALSReader Contract Review
```

Question:

```text
Is ALSReader JSON only a user diagnostic report,
or also the first inter-module contract for package/rewrite/missing recovery?
```
