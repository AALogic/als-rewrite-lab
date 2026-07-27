# Fixture Contract: 015 LaboratoryPipeline

Automated tests create a complete synthetic ALS, audio source, scan scope,
staging root, target root, and private-ledger parent under a temporary directory.
All destructive setup operations affect only that temporary fixture.

The real-data laboratory run uses only ALS and audio copies under the dedicated
overnight laboratory directory. Original user projects and samples remain
read-only and their hashes are checked before and after the run. Private
evidence may contain local paths and must remain outside the portable package.
