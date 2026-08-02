# Product Specification: Ableton Project Rescue

Status: long-form product reference; not an active implementation source
Date: 2026-07-27
Scope: full product specification, not MVP-only
Primary platform: local desktop app, macOS first, platform-portable core
Data policy: local-only, no cloud requirement

Active routing:

```text
CURRENT_STATE.md owns the current project position.
PRODUCT_SPINE.md owns the active product/domain direction and MVP boundary.
The one active module spec owns implementation behavior.
```

This document preserves the wider product vision and historical detail. Where
it conflicts with the active routing above, it must be corrected through a
separate product review and must not silently drive implementation.

## 1. Product Definition

Ableton Project Rescue is a local desktop application for Ableton Live users.

The product scans the user's computer, finds Ableton Live projects, analyzes
their dependencies, detects missing or risky files, searches for missing samples
on the local disk, builds a safe relocation/package plan, and creates a new
portable Ableton project copy without modifying original projects.

The product is not just an ALS rewriter.

It is a local dependency manager and packaging/recovery tool for Ableton Live
projects.

Core promise:

```text
Help a music producer understand, recover and safely package Ableton projects
whose audio dependencies are spread across messy local folders.
```

## 2. Target Users

Primary users:

```text
Music producer with old and current Ableton projects spread across a local disk.
Producer moving projects from one computer to another.
Producer who downloads samples into Downloads, Splice folders, desktop folders,
old project folders and ad hoc locations.
Producer who wants to send a project to another person without missing samples.
Producer who wants to audit which projects are safe and which are broken.
```

Secondary future users:

```text
Studio engineer maintaining many client projects.
Producer migrating from Windows to macOS.
Producer upgrading from old Ableton versions to newer Ableton versions.
Collaborator receiving project archives from other users.
```

## 3. Main Product Outcomes

The product should let the user achieve these outcomes:

```text
1. See all detected Ableton projects on the computer.
2. Select one or more projects for analysis.
3. See which audio files each project depends on.
4. See whether those files still exist in expected locations.
5. See which dependencies are local, external, missing, User Library, Core
   Library, Factory Pack or risky.
6. Search the whole local computer for missing samples.
7. Match missing samples to candidate files using evidence and confidence
   scoring.
8. Build a plan for creating a portable project copy.
9. Copy accepted dependencies into a new Ableton-like project folder.
10. Rewrite only the copied ALS file so it points to the new local dependency
    locations.
11. Validate that the package was created safely.
12. Produce a manifest explaining what happened.
13. Let the user manually open the new project in Ableton for final verification.
```

## 4. Non-Negotiable Safety Rules

The product must never silently damage original data.

Hard rules:

```text
Never overwrite an original ALS file.
Never rewrite an original ALS file.
Never delete original audio files.
Never delete original project folders.
Never choose between ambiguous sample matches without policy support.
Never claim that a package is Ableton-verified unless the user actually opens
and verifies it in Ableton.
Never use global text search/replace as ALS rewrite strategy.
Never treat Ableton OriginalCrc as sole proof of file identity.
```

Safe default:

```text
Read originals.
Copy originals when needed.
Modify only working copies / staged package copies.
Validate before presenting success.
Keep a manifest.
```

## 5. Key Concepts

### 5.1 Ableton Live Set

An Ableton Live Set is a `.als` file.

Current known fact:

```text
.als is gzip-compressed XML.
```

The product reads `.als` files by decompressing gzip and parsing XML.

### 5.2 Ableton Project

An Ableton Project is a folder that contains one or more `.als` files and may
contain subfolders such as:

```text
Samples/
Backup/
Ableton Project Info/
Presets/
```

The product must distinguish between:

```text
main project ALS
backup ALS
template ALS
orphan ALS outside a normal project folder
```

### 5.3 Main ALS

The main ALS is the set that should appear in the user's project list.

Backup ALS files should be hidden by default but recorded as related metadata.

### 5.4 Backup ALS

An ALS is probably a backup if:

```text
it lives inside a folder named Backup or Backups
or it follows Ableton backup naming patterns
```

Backups are not discarded. They may be useful later for:

```text
recovering older versions
finding when a sample was still available
comparing dependency history
```

### 5.5 Audio Dependency

An audio dependency is an audio file actively referenced by a Live Set.

Examples:

```text
wav
aif
aiff
flac
mp3
ogg
```

The current product focuses on audio dependencies first.

### 5.6 Preset Dependency

Preset dependency means an Ableton preset, rack, Max for Live device, or plugin
preset file that may be needed to recreate project behavior.

Current working rule:

```text
Ableton/User presets should be copied when referenced and locatable.
Max for Live devices need a dedicated test.
Third-party VST/AU plugin binaries are not copied.
Third-party plugin preset portability is not yet proven.
```

### 5.7 Plugin Dependency

Plugin dependency means a third-party plugin used by the project.

The product should report plugin information when available:

```text
plugin name
plugin format, if detectable
plugin identifier, if detectable
plugin version, if detectable
track/device context, if detectable
```

The product does not install or copy plugin binaries.

### 5.8 Core Library / Factory Packs

Core Library and Factory Packs are Ableton-provided content.

Current working rule:

```text
Treat Core Library / Factory Pack files as system_dependency by default.
Do not copy them by default.
Mark them as portable_risk if target computer/version/packs are unknown.
Future strict portable mode may collect factory pack audio if user requests it.
```

This requires more Ableton-version testing.

### 5.9 User Library

User Library is user-specific content.

Current working rule:

```text
Copy User Library audio/preset dependencies into portable packages by default.
```

Reason:

```text
Another computer probably does not have the same User Library.
```

### 5.10 Manifest

A manifest is a structured record of a package/rewrite operation.

The manifest is required.

It is used for:

```text
debugging
rollback reasoning
support
auditability
future project index
future safe cleanup
explaining what the tool did
```

The manifest does not need to expose product algorithms. It records operation
facts and decisions.

