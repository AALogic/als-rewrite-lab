# Module Spec 001: ALSReader

Status: draft v0.2 contract update; v0.1 implementation exists as read-only diagnostic slice  
Date: 2026-06-09  
Owner: Product Office / Codex  
Implementation target: Rust core + CLI  
Product source: `PRODUCT_SPEC.md`

## 1. Responsibility

`ALSReader` reads an Ableton `.als` file and extracts structured facts from it.

It is the first technical module in the product pipeline.

Primary responsibility:

```text
Turn an Ableton Live Set file into ALSReadModel v0.2 without modifying any
input file or making package/rewrite decisions.
```

`ALSReader` is not the product itself. It is a safe read-only foundation for:

```text
DependencyExtractor
PathVerifier
DependencyClassifier
PreflightReportBuilder
SampleMatcher
PackagePlanner
ALSRewriter
Validator / SemanticDiff
ManifestWriter
```

## 2. Product Traceability

Product promise:

```text
Help a music producer understand, recover and safely package Ableton projects
whose audio dependencies are spread across messy local folders.
```

Supported use case:

```text
Inspect one selected Ableton project and extract dependency facts needed for
later preflight, recovery, package planning and safe rewrite.
```

Operation flow position:

```text
selected project
-> ALSReader
-> DependencyExtractor
-> PathVerifier
-> DependencyClassifier / Preflight
-> later matching/planning/copy/rewrite/validation/manifest
```

MVP classification:

```text
MVP
```

## 3. Key Contract Decision

`ALSReader v0.2` has two output layers:

```text
1. ALSReadModel
   Internal downstream contract.
   Rich model used by later modules.

2. User-facing JSON / CLI report
   Simplified view derived from ALSReadModel.
   Useful for CLI and human inspection.
```

Rule:

```text
User-facing JSON must not be treated as the full downstream contract.
Downstream modules consume ALSReadModel or models derived from it by explicit
module contracts.
```

Handoff discipline:

```text
ALSReader may collect more ALS facts than the next module immediately needs.
That does not mean every collected field is part of the next module's required
input contract.

Each downstream consumer must declare which ALSReadModel fields are:
  core_handoff
  supporting_evidence
  diagnostic_only
  preserve_for_future
  unknown_semantics

DependencyExtractor must depend only on the core_handoff subset until its spec
explicitly promotes another field.
```

## 4. Inputs

Primary input:

```text
path: filesystem path to an existing `.als` file
```

Allowed future input:

```text
bytes: in-memory ALS bytes with source metadata
```

Input constraints:

```text
file must exist
file must be readable
file must be gzip-decompressible
decompressed content must parse as XML
XML root must be recognized as an Ableton document or fail with structured error
```

## 5. Outputs

Primary output:

```text
ALSReadModel v0.2
```

The model contains six groups:

```text
set_metadata
active_audio_references
historical_refs
non_audio_dependency_signals
structure/context information included per ref/signal
warnings/errors
```

Fatal errors return structured errors and no trusted `ALSReadModel`.

Warnings may return with a valid `ALSReadModel`.

## 6. ALSReadModel v0.2

Top-level fields:

```text
set_metadata: SetMetadata
active_audio_references: list<ActiveAudioReference>
historical_refs: list<HistoricalReference>
non_audio_dependency_signals: list<NonAudioDependencySignal>
warnings: list<ALSReadWarning>
errors: list<ALSReadError>
```

Counts may be duplicated in `set_metadata` for diagnostics and fast reporting,
but lists are the source of truth for downstream modules.

## 6A. ALSReader Handoff Field Policy

Purpose:

```text
Prevent later modules from accidentally depending on ALSReader fields that were
collected for diagnostics, evidence preservation or future research.
```

Rule:

```text
ALSReadModel is the producer output.
DependencyExtractor does not automatically consume every ALSReadModel field.
The next module may consume only the fields classified as core_handoff for its
current contract.
```

Field maturity labels:

```text
core_handoff:
  Required by the next module to build DependencyRef v0.1.

supporting_evidence:
  Useful for scoring, reports, later matching or manual review.
  Must not be required for basic dependency extraction.

diagnostic_only:
  Useful for debugging, CLI output, audit or tests.
  Must not drive product behavior.

preserve_for_future:
  Preserved to avoid losing ALS information before we understand later modules.
  Must not be consumed without a contract change.

unknown_semantics:
  Observed ALS field whose meaning is not confirmed.
  May be displayed or carried forward as evidence, but must not drive automatic
  decisions.
```

Current minimal handoff to `DependencyExtractor v0.1`:

```text
From set_metadata:
  source_als_path
  source_file_hash
  reader_version
  als_read_model_version

From active_audio_references:
  ref_id
  source_kind
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
  rewrite_support_status
  warnings

From warnings/errors:
  warning/error code and severity when they affect trust in extracted
  active_audio_references.
```

