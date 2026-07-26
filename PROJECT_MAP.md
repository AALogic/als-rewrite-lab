# Project Map

Status: initial domain map  
Date: 2026-06-02  
Scope: responsibility boundaries for the Ableton dependency safety system

## 1. Purpose

This project builds a local desktop-first safety tool for Ableton projects.

Current platform stance:

```text
macOS first
platform-portable core
Windows migration later, not MVP
do not block future Windows/macOS support with avoidable core assumptions
```

The system should help users:

```text
inspect Ableton project dependencies
understand which audio files are active
create safe project copies/packages
rewrite only supported references in copies
validate changes before trusting them
eventually manage a safer audio library
```

The system is not:

```text
a general ALS editor
a plugin host
a sample manager first
a delete/cleanup tool in MVP
a UI-first prototype
```

## 2. Current Product Slice

Current vertical slice:

```text
ALSReader
-> ALSReadModel v0.2
-> DependencyExtractor
-> DependencyExtractionResult v0.1
-> diagnostic JSON
```

Current review status:

```text
ALSReader and DependencyExtractor are implemented and tested.
DependencyRef v0.1 is explicitly a reference occurrence compatibility model.
The old 003 PathVerifier selection contract is rejected.
The next flow must produce path observations without deciding asset identity.
```

Current command target:

```text
rescue analyze path/to/project.als
rescue extract path/to/project.als
```

Current rules:

```text
read only
no copy
no rewrite
no delete
no UI
no platform-specific path resolution
no inferred Project root
no candidate selection
```

## 3. Core Domains

### 3.0 Product Spine

Owns:

```text
product promise
primary use cases
capability map
global gates from vision to code
clarification requests when AI would need to guess
```

Does not own:

```text
module implementation details
crate boundaries
low-level ALS parsing rules
test fixture values
```

Primary files:

```text
PRODUCT_SPINE.md
```

Boundary rule:

```text
Product Spine says why and when to build.
Specs say exactly what this module must do.
Project Map says where responsibility lives.
```

### 3.1 Product Office

Owns:

```text
messy conversation intake
session digests
MVP/later classification
hypothesis vs fact classification
proposed writes to project files
```

Does not own:

```text
code implementation
ALS parsing rules
rewrite safety rules
```

Primary files:

```text
PRODUCT_OFFICE.md
PRODUCT_BACKLOG.md
session-digests/
intake/
CURRENT_STATE.md
```

### 3.2 Specification Layer

Owns:

```text
module behavior contracts
non-goals
acceptance criteria
test cases
implementation plans
task lists
```

Does not own:

```text
experimental raw notes
final architecture decisions unless accepted
implementation code
```

Primary files:

```text
specs/
```

Rule:

```text
No product code without a relevant spec.
No spec acceptance without Product Spine traceability.
```

### 3.3 rescue_core

Owns:

```text
read-only ALS file loading
gzip decompression
XML parsing
raw Ableton document metadata
raw SampleRef/FileRef extraction
raw reference models
basic path/value models
JSON-compatible output models
one-to-one extraction of reference occurrences
```

Does not own:

```text
source classification policy
storage_state
Project root discovery
filesystem availability observations
asset identity decisions
package planning
copy staging
ALS rewrite
semantic diff
SQLite index
UI decisions
```

Current implemented modules:

```text
ALSReader
DependencyExtractor
PathParser
```

Boundary rule:

```text
rescue_core reads and models raw facts. It should not decide what to do with them.
It preserves raw ALS path values without assuming macOS or Windows semantics.
DependencyRef v0.1 remains a reference occurrence, not a resolved asset.
```

### 3.4 rescue_analyzer

Owns later:

```text
Project root discovery evidence
grouping ReferenceOccurrence into RequiredAsset
filesystem path observations
dependency assessment
source classification
preflight report
unsupported/needs-review states
```

Examples:

```text
external_linked
project_managed
library_managed
factory_managed
missing
ambiguous
unsupported
```

Does not own:

```text
copying files
rewriting ALS
deleting files
UI approval flow
```

Boundary rule:

```text
rescue_analyzer interprets facts and observations. It may assess availability,
but asset resolution remains an explicit evidence-and-decision step.
```

### 3.5 rescue_rewriter

Owns:

```text
rewrite plans
copy/staging operations
active FileRef rewrite
semantic diff
rewrite validation
manifest data for rewrite/package operations
```

Does not own:

```text
raw ALS discovery
UI rendering
SQLite long-term index
unsafe original mutation
```

Boundary rule:

```text
rescue_rewriter only writes copies and only from explicit plans.
```

