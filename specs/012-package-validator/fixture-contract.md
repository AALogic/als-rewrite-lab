# Fixture Contract: 012 Validator / SemanticDiff

Tests create a source ALS and audio file in a temporary directory, then invoke
the real StagingExecutor and ALSRewriter before validation. Mutation cases alter
only temporary outputs or sources. No user project or final target is touched.

