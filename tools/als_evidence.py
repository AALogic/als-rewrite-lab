#!/usr/bin/env python3
"""Build a privacy-safe structural evidence report from ALS files."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import re
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path
from typing import Any


def path_shape(value: str) -> str:
    if not value:
        return "empty"
    if re.match(r"^[A-Za-z]:[\\/]", value):
        return "windows_drive_absolute"
    if value.startswith("\\\\"):
        return "windows_unc"
    if value.startswith("/"):
        return "posix_absolute"
    if value.startswith("../") or value.startswith("..\\"):
        return "parent_relative"
    if "/" in value:
        return "relative_forward_slash"
    if "\\" in value:
        return "relative_backslash"
    return "bare_filename"


def file_ref_class(ancestors: list[str]) -> str:
    if ancestors and ancestors[-1] == "SampleRef":
        return "active_direct_sample_ref"
    if "OriginalFileRef" in ancestors:
        return "historical_original_file_ref"
    return "other_file_ref"


def inspect_document(path: Path) -> dict[str, Any]:
    compressed = path.read_bytes()
    digest = hashlib.sha256(compressed).hexdigest()
    root = ET.fromstring(gzip.decompress(compressed))
    classes: Counter[str] = Counter()
    path_types: Counter[str] = Counter()
    shapes: Counter[str] = Counter()
    contexts: Counter[str] = Counter()
    active_paths: list[tuple[str, str, str]] = []

    def walk(node: ET.Element, ancestors: list[str]) -> None:
        current = [*ancestors, node.tag]
        if node.tag == "FileRef":
            values = {child.tag: child.attrib.get("Value", "") for child in node}
            classification = file_ref_class(ancestors)
            classes[classification] += 1
            context = "/".join([*ancestors[-5:], "FileRef"])
            contexts[f"{classification}:{context}"] += 1
            if classification == "active_direct_sample_ref":
                path_type = values.get("RelativePathType", "<missing>")
                raw_path = values.get("Path", "")
                raw_relative = values.get("RelativePath", "")
                path_types[path_type] += 1
                shapes[
                    f"type={path_type};path={path_shape(raw_path)};"
                    f"relative={path_shape(raw_relative)}"
                ] += 1
                active_paths.append((raw_path, raw_relative, path_type))
        for child in node:
            walk(child, current)

    walk(root, [])
    return {
        "document_id": digest[:12],
        "sha256": digest,
        "compressed_bytes": len(compressed),
        "ableton_major_version": root.attrib.get("MajorVersion"),
        "ableton_creator": root.attrib.get("Creator"),
        "file_ref_classes": dict(sorted(classes.items())),
        "active_relative_path_types": dict(sorted(path_types.items())),
        "active_path_shapes": dict(sorted(shapes.items())),
        "active_occurrence_count": len(active_paths),
        "active_unique_raw_path_count": len({item[0] for item in active_paths}),
        "xml_context_counts": dict(sorted(contexts.items())),
    }


def build_report(input_dir: Path) -> dict[str, Any]:
    documents = []
    aggregate_classes: Counter[str] = Counter()
    aggregate_types: Counter[str] = Counter()
    aggregate_shapes: Counter[str] = Counter()
    aggregate_contexts: Counter[str] = Counter()

    for path in sorted(input_dir.rglob("*.als")):
        document = inspect_document(path)
        documents.append(document)
        aggregate_classes.update(document["file_ref_classes"])
        aggregate_types.update(document["active_relative_path_types"])
        aggregate_shapes.update(document["active_path_shapes"])
        aggregate_contexts.update(document["xml_context_counts"])

    return {
        "schema_version": "0.1",
        "privacy": "No filenames or raw path values are emitted.",
        "document_count": len(documents),
        "aggregate": {
            "file_ref_classes": dict(sorted(aggregate_classes.items())),
            "active_relative_path_types": dict(sorted(aggregate_types.items())),
            "active_path_shapes": dict(sorted(aggregate_shapes.items())),
            "xml_context_counts": dict(sorted(aggregate_contexts.items())),
        },
        "documents": documents,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input_dir", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    report = build_report(args.input_dir)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
