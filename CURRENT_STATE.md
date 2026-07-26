# Current State

Status: active  
Date: 2026-07-26  
Purpose: one short source of truth for the current project position

## 1. What We Are Building

This is a local, read-only-first safety tool for Ableton Live projects.

The product should help a user:

```text
understand which audio assets a selected Live Set requires
see what is available, unavailable, unknown or unsupported
review evidence before accepting a replacement for a missing asset
later create a staged and validated portable audio package
never damage the original ALS or original media
```

The core product problem is evidence-based dependency resolution and safe
project recovery. Parsing gzip/XML is an adapter capability, not the product's
main domain.

## 2. What Exists In Code

Implemented and verified:

```text
001 ALSReader
  reads gzip/XML in read-only mode
  extracts recognized active audio references
  preserves historical and non-audio signals separately

002 DependencyExtractor
  converts active ALS references one-to-one into DependencyRef v0.1
  preserves raw evidence and order
  does not inspect the filesystem or resolve assets

PathParser
  classifies raw ALS path text without applying host-path semantics
```

The current `DependencyRef v0.1` represents a reference occurrence, not a
unique required asset and not a file found on disk.

Current verified baseline after the corrective work:

```text
cargo fmt --check: PASS
cargo check --workspace --locked: PASS
cargo test --workspace --locked: PASS, 43 tests
cargo clippy --workspace --all-targets -- -D warnings: PASS
workflow guard verify-module 001: PASS
workflow guard verify-module 002: PASS
workflow guard module-ready 003: EXPECTED BLOCK on BLOCKING_UNKNOWN
workflow guard Python unit test: PASS
guard hardening regression: PASS, 10/10 bad scenarios caught
```

## 3. Active Architecture Decisions

```text
ReferenceOccurrence != RequiredAsset
RequiredAsset != FileOccurrence
FileOccurrence != ContentIdentity

path is location evidence, not identity
the parent directory of an ALS is not automatically the Ableton Project root
filesystem existence does not prove that a file is the expected sample
OriginalCrc is weak evidence only
matching scores may rank candidates but may not silently confirm identity
```

Original ALS and original media remain read-only.

## 4. Completed Corrective Slice

The independent audit found that the old 003 PathVerifier contract mixed:

```text
candidate generation
filesystem availability
asset identity
candidate selection
```

Completed:

1. keep raw ALS facts separate from filesystem observations;
2. remove inferred project root from ALSReader output semantics;
3. make 003 return all candidate path observations without selecting a file;
4. add explicit path-safety and platform states;
5. fix duplicated XML context and the misleading CLI output flag;
6. make workflow guard quality limits cover implementation files;
7. preserve the old long state document in
   `docs/archive/CURRENT_STATE.pre-audit-2026-07-26.md`.
8. split ALS file decoding/I/O from reference extraction after the truthful
   quality guard exposed an oversized function;
9. initialize a local Git repository and ignore private/generated corpora.

## 5. The One Next Product Step

After this corrective slice is green:

```text
run E-01: project-root and path-semantics experiment
run E-02: dependency-coverage comparison against Ableton File Manager / CAS
then implement one-project read-only path observations and dependency assessment
```

Do not implement the old 003 contract. Do not start AssetIndexer, SampleMatcher,
PackagePlanner, copying or ALS rewrite before E-01 and E-02 have been reviewed.

## 6. Current Blockers And Unknowns

Blocking 003 implementation:

```text
how Project root is supplied or discovered
the exact safe candidate-generation rules
the support boundary for recognized active audio references
native path and symlink policy across macOS and Windows
```

Repository blocker:

```text
the local Git repository has been initialized
private experiments and binary audio/ALS data are ignored
the first commit must wait for an explicit fixture privacy/license decision
```

Important later unknowns:

```text
safe rewrite support matrix for Live 9/10/11/12
matching without a historical full-content hash
plugin, preset and Pack portability
cloud placeholder and disconnected-volume behavior
```

## 7. Active Sources And Routing

Read in this order:

```text
1. CURRENT_STATE.md
2. PRODUCT_SPINE.md
3. the one active module spec
4. AI_CONTRACT.md for safety-sensitive work
5. ENGINEERING_RULES.md for code/refactor work
```

Supporting references are read only when the task needs them:

```text
PROJECT_MAP.md
ALS_REWRITE_METHODOLOGY.md
docs/architecture/
docs/audits/ableton-domain-architecture-audit.md
session-digests/
```

History is not an active instruction source.

## 8. Scope Boundaries

Now:

```text
one selected project
recognized audio dependency facts
read-only path observations
honest available / unavailable / unknown / unsupported report
```

Next:

```text
selected-scope inventory
candidate evidence
user decisions
```

Later:

```text
staged package
validated limited ALS rewrite
batch processing
desktop UI
Windows adapter
```

Outside the current product:

```text
automatic deletion
copying or installing plugins
universal Finder/Explorer delete protection
cloud collaboration
guaranteed compatibility with every Live version
```
