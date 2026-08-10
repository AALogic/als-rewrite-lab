# MAKID clean-room product catalog audit

Date: 2026-08-03
Observed version: MAKID 0.7.2 arm64 for macOS
Scope: read-only product, data-flow, filesystem, performance, and architecture observation
Purpose: derive independent product and architecture lessons for Ableton Project Rescue

## 1. Clean-room boundary

This audit does not recommend copying MAKID source code, database names, internal identifiers, or UI implementation. The evidence was used to reconstruct externally observable capabilities and generic engineering patterns.

Allowed outputs from this audit:

- product capabilities and workflow observations;
- independently described domain concepts;
- filesystem and performance risks;
- generic architecture patterns;
- requirements and tests for our own implementation.

The target implementation must continue to be derived from our own product specification, contracts, tests, and safety rules.

## 2. Evidence collected

The following evidence was inspected locally:

- the supplied DMG and the installed application bundle;
- application metadata, signing status, runtime architecture, and updater configuration;
- the visible desktop UI and its accessibility tree;
- the local SQLite schema and aggregate record counts;
- application logs and running processes during a discovery pass;
- hidden metadata files written into previously imported Ableton project folders;
- a controlled folder containing copied ALS files;
- the current Ableton Project Rescue product documents and implemented pipeline.

No original ALS file was edited. The controlled ALS copies retained their original SHA-256 values. The controlled folder was not successfully imported because the UI requires drag and drop that was not reliably reproducible through the available automation. Existing indexed projects, the database, the application bundle, and runtime logs provided the evidence for the remaining observations.

## 3. What MAKID is doing as a product

MAKID is primarily a local project catalog and organization tool. It is not a sample-dependency recovery engine.

Its central product model appears to be:

1. A project is represented by an Ableton project folder.
2. Each root ALS file is represented as a separate Set or session inside that project.
3. One Set is selected as the primary Set, normally based on the newest modification time.
4. The user navigates a folder tree and a searchable project table.
5. Projects can carry organizational metadata such as notes, tags, genre, progress, status, tier, and pinning.
6. Additional project files can be inventoried and selected audio can receive waveform or content metadata.
7. Collections and smart collections provide views across physical folders.

The visible UI supports project and Set browsing, configurable table columns, search, filters, notes, tags, bulk editing, collections, and project opening. Cloud, mobile, feedback, analytics, and update features also exist, so the whole application should not be treated as local-only even though the local catalog can work with local data.

## 4. High-level technical model

The application uses Electron, a local SQLite database, a background worker, filesystem watching, gzip/XML parsing, and bounded work queues.

The observed flow can be described independently as:

```text
selected scan roots
  -> bounded directory discovery
  -> project-folder recognition
  -> ALS and file inventory
  -> selected ALS metadata extraction
  -> local catalog persistence
  -> searchable UI model
  -> filesystem watcher for incremental refresh
```

The background worker separates expensive discovery and metadata work from the UI. Different operations use different concurrency limits. Filesystem events are debounced before a project is rescanned.

## 5. Project recognition behavior

The observed scanner uses an exact `Ableton Project Info` marker as a strong project-folder signal. Root ALS files are then treated as sessions belonging to that project. A limited nested search is also possible when the marker exists.

The scanner excludes several well-known directories from ordinary project inventory, including:

- `Backup`;
- `Samples`;
- `Ableton Project Info`;
- `.MAKID`;
- some Ableton library and preset directories.

This has useful performance properties, but it also means that backups and sample dependencies are outside the main catalog model. Database evidence confirmed that backup ALS files were not represented as ordinary sessions.

This is not sufficient for Ableton Project Rescue. Our product needs to know that backup Sets exist and needs to inspect active sample references. We can hide backups in the default view without discarding their existence.

## 6. ALS parsing behavior

The application reads gzip-compressed ALS XML and extracts a deliberately small metadata projection, including information such as:

- Ableton creator and document version;
- tempo;
- scene count;
- track count;
- key for supported Live versions;
- locators.