Supporting evidence, not required for basic dependency extraction:

```text
xml_locator
is_rewrite_candidate
ableton_document_version
ableton_creator_version
decompressed_xml_size
sample_ref_count
active_audio_ref_count
historical_ref_count
non_audio_signal_count
historical_refs
non_audio_dependency_signals
```

Rewrite evidence handoff, promoted for an explicit writer consumer:

```text
SetMetadata.source_als_path
SetMetadata.source_file_hash
SetMetadata.ableton_document_version
SetMetadata.ableton_creator_version
SetMetadata.ableton_minor_version
ActiveAudioReference.ref_id
ActiveAudioReference.raw_path
ActiveAudioReference.raw_relative_path
ActiveAudioReference.relative_path_type
ActiveAudioReference.xml_locator
ActiveAudioReference.is_rewrite_candidate
ActiveAudioReference.rewrite_support_status
```

This is not part of the DependencyExtractor core handoff. PackagePlanner and
ALSRewriter may consume it only under ADR-005, tied to the exact source hash and
a versioned E-03 support rule. A locator is not stable identity across an
arbitrary Ableton save.

Diagnostic-only or preserve-for-future examples:

```text
analysis_started_at
analysis_completed_at
source_file_size
xml_root_name
warning_count
error_count
any raw context that has UNKNOWN semantics
```

Contract change rule:

```text
If DependencyExtractor, PathVerifier, SampleMatcher, PackagePlanner or
ALSRewriter begins to require any non-core_handoff field, the module spec and
contract tests must be updated first.
```

## 7. SetMetadata

`SetMetadata` contains technical facts about the ALS file and the analysis run.

Fields:

```text
source_als_path
source_als_filename
source_project_root
source_file_size
source_file_hash
analysis_started_at
analysis_completed_at
reader_version
als_read_model_version
ableton_document_version
ableton_creator_version
ableton_minor_version
ableton_schema_change_count
decompressed_xml_size
xml_root_name
sample_ref_count
active_audio_ref_count
historical_ref_count
non_audio_signal_count
warning_count
error_count
```

Rules:

```text
SetMetadata contains read/analysis facts only.
SetMetadata does not classify project readiness.
SetMetadata does not decide package or rewrite behavior.
source_project_root is a nullable v0.2 compatibility field.
ALSReader must leave source_project_root null and must not infer Project root
from the parent directory of source_als_path.
Project root belongs to ProjectDiscovery or explicit application context.
source_file_hash is required so later validation can prove the original ALS was
not changed.
```

## 8. ActiveAudioReference

An `ActiveAudioReference` represents a current active audio dependency used by
the Live Set.

Fields:

```text
ref_id
source_kind
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
is_rewrite_candidate
rewrite_support_status
warnings
```

Allowed `source_kind` initial value:

```text
sample_ref_file_ref
```

Allowed `usage_context` initial values:

```text
audio_clip
take_lane
session_clip
arrangement_clip
simpler_multisample
impulse_sample
unknown
```

Allowed `rewrite_support_status` values:

```text
supported
unsupported
unknown
requires_test
```

Rules:

```text
raw_path, raw_relative_path and relative_path_type must preserve ALS values
without guessing.
original_crc is a weak signal only and must not be treated as unique identity.
xml_context and xml_locator are required when possible because later rewrite and
semantic diff need stable context.
is_rewrite_candidate and rewrite_support_status are cautious parser facts, not
permission to perform rewrite.
```

## 9. HistoricalReference

`HistoricalReference` stores provenance/history references such as
`SourceContext/OriginalFileRef`.

Fields:

```text
ref_id
source_kind
raw_path
raw_relative_path
relative_path_type
filename
extension
original_file_size
original_crc
xml_context
xml_locator
linked_active_ref_id
usage_note
warnings
```

Rules:

```text
historical_refs are not active audio dependencies.
historical_refs must not inflate active dependency counts.
historical_refs are never rewrite candidates in MVP.
historical_refs may later be used as weak evidence for missing sample search.
```

Example:

```text
If a fixture has 0 active SampleRef/FileRef dependencies and 6 OriginalFileRef
nodes, ALSReadModel must report:

active_audio_ref_count: 0
historical_ref_count: 6
```

## 10. NonAudioDependencySignal

`NonAudioDependencySignal` records non-audio dependency evidence found in the
ALS.

It is intentionally cautious.

It says:

```text
ALSReader saw a signal.
Later modules or research decide what it means.
```

Fields:

```text
signal_id
signal_kind
name
raw_path
raw_relative_path
relative_path_type
plugin_format
plugin_identifier
plugin_version
xml_context
xml_locator
support_status
warnings
```

Allowed `signal_kind` values:

