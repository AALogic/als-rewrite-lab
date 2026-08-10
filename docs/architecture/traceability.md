# Product Traceability

Status: active trace map  
Date: 2026-08-05

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
| PC-010 | Copy planned bytes into fresh staging | 010 StagingExecutor | no-overwrite, policy-specific verification and source-immutability tests |
| PC-011 | Rewrite only snapshot-bound approved ALS values | 011 ALSRewriter | locator, old-value, allowlist and retry tests |
| PC-012 | Validate files and semantic ALS difference | 012 PackageValidator | tamper, historical, unrelated-change and read-only tests |
| PC-013 | Record private and portable evidence | 013 Ledger/Manifest | privacy, atomic write and idempotency tests |
| PC-014 | Promote validated staging to an absent target | 014 PackagePromoter | complete-tree, rollback gate and retry tests |
| PC-015 | Compose one safe laboratory vertical slice | 015 LaboratoryPipeline | eight end-to-end scenarios plus real private run |
| PC-016 | Expose one read-only desktop analysis flow without duplicating domain policy | 016 DesktopApplicationService | valid project, redaction, failure, read-only and determinism tests |
| PC-017 | Create a complete or incomplete copy from files still present at recorded ALS paths | 017 CurrentPathCopyPipeline | complete, partial, all-missing, rewrite and source-read-only integration tests |
| PC-018 | Expose preview, consent and copy execution to the desktop without moving policy into UI | 018 DesktopCopyApplicationService | preview, consent, stale-source, complete and incomplete application tests |
| PC-020 | Observe ALS files and Project markers under approved roots without parsing or writing | 020 ALSProjectScanner | bounded walk, exclusion, partial coverage, symlink/reparse, cancellation, determinism and read-only tests plus controlled macOS scan |
| PC-021 | Build a physical ProjectFolder / LiveSet / BackupSet catalog without family inference | 021 ProjectCatalogBuilder | 15 pure grouping, duplicate-name, multi-Set, backup-boundary, ambiguity, determinism and contract-validation tests |
| PC-022 | Persist Project catalog snapshots without treating partial absence as deletion | 022 ProjectCatalogStore | 14 round-trip, idempotency, complete/partial freshness, changed-scope preservation, corruption, symlink and atomic-write tests plus Windows x64 compile check |
| PC-023 | Expose a simple desktop list and explicit ProjectSelection contract | 023 ProjectCatalogApplicationService | 14 refresh, projection, revision, privacy, manual/catalog convergence and wire-contract tests plus frontend contract tests |
| PC-024 | Run the unchanged one-project flow sequentially for many selections | 024 BatchCopyApplicationService | 13 all-previews-before-write, collision, boundary-revalidation, isolation, cancellation, sequential-write, privacy and wire-contract tests plus frontend contract tests |
| PC-025 | Launch one-project copy from one explicit ALS through a compact assistant without duplicating copy policy | 025 QuickCopyAssistant | Rust launch/contract tests, pure frontend state tests, packaged cold/warm macOS launch, complete/incomplete real copy evidence and source-safety comparison |
| PC-026 | Coordinate one trusted browser destination and native copy-only drag for a validated payload attempt | 026 ExternalFolderHandoff | trusted-provider, path-free IPC, copy-only adapter, geometry and frontend transfer-state tests; live WeTransfer behavior remains owner verification |
| PC-027 | Render the compact courier without giving presentation ownership of copy or payload policy | 027 AssistantHost | static presentation, control projection and separate interaction-geometry frontend tests |
| PC-028 | Keep the latest completed Project folder private and issue validated one-shot native drag attempts | 028 TransferPayload | candidate identity, retry, concurrency, stale attempt, symlink, reset and path-privacy tests |
| PC-029 | Accept and maintain an ordered ALS work queue without parsing or copying | 029 CourierWorkQueue | validation, ordering, deduplication, removal and path-redaction tests |
| PC-030 | Turn a live queue into immutable module-024 waves and one versioned collection | 030 CourierCollectionOrchestrator | wave immutability, during-run intake, drain-boundary, merge, revision and result-isolation tests |
| PC-031 | Expose one validated collection snapshot as a native copy-only multi-directory drag | 031 UniversalPayloadDrag | multi-path validation, copy-only adapter, one-shot attempt, retry, stale revision and path-privacy tests |
| PC-032 | Deliver one collection snapshot to an explicitly configured local directory without overwrite | 032 CollectionDelivery | plan, collision, staging, validation, repeat-delivery, source-immutability and path-redaction tests |

## Next Capability Rule

No later capability becomes active merely because it appears in a backlog.
The next module still requires an accepted product boundary, specification,
machine contract and readiness gate.

## Handoff Map

```text
ProjectDiscoveryResult + ALSReadModel
-> DependencyExtractionResult
-> PathObservationResult
-> DependencyAssessmentResult
-> PreflightReport
-> DesktopAnalyzeResult

DependencyAssessmentResult
-> CurrentPathBindingResult
-> PackagePlan v0.4
-> CurrentPathCopyResult
-> DesktopCopyPreview / DesktopCopyResult

ProjectScanResult
-> ProjectCatalogSnapshot
-> ProjectListItem / ProjectSelection
-> one-project DesktopAnalyze / DesktopCopy services
-> BatchCopyResult

macOS opened ALS URL
-> QuickCopyLaunchContext
-> CourierWorkQueue
-> immutable BatchWave
-> existing BatchCopyResult
-> CourierCollectionSnapshot
-> private TransferPayload
-> native collection drag or local collection delivery

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
| INV-013 | Desktop UI does not implement domain analysis policy | ADR-006, module 016 contract and service tests |
| INV-014 | A missing recorded-path asset may be omitted without authorizing search or rewrite | modules 009, 013, 017 tests and manifest omission records |
| INV-015 | Desktop execution is bound to the previewed ALS hash and explicit consent | module 018 stale-preview and consent tests |
| INV-016 | Current-path audio is copied without inventing content identity or repeatedly hashing audio | ADR-007, modules 008-014 and 017 policy-specific tests |
| INV-017 | Physical ProjectFolder, concrete LiveSet and future ProjectWork identity remain separate | ADR-011, modules 020-023 contracts and tests |
| INV-018 | Partial Project scan coverage cannot silently remove or hide previously known projects | ADR-011, modules 020-022 contracts and tests |
| INV-019 | Batch mode reuses one-project policy and isolates every selected Set as a separate job | ADR-011, module 024 contract and tests |
| INV-020 | Quick mode is presentation and launch routing only; it reuses module 018 preview and execution policy | ADR-012, module 025 contract and tests |
| INV-021 | External handoff can expose only the latest successful final Project folder, uses copy-only native drag and never claims remote upload success | ADR-013, module 026 contract and tests |
| INV-022 | Character presentation cannot own capability effects, private payload paths or provider policy | ADR-014, modules 025/027 contracts and tests |
| INV-023 | React cannot nominate the native folder payload; each drag attempt is revalidated and one-shot | ADR-014, modules 026/028 contracts and tests |
| INV-024 | A live queue never mutates an active module-024 request; new intake becomes a later immutable wave | ADR-015, modules 029/030 contracts and tests |
| INV-025 | Delivery binds one immutable collection revision and never consumes or mutates source Project copies | ADR-015, modules 028/031/032 contracts and tests |
| INV-026 | Local cloud-folder delivery proves only a validated local copy, never remote synchronization | ADR-015, module 032 contract and tests |

## Review Rule

Update this map only when a change adds or alters a product capability, public
handoff, safety invariant, or its executable evidence.
