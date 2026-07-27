#!/usr/bin/env python3
"""Copy and inspect a small ALS corpus for research.

This is an experiment helper, not product code.
It copies selected .als files into an experiment folder and analyzes the copies.
"""

from __future__ import annotations

import argparse
import collections
import csv
import gzip
import hashlib
import json
import os
import re
import shutil
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any


EXCLUDE_PARTS = {
    ".Trash",
    ".git",
    "node_modules",
    "target",
}

KNOWN_ANALYZED_HINTS = [
    "als-rewrite-lab",
    "first new Project",
    "kombinacja piejo",
    "TEMPLATE 1.0_TRUSIAK",
    "OAKS REMIX VOC",
    "remixG Project",
    "na szczycie Project",
    "Blabla_stemiki_2 Project",
    "cziki Project",
    "COXED twoja kolej",
    "calakazesplice_v2.als",
    "most%20-%20new%20location%20rewritten.als",
    "new%20location%20-%20rewritten.als",
]

INTERESTING_TAG_PATTERNS = [
    "Sample",
    "FileRef",
    "OriginalFileRef",
    "SourceContext",
    "Path",
    "Track",
    "Clip",
    "Device",
    "Plugin",
    "Vst",
    "Au",
    "Max",
    "Preset",
    "Groove",
    "Warp",
    "Automation",
    "Midi",
    "Audio",
    "Video",
    "Locator",
]

AUDIO_EXTENSIONS = {
    ".wav",
    ".wave",
    ".aif",
    ".aiff",
    ".mp3",
    ".flac",
    ".m4a",
    ".ogg",
    ".aac",
    ".sd2",
}


@dataclass(frozen=True)
class Candidate:
    path: Path
    project_key: str
    is_backup: bool
    size: int
    mtime: float


def tag_name(raw: str) -> str:
    if "}" in raw:
        return raw.rsplit("}", 1)[1]
    return raw


def value_of(element: ET.Element | None) -> str | None:
    if element is None:
        return None
    if "Value" in element.attrib:
        return element.attrib.get("Value")
    if element.text is not None:
        text = element.text.strip()
        if text:
            return text
    return None


def direct_child(element: ET.Element, wanted: str) -> ET.Element | None:
    for child in list(element):
        if tag_name(child.tag) == wanted:
            return child
    return None


def child_value(element: ET.Element, wanted: str) -> str | None:
    return value_of(direct_child(element, wanted))


def all_text_values(element: ET.Element) -> list[str]:
    values = []
    if "Value" in element.attrib:
        values.append(element.attrib["Value"])
    if element.text and element.text.strip():
        values.append(element.text.strip())
    return values


def safe_name(path: Path) -> str:
    raw = path.stem
    raw = re.sub(r"[^A-Za-z0-9._ -]+", "_", raw)
    raw = re.sub(r"\s+", " ", raw).strip()
    return raw[:90] or "untitled"


def project_key_for(path: Path) -> str:
    parts = path.parts
    for idx in range(len(parts) - 1, -1, -1):
        if parts[idx].endswith("Project"):
            return "/".join(parts[: idx + 1])
    if "Backup" in parts:
        backup_idx = parts.index("Backup")
        return "/".join(parts[:backup_idx])
    return str(path.parent)


def is_excluded(path: Path) -> bool:
    text = str(path)
    if any(hint in text for hint in KNOWN_ANALYZED_HINTS):
        return True
    return any(part in EXCLUDE_PARTS for part in path.parts)


def find_candidates(roots: list[Path]) -> list[Candidate]:
    candidates: list[Candidate] = []
    for root in roots:
        if not root.exists():
            continue
        for dirpath, dirnames, filenames in os.walk(root):
            dirnames[:] = [name for name in dirnames if name not in EXCLUDE_PARTS]
            current = Path(dirpath)
            for filename in filenames:
                if not filename.lower().endswith(".als"):
                    continue
                path = current / filename
                if is_excluded(path):
                    continue
                try:
                    stat = path.stat()
                except OSError:
                    continue
                candidates.append(
                    Candidate(
                        path=path,
                        project_key=project_key_for(path),
                        is_backup="Backup" in path.parts,
                        size=stat.st_size,
                        mtime=stat.st_mtime,
                    )
                )
    return candidates


