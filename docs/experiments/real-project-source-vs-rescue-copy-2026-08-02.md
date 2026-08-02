# Real Project Source Versus Rescue Copy, 2026-08-02

## Status

`CONFIRMED` for the observed Live 11.3.43 Set and the generated package.

The comparison was read-only. It did not modify the source Set, copied audio,
generated Set, manifest or Ableton-created sidecars.

## Scope

The experiment compared:

- one selected source Set inside an existing multi-Set Ableton Project;
- the package created by the Desktop current-path copy flow;
- the same package after it had been opened successfully in Ableton Live.

Private absolute paths are intentionally replaced with logical names in this
report.

## Source Environment

The selected source Project folder contained:

- 17 files and 25,699,135 bytes;
- 9 ALS files, including alternate Sets and Backups;
- 3 local audio files and 3 local `.asd` sidecars;
- one product-specific metadata file unrelated to Ableton portability.

The selected Set did not actively reference audio inside that Project folder.
Its 3,744 active references resolved to 29 unique audio paths elsewhere under
the user's home directory. The unique source files were spread across eight
location groups, including Splice storage, Downloads and other Ableton Project
folders. Their combined audio size was 203,704,339 bytes.

This is direct evidence for the product problem: the Set is small and appears
inside one Project folder, while its actual active audio dependencies are
distributed across unrelated locations.

## Generated Package

Before Ableton opened it, ALS Rescue created:

- one rewritten ALS;
- 29 audio files under `Samples/Imported`;
- an empty `Ableton Project Info` marker directory;
- one public package manifest;
- no Backups, alternate Sets, unrelated local audio or third-party metadata.

The generated payload before Ableton-created sidecars was 208,063,682 bytes:

| Component | Bytes |
| --- | ---: |
| Copied audio | 203,704,339 |
| Rewritten ALS | 3,124,033 |
| Public manifest | 1,235,310 |

This behavior is a portable copy of the selected Set, not a clone of the entire
source Ableton Project folder. The product UI and documentation should retain
that distinction.

## Content Identity Evidence

- All 29 copied audio files existed at the recorded source paths.
- All 29 destination files existed.
- All 29 source/destination SHA-256 comparisons matched.
- The public manifest's source ALS hash matched the source ALS.
- All 30 files listed in the manifest still matched their recorded size and
  SHA-256 after the package was opened in Ableton.
- No copied audio bytes changed when Ableton opened the package.

## ALS Semantic Diff

The source and generated ALS files both contained 651,645 XML elements.

There were no differences in:

- element tags;
- element count or child count;
- attribute names;
- text or tail content;
- active-reference filenames, extensions, original sizes, CRC values,
  durations, sample rates, usage contexts or XML locators;
- 982 historical references;
- 368 non-audio dependency signals.

Exactly 11,232 XML attribute values changed:

| Field | Changed occurrences |
| --- | ---: |
| `Path/Value` | 3,744 |
| `RelativePath/Value` | 3,744 |
| `RelativePathType/Value` | 3,744 |

Every generated active reference used `Samples/Imported/<filename>` and
`RelativePathType=3`. No structural or unrelated semantic XML change was
observed.

The decompressed XML became 295,328 bytes smaller and the gzip ALS became
44,237 bytes smaller. This is explained by replacing long external absolute and
relative paths with shorter project-local paths; it is not evidence of removed
project content.

## Ableton-Created ASD Sidecars

ALS Rescue did not list or copy `.asd` sidecars. After the generated Set was
opened, Ableton created 29 `.asd` files next to the 29 imported audio files.
Their timestamps were later than the package files and manifest.

All 29 source audio files also had source-side `.asd` siblings. Comparing the
source and newly generated sidecars showed:

- 29 source/target sidecar pairs;
- 9 identical SHA-256 pairs;
- 16 pairs with equal size;
- 20 pairs with different content hashes;
- 13 pairs with different sizes.

The generated sidecars added 6,172,717 bytes. This proves that sidecars are not
required merely to open this Set, because Ableton recreated them. It does not
yet prove that omitting source sidecars is always semantically equivalent for
all analysis, warp, cache or cross-version behavior. Sidecar copying remains a
domain test, not an authorized product rule.

