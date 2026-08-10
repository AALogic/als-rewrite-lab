# Deferred Plan: Performance Trace Sidecar

Status: deferred laboratory tooling idea
Date: 2026-08-04
Scope: performance investigation only; not a product module or release feature

## 1. Purpose

Build an external, locally operated tracer that explains how ALS Rescue spends
time during one-project and batch copy operations. The tracer should reveal the
order, duration and repetition of filesystem and CPU work before any product
optimization is attempted.

The tracer is evidence-gathering infrastructure. It must not become part of the
application contract, package manifest, installer or ordinary user workflow.

## 2. Isolation Boundary

The laboratory should live beside the product repository:

```text
Documents/New project/
  als-rewrite-lab-windows/       product repository
  als-rescue-performance-lab/    external tracer and private run data
```

Requirements:

```text
- do not modify product contracts or public result models
- do not write tracing data into generated Ableton Project packages
- do not alter source ALS or source audio
- use a separate Cargo target directory for a symbolized profiling build
- keep raw traces and native paths outside Git
- keep instrumentation failure independent from product success or failure
```

## 3. Measurement Modes

### Light Trace

Use for representative elapsed-time and resource measurements with low
observer overhead.

Capture:

```text
- total and per-observed-run wall time
- process CPU and resident memory samples
- system disk throughput samples
- input/output file count and byte totals
- source and destination volume context
```

### Deep Trace

Use to explain behavior and repetition. Its elapsed time is not the product
baseline because detailed filesystem tracing can add overhead.

Capture:

```text
- filesystem operations and timestamps
- open/read/write/stat/fsync/rename/mkdir/read-dir counts
- repeated operations against the same normalized path
- symbolized Rust stack samples
- slowest files and operation classes
- staging, manifest and final-package path lifecycles
```

## 4. Available macOS Sources

The host currently provides:

```text
fs_usage   exact filesystem activity; requires user-approved sudo
sample     statistical process stack sampling
iostat     system disk activity
ps         CPU, memory and thread observations
vm_stat    memory pressure context
```

`xctrace` is present only as a Command Line Tools stub and is not currently
usable without installing full Xcode. Full Instruments is optional and is not
required for tracer v0.1.

## 5. Planned Run Workflow

```text
build an optimized application with debug symbols into the lab target dir
-> launch the exact lab bundle and identify its PID and binary hash
-> select copied test Projects and a fresh destination in the UI
-> start light or deep collectors
-> execute the batch once
-> detect completion or accept an explicit stop signal
-> stop every collector cleanly
-> inventory generated files and sizes
-> parse and correlate raw observations
-> produce machine-readable and human-readable reports
```

The user action should be limited to operating the application and approving
`sudo` when a deep `fs_usage` capture is requested. Codex should manage the
remaining setup, collection, parsing and analysis.

## 6. Run Artifacts

Each run should use a new private directory:

```text
runs/<date>_<run-id>/
  raw/
    filesystem.log
    cpu-sample.txt
    process-metrics.csv
    disk-metrics.csv
    environment.json
  performance-report.json
  timeline.csv
  findings.md
```

`performance-report.json` should contain stable categories and normalized
identifiers rather than native paths. Raw files may retain native paths for
local diagnosis but must remain private and untracked.

## 7. Required Analysis

The analyzer should report:

```text
- total files and bytes copied
- observed throughput in MiB/s
- time and operation counts by category
- repeated reads of the same ALS
- repeated visits to the same tree or file
- fsync count and accumulated observed latency
- source-read, staging-write and final-rename sequence
- XML/gzip/rewrite-related stack hot spots
- validation, manifest and promotion-related hot spots
- top slow operations and files by anonymized ID
- likely I/O-bound, CPU-bound or metadata-bound classification
- optimization hypotheses ranked by evidence and expected impact
```

The report must distinguish confirmed observations from inferences. A sampled
CPU stack indicates relative hotness, not exact invocation count. Detailed
filesystem tracing can explain exact file-operation order but can distort total
runtime.

## 8. Safety And Validity Tests

Before trusting the tracer:

```text
- confirm it writes only inside its private lab directory
- confirm source Projects remain unchanged
- confirm tracing does not change generated package content
- compare tracing disabled versus Light Trace overhead
- label Deep Trace elapsed time as non-baseline
- verify all collector processes terminate after a run
- verify the sanitized report contains no user-home paths or Project names
- repeat the same test at least three times before drawing timing conclusions
```

## 9. Escalation Path

Start with external observation only. If external traces cannot separate two
internal stages, create a disposable instrumented snapshot or temporary
performance-only build outside the product repository. Do not merge diagnostic
hooks into the product merely to answer a one-time optimization question.

## 10. Resume Trigger

Return to this plan when performance optimization becomes the active task and
a fresh, reproducible batch fixture plus destination directory are available.
The first implementation should build the external lab, run one Light Trace
and one Deep Trace, then review evidence before changing product code.
