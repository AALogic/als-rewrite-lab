# Product Architect Brief: Ableton Project Rescue

Status: historical architecture-review brief; not an active implementation source
Date: 2026-07-27
Audience: software architect / technical lead
Scope: product-level understanding, architecture input, MVP direction

Active routing:

```text
CURRENT_STATE.md owns current implementation status and blockers.
PRODUCT_SPINE.md owns active product direction and the MVP boundary.
PROJECT_MAP.md owns current module responsibilities.
The active module specifications own exact behavior.
```

## 1. Executive Summary

Ableton Project Rescue is a local-first desktop application for Ableton Live
users who have projects with audio files scattered across many folders,
drives, old projects, Downloads, Splice folders, User Library and ad hoc
locations.

The product helps the user:

```text
find Ableton projects on the computer
read their .als files safely
extract audio sample dependencies
check whether referenced files still exist
search for missing or moved samples
build a safe relocation/package plan
copy required files into a new portable project folder
rewrite only the copied ALS file
validate the result
write a manifest explaining every decision and operation
```

The product is not merely an ALS rewriter. It should be understood as a local
dependency analysis, recovery and packaging tool for Ableton Live projects.

Core product promise:

```text
Help music producers understand, recover and safely package Ableton projects
whose audio dependencies are spread across messy local folders.
```

## 2. Primary User Problem

Music producers often work with Ableton projects that reference files outside
the project folder. Over time, samples may live in:

```text
Downloads
Desktop
Splice folders
old project folders
User Library
external drives
shared folders
ad hoc sample folders
```

Ableton projects may continue to work on the original machine for years, but
become fragile when the user:

```text
moves to a new computer
opens old projects
upgrades Ableton
sends a project to another person
cleans Downloads or sample folders
changes disk structure
works across macOS and Windows
```

The user needs to know:

```text
what files a project actually depends on
where those files currently are
which files are missing
which files are risky external dependencies
which projects can be safely packaged
which operations are safe to perform automatically
which operations require manual confirmation
```

## 3. Target Users

Primary users:

```text
music producers with old and current Ableton projects
producers who download samples into messy local folders
producers moving projects between computers
producers sending projects to collaborators
producers wanting to make a project self-contained
```

Secondary/future users:

```text
studio engineers managing many client projects
producers migrating Windows -> macOS
producers upgrading Ableton versions
collaborators receiving incomplete Ableton archives
```

## 4. Product Modes

### 4.1 Audit Only

The user selects an Ableton project and receives a report:

```text
which audio files are referenced
which paths are stored in ALS
which files exist
which files are missing
which dependencies are external/risky
which dependencies are ignored or report-only
```

No files are copied or modified.

### 4.2 Missing Sample Recovery

The product knows what the ALS expected and searches the local machine for
candidate files. Matching should use evidence such as:

```text
file name
file extension
file size
available ALS metadata
hashes from scanned files
audio metadata when available later
confidence scoring
```

Ambiguous matches must not be silently accepted.

### 4.3 Portable Package

The product creates a new project copy in a user-selected location. It copies
accepted dependencies into an Ableton-like folder structure and rewrites only
the copied ALS so that it points to the new local files.

The original project remains untouched.

### 4.4 Future Batch Mode

Later, the same flow should support many projects at once:

```text
scan many projects
analyze dependencies
reuse file index
build per-project plans
run safe package operations
write manifests
```

### 4.5 Future Library Maintenance / Cleanup

Later, the product may help maintain a safer audio library:

```text
deduplicate by content hash
track which files are reachable from which projects
detect unused files
quarantine before deletion
support rollback
```

This is not MVP.

## 5. Non-Negotiable Safety Rules

The product must be conservative. It operates on user creative work and must
not silently destroy or corrupt data.

Hard rules:

```text
never overwrite an original ALS file
never rewrite an original ALS file
never delete original audio files
never delete original project folders
never perform destructive cleanup in MVP
never choose between ambiguous matches without policy/user decision
never claim Ableton verification unless the user actually opens the result
never use global text search/replace as ALS rewrite strategy
never treat Ableton OriginalCrc as sole proof of file identity
```

Safe default:

```text
read originals
copy originals when needed
modify only staged/copied artifacts
validate before success
write manifest
allow user inspection
```

## 6. Known Technical Fact: ALS Format

Current confirmed fact:

```text
.als is gzip-compressed XML
```

The product can:

```text
read the ALS file
decompress gzip
parse XML
extract selected XML structures
preserve raw path values
produce structured JSON-compatible models
```

Important distinction:

```text
reading ALS safely is not the same as rewriting ALS safely
```

Rewrite needs separate rules, tests, semantic diff and validation.

## 7. High-Level Workflow

The intended full workflow:

