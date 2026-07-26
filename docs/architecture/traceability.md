# Product Traceability

Status: active trace map  
Date: 2026-07-26

## Product Capability Map

| ID | Capability | Current module path | Evidence |
| --- | --- | --- | --- |
| PC-001 | Read Ableton set facts without modifying user files | 001 ALSReader | `specs/001-als-reader/`, `analyze_als` tests |
| PC-002 | Convert active audio references one-to-one into reference occurrence records | 002 DependencyExtractor | `specs/002-dependency-extractor/`, `extract_dependencies` tests |
| PC-003 | Observe all explicit candidate paths without selecting asset identity | 003 PathObservation | `specs/003-path-verifier/`, ADR-004; blocked by E-01 |
| PC-004 | Group reference occurrences and assess recognized dependency availability | future DependencyAssessment | product spine |
| PC-005 | Search selected local scopes and resolve missing requirements | future AssetInventory + AssetResolution | backlog |
| PC-006 | Build a safe package/copy/rewrite plan | future PackagePlanner | backlog |
| PC-007 | Create a portable audio package | future StagingExecutor + Validator + PrivateLedger + PackageManifest | backlog |

## Module Handoff Map

```text
ALSReader
  output: ALSReadModel
  downstream: DependencyExtractor

DependencyExtractor
  input: ALSReadModel
  output: DependencyExtractionResult
  downstream: PathObservation

PathObservation
  input: DependencyExtractionResult + explicit PathObservationContext
  output: PathObservationResult
  downstream: DependencyAssessment

DependencyAssessment
  input: reference occurrences + path observations
  output: grouped RequiredAsset assessments
  downstream: read-only report, later AssetResolution
```

## Cross-Cutting Safety Requirements

| ID | Requirement | Enforced by |
| --- | --- | --- |
| INV-001 | Original files are read-only | ADR-001, `ENGINEERING_RULES.md`, module guards |
| INV-002 | Raw ALS values are preserved | 001/002 contracts, ADR-002 |
| INV-003 | Path interpretation is separate from path existence checks | ADR-002, 003 spec |
| INV-004 | Reference, required asset, file occurrence and content identity remain separate | ADR-003 |
| INV-005 | Path observations never select or verify asset identity | ADR-004, 003 contract |
| INV-006 | No copy/rewrite without a plan | future PackagePlanner contract |
| INV-007 | No success without validation and ledger/manifest evidence | future Validator contract |
| INV-008 | Ambiguity blocks automatic resolution or rewrite | product spec and future AssetResolution contract |

## Current Implementation Trace

```text
crates/rescue_core/src/als_reader.rs
crates/rescue_core/src/als_reader_impl.rs
crates/rescue_core/src/dependency_extractor.rs
crates/rescue_core/src/dependency_extractor_impl.rs
crates/rescue_core/src/path_parser.rs
crates/rescue_core/src/path_text.rs
cli/rescue-cli/src/main.rs
```

## Review Rule

When adding a new module, update this file only if it changes one of these
handoffs:

```text
product capability
module input/output contract
safety invariant
downstream consumer
```
