#!/usr/bin/env python3
"""Probe hypotheses for Ableton ALS OriginalCrc values.

This script is intentionally experimental. It does not modify source samples.
It reads the copied sample subset manifest and compares ALS OriginalCrc against
many cheap checksum/CRC hypotheses over full files, audio payload chunks and
selected file slices.
"""

from __future__ import annotations

import argparse
import csv
import json
import struct
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable


ROOT = Path(__file__).resolve().parents[3]
SLOW_CONTEXT_MAX_BYTES = 128 * 1024


def reflect(value: int, bits: int) -> int:
    out = 0
    for _ in range(bits):
        out = (out << 1) | (value & 1)
        value >>= 1
    return out


@dataclass(frozen=True)
class CrcSpec:
    name: str
    width: int
    poly: int
    init: int
    refin: bool
    refout: bool
    xorout: int

    @property
    def mask(self) -> int:
        return (1 << self.width) - 1

    @property
    def topbit(self) -> int:
        return 1 << (self.width - 1)

    def table(self) -> list[int]:
        mask = self.mask
        table = []
        if self.refin:
            poly = reflect(self.poly, self.width)
            for i in range(256):
                crc = i
                for _ in range(8):
                    if crc & 1:
                        crc = (crc >> 1) ^ poly
                    else:
                        crc >>= 1
                table.append(crc & mask)
        else:
            shift = self.width - 8
            for i in range(256):
                crc = i << shift
                for _ in range(8):
                    if crc & self.topbit:
                        crc = (crc << 1) ^ self.poly
                    else:
                        crc <<= 1
                table.append(crc & mask)
        return table

    def compute(self, data: bytes) -> int:
        mask = self.mask
        table = CRC_TABLES[self.name]
        crc = self.init & mask
        if self.refin:
            for byte in data:
                crc = ((crc >> 8) ^ table[(crc ^ byte) & 0xFF]) & mask
        else:
            shift = self.width - 8
            for byte in data:
                crc = ((crc << 8) ^ table[((crc >> shift) ^ byte) & 0xFF]) & mask
        if self.refout != self.refin:
            crc = reflect(crc, self.width)
        return (crc ^ self.xorout) & mask


CRC16_SPECS = [
    CrcSpec("crc16_arc", 16, 0x8005, 0x0000, True, True, 0x0000),
    CrcSpec("crc16_modbus", 16, 0x8005, 0xFFFF, True, True, 0x0000),
    CrcSpec("crc16_usb", 16, 0x8005, 0xFFFF, True, True, 0xFFFF),
    CrcSpec("crc16_maxim", 16, 0x8005, 0x0000, True, True, 0xFFFF),
    CrcSpec("crc16_ccitt_false", 16, 0x1021, 0xFFFF, False, False, 0x0000),
    CrcSpec("crc16_xmodem", 16, 0x1021, 0x0000, False, False, 0x0000),
    CrcSpec("crc16_kermit", 16, 0x1021, 0x0000, True, True, 0x0000),
    CrcSpec("crc16_x25", 16, 0x1021, 0xFFFF, True, True, 0xFFFF),
    CrcSpec("crc16_dnp", 16, 0x3D65, 0x0000, True, True, 0xFFFF),
    CrcSpec("crc16_t10_dif", 16, 0x8BB7, 0x0000, False, False, 0x0000),
    CrcSpec("crc16_dect_r", 16, 0x0589, 0x0000, False, False, 0x0001),
    CrcSpec("crc16_cdma2000", 16, 0xC867, 0xFFFF, False, False, 0x0000),
]

CRC32_SPECS = [
    CrcSpec("crc32_mpeg2", 32, 0x04C11DB7, 0xFFFFFFFF, False, False, 0x00000000),
    CrcSpec("crc32_bzip2", 32, 0x04C11DB7, 0xFFFFFFFF, False, False, 0xFFFFFFFF),
    CrcSpec("crc32_posix_like_no_len", 32, 0x04C11DB7, 0x00000000, False, False, 0xFFFFFFFF),
    CrcSpec("crc32c_castagnoli", 32, 0x1EDC6F41, 0xFFFFFFFF, True, True, 0xFFFFFFFF),
    CrcSpec("crc32_jamcrc", 32, 0x04C11DB7, 0xFFFFFFFF, True, True, 0x00000000),
]

CRC_TABLES = {spec.name: spec.table() for spec in [*CRC16_SPECS, *CRC32_SPECS]}