```text
1. User installs local desktop app.
2. User grants permission for disk scanning.
3. Product scans for Ableton projects / .als files.
4. Product lists detected projects.
5. User selects one project for analysis.
6. Product creates an analysis snapshot / working context.
7. ALSReader reads the ALS file as gzip XML.
8. DependencyExtractor extracts active audio dependencies.
9. PathVerifier checks whether referenced paths exist on disk.
10. PreflightReport explains project health.
11. AssetIndexer scans candidate audio files on disk.
12. SampleMatcher searches for missing/moved samples.
13. PackagePlanner builds a proposed copy/rewrite plan.
14. User reviews or accepts the plan.
15. CopyStager copies files into a staged target folder.
16. ALSRewriter rewrites only the copied ALS.
17. Validator checks file existence and ALS semantic diff.
18. ManifestWriter records all operations and decisions.
19. User opens the new project manually in Ableton for final verification.
```

## 8. Module Map

### 8.1 ProjectScanner

Finds Ableton projects and .als files on disk.

Responsibilities:

```text
scan allowed folders/disks
find .als files
group them into likely Ableton projects
identify main sets vs backups where possible
record discovered projects
```

Does not:

```text
parse ALS deeply
copy files
rewrite files
delete files
```

### 8.2 ALSReader

Reads an ALS file and produces raw structured facts.

Responsibilities:

```text
read ALS from path
decompress gzip
parse XML
extract document metadata
extract active SampleRef/FileRef audio references
extract historical OriginalFileRef references
extract report-only non-audio FileRef signals
preserve raw path strings
return ALSReadModel
```

Does not:

```text
check whether sample files exist
classify source folders
search disk
match missing files
plan copying
rewrite ALS
```

Current status:

```text
implemented as 001-als-reader
read-only
tested against ALS fixtures
outputs ALSReadModel v0.2
```

### 8.3 DependencyExtractor

Transforms raw ALSReader references into clean downstream dependency records.

Responsibilities:

```text
consume ALSReadModel
emit DependencyExtractionResult
produce one DependencyRef per active audio dependency
preserve relevant ALS evidence
mark incomplete path information
propagate upstream warnings
summarize ignored historical/non-audio input
```

Does not:

```text
read files from disk
verify paths
deduplicate dependencies
classify local/external/core/user library
match samples
plan copying
rewrite ALS
```

Current status:

```text
implemented as 002-dependency-extractor
tested and verified
outputs DependencyExtractionResult v0.1
```

### 8.4 PathVerifier

Checks whether exact paths from dependencies resolve on the current system.

Responsibilities:

```text
check raw absolute path
check raw relative path against source project root
detect existing file
detect missing file
detect not-a-file
detect unreadable file
compare file size when ALS provides size
preserve both absolute and relative path evidence
report platform-unsupported paths
```

Does not:

```text
search nearby folders by name
scan the whole disk
match alternate candidates
hash audio content
modify dependency records in place
```

Current status:

```text
spec interview mostly completed
not yet implemented
```

### 8.5 AssetIndexer

Builds a local index of candidate audio files.

Responsibilities:

```text
scan selected or full-disk locations
find audio files
record path, filename, extension, size
compute content hash where useful
cache results to avoid rescanning everything
support future multi-project workflows
```

Does not:

```text
decide which candidate is correct
rewrite ALS
copy project packages
```

### 8.6 SampleMatcher

Matches missing dependencies to candidate files found by AssetIndexer.

Responsibilities:

```text
compare DependencyRef / PathVerification result to AssetRecord candidates
use strong evidence where available
use weak evidence only as supporting signal
produce confidence scores
detect ambiguous candidates
block low-confidence automatic choices
```

Does not:

```text
silently choose ambiguous matches
copy files
rewrite ALS
delete candidates
```

### 8.7 PreflightReport

Explains project state before any write operation.

Responsibilities:

```text
summarize dependency health
show ready/missing/risky/unsupported dependencies
show what can be packaged now
show what needs user decision
show what is report-only
```

### 8.8 PackagePlanner

Creates a planned operation before any file writes.

Responsibilities:

```text
decide target package layout
map source files to target files
handle duplicate filenames
include rewrite operations
mark unsupported items
require user decisions where needed
produce PackagePlan
```

Does not:

```text
copy files directly
rewrite ALS directly
validate Ableton runtime behavior
```

### 8.9 CopyStager

Executes the file-copy portion of an accepted plan.

Responsibilities:

```text
create staging folder
copy selected audio files
preserve required folder layout
avoid overwriting unless plan explicitly allows
verify copied file size/hash
record copy results
```

Does not:

```text
decide what to copy
rewrite ALS
delete originals
```

### 8.10 ALSRewriter

Rewrites only supported references in a copied ALS file.

Responsibilities:

```text
operate only on a copied ALS
load gzip XML
apply explicit RewritePlan
change only supported active FileRef values
write temporary output first
validate gzip/XML
preserve unrelated XML
produce rewrite result
```

Does not:

