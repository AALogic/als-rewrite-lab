# Plan: 016 DesktopApplicationService

1. Define versioned desktop request, result, diagnostic and error contracts.
2. Compose existing read-only modules without duplicating domain policy.
3. Build a default-redacted diagnostic projection.
4. Add real gzip/XML fixture tests and read-only tree checks.
5. Route CLI preflight through the service while preserving CLI output.
6. Expose the service through a narrow Tauri command.
7. Lock representative Rust/TypeScript wire payloads with shared JSON fixtures.
8. Keep the service synchronous while the Tauri adapter schedules it off the UI thread.