## 5A. Architecture Principles

These principles exist to prevent product-level intent from turning into
tightly coupled or unsafe code.

Core architecture rules:

```text
One module should have one primary responsibility.
Modules communicate through explicit public contracts.
Modules must not depend on another module's private internal details.
Storage is an adapter, not the owner of domain logic.
The UI must not perform dependency analysis, copy planning or ALS rewrite logic.
No write operation may bypass PackagePlan.
No ALS rewrite may bypass validation.
No module may modify original user data unless a future spec explicitly allows
that operation with stronger safety rules.
```

Platform portability guardrail:

```text
Build and test on macOS first.
Do not implement full Windows migration in MVP.
Do not hardcode macOS-only assumptions into core domain modules unless the
module spec explicitly marks them as platform-specific.
Preserve raw ALS path values exactly as data.
Resolve filesystem paths through explicit path/filesystem modules or adapters.
Keep platform-specific behavior out of ALSReader, DependencyExtractor,
SampleMatcher, PackagePlanner and ALSRewriter unless a contract requires it.
When platform behavior is unknown, mark it UNKNOWN / requires_test instead of
guessing.
```

Meaning:

```text
Windows support is not promised for MVP.
Windows support must not be accidentally blocked by avoidable architecture
choices.
```

Public contracts:

```text
Every public module output must have a named data contract.
Every named data contract should have a version once it is used by downstream
modules.
Contract changes must be treated as product/technical decisions, not accidental
refactors.
```

Initial contract names:

```text
ProjectRecord v0.1
ALSReadModel v0.2
DependencyRef v0.1
AssetRecord v0.1
MatchCandidate v0.1
PackagePlan v0.1
CopyOperation v0.1
RewriteOperation v0.1
Manifest v0.1
```

## 5B. Module Boundary Table

This table defines product-level module boundaries. Each module still needs its
own detailed module specification before implementation.

| Module | Primary Responsibility | Input | Output | Does Not Do |
| --- | --- | --- | --- | --- |
| PermissionAndScanScope | Explain scan scope and obtain user permission. | User request, platform permissions. | Approved scan scope. | Does not scan files or analyze projects. |
| FullDiskScanner | Walk approved local locations and find candidate files. | Approved scan scope. | Raw file candidates and scan events. | Does not decide project grouping or dependency meaning. |
| ProjectDiscovery | Detect and group Ableton projects from `.als` files. | Raw `.als` candidates and filesystem context. | `ProjectRecord` list. | Does not parse ALS internals or analyze dependencies. |
| ProjectRegistry | Persist discovered project metadata. | `ProjectRecord` updates. | Stored project registry records. | Does not own discovery rules or package decisions. |
| ALSReader | Read ALS bytes and extract raw active reference facts. | `.als` path or bytes, project root context. | `ALSReadModel` / raw dependency facts. | Does not classify storage policy, match files, copy, rewrite or validate packages. |
| DependencyExtractor | Convert raw ALS facts into dependency records. | `ALSReadModel`. | `DependencyRef` list. | Does not check disk existence or decide package actions. |
| PathVerifier | Check whether referenced paths resolve to existing files. | `DependencyRef`, project root, filesystem adapter. | Updated dependency existence state. | Does not classify product risk beyond existence facts. |
| DependencyClassifier | Classify dependency source and risk. | Verified `DependencyRef` records. | Classified dependency records and risk flags. | Does not search for missing files or copy anything. |
| PreflightReportBuilder | Build the user-facing dependency report. | Classified dependencies, project status facts. | Preflight report. | Does not mutate project state or perform package operations. |
| AssetIndexer | Build and update the local asset index. | File candidates and metadata readers. | `AssetRecord` index. | Does not decide whether an asset should be used for a project. |
| SampleMatcher | Match missing dependencies to indexed assets. | Missing `DependencyRef`, `AssetRecord` index. | `MatchCandidate` list. | Does not copy files or rewrite ALS; ambiguous decisions remain explicit. |
| PackagePlanner | Build an inspectable package plan. | Dependencies, matches, target location, user choices. | `PackagePlan`. | Does not execute copies or rewrite ALS. |
| CopyStager | Copy accepted files into the target package/staging area. | Accepted `PackagePlan`. | Completed `CopyOperation` results. | Does not decide what should be copied and does not rewrite ALS. |
| ALSRewriter | Rewrite only the copied ALS according to approved operations. | Copied ALS, accepted `RewriteOperation` list. | Rewritten copied ALS. | Does not rewrite originals, match samples, copy files or modify unrelated XML. |
| Validator / SemanticDiff | Verify copied files, rewritten ALS and package integrity. | Package output, manifest draft, original hashes. | Validation result. | Does not fix errors silently or change the package. |
| ManifestWriter | Persist operation evidence and decisions. | Plan, copy results, rewrite results, validation result. | `Manifest`. | Does not make domain decisions after the fact. |
| BatchRunner | Orchestrate the same workflow for many projects later. | Project list and per-project workflow inputs. | Per-project results and aggregate report. | Does not introduce separate business rules for batch mode. |

Preflight is a product stage, not a single large domain module.

Preflight is composed from:

```text
PathVerifier
DependencyClassifier
PreflightReportBuilder
```

## 5C. Contract And Encapsulation Rules

Module internals are not public API.

Examples:

```text
ALSReader may parse XML internally, but downstream modules should consume
ALSReadModel / DependencyRef contracts, not arbitrary internal parser state.

AssetIndexer may use SQLite internally, but domain modules should not depend on
raw table shapes or SQL queries.

ALSRewriter may use XML locators internally, but only locators included in the
public rewrite contract may be relied on by other modules.
```

Storage rule:

```text
SQLite stores facts and indexes.
SQLite does not decide package policy.
SQLite does not own scoring logic.
SQLite does not own rewrite rules.
Domain logic must live in testable modules above storage adapters.
```

UI rule:

