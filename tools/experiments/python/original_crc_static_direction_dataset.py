#!/usr/bin/env python3
"""Build a static sample dataset for Ableton OriginalCrc research.

This is an experiment helper, not product code.

Safety posture:
- Read copied ALS files from the existing corpus.
- Read/copy referenced audio files only when they already exist.
- Never modify original ALS files or original sample files.
- Write all experiment artifacts into a new experiment directory.
"""

from __future__ import annotations

import argparse
import collections
import gzip
import hashlib
import json
import shutil
import sys
import urllib.parse
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
CORPUS_DIR = ROOT / "experiments/2026-06-02_als_structure_corpus_20"
DEFAULT_OUTPUT_DIR = CORPUS_DIR / "original_crc_static_direction_dataset"
AUDIO_EXTENSIONS = {".wav", ".wave", ".aif", ".aiff", ".mp3", ".flac", ".m4a", ".ogg", ".aac"}


def tag_name(raw: str) -> str:
    if "}" in raw:
        return raw.rsplit("}", 1)[1]
    return raw


def value_of(element: ET.Element | None) -> str | None:
    if element is None:
        return None
    if "Value" in element.attrib:
        return element.attrib["Value"]
    if element.text is not None and element.text.strip():
        return element.text.strip()
    return None


def direct_child(element: ET.Element, wanted: str) -> ET.Element | None:
    for child in list(element):
        if tag_name(child.tag) == wanted:
            return child
    return None


def child_value(element: ET.Element, wanted: str) -> str | None:
    return value_of(direct_child(element, wanted))


def parse_int(value: str | None) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def safe_name(value: str, fallback: str = "sample") -> str:
    keep = []
    for char in value:
        if char.isalnum() or char in " ._-[()]":
            keep.append(char)
        else:
            keep.append("_")
    cleaned = "".join(keep).strip()
    while "  " in cleaned:
        cleaned = cleaned.replace("  ", " ")
    return (cleaned or fallback)[:120]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def file_extension(*values: str | None) -> str:
    for value in values:
        if value:
            suffix = Path(value).suffix.lower()
            if suffix:
                return suffix
    return ""


def unique_path_candidates(raw_value: str | None, project_dir: Path) -> list[Path]:
    if not raw_value:
        return []
    decoded = urllib.parse.unquote(raw_value)
    candidates: list[Path] = []
    raw_path = Path(decoded)
    if raw_path.is_absolute():
        candidates.append(raw_path)
    else:
        candidates.append(project_dir / raw_path)
    return candidates


def resolve_source_path(path_value: str | None, relative_path: str | None, project_dir: Path) -> tuple[Path | None, list[str]]:
    candidates: list[Path] = []
    for raw in [path_value, relative_path]:
        for candidate in unique_path_candidates(raw, project_dir):
            if candidate not in candidates:
                candidates.append(candidate)

    tried = [str(candidate) for candidate in candidates]
    for candidate in candidates:
        if candidate.exists() and candidate.is_file():
            return candidate, tried
    return None, tried


@dataclass
class RefRecord:
    als_id: int
    als_copy_path: Path
    als_source_path: Path
    ref_index: int
    path_value: str | None
    relative_path: str | None
    relative_path_type: str | None
    type_value: str | None
    original_file_size: int | None
    original_crc: int | None
    default_duration: int | None
    default_sample_rate: int | None
    extension: str
    resolved_source_path: Path | None
    resolution_candidates: list[str]
    source_exists: bool
    source_size: int | None
    sidecar_asd_path: Path | None


