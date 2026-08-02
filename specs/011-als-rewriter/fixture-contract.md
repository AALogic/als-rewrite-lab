# Fixture Contract: 011 ALSRewriter

Synthetic gzip/XML fixtures contain two active SampleRefs and one historical
OriginalFileRef. One active reference is approved for rewrite. Tests make all
writes inside temporary staging and compare the staged file before/after. No
real Ableton project is modified by unit tests.

The fixture matrix includes external Type 1 three-field rewrite and
project-local Type 3 Path-only rewrite. The latter must preserve RelativePath
and RelativePathType byte-for-byte.

Write-success cases run only where the Unix replacement adapter is supported.
Other platforms assert the documented fail-closed replacement result.
