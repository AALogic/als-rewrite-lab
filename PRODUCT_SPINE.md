# Product Spine

Status: active product spine  
Date: 2026-06-02  
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
The parent of an ALS file is not automatically the Ableton Project root.
Facts, observations, evidence and decisions must not share one catch-all model.
```

## 4. Primary Use Cases

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

Still outside the current MVP:

```text
automatic full-disk UX and persistent incremental index
batch execution and resume
desktop UI
plugin, preset, Pack and Max for Live portability
cleanup or deletion
Live 9/10/12 and Windows rewrite support claims
```

Implemented write modules remain laboratory-only until the real Ableton runtime
gate and release hardening are complete.

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