The parser does not appear to build the sample dependency graph required by Ableton Project Rescue. Plugin extraction is also not a completed core capability in the observed version.

This supports an important architecture conclusion: project catalog parsing and dependency recovery parsing are different responsibilities. The catalog needs cheap metadata suitable for lists and filtering. Rescue analysis needs accurate references, evidence, path interpretation, rewrite support, and validation.

## 7. Persistence model

The local database distinguishes at least three useful concepts:

- project folder;
- ALS Set or session;
- indexed file occurrence.

It also stores organizational metadata, associations, parsed Ableton metadata, comments, and selected audio metadata.

This separation validates a direction already discussed for our product: a physical project folder must not be confused with one ALS file, and multiple ALS versions may belong to one logical project.

The exact MAKID schema should not be copied. Our independent model should be shaped around our product needs, for example:

```text
CatalogProject
  - stable product identity
  - observed project root
  - discovery evidence
  - current availability

CatalogSet
  - one ALS occurrence
  - relative location within project
  - backup/main classification
  - observed Ableton version
  - modification metadata

ScanRun
  - requested roots
  - coverage and exclusions
  - permission failures
  - timing and cancellation state
```

Dependency references, resolved assets, package plans, and rewrite operations remain in the existing rescue pipeline and should not be merged into this catalog model.

## 8. Hidden metadata written into projects

Previously imported project folders commonly contain `.MAKID/project-data.json`. This file stores portable organization data and some machine or user linkage data. It allows project annotations to travel with the folder and supports cross-machine reconciliation.

This pattern has a real product advantage, but it conflicts with our current safety invariant that source project directories are read-only.

Recommendation for Ableton Project Rescue:

- keep the default catalog and user metadata under application-controlled storage;
- do not write metadata into every discovered source project;
- if portable metadata is needed later, expose an explicit export operation;
- place exported metadata inside a generated Rescue package, not silently inside the original project;
- record this operation in the manifest.

## 9. Signing defect found during the audit

The pristine application inside the supplied DMG has a valid Developer ID signature and is accepted by macOS assessment. The installed copy fails strict signature verification because a bundled demo project's `.MAKID/project-data.json` differs from the signed resource.

The changed values are ordinary runtime metadata such as timestamps and local identity values. This indicates that the running application wrote project metadata into a project stored inside its own signed application bundle.

This is an architecture and release-safety defect. A signed application bundle must be treated as immutable at runtime. All writable demo state should be copied to Application Support or another writable user-data directory before first use.

Lesson for our product:

- installed application resources are read-only fixtures;
- examples and demo projects must be copied out before use;
- CI should verify the final signed artifact before release;
- a launch and smoke test should be followed by another signature verification to detect accidental bundle mutation.

## 10. Discovery performance issue found during the audit

During startup discovery, the worker traversed a broad part of the user home directory, repeatedly encountered protected macOS directories, produced many permission errors, and used several CPU cores for multiple minutes.

The observed design can derive a scan root from parents of already known projects. A shallow project can therefore cause a very broad parent, including the home directory, to become a discovery root.

This is not an acceptable default for our planned full-computer project scan. User consent alone does not make an unbounded traversal operationally safe.

Our scanner should require:

- explicit volumes or scan roots represented in the request;
- platform-specific system exclusions;
- bounded traversal and cancellation;
- visible progress and current stage;
- aggregated permission errors instead of repeated log spam;
- checkpoints or resumability for long scans;
- no following of symlinks or junctions by default;
- a clear coverage report showing what was scanned and skipped;
- incremental refresh after the initial scan.

## 11. Patterns worth borrowing

The following patterns are useful independently of MAKID's implementation:

1. Separate project folders from individual ALS Sets.
2. Keep every discovered Set and choose a primary Set only as a UI convenience.
3. Use a local persistent catalog so the application does not start from zero each time.
4. Perform cheap discovery first and deeper ALS analysis later.
5. Run filesystem work outside the UI thread.
6. Use bounded queues and debounce filesystem events.
7. Present a folder tree together with a searchable and filterable project list.
8. Let the user select either projects or specific Set versions.
9. Record discovery coverage, permission failures, and freshness.
10. Refresh changed projects incrementally with a watcher.

