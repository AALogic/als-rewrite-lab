# Plan 001: ALSReader

Status: v0.1 implementation plan; superseded by spec.md v0.2 contract update  
Date: 2026-06-02  
Scope: implementation plan for the first read-only ALS module

Update note 2026-06-09:

```text
This is the historical v0.1 plan.
The controlling contract is now spec.md v0.2.
Any mention of ALSAnalysis or filesystem path existence checks is legacy.
The next implementation pass must build ALSReadModel v0.2.
```

## 1. Goal

Build the smallest product slice that can answer:

```text
What active audio files does this Ableton .als file reference?
```

The first usable flow is:

```text
rescue analyze path/to/file.als
```

This command is only the outside door. The real reading logic lives in
`rescue_core` as `ALSReader`.

## 2. Simple Mental Model

```text
.als file
-> gzip decompression
-> XML document
-> SampleRef nodes
-> direct FileRef children
-> ALSReadModel v0.2
-> diagnostic JSON from CLI
```

In plain terms:

```text
ALSReader opens the Ableton set like a sealed folder,
reads the list of active audio references,
and returns a structured report without moving or changing anything.
```

## 3. Module Boundaries

`rescue_core` owns:

```text
reading bytes
decompressing gzip
parsing XML
extracting raw active SampleRef/FileRef facts
building ALSReadModel v0.2
```

`rescue_core` does not own:

```text
project cleanup
sample matching
source classification policy
copying files
rewriting ALS
SQLite persistence
UI decisions
```

`rescue-cli` owns:

```text
command line arguments
calling rescue_core
printing JSON
returning process exit codes
```

The CLI must not contain ALS parsing logic.

## 4. Data Contract

The v0.1 product object was:

```text
ALSAnalysis
```

This is now legacy. The v0.2 internal product object is:

```text
ALSReadModel v0.2
```

The CLI may still print simplified JSON, but downstream modules must not treat
that JSON as the full internal contract.

The historical v0.1 minimum fields were:

```text
source_path
ableton_creator
ableton_minor_version
ableton_schema_change_count
sample_ref_count
active_file_ref_count
sample_ref_historical_original_file_ref_count
total_original_file_ref_count
active_refs
warnings
errors
```

For v0.2, the complete field contract lives in `spec.md`.

Each `active_ref` should represent only the direct `FileRef` child of a
`SampleRef`.

Nested `OriginalFileRef` values are provenance/history and must not be returned
as active audio dependencies.

## 5. Implementation Steps

1. Create minimal Rust workspace files.
2. Create `crates/rescue_core` library crate.
3. Create `cli/rescue-cli` binary crate.
4. Add a read-only `analyze_als(path)` API in `rescue_core`.
5. Add JSON-serializable models for `ALSAnalysis`, `ActiveFileRef`,
   `ALSWarning`, and `ALSError`.
6. Implement file existence/readability checks.
7. Implement gzip decompression.
8. Implement XML parse.
9. Extract Ableton root metadata.
10. Extract `SampleRef` nodes.
11. Extract only direct child `FileRef` from each `SampleRef`.
12. Count `OriginalFileRef` nodes separately.
13. Add `rescue analyze path` with JSON as its single output format.
14. Add fixture/golden tests.
15. Verify read-only behavior.

v0.2 implementation update:

```text
replace ALSAnalysis with ALSReadModel v0.2 as the internal contract
separate active audio references from historical references
capture non-audio dependency signals as report-only evidence
remove path existence checks from ALSReader
move path existence checks to PathVerifier
preserve raw paths, raw RelativePathType and rewrite-relevant XML locators
keep CLI JSON as a derived diagnostic report
```

## 6. Test Strategy

Use fixture-based tests first.

The important tests are:

```text
valid_als_returns_analysis_json
active_refs_are_not_historical_refs
relative_path_type_is_captured_raw
invalid_gzip_returns_structured_error
read_only_safety
```

Why this matters:

```text
The parser should be judged against known Ableton files, not against a vague
feeling that it prints something plausible.
```

## 7. Safety Rules

ALSReader must not:

```text
write to the source ALS
write to the source project folder
copy audio files
rewrite paths
delete anything
infer sample matches
```

If something is unknown:

```text
return a warning or structured error
```

Do not guess.

## 8. Acceptance

This plan is complete when:

```text
cargo test passes
rescue analyze fixture.als prints valid JSON
valid fixtures match expected counts
invalid gzip returns structured not_gzip
source fixture bytes remain unchanged after analysis
```

## 8A. Retrospective Guard Acceptance

After the guarded workflow was introduced for later modules, ALSReader must also
be accepted through the same contract mechanism.

Required guard commands:

```text
Run workflow_guard module-ready 001-als-reader.
Run workflow_guard verify-module 001-als-reader.
```

Why:

```text
ALSReader was implemented before module.contract.json existed.
The retroactive guard proves that the reader contract, tests, public data
models, module boundaries and engineering rules are now checked like later
modules.
```

## 9. Deferred

Explicitly not in this plan:

```text
rescue_analyzer
source classification
package planning
ALS rewrite
semantic diff
desktop UI
SQLite index
Windows migration
```