```text
rewrite original ALS
globally replace strings
rewrite unsupported XML structures
choose sample matches
copy files
```

### 8.11 Validator / SemanticDiff

Checks whether the output package is internally consistent.

Responsibilities:

```text
confirm target files exist
confirm copied files match expected evidence
confirm rewritten ALS is gzip-valid
confirm rewritten ALS is XML-valid
confirm semantic diff changed only expected references
produce validation report
```

### 8.12 ManifestWriter / Operation Ledger

Records what happened.

Responsibilities:

```text
write structured manifest
record source project
record input ALS hash
record dependency facts
record user decisions
record copy operations
record rewrite operations
record validation results
record tool/ruleset versions
```

Future use:

```text
debugging
support
rollback reasoning
safe cleanup
multi-project dependency graph
```

## 9. Core Data Models

Important current/future models:

```text
ALSReadModel
ActiveAudioReference
HistoricalReference
NonAudioDependencySignal
DependencyExtractionResult
DependencyRef
PathVerificationResult
AssetRecord
MatchCandidate
PackagePlan
CopyOperation
RewriteOperation
ValidationResult
Manifest
```

Important principle:

```text
raw ALS facts and downstream product decisions must remain separate.
```

Example:

```text
ALSReader may say: ALS contains raw_path X.
PathVerifier may say: X exists / does not exist on this computer.
SampleMatcher may say: candidate Y probably matches.
PackagePlanner may say: copy Y to target path Z.
ALSRewriter may say: rewrite copied ALS path X -> Z.
```

These responsibilities should not collapse into one module.

## 10. Matching And Evidence Philosophy

The product needs a conservative confidence model.

Possible evidence:

```text
exact path exists
file size matches OriginalFileSize
filename matches
extension matches
content hash matches
audio metadata matches
Ableton OriginalCrc present
duration/sample rate signals
```

Current known rule:

```text
OriginalCrc is useful as a signal, but must not be treated as sole proof.
```

Proposed behavior:

```text
high confidence exact match -> may proceed according to policy
ambiguous match -> user decision required
low confidence -> do not rewrite automatically
unsupported structure -> block rewrite
```

## 11. Platform Stance

Current product stance:

```text
macOS first
platform-portable core
Windows migration later
do not hardcode avoidable macOS-only assumptions into core modules
```

Important design implication:

```text
preserve raw ALS paths as data
interpret platform-specific paths in PathVerifier/filesystem adapters
keep core logic mostly platform-neutral
```

Examples:

```text
macOS absolute paths
Windows drive-letter paths
Windows UNC paths
external volumes
relative paths escaping project root
```

These should be modeled explicitly rather than hidden inside string hacks.

## 12. Storage And Persistence

The current implemented product core returns Rust structs / JSON-compatible
models. It does not yet have a database.

Future persistence may include:

```text
SQLite local index
manifest JSON files
operation ledger
cached asset index
project history
```

Architectural note:

```text
Do not confuse current in-memory domain models with future database tables.
If a relational schema is introduced, storage IDs and foreign keys should be
treated as persistence concerns, not core domain identifiers.
```

## 13. Implementation Status Ownership

This dated brief no longer owns implementation inventory. Read
`CURRENT_STATE.md` for what exists, `PROJECT_MAP.md` for current responsibility
boundaries, and the active module specifications for exact contracts.

## 14. MVP Direction

The MVP should prove one safe, narrow vertical slice:

```text
one selected Ableton project
read ALS
extract active audio dependencies
verify exact referenced paths
report missing/existing dependencies
eventually create a portable package for supported cases
write manifest
never touch originals
```

MVP should not include:

```text
batch mode
cleanup/delete mode
plugin portability
full Max for Live support
automatic Ableton runtime verification
Windows migration flow
complex library management
```

## 15. Key Architecture Questions For Review

Architecture review should focus on:

```text
1. Are module boundaries correctly separated?
2. Should persistence be introduced now or after PathVerifier/PackagePlanner?
3. What is the right shape of domain models vs storage models?
4. How should platform-specific path interpretation be isolated?
5. What should be the minimum manifest schema?
6. How should confidence scoring be represented safely?
7. What should block automation vs ask the user?
8. How should ALS rewrite be validated before trust?
9. How should the codebase remain cross-platform-ready while macOS-first?
10. What is the smallest useful MVP vertical slice?
```

## 16. Suggested Architecture Principle

The system should be designed as a staged pipeline:

```text
discover
read
extract
verify
index
match
plan
stage
rewrite
validate
record
```

Each stage should:

```text
have one responsibility
take explicit input models
return explicit output models
not mutate upstream models
not perform work owned by later stages
emit warnings/errors in structured form
be testable in isolation
```

Critical product invariant:

```text
No write operation should happen without a prior plan.
No rewrite should happen on an original ALS.
No success state should exist without validation and manifest.
```
