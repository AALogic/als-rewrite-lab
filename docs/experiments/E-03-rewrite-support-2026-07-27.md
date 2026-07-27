# E-03: Narrow ALS Rewrite Support Matrix

Status: partial evidence; relocation confirmed, rescue-package rule experimental
Date: 2026-07-27
Scope: laboratory copies only, macOS, Ableton Live 11.3.x

## Question

Can a copied ALS be changed through a small, versioned allowlist while proving
that every unrelated semantic value stayed unchanged?

## Evidence Inputs

Private copied artifacts:

~~~text
01_cziki_before_cas.als
02_cziki_after_cas.als
03_cziki_reconstructed.als
~~~

The evidence harness is `tools/als_semantic_diff.py`. It compares active
`SampleRef/FileRef` values by snapshot-bound locator and hashes a canonical
XML representation after redacting only explicitly allowed active fields. Its
JSON output contains no filenames or raw paths.

Private reports:

~~~text
evidence/e03_relocate_semantic_diff.json
evidence/e03_cas_oracle_semantic_diff.json
~~~

## Result A: Self-Contained Relocation

~~~text
active refs: 153 -> 153
active locator sequence equal: yes
active Path changes: 33
all other active fields changed: 0
historical refs: 47 -> 47
historical field changes: 0
canonical semantic hash equal after redacting active Path: yes
manual Ableton open from the original experiment: success
~~~

Accepted narrow rule:

~~~text
ruleset: live11_3_relocate_path_only_v0.1
document: MajorVersion 5, MinorVersion 11.0_11300
observed creator: Ableton Live 11.3.43
input reference: direct SampleRef/FileRef
precondition: RelativePathType 3
precondition: safe RelativePath below Samples/
precondition: copied target file exists under the new project root
change: active Path only
preserve: every other active and historical field
~~~

The rule is confirmed only for the tested laboratory family. It is not a claim
about every Live 11 Set, older/newer Live, Windows, Packs, User Library or
unknown SampleRef contexts.

## Result B: Ableton Collect All And Save Oracle

~~~text
active refs: 153 -> 153
active locator sequence equal: yes
Path changed: 33
RelativePath changed: 33
RelativePathType changed: 33
OriginalFileSize changed: 22
OriginalCrc changed: 22
historical RelativePath changed: 32
historical OriginalFileSize changed: 21
historical OriginalCrc changed: 21
canonical hash equal after active-field redaction: no
~~~

Ableton CAS is broader than a minimal active-path rewrite. This oracle supports
the observed direction `external -> Samples/Imported -> RelativePathType 3`,
but it does not prove that omitting Ableton's metadata and historical updates
is valid for a generated rescue package.

## Snapshot-Bound Locator Rule

`SampleRef[index]/FileRef` remained one-to-one in both controlled pairs. It may
be used only together with:

~~~text
source ALS SHA-256
expected old Path / RelativePath / RelativePathType
supported document version
supported rewrite ruleset
~~~

The index is not a permanent identity across an arbitrary Ableton save. A plan
becomes stale when the source ALS hash changes.

## Support Matrix

| Rule | State | Allowed now |
| --- | --- | --- |
| Live 11.3 self-contained relocation, Type 3, Path-only | CONFIRMED_LAB | laboratory implementation and validation |
| Live 11.3 external Type 1 -> Samples/Imported Type 3 | EXPERIMENTAL | one controlled generated fixture; requires semantic diff and manual Live open |
| Same-name different-content target collision | UNKNOWN | block |
| Core/Packs Type 5 rewrite | UNSUPPORTED | leave unchanged |
| User Library, presets, plugins, M4L | UNSUPPORTED | report only |
| Live 9/10/12 or Windows rewrite | UNKNOWN | block |
| Unknown SampleRef context or locator mismatch | UNSUPPORTED | block |

## Next Controlled Experiment

Use the copied one-reference external fixture. Copy its selected audio source to
`Samples/Imported`, change only active `Path`, `RelativePath` and
`RelativePathType`, preserve all other fields, run structural and semantic
validation, then manually open the result in Ableton Live 11.3. Promotion from
`EXPERIMENTAL` requires all gates to pass.
