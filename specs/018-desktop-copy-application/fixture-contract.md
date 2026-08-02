# Fixture Contract: 018 DesktopCopyApplicationService

Reuse the complete and partial copied Live 11.3 fixtures from module 017.
Preview must not change the fixture tree. Execute requires explicit consent,
fresh output paths and an unchanged source hash. Results expose counts and the
final target but no private ledger content.

Add preview-drift fixtures where audio appears, disappears or changes size
without changing the ALS. Execution must reject them before staging. A
same-path replacement with the same size remains accepted by the current-path
policy and must not introduce audio hashing. Versioned JSON wire fixtures cover
analysis, preview, execute request and execution result payloads shared by Rust
and TypeScript.