```text
UI may display reports, collect user choices and trigger approved workflows.
UI must not classify dependencies, match samples, decide copy operations or
rewrite ALS directly.
```

Refactor rule:

```text
Internal code may be refactored when public contracts and tests remain stable.
Changing a public contract is not a refactor; it is a contract change and must
update module specs, tests and downstream consumers.
```

Minimal handoff rule:

```text
Each producer module may collect more facts than the next module currently
needs, but each consumer module must declare its minimal required input fields.

Downstream modules must not depend on fields collected "for later", diagnostic
fields, raw evidence fields or UNKNOWN/HYPOTHESIS fields unless their module
spec explicitly promotes those fields into its input contract.

If a downstream module starts using a previously diagnostic field, that is a
contract change. The change must update:
  producer module spec
  consumer module spec
  contract tests
  downstream acceptance criteria
```

Field maturity labels:

```text
core_handoff:
  Minimal field required by the next module's current contract.

supporting_evidence:
  Field may help scoring, reports or manual review, but must not be required
  for the next module to function unless explicitly promoted.

diagnostic_only:
  Field exists for debugging, reporting or exploratory research.
  Downstream product logic must not depend on it.

preserve_for_future:
  Field is intentionally retained to avoid losing ALS information, but no
  current module may require it.

unknown_semantics:
  Field is observed but not understood enough to drive behavior.
```

## 5D. Testing And Uncertainty Strategy

The product contains both normal software uncertainty and Ableton-domain
uncertainty. These must be tracked separately from confirmed behavior.

Evidence statuses:

```text
UNKNOWN
  Required information is missing.

HYPOTHESIS
  Plausible behavior that has not yet been proven.

CONFIRMED
  Behavior supported by fixture, domain experiment, manual Ableton verification,
  official documentation or repeatable test.

REJECTED
  Hypothesis contradicted by test, documentation or observed behavior.
```

Rule:

```text
Only CONFIRMED behavior may become an automatic rewrite/copy rule.
UNKNOWN or HYPOTHESIS behavior may be reported, tested or blocked, but must not
be silently used as a destructive or irreversible operation rule.
```

Minimum module test strategy:

```text
Unit tests:
  pure logic, scoring, classification and path handling.

Fixture tests:
  known ALS files, known dependency counts, known parser outputs.

Contract tests:
  public input/output model compatibility for downstream modules.

Safety tests:
  original files are not modified, invalid paths fail safely, rewrite cannot
  target the original ALS.

Integration tests:
  module-to-module flow for selected slices, such as ALSReader -> Preflight.

Domain/Ableton tests:
  manual or semi-automated before/after Ableton experiments for behavior that
  cannot be inferred safely from XML alone.
```

Each module specification must declare:

```text
required fixtures
required safety tests
contract tests
known unknowns
downstream consumers
acceptance criteria
```

## 5E. Runtime Safety And Code Quality Checklist

These rules protect the implementation from becoming hard to debug, unsafe to
run or difficult to refactor.

They apply to every module specification and every implementation task.

Detailed engineering rules live in `ENGINEERING_RULES.md`. This product spec
defines product-level safety and architecture intent. `ENGINEERING_RULES.md`
defines the practical implementation standard that Codex must apply while
designing, coding, testing and refactoring modules.

### 5E.1 Determinism

The same input, the same ruleset version and the same indexed filesystem state
should produce the same output.

Required behavior:

```text
same ALS + same ProjectRecord + same AssetIndex + same ruleset
  -> same DependencyRef set
  -> same MatchCandidate scores
  -> same PackagePlan
  -> same RewriteOperation list
  -> same Manifest content except timestamps/run ids
```

If a module uses nondeterministic ordering, random ids or filesystem traversal
order, it must normalize output before returning public results.

### 5E.2 Idempotency

Repeating the same safe operation should not duplicate files, corrupt packages
or produce incompatible output.

Rules:

```text
Re-running scan should update the index, not duplicate project records.
Re-running package planning should produce the same plan for the same inputs.
Re-running copy staging should detect already-copied matching files.
Re-running validation should not mutate package contents.
ALS rewrite should not stack repeated path rewrites on already-rewritten output.
```

When true idempotency is not possible, the module must return an explicit status
such as:

```text
already_exists
already_packaged
requires_new_target
operation_not_repeatable
```

### 5E.3 Atomic Write

Generated files must not be written in a way that can leave a corrupted final
artifact after interruption.

Rules:

```text
Never write directly to the final ALS path.
Write generated ALS bytes to a temporary path first.
Validate gzip/XML and semantic diff before promotion.
Promote temporary file to final target path only after validation.
If validation fails, mark the artifact failed and keep or remove it according to
debug policy.
Original ALS remains untouched in all cases.
```

This applies especially to:

```text
rewritten ALS files
manifest files
local index snapshots
package plan exports
```

### 5E.4 Staging Completion Policy

A target package must not look finished until it passes validation.

Rules:

```text
Package work happens in a staging state.
Validation decides whether the package becomes ready.
Partial packages must be marked partial/failed.
The UI must not present partial packages as complete.
```

Recommended package statuses:

```text
staging
copy_failed
rewrite_failed
validation_failed
validation_passed
ready_for_manual_ableton_check
user_verified
user_rejected
```

### 5E.5 Error Taxonomy

Expected failure modes must have explicit codes and meanings.

Initial error codes:

```text
PERMISSION_DENIED
SCAN_INTERRUPTED
PROJECT_DISCOVERY_FAILED
ALS_NOT_FOUND
ALS_NOT_GZIP
ALS_XML_INVALID
ALS_UNSUPPORTED_STRUCTURE
ACTIVE_FILEREF_NOT_FOUND
DEPENDENCY_PATH_UNRESOLVED
SAMPLE_NOT_FOUND
AMBIGUOUS_MATCH
LOW_CONFIDENCE_MATCH
TARGET_NOT_WRITABLE
TARGET_EQUALS_SOURCE
COPY_FAILED
COPY_SIZE_MISMATCH
COPY_HASH_MISMATCH
REWRITE_PLAN_INVALID
REWRITE_FAILED
SEMANTIC_DIFF_FAILED
VALIDATION_FAILED
ORIGINAL_FILE_CHANGED
MANIFEST_WRITE_FAILED
INDEX_READ_FAILED
INDEX_WRITE_FAILED
```