def select_corpus(candidates: list[Candidate], limit: int) -> list[Candidate]:
    by_project: dict[str, list[Candidate]] = collections.defaultdict(list)
    for candidate in candidates:
        by_project[candidate.project_key].append(candidate)

    selected: list[Candidate] = []
    for project_candidates in by_project.values():
        project_candidates.sort(key=lambda c: (c.is_backup, -c.mtime, -c.size, str(c.path)))
        selected.append(project_candidates[0])

    # Prefer non-backup, diverse latest projects, then largest files as likely richer structures.
    selected.sort(key=lambda c: (c.is_backup, -c.mtime, -c.size, str(c.path)))
    return selected[:limit]


def copy_selected(selected: list[Candidate], copies_dir: Path) -> list[dict[str, Any]]:
    copies_dir.mkdir(parents=True, exist_ok=True)
    manifest = []
    for index, candidate in enumerate(selected, start=1):
        copy_name = f"{index:02d}__{safe_name(candidate.path)}.als"
        target = copies_dir / copy_name
        shutil.copy2(candidate.path, target)
        manifest.append(
            {
                "id": index,
                "copy_path": str(target),
                "source_path": str(candidate.path),
                "project_key": candidate.project_key,
                "is_backup": candidate.is_backup,
                "size_bytes": candidate.size,
                "mtime": datetime.fromtimestamp(candidate.mtime).isoformat(),
                "sha256": sha256_file(target),
            }
        )
    return manifest


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def classify_path(value: str | None) -> str:
    if not value:
        return "empty"
    lowered = value.lower()
    if value.startswith("/Users/"):
        if "/ableton/" in lowered and "/core library/" in lowered:
            return "absolute_ableton_core_library"
        if "/ableton/" in lowered and "/user library/" in lowered:
            return "absolute_ableton_user_library"
        if "/downloads/" in lowered:
            return "absolute_downloads"
        return "absolute_user_path"
    if value.startswith("/"):
        return "absolute_other"
    if value.startswith("../") or value.startswith("./"):
        return "relative_dot"
    if value.startswith("Samples/") or "/Samples/" in value:
        return "project_samples_like"
    if "%" in value:
        return "url_encoded_or_percent"
    return "relative_or_symbolic"


def extension_for(value: str | None) -> str:
    if not value:
        return ""
    suffix = Path(value).suffix.lower()
    if suffix:
        return suffix
    match = re.search(r"(\.[A-Za-z0-9]{2,5})(?:$|[?#])", value)
    return match.group(1).lower() if match else ""


def path_for_stack(stack: list[str]) -> str:
    return "/" + "/".join(stack)


def traverse(root: ET.Element):
    stack: list[str] = []

    def walk(node: ET.Element):
        name = tag_name(node.tag)
        stack.append(name)
        yield node, tuple(stack)
        for child in list(node):
            yield from walk(child)
        stack.pop()

    yield from walk(root)


def last_context(stack: tuple[str, ...], n: int = 7) -> str:
    return "/" + "/".join(stack[-n:])


