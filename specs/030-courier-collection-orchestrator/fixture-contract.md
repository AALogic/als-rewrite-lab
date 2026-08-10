# Fixture Contract 030: CourierCollectionOrchestrator

Status: accepted
Date: 2026-08-05

Use three resolved `ProjectSelection` fixtures and a controlled module-024
result factory. Tests simulate intake before Play, during an active wave and at
the drain transition. They compare the frozen wave before and after later
intake and prove byte-for-byte structural equality.

Result fixtures include completed, completed-incomplete, blocked, failed and
cancelled jobs. Output directories exist only where a collection item is
expected. No test modifies source ALS bytes.

Handoff fixtures bind an exact collection ID and revision to native-drag and
local-delivery channels. They cover success, cancellation, retryable failure,
stale revision, simultaneous attempts and explicit reopen of an unchanged
delivery snapshot.