@dataclass
class SampleCandidate:
    source_path: Path
    refs: list[RefRecord] = field(default_factory=list)
    categories: set[str] = field(default_factory=set)
    reasons: list[str] = field(default_factory=list)

    @property
    def source_size(self) -> int:
        for ref in self.refs:
            if ref.source_size is not None:
                return ref.source_size
        return self.source_path.stat().st_size

    @property
    def extension(self) -> str:
        return self.source_path.suffix.lower()

    @property
    def has_asd(self) -> bool:
        return Path(str(self.source_path) + ".asd").exists()

    @property
    def crc_values(self) -> set[int]:
        return {ref.original_crc for ref in self.refs if ref.original_crc is not None}

    @property
    def original_size_values(self) -> set[int]:
        return {ref.original_file_size for ref in self.refs if ref.original_file_size is not None}

    @property
    def sample_rate_values(self) -> set[int]:
        return {ref.default_sample_rate for ref in self.refs if ref.default_sample_rate is not None}

    @property
    def duration_values(self) -> set[int]:
        return {ref.default_duration for ref in self.refs if ref.default_duration is not None}

    @property
    def rpt_values(self) -> set[str]:
        return {ref.relative_path_type for ref in self.refs if ref.relative_path_type is not None}


def load_manifest() -> list[dict[str, Any]]:
    manifest_path = CORPUS_DIR / "selected_als_manifest.json"
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def extract_refs(manifest: list[dict[str, Any]]) -> list[RefRecord]:
    records: list[RefRecord] = []
    for item in manifest:
        als_id = int(item["id"])
        als_copy_path = ROOT / item["copy_path"]
        als_source_path = Path(item["source_path"])
        project_dir = als_source_path.parent

        xml_bytes = gzip.decompress(als_copy_path.read_bytes())
        root = ET.fromstring(xml_bytes)

        parent_by_id: dict[int, ET.Element] = {}
        for parent in root.iter():
            for child in list(parent):
                parent_by_id[id(child)] = parent

        ref_index = 0
        for node in root.iter():
            if tag_name(node.tag) != "FileRef":
                continue
            parent = parent_by_id.get(id(node))
            if parent is None or tag_name(parent.tag) != "SampleRef":
                continue

            path_value = child_value(node, "Path")
            relative_path = child_value(node, "RelativePath")
            extension = file_extension(path_value, relative_path)
            resolved_source_path, resolution_candidates = resolve_source_path(path_value, relative_path, project_dir)
            source_exists = resolved_source_path is not None
            source_size = resolved_source_path.stat().st_size if resolved_source_path else None
            sidecar_asd_path = None
            if resolved_source_path:
                candidate_asd = Path(str(resolved_source_path) + ".asd")
                if candidate_asd.exists() and candidate_asd.is_file():
                    sidecar_asd_path = candidate_asd

            records.append(
                RefRecord(
                    als_id=als_id,
                    als_copy_path=als_copy_path,
                    als_source_path=als_source_path,
                    ref_index=ref_index,
                    path_value=path_value,
                    relative_path=relative_path,
                    relative_path_type=child_value(node, "RelativePathType"),
                    type_value=child_value(node, "Type"),
                    original_file_size=parse_int(child_value(node, "OriginalFileSize")),
                    original_crc=parse_int(child_value(node, "OriginalCrc")),
                    default_duration=parse_int(child_value(node, "DefaultDuration")),
                    default_sample_rate=parse_int(child_value(node, "DefaultSampleRate")),
                    extension=extension,
                    resolved_source_path=resolved_source_path,
                    resolution_candidates=resolution_candidates,
                    source_exists=source_exists,
                    source_size=source_size,
                    sidecar_asd_path=sidecar_asd_path,
                )
            )
            ref_index += 1
    return records


def build_candidates(refs: list[RefRecord]) -> dict[str, SampleCandidate]:
    by_source: dict[str, SampleCandidate] = {}
    for ref in refs:
        if not ref.source_exists or ref.resolved_source_path is None:
            continue
        if ref.extension not in AUDIO_EXTENSIONS:
            continue
        key = str(ref.resolved_source_path)
        by_source.setdefault(key, SampleCandidate(source_path=ref.resolved_source_path)).refs.append(ref)
    return by_source


