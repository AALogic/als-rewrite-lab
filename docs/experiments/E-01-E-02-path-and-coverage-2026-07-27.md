# E-01 / E-02: Path Semantics And Recognized Audio Coverage

Status: accepted for a narrowed Live 11/macOS read-only implementation
Date: 2026-07-27
Scope: evidence required before ProjectDiscovery and PathObservation

## 1. Questions

E-01 asks:

```text
Which ALS path forms may safely produce a local filesystem candidate?
What evidence identifies an Ableton Project root?
```

E-02 asks:

```text
Which audio dependency structures does ALSReader recognize?
How honestly may the product describe parser completeness?
```

## 2. Evidence

Private inputs were copied read-only into an isolated laboratory. The committed
repository contains no ALS or audio content. Evidence was reduced to hashes,
counts and XML element names.

Evidence sets:

```text
one Live 11 Set before Collect All and Save
the corresponding Set after Collect All and Save
one independently reconstructed portable copy
four focused structural Sets
twenty additional real-world ALS documents
official Ableton Live 11 file-management documentation
synthetic cross-platform and path-safety cases
```

The aggregate harness is:

```text
tools/als_evidence.py
```

It intentionally emits no project filename and no raw path value.

Official product evidence:

- Live Projects are identified by an `Ableton Project Info` directory.
- a Set may live in any nested directory inside a Project, so `parent(ALS)` is
  not a reliable Project root;
- Live's File Manager lists one line per referenced file and can distinguish
  Project, User Library, external and missing locations;
- Collect All and Save copies external audio into the Project and can include
  Core Library or Pack content.

Sources:

```text
https://www.ableton.com/en/live-manual/11/managing-files-and-sets/
https://help.ableton.com/hc/en-us/articles/115000915804-Saving-Projects
https://help.ableton.com/hc/en-us/articles/209775645-Collect-All-and-Save
```

## 3. E-01 Results

Across the 20-document corpus, active audio path shapes were:

```text
RelativePathType 0:
  46,842 occurrences
  Path is project-relative text
  RelativePath is empty

RelativePathType 1:
  2,316 occurrences
  Path is an absolute external path
  RelativePath escapes through parent segments

RelativePathType 3:
  64,362 occurrences
  Path is an absolute path
  RelativePath is a safe Project-relative path

RelativePathType 5:
  733 occurrences
  Path is an absolute Core Library path in the observed corpus
  RelativePath is library-relative, not Project-relative
```

The controlled before/after CAS pair preserved 153 reference occurrences and
126 unique raw paths. CAS changed 33 external occurrences representing six
unique audio files from type 1 to type 3. The six copied files appeared under
`Samples/Imported`. The 120 type 5 Core Library occurrences remained type 5 in
this observed run.

Accepted candidate rules for PathObservation v0.2:

```text
all types:
  an absolute raw Path may produce a direct candidate only when it is native
  and checkable on the current host

type 0:
  a safe relative raw Path may be joined to a confirmed Project root

type 3:
  a safe raw RelativePath may be joined to a confirmed Project root

type 1:
  do not join the observed parent-escaping RelativePath to the Project root
  in v0.2; retain and check the native absolute raw Path only

type 5:
  do not join its library-relative value to the Project root; retain and check
  the native absolute raw Path only

unknown types:
  never invent relative semantics; preserve evidence and observe only a native
  absolute raw Path
```

Project root rule:

```text
search ancestors of the selected ALS for an exact Ableton Project Info
directory; otherwise project root remains unknown unless explicitly selected
by the user or supplied by a controlled fixture
```

Safety policy:

```text
lexically reject absolute, prefixed or parent-escaping values before a
Project-relative join
use symlink_metadata and do not follow symlinks
preserve raw text exactly
foreign-platform paths are evidence but are not checked on the current host
NFC/NFD and case variants are not normalized into equality
```

This policy is intentionally narrower than Ableton's internal resolver. It
supports read-only evidence without claiming to reproduce undocumented logic.

## 4. E-02 Results

In the 20-document corpus:

```text
active direct SampleRef/FileRef: 114,253
historical OriginalFileRef/FileRef: 3,773
other FileRef structures: 3,242
```

Recognized active audio occurred in these observed contexts:

```text
Arrangement and take-lane AudioClip/SampleRef/FileRef
Session ClipSlot AudioClip/SampleRef/FileRef
MultiSampleMap SampleRef/FileRef
```

The other non-historical FileRef structures were predominantly Live device,
Rack, AU preset and default-preset references. They remain report-only
non-audio signals and are not silently promoted to audio dependencies.

The controlled CAS pair gave a one-to-one occurrence comparison:

```text
before: 153 recognized occurrences, 126 unique raw paths
after:  153 recognized occurrences, 126 unique raw paths
copied external files: six
recognized type 1 -> type 3 unique paths: six
```

Accepted product language:

```text
recognized audio dependencies
recognized reference occurrences
recognized unique path groups
```

Rejected product language:

```text
all Ableton dependencies
complete compatibility with every Live version or device
File Manager equivalence
```

Live 11 UI oracle comparison remains a useful regression layer, but lack of UI
automation must not cause the parser to claim completeness. New ALS structures
remain unsupported or report-only until a controlled fixture confirms them.

## 5. Gate Decision

E-01 is clear for the narrowed PathObservation v0.2 rules above.

E-02 is partial but sufficient for a read-only vertical slice:

```text
support recognized direct SampleRef/FileRef audio structures
preserve other dependency signals separately
report coverage as recognized, never universal
add new contexts only through evidence and fixture tests
```

These findings allow ProjectDiscovery, PathObservation and a conservative
DependencyAssessment to proceed. They do not authorize disk-wide scanning,
asset identity selection, copying or ALS rewrite.
