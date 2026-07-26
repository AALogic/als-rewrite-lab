#!/usr/bin/env python3
"""Run static OriginalCrc direction tests on the balanced copied dataset.

This is an experiment helper, not product code.

It reads only copied experiment files and writes derived reports into a new
experiment output folder. It does not modify original Ableton projects or
original sample files.
"""

from __future__ import annotations

import argparse
import collections
import csv
import hashlib
import json
import math
import os
import re
import shutil
import struct
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
DATASET_DIR = ROOT / "experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset_balanced"
DEFAULT_OUTPUT_DIR = DATASET_DIR / "static_analysis_probe"
C_HELPER_SOURCE = ROOT / "tools/experiments/c/original_crc_fast_probe.c"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def byteswap16(value: int) -> int:
    return ((value & 0xFF) << 8) | ((value >> 8) & 0xFF)


def low16(value: int | None) -> int | None:
    return None if value is None else value & 0xFFFF


def high16(value: int | None) -> int | None:
    return None if value is None else (value >> 16) & 0xFFFF


def parse_int(value: str | int | float | None) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def parse_float(value: str | int | float | None) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def read_wav_data_chunk(path: Path) -> bytes | None:
    data = path.read_bytes()
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        return None
    pos = 12
    while pos + 8 <= len(data):
        chunk_id = data[pos : pos + 4]
        size = struct.unpack_from("<I", data, pos + 4)[0]
        start = pos + 8
        end = min(start + size, len(data))
        if chunk_id == b"data":
            return data[start:end]
        pos = end + (size % 2)
    return None


def run_command(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, text=True, capture_output=True, check=False)


def compile_helper(output_dir: Path) -> Path:
    bin_dir = output_dir / "_bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    binary = bin_dir / "original_crc_fast_probe"
    result = run_command(["cc", "-O2", str(C_HELPER_SOURCE), "-o", str(binary)])
    if result.returncode != 0:
        raise RuntimeError(f"Failed to compile C helper:\n{result.stderr}")
    return binary


def run_crc_helper(helper: Path, expected_crc: int, file_path: Path, context_prefix: str) -> list[dict[str, Any]]:
    result = run_command([str(helper), str(expected_crc), str(file_path)])
    rows: list[dict[str, Any]] = []
    for line in result.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) != 5:
            continue
        path, context, function, value, variant = parts
        rows.append(
            {
                "tested_file": path,
                "context": f"{context_prefix}:{context}",
                "function": function,
                "value": parse_int(value),
                "variant": variant,
                "expected_original_crc": expected_crc,
            }
        )
    return rows


def find_encoded_value(path: Path, value: int) -> list[dict[str, Any]]:
    data = path.read_bytes()
    needles = {
        "uint16_le": value.to_bytes(2, "little", signed=False),
        "uint16_be": value.to_bytes(2, "big", signed=False),
        "ascii_decimal": str(value).encode("ascii"),
    }
    rows = []
    for encoding, needle in needles.items():
        offsets = []
        start = 0
        while True:
            found = data.find(needle, start)
            if found < 0:
                break
            offsets.append(found)
            start = found + 1
            if len(offsets) >= 20:
                break
        if offsets:
            rows.append(
                {
                    "encoding": encoding,
                    "needle_hex": needle.hex(),
                    "match_count_capped": len(offsets),
                    "first_offsets": ",".join(str(offset) for offset in offsets[:20]),
                }
            )
    return rows


