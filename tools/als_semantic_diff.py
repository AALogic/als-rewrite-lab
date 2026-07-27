#!/usr/bin/env python3
"""Compare ALS XML semantics without emitting private paths."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path
from typing import Any, Iterable


ACTIVE_FIELDS = (
    "Path",
    "RelativePath",
    "RelativePathType",
    "Type",
    "OriginalFileSize",
    "OriginalCrc",
    "DefaultDuration",
    "DefaultSampleRate",
)


def read_root(path: Path) -> ET.Element:
    return ET.fromstring(gzip.decompress(path.read_bytes()))


def direct_child(node: ET.Element, tag: str) -> ET.Element | None:
    return next((child for child in node if child.tag == tag), None)


def child_values(node: ET.Element) -> dict[str, str | None]:
    return {
        tag: (
            direct_child(node, tag).attrib.get("Value")
            if direct_child(node, tag) is not None
            else None
        )
        for tag in ACTIVE_FIELDS
    }


def active_refs(root: ET.Element) -> list[dict[str, Any]]:
    refs = []
    for index, sample_ref in enumerate(root.iter("SampleRef")):
        file_ref = direct_child(sample_ref, "FileRef")
        if file_ref is None:
            continue
        refs.append(
            {
                "locator": f"SampleRef[{index}]/FileRef",
                "fields": child_values(file_ref),
            }
        )
    return refs


def historical_refs(root: ET.Element) -> list[dict[str, str | None]]:
    refs = []
    for original in root.iter("OriginalFileRef"):
        file_ref = direct_child(original, "FileRef")
        if file_ref is not None:
            refs.append(child_values(file_ref))
    return refs


def canonical_node(
    node: ET.Element,
    parent_tag: str | None,
    parent_is_direct_active_file_ref: bool,
    allowed_active_fields: set[str],
) -> Any:
    attributes = dict(sorted(node.attrib.items()))
    if (
        parent_is_direct_active_file_ref
        and node.tag in allowed_active_fields
        and node.attrib.get("Value") is not None
    ):
        attributes["Value"] = "<allowed-active-value>"
    children = []
    is_direct_active_file_ref = node.tag == "FileRef" and parent_tag == "SampleRef"
    for child in node:
        children.append(
            canonical_node(
                child,
                node.tag,
                is_direct_active_file_ref,
                allowed_active_fields,
            )
        )
    return [
        node.tag,
        attributes,
        (node.text or "").strip(),
        children,
    ]


def semantic_hash(root: ET.Element, allowed_active_fields: Iterable[str]) -> str:
    canonical = canonical_node(root, None, False, set(allowed_active_fields))
    encoded = json.dumps(
        canonical,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def changed_field_counts(
    before: list[dict[str, Any]],
    after: list[dict[str, Any]],
) -> Counter[str]:
    changes: Counter[str] = Counter()
    for left, right in zip(before, after):
        for field in ACTIVE_FIELDS:
            if left["fields"][field] != right["fields"][field]:
                changes[field] += 1
    return changes


def changed_historical_counts(
    before: list[dict[str, str | None]],
    after: list[dict[str, str | None]],
) -> Counter[str]:
    changes: Counter[str] = Counter()
    for left, right in zip(before, after):
        for field in ACTIVE_FIELDS:
            if left[field] != right[field]:
                changes[field] += 1
    return changes


def compare_documents(
    before_path: Path,
    after_path: Path,
    allowed_active_fields: Iterable[str],
) -> dict[str, Any]:
    allowed = tuple(allowed_active_fields)
    before_root = read_root(before_path)
    after_root = read_root(after_path)
    before_active = active_refs(before_root)
    after_active = active_refs(after_root)
    before_historical = historical_refs(before_root)
    after_historical = historical_refs(after_root)
    before_hash = semantic_hash(before_root, allowed)
    after_hash = semantic_hash(after_root, allowed)
    locator_sequences_equal = [item["locator"] for item in before_active] == [
        item["locator"] for item in after_active
    ]
    return {
        "schema_version": "0.1",
        "privacy": "No raw path values or filenames are emitted.",
        "allowed_active_fields": list(allowed),
        "before_document_id": hashlib.sha256(before_path.read_bytes()).hexdigest()[:12],
        "after_document_id": hashlib.sha256(after_path.read_bytes()).hexdigest()[:12],
        "before_creator": before_root.attrib.get("Creator"),
        "after_creator": after_root.attrib.get("Creator"),
        "before_minor_version": before_root.attrib.get("MinorVersion"),
        "after_minor_version": after_root.attrib.get("MinorVersion"),
        "active_ref_count_before": len(before_active),
        "active_ref_count_after": len(after_active),
        "active_locator_sequences_equal": locator_sequences_equal,
        "active_changed_fields": dict(
            sorted(changed_field_counts(before_active, after_active).items())
        ),
        "historical_ref_count_before": len(before_historical),
        "historical_ref_count_after": len(after_historical),
        "historical_changed_fields": dict(
            sorted(
                changed_historical_counts(
                    before_historical,
                    after_historical,
                ).items()
            )
        ),
        "semantic_hash_before": before_hash,
        "semantic_hash_after": after_hash,
        "equal_after_allowed_active_redaction": before_hash == after_hash,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--allow-active-field", action="append", default=[])
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    report = compare_documents(
        args.before,
        args.after,
        args.allow_active_field,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
