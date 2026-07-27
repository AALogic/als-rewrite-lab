# Plan: 015 LaboratoryPipeline

1. Reject unsafe or conflicting request paths before any stage runs.
2. Compose read-only discovery, reading, extraction, observation, assessment,
   preflight, inventory, resolution, and planning.
3. Call write-capable stages only for a ready immutable plan.
4. Preserve completed outputs and structured failure context at every stop.
5. Prove source immutability and final package behavior with one end-to-end
   synthetic fixture.
6. Expose the same request through an explicitly named laboratory CLI command.