def byteswap16(value: int) -> int:
    return ((value & 0xFF) << 8) | ((value >> 8) & 0xFF)


def sum8(data: bytes) -> int:
    return sum(data) & 0xFFFF


def sum_words(data: bytes, endian: str) -> int:
    if len(data) % 2:
        data = data + b"\x00"
    total = 0
    for i in range(0, len(data), 2):
        total = (total + int.from_bytes(data[i : i + 2], endian)) & 0xFFFF
    return total


def ones_complement_sum16(data: bytes, endian: str) -> int:
    if len(data) % 2:
        data = data + b"\x00"
    total = 0
    for i in range(0, len(data), 2):
        total += int.from_bytes(data[i : i + 2], endian)
        total = (total & 0xFFFF) + (total >> 16)
    return (~total) & 0xFFFF


def fletcher16(data: bytes) -> int:
    sum1 = 0
    sum2 = 0
    for byte in data:
        sum1 = (sum1 + byte) % 255
        sum2 = (sum2 + sum1) % 255
    return ((sum2 << 8) | sum1) & 0xFFFF


def fletcher32(data: bytes) -> int:
    if len(data) % 2:
        data = data + b"\x00"
    sum1 = 0xFFFF
    sum2 = 0xFFFF
    words = [int.from_bytes(data[i : i + 2], "big") for i in range(0, len(data), 2)]
    i = 0
    while i < len(words):
        block = words[i : i + 360]
        for word in block:
            sum1 = (sum1 + word) & 0xFFFFFFFF
            sum2 = (sum2 + sum1) & 0xFFFFFFFF
        sum1 = (sum1 & 0xFFFF) + (sum1 >> 16)
        sum2 = (sum2 & 0xFFFF) + (sum2 >> 16)
        i += 360
    sum1 = (sum1 & 0xFFFF) + (sum1 >> 16)
    sum2 = (sum2 & 0xFFFF) + (sum2 >> 16)
    return ((sum2 << 16) | sum1) & 0xFFFFFFFF


def strip_id3(data: bytes) -> bytes | None:
    if not data.startswith(b"ID3") or len(data) < 10:
        return None
    size = 0
    for byte in data[6:10]:
        size = (size << 7) | (byte & 0x7F)
    extra_footer = 10 if (data[5] & 0x10) else 0
    start = 10 + size + extra_footer
    return data[start:] if start < len(data) else b""


def wav_data_chunk(data: bytes) -> bytes | None:
    if len(data) < 12 or data[0:4] != b"RIFF" or data[8:12] != b"WAVE":
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


def aiff_ssnd_payload(data: bytes) -> bytes | None:
    if len(data) < 12 or data[0:4] != b"FORM" or data[8:12] not in (b"AIFF", b"AIFC"):
        return None
    pos = 12
    while pos + 8 <= len(data):
        chunk_id = data[pos : pos + 4]
        size = struct.unpack_from(">I", data, pos + 4)[0]
        start = pos + 8
        end = min(start + size, len(data))
        if chunk_id == b"SSND" and end - start >= 8:
            offset = struct.unpack_from(">I", data, start)[0]
            payload_start = min(start + 8 + offset, end)
            return data[payload_start:end]
        pos = end + (size % 2)
    return None


def pair_swap(data: bytes) -> bytes:
    if len(data) % 2:
        data = data[:-1]
    return b"".join(data[i + 1 : i + 2] + data[i : i + 1] for i in range(0, len(data), 2))


def data_contexts(data: bytes) -> dict[str, bytes]:
    contexts: dict[str, bytes] = {
        "full_file": data,
        "first_64k": data[: 64 * 1024],
        "last_64k": data[-64 * 1024 :],
        "first_1m": data[: 1024 * 1024],
        "last_1m": data[-1024 * 1024 :],
    }
    mid = len(data) // 2
    start = max(0, mid - 32 * 1024)
    contexts["middle_64k"] = data[start : start + 64 * 1024]
    contexts["first_last_64k"] = data[: 64 * 1024] + data[-64 * 1024 :]

    wav_payload = wav_data_chunk(data)
    if wav_payload is not None:
        contexts["audio_payload_wav_data"] = wav_payload
        contexts["audio_payload_first_64k"] = wav_payload[: 64 * 1024]
        contexts["audio_payload_last_64k"] = wav_payload[-64 * 1024 :]
        if len(wav_payload) <= SLOW_CONTEXT_MAX_BYTES:
            contexts["audio_payload_pair_swapped"] = pair_swap(wav_payload)

    aiff_payload = aiff_ssnd_payload(data)
    if aiff_payload is not None:
        contexts["audio_payload_aiff_ssnd"] = aiff_payload
        contexts["audio_payload_first_64k"] = aiff_payload[: 64 * 1024]
        contexts["audio_payload_last_64k"] = aiff_payload[-64 * 1024 :]
        if len(aiff_payload) <= SLOW_CONTEXT_MAX_BYTES:
            contexts["audio_payload_pair_swapped"] = pair_swap(aiff_payload)

    mp3_no_id3 = strip_id3(data)
    if mp3_no_id3 is not None:
        contexts["mp3_without_id3"] = mp3_no_id3

    return contexts