Rules:

```text
Errors should be structured, not only free text.
User-facing messages may be friendly, but internal errors need stable codes.
Modules should return known errors instead of panicking on expected bad input.
Unexpected errors should preserve enough context for debugging without exposing
private user data unnecessarily.
```

### 5E.6 Input And Output Contract Validation

Each module should validate its important inputs and outputs.

Examples:

```text
ALSReader rejects non-gzip input.
DependencyExtractor rejects missing ALSReadModel fields required downstream.
PackagePlanner rejects target paths equal to source paths.
CopyStager rejects plans with unresolved required dependencies.
ALSRewriter rejects rewrite operations without XML locators.
Validator rejects manifests that do not match actual package results.
```

Rule:

```text
Invalid input should fail early with a structured error.
Invalid output should fail validation before downstream modules consume it.
```

### 5E.7 Regression And Golden Files

Confirmed ALS behavior must be protected by repeatable tests.

Rules:

```text
Keep fixture ALS files for known cases.
Keep expected outputs or expected counts for fixture tests.
When a bug is fixed, add a regression test if practical.
Before changing parser/rewrite behavior, run the existing fixture suite.
Do not update golden expectations unless the product/spec contract intentionally
changed.
```

Golden tests should cover:

```text
ALS parsing counts
RelativePathType handling
active vs historical refs
path classification
match scoring
package plan generation
semantic diff expectations
```

### 5E.8 Structured Logging And Pipeline Events

The system should produce enough structured events to debug workflow behavior.

Important event types:

```text
scan_started
scan_completed
project_discovered
als_read_started
als_read_completed
dependency_extracted
path_verified
preflight_completed
index_updated
match_candidate_generated
package_plan_created
copy_started
copy_completed
rewrite_started
rewrite_completed
validation_completed
manifest_written
operation_blocked
operation_failed
user_decision_recorded
```

Rules:

```text
Logs should identify pipeline stage, project id and operation id.
Logs should not expose private paths in exported support bundles unless the user
explicitly chooses to include them.
Manifest records final facts; logs explain process behavior.
```

### 5E.9 Performance Boundaries

The product must remain usable on messy real computers.

Design assumptions to preserve:

```text
one project can have hundreds or thousands of dependencies
one computer can contain many Ableton projects
one sample library can contain tens or hundreds of thousands of files
hashing every file eagerly can be too expensive
full disk scan can be long-running and interruptible
```

Rules:

```text
Indexing should be incremental.
Expensive hashes should be lazy or prioritized.
Large scans should be resumable or safely restartable.
UI should show progress for long operations.
Module specs should state expected input sizes when relevant.
```

### 5E.10 Privacy And Local-Only Behavior

The product handles private project names, file paths and audio metadata.

Rules:

```text
Default operation is local-only.
Do not upload project paths, sample names, ALS contents, manifests or logs by
default.
Support/debug exports must be user-initiated.
Support/debug exports should offer path anonymization.
```

### 5E.11 Platform Portability

The product is macOS-first, but core code should remain portable unless a module
is explicitly platform-specific.

Rules:

```text
Do not hardcode `/Users/...` as a rule; treat it only as observed data.
Do not assume `/` path separators when constructing new paths.
Do not assume drive letters, volumes or case sensitivity behavior without a
platform-specific test.
Do not normalize raw ALS paths before preserving the original value.
Do not mix path reading, path resolution and path rewrite policy in one module.
Use path/filesystem adapters for platform-specific resolution and permissions.
Keep Windows migration in Later/Research, but keep core contracts able to
represent Windows-style paths.
```

Required module-spec question:

```text
Does this module introduce any platform-specific assumption?
If yes, is it isolated, tested and documented?
If no, does the implementation avoid blocking future macOS/Windows support?
```

### 5E.12 Scope Classification

Every substantial product or engineering change should be classified before
implementation.

Allowed classifications:

```text
MVP
Next
Later
Research
Parking Lot
Rejected
```

Rule:

```text
If a change cannot be classified, do not implement it yet.
Clarify whether it belongs to the current module, a later module or research.
```

### 5E.13 Decision Documentation

Important decisions should be documented where future implementation work can
find them.

Examples:

```text
rewrite safety decisions
matching confidence thresholds
Core Library / User Library handling
package folder structure
contract version changes
error taxonomy changes
storage model decisions
```

Rule:

```text
Do not leave major product/architecture decisions only in chat history.
Promote them into product spec, module spec, backlog, manifest schema or a
future decision log depending on permanence and scope.
```

## 6. Full Product Workflow

### 6.1 Install And Permission

The user installs the application locally.

The application explains that it will scan the local computer to find:

```text
Ableton projects
ALS files
audio files
presets
potential dependencies
```

The user grants permission for broad/full disk scanning.

The application must be clear that:

```text
scan is local
data is not uploaded by default
original projects are not modified
scan may take time
index can be reused later
```

### 6.2 Full Disk Scan

The product performs a broad local scan.

The scan should include:

```text
user home folder
Documents
Desktop
Downloads
Music
Ableton User Library locations
external drives available to the system
cloud folders that are locally visible
custom locations if user adds them later
```

The scan may skip technical/system-heavy folders by default for performance and
safety, for example:

```text
system folders
application bundles unless needed for Ableton Core detection
build folders
node_modules
Trash
temporary caches
```

If the user explicitly enables an advanced exhaustive scan, the product can
include more locations.

### 6.3 Project Discovery

The product finds `.als` files and groups them into projects.

For each discovered project, store:

