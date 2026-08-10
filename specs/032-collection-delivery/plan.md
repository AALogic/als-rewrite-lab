# Plan 032: CollectionDelivery

Status: accepted, v0.2
Date: 2026-08-06

1. Add delivery request, plan, operation, result and preference contracts.
2. Implement pure validation and deterministic collision planning.
3. Implement recursive regular-file staging copy and metadata validation.
4. Promote each validated staging directory to its absent final target.
5. Add process-lifetime delivered-item tracking in the Tauri adapter.
6. Add atomic local preference storage and folder-picker fallback.
7. Seed the owner machine preferences outside the repository.
8. Add unit, integration and source-immutability tests.
9. Mark the exact collection handoff in progress before local copy starts.
10. Complete only a fully successful result; preserve retry for partial and
    failed results.