def candidate_values(data: bytes) -> Iterable[tuple[str, int]]:
    crc32 = zlib.crc32(data) & 0xFFFFFFFF
    yield "crc32_ieee_low16", crc32 & 0xFFFF
    yield "crc32_ieee_high16", (crc32 >> 16) & 0xFFFF
    yield "crc32_ieee_low16_byteswap", byteswap16(crc32 & 0xFFFF)
    yield "crc32_ieee_high16_byteswap", byteswap16((crc32 >> 16) & 0xFFFF)

    adler = zlib.adler32(data) & 0xFFFFFFFF
    yield "adler32_low16", adler & 0xFFFF
    yield "adler32_high16", (adler >> 16) & 0xFFFF
    yield "adler32_low16_byteswap", byteswap16(adler & 0xFFFF)
    yield "adler32_high16_byteswap", byteswap16((adler >> 16) & 0xFFFF)

    if len(data) > SLOW_CONTEXT_MAX_BYTES:
        return

    for spec in CRC16_SPECS:
        value = spec.compute(data)
        yield spec.name, value
        yield f"{spec.name}_byteswap", byteswap16(value)

    for spec in CRC32_SPECS:
        value = spec.compute(data)
        yield f"{spec.name}_low16", value & 0xFFFF
        yield f"{spec.name}_high16", (value >> 16) & 0xFFFF
        yield f"{spec.name}_low16_byteswap", byteswap16(value & 0xFFFF)
        yield f"{spec.name}_high16_byteswap", byteswap16((value >> 16) & 0xFFFF)

    f32 = fletcher32(data)
    yield "fletcher32_low16", f32 & 0xFFFF
    yield "fletcher32_high16", (f32 >> 16) & 0xFFFF
    yield "fletcher16", fletcher16(data)
    yield "sum8_mod65536", sum8(data)
    yield "sum16_words_little", sum_words(data, "little")
    yield "sum16_words_big", sum_words(data, "big")
    yield "ones_complement_sum16_little", ones_complement_sum16(data, "little")
    yield "ones_complement_sum16_big", ones_complement_sum16(data, "big")


