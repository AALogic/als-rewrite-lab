# Fixture Contract 029: CourierWorkQueue

Status: accepted
Date: 2026-08-05

Tests create temporary regular files named `A.als`, `B.als` and `C.ALS` plus a
non-ALS file, directory and platform-supported symlink. File bytes are arbitrary
because this module validates intake metadata only and never opens ALS content.

Every safety test records source metadata and bytes before and after queue
operations. No fixture path may appear in serialized `CourierQueueError`.