def analyze_copy(path: Path, source_info: dict[str, Any]) -> dict[str, Any]:
    compressed = path.read_bytes()
    try:
        xml_bytes = gzip.decompress(compressed)
    except OSError as error:
        return {
            "copy_path": str(path),
            "source_path": source_info["source_path"],
            "error": f"not_gzip: {error}",
        }

    try:
        root = ET.fromstring(xml_bytes)
    except ET.ParseError as error:
        return {
            "copy_path": str(path),
            "source_path": source_info["source_path"],
            "compressed_bytes": len(compressed),
            "xml_bytes": len(xml_bytes),
            "error": f"invalid_xml: {error}",
        }

    tag_counts: collections.Counter[str] = collections.Counter()
    attr_counts: collections.Counter[str] = collections.Counter()
    attr_value_samples: dict[str, list[str]] = collections.defaultdict(list)
    path_tag_counts: collections.Counter[str] = collections.Counter()
    path_value_classes: collections.Counter[str] = collections.Counter()
    path_extensions: collections.Counter[str] = collections.Counter()
    interesting_tag_counts: collections.Counter[str] = collections.Counter()
    direct_child_sets: dict[str, collections.Counter[tuple[str, ...]]] = collections.defaultdict(collections.Counter)

    file_refs = []
    original_file_refs = []
    sample_refs = []

    parent_by_id: dict[int, ET.Element | None] = {id(root): None}
    stack_by_id: dict[int, tuple[str, ...]] = {}
    for node, stack in traverse(root):
        stack_by_id[id(node)] = stack
        name = tag_name(node.tag)
        tag_counts[name] += 1

        children_names = tuple(tag_name(child.tag) for child in list(node))
        if children_names and should_collect_child_signature(name):
            direct_child_sets[name][children_names] += 1

        for attr_name, attr_value in node.attrib.items():
            key = f"{name}@{attr_name}"
            attr_counts[key] += 1
            samples = attr_value_samples[key]
            if len(samples) < 5 and attr_value not in samples:
                samples.append(attr_value)

        if any(pattern.lower() in name.lower() for pattern in INTERESTING_TAG_PATTERNS):
            interesting_tag_counts[name] += 1

        if name.lower().endswith("path") or name in {"Path", "RelativePath"}:
            for value in all_text_values(node):
                path_tag_counts[name] += 1
                path_value_classes[classify_path(value)] += 1
                ext = extension_for(value)
                if ext:
                    path_extensions[ext] += 1

        for child in list(node):
            parent_by_id[id(child)] = node

    for node, stack in traverse(root):
        name = tag_name(node.tag)
        if name == "FileRef":
            parent = parent_by_id.get(id(node))
            parent_name = tag_name(parent.tag) if parent is not None else None
            ref = file_ref_summary(node, stack, parent_name)
            file_refs.append(ref)
        elif name == "OriginalFileRef":
            parent = parent_by_id.get(id(node))
            parent_name = tag_name(parent.tag) if parent is not None else None
            original_file_refs.append(file_ref_summary(node, stack, parent_name))
        elif name == "SampleRef":
            sample_refs.append(sample_ref_summary(node, stack))

    file_ref_context_counts = collections.Counter(ref["context"] for ref in file_refs)
    original_ref_context_counts = collections.Counter(ref["context"] for ref in original_file_refs)
    active_sample_file_refs = [
        ref for ref in file_refs if ref["parent_tag"] == "SampleRef" and ref["context"].endswith("/SampleRef/FileRef")
    ]
    non_sample_file_refs = [ref for ref in file_refs if ref["parent_tag"] != "SampleRef"]

    relative_path_type_counts = collections.Counter(
        ref["relative_path_type"] or "<missing>" for ref in active_sample_file_refs
    )
    type_counts = collections.Counter(ref["type"] or "<missing>" for ref in active_sample_file_refs)
    active_path_classes = collections.Counter(classify_path(ref["path"]) for ref in active_sample_file_refs)
    active_extensions = collections.Counter(extension_for(ref["path"]) for ref in active_sample_file_refs if extension_for(ref["path"]))
    sample_ref_child_signatures = collections.Counter(sample["direct_child_signature"] for sample in sample_refs)

    root_info = {
        "tag": tag_name(root.tag),
        "attributes": dict(root.attrib),
        "direct_children": [tag_name(child.tag) for child in list(root)[:30]],
    }

    return {
        "copy_path": str(path),
        "source_path": source_info["source_path"],
        "project_key": source_info["project_key"],
        "is_backup": source_info["is_backup"],
        "compressed_bytes": len(compressed),
        "xml_bytes": len(xml_bytes),
        "root": root_info,
        "element_count": sum(tag_counts.values()),
        "unique_tag_count": len(tag_counts),
        "top_tags": tag_counts.most_common(60),
        "interesting_tag_counts": interesting_tag_counts.most_common(),
        "top_xml_paths": [],
        "attribute_counts": attr_counts.most_common(80),
        "attribute_value_samples": dict(attr_value_samples),
        "path_tag_counts": path_tag_counts.most_common(),
        "all_path_value_classes": path_value_classes.most_common(),
        "all_path_extensions": path_extensions.most_common(),
        "sample_ref_count": len(sample_refs),
        "sample_ref_child_signatures": sample_ref_child_signatures.most_common(20),
        "sample_refs_without_direct_file_ref": sum(1 for sample in sample_refs if not sample["has_direct_file_ref"]),
        "file_ref_count": len(file_refs),
        "active_sample_file_ref_count": len(active_sample_file_refs),
        "non_sample_file_ref_count": len(non_sample_file_refs),
        "original_file_ref_count": len(original_file_refs),
        "file_ref_context_counts": file_ref_context_counts.most_common(),
        "original_file_ref_context_counts": original_ref_context_counts.most_common(),
        "active_relative_path_type_counts": relative_path_type_counts.most_common(),
        "active_type_counts": type_counts.most_common(),
        "active_path_classes": active_path_classes.most_common(),
        "active_extensions": active_extensions.most_common(),
        "active_ref_examples": active_sample_file_refs[:12],
        "non_sample_file_ref_examples": non_sample_file_refs[:12],
        "original_file_ref_examples": original_file_refs[:12],
        "direct_child_signatures": summarize_child_signatures(direct_child_sets),
    }