```text
project_id
project_name
project_root_path
main_als_path
all_als_paths
backup_als_paths
latest_modified_time
ableton_project_info_present
samples_folder_present
backup_folder_present
project_size_estimate
discovery_confidence
```

The user-facing project list should show grouped projects, not every raw ALS.

Backups are hidden by default but counted and available in details.

### 6.4 User Selects Project

The user selects one or more projects for analysis.

Product goal supports many projects, but first high-quality workflow should
handle one selected project very well.

Future multi-project mode should reuse:

```text
same project registry
same asset index
same dependency extraction
same matcher
same planner
batch orchestration
```

### 6.5 Working Copy / Analysis Snapshot

Before parsing or rewriting:

```text
original ALS path is recorded
original ALS file hash is recorded
working analysis copy may be created
original file remains unchanged
```

Read-only analysis may read original bytes, but no operation is allowed to
write to the original path.

Rewrite operations always write a new copied ALS.

### 6.6 ALS Reading

The ALSReader:

```text
opens ALS bytes
validates gzip
decompresses XML
parses XML
finds active SampleRef/FileRef references
preserves raw path fields
preserves RelativePathType values exactly
extracts metadata useful for matching
extracts context useful for later rewrite
counts historical OriginalFileRef separately
returns structured read model
does not classify storage policy
does not copy files
does not rewrite files
```

Minimum active audio reference fields:

```text
ref_id
raw_path
raw_relative_path
relative_path_type
file_type
filename
extension
original_file_size
original_crc
default_duration
default_sample_rate
usage_context
xml_context
xml_locator
```

Important note:

```text
OriginalCrc is useful as a weak signal but not enough as sole identity proof.
```

### 6.7 Dependency Extraction

Dependency extraction turns raw ALS facts into dependency records.

Dependency categories:

```text
audio_sample
video_file
ableton_preset
max_for_live_device
plugin_dependency
core_library_dependency
factory_pack_dependency
user_library_dependency
unknown_dependency
```

Current product handling:

```text
audio_sample:
  managed and copied

user_library audio/preset:
  copied by default for portable package

core_library / factory_pack:
  report as system dependency by default

VST/AU plugin:
  report only

Max for Live:
  investigate and support later according to CAS behavior
```

### 6.8 Path Verification

For every active dependency path:

```text
resolve path according to ALS path fields and project root
check whether file exists
if exists, collect current file metadata
compare current file metadata against ALS-provided metadata when possible
classify state
```

Possible dependency states:

```text
exists_expected_location
exists_but_metadata_mismatch
missing
external
project_local
user_library
core_library
factory_pack
unknown_location
ambiguous_resolution
```

### 6.9 Preflight Report

Preflight is the first major user-facing report.

It answers:

```text
Can this project be packaged now?
Which files are present?
Which files are missing?
Which dependencies are external?
Which dependencies are system dependencies?
Which dependencies require user attention?
Which plugin/preset warnings exist?
```

Project-level statuses:

```text
READY_TO_PACKAGE
  All required managed dependencies exist or have accepted matches.

MISSING_SAMPLES
  One or more required audio samples are missing.

NEEDS_USER_DECISION
  The project has ambiguous matches, filename collisions, plugin/preset warnings
  or package decisions.

PACKAGED_VALIDATION_PASS
  Package was created and technical validation passed.

PACKAGED_NEEDS_ABLETON_CHECK
  Package is technically valid but must be opened manually in Ableton.

BLOCKED
  The operation cannot continue safely.
```

### 6.10 Asset Index

The product maintains a local asset index.

Preferred storage:

```text
SQLite database
```

The index is local-only.

Asset records should include:

```text
asset_id
path
filename
extension
file_kind
size_bytes
mtime
hash_full, when computed
hash_partial, optional for fast prefiltering
duration, if audio metadata can be read
sample_rate, if audio metadata can be read
channels, if audio metadata can be read
bit_depth, if audio metadata can be read
scan_time
volume_id, if available
availability_status
```

The index supports:

```text
missing sample search
future multi-project analysis
future duplicate detection
future safe cleanup
future dependency graph
```

Indexing should be incremental:

```text
do not rescan and rehash unchanged files unnecessarily
use path + size + mtime as quick change detector
compute expensive hashes lazily or when needed
```

### 6.11 Missing Sample Search

If a dependency is missing, the product searches the asset index and, if needed,
the disk for candidates.

Candidate matching signals:

```text
full hash match
file size match
duration match
sample rate match
channel count match
filename exact match
filename normalized match
extension match
folder/path similarity
Ableton OriginalCrc weak signal
```

Confidence scoring:

```text
100:
  known full hash match from previous manifest/index evidence

95-99:
  very strong match from size + duration + sample rate + filename/context

70-94:
  plausible match requiring user review

50-69:
  weak diagnostic candidate

<50:
  do not present as main candidate by default
```

Current rule:

```text
score >= 95 can be auto-selected by the system
but the full package plan is still shown to the user before execution
```

If multiple candidates have similar high scores:

```text
do not silently choose
require user decision or mark NEEDS_USER_DECISION
```

### 6.12 Package Planning

The planner creates a plan before any copy or rewrite.

The plan contains:

```text
source project
target project folder
source ALS
target ALS
dependency list
copy operations
rewrite operations
dependencies not copied
system dependencies
plugin warnings
preset warnings
missing dependencies
filename collisions
rescue candidate files
validation steps
estimated storage usage
```

The planner must be inspectable by the user.

No package operation starts before the plan exists.

### 6.13 Target Folder Structure

Goal:

```text
Create a project folder that Ableton can open without searching across many old
local paths.
```

Default target structure:

```text
Target Project Name Project/
  Target Project Name.als
  Ableton Project Info/
  Samples/
    Collected/
    Processed/
    Recorded/
  Presets/
  Rescue Manifest/
```

Rules:

```text
Preserve existing project-local files when copying an existing project.
Copy external accepted audio dependencies into an Ableton-like Samples area.
Copy User Library audio/preset dependencies into the package by default.
Keep alternative rescue candidates outside active ALS references.
```

