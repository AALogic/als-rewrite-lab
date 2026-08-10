# Plan 030: CourierCollectionOrchestrator

Status: accepted, v0.2
Date: 2026-08-06

1. Add collection, wave and delivery-snapshot contracts.
2. Compose module 029 through private transition helpers.
3. Freeze immutable waves and map them to module-024 requests in the Tauri
   adapter, leaving module 024 unchanged.
4. Merge validated module-024 results and create later waves until the queue is
   empty.
5. Add concurrency-boundary tests with a controlled fake batch runner.
6. Add path-redacted frontend projection at the Tauri boundary.
7. Add a provider-independent handoff state bound to collection ID and revision.
8. Make completed handoff terminal for the active order and support an explicit
   reopen of the same immutable delivery snapshot.
9. Let the application adapter rotate post-completion intake into a fresh
   collection and hold intake arriving during handoff for that next order.
