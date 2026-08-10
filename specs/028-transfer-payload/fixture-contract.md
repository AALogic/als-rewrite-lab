# Fixture Contract 028: TransferPayload v0.2

Status: accepted
Date: 2026-08-05

Use collection revision 1 with two fresh directories, revision 2 with a third
directory, plus missing, regular-file and symlink members. Tests prove ordered
targets, latest-revision binding, whole-attempt rejection for one invalid
member, one-shot begin, retry and path-free errors. Payload validation never
enumerates or mutates directory contents.
