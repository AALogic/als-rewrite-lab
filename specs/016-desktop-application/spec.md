# Module Specification: 016 DesktopApplicationService

Status: implementation update approved, diagnostic v0.2
Date: 2026-08-03

## Purpose

DesktopApplicationService is the application-facing boundary between UI/CLI
adapters and the existing domain modules. Its first read-only operation accepts
one selected ALS and returns a desktop analysis result without changing any
user file.

## Input

`DesktopAnalyzeRequest v0.1`:

- non-empty request ID;
- absolute source ALS path selected by the user.

## Output

`DesktopAnalyzeResult v0.1` contains:

- service and request identity;
- `analysis_complete` or `analysis_failed`;
- the existing local `PreflightReport v0.2` when available;
- `DesktopDiagnosticReport v0.2`;
- structured application errors.

The local preflight may contain filenames and paths because it remains inside
the local UI. The diagnostic report is safe to share by default: it contains
counts, statuses, risk flags, hashes and error codes, but no ALS bytes, audio
bytes, filenames, candidate paths, source paths or confirmed project roots.
Each diagnostic requirement also preserves its source category, management class
and portability status so the UI can distinguish user-managed audio from confirmed
Ableton system dependencies without reimplementing domain policy.

## Flow

```text
DesktopAnalyzeRequest
-> ProjectDiscovery
-> ALSReader
-> DependencyExtractor
-> PathObservation
-> DependencyAssessment
-> PreflightReport
-> DesktopAnalyzeResult
```

The service delegates all domain behavior. It does not reimplement parsing,
path policy, grouping, matching, packaging or rewrite logic.

Rust serialization is the source of truth for the desktop IPC wire contract.
Representative versioned JSON fixtures are consumed by both Rust tests and the
TypeScript frontend so field or status drift fails in CI.

## Safety

- Read-only operation.
- No report is uploaded or written automatically.
- Diagnostic export remains user-initiated in the desktop adapter.
- Unknown or invalid project input produces structured failure.
- The same request and unchanged filesystem snapshot produce the same result.

## Does Not Do

No candidate search, user selection, package planning, copy, rewrite,
validation, promotion, persistent index, telemetry, cloud call or UI rendering.

## Acceptance

- a valid project returns the existing preflight contract;
- diagnostic output omits private names and paths;
- missing input returns a structured error;
- source ALS and project tree remain unchanged;
- repeated analysis is deterministic;
- representative analysis and error payloads match the shared IPC fixtures;
- frontend and CLI do not duplicate the analysis sequence.

The Tauri adapter may execute this synchronous service on a blocking worker.
Async scheduling and UI progress belong to the adapter, not to this domain
service or its downstream modules.
