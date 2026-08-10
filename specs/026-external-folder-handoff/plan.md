# Plan 026: External Provider Opener v0.3

Status: accepted
Date: 2026-08-05

1. Preserve the closed `wetransfer_web` provider mapping.
2. Replace combined provider/drag IPC with one provider-only request.
3. Remove module-028 and native-surface dependencies from module 026.
4. Keep errors and responses path-free.
5. Move native drag lifecycle and AppKit evidence to module 031.
6. Verify provider opening independently from payload state.

Live provider hover/drop remains a module-031 manual compatibility test.