def add_candidate(
    selected: dict[str, SampleCandidate],
    candidate: SampleCandidate,
    category: str,
    reason: str,
    max_samples: int,
    max_total_bytes: int,
) -> None:
    key = str(candidate.source_path)
    if key not in selected:
        current_total = sum(item.source_size for item in selected.values())
        if len(selected) >= max_samples:
            return
        if current_total + candidate.source_size > max_total_bytes:
            return
        selected[key] = candidate
    selected[key].categories.add(category)
    if reason not in selected[key].reasons:
        selected[key].reasons.append(reason)


def sorted_small(candidates: list[SampleCandidate]) -> list[SampleCandidate]:
    return sorted(candidates, key=lambda item: (item.source_size, str(item.source_path)))


def choose_dataset(candidates: dict[str, SampleCandidate], max_samples: int, max_total_bytes: int) -> dict[str, SampleCandidate]:
    selected: dict[str, SampleCandidate] = {}
    candidate_list = list(candidates.values())

    # Category 1: format diversity for future payload/PCM tests.
    for ext in [".wav", ".aif", ".aiff", ".mp3"]:
        ext_candidates = [item for item in candidate_list if item.extension == ext]
        ext_candidates.sort(key=lambda item: (not item.has_asd, item.source_size, str(item.source_path)))
        for item in ext_candidates[:4]:
            add_candidate(
                selected,
                item,
                "format_diversity",
                f"Representative {ext} sample for payload/PCM comparison.",
                max_samples,
                max_total_bytes,
            )

    # Category 2: RelativePathType diversity.
    for rpt in ["0", "1", "3", "5"]:
        rpt_candidates = [item for item in candidate_list if rpt in item.rpt_values]
        rpt_candidates.sort(key=lambda item: (not item.has_asd, item.source_size, str(item.source_path)))
        for item in rpt_candidates[:4]:
            add_candidate(
                selected,
                item,
                "relative_path_type_diversity",
                f"Representative sample for RelativePathType {rpt}.",
                max_samples,
                max_total_bytes,
            )

    # Category 3: .asd sidecar presence.
    asd_candidates = [item for item in candidate_list if item.has_asd]
    for item in sorted_small(asd_candidates)[:12]:
        add_candidate(
            selected,
            item,
            "asd_sidecar_available",
            "Sidecar .asd exists for later Ableton analysis correlation tests.",
            max_samples,
            max_total_bytes,
        )

    # Category 4: small files useful for expensive decoded-PCM experiments.
    small_candidates = [item for item in candidate_list if item.source_size <= 8 * 1024 * 1024]
    for item in sorted_small(small_candidates)[:16]:
        add_candidate(
            selected,
            item,
            "small_pcm_probe_target",
            "Small file is practical for expensive decoded-PCM/fingerprint probes.",
            max_samples,
            max_total_bytes,
        )

    # Category 5: same physical file appears many times in ALS references.
    repeated = [item for item in candidate_list if len(item.refs) >= 2]
    for item in sorted(repeated, key=lambda item: (-len(item.refs), item.source_size, str(item.source_path)))[:8]:
        add_candidate(
            selected,
            item,
            "same_source_path_repeated",
            f"Same source path is referenced {len(item.refs)} times.",
            max_samples,
            max_total_bytes,
        )

    # Category 6: same source file has more than one ALS OriginalCrc value.
    for item in sorted_small([item for item in candidate_list if len(item.crc_values) > 1])[:8]:
        add_candidate(
            selected,
            item,
            "same_source_path_different_original_crc",
            f"Same source path has ALS OriginalCrc values {sorted(item.crc_values)}.",
            max_samples,
            max_total_bytes,
        )

    # Category 7: multiple different files share OriginalCrc and OriginalFileSize.
    by_crc_size: dict[tuple[int, int], list[SampleCandidate]] = collections.defaultdict(list)
    for item in candidate_list:
        for crc in item.crc_values:
            for size in item.original_size_values:
                by_crc_size[(crc, size)].append(item)
    groups = [group for group in by_crc_size.values() if len({str(item.source_path) for item in group}) >= 2]
    for group in sorted(groups, key=lambda group: (-len(group), sum(item.source_size for item in group) / len(group)))[:5]:
        key_item = group[0]
        reason = f"Group of {len(group)} files shares OriginalCrc/OriginalFileSize."
        for item in sorted_small(group)[:5]:
            add_candidate(selected, item, "same_original_crc_and_size", reason, max_samples, max_total_bytes)

    # Category 8: same OriginalCrc appears across different ALS sizes.
    by_crc: dict[int, list[SampleCandidate]] = collections.defaultdict(list)
    for item in candidate_list:
        for crc in item.crc_values:
            by_crc[crc].append(item)
    crc_groups = [
        group
        for group in by_crc.values()
        if len({str(item.source_path) for item in group}) >= 2
        and len({size for item in group for size in item.original_size_values}) >= 2
    ]
    for group in sorted(crc_groups, key=lambda group: (-len(group), sum(item.source_size for item in group) / len(group)))[:4]:
        reason = f"Group of {len(group)} files shares OriginalCrc across different ALS OriginalFileSize values."
        for item in sorted_small(group)[:4]:
            add_candidate(selected, item, "same_original_crc_different_size", reason, max_samples, max_total_bytes)

    # Category 9: same ALS OriginalFileSize with different OriginalCrc values.
    by_original_size: dict[int, list[SampleCandidate]] = collections.defaultdict(list)
    for item in candidate_list:
        for size in item.original_size_values:
            by_original_size[size].append(item)
    size_groups = [
        group
        for group in by_original_size.values()
        if len({str(item.source_path) for item in group}) >= 2
        and len({crc for item in group for crc in item.crc_values}) >= 2
    ]
    for group in sorted(size_groups, key=lambda group: (-len(group), sum(item.source_size for item in group) / len(group)))[:4]:
        reason = f"Group of {len(group)} files shares ALS OriginalFileSize but has different OriginalCrc values."
        for item in sorted_small(group)[:4]:
            add_candidate(selected, item, "same_original_size_different_crc", reason, max_samples, max_total_bytes)

    return selected