```text
plugin
ableton_preset
max_for_live
core_library
factory_pack
user_library
video
unknown_file_ref
unknown_device_ref
```

Allowed `support_status` values:

```text
report_only
unsupported
requires_test
future_supported
```

Rules:

```text
Plugin binaries are never copied by ALSReader.
Plugin preset portability remains UNKNOWN / requires_test.
Max for Live remains requires_test until domain tests confirm behavior.
Core Library / Factory Pack signals are report_only/system_dependency evidence.
User Library signals are captured for later package policy.
```

## 11. Warnings

Warnings mean the analysis can complete, but downstream modules must see the
risk, unknown or unsupported structure.

Fields:

```text
warning_id
warning_code
severity
message
xml_context
related_ref_id
evidence_status
```

Allowed `severity` values:

```text
info
warning
risk
```

Allowed `evidence_status` values:

```text
unknown
hypothesis
confirmed
rejected
```

Initial warning codes:

```text
UNKNOWN_RELATIVE_PATH_TYPE
UNKNOWN_FILEREF_CONTEXT
NON_AUDIO_FILEREF_FOUND
PLUGIN_SIGNAL_REPORT_ONLY
MAX_FOR_LIVE_REQUIRES_TEST
CORE_LIBRARY_SYSTEM_DEPENDENCY
USER_LIBRARY_POLICY_APPLIES_LATER
HISTORICAL_REF_NOT_ACTIVE
MISSING_OPTIONAL_FILEREF_FIELD
EMPTY_ACTIVE_PATH
```

Rules:

```text
Unknown structure must not be silently ignored.
Unsupported structure must be reported, not guessed.
Warnings do not necessarily fail analysis.
```

## 12. Errors

Errors mean the analysis failed or no trusted `ALSReadModel` can be produced.

Fields:

```text
error_id
error_code
severity
message
source_path
xml_context
```

Allowed `severity` values:

```text
fatal
blocking
```

Initial error codes:

```text
ALS_NOT_FOUND
ALS_NOT_READABLE
ALS_NOT_GZIP
ALS_DECOMPRESS_FAILED
ALS_XML_INVALID
ALS_UNSUPPORTED_ROOT
ALS_READ_INTERRUPTED
ALS_INTERNAL_ERROR
```

Rules:

```text
Fatal error = no trusted ALSReadModel.
Errors must be structured, not only free text.
Expected bad input must return a known error instead of panic.
```

## 13. Boundary: ALSReader vs PathVerifier

`ALSReader` does not check whether referenced sample files exist on disk.

That belongs to `PathVerifier`.

Rules:

```text
ALSReader reads ALS and preserves path facts.
PathVerifier resolves paths and checks filesystem existence.
PathVerifier compares existing files against ALS metadata when needed.
DependencyClassifier decides source categories and risks.
```

Platform portability rule:

```text
ALSReader must preserve raw ALS path strings exactly as read.
ALSReader must not normalize path separators, infer macOS vs Windows semantics,
or convert raw paths into platform-specific resolved paths.
Any platform-specific path interpretation belongs to PathVerifier or later
filesystem/path adapters.
```

Why:

```text
Filesystem existence depends on current machine, permissions, connected drives,
cloud sync state, project root and scan mode.
ALSReader should stay testable with only `.als` fixtures.
Raw path preservation keeps future macOS/Windows support possible.
```

Forbidden in ALSReader v0.2:

```text
exists_on_disk
source_category
storage_state
Downloads/User Library/Core classification policy
macOS/Windows path interpretation
sample matching
copy planning
rewrite planning
```

## 14. CLI Contract

CLI command:

```text
rescue analyze path/to/file.als
```

Behavior:

```text
reads ALS through ALSReader
always prints a JSON diagnostic view derived from ALSReadModel
prints fatal errors in structured form
does not modify any file
exits non-zero on fatal error
```

Rule:

```text
CLI JSON is a report view.
CLI JSON is not automatically the full downstream contract unless explicitly
documented as ALSReadModel serialization.
```

## 15. Fixtures

Existing fixture copies:

```text
tests/fixtures/als/cziki_after_cas.als
tests/fixtures/als/cziki_before_cas.als
tests/fixtures/als/template_zero_active.als
tests/fixtures/als/kombinacja_piejo.als
tests/fixtures/invalid/not_gzip.als
```

Current known expectations are recorded in:

```text
specs/001-als-reader/fixture-contract.md
```

Fixture policy:

```text
Do not copy audio folders into ALSReader fixtures.
ALSReader must remain testable with `.als` files only.
Path existence checks belong to PathVerifier fixtures/tests.
```

Required v0.2 fixture updates:

```text
add expectations for ALSReadModel top-level groups
add expectations for active_audio_references fields
add expectations for historical_refs list/count separation
add expectations for RelativePathType 0 using corpus evidence or new fixture
add expectations for xml_context / usage_context presence when possible
add expectations for non_audio_dependency_signals if fixture exposes them
```