Recommended rescue candidate location:

```text
Target Project Name Rescue/
```

Reason:

```text
Alternative candidates should not pollute the active Ableton project folder or
become accidental unused project files.
```

### 6.14 Filename Collisions

Filename collision example:

```text
/Downloads/kick.wav
/Sample Packs/Techno/kick.wav
```

These may be completely different audio files.

Rules:

```text
Never overwrite a copied file because another file has the same filename.
Never assume same filename means same audio.
Generate unique target paths when needed.
Record old path -> new path mapping in manifest.
Rewrite ALS to the exact generated target path.
```

Temporary rule until Ableton CAS collision behavior is tested:

```text
Use deterministic unique names or collision-safe subfolders.
```

### 6.15 Copy Staging

CopyStager performs the package copy.

Current desktop MVP rule:

```text
audio still present at its recorded path is a current binding, not proven
historical content identity
do not compute SHA-256 for current-path audio
copy it once through a create-new temporary file
verify stable source metadata and counted byte size during that copy
record hash as not_computed in the manifest
introduce reusable audio hashes only with the persistent SQLite index
```

Rules:

```text
copy into a staging/target location
never copy over originals
verify copied bytes by size and hash where possible
copy .asd sidecar files when present next to copied audio
record every copy operation
fail safely on partial copy errors
```

`.asd` sidecar policy:

```text
Copy adjacent .asd files when they exist and correspond to copied audio.
If missing, do not block package; Ableton can often recreate analysis data.
```

### 6.16 ALS Rewrite

ALSRewriter modifies only copied ALS files.

Rules:

```text
input ALS must not equal output ALS
rewrite requires an accepted package plan
rewrite only supported active FileRef nodes
rewrite only explicitly listed fields
do not modify historical OriginalFileRef in initial versions
do not modify plugin state
do not modify unrelated XML
do not use unrestricted global search/replace
```

Potential fields to rewrite, depending on confirmed rule:

```text
Path
RelativePath
RelativePathType
```

The exact rewrite rule must be backed by before/after Ableton tests.

### 6.17 Validation

Validation has three levels.

Level 1: file validation

```text
target ALS exists
copied dependency files exist
copied file sizes match expected values
copied hashes match where computed
no original ALS hash changed
```

Level 2: ALS semantic diff

```text
target ALS decompresses as gzip
target ALS parses as XML
only expected FileRef path fields changed
number of active refs stays consistent unless explicitly expected
no unexpected device/plugin/preset XML changes
```

Level 3: package validation

```text
all managed dependencies referenced by rewritten ALS exist in target package
manifest matches plan
no unresolved high-risk missing managed dependencies remain
system dependencies and plugin dependencies are reported clearly
```

Final validation status:

```text
Technical package validation can pass.
Ableton runtime verification remains manual unless future automation proves
otherwise.
```

### 6.18 User Verification

The product does not automatically open Ableton in the current specification.

The product tells the user:

```text
Open this copied ALS manually in Ableton.
Confirm whether the project opens correctly.
Mark package as verified or failed.
```

The user's verification result is recorded in the manifest or project registry.

## 7. Data Models

### 7.1 ProjectRecord

```text
project_id
project_name
project_root_path
main_als_path
backup_als_paths
all_als_paths
latest_modified_time
discovery_confidence
project_status
last_scan_time
```

### 7.2 DependencyRef

```text
dependency_id
project_id
als_ref_id
dependency_kind
raw_path
raw_relative_path
relative_path_type
resolved_path
filename
extension
original_file_size
original_crc
default_duration
default_sample_rate
usage_context
xml_context
xml_locator
existence_status
source_category
risk_flags
```

### 7.3 AssetRecord

```text
asset_id
path
filename
extension
file_kind
size_bytes
mtime
hash_full
hash_partial
duration
sample_rate
channels
bit_depth
scan_time
availability_status
```

### 7.4 MatchCandidate

```text
candidate_id
dependency_id
asset_id
score
score_reasons
auto_selected
requires_user_review
accepted_by_user
rejected_by_user
```

### 7.5 PackagePlan

```text
plan_id
project_id
source_project_root
target_project_root
source_als
target_als
copy_operations
rewrite_operations
system_dependencies
plugin_warnings
preset_warnings
missing_dependencies
rescue_candidates
estimated_size
plan_status
```

### 7.6 CopyOperation

```text
operation_id
dependency_id
source_path
target_path
copy_kind
expected_size
expected_hash
collision_strategy
status
```

### 7.7 RewriteOperation

```text
operation_id
dependency_id
als_ref_id
xml_locator
old_path
new_path
old_relative_path
new_relative_path
old_relative_path_type
new_relative_path_type
fields_to_change
rule_id
confidence
status
```

### 7.8 Manifest

```text
manifest_id
created_at
app_version
ruleset_version
source_project_root
target_project_root
source_als
target_als
source_als_hash_before
source_als_hash_after
target_als_hash
package_plan_id
files_copied
rewrite_operations
match_decisions
validation_result
system_dependencies
plugin_report
preset_report
user_verification_status
```

## 8. Dependency Handling Rules

### 8.1 Audio Samples

```text
Managed by product.
Copied into portable package when needed.
Missing samples can be searched and matched.
Rewritten ALS points to copied/accepted sample location.
```

### 8.2 User Library

```text
Copy by default for portable package.
Reason: User Library is not guaranteed to exist on another computer.
```

### 8.3 Core Library / Factory Packs

```text
Report as system_dependency by default.
Do not copy by default.
Mark as portable_risk when target machine/version/pack availability is unknown.
Future strict portable mode may copy factory pack audio.
```

### 8.4 VST / AU Plugins

```text
Report only.
Do not copy plugin binary.
Do not attempt installation.
Do not promise portability unless target machine has same plugin installed.
```

### 8.5 Plugin Presets

Current status:

```text
Unknown / needs test.
```