## Required-Asset Count Versus Unique Files

The pipeline reported 30 required assets but created 29 copy operations. This
was not a lost file.

One exact source path appeared 822 times with two reference claims:

- 738 occurrences carried a positive original size and CRC;
- 84 occurrences carried `OriginalFileSize=0` and `OriginalCrc=0`.

DependencyAssessment correctly kept the conflicting claims separate, while
PackagePlanner correctly bound both claims to the same observed file and copied
the bytes once. The current Desktop count is nevertheless confusing: it labels
claim groups as assets without also showing unique files.

Recommended contract language:

- `reference_occurrence_count = 3744`;
- `required_claim_group_count = 30`;
- `unique_source_file_count = 29`;
- `copy_operation_count = 29`.

Do not collapse conflicting claims silently. Expose the different meanings
instead.

## Repeat-Copy Finding

A read-only preview used the generated ALS as input for a second destination.
It returned:

```text
preview_status: incomplete_copy_preview_ready
required_asset_count: 30
copy_asset_count: 29
rewrite_reference_count: 0
omitted_asset_count: 3744
```

The current rewrite ruleset accepts source `RelativePathType=1` and deliberately
does not accept the generated `RelativePathType=3`. The tool can therefore
create a working package but cannot yet repackage its own output.

Two separate issues must be resolved:

1. Product capability: decide and test whether already project-local Type 3
   references are copied unchanged, rewritten to a second package, or reported
   as already portable.
2. Contract semantics: `omitted_asset_count=3744` is occurrence-shaped, not
   asset-shaped, and conflicts with `required_asset_count=30`. The output model
   needs explicit occurrence, claim-group and unique-file counts.

## Manifest Size Finding

The pretty public manifest was 1,235,310 bytes; compact JSON would be 919,355
bytes. Its compact `rewrites` array alone occupied 912,428 bytes.

All 3,744 rewrite entries repeated the same:

- three changed fields;
- ruleset version;
- verified status.

Only operation IDs and XML locators varied. This detail is auditable but highly
repetitive. A future manifest schema may retain exact auditability while using
grouped rewrite summaries, locator ranges or a separate detailed diagnostic
artifact. This is a size and maintainability improvement, not a correctness
blocker.

## Filesystem Metadata Finding

Audio contents were identical, but the copy is currently content-preserving,
not metadata-preserving:

- none of the 29 modification timestamps were retained;
- 10 source permission modes were normalized to the destination creation mode;
- macOS Finder, download-origin and quarantine attributes were not retained;
- macOS added a provenance attribute to generated files.

This did not prevent Ableton from opening the observed package. The behavior is
currently an implicit consequence of exclusive stream-copy creation. Before a
commercial release, the project should define an explicit cross-platform file
metadata policy and test it on macOS and Windows. A sensible default is likely
to preserve audio bytes, use safe user-writable regular-file permissions and
avoid copying platform-specific quarantine or Finder metadata, but this remains
a product/security decision.

## Conclusions

The primary copy and rewrite behavior is strongly supported by evidence:

- source bytes were preserved exactly;
- only authorized ALS fields changed;
- all generated active paths point into the package;
- Ableton opened the package;
- manifest-listed files remained unchanged after opening.

The experiment also found four follow-up areas:

1. Correct the count vocabulary across contracts and Desktop UI.
2. Define and test repeat-copy behavior for project-local Type 3 references.
3. Decide the `.asd` sidecar policy through a controlled Ableton comparison.
4. Define the cross-platform filesystem metadata policy.

Manifest compaction is useful but lower priority than the contract and repeat-
copy findings.

## Recommended Next Experiments

1. Create Ableton Collect All and Save output for the same source Set and compare
   its folders, sidecars and FileRef fields against ALS Rescue.
2. Copy the generated package to a second parent directory and open it without
   relying on its original generated location.
3. Save the generated Set once in Ableton, then compare the Ableton-saved ALS
   with the source and ALS Rescue versions.
4. Test a second-generation package policy for Type 3 references.
5. Compare source-side `.asd`, regenerated `.asd` and a package where sidecars
   are deliberately copied before opening.
