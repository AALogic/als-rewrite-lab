# Fixture Contract: 012 Validator / SemanticDiff

Tests create a source ALS and audio file in a temporary directory, then invoke
the real StagingExecutor before validation. Unix tests invoke the supported real
ALSRewriter adapter; other platforms build the same approved rewritten fixture
directly so Validator coverage stays native while ALSRewriter separately proves
its fail-closed replacement behavior. Mutation cases alter only temporary
outputs or sources. No user project or final target is touched.

A dedicated Type 3 fixture allows only the `Path` attribute range to differ;
changing RelativePath or RelativePathType must fail semantic validation.
