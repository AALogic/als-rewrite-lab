# Plan: 018 DesktopCopyApplicationService

1. Define preview, execution, result and error contracts.
2. Delegate preview and execution to CurrentPathCopyPipeline.
3. Bind execution to the preview source hash and target.
4. Expose narrow Tauri commands.
5. Add desktop target selection, confirmation and result UI.
6. Transport a backend-computed PlanFingerprint through preview and verify it
   before any execute write.
7. Lock representative Rust/TypeScript wire payloads with shared fixtures.
8. Produce one redacted diagnostic for preview and execution and let the UI
   copy it without interpreting domain errors.
