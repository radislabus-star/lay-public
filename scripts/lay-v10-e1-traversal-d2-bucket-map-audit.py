#!/usr/bin/env python3
"""Independent read-only audit of the sealed primary-only D2 bucket map."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import shlex
import shutil
import stat
import subprocess
import sys
import time
from collections import Counter
from typing import Any, Iterable, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

AUDITOR = pathlib.Path(__file__).resolve()
PROJECT_ROOT = AUDITOR.parents[1]
MAP_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUCKET_MAP_V1_2026-08-26"
)
BUILD_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUILD_V1_2026-08-25"
)
AUDIT_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUCKET_MAP_AUDIT_V1_2026-08-26"
)
D2_ELF = BUILD_RESULT / "REMOTE_EVIDENCE/d2-test-elf"
BUCKET_MAP = MAP_RESULT / "REMOTE_EVIDENCE/D2_BUCKET_MAP.json"
REMOTE_RECEIPT = MAP_RESULT / "REMOTE_EVIDENCE/D2_BUCKET_MAP_RECEIPT.json"
LOCAL_RECEIPT = MAP_RESULT / "LOCAL_BUCKET_MAP_RECEIPT.json"
SEALED_ADDRESSES = MAP_RESULT / "REMOTE_EVIDENCE/SEALED_ADDRESSES.txt"
ADDR2LINE_OUTPUT = MAP_RESULT / "REMOTE_EVIDENCE/raw/ADDR2LINE.txt"
ADDR2LINE_PRODUCER = MAP_RESULT / "REMOTE_EVIDENCE/raw/ADDR2LINE_PRODUCER.json"

REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_RESULT = REMOTE_PARENT / "bucket-map-v1"
REMOTE_FAILURE = REMOTE_PARENT / "bucket-map-failure-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
ELF_SIZE = 317_706_232
BUILD_ID = "eb951f1a7526a9f1cb365040c10989aa5d3fc50f"
TEXT_START = 0x3F72C0
TEXT_OFFSET = 0x3F62C0
TEXT_SIZE = 15_980_919
TEXT_END = TEXT_START + TEXT_SIZE
TEXT_SHA256 = "f57eba60bc4b1cadbeb2dfc524af59a7ab011a2e64afb0e1a0fe610755129d94"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
REMOTE_RECEIPT_SHA256 = "8197087fff1853b7e4c167e92ae4900eb57821c298b918bfba7dee4c477f3430"
LOCAL_RECEIPT_SHA256 = "21d79d9c5b3b0f4b9e60ce23bab07a689ae7c768c37b01aaf7ce94b95a7c33cc"
LOCAL_MANIFEST_SHA256 = "48fce814340a62afdfbfbd62f382539d9e29fe385d0bf8cdcb7f16b0fe8a079d"
REMOTE_MANIFEST_SHA256 = "911b142e9eeae85547765b92568c250a4682f570dd40ea5dd8e6a6d24b6d242d"
ADDRESS_LIST_SHA256 = "fca1804a3d0af34ae462938f970fd181706846da6e42dbb725c5ef1775581c58"
ADDR2LINE_SHA256 = "8b9b4767557a3ea019bbaebb280d1a56ab2180f34ad1e05aed9c2affb4c8a9e6"
INSTRUCTION_COUNT = 1_064

SYMBOLS = {
    "hot": (0x778320, 0x7793AE),
    "edge": (0x926520, 0x926643),
    "state": (0x9266B0, 0x926808),
}
EXPECTED_HOT_RANGES = (
    (0x778320, 0x7783DA, "STACK_CONTROL", "BUDGET_DEADLINE", False),
    (0x7783DA, 0x7784E3, "STACK_CONTROL", "STACK_PUSH", False),
    (0x7784E3, 0x7785BE, "STACK_CONTROL", "SCRATCH_BOOKKEEPING", False),
    (0x7785BE, 0x7785DA, "STACK_CONTROL", "STACK_POP", False),
    (0x7785DA, 0x7786AF, "STACK_CONTROL", "BUDGET_DEADLINE", False),
    (0x7786AF, 0x7786F0, "DAFSA_DECODE_MEMORY", "STATE_DECODE", False),
    (0x7786F0, 0x7786FB, "TERMINAL", "TERMINAL_PREDICATE", False),
    (0x7786FB, 0x778725, "DAFSA_DECODE_MEMORY", "EDGE_RANGE_CONTROL", False),
    (0x778725, 0x778A7C, "TRANSITION", "FUSED_SCALAR_U64_ADVANCE", False),
    (0x778A7C, 0x778AA9, "STACK_CONTROL", "STACK_PUSH", False),
    (0x778AA9, 0x778AC2, "STACK_CONTROL", "PRUNE_AND_LOOP", False),
    (0x778AC2, 0x778B0E, "DAFSA_DECODE_MEMORY", "EDGE_DECODE", False),
    (0x778B0E, 0x778B18, "RANK", "EDGE_RANK_ADD", False),
    (0x778B18, 0x778B8F, "TRANSITION", "ALPHABET_ID", False),
    (0x778B8F, 0x778C2E, "TRANSITION", "EQUALITY_WINDOW", False),
    (0x778C2E, 0x778DB7, "TRANSITION", "FUSED_SCALAR_U64_ADVANCE", False),
    (0x778DB7, 0x778DC4, "STACK_CONTROL", "PRUNE_AND_LOOP", False),
    (0x778DC4, 0x778E09, "STACK_CONTROL", "STACK_PUSH", False),
    (0x778E09, 0x778F26, "TRANSITION", "FUSED_SCALAR_U64_ADVANCE", False),
    (0x778F26, 0x778F30, "UNATTRIBUTED", "UNATTRIBUTED", True),
    (0x778F30, 0x778F6F, "TERMINAL", "TERMINAL_DISTANCE", False),
    (0x778F6F, 0x778F7C, "TERMINAL", "TERMINAL_PREDICATE", False),
    (0x778F7C, 0x778FA5, "TERMINAL", "FORM_REF_COLLECTION", False),
    (0x778FA5, 0x778FF9, "STACK_CONTROL", "BUDGET_DEADLINE", False),
    (0x778FF9, 0x779055, "STACK_CONTROL", "PRUNE_AND_LOOP", False),
    (0x779055, 0x77909A, "STACK_CONTROL", "BUDGET_DEADLINE", False),
    (0x77909A, 0x7790C2, "DAFSA_DECODE_MEMORY", "STATE_DECODE", False),
    (0x7790C2, 0x779120, "DAFSA_DECODE_MEMORY", "EDGE_RANGE_CONTROL", False),
    (0x779120, 0x779160, "STACK_CONTROL", "BUDGET_DEADLINE", False),
    (0x779160, 0x7791BB, "STACK_CONTROL", "SCRATCH_BOOKKEEPING", False),
    (0x7791BB, 0x7791EA, "UNATTRIBUTED", "UNATTRIBUTED", True),
    (0x7791EA, 0x779245, "DAFSA_DECODE_MEMORY", "EDGE_DECODE", False),
    (0x779245, 0x77924F, "RANK", "EDGE_RANK_ADD", False),
    (0x77924F, 0x7792C3, "TRANSITION", "ALPHABET_ID", False),
    (0x7792C3, 0x7792FF, "STACK_CONTROL", "PRUNE_AND_LOOP", False),
    (0x7792FF, 0x779344, "RANK", "EDGE_RANK_ADD", False),
    (0x779344, 0x77935C, "TRANSITION", "ALPHABET_ID", False),
    (0x77935C, 0x7793AE, "UNATTRIBUTED", "UNATTRIBUTED", True),
)
EXPECTED_MECHANISM_RANGES = (
    *EXPECTED_HOT_RANGES,
    (0x926520, 0x92653A, "DAFSA_DECODE_MEMORY", "EDGE_DECODE", False),
    (0x92653A, 0x926551, "DAFSA_DECODE_MEMORY", "SYMBOL_DECODE", False),
    (0x926551, 0x926643, "DAFSA_DECODE_MEMORY", "EDGE_DECODE", False),
    (0x9266B0, 0x926808, "DAFSA_DECODE_MEMORY", "STATE_DECODE", False),
)
EXPECTED_OUTSIDE = (
    (TEXT_START, SYMBOLS["hot"][0]),
    (SYMBOLS["hot"][1], SYMBOLS["edge"][0]),
    (SYMBOLS["edge"][1], SYMBOLS["state"][0]),
    (SYMBOLS["state"][1], TEXT_END),
)
EXPECTED_BUCKET_BYTES = {
    "DAFSA_DECODE_MEMORY": 1_043,
    "OUTSIDE_TRAVERSAL": 15_976_046,
    "RANK": 89,
    "STACK_CONTROL": 1_523,
    "TERMINAL": 128,
    "TRANSITION": 1_951,
    "UNATTRIBUTED": 139,
}
MARKERS = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.consumed-before-exec": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.available": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "u-single.available": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.available": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.available": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.available": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.available": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
    "t-single.available": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def file_identity(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def require_file(path: pathlib.Path, *, digest: str | None = None, size: int | None = None, mode: str | None = None) -> dict[str, Any]:
    value = file_identity(path)
    if digest is not None:
        require(value["sha256"] == digest, f"SHA mismatch: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode mismatch: {path}")
    return value


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    rows = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            rows.append({"path": path.relative_to(root).as_posix(), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)})
    return rows


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts, f"unsafe manifest path: {relative}")
        require(relative not in seen and relative != "SHA256SUMS", f"duplicate manifest row: {relative}")
        seen.add(relative)
        require(sha256_file(root / path) == digest, f"manifest mismatch: {relative}")
    actual = {row["path"] for row in inventory(root) if row["path"] != "SHA256SUMS"}
    require(seen == actual, f"manifest membership mismatch: {root}")
    return len(seen)


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any) -> None:
    write_new_bytes(path, json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n")


def write_sha256sums(root: pathlib.Path) -> None:
    rows = [row for row in inventory(root) if row["path"] != "SHA256SUMS"]
    write_new_bytes(root / "SHA256SUMS", "".join(f"{row['sha256']}  {row['path']}\n" for row in rows).encode())


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        require(not path.is_symlink(), f"symlink before seal: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def run(command: Sequence[str], *, timeout: int = 180) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=timeout)
    if result.returncode != 0:
        raise AuditError(f"command failed ({result.returncode}): {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-4000:]}")
    return result


def hash_region(path: pathlib.Path, offset: int, size: int) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        source.seek(offset)
        remaining = size
        while remaining:
            block = source.read(min(remaining, 1024 * 1024))
            require(bool(block), f"short ELF region: {path}")
            digest.update(block)
            remaining -= len(block)
    return digest.hexdigest()


def parse_addr2line(path: pathlib.Path, addresses: Sequence[int]) -> dict[int, list[dict[str, str]]]:
    lines = path.read_text(errors="strict").splitlines()
    result: dict[int, list[dict[str, str]]] = {}
    index = 0
    while index < len(lines):
        require(re.fullmatch(r"0x[0-9a-f]+", lines[index]) is not None, f"bad addr2line address row: {lines[index]!r}")
        address = int(lines[index], 16)
        index += 1
        frames = []
        while index < len(lines) and re.fullmatch(r"0x[0-9a-f]+", lines[index]) is None:
            require(index + 1 < len(lines), "truncated addr2line frame")
            frames.append({"function": lines[index], "location": lines[index + 1]})
            index += 2
        require(bool(frames) and address not in result, f"bad addr2line frame cardinality: {address:#x}")
        result[address] = frames
    require(list(result) == list(addresses), "addr2line address order or membership drift")
    return result


def unique_frames(addresses: Iterable[int], frames: Mapping[int, list[dict[str, str]]]) -> list[list[dict[str, str]]]:
    seen: set[bytes] = set()
    values: list[list[dict[str, str]]] = []
    for address in addresses:
        encoded = canonical_json_bytes(frames[address])
        if encoded not in seen:
            seen.add(encoded)
            values.append(frames[address])
    return values


def fresh_instruction_stream() -> tuple[list[dict[str, Any]], dict[str, bytes]]:
    pattern = re.compile(r"^\s*([0-9a-fA-F]+):\s+((?:[0-9a-fA-F]{2}\s)+)\s*(.*)$")
    instructions: list[dict[str, Any]] = []
    outputs: dict[str, bytes] = {}
    for key, (start, end) in SYMBOLS.items():
        argv = [
            "/usr/bin/objdump", "--disassemble", "--demangle", "--wide",
            f"--start-address={start:#x}", f"--stop-address={end:#x}", str(D2_ELF),
        ]
        output = run(argv).stdout
        outputs[f"AUDIT_OBJDUMP.{key}.txt"] = output
        cursor = start
        for line in output.decode(errors="replace").splitlines():
            match = pattern.match(line)
            if match is None:
                continue
            address = int(match.group(1), 16)
            if not start <= address < end:
                continue
            machine = bytes.fromhex(match.group(2))
            require(address == cursor, f"instruction gap or overlap in {key} at {cursor:#x}")
            instructions.append({"key": key, "address": address, "end_exclusive": address + len(machine), "machine_hex": machine.hex()})
            cursor += len(machine)
        require(cursor == end, f"instruction end drift in {key}: {cursor:#x}")
    addresses = [row["address"] for row in instructions]
    encoded = "".join(f"0x{address:x}\n" for address in addresses).encode()
    require(len(addresses) == INSTRUCTION_COUNT, "fresh instruction count drift")
    require(sha256_bytes(encoded) == ADDRESS_LIST_SHA256, "fresh address-list SHA drift")
    require(encoded == SEALED_ADDRESSES.read_bytes(), "fresh and sealed address lists differ")
    outputs["AUDIT_SEALED_ADDRESSES.txt"] = encoded
    return instructions, outputs


def verify_elf() -> dict[str, Any]:
    elf = require_file(D2_ELF, digest=ELF_SHA256, size=ELF_SIZE, mode="0555")
    header = run(["/usr/bin/readelf", "-hW", str(D2_ELF)]).stdout.decode(errors="replace")
    notes = run(["/usr/bin/readelf", "-nW", str(D2_ELF)]).stdout.decode(errors="replace")
    sections = run(["/usr/bin/readelf", "-SW", str(D2_ELF)]).stdout.decode(errors="replace")
    require(re.search(r"^\s*Type:\s+DYN\b", header, re.MULTILINE) is not None, "ELF type drift")
    build = re.search(r"Build ID:\s*([0-9a-f]+)", notes)
    require(build is not None and build.group(1) == BUILD_ID, "Build ID drift")
    text = re.search(r"^\s*\[\s*\d+\]\s+\.text\s+\S+\s+([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+", sections, re.MULTILINE)
    require(text is not None, ".text parse failed")
    address, offset, size = (int(text.group(index), 16) for index in (1, 2, 3))
    require((address, offset, size) == (TEXT_START, TEXT_OFFSET, TEXT_SIZE), ".text geometry drift")
    require(hash_region(D2_ELF, offset, size) == TEXT_SHA256, ".text SHA drift")
    return {"elf": elf, "elf_type": "ET_DYN", "build_id": BUILD_ID, "text": {"start": address, "offset": offset, "size_bytes": size, "end_exclusive": address + size, "sha256": TEXT_SHA256}}


def verify_map() -> tuple[dict[str, Any], dict[str, bytes]]:
    require_file(BUCKET_MAP, digest=MAP_SHA256, size=390_324, mode="0444")
    value = json.loads(BUCKET_MAP.read_text())
    require(value.get("schema") == "lay.v10.e1-traversal-d2-bucket-map.v1", "map schema drift")
    require(value.get("elf_sha256") == ELF_SHA256 and value.get("build_id") == BUILD_ID, "map ELF identity drift")
    require(value.get("address_space") == "ELF virtual address", "map address-space drift")
    require(value.get("join_key") == ["Build ID", "normalized ELF virtual IP"], "map join key drift")
    require(value.get("post_sample_reassignment_permitted") is False, "map permits post-sample reassignment")
    require(value.get("range_count") == 46, "map range count drift")

    instructions, outputs = fresh_instruction_stream()
    addresses = [row["address"] for row in instructions]
    frames = parse_addr2line(ADDR2LINE_OUTPUT, addresses)
    instruction_by_address = {row["address"]: row for row in instructions}
    mechanism_expected = list(EXPECTED_MECHANISM_RANGES)
    outside_expected = list(EXPECTED_OUTSIDE)
    mechanism_observed = []
    outside_observed = []
    bucket_bytes: Counter[str] = Counter()
    bucket_ranges: Counter[str] = Counter()
    sub_bucket_bytes: Counter[str] = Counter()
    cursor = TEXT_START
    ambiguous_bytes = 0

    for row in value.get("ranges", []):
        start = row.get("start")
        end = row.get("end_exclusive")
        require(isinstance(start, int) and isinstance(end, int) and start == cursor < end <= TEXT_END, f"map range topology drift at {cursor:#x}")
        require(row.get("length_bytes") == end - start, f"map range length drift at {start:#x}")
        require(row.get("elf_sha256") == ELF_SHA256 and row.get("build_id") == BUILD_ID, f"range ELF identity drift at {start:#x}")
        require(row.get("machine_bytes_sha256") == hash_region(D2_ELF, TEXT_OFFSET + start - TEXT_START, end - start), f"range byte SHA drift at {start:#x}")
        bucket = row.get("bucket")
        sub_bucket = row.get("sub_bucket")
        ambiguous = row.get("ambiguous")
        require(isinstance(bucket, str) and isinstance(sub_bucket, str) and isinstance(ambiguous, bool), f"range taxonomy type drift at {start:#x}")
        selected_addresses = [address for address in addresses if start <= address < end]
        if bucket == "OUTSIDE_TRAVERSAL":
            outside_observed.append((start, end))
            require(sub_bucket == "OUTSIDE_TRAVERSAL" and not ambiguous, "outside range taxonomy drift")
            require(row.get("instruction_count") == 0 and row.get("source_inlined_frames") == [], "outside range acquired traversal ownership")
        else:
            mechanism_observed.append((start, end, bucket, sub_bucket, ambiguous))
            require(start in instruction_by_address and (end in instruction_by_address or end in {stop for _key, (_begin, stop) in SYMBOLS.items()}), f"mechanism boundary not instruction aligned: {start:#x}..{end:#x}")
            require(row.get("instruction_count") == len(selected_addresses), f"range instruction count drift at {start:#x}")
            require(row.get("source_inlined_frames") == unique_frames(selected_addresses, frames), f"range frame ownership drift at {start:#x}")
            require(ambiguous == (bucket == "UNATTRIBUTED"), f"ambiguous taxonomy drift at {start:#x}")
        bucket_bytes[bucket] += end - start
        bucket_ranges[bucket] += 1
        sub_bucket_bytes[f"{bucket}/{sub_bucket}"] += end - start
        if ambiguous:
            ambiguous_bytes += end - start
        cursor = end

    require(cursor == TEXT_END, "map does not cover complete .text")
    require(mechanism_observed == mechanism_expected, "mechanism range classification drift")
    require(outside_observed == outside_expected, "outside complement drift")
    require(dict(sorted(bucket_bytes.items())) == EXPECTED_BUCKET_BYTES, "bucket byte totals drift")
    require(value.get("bucket_bytes") == dict(sorted(bucket_bytes.items())), "published bucket byte totals drift")
    require(value.get("bucket_range_counts") == dict(sorted(bucket_ranges.items())), "published bucket range totals drift")
    require(value.get("sub_bucket_bytes") == dict(sorted(sub_bucket_bytes.items())), "published sub-bucket totals drift")
    require(ambiguous_bytes == 139 and value.get("ambiguous_bytes") == 139, "ambiguous byte total drift")
    coverage = value.get("coverage") or {}
    require(coverage == {"text_bytes": TEXT_SIZE, "covered_bytes": TEXT_SIZE, "gap_bytes": 0, "overlap_count": 0, "range_outside_exec_pt_load": 0, "machine_byte_hash_mismatches": 0, "instruction_alignment_mismatches": 0}, "published coverage ledger drift")
    reserved = value.get("reserved_absent_sub_buckets") or {}
    require(reserved.get("REDUNDANT_STATE_DECODE") == {"status": "ABSENT", "range_count": 0, "length_bytes": 0}, "redundant state decode drift")

    symbol_decode = [address for address in addresses if SYMBOLS["edge"][0] <= address < SYMBOLS["edge"][1] and any(frame["location"].endswith("v13_typed_peak.rs:602") for frame in frames[address])]
    require(symbol_decode and symbol_decode[0] == 0x92653A and instruction_by_address[symbol_decode[-1]]["end_exclusive"] == 0x926551, "symbol-decode DWARF ownership drift")
    return {
        "map": file_identity(BUCKET_MAP),
        "range_count": len(value["ranges"]),
        "covered_bytes": sum(bucket_bytes.values()),
        "bucket_bytes": dict(sorted(bucket_bytes.items())),
        "bucket_range_counts": dict(sorted(bucket_ranges.items())),
        "sub_bucket_bytes": dict(sorted(sub_bucket_bytes.items())),
        "ambiguous_bytes": ambiguous_bytes,
        "instruction_count": len(addresses),
        "address_list_sha256": ADDRESS_LIST_SHA256,
        "addr2line_sha256": sha256_file(ADDR2LINE_OUTPUT),
        "machine_byte_hash_mismatches": 0,
        "gap_bytes": 0,
        "overlap_count": 0,
        "mechanism_classification_exact": True,
        "source_frame_ownership_exact": True,
    }, outputs


def verify_local_admission() -> dict[str, Any]:
    require(not AUDIT_RESULT.exists(), "bucket-map audit already exists")
    require_file(SSH_IDENTITY, mode="0600")
    require(mode_string(MAP_RESULT) == "0555", "map result mode drift")
    require_file(MAP_RESULT / "SHA256SUMS", digest=LOCAL_MANIFEST_SHA256, size=3_197, mode="0444")
    require_file(MAP_RESULT / "REMOTE_EVIDENCE/SHA256SUMS", digest=REMOTE_MANIFEST_SHA256, size=1_944, mode="0444")
    require(verify_sha256sums(MAP_RESULT) == 32, "local map manifest count drift")
    require(verify_sha256sums(MAP_RESULT / "REMOTE_EVIDENCE") == 22, "remote map manifest count drift")
    require_file(LOCAL_RECEIPT, digest=LOCAL_RECEIPT_SHA256, size=2_897, mode="0444")
    require_file(REMOTE_RECEIPT, digest=REMOTE_RECEIPT_SHA256, size=24_380, mode="0444")
    require_file(SEALED_ADDRESSES, digest=ADDRESS_LIST_SHA256, size=9_576, mode="0444")
    require_file(ADDR2LINE_OUTPUT, digest=ADDR2LINE_SHA256, size=697_799, mode="0444")
    require_file(ADDR2LINE_PRODUCER, digest="0c87c35844081a1117af380969c8d1ff2ca2fd9e81325f922cb9e7cbf6e35484", size=19_209, mode="0444")
    local = json.loads(LOCAL_RECEIPT.read_text())
    remote = json.loads(REMOTE_RECEIPT.read_text())
    require(local.get("verdict") == remote.get("verdict") == "D2_BUCKET_MAP_SEALED", "producer verdict drift")
    require(local.get("map_sha256") == remote.get("map", {}).get("sha256") == MAP_SHA256, "producer map SHA drift")
    require(local.get("parity_executed") is False and remote.get("parity_executed") is False, "parity crossed map boundary")
    return {"local_receipt": file_identity(LOCAL_RECEIPT), "remote_receipt": file_identity(REMOTE_RECEIPT), "local_manifest_entries": 32, "remote_manifest_entries": 22}


def remote_probe_source() -> str:
    return f'''import hashlib,json,os,pathlib,stat
TASK_ID={TASK_ID!r}
TRANSACTION_ID={TRANSACTION_ID!r}
PARENT=pathlib.Path({str(REMOTE_PARENT)!r})
RESULT=pathlib.Path({str(REMOTE_RESULT)!r})
FAILURE=pathlib.Path({str(REMOTE_FAILURE)!r})
STATE=pathlib.Path({str(REMOTE_STATE)!r})
ELF=PARENT/'build-v1/d2-test-elf'
def need(value,message):
    if not value: raise RuntimeError(message)
def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda:source.read(1024*1024),b''): h.update(block)
    return h.hexdigest()
def row(path):
    need(path.is_file() and not path.is_symlink(),f'invalid file: {{path}}')
    return {{'path':str(path),'mode':f'{{stat.S_IMODE(path.stat().st_mode):04o}}','size_bytes':path.stat().st_size,'sha256':sha(path)}}
def verify_manifest(root):
    lines=(root/'SHA256SUMS').read_text().splitlines(); seen=set()
    for line in lines:
        digest,separator,relative=line.partition('  ')
        need(separator=='  ' and len(digest)==64,'bad manifest row')
        path=pathlib.PurePosixPath(relative)
        need(not path.is_absolute() and '..' not in path.parts and relative not in seen,'unsafe manifest row')
        seen.add(relative); need(sha(root/path)==digest,f'manifest mismatch: {{relative}}')
    actual={{path.relative_to(root).as_posix() for path in root.rglob('*') if path.is_file() and path.name!='SHA256SUMS'}}
    need(seen==actual,'manifest membership drift')
    return len(seen)
markers=[]
for path in sorted((STATE/'markers').iterdir()):
    markers.append({{**row(path),'name':path.name,'value':json.loads(path.read_text())}})
result={{
 'hostname':os.uname().nodename,
 'machine_id_sha256':sha(pathlib.Path('/etc/machine-id')),
 'parent_entries':sorted(path.name for path in PARENT.iterdir()),
 'state_entries':sorted(path.name for path in STATE.iterdir()),
 'result_mode':f'{{stat.S_IMODE(RESULT.stat().st_mode):04o}}',
 'result_manifest':row(RESULT/'SHA256SUMS'),
 'result_manifest_entries':verify_manifest(RESULT),
 'map':row(RESULT/'D2_BUCKET_MAP.json'),
 'receipt':row(RESULT/'D2_BUCKET_MAP_RECEIPT.json'),
 'map_state':row(STATE/'BUCKET_MAP_STATE.json'),
 'map_state_value':json.loads((STATE/'BUCKET_MAP_STATE.json').read_text()),
 'failure_present':FAILURE.exists(),
 'elf':row(ELF),
 'markers':markers,
 'remote_writes':0,
}}
print(json.dumps(result,sort_keys=True,separators=(',',':')))
'''


def remote_projection() -> dict[str, Any]:
    source = remote_probe_source()
    compile(source, "<d2-bucket-map-audit-remote-probe>", "exec")
    command = ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", REMOTE, shlex.join(["/usr/bin/python3", "-c", source])]
    return json.loads(run(command, timeout=240).stdout)


def verify_remote_projection(value: Mapping[str, Any]) -> None:
    require(value.get("hostname") == REMOTE_HOSTNAME and value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote host identity drift")
    require(value.get("parent_entries") == ["bucket-map-v1", "build-v1", "d2a-v1"], "remote parent tree drift")
    require(value.get("state_entries") == ["BUCKET_MAP_STATE.json", "BUILD_STATE.json", "STATE.json", "markers", "route.lock"], "remote state tree drift")
    require(value.get("result_mode") == "0555" and value.get("result_manifest_entries") == 22, "remote result seal drift")
    require(value.get("result_manifest", {}).get("sha256") == REMOTE_MANIFEST_SHA256, "remote manifest SHA drift")
    require(value.get("map", {}).get("sha256") == MAP_SHA256 and value.get("map", {}).get("mode") == "0444", "remote map identity drift")
    require(value.get("receipt", {}).get("sha256") == REMOTE_RECEIPT_SHA256, "remote receipt identity drift")
    require(value.get("failure_present") is False and value.get("remote_writes") == 0, "remote failure or write drift")
    require(value.get("elf", {}).get("sha256") == ELF_SHA256 and value.get("elf", {}).get("size_bytes") == ELF_SIZE, "remote ELF drift")
    state = value.get("map_state_value") or {}
    require(state.get("state") == "D2_BUCKET_MAP_SEALED" and state.get("transaction_id") == TRANSACTION_ID, "remote map state drift")
    require(state.get("receipt_sha256") == REMOTE_RECEIPT_SHA256 and state.get("retry_permitted") is False, "remote map state receipt drift")
    observed = {row["name"]: row for row in value.get("markers", [])}
    require(set(observed) == set(MARKERS), f"remote marker set drift: {sorted(observed)}")
    for name, (digest, size) in MARKERS.items():
        row = observed[name]
        require(row.get("sha256") == digest and row.get("size_bytes") == size and row.get("mode") == "0400", f"marker identity drift: {name}")


def command_graph() -> dict[str, Any]:
    return {
        "external_actions": ["self-check", "audit"],
        "local_readers": ["readelf", "objdump x3", "sealed addr2line output", "direct ELF range hashing"],
        "remote_readers": ["sealed map manifest", "state projection", "marker ledger"],
        "remote_writes": [],
        "marker_mutations": [],
        "elf_execution": [],
        "perf_routes": [],
        "parity_routes": [],
    }


def self_check() -> dict[str, Any]:
    compile(AUDITOR.read_text(), str(AUDITOR), "exec")
    admission = verify_local_admission()
    graph = command_graph()
    require(graph["external_actions"] == ["self-check", "audit"], "auditor action graph drift")
    require(not graph["remote_writes"] and not graph["marker_mutations"], "auditor has a mutation route")
    require(not graph["elf_execution"] and not graph["perf_routes"] and not graph["parity_routes"], "auditor has a forbidden execution route")
    source = remote_probe_source()
    for forbidden in ("write_text", "write_bytes", "os.rename", "unlink(", "mkdir(", "subprocess", "perf record", "perf stat"):
        require(forbidden not in source, f"remote probe contains forbidden effect: {forbidden}")
    elf = verify_elf()
    map_audit, outputs = verify_map()
    remote = remote_projection()
    verify_remote_projection(remote)
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-bucket-map-auditor-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D2_BUCKET_MAP_AUDITOR_VERIFIED_UNRUN",
        "auditor": file_identity(AUDITOR),
        "admission": admission,
        "elf": elf,
        "map_audit": map_audit,
        "fresh_reader_outputs_sha256": {name: sha256_bytes(value) for name, value in outputs.items()},
        "remote_projection_sha256": sha256_bytes(canonical_json_bytes(remote)),
        "command_graph": graph,
        "remote_writes": 0,
        "marker_mutations": 0,
        "elf_executed": False,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "parity_executed": False,
    }


def audit() -> dict[str, Any]:
    check = self_check()
    before = remote_projection()
    verify_remote_projection(before)
    elf = verify_elf()
    map_audit, outputs = verify_map()
    after = remote_projection()
    verify_remote_projection(after)
    require(before == after, "remote state changed during bucket-map audit")

    stage = pathlib.Path(f"{AUDIT_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        shutil.copy2(AUDITOR, stage / "auditor.py")
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "REMOTE_BEFORE.json", before)
        write_new_json(stage / "REMOTE_AFTER.json", after)
        for name, value in outputs.items():
            write_new_bytes(stage / name, value)
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-bucket-map-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_BUCKET_MAP_AUDITED",
            "auditor": file_identity(stage / "auditor.py"),
            "producer_local_receipt_sha256": LOCAL_RECEIPT_SHA256,
            "producer_remote_receipt_sha256": REMOTE_RECEIPT_SHA256,
            "map": map_audit,
            "elf": elf,
            "live_remote_projection_stable": True,
            "map_marker_consumed": True,
            "parity_marker_available": True,
            "remaining_available_markers": 9,
            "total_consumed_markers": 2,
            "remote_writes": 0,
            "marker_mutations": 0,
            "elf_executed": False,
            "cargo_invocations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "d2_subject_executed": False,
            "parity_executed": False,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "runtime_authority_changed": False,
            "stage_path_metadata_in_producer_receipt": True,
            "stage_path_metadata_affects_map_identity": False,
            "next_action_admitted": "separate parity controller may verify this receipt and consume parity.available",
        }
        write_new_json(stage / "D2_BUCKET_MAP_AUDIT_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, AUDIT_RESULT)
        fsync_directory(AUDIT_RESULT.parent)
        return receipt
    except BaseException:
        if stage.exists():
            for path in [stage, *stage.rglob("*")]:
                path.chmod(0o700 if path.is_dir() else 0o600)
            shutil.rmtree(stage)
        raise


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "audit"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2 BUCKET MAP AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