def file_ref_summary(node: ET.Element, stack: tuple[str, ...], parent_name: str | None) -> dict[str, Any]:
    path_value = child_value(node, "Path")
    relative_path = child_value(node, "RelativePath")
    context = last_context(stack)
    return {
        "context": context,
        "parent_tag": parent_name,
        "path": path_value,
        "relative_path": relative_path,
        "relative_path_type": child_value(node, "RelativePathType"),
        "type": child_value(node, "Type"),
        "original_file_size": child_value(node, "OriginalFileSize"),
        "original_crc": child_value(node, "OriginalCrc"),
        "default_duration": child_value(node, "DefaultDuration"),
        "default_sample_rate": child_value(node, "DefaultSampleRate"),
        "path_class": classify_path(path_value),
        "path_extension": extension_for(path_value),
        "relative_path_class": classify_path(relative_path),
        "relative_path_extension": extension_for(relative_path),
        "direct_children": [tag_name(child.tag) for child in list(node)],
    }


def sample_ref_summary(node: ET.Element, stack: tuple[str, ...]) -> dict[str, Any]:
    child_names = [tag_name(child.tag) for child in list(node)]
    return {
        "context": last_context(stack),
        "has_direct_file_ref": "FileRef" in child_names,
        "direct_child_signature": " / ".join(child_names),
    }


def should_collect_child_signature(tag: str) -> bool:
    if tag in {
        "Ableton",
        "LiveSet",
        "Tracks",
        "AudioTrack",
        "MidiTrack",
        "ReturnTrack",
        "MasterTrack",
        "SampleRef",
        "FileRef",
        "OriginalFileRef",
        "DeviceChain",
        "Devices",
        "ClipSlot",
        "AudioClip",
        "MidiClip",
    }:
        return True
    return any(pattern.lower() in tag.lower() for pattern in INTERESTING_TAG_PATTERNS)


def summarize_child_signatures(
    direct_child_sets: dict[str, collections.Counter[tuple[str, ...]]]
) -> dict[str, list[dict[str, Any]]]:
    interesting = {}
    for tag, signatures in direct_child_sets.items():
        if tag in {
            "Ableton",
            "LiveSet",
            "Tracks",
            "AudioTrack",
            "MidiTrack",
            "ReturnTrack",
            "MasterTrack",
            "SampleRef",
            "FileRef",
            "OriginalFileRef",
            "DeviceChain",
            "Devices",
            "ClipSlot",
            "AudioClip",
            "MidiClip",
        } or any(pattern.lower() in tag.lower() for pattern in INTERESTING_TAG_PATTERNS):
            interesting[tag] = [
                {"count": count, "children": list(signature)}
                for signature, count in signatures.most_common(12)
            ]
    return interesting