def copy_dataset(selected: dict[str, SampleCandidate], output_dir: Path, overwrite: bool) -> dict[str, Any]:
    if output_dir.exists():
        if not overwrite:
            raise SystemExit(f"Output directory already exists: {output_dir}")
        shutil.rmtree(output_dir)

    samples_dir = output_dir / "samples"
    als_dir = output_dir / "als_copies"
    samples_dir.mkdir(parents=True)
    als_dir.mkdir(parents=True)

    selected_als_paths: dict[str, Path] = {}
    for candidate in selected.values():
        for ref in candidate.refs:
            selected_als_paths[str(ref.als_copy_path)] = ref.als_copy_path

    copied_als = []
    for als_path in sorted(selected_als_paths.values(), key=lambda path: path.name):
        target = als_dir / als_path.name
        shutil.copy2(als_path, target)
        copied_als.append(
            {
                "source_copy_path": str(als_path.relative_to(ROOT)),
                "copied_path": str(target.relative_to(ROOT)),
                "sha256": sha256_file(target),
                "size_bytes": target.stat().st_size,
            }
        )

    copied_samples = []
    original_stat_checks = []
    for index, candidate in enumerate(sorted(selected.values(), key=lambda item: (min(ref.als_id for ref in item.refs), item.source_size, str(item.source_path))), start=1):
        source = candidate.source_path
        before_stat = source.stat()
        target_name = f"{index:04d}__{safe_name(source.name)}"
        target = samples_dir / target_name
        shutil.copy2(source, target)
        after_stat = source.stat()
        source_unchanged = (
            before_stat.st_size == after_stat.st_size
            and before_stat.st_mtime_ns == after_stat.st_mtime_ns
            and before_stat.st_ino == after_stat.st_ino
        )
        original_stat_checks.append(source_unchanged)

        asd_source = Path(str(source) + ".asd")
        copied_asd = None
        copied_asd_sha256 = None
        if asd_source.exists() and asd_source.is_file():
            copied_asd = Path(str(target) + ".asd")
            shutil.copy2(asd_source, copied_asd)
            copied_asd_sha256 = sha256_file(copied_asd)

        copied_sha256 = sha256_file(target)
        refs_payload = []
        for ref in sorted(candidate.refs, key=lambda ref: (ref.als_id, ref.ref_index))[:40]:
            refs_payload.append(
                {
                    "als_id": ref.als_id,
                    "als_copy": str(ref.als_copy_path.relative_to(ROOT)),
                    "als_source": str(ref.als_source_path),
                    "ref_index": ref.ref_index,
                    "path": ref.path_value,
                    "relative_path": ref.relative_path,
                    "relative_path_type": ref.relative_path_type,
                    "type": ref.type_value,
                    "original_file_size": ref.original_file_size,
                    "original_crc": ref.original_crc,
                    "default_duration": ref.default_duration,
                    "default_sample_rate": ref.default_sample_rate,
                }
            )

        copied_samples.append(
            {
                "selection_id": index,
                "categories": sorted(candidate.categories),
                "reasons": candidate.reasons,
                "source_sample_path": str(source),
                "copied_sample_path": str(target.relative_to(ROOT)),
                "source_size_bytes": before_stat.st_size,
                "copied_size_bytes": target.stat().st_size,
                "source_unchanged_after_copy": source_unchanged,
                "sha256": copied_sha256,
                "extension": candidate.extension,
                "source_ref_count": len(candidate.refs),
                "als_ids": sorted({ref.als_id for ref in candidate.refs}),
                "original_crc_values": sorted(candidate.crc_values),
                "original_file_size_values": sorted(candidate.original_size_values),
                "default_duration_values": sorted(candidate.duration_values),
                "default_sample_rate_values": sorted(candidate.sample_rate_values),
                "relative_path_type_values": sorted(candidate.rpt_values),
                "asd_sidecar": {
                    "exists": asd_source.exists(),
                    "source_path": str(asd_source) if asd_source.exists() else None,
                    "copied_path": str(copied_asd.relative_to(ROOT)) if copied_asd else None,
                    "sha256": copied_asd_sha256,
                },
                "refs": refs_payload,
                "refs_truncated": len(candidate.refs) > len(refs_payload),
            }
        )

    return {
        "status": "created",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "output_dir": str(output_dir.relative_to(ROOT)),
        "source_corpus": str(CORPUS_DIR.relative_to(ROOT)),
        "safety": {
            "read_copied_als_only": True,
            "modified_original_als": False,
            "modified_original_samples": False,
            "all_source_stat_checks_passed": all(original_stat_checks),
        },
        "copied_als_count": len(copied_als),
        "copied_sample_count": len(copied_samples),
        "total_copied_sample_bytes": sum(item["copied_size_bytes"] for item in copied_samples),
        "category_counts": collections.Counter(
            category for item in copied_samples for category in item["categories"]
        ),
        "copied_als": copied_als,
        "copied_samples": copied_samples,
    }