## 12. Patterns not to borrow directly

1. Do not use `Ableton Project Info` as the only condition for acknowledging an ALS candidate.
2. Do not discard Backup Sets from the data model.
3. Do not silently write metadata into original project folders.
4. Do not derive an unbounded scan root from a known project's parent.
5. Do not mix the project catalog with dependency recovery decisions.
6. Do not hash every discovered file during the first catalog pass.
7. Do not mutate files inside the signed application bundle.
8. Do not present a scan as complete without reporting denied or skipped locations.

## 13. Recommended recognition policy for our scanner

Discovery should produce evidence and classification rather than a binary answer:

```text
confirmed_standard_project
  exact Ableton Project Info marker plus a main ALS

probable_project
  one or more ALS files in a plausible folder without the marker

backup_set
  ALS located under an Ableton Backup directory

container_candidate
  marker or project-like structure with nested ALS files

unsupported_or_unknown
  evidence is insufficient or the location cannot be inspected safely
```

Only confirmed and explicitly selected candidates should flow automatically into write-capable rescue operations. Probable and unknown candidates can still appear in the catalog and be analyzed read-only.

## 14. Relationship to the current architecture

The current `ProjectDiscovery` module starts from one explicitly selected ALS and examines its ancestry. That responsibility remains useful and should stay small.

A whole-computer catalog is a separate capability. It should not be implemented by expanding the existing selected-ALS module until it performs every kind of discovery.

Recommended boundary:

```text
ProjectCatalogScanner
  discovers candidate project folders and ALS Sets under approved roots

ProjectCatalogStore
  persists scan evidence and current observations

ProjectDiscovery
  resolves the project context of one selected ALS

existing rescue pipeline
  reads dependencies, verifies current paths, plans, copies, rewrites, validates,
  and promotes one safe project copy
```

The catalog can later feed selected Set IDs into the existing one-project pipeline. A batch orchestrator can process those jobs sequentially at first. Bounded parallelism should be considered only after measurements show a real need.

## 15. Smallest next product experiment

Before building the full database and desktop list, specify and test a read-only `ProjectCatalogScanner` over explicit roots.

Minimum request:

```text
ScanRequest
  roots
  exclusions
  maximum_depth
  consent_record
  cancellation_token
```

Minimum result:

```text
ScanReport
  candidates
  sets
  backups
  scanned_directory_count
  denied_path_count
  skipped_path_count
  coverage_status
  elapsed_time
  cancellation_status
```

Acceptance evidence should include:

- a standard Ableton project with a marker and main ALS;
- a project-like folder without the marker;
- a Backup folder containing ALS files;
- nested projects;
- an inaccessible directory;
- a symlink or junction cycle;
- cancellation during a large scan;
- macOS and Windows path fixtures;
- deterministic ordering of the report.

SQLite persistence and filesystem watching should follow only after this scanner contract is validated.

## 16. Product verdict

MAKID validates that a project catalog, Set grouping, filtering, and incremental refresh are useful recurring capabilities. These features can make Ableton Project Rescue useful before the user starts a rescue operation and can later provide the selection surface for multi-project work.

It does not invalidate our core direction. The differentiating work remains:

- understanding active sample dependencies;
- distinguishing exact-path availability from identity confidence;
- safely planning a portable copy;
- rewriting only supported references in a copy;
- validating the result;
- producing an auditable manifest;
- preserving original user data.

The strongest combined product direction is therefore:

```text
local Ableton project catalog
  -> choose project or Set versions
  -> inspect dependency health
  -> create a safe portable Rescue copy
  -> retain evidence and manifest for future maintenance
```

This uses the catalog pattern as an entry point while keeping dependency recovery as the independent core value of the product.
