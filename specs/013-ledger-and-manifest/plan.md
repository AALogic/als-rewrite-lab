# Plan: 013 PrivateLedger / PackageManifest

1. Define separate private and portable data models.
2. Build portable facts only from validated relative file records.
3. Run structural absolute-path privacy checks before writing.
4. Write both JSON artifacts atomically without clobbering.
5. Make repeat execution byte-idempotent and auditable.