Working rule:

```text
Report plugin preset evidence if detectable.
Do not assume plugin preset files are copied or portable.
```

### 8.6 Max for Live

Current status:

```text
Ableton documentation says used Max for Live devices are copied to Presets by
Collect All and Save.
```

Product rule:

```text
Treat as domain test before implementing full support.
```

## 9. User-Facing Modes

### 9.1 Audit Only

```text
Scan and analyze.
Show project/dependency report.
Do not copy.
Do not rewrite.
```

### 9.2 Missing Sample Recovery

```text
Analyze selected project.
Find missing audio dependencies.
Search local disk/index.
Show candidates and confidence.
Let system auto-select >=95 matches.
Let user review ambiguous matches.
```

### 9.3 Portable Package

```text
Analyze selected project.
Resolve dependencies.
Build package plan.
Copy files to new location.
Rewrite copied ALS.
Validate package.
Write manifest.
User verifies in Ableton.
```

### 9.4 Future Batch Mode

```text
Run audit/package workflows for many projects.
Reuse asset index.
Avoid repeated scans.
Surface per-project status and blockers.
```

### 9.5 Future Cleanup Mode

```text
Not in current scope.
Would require reachability graph, safe garbage collection, rollback strategy and
very strict confirmation.
```

## 10. Acceptance Criteria For Full Product

A full product version is successful when:

```text
1. It can scan the local computer with user permission.
2. It can discover and group Ableton projects.
3. It can hide backup ALS files by default while recording them.
4. It can analyze a selected ALS without modifying it.
5. It can extract active audio dependencies.
6. It can verify whether referenced files exist.
7. It can classify dependency sources and risks.
8. It can build a local asset index.
9. It can search for missing samples.
10. It can score match candidates.
11. It can build a clear package plan.
12. It can copy accepted dependencies to a new project folder.
13. It can rewrite only the copied ALS file.
14. It can validate expected changes.
15. It can create a manifest.
16. It can present final manual Ableton verification instructions.
```

## 11. Known Unknowns And Required Domain Tests

### 11.1 Duplicate Filename CAS Test

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
5. Inspect rewritten ALS FileRef data.
```

Questions:

```text
Where does Ableton copy each file?
Does Ableton rename either file?
Does Ableton preserve folder structure or flatten into one folder?
How does Ableton rewrite FileRef / Path / RelativePath values?
Do copied files preserve hash/size/content exactly?
```

Current product rule until tested:

```text
Never overwrite on filename collision.
Generate unique target paths.
Record old -> new mapping in manifest.
```

### 11.2 Core Library / Factory Packs Portability Test

Purpose:

```text
Decide when Core Library / Factory Packs can be treated as system dependencies
and when they become portable package risks.
```

Questions:

```text
Are Core Library references stable between Ableton versions?
Which Factory Pack files are safe to treat as system dependencies?
When does Collect All and Save copy Factory Pack files?
How should missing Core/Factory files be classified?
```

### 11.3 User Library Collect Test

Purpose:

```text
Confirm how Ableton Collect All and Save handles User Library audio and presets.
```

Questions:

```text
Which User Library files are copied?
Where are they copied?
How are ALS references rewritten?
Are presets copied differently from audio samples?
```

### 11.4 Plugin / VST Preset Portability Test

Purpose:

```text
Discover what Ableton stores inside ALS for plugin state and what external
preset files, if any, Collect All and Save copies.
```

Suggested test sets:

```text
one VST2 preset
one VST3 preset
one AU preset on macOS
one Ableton Rack preset
one Max for Live device
```

Questions:

```text
Is active plugin state stored inside ALS?
Are external plugin preset files referenced?
Can the product detect those files?
Does Collect All and Save copy them?
How does behavior differ between VST2, VST3, AU, Ableton Rack and Max for Live?
```

### 11.5 ALS Rewrite Rule Tests

Purpose:

```text
Prove exact path fields to rewrite for supported active audio FileRef nodes.
```

Questions:

```text
Which fields must change for absolute external files?
Which fields must change for project-local files?
Which RelativePathType values represent each source category?
How should RelativePathType 0 be interpreted?
Which XML contexts are safe to rewrite?
```

### 11.6 OriginalCrc Controlled Ableton Test

Purpose:

```text
Discover whether Ableton OriginalCrc is derived from file bytes, decoded audio
content, Ableton analysis data, file metadata, path/name information, or another
internal Ableton sample identifier.
```

Current evidence:

```text
OriginalFileSize matched actual file size for the copied sample subset.
OriginalCrc did not match simple full-file CRC32 low16 or Adler32 low16.
OriginalCrc did not match the tested common CRC16/CRC32/sum/Fletcher hypotheses
over full files, file fragments, WAV data chunks or AIFF SSND payloads.
OriginalCrc remains UNKNOWN and must be treated only as weak evidence.
```

Controlled test setup:

```text
1. Create a clean Ableton test project, not based on a production project.
2. Generate a few deterministic small WAV files with known content.
3. Import one sample per track or clip so each reference is easy to inspect.
4. Save an initial baseline ALS.
5. After every controlled change, save a new ALS with a clear name.
6. Copy each ALS and each referenced sample into the experiment folder.
7. Extract and compare OriginalCrc, OriginalFileSize, FileRef path fields,
   file hash, audio payload hash and decoded PCM hash if available.
