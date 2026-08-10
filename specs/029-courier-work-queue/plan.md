# Plan 029: CourierWorkQueue

Status: accepted
Date: 2026-08-05

1. Add queue contracts and a private path-key helper.
2. Validate selections at the backend boundary without opening ALS content.
3. Implement deterministic add, remove, transition and snapshot operations.
4. Add unit tests using copied temporary ALS fixtures and symlink fixtures.
5. Export only the queue API required by module 030.
6. Run module guard and workspace tests.