def afinfo(path: Path) -> dict[str, Any]:
    result = run_command(["afinfo", str(path)])
    metadata: dict[str, Any] = {
        "afinfo_ok": result.returncode == 0,
        "afinfo_error": result.stderr.strip() if result.returncode != 0 else None,
    }
    text = result.stdout
    metadata["raw_summary"] = "\n".join(text.splitlines()[:12])

    file_type = re.search(r"File type ID:\s+(.+)", text)
    if file_type:
        metadata["file_type_id"] = file_type.group(1).strip()

    data_format = re.search(r"Data format:\s+(\d+)\s+ch,\s+([0-9.]+)\s+Hz,\s+([^\s]+)(.*)", text)
    if data_format:
        metadata["channels"] = int(data_format.group(1))
        metadata["sample_rate_hz"] = int(float(data_format.group(2)))
        metadata["codec"] = data_format.group(3)
        tail = data_format.group(4)
        bit_depth = re.search(r"(\d+)-bit", tail)
        bits_channel = re.search(r"(\d+)\s+bits/channel", tail)
        if bit_depth:
            metadata["bit_depth"] = int(bit_depth.group(1))
        elif bits_channel:
            metadata["bit_depth"] = int(bits_channel.group(1))

    patterns = {
        "estimated_duration_sec": r"estimated duration:\s+([0-9.]+)\s+sec",
        "audio_bytes": r"audio bytes:\s+(\d+)",
        "audio_packets": r"audio packets:\s+(\d+)",
        "bit_rate": r"bit rate:\s+(\d+)",
        "packet_size_upper_bound": r"packet size upper bound:\s+(\d+)",
        "maximum_packet_size": r"maximum packet size:\s+(\d+)",
        "audio_data_file_offset": r"audio data file offset:\s+(\d+)",
    }
    for key, pattern in patterns.items():
        match = re.search(pattern, text)
        if not match:
            continue
        metadata[key] = parse_float(match.group(1)) if key.endswith("_sec") else int(match.group(1))

    source_bit_depth = re.search(r"source bit depth:\s+(.+)", text)
    if source_bit_depth:
        metadata["source_bit_depth"] = source_bit_depth.group(1).strip()

    if metadata.get("estimated_duration_sec") is not None and metadata.get("sample_rate_hz") is not None:
        metadata["estimated_frame_count"] = int(round(metadata["estimated_duration_sec"] * metadata["sample_rate_hz"]))
        metadata["estimated_duration_ms"] = int(round(metadata["estimated_duration_sec"] * 1000))

    return metadata


def decode_to_pcm_wav(source: Path, target: Path) -> tuple[bool, str | None]:
    result = run_command(["afconvert", "-f", "WAVE", "-d", "LEI16", str(source), str(target)])
    if result.returncode != 0:
        return False, (result.stderr or result.stdout).strip()
    return True, None


def unique_nonzero_crcs(sample: dict[str, Any]) -> list[int]:
    values = []
    for value in sample.get("original_crc_values", []):
        parsed = parse_int(value)
        if parsed is not None and parsed != 0 and parsed not in values:
            values.append(parsed)
    return sorted(values)


def all_crcs(sample: dict[str, Any]) -> list[int]:
    values = []
    for value in sample.get("original_crc_values", []):
        parsed = parse_int(value)
        if parsed is not None and parsed not in values:
            values.append(parsed)
    return sorted(values)


def field_candidates(sample: dict[str, Any], metadata: dict[str, Any]) -> list[tuple[str, int]]:
    values: dict[str, int] = {}
    values["source_size_bytes"] = int(sample["source_size_bytes"])
    values["copied_size_bytes"] = int(sample["copied_size_bytes"])
    for idx, value in enumerate(sample.get("original_file_size_values", [])):
        parsed = parse_int(value)
        if parsed is not None:
            values[f"als_original_file_size_{idx}"] = parsed
    for idx, value in enumerate(sample.get("default_duration_values", [])):
        parsed = parse_int(value)
        if parsed is not None:
            values[f"als_default_duration_{idx}"] = parsed
    for idx, value in enumerate(sample.get("default_sample_rate_values", [])):
        parsed = parse_int(value)
        if parsed is not None:
            values[f"als_default_sample_rate_{idx}"] = parsed

    for key in [
        "channels",
        "sample_rate_hz",
        "bit_depth",
        "estimated_duration_ms",
        "estimated_frame_count",
        "audio_bytes",
        "audio_packets",
        "bit_rate",
        "packet_size_upper_bound",
        "maximum_packet_size",
        "audio_data_file_offset",
    ]:
        parsed = parse_int(metadata.get(key))
        if parsed is not None:
            values[f"afinfo_{key}"] = parsed

    candidates: list[tuple[str, int]] = []
    for key, value in values.items():
        transforms = {
            "direct": value,
            "low16": value & 0xFFFF,
            "high16": (value >> 16) & 0xFFFF,
            "low16_byteswap": byteswap16(value & 0xFFFF),
            "high16_byteswap": byteswap16((value >> 16) & 0xFFFF),
        }
        for transform, candidate in transforms.items():
            candidates.append((f"{key}:{transform}", candidate))
    return candidates