```

Test cases:

```text
T1: Same file bytes, different folder.
T2: Same file bytes, different filename.
T3: Same file bytes, different folder and filename.
T4: Same decoded PCM, changed WAV metadata/header only.
T5: Same filename and size, changed audio content.
T6: Same filename, changed audio content and changed size.
T7: Same decoded PCM exported through another container/settings if practical.
T8: Ableton Collect All and Save copy of the same baseline project.
T9: Ableton Save As / project copy without audio content change.
T10: Ableton analysis/asd regeneration if analysis files are present.
```

Questions:

```text
Does OriginalCrc stay stable when only path changes?
Does OriginalCrc stay stable when only filename changes?
Does OriginalCrc change when file bytes change but decoded PCM stays the same?
Does OriginalCrc change when decoded PCM changes?
Does OriginalCrc change after Collect All and Save?
Does OriginalCrc correlate with .asd analysis data?
Is OriginalCrc stable across repeated saves of the same ALS and same sample?
Is OriginalCrc stable across Ableton versions?
```

Product rule until tested:

```text
Do not use OriginalCrc as sole identity proof.
Use it only as a weak match signal together with stronger evidence such as
file size, strong file hash, audio fingerprint, path/name evidence and user
confirmation when needed.
```

## 12. Suggested Build Order

This specification describes the full product. Implementation should still be
incremental.

Recommended build order:

```text
1. ALSReader contract review and v0.2 data model.
2. ProjectDiscovery for local ALS/project grouping.
3. ProjectAnalyzer / Preflight for one selected project.
4. PathVerifier for existing/missing dependencies.
5. AssetIndexer for local audio files.
6. SampleMatcher with scoring.
7. PackagePlanner.
8. CopyStager.
9. ALSRewriter for copied ALS only.
10. Validator / SemanticDiff.
11. Manifest writer.
12. Batch mode and later cleanup only after safe core is proven.
```

First meaningful user-visible slice:

```text
User selects one Ableton project.
Product reads ALS.
Product reports all active audio dependencies.
Product shows which files exist, which are missing and which are risky.
No rewrite required.
```

First meaningful package slice:

```text
User selects one project with all managed audio dependencies available.
Product builds package plan.
User accepts.
Product copies files to new Ableton-like project folder.
Product rewrites copied ALS.
Product validates package and writes manifest.
User manually opens project in Ableton.
```

## 13. Specification Quality Rule

No module should be implemented from this full product specification alone.

Each module needs its own module specification with:

```text
module responsibility
inputs
outputs
data contract
what it does not do
test fixtures
acceptance criteria
safety rules
downstream consumers
```

`PRODUCT_SPINE.md` is the active product-level source of truth. This long-form
reference preserves broader context, while module specifications translate the
active product intent into buildable contracts.

## 14. MVP Boundary

This document describes the full product. MVP is a smaller, controlled slice.

MVP goal:

```text
Analyze one selected Ableton project, understand its managed audio dependencies,
find missing samples when possible, create a safe portable package for supported
cases, validate the result, and write a manifest.
```

In MVP:

```text
one selected project at a time
read-only ALS analysis
active audio dependency extraction
project/dependency preflight report
path existence verification
local asset indexing for audio files
missing sample matching with confidence scoring
inspectable PackagePlan
copy staging into a new target package
ALS rewrite only for copied ALS files and only for confirmed supported refs
validation / semantic diff
manifest writing
manual Ableton verification by user
```

Not in MVP:

```text
batch mode for many projects
safe cleanup / garbage collection
automatic deletion of unused files
Windows migration as a complete supported workflow
automatic Ableton runtime verification
plugin installation or plugin binary copying
full VST/AU preset portability
full Max for Live support
strict portable mode for Core Library / Factory Packs
cloud sync or remote storage
sample library reorganization across all projects
```

MVP platform rule:

```text
MVP is macOS-first.
MVP still must follow the platform portability guardrail.
Not implementing Windows migration does not allow hardcoding avoidable macOS
assumptions into the core.
```

Research before MVP implementation:

```text
duplicate filename Collect All and Save behavior
ALSReader v0.2 downstream contract
exact supported ALS rewrite fields
Core Library / Factory Packs portability behavior
User Library collect behavior
plugin/VST preset portability behavior
```

Rule:

```text
If a proposed task is not clearly MVP, classify it as Next, Later, Research,
Parking Lot or Rejected before implementation.
```

## 15. Definition Of Done For Any Module

A module is not done when code merely compiles.

A module is done only when:

```text
module specification exists
module responsibility is clear
inputs and outputs are defined
public data contracts are named and versioned when needed
known errors are listed
unsupported cases are explicit
unit tests exist for pure logic
fixture tests exist when the module touches ALS/domain data
safety tests exist when the module touches filesystem/write behavior
contract tests exist when downstream modules depend on the output
module.contract.json exists for new implementation modules
module.contract.json references active ENGINEERING_RULES.md version
workflow_guard module-ready passes before coding
workflow_guard verify-module passes before acceptance
engineering quality obligations are satisfied or explicitly marked REVIEW_ONLY
guard v0.2 quality gates pass when configured
validation behavior is defined
manifest/logging impact is defined when relevant
downstream consumers are identified
documentation/state is updated if product direction changed
remaining risks are stated
```

For write-capable modules, done also requires:

```text
original user files remain unchanged in tests or verification
operation can fail safely
partial output is marked partial/failed
manifest or operation evidence is produced
```

## 16. Module Specification Template

Every implementation module should get a focused module spec before build work.

Template:

```text
# Module Spec: MODULE_NAME

Status:
Owner:
Product capability:
Supported use case:
MVP / Next / Later / Research:

## Responsibility
What this module owns.

## Inputs
Required inputs and their contracts.

## Outputs
Returned outputs and their contracts.

## Data Contracts
Named public data models and versions.

## Machine Contract
Machine-readable module.contract.json fields, required tests, forbidden fields,
guard checks and verification commands.

## Does Not Do
Responsibilities explicitly outside this module.

## Algorithm Notes
Important implementation rules without over-specifying internals.

## Safety Rules
Original-file protection, write restrictions and blocking conditions.

## Error Codes
Expected structured errors.

## Tests
Unit, fixture, contract, safety, integration and domain tests.

## Fixtures
Required files or generated test data.

## Acceptance Criteria
What must be true before the module is accepted.

## Downstream Consumers
Which modules consume this output.

## Known Unknowns
UNKNOWN / HYPOTHESIS / CONFIRMED / REJECTED items relevant to this module.
```