def flatten_samples(manifest: dict) -> list[dict]:
    out = []
    for als in manifest["selected_als"]:
        for sample in als["copied_samples"]:
            item = dict(sample)
            item["als_id"] = als["als_id"]
            item["als_copy_path"] = als["copy_path"]
            out.append(item)
    return out


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        type=Path,
        default=ROOT
        / "experiments/2026-06-02_als_structure_corpus_20/crc_probe_sample_subset/crc_probe_sample_subset_manifest.json",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=ROOT
        / "experiments/2026-06-02_als_structure_corpus_20/crc_probe_sample_subset/hypothesis_probe",
    )
    args = parser.parse_args()

    manifest = json.loads(args.manifest.read_text())
    samples = flatten_samples(manifest)
    args.out_dir.mkdir(parents=True, exist_ok=True)

    match_rows = []
    per_sample = []
    candidate_counts: dict[str, int] = {}
    candidate_contexts: dict[str, set[str]] = {}
    total_candidates_per_sample = None

    for sample in samples:
        original_crc_raw = sample.get("als_original_crc")
        if not str(original_crc_raw or "").isdigit():
            continue
        target = int(original_crc_raw)
        copied_path = ROOT / sample["copied_sample_path"]
        data = copied_path.read_bytes()
        contexts = data_contexts(data)
        sample_matches = []
        candidate_counter = 0
        skipped_slow_contexts = []
        for context_name, context_data in contexts.items():
            if len(context_data) > SLOW_CONTEXT_MAX_BYTES:
                skipped_slow_contexts.append(context_name)
            for function_name, value in candidate_values(context_data):
                candidate_counter += 1
                candidate_id = f"{context_name}:{function_name}"
                if value == target:
                    row = {
                        "als_id": sample["als_id"],
                        "ref_id": sample["ref_id"],
                        "filename": sample["filename"],
                        "extension": sample["extension"],
                        "als_original_crc": target,
                        "context": context_name,
                        "function": function_name,
                        "candidate_id": candidate_id,
                        "copied_sample_path": sample["copied_sample_path"],
                    }
                    match_rows.append(row)
                    sample_matches.append(row)
                    candidate_counts[candidate_id] = candidate_counts.get(candidate_id, 0) + 1
                    candidate_contexts.setdefault(function_name, set()).add(context_name)
        total_candidates_per_sample = candidate_counter
        per_sample.append(
            {
                "als_id": sample["als_id"],
                "ref_id": sample["ref_id"],
                "filename": sample["filename"],
                "extension": sample["extension"],
                "als_original_crc": target,
                "context_count": len(contexts),
                "candidate_count": candidate_counter,
                "slow_functions_skipped_for_contexts": sorted(set(skipped_slow_contexts)),
                "match_count": len(sample_matches),
                "matches": sample_matches,
            }
        )

    ranked = sorted(
        (
            {"candidate_id": candidate_id, "match_count": count}
            for candidate_id, count in candidate_counts.items()
        ),
        key=lambda row: (-row["match_count"], row["candidate_id"]),
    )

    strong_candidates = [row for row in ranked if row["match_count"] >= 2]

    report = {
        "manifest": str(args.manifest.relative_to(ROOT)),
        "sample_count": len(samples),
        "total_candidates_per_sample": total_candidates_per_sample,
        "total_match_rows": len(match_rows),
        "samples_with_any_match": sum(1 for row in per_sample if row["match_count"] > 0),
        "strong_candidate_threshold": 2,
        "strong_candidates": strong_candidates,
        "top_candidates": ranked[:25],
        "per_sample": per_sample,
        "interpretation": {
            "all_samples_match_required_for_confirmed_algorithm": len(samples),
            "status": "no_confirmed_algorithm"
            if not any(row["match_count"] == len(samples) for row in ranked)
            else "candidate_found",
            "note": "Single matches are expected by chance when many 16-bit candidates are tested.",
            "slow_context_max_bytes": SLOW_CONTEXT_MAX_BYTES,
            "slow_context_note": "CRC16 and slow word-sum hypotheses are skipped for contexts above this byte limit; fast CRC32/Adler32 transforms still run.",
        },
    }

    (args.out_dir / "original_crc_hypothesis_report.json").write_text(
        json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    )

    with (args.out_dir / "original_crc_hypothesis_matches.csv").open("w", newline="") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=[
                "als_id",
                "ref_id",
                "filename",
                "extension",
                "als_original_crc",
                "context",
                "function",
                "candidate_id",
                "copied_sample_path",
            ],
        )
        writer.writeheader()
        writer.writerows(match_rows)

    summary_lines = [
        "# OriginalCrc Hypothesis Probe",
        "",
        "Status: completed",
        "",
        "## Summary",
        "",
        "```text",
        f"samples tested: {len(samples)}",
        f"candidate values per sample: {total_candidates_per_sample}",
        f"total exact match rows: {len(match_rows)}",
        f"samples with any exact match: {report['samples_with_any_match']}",
        f"strong candidates matching >=2 samples: {len(strong_candidates)}",
        f"status: {report['interpretation']['status']}",
        "```",
        "",
        "## Strong Candidates",
        "",
    ]
    if strong_candidates:
        summary_lines.append("```text")
        for row in strong_candidates[:25]:
            summary_lines.append(f"{row['candidate_id']}: {row['match_count']}")
        summary_lines.append("```")
    else:
        summary_lines.append("```text")
        summary_lines.append("none")
        summary_lines.append("```")
    summary_lines.extend(
        [
            "",
            "## Interpretation",
            "",
            "```text",
            "No tested hypothesis matched all samples.",
            "Single exact matches are weak evidence because OriginalCrc is 16-bit-like",
            "and this probe tests many candidate values per sample.",
            "The result weakens common CRC/checksum hypotheses but does not prove",
            "Ableton's internal algorithm.",
            "```",
            "",
        ]
    )
    (args.out_dir / "README.md").write_text("\n".join(summary_lines))

    print(json.dumps(report, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
