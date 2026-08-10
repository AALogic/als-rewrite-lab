# Fixture Contract 031: UniversalPayloadDrag

Status: accepted
Date: 2026-08-05

Use two fresh Project directories, one missing directory, one regular file and
one platform-supported symlink. The successful attempt preserves the two
directory paths in snapshot order. Each invalid member fails before native drag
installation. Serialized commands, events and failures may contain IDs and
counts but no fixture path or display name.

Terminal outcome fixtures prove that `dropped`, `cancelled` and `failed` map to
completed, available and retryable collection handoff states respectively.