def inventory_summary(refs: list[RefRecord], candidates: dict[str, SampleCandidate]) -> dict[str, Any]:
    accessible_refs = [ref for ref in refs if ref.source_exists and ref.resolved_source_path is not None]
    return {
        "active_ref_count": len(refs),
        "accessible_ref_count": len(accessible_refs),
        "unique_accessible_sample_count": len(candidates),
        "accessible_ref_extensions": collections.Counter(ref.extension for ref in accessible_refs).most_common(),
        "accessible_ref_relative_path_types": collections.Counter(ref.relative_path_type or "<missing>" for ref in accessible_refs).most_common(),
        "refs_with_asd_sidecar": sum(1 for ref in accessible_refs if ref.sidecar_asd_path is not None),
        "unique_samples_with_asd_sidecar": sum(1 for item in candidates.values() if item.has_asd),
        "unique_samples_referenced_more_than_once": sum(1 for item in candidates.values() if len(item.refs) > 1),
        "unique_samples_with_multiple_original_crc_values": sum(1 for item in candidates.values() if len(item.crc_values) > 1),
    }


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2, default=dict), encoding="utf-8")


def write_readme(output_dir: Path, manifest: dict[str, Any], inventory: dict[str, Any]) -> None:
    category_counts = dict(manifest["category_counts"])
    lines = [
        "# OriginalCrc Static Direction Dataset",
        "",
        "Status: created",
        f"Date: {datetime.now().date().isoformat()}",
        "",
        "## Purpose",
        "",
        "Select the best available static test data from the 20 copied ALS corpus",
        "for the next phase of Ableton `OriginalCrc` research.",
        "",
        "This dataset is meant to guide which hypothesis is worth testing next:",
        "",
        "```text",
        "same file across references",
        "same OriginalCrc across different files",
        "same OriginalFileSize with different OriginalCrc",
        "format and RelativePathType diversity",
        ".asd sidecar correlation",
        "small decoded-PCM probe targets",
        "```",
        "",
        "## Safety",
        "",
        "```text",
        "Original ALS files were not read directly; existing ALS copies were used.",
        "Original sample files were copied only.",
        "Original sample stat checks after copy passed: "
        + str(manifest["safety"]["all_source_stat_checks_passed"]),
        "```",
        "",
        "## Inventory Before Selection",
        "",
        "```json",
        json.dumps(inventory, ensure_ascii=False, indent=2),
        "```",
        "",
        "## Dataset Summary",
        "",
        "```text",
        f"copied ALS files: {manifest['copied_als_count']}",
        f"copied sample files: {manifest['copied_sample_count']}",
        f"total copied sample bytes: {manifest['total_copied_sample_bytes']}",
        "```",
        "",
        "## Category Counts",
        "",
        "```json",
        json.dumps(category_counts, ensure_ascii=False, indent=2),
        "```",
        "",
        "## Files",
        "",
        "```text",
        "manifest.json",
        "candidate_inventory.json",
        "als_copies/",
        "samples/",
        "```",
        "",
        "## Current Interpretation",
        "",
        "This dataset does not prove the `OriginalCrc` algorithm.",
        "It gives us stronger static material for the next probes, especially",
        "PCM/audio-fingerprint tests and `.asd` correlation tests.",
        "",
    ]
    (output_dir / "README.md").write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--max-samples", type=int, default=60)
    parser.add_argument("--max-total-bytes", type=int, default=700 * 1024 * 1024)
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()

    manifest = load_manifest()
    refs = extract_refs(manifest)
    candidates = build_candidates(refs)
    selected = choose_dataset(candidates, args.max_samples, args.max_total_bytes)

    output_dir = args.output_dir if args.output_dir.is_absolute() else ROOT / args.output_dir
    dataset_manifest = copy_dataset(selected, output_dir, args.overwrite)
    inventory = inventory_summary(refs, candidates)

    # Convert Counter to normal dict for stable JSON.
    dataset_manifest["category_counts"] = dict(dataset_manifest["category_counts"])
    write_json(output_dir / "manifest.json", dataset_manifest)
    write_json(output_dir / "candidate_inventory.json", inventory)
    write_readme(output_dir, dataset_manifest, inventory)

    print(json.dumps(
        {
            "output_dir": str(output_dir),
            "copied_als_count": dataset_manifest["copied_als_count"],
            "copied_sample_count": dataset_manifest["copied_sample_count"],
            "total_copied_sample_bytes": dataset_manifest["total_copied_sample_bytes"],
            "all_source_stat_checks_passed": dataset_manifest["safety"]["all_source_stat_checks_passed"],
            "category_counts": dataset_manifest["category_counts"],
        },
        ensure_ascii=False,
        indent=2,
    ))
    return 0


if __name__ == "__main__":
    sys.exit(main())
