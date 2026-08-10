# Plan 024: BatchCopyApplicationService

Status: completed
Date: 2026-08-03

1. Define versioned prepare, execute, job, summary, diagnostic and progress contracts.
2. Implement a pure target assignment and collision pass.
3. Add orchestration around the existing one-project prepare/execute functions.
4. Add cancellation checks between sequential jobs and path-free progress events.
5. Test orchestration with injected fake one-project operations.
6. Add Tauri commands, cancellation state and progress event adapter.
7. Extend the existing Project catalog UI with batch preview, consent and results.
8. Add shared Rust/TypeScript wire fixtures.
9. Run module guard, workspace, frontend and rendered UI verification.