def aggregate_reports(reports: list[dict[str, Any]]) -> dict[str, Any]:
    aggregate_counters: dict[str, collections.Counter[str]] = {
        "tags": collections.Counter(),
        "interesting_tags": collections.Counter(),
        "file_ref_contexts": collections.Counter(),
        "original_ref_contexts": collections.Counter(),
        "active_relative_path_type": collections.Counter(),
        "active_path_classes": collections.Counter(),
        "active_extensions": collections.Counter(),
        "all_path_classes": collections.Counter(),
    }
    totals = collections.Counter()

    for report in reports:
        if "error" in report:
            totals["errors"] += 1
            continue
        totals["files"] += 1
        for total_key in [
            "compressed_bytes",
            "xml_bytes",
            "element_count",
            "unique_tag_count",
            "sample_ref_count",
            "file_ref_count",
            "active_sample_file_ref_count",
            "non_sample_file_ref_count",
            "original_file_ref_count",
            "sample_refs_without_direct_file_ref",
        ]:
            totals[total_key] += int(report.get(total_key, 0))
        merge_pairs(aggregate_counters["tags"], report.get("top_tags", []))
        merge_pairs(aggregate_counters["interesting_tags"], report.get("interesting_tag_counts", []))
        merge_pairs(aggregate_counters["file_ref_contexts"], report.get("file_ref_context_counts", []))
        merge_pairs(aggregate_counters["original_ref_contexts"], report.get("original_file_ref_context_counts", []))
        merge_pairs(aggregate_counters["active_relative_path_type"], report.get("active_relative_path_type_counts", []))
        merge_pairs(aggregate_counters["active_path_classes"], report.get("active_path_classes", []))
        merge_pairs(aggregate_counters["active_extensions"], report.get("active_extensions", []))
        merge_pairs(aggregate_counters["all_path_classes"], report.get("all_path_value_classes", []))

    return {
        "totals": dict(totals),
        "top_tags": aggregate_counters["tags"].most_common(80),
        "interesting_tags": aggregate_counters["interesting_tags"].most_common(120),
        "file_ref_contexts": aggregate_counters["file_ref_contexts"].most_common(),
        "original_ref_contexts": aggregate_counters["original_ref_contexts"].most_common(),
        "active_relative_path_type": aggregate_counters["active_relative_path_type"].most_common(),
        "active_path_classes": aggregate_counters["active_path_classes"].most_common(),
        "active_extensions": aggregate_counters["active_extensions"].most_common(),
        "all_path_classes": aggregate_counters["all_path_classes"].most_common(),
    }


def merge_pairs(counter: collections.Counter[str], pairs: list[list[Any]] | list[tuple[Any, Any]]) -> None:
    for key, count in pairs:
        counter[str(key)] += int(count)


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