## 16. Acceptance Criteria

`ALSReader v0.2` is accepted when:

```text
1. It accepts a filesystem path to `.als`.
   In-memory byte input is allowed future input, not required for v0.2 closeout.
2. It rejects missing / unreadable / non-gzip / invalid XML input with structured errors.
3. It returns ALSReadModel v0.2, not only user-facing JSON.
4. It returns SetMetadata.
5. It returns active_audio_references.
6. It separates active_audio_references from historical_refs.
7. It returns non_audio_dependency_signals as report_only / requires_test.
8. It preserves raw_path, raw_relative_path and relative_path_type without guessing.
9. It preserves usage_context, xml_context and xml_locator when possible.
10. It does not check sample existence on disk.
11. It does not classify paths as Downloads/User Library/Core/etc.
12. It does not copy, delete or rewrite files.
13. It does not modify the original ALS.
14. It returns warnings for unknown structures instead of guessing.
15. It has fixture tests for known ALS files.
16. It has read-only safety tests.
17. It has contract tests proving output fields needed by downstream modules.
18. It labels which output fields are core_handoff, supporting_evidence,
    diagnostic_only, preserve_for_future or unknown_semantics.
19. It does not allow the next module to depend on diagnostic/future fields
    without an explicit contract change.
```

## 17. Required Test Cases

Minimum v0.2 tests:

```text
valid_als_returns_als_read_model
  Given a valid .als fixture
  When ALSReader reads it
  Then output contains SetMetadata, active_audio_references, historical_refs,
  non_audio_dependency_signals, warnings and errors.

active_refs_are_separate_from_historical_refs
  Given a fixture with SourceContext/OriginalFileRef
  When ALSReader reads it
  Then active_audio_references do not include historical OriginalFileRef nodes
  And historical refs are counted/listed separately.

relative_path_type_is_preserved_raw
  Given fixtures with RelativePathType values
  When ALSReader reads them
  Then raw RelativePathType values are preserved exactly, including unknown or
  newly observed values such as 0.

usage_context_or_xml_context_is_present_for_active_refs
  Given a valid ALS with active refs
  When ALSReader reads it
  Then each active ref has usage_context or xml_context when derivable.

non_audio_signals_are_reported_without_policy_action
  Given ALS structures containing plugin/preset/device/FileRef signals
  When ALSReader reads them
  Then signals are reported with support_status but no copy/rewrite policy is applied.

invalid_gzip_returns_structured_error
  Given a non-gzip file
  When ALSReader reads it
  Then it returns ALS_NOT_GZIP or equivalent structured error.

read_only_safety
  Given any valid input fixture
  When ALSReader reads it
  Then source file bytes remain unchanged.

does_not_check_filesystem_dependency_existence
  Given an ALS with referenced paths that may or may not exist locally
  When ALSReader reads it
  Then output does not contain exists_on_disk/storage_state/source_category.
```

## 18. Safety Criteria

ALSReader is a read-only module.

It must prove:

```text
no writes to source ALS
no writes to source project folder
no audio file copies
no path rewrites
no delete operations
no filesystem dependency existence checks
```

## 19. Does Not Do

Explicitly out of scope:

```text
ProjectDiscovery
PathVerifier
DependencyClassifier
PreflightReportBuilder
AssetIndexer
SampleMatcher
PackagePlanner
CopyStager
ALSRewriter
SemanticDiff
ManifestWriter
Tauri desktop UI
SQLite asset index
safe cleanup
Windows migration
Ableton bridge
```

## 20. Known Unknowns

```text
Exact stable XML locator strategy for all supported active refs.
Exact usage_context labels for every observed ALS context.
Exact handling of all non-audio FileRef/device contexts.
Meaning of RelativePathType 0.
Which XML contexts are safe rewrite candidates.
```

Evidence rule:

```text
unknown and hypothesis behavior may be reported or tested.
Only confirmed behavior may become an automatic copy/rewrite rule.
```

## 21. Downstream Consumers

Downstream modules must treat `ALSReadModel v0.2` as the reader contract.

Expected consumers:

```text
DependencyExtractor
PathVerifier
DependencyClassifier
PreflightReportBuilder
SampleMatcher
PackagePlanner
ALSRewriter
Validator / SemanticDiff
ManifestWriter
```

## 22. Next Step After Spec Acceptance

After this spec is accepted:

```text
1. Update fixture-contract.md for ALSReadModel v0.2 expectations.
2. Update tasks.md with v0.2 implementation tasks.
3. Add failing/contract tests for ALSReadModel v0.2.
4. Update Rust models from ALSAnalysis/ActiveFileRef to ALSReadModel/ActiveAudioReference.
5. Keep CLI JSON as a report view.
```