def metadata_signatures(sample: dict[str, Any], metadata: dict[str, Any]) -> dict[str, str]:
    def join(values: list[Any]) -> str:
        return "|".join("<none>" if value is None else str(value) for value in values)

    return {
        "format_signature": join(
            [
                sample.get("extension"),
                metadata.get("file_type_id"),
                metadata.get("codec"),
                metadata.get("channels"),
                metadata.get("sample_rate_hz"),
                metadata.get("bit_depth"),
                metadata.get("source_bit_depth"),
            ]
        ),
        "duration_signature": join(
            [
                metadata.get("channels"),
                metadata.get("sample_rate_hz"),
                metadata.get("bit_depth"),
                metadata.get("audio_bytes"),
                metadata.get("audio_packets"),
                metadata.get("estimated_duration_ms"),
            ]
        ),
        "als_audio_signature": join(
            [
                sample.get("original_file_size_values"),
                sample.get("default_duration_values"),
                sample.get("default_sample_rate_values"),
            ]
        ),
    }


def write_csv(path: Path, rows: list[dict[str, Any]], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, extrasaction="ignore")
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


def crcs_from_row(row: dict[str, Any], field: str = "original_crc_values", nonzero: bool = False) -> list[int]:
    raw = row.get(field, [])
    if isinstance(raw, str):
        values = raw.split(",") if raw else []
    else:
        values = raw
    parsed_values: list[int] = []
    for value in values:
        parsed = parse_int(value)
        if parsed is None:
            continue
        if nonzero and parsed == 0:
            continue
        if parsed not in parsed_values:
            parsed_values.append(parsed)
    return sorted(parsed_values)