def write_csv(path: Path, rows: list[dict[str, Any]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def write_markdown(path: Path, manifest: list[dict[str, Any]], reports: list[dict[str, Any]], summary: dict[str, Any]) -> None:
    lines = []
    lines.append("# ALS Structure Corpus 20")
    lines.append("")
    lines.append("Status: research experiment")
    lines.append(f"Date: {datetime.now().date().isoformat()}")
    lines.append("")
    lines.append("## Scope")
    lines.append("")
    lines.append("20 copied `.als` files were analyzed. Originals were not modified.")
    lines.append("")
    lines.append("## Selected Files")
    lines.append("")
    lines.append("| ID | Copy | Source | Size | Backup |")
    lines.append("|---:|---|---|---:|---|")
    for item in manifest:
        lines.append(
            f"| {item['id']} | `{Path(item['copy_path']).name}` | `{item['source_path']}` | "
            f"{item['size_bytes']} | {item['is_backup']} |"
        )
    lines.append("")
    lines.append("## Corpus Totals")
    lines.append("")
    for key, value in summary["totals"].items():
        lines.append(f"- `{key}`: {value}")
    lines.append("")
    add_pairs_section(lines, "Active RelativePathType", summary["active_relative_path_type"])
    add_pairs_section(lines, "Active Path Classes", summary["active_path_classes"])
    add_pairs_section(lines, "Active Audio Extensions", summary["active_extensions"])
    add_pairs_section(lines, "FileRef Contexts", summary["file_ref_contexts"][:20])
    add_pairs_section(lines, "OriginalFileRef Contexts", summary["original_ref_contexts"][:20])
    add_pairs_section(lines, "Interesting Tags", summary["interesting_tags"][:60])
    lines.append("## Per-File Snapshot")
    lines.append("")
    lines.append("| ID | Copy | XML MB | Tags | SampleRef | Active FileRef | Non-Sample FileRef | OriginalFileRef | RPT | Path classes |")
    lines.append("|---:|---|---:|---:|---:|---:|---:|---:|---|---|")
    for idx, report in enumerate(reports, start=1):
        if "error" in report:
            lines.append(f"| {idx} | `{Path(report['copy_path']).name}` | - | - | - | - | - | - | ERROR | {report['error']} |")
            continue
        rpt = ", ".join(f"{key}:{count}" for key, count in report["active_relative_path_type_counts"])
        classes = ", ".join(f"{key}:{count}" for key, count in report["active_path_classes"])
        lines.append(
            f"| {idx} | `{Path(report['copy_path']).name}` | {report['xml_bytes'] / 1024 / 1024:.2f} | "
            f"{report['unique_tag_count']} | {report['sample_ref_count']} | "
            f"{report['active_sample_file_ref_count']} | {report['non_sample_file_ref_count']} | "
            f"{report['original_file_ref_count']} | {rpt} | {classes} |"
        )
    lines.append("")
    lines.append("## Early Observations")
    lines.append("")
    lines.append("- `FileRef` is not only an active sample dependency; there are non-sample contexts that must be separated.")
    lines.append("- `OriginalFileRef` appears as historical/provenance material and needs separate handling.")
    lines.append("- RelativePathType appears as a useful observed signal, but this report does not treat it as official Ableton semantics.")
    lines.append("- The corpus contains many plugin/device/clip/track tags, so a future raw model may need context preservation beyond paths.")
    lines.append("- This experiment is evidence for contract review, not final product behavior.")
    lines.append("")
    lines.append("## Output Files")
    lines.append("")
    lines.append("- `selected_als_manifest.json`")
    lines.append("- `reports/corpus_summary.json`")
    lines.append("- `reports/per_file/*.json`")
    lines.append("- `reports/file_ref_contexts.csv`")
    lines.append("- `reports/interesting_tags.csv`")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def add_pairs_section(lines: list[str], title: str, pairs: list[list[Any]] | list[tuple[Any, Any]]) -> None:
    lines.append(f"## {title}")
    lines.append("")
    if not pairs:
        lines.append("_None_")
        lines.append("")
        return
    lines.append("| Value | Count |")
    lines.append("|---|---:|")
    for key, count in pairs:
        lines.append(f"| `{key}` | {count} |")
    lines.append("")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--experiment-dir", required=True, type=Path)
    parser.add_argument("--limit", default=20, type=int)
    parser.add_argument("--roots", nargs="+", required=True, type=Path)
    args = parser.parse_args()

    experiment_dir = args.experiment_dir
    copies_dir = experiment_dir / "copies"
    reports_dir = experiment_dir / "reports"
    per_file_dir = reports_dir / "per_file"

    candidates = find_candidates(args.roots)
    selected = select_corpus(candidates, args.limit)
    if len(selected) < args.limit:
        print(f"Only selected {len(selected)} files out of requested {args.limit}", file=sys.stderr)

    manifest = copy_selected(selected, copies_dir)
    write_json(experiment_dir / "selected_als_manifest.json", manifest)

    reports = []
    for item in manifest:
        report = analyze_copy(Path(item["copy_path"]), item)
        reports.append(report)
        write_json(per_file_dir / f"{item['id']:02d}__{Path(item['copy_path']).stem}.json", report)

    summary = aggregate_reports(reports)
    write_json(reports_dir / "corpus_summary.json", summary)

    write_csv(
        reports_dir / "file_ref_contexts.csv",
        [{"context": key, "count": count} for key, count in summary["file_ref_contexts"]],
        ["context", "count"],
    )
    write_csv(
        reports_dir / "interesting_tags.csv",
        [{"tag": key, "count": count} for key, count in summary["interesting_tags"]],
        ["tag", "count"],
    )
    write_markdown(experiment_dir / "structure_report.md", manifest, reports, summary)

    print(json.dumps({"experiment_dir": str(experiment_dir), "selected": len(manifest)}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