### 3.6 rescue_index

Owns later:

```text
SQLite schema
asset index
content hashes
redirect ledger
project registry
reachability graph
verification history
quarantine records
```

Does not own yet:

```text
MVP ALSReader
first CLI analyze flow
```

Boundary rule:

```text
rescue_index enters when the product needs memory across projects/sessions.
```

### 3.7 CLI

Owns:

```text
command line entrypoints
JSON output
error display
connecting user commands to core crates
```

Does not own:

```text
domain rules
ALS parsing logic
rewrite policy
UI workflows
```

Boundary rule:

```text
CLI calls domain code. It does not contain domain logic.
```

### 3.8 Desktop UI

Owns later:

```text
visual reports
review screens
user approvals
warnings
plan/apply interaction
project navigation
```

Does not own:

```text
ALS parsing
copy execution
rewrite execution
delete decisions
classification rules
```

Boundary rule:

```text
UI shows and asks. Core decides and executes.
```

### 3.9 Ableton Bridge

Possible future adapter:

```text
Max for Live
JUCE/VST/AU
local protocol
metadata handoff
verification helper
```

Does not own:

```text
rewrite engine
file deletion
library management
core safety rules
```

Boundary rule:

```text
Ableton bridge is an adapter, not the product core.
```

## 4. High-Level Data Flow

### 4.1 Current read-only flow

```text
CLI command
-> rescue_core ALSReader
-> gzip/XML read
-> ALSReadModel raw facts
-> DependencyExtractor reference occurrences
-> candidate path observations
-> dependency assessment
-> honest available / unavailable / unknown / unsupported report
-> fixture tests
```

### 4.2 Future package/rewrite flow

```text
Preflight
-> rescue_core reads ALS
-> rescue_analyzer classifies refs
-> planner builds explicit operation plan
-> rescue_rewriter stages copies
-> rescue_rewriter rewrites ALS copy only
-> semantic diff validates expected changes
-> manifest is written
-> user verifies in Ableton
```

### 4.3 Future long-term library flow

```text
scan projects
-> hash assets
-> store asset index
-> maintain redirect ledger
-> compute reachability
-> propose cleanup dry-run
-> quarantine before delete
```

## 5. Ownership Rules

```text
ALS reading lives in rescue_core.
ReferenceOccurrence extraction lives in rescue_core.
Project discovery and dependency assessment live in rescue_analyzer.
Asset resolution owns candidate evidence and decisions.
Rewrite lives in rescue_rewriter.
Persistent project/library memory lives in rescue_index.
CLI contains commands, not domain logic.
Desktop UI never rewrites ALS directly.
Product Office handles messy conversations before specs.
Product Spine owns product promise, use cases, capability map and gates.
Specs define module behavior before code.
AI_CONTRACT defines forbidden actions.
AGENTS defines how Codex works in this repo.
Path observation never owns asset identity selection.
```

## 6. Forbidden Placements

Do not put:

```text
rewrite logic in UI
rewrite logic in ALSReader
classification policy inside low-level XML parsing
SQLite index logic in ALSReader
file delete logic before reachability exists
sample matching shortcuts in UI
one-off ALS behavior assumptions in code without fixtures
domain safety rules only in chat
```

If tempted:

```text
stop
update spec or project map
ask whether a new domain/module is needed
```

## 7. Open Boundaries

These are intentionally not final:

```text
Whether SampleRef extraction is part of ALSReader or a submodule inside rescue_core.
Whether usage_context is resolved in rescue_core or rescue_analyzer.
Whether semantic_diff belongs inside rescue_rewriter or its own module.
Exact JSON contract between rescue_core and rescue_analyzer.
Exact shape of storage_state.
Exact location of manifest models.
When SQLite becomes necessary.
Whether ALSReader JSON is a user-facing diagnostic report, an inter-module
contract, or both.
```

Rule:

```text
Do not resolve open boundaries before evidence or implementation pressure requires it.
```

## 8. Test And Gate Map

Current required gates for ALSReader:

```text
Product Spine Contract Review
fixture-based parse tests
read-only safety test
structured error tests
JSON shape check
```

Future gates:

```text
cargo test
cargo fmt --check
cargo clippy
semantic diff checks
forbidden write/delete checks
manifest validation
manual Ableton-open verification for rewrite features
```

## 9. Update Rule

Update this file when:

```text
a new domain is introduced
responsibility moves between crates/modules
UI is allowed to call a new backend operation
rewrite or package flow changes
an open boundary is resolved
```

Do not update this file for:

```text
small wording changes
one-off experiments
temporary research scripts
minor fixture additions
```