def grouped_rows_by_key(samples: list[dict[str, Any]], key_name: str) -> list[dict[str, Any]]:
    groups: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
    for sample in samples:
        groups[str(sample[key_name])].append(sample)

    rows = []
    for key, group in sorted(groups.items(), key=lambda item: (-len(item[1]), item[0])):
        crc_values = sorted({crc for sample in group for crc in crcs_from_row(sample)})
        nonzero_crc_values = sorted({crc for sample in group for crc in crcs_from_row(sample, nonzero=True)})
        rows.append(
            {
                key_name: key,
                "sample_count": len(group),
                "selection_ids": ",".join(str(sample["selection_id"]) for sample in group),
                "original_crc_values": ",".join(str(value) for value in crc_values),
                "nonzero_original_crc_values": ",".join(str(value) for value in nonzero_crc_values),
                "distinct_nonzero_original_crc_count": len(nonzero_crc_values),
                "source_paths": "\n".join(sample["source_sample_path"] for sample in group[:10]),
            }
        )
    return rows


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset-dir", type=Path, default=DATASET_DIR)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()

    dataset_dir = args.dataset_dir if args.dataset_dir.is_absolute() else ROOT / args.dataset_dir
    output_dir = args.output_dir if args.output_dir.is_absolute() else ROOT / args.output_dir
    if output_dir.exists():
        if not args.overwrite:
            raise SystemExit(f"Output directory already exists: {output_dir}")
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True)

    manifest_path = dataset_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    helper = compile_helper(output_dir)

    samples = manifest["copied_samples"]
    sample_rows: list[dict[str, Any]] = []
    metadata_match_rows: list[dict[str, Any]] = []
    pcm_match_rows: list[dict[str, Any]] = []
    asd_match_rows: list[dict[str, Any]] = []
    asd_embedded_rows: list[dict[str, Any]] = []
    decode_errors: list[dict[str, Any]] = []

    with tempfile.TemporaryDirectory(prefix="als-original-crc-pcm-") as temp_dir_raw:
        temp_dir = Path(temp_dir_raw)

        for sample in samples:
            copied_sample_path = ROOT / sample["copied_sample_path"]
            sample_sha256 = sha256_file(copied_sample_path)
            if sample_sha256 != sample["sha256"]:
                raise RuntimeError(f"SHA mismatch for copied sample {copied_sample_path}")

            metadata = afinfo(copied_sample_path)
            signatures = metadata_signatures(sample, metadata)
            nonzero_crcs = unique_nonzero_crcs(sample)

            pcm_wav_path = temp_dir / f"{sample['selection_id']:04d}.wav"
            decode_ok, decode_error = decode_to_pcm_wav(copied_sample_path, pcm_wav_path)
            pcm_sha256 = None
            pcm_size = None
            pcm_data_sha256 = None
            pcm_data_size = None
            if decode_ok:
                pcm_sha256 = sha256_file(pcm_wav_path)
                pcm_data = read_wav_data_chunk(pcm_wav_path)
                if pcm_data is not None:
                    pcm_data_size = len(pcm_data)
                    pcm_data_sha256 = hashlib.sha256(pcm_data).hexdigest()
                for crc in nonzero_crcs:
                    for row in run_crc_helper(helper, crc, pcm_wav_path, "decoded_pcm_wav"):
                        row.update(
                            {
                                "selection_id": sample["selection_id"],
                                "sample_path": sample["copied_sample_path"],
                                "sample_sha256": sample_sha256,
                                "pcm_data_sha256": pcm_data_sha256,
                            }
                        )
                        pcm_match_rows.append(row)
                pcm_size = pcm_wav_path.stat().st_size
            else:
                decode_errors.append(
                    {
                        "selection_id": sample["selection_id"],
                        "sample_path": sample["copied_sample_path"],
                        "error": decode_error,
                    }
                )

            asd = sample.get("asd_sidecar", {})
            asd_path = ROOT / asd["copied_path"] if asd.get("copied_path") else None
            asd_sha256 = None
            asd_size = None
            if asd_path and asd_path.exists():
                asd_sha256 = sha256_file(asd_path)
                asd_size = asd_path.stat().st_size
                for crc in nonzero_crcs:
                    for row in run_crc_helper(helper, crc, asd_path, "asd_sidecar"):
                        row.update(
                            {
                                "selection_id": sample["selection_id"],
                                "sample_path": sample["copied_sample_path"],
                                "sample_sha256": sample_sha256,
                                "asd_sha256": asd_sha256,
                            }
                        )
                        asd_match_rows.append(row)
                    for row in find_encoded_value(asd_path, crc):
                        row.update(
                            {
                                "selection_id": sample["selection_id"],
                                "sample_path": sample["copied_sample_path"],
                                "sample_sha256": sample_sha256,
                                "asd_sha256": asd_sha256,
                                "expected_original_crc": crc,
                                "asd_size": asd_size,
                            }
                        )
                        asd_embedded_rows.append(row)

            for crc in nonzero_crcs:
                for candidate_name, candidate_value in field_candidates(sample, metadata):
                    if candidate_value == crc:
                        metadata_match_rows.append(
                            {
                                "selection_id": sample["selection_id"],
                                "sample_path": sample["copied_sample_path"],
                                "sample_sha256": sample_sha256,
                                "original_crc": crc,
                                "candidate": candidate_name,
                                "candidate_value": candidate_value,
                            }
                        )

            sample_rows.append(
                {
                    "selection_id": sample["selection_id"],
                    "source_sample_path": sample["source_sample_path"],
                    "copied_sample_path": sample["copied_sample_path"],
                    "extension": sample["extension"],
                    "sample_sha256": sample_sha256,
                    "source_size_bytes": sample["source_size_bytes"],
                    "original_crc_values": ",".join(str(value) for value in all_crcs(sample)),
                    "nonzero_original_crc_values": ",".join(str(value) for value in nonzero_crcs),
                    "original_file_size_values": ",".join(str(value) for value in sample.get("original_file_size_values", [])),
                    "default_duration_values": ",".join(str(value) for value in sample.get("default_duration_values", [])),
                    "default_sample_rate_values": ",".join(str(value) for value in sample.get("default_sample_rate_values", [])),
                    "relative_path_type_values": ",".join(str(value) for value in sample.get("relative_path_type_values", [])),
                    "afinfo_ok": metadata.get("afinfo_ok"),
                    "file_type_id": metadata.get("file_type_id"),
                    "codec": metadata.get("codec"),
                    "channels": metadata.get("channels"),
                    "sample_rate_hz": metadata.get("sample_rate_hz"),
                    "bit_depth": metadata.get("bit_depth"),
                    "source_bit_depth": metadata.get("source_bit_depth"),
                    "estimated_duration_sec": metadata.get("estimated_duration_sec"),
                    "estimated_duration_ms": metadata.get("estimated_duration_ms"),
                    "estimated_frame_count": metadata.get("estimated_frame_count"),
                    "audio_bytes": metadata.get("audio_bytes"),
                    "audio_packets": metadata.get("audio_packets"),
                    "bit_rate": metadata.get("bit_rate"),
                    "audio_data_file_offset": metadata.get("audio_data_file_offset"),
                    "format_signature": signatures["format_signature"],
                    "duration_signature": signatures["duration_signature"],
                    "als_audio_signature": signatures["als_audio_signature"],
                    "pcm_decode_ok": decode_ok,
                    "pcm_wav_sha256": pcm_sha256,
                    "pcm_wav_size": pcm_size,
                    "pcm_data_sha256": pcm_data_sha256,
                    "pcm_data_size": pcm_data_size,
                    "asd_exists": bool(asd_path and asd_path.exists()),
                    "asd_sha256": asd_sha256,
                    "asd_size": asd_size,
                }
            )

    same_sha_rows = grouped_rows_by_key(sample_rows, "sample_sha256")
    same_pcm_rows = [row for row in grouped_rows_by_key(sample_rows, "pcm_data_sha256") if row["pcm_data_sha256"] != "None"]
    same_asd_rows = [row for row in grouped_rows_by_key(sample_rows, "asd_sha256") if row["asd_sha256"] != "None"]

    by_crc: dict[int, list[dict[str, Any]]] = collections.defaultdict(list)
    for row in sample_rows:
        for crc_text in row["nonzero_original_crc_values"].split(","):
            if not crc_text:
                continue
            by_crc[int(crc_text)].append(row)

    collision_rows = []
    for crc, group in sorted(by_crc.items(), key=lambda item: (-len(item[1]), item[0])):
        sha_values = sorted({row["sample_sha256"] for row in group})
        pcm_values = sorted({row["pcm_data_sha256"] for row in group if row["pcm_data_sha256"]})
        asd_values = sorted({row["asd_sha256"] for row in group if row["asd_sha256"]})
        size_values = sorted({int(row["source_size_bytes"]) for row in group})
        collision_rows.append(
            {
                "original_crc": crc,
                "sample_count": len(group),
                "sha256_count": len(sha_values),
                "pcm_data_sha256_count": len(pcm_values),
                "asd_sha256_count": len(asd_values),
                "source_size_count": len(size_values),
                "selection_ids": ",".join(str(row["selection_id"]) for row in group),
                "source_sizes": ",".join(str(value) for value in size_values[:20]),
                "sample_paths": "\n".join(row["copied_sample_path"] for row in group[:20]),
            }
        )

    signature_rows = []
    for signature_key in ["format_signature", "duration_signature", "als_audio_signature"]:
        groups: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
        for row in sample_rows:
            groups[row[signature_key]].append(row)
        for signature, group in sorted(groups.items(), key=lambda item: (-len(item[1]), item[0])):
            crc_values = sorted({crc for row in group for crc in [parse_int(v) for v in row["nonzero_original_crc_values"].split(",") if v] if crc is not None})
            signature_rows.append(
                {
                    "signature_type": signature_key,
                    "signature": signature,
                    "sample_count": len(group),
                    "nonzero_original_crc_count": len(crc_values),
                    "nonzero_original_crc_values": ",".join(str(value) for value in crc_values[:30]),
                    "selection_ids": ",".join(str(row["selection_id"]) for row in group[:30]),
                }
            )

    metadata_candidate_counts = collections.Counter(row["candidate"] for row in metadata_match_rows)
    pcm_match_counts = collections.Counter(f"{row['context']}:{row['function']}:{row['variant']}" for row in pcm_match_rows)
    asd_match_counts = collections.Counter(f"{row['context']}:{row['function']}:{row['variant']}" for row in asd_match_rows)
    asd_embedded_counts = collections.Counter(row["encoding"] for row in asd_embedded_rows)

    report = {
        "status": "completed",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "dataset_dir": str(dataset_dir.relative_to(ROOT)),
        "output_dir": str(output_dir.relative_to(ROOT)),
        "tooling": {
            "afinfo": shutil.which("afinfo"),
            "afconvert": shutil.which("afconvert"),
            "c_helper_source": str(C_HELPER_SOURCE.relative_to(ROOT)),
        },
        "sample_count": len(sample_rows),
        "decode_error_count": len(decode_errors),
        "same_sha256": {
            "unique_sha256_count": len({row["sample_sha256"] for row in sample_rows}),
            "groups_with_more_than_one_sample": sum(1 for row in same_sha_rows if int(row["sample_count"]) > 1),
            "groups_with_multiple_nonzero_original_crc": sum(1 for row in same_sha_rows if int(row["distinct_nonzero_original_crc_count"]) > 1),
        },
        "original_crc_collisions": {
            "unique_nonzero_original_crc_count": len(by_crc),
            "crc_values_with_more_than_one_sample": sum(1 for row in collision_rows if row["sample_count"] > 1),
            "crc_values_with_more_than_one_sha256": sum(1 for row in collision_rows if row["sha256_count"] > 1),
            "largest_collision_sample_count": max((row["sample_count"] for row in collision_rows), default=0),
        },
        "decoded_pcm": {
            "decoded_sample_count": sum(1 for row in sample_rows if row["pcm_decode_ok"]),
            "unique_pcm_data_sha256_count": len({row["pcm_data_sha256"] for row in sample_rows if row["pcm_data_sha256"]}),
            "pcm_hash_groups_with_more_than_one_sample": sum(1 for row in same_pcm_rows if int(row["sample_count"]) > 1),
            "pcm_hash_groups_with_multiple_nonzero_original_crc": sum(1 for row in same_pcm_rows if int(row["distinct_nonzero_original_crc_count"]) > 1),
            "checksum_match_rows": len(pcm_match_rows),
            "checksum_match_candidates": pcm_match_counts.most_common(20),
        },
        "asd": {
            "samples_with_asd": sum(1 for row in sample_rows if row["asd_exists"]),
            "unique_asd_sha256_count": len({row["asd_sha256"] for row in sample_rows if row["asd_sha256"]}),
            "asd_hash_groups_with_more_than_one_sample": sum(1 for row in same_asd_rows if int(row["sample_count"]) > 1),
            "asd_hash_groups_with_multiple_nonzero_original_crc": sum(1 for row in same_asd_rows if int(row["distinct_nonzero_original_crc_count"]) > 1),
            "checksum_match_rows": len(asd_match_rows),
            "checksum_match_candidates": asd_match_counts.most_common(20),
            "embedded_original_crc_rows": len(asd_embedded_rows),
            "embedded_original_crc_encodings": asd_embedded_counts.most_common(),
        },
        "metadata": {
            "metadata_candidate_match_rows": len(metadata_match_rows),
            "metadata_candidate_counts": metadata_candidate_counts.most_common(30),
            "format_signatures_with_multiple_crc": sum(
                1 for row in signature_rows if row["signature_type"] == "format_signature" and row["nonzero_original_crc_count"] > 1
            ),
            "duration_signatures_with_multiple_crc": sum(
                1 for row in signature_rows if row["signature_type"] == "duration_signature" and row["nonzero_original_crc_count"] > 1
            ),
            "als_audio_signatures_with_multiple_crc": sum(
                1 for row in signature_rows if row["signature_type"] == "als_audio_signature" and row["nonzero_original_crc_count"] > 1
            ),
        },
        "interpretation": {
            "original_crc_as_file_sha": "not_supported_if_same_sha_groups_have_multiple_crc_or_crc_collision_groups_have_multiple_sha",
            "original_crc_as_decoded_pcm_hash": "not_supported_by_direct_hash_grouping; checksum matches need repeated candidate evidence",
            "original_crc_as_asd_hash": "not_supported_by direct hash grouping; checksum matches need repeated candidate evidence",
            "metadata_only_formula": "not_supported_if same metadata signatures map to multiple OriginalCrc values",
        },
    }

    write_json(output_dir / "static_analysis_report.json", report)
    write_csv(
        output_dir / "samples_enriched.csv",
        sample_rows,
        [
            "selection_id",
            "copied_sample_path",
            "extension",
            "sample_sha256",
            "source_size_bytes",
            "original_crc_values",
            "nonzero_original_crc_values",
            "original_file_size_values",
            "default_duration_values",
            "default_sample_rate_values",
            "relative_path_type_values",
            "file_type_id",
            "codec",
            "channels",
            "sample_rate_hz",
            "bit_depth",
            "estimated_duration_sec",
            "estimated_frame_count",
            "audio_bytes",
            "audio_packets",
            "format_signature",
            "duration_signature",
            "als_audio_signature",
            "pcm_decode_ok",
            "pcm_data_sha256",
            "pcm_data_size",
            "asd_exists",
            "asd_sha256",
            "asd_size",
        ],
    )
    write_csv(
        output_dir / "same_sha256_vs_original_crc.csv",
        same_sha_rows,
        [
            "sample_sha256",
            "sample_count",
            "selection_ids",
            "original_crc_values",
            "nonzero_original_crc_values",
            "distinct_nonzero_original_crc_count",
            "source_paths",
        ],
    )
    write_csv(
        output_dir / "original_crc_collisions.csv",
        collision_rows,
        [
            "original_crc",
            "sample_count",
            "sha256_count",
            "pcm_data_sha256_count",
            "asd_sha256_count",
            "source_size_count",
            "selection_ids",
            "source_sizes",
            "sample_paths",
        ],
    )
    write_csv(
        output_dir / "same_pcm_sha256_vs_original_crc.csv",
        same_pcm_rows,
        [
            "pcm_data_sha256",
            "sample_count",
            "selection_ids",
            "original_crc_values",
            "nonzero_original_crc_values",
            "distinct_nonzero_original_crc_count",
            "source_paths",
        ],
    )
    write_csv(
        output_dir / "same_asd_sha256_vs_original_crc.csv",
        same_asd_rows,
        [
            "asd_sha256",
            "sample_count",
            "selection_ids",
            "original_crc_values",
            "nonzero_original_crc_values",
            "distinct_nonzero_original_crc_count",
            "source_paths",
        ],
    )
    write_csv(
        output_dir / "decoded_pcm_checksum_matches.csv",
        pcm_match_rows,
        [
            "selection_id",
            "sample_path",
            "sample_sha256",
            "pcm_data_sha256",
            "context",
            "function",
            "variant",
            "value",
            "expected_original_crc",
        ],
    )
    write_csv(
        output_dir / "asd_checksum_matches.csv",
        asd_match_rows,
        [
            "selection_id",
            "sample_path",
            "sample_sha256",
            "asd_sha256",
            "context",
            "function",
            "variant",
            "value",
            "expected_original_crc",
        ],
    )
    write_csv(
        output_dir / "asd_embedded_original_crc_matches.csv",
        asd_embedded_rows,
        [
            "selection_id",
            "sample_path",
            "sample_sha256",
            "asd_sha256",
            "asd_size",
            "expected_original_crc",
            "encoding",
            "needle_hex",
            "match_count_capped",
            "first_offsets",
        ],
    )
    write_csv(
        output_dir / "metadata_candidate_matches.csv",
        metadata_match_rows,
        ["selection_id", "sample_path", "sample_sha256", "original_crc", "candidate", "candidate_value"],
    )
    write_csv(
        output_dir / "metadata_signature_groups.csv",
        signature_rows,
        ["signature_type", "signature", "sample_count", "nonzero_original_crc_count", "nonzero_original_crc_values", "selection_ids"],
    )
    write_json(output_dir / "decode_errors.json", decode_errors)

    readme_lines = [
        "# OriginalCrc Static Analysis Probe",
        "",
        "Status: completed",
        f"Date: {datetime.now().date().isoformat()}",
        "",
        "## Scope",
        "",
        "Tests run on copied balanced dataset only.",
        "",
        "```text",
        f"samples: {report['sample_count']}",
        f"decoded PCM samples: {report['decoded_pcm']['decoded_sample_count']}",
        f"samples with .asd: {report['asd']['samples_with_asd']}",
        "```",
        "",
        "## Key Outputs",
        "",
        "```text",
        "static_analysis_report.json",
        "samples_enriched.csv",
        "same_sha256_vs_original_crc.csv",
        "original_crc_collisions.csv",
        "same_pcm_sha256_vs_original_crc.csv",
        "decoded_pcm_checksum_matches.csv",
        "same_asd_sha256_vs_original_crc.csv",
        "asd_checksum_matches.csv",
        "asd_embedded_original_crc_matches.csv",
        "metadata_candidate_matches.csv",
        "metadata_signature_groups.csv",
        "```",
        "",
        "## Safety",
        "",
        "```text",
        "Original ALS files were not modified.",
        "Original sample files were not modified.",
        "The script reads/copied dataset files and writes reports only.",
        "```",
        "",
    ]
    (output_dir / "README.md").write_text("\n".join(readme_lines), encoding="utf-8")

    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
