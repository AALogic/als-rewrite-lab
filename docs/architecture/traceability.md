# Product Traceability

Status: active trace map  
Date: 2026-07-27

## Capability Map

| ID | Capability | Module | Executable evidence |
| --- | --- | --- | --- |
| PC-001 | Read ALS facts without modifying files | 001 ALSReader | `specs/001-als-reader`, fixture and safety tests |
| PC-002 | Preserve active audio reference occurrences | 002 DependencyExtractor | one-to-one, duplicate and ignored-input tests |
| PC-003 | Observe recorded paths without selecting identity | 003 PathObservation | path safety, platform and no-selection tests |
| PC-004 | Establish project context conservatively | 004 ProjectDiscovery | marker, ambiguity, backup and symlink tests |
| PC-005 | Group occurrences into required audio assets | 005 DependencyAssessment | grouping, snapshot and conservative-status tests |
| PC-006 | Explain project readiness without writing | 006 PreflightReport | missing, unknown and untrusted-input tests |
| PC-007 | Build a content-addressed selected-scope inventory | 007 AssetInventory | hash, duplicate, partial, symlink and read-only tests |
| PC-008 | Rank and select only sufficiently certain candidates | 008 AssetResolution | threshold, tie, partial-index and handoff tests |
| PC-009 | Produce an immutable copy/rewrite plan | 009 PackagePlanner | purity, collision, unsupported and determinism tests |
| PC-010 | Copy planned bytes into fresh staging | 010 StagingExecutor | no-overwrite, hash and source-immutability tests |
| PC-011 | Rewrite only snapshot-bound approved ALS values | 011 ALSRewriter | locator, old-value, allowlist and retry tests |
| PC-012 | Validate files and semantic ALS difference | 012 PackageValidator | tamper, historical, unrelated-change and read-only tests |
| PC-013 | Record private and portable evidence | 013 Ledger/Manifest | privacy, atomic write and idempotency tests |
| PC-014 | Promote validated staging to an absent target | 014 PackagePromoter | complete-tree, rollback gate and retry tests |
| PC-015 | Compose one safe laboratory vertical slice | 015 LaboratoryPipeline | eight end-to-end scenarios plus real private run |

## Handoff Map

```text
ProjectDiscoveryResult + ALSReadModel
-> DependencyExtractionResult
-> PathObservationResult
-> DependencyAssessmentResult
-> PreflightReport

DependencyAssessmentResult + AssetInventoryResult
-> AssetResolutionResult
-> PackagePlan
-> StagingExecutionResult
-> ALSRewriteResult
-> PackageValidationResult
-> ManifestWriteResult
-> PackagePromotionResult
```

Every arrow is a public typed contract. Downstream modules do not read private
implementation state from upstream modules.

## Cross-Cutting Safety Requirements

| ID | Requirement | Enforced by |
| --- | --- | --- |
| INV-001 | Original files remain read-only | ADR-001, modules 001, 010-015 tests |
| INV-002 | Raw ALS facts remain separate from interpretation | 001/002 contracts, ADR-002 |
| INV-003 | Path observation never proves identity | 003 contract, ADR-004 |
| INV-004 | Reference, requirement, occurrence and content remain distinct | ADR-003, modules 005/007/008 |
| INV-005 | Partial inventory cannot auto-select | 008 test and 015 blocked scenario |
| INV-006 | Ambiguity cannot auto-select | 008 test and 015 ambiguous scenario |
| INV-007 | No copy or rewrite without a ready plan | 009/010/015 contracts |
| INV-008 | Rewrite is source-hash, locator and old-value bound | ADR-005, 009/011 |
| INV-009 | Only approved XML fields may differ | 011/012 semantic tests |
| INV-010 | No promotion without validation and manifests | 012-014 contracts |
| INV-011 | Existing targets are not overwritten or merged | 010, 012-015 tests |
| INV-012 | Portable package does not disclose private absolute paths | 013/015 privacy tests |

## Review Rule

Update this map only when a change adds or alters a product capability, public
handoff, safety invariant, or its executable evidence.
