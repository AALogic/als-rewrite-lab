# ADR-006: Desktop Application Boundary

Status: accepted
Date: 2026-08-02

## Context

The product now needs a usable desktop window. The CLI already composed project
discovery, ALS reading, dependency extraction, path observation, assessment and
preflight reporting. Repeating that sequence in React or a Tauri command would
create two product behaviors and allow UI code to acquire domain policy.

## Decision

`DesktopApplicationService` is the single application-facing boundary for the
desktop analysis flow. Tauri and the CLI may translate user input and render
output, but they delegate the ordered domain work to this service.

The first desktop operation is read-only and one-project-at-a-time. The local
preflight may contain paths needed by the user. The separately projected
diagnostic report is redacted by default and contains no filenames, source
paths, candidate paths or project roots.

React + TypeScript + Vite is the desktop presentation layer. Tauri 2 is the
native shell. Rust remains the owner of application and domain behavior.

## Consequences

- CLI and desktop analysis share one flow.
- UI changes cannot silently alter dependency interpretation.
- Future package execution must be added as another explicit application
  service operation with its own request, result and safety gate.
- Diagnostic export remains local and user-initiated.
