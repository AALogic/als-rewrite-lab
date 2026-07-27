# Fixture Contract: 011 ALSRewriter

Synthetic gzip/XML fixtures contain two active SampleRefs and one historical
OriginalFileRef. One active reference is approved for rewrite. Tests make all
writes inside temporary staging and compare the staged file before/after. No
real Ableton project is modified by unit tests.

