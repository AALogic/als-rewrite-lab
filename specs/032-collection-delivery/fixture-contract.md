# Fixture Contract 032: CollectionDelivery

Status: accepted
Date: 2026-08-05

Create two source Project trees containing nested ALS/audio placeholder files,
an absent destination root child, a colliding destination name, one source link
and one forced-copy-failure fixture. Record every source relative path, file
size and byte content before and after delivery.

The settings fixture uses a fresh private directory and verifies temporary-file
cleanup plus exact JSON round trip. No fixture path may enter serialized
delivery errors or aggregate error codes.

Adapter fixtures map `completed` to terminal handoff and both
`completed_with_issues` and `failed` to a retryable handoff. A stale collection
revision must not be completed.
