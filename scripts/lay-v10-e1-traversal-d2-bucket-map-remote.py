#!/usr/bin/env python3
"""Remote one-shot producer for the primary-only D2 machine bucket map."""

from __future__ import annotations

import base64
import fcntl
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import subprocess
import sys
import time
from collections import Counter
from typing import Any, Iterable, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
ELF = PARENT / "build-v1/d2-test-elf"
RESULT = PARENT / "bucket-map-v1"
FAILURE = PARENT / "bucket-map-failure-v1"
MAP_STATE = STATE / "BUCKET_MAP_STATE.json"
LOCK = STATE / "route.lock"

ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
ELF_SIZE = 317_706_232
BUILD_ID = "eb951f1a7526a9f1cb365040c10989aa5d3fc50f"
TEXT_START = 0x3F72C0
TEXT_OFFSET = 0x3F62C0
TEXT_SIZE = 15_980_919
TEXT_END = TEXT_START + TEXT_SIZE
TEXT_SHA256 = "f57eba60bc4b1cadbeb2dfc524af59a7ab011a2e64afb0e1a0fe610755129d94"
EXEC_LOAD_START = 0x3F72C0
EXEC_LOAD_END = 0x1334D70
INSTRUCTION_COUNT = 1_064
ADDRESS_LIST_SHA256 = "fca1804a3d0af34ae462938f970fd181706846da6e42dbb725c5ef1775581c58"
ADDR2LINE_OUTPUT_SIZE = 697_799
ADDR2LINE_OUTPUT_SHA256 = "8b9b4767557a3ea019bbaebb280d1a56ab2180f34ad1e05aed9c2affb4c8a9e6"
LOCAL_ELF_PATH = (
    "/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUILD_V1_2026-08-25/REMOTE_EVIDENCE/d2-test-elf"
)

BUILD_AUDIT_SHA256 = "4e19e2e806e2b3f04f8c0286a63f64631e7cf6ff143d9a90da4036654bc8339c"
READER_CORRECTION_V3_SHA256 = "af1761fe71a976939cfaeac5248a1f9cd5f474e85654ca1b5563baf94dcb9214"
BUCKET_MARKER_SHA256 = "4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7"
BUCKET_MARKER_SIZE = 483
BUILD_MARKER_SHA256 = "d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964"

EXPECTED_SYMBOLS = {
    "hot": (
        0x778320,
        0x108E,
        "lay::nanda_wave::l2_field::v13_typed_peak::tests::d1_enumerate_lane_prepared::<false>",
    ),
    "edge": (
        0x926520,
        0x123,
        "<lay::nanda_wave::l2_field::v13_typed_peak::V13DafsaView>::edge",
    ),
    "state": (
        0x9266B0,
        0x158,
        "<lay::nanda_wave::l2_field::v13_typed_peak::V13DafsaView>::state",
    ),
}
LOCAL_OBJDUMP_ARGVS = tuple(
    (
        "/usr/bin/objdump",
        "--disassemble",
        "--demangle",
        "--wide",
        f"--start-address={start:#x}",
        f"--stop-address={start + size:#x}",
        LOCAL_ELF_PATH,
    )
    for start, size, _name in EXPECTED_SYMBOLS.values()
)
LOCAL_ADDR2LINE_PREFIX = (
    "/usr/bin/addr2line",
    f"--exe={LOCAL_ELF_PATH}",
    "--functions",
    "--inlines",
    "--demangle",
    "--addresses",
)

# These intervals are sealed-D2 machine-code facts, not source-line guesses.
# They partition the complete hot symbol and are frozen before any D2 sample.
HOT_RULES = (
    (0x778320, 0x7783DA, "STACK_CONTROL", "BUDGET_DEADLINE", "function prologue and input-budget exits", False),
    (0x7783DA, 0x7784E3, "STACK_CONTROL", "STACK_PUSH", "initial packed-node stack construction", False),
    (0x7784E3, 0x7785BE, "STACK_CONTROL", "SCRATCH_BOOKKEEPING", "initial vectors and scratch baseline", False),
    (0x7785BE, 0x7785DA, "STACK_CONTROL", "STACK_POP", "packed-node pop path", False),
    (0x7785DA, 0x7786AF, "STACK_CONTROL", "BUDGET_DEADLINE", "expanded-state and elapsed-budget checks", False),
    (0x7786AF, 0x7786F0, "DAFSA_DECODE_MEMORY", "STATE_DECODE", "state decoder call and result branch", False),
    (0x7786F0, 0x7786FB, "TERMINAL", "TERMINAL_PREDICATE", "packed-state terminal flag test", False),
    (0x7786FB, 0x778725, "DAFSA_DECODE_MEMORY", "EDGE_RANGE_CONTROL", "first-edge range and bounds control", False),
    (0x778725, 0x778A7C, "TRANSITION", "FUSED_SCALAR_U64_ADVANCE", "compiler-hoisted fused-transition precompute and state pack", False),
    (0x778A7C, 0x778AA9, "STACK_CONTROL", "STACK_PUSH", "packed child write and stack length update", False),
    (0x778AA9, 0x778AC2, "STACK_CONTROL", "PRUNE_AND_LOOP", "edge-loop increment and backedge", False),
    (0x778AC2, 0x778B0E, "DAFSA_DECODE_MEMORY", "EDGE_DECODE", "edge decoder call and result extraction", False),
    (0x778B0E, 0x778B18, "RANK", "EDGE_RANK_ADD", "checked rank-delta addition", False),
    (0x778B18, 0x778B8F, "TRANSITION", "ALPHABET_ID", "dense alphabet mapping and invalid-symbol branch", False),
    (0x778B8F, 0x778C2E, "TRANSITION", "EQUALITY_WINDOW", "equality-mask selection and seven-bit extraction", False),
    (0x778C2E, 0x778DB7, "TRANSITION", "FUSED_SCALAR_U64_ADVANCE", "fused scalar cell recurrence and minimum", False),
    (0x778DB7, 0x778DC4, "STACK_CONTROL", "PRUNE_AND_LOOP", "minimum-versus-radius prune branch", False),
    (0x778DC4, 0x778E09, "STACK_CONTROL", "STACK_PUSH", "survivor materialization and vector growth path", False),
    (0x778E09, 0x778F26, "TRANSITION", "FUSED_SCALAR_U64_ADVANCE", "remaining fused scalar cells and recurrence joins", False),
    (0x778F26, 0x778F30, "UNATTRIBUTED", "UNATTRIBUTED", "alignment bytes with no unique mechanism owner", True),
    (0x778F30, 0x778F6F, "TERMINAL", "TERMINAL_DISTANCE", "packed terminal-distance extraction", False),
    (0x778F6F, 0x778F7C, "TERMINAL", "TERMINAL_PREDICATE", "terminal radius acceptance branch", False),
    (0x778F7C, 0x778FA5, "TERMINAL", "FORM_REF_COLLECTION", "accepted form-reference vector push", False),
    (0x778FA5, 0x778FF9, "STACK_CONTROL", "BUDGET_DEADLINE", "terminal-count budget and return construction", False),
    (0x778FF9, 0x779055, "STACK_CONTROL", "PRUNE_AND_LOOP", "normal traversal completion and cleanup", False),
    (0x779055, 0x77909A, "STACK_CONTROL", "BUDGET_DEADLINE", "product-state budget return path", False),
    (0x77909A, 0x7790C2, "DAFSA_DECODE_MEMORY", "STATE_DECODE", "state decoder residual propagation", False),
    (0x7790C2, 0x779120, "DAFSA_DECODE_MEMORY", "EDGE_RANGE_CONTROL", "edge-range bounds error construction", False),
    (0x779120, 0x779160, "STACK_CONTROL", "BUDGET_DEADLINE", "elapsed-deadline return path", False),
    (0x779160, 0x7791BB, "STACK_CONTROL", "SCRATCH_BOOKKEEPING", "scratch-budget return and cleanup", False),
    (0x7791BB, 0x7791EA, "UNATTRIBUTED", "UNATTRIBUTED", "cold bounds and allocation failure block", True),
    (0x7791EA, 0x779245, "DAFSA_DECODE_MEMORY", "EDGE_DECODE", "edge decoder alternate and residual path", False),
    (0x779245, 0x77924F, "RANK", "EDGE_RANK_ADD", "alternate checked rank-delta addition", False),
    (0x77924F, 0x7792C3, "TRANSITION", "ALPHABET_ID", "invalid alphabet formatting and residual propagation", False),
    (0x7792C3, 0x7792FF, "STACK_CONTROL", "PRUNE_AND_LOOP", "result-vector cleanup", False),
    (0x7792FF, 0x779344, "RANK", "EDGE_RANK_ADD", "rank-overflow error construction", False),
    (0x779344, 0x77935C, "TRANSITION", "ALPHABET_ID", "cold alphabet expect failure", False),
    (0x77935C, 0x7793AE, "UNATTRIBUTED", "UNATTRIBUTED", "unwind-only cleanup without unique mechanism owner", True),
)

READ_ELF_ARGV = (
    "/usr/bin/readelf",
    "--file-header",
    "--program-headers",
    "--sections",
    "--symbols",
    "--notes",
    "--debug-dump=decodedline",
    str(ELF),
)
OBJDUMP_ARGVS = tuple(
    (
        "/usr/bin/objdump",
        "--disassemble",
        "--demangle",
        "--wide",
        f"--start-address={start:#x}",
        f"--stop-address={start + size:#x}",
        str(ELF),
    )
    for start, size, _name in EXPECTED_SYMBOLS.values()
)
NM_ARGV = (
    "/usr/bin/nm",
    "--numeric-sort",
    "--print-size",
    "--demangle",
    str(ELF),
)


class MapError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise MapError(message)


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


def row(path: pathlib.Path) -> dict[str, Any]:
    need(path.is_file(), f"missing file: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            need(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new_bytes(path, json.dumps(value, sort_keys=True, indent=2).encode() + b"\n", mode)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        need(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            values.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return values


def write_sha256sums(root: pathlib.Path) -> None:
    values = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{value['sha256']}  {value['path']}\n" for value in values).encode(),
    )


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        need(not path.is_symlink(), f"symlink before seal: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def run_capture(argv: Sequence[str]) -> bytes:
    result = subprocess.run(list(argv), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode != 0:
        raise MapError(f"command failed ({result.returncode}): {argv!r}: {result.stderr[-4000:]!r}")
    return result.stdout


def run_to_files(argv: Sequence[str], stdout_path: pathlib.Path, stderr_path: pathlib.Path) -> None:
    with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
        result = subprocess.run(list(argv), stdout=stdout, stderr=stderr, check=False)
        stdout.flush()
        stderr.flush()
        os.fsync(stdout.fileno())
        os.fsync(stderr.fileno())
    need(result.returncode == 0, f"command failed ({result.returncode}): {argv!r}")


def file_region_sha256(path: pathlib.Path, offset: int, size: int) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        source.seek(offset)
        remaining = size
        while remaining:
            block = source.read(min(remaining, 1024 * 1024))
            need(bool(block), f"short region read: {path}")
            digest.update(block)
            remaining -= len(block)
    return digest.hexdigest()


def marker_projection() -> list[dict[str, Any]]:
    values = []
    for path in sorted((STATE / "markers").iterdir()):
        need(path.is_file() and not path.is_symlink(), f"invalid marker: {path}")
        values.append({**row(path), "name": path.name, "value": json.loads(path.read_text())})
    return values


def verify_marker(path: pathlib.Path, digest: str, size: int, route_id: str) -> dict[str, Any]:
    value = row(path)
    need(value["sha256"] == digest and value["size_bytes"] == size, f"marker identity drift: {path.name}")
    need(value["mode"] == "0400", f"marker mode drift: {path.name}")
    body = json.loads(path.read_text())
    need(body.get("task_id") == TASK_ID, f"marker task drift: {path.name}")
    need(body.get("transaction_id") == TRANSACTION_ID, f"marker transaction drift: {path.name}")
    need(body.get("route_id") == route_id and body.get("state") == "available", f"marker route drift: {path.name}")
    need(body.get("retry_permitted") is False, f"marker retry drift: {path.name}")
    return {**value, "value": body}


def verify_admission(payload: Mapping[str, Any]) -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(sha256_file(pathlib.Path("/etc/machine-id")) == MACHINE_ID_SHA256, "machine identity drift")
    need(payload.get("build_audit_sha256") == BUILD_AUDIT_SHA256, "build-audit payload identity drift")
    audit_bytes = base64.b64decode(payload.get("build_audit_receipt_b64", ""), validate=True)
    need(sha256_bytes(audit_bytes) == BUILD_AUDIT_SHA256, "build-audit receipt SHA drift")
    audit = json.loads(audit_bytes)
    need(audit.get("verdict") == "D2_BUILD_AUDITED", "build-audit verdict drift")
    need(audit.get("d2", {}).get("elf", {}).get("sha256") == ELF_SHA256, "build-audit ELF drift")
    need(audit.get("d2", {}).get("build_id") == BUILD_ID, "build-audit Build ID drift")
    need(audit.get("bucket_map_marker_available") is True, "build-audit map admission drift")
    correction_bytes = base64.b64decode(payload.get("reader_correction_v3_b64", ""), validate=True)
    need(payload.get("reader_correction_v3_sha256") == READER_CORRECTION_V3_SHA256, "reader-correction declared SHA drift")
    need(sha256_bytes(correction_bytes) == READER_CORRECTION_V3_SHA256, "reader-correction byte SHA drift")
    local_addr2line = decode_local_addr2line_payload(payload)

    need(sorted(path.name for path in PARENT.iterdir()) == ["build-v1", "d2a-v1"], "task parent drift")
    need(sorted(path.name for path in STATE.iterdir()) == ["BUILD_STATE.json", "STATE.json", "markers", "route.lock"], "state tree drift")
    need(not RESULT.exists() and not FAILURE.exists() and not MAP_STATE.exists(), "map result already exists")
    need(row(ELF)["sha256"] == ELF_SHA256 and ELF.stat().st_size == ELF_SIZE, "D2 ELF drift")
    need(mode_string(ELF) == "0555", "D2 ELF mode drift")
    need(file_region_sha256(ELF, TEXT_OFFSET, TEXT_SIZE) == TEXT_SHA256, "D2 .text drift")

    build_state = json.loads((STATE / "BUILD_STATE.json").read_text())
    need(build_state.get("state") == "D2_BUILD_CREATED_UNAUDITED", "build state drift")
    need(build_state.get("transaction_id") == TRANSACTION_ID, "build transaction drift")
    bucket = verify_marker(STATE / "markers/bucket-map.available", BUCKET_MARKER_SHA256, BUCKET_MARKER_SIZE, "BUCKET-MAP")
    build = verify_marker(STATE / "markers/build.consumed-before-exec", BUILD_MARKER_SHA256, 478, "BUILD")
    markers = marker_projection()
    need(len(markers) == 11, "marker count drift")
    need(sum(value["name"].endswith(".available") for value in markers) == 10, "available marker count drift")
    need(sum("consumed" in value["name"] for value in markers) == 1, "consumed marker count drift")

    usage = shutil.disk_usage(PARENT)
    need(usage.free >= 2_000_000_000, "insufficient free disk for raw map evidence")
    return {
        "hostname": os.uname().nodename,
        "machine_id_sha256": MACHINE_ID_SHA256,
        "build_state": build_state,
        "elf": row(ELF),
        "text_sha256": TEXT_SHA256,
        "bucket_marker": bucket,
        "build_marker": build,
        "local_addr2line": local_addr2line["summary"],
        "markers": markers,
        "free_bytes": usage.free,
        "parent_entries": ["build-v1", "d2a-v1"],
        "state_entries": ["BUILD_STATE.json", "STATE.json", "markers", "route.lock"],
        "map_artifacts_present": False,
        "remote_writes": 0,
    }


def parse_symbols(nm_path: pathlib.Path) -> dict[str, dict[str, Any]]:
    found: dict[str, dict[str, Any]] = {}
    pattern = re.compile(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\w\s+(.+)$")
    for line in nm_path.read_text(errors="replace").splitlines():
        match = pattern.match(line)
        if not match:
            continue
        start = int(match.group(1), 16)
        size = int(match.group(2), 16)
        name = match.group(3)
        for key, expected in EXPECTED_SYMBOLS.items():
            if name == expected[2]:
                need(key not in found, f"duplicate target symbol: {key}")
                need(start == expected[0] and size == expected[1], f"target symbol drift: {key}")
                found[key] = {"key": key, "start": start, "size": size, "end_exclusive": start + size, "name": name}
    need(set(found) == set(EXPECTED_SYMBOLS), f"missing target symbols: {sorted(set(EXPECTED_SYMBOLS) - set(found))}")
    return found


def parse_target_instructions(
    objdump_paths: Sequence[pathlib.Path],
    symbols: Mapping[str, Mapping[str, Any]],
) -> dict[str, list[dict[str, Any]]]:
    values = {key: [] for key in symbols}
    pattern = re.compile(r"^\s*([0-9a-fA-F]+):\s+((?:[0-9a-fA-F]{2}\s)+)\s*(.*)$")
    for objdump_path in objdump_paths:
        for line in objdump_path.read_text(errors="replace").splitlines():
            match = pattern.match(line)
            if not match:
                continue
            address = int(match.group(1), 16)
            for key, symbol in symbols.items():
                if symbol["start"] <= address < symbol["end_exclusive"]:
                    machine = bytes.fromhex(match.group(2))
                    values[key].append(
                        {
                            "address": address,
                            "end_exclusive": address + len(machine),
                            "size_bytes": len(machine),
                            "machine_hex": machine.hex(),
                            "assembly": match.group(3),
                        }
                    )
                    break
    for key, instructions in values.items():
        need(bool(instructions), f"no instructions for target symbol: {key}")
        need(instructions[0]["address"] == symbols[key]["start"], f"instruction start drift: {key}")
        cursor = symbols[key]["start"]
        for instruction in instructions:
            need(instruction["address"] == cursor, f"instruction gap or overlap in {key} at {cursor:#x}")
            cursor = instruction["end_exclusive"]
        need(cursor == symbols[key]["end_exclusive"], f"instruction end drift: {key}: {cursor:#x}")
    return values


def parse_addr2line_text(text: str, addresses: Sequence[int]) -> dict[int, list[dict[str, str]]]:
    lines = text.splitlines()
    result: dict[int, list[dict[str, str]]] = {}
    index = 0
    while index < len(lines):
        need(re.fullmatch(r"0x[0-9a-f]+", lines[index]) is not None, f"bad addr2line address row: {lines[index]!r}")
        address = int(lines[index], 16)
        index += 1
        frames = []
        while index < len(lines) and re.fullmatch(r"0x[0-9a-f]+", lines[index]) is None:
            need(index + 1 < len(lines), "truncated addr2line frame")
            frames.append({"function": lines[index], "location": lines[index + 1]})
            index += 2
        need(bool(frames), f"addr2line returned no frame for {address:#x}")
        need(address not in result, f"duplicate addr2line address: {address:#x}")
        result[address] = frames
    need(list(result) == list(addresses), "addr2line address order or membership drift")
    return result


def parse_addr2line(path: pathlib.Path, addresses: Sequence[int]) -> dict[int, list[dict[str, str]]]:
    return parse_addr2line_text(path.read_text(errors="strict"), addresses)


def decode_local_addr2line_payload(
    payload: Mapping[str, Any],
    *,
    expected_addresses: Sequence[int] | None = None,
) -> dict[str, Any]:
    evidence = payload.get("local_addr2line")
    need(isinstance(evidence, Mapping), "missing local addr2line evidence")
    need(evidence.get("schema") == "lay.v10.e1-traversal-d2-local-addr2line-evidence.v1", "local addr2line schema drift")
    elf = evidence.get("elf")
    need(isinstance(elf, Mapping), "missing local addr2line ELF identity")
    need(elf.get("path") == LOCAL_ELF_PATH, "local addr2line ELF path drift")
    need(elf.get("sha256") == ELF_SHA256, "local addr2line ELF SHA drift")
    need(elf.get("size_bytes") == ELF_SIZE and elf.get("mode") == "0555", "local addr2line ELF size or mode drift")

    address_bytes = base64.b64decode(evidence.get("address_list_b64", ""), validate=True)
    need(sha256_bytes(address_bytes) == ADDRESS_LIST_SHA256, "local address-list byte SHA drift")
    need(evidence.get("address_list_sha256") == ADDRESS_LIST_SHA256, "local address-list declared SHA drift")
    address_lines = address_bytes.decode(errors="strict").splitlines()
    need(all(re.fullmatch(r"0x[0-9a-f]+", line) is not None for line in address_lines), "bad local address-list row")
    addresses = [int(line, 16) for line in address_lines]
    need(len(addresses) == INSTRUCTION_COUNT, "local instruction count drift")
    need(evidence.get("instruction_count") == INSTRUCTION_COUNT, "local declared instruction count drift")
    need(address_bytes == "".join(f"0x{address:x}\n" for address in addresses).encode(), "local address-list encoding drift")
    need(addresses == sorted(addresses) and len(addresses) == len(set(addresses)), "local address-list ordering drift")
    if expected_addresses is not None:
        need(addresses == list(expected_addresses), "remote/local instruction address mismatch")

    need(evidence.get("objdump_tool_version") == "GNU objdump (GNU Binutils for Ubuntu) 2.46", "local objdump version drift")
    need(evidence.get("objdump_argvs") == [list(argv) for argv in LOCAL_OBJDUMP_ARGVS], "local objdump argv drift")
    need(evidence.get("addr2line_tool_version") == "GNU addr2line (GNU Binutils for Ubuntu) 2.46", "local addr2line version drift")
    need(evidence.get("addr2line_argv") == [*LOCAL_ADDR2LINE_PREFIX, *address_lines], "local addr2line argv drift")

    stdout = base64.b64decode(evidence.get("addr2line_stdout_b64", ""), validate=True)
    stderr = base64.b64decode(evidence.get("addr2line_stderr_b64", ""), validate=True)
    need(len(stdout) == ADDR2LINE_OUTPUT_SIZE, "local addr2line output size drift")
    need(evidence.get("addr2line_stdout_size_bytes") == ADDR2LINE_OUTPUT_SIZE, "local declared addr2line size drift")
    need(sha256_bytes(stdout) == ADDR2LINE_OUTPUT_SHA256, "local addr2line output byte SHA drift")
    need(evidence.get("addr2line_stdout_sha256") == ADDR2LINE_OUTPUT_SHA256, "local declared addr2line output SHA drift")
    need(stderr == b"", "local addr2line stderr is not empty")
    need(evidence.get("addr2line_stderr_sha256") == sha256_bytes(b""), "local addr2line stderr SHA drift")
    frames = parse_addr2line_text(stdout.decode(errors="strict"), addresses)
    return {
        "addresses": addresses,
        "address_bytes": address_bytes,
        "stdout": stdout,
        "stderr": stderr,
        "frames": frames,
        "summary": {
            "elf_sha256": ELF_SHA256,
            "instruction_count": INSTRUCTION_COUNT,
            "address_list_sha256": ADDRESS_LIST_SHA256,
            "objdump_tool_version": evidence["objdump_tool_version"],
            "objdump_argvs": evidence["objdump_argvs"],
            "addr2line_tool_version": evidence["addr2line_tool_version"],
            "addr2line_argv": evidence["addr2line_argv"],
            "addr2line_stdout_size_bytes": ADDR2LINE_OUTPUT_SIZE,
            "addr2line_stdout_sha256": ADDR2LINE_OUTPUT_SHA256,
            "addr2line_stderr_sha256": sha256_bytes(b""),
        },
    }


def parse_readelf(path: pathlib.Path) -> dict[str, Any]:
    text = path.read_text(errors="replace")
    need(re.search(r"^\s*Type:\s+DYN\b", text, re.MULTILINE) is not None, "ELF type drift")
    build = re.search(r"Build ID:\s*([0-9a-f]+)", text)
    need(build is not None and build.group(1) == BUILD_ID, "Build ID drift")
    section = re.search(
        r"^\s*\[\s*16\]\s+\.text\s+PROGBITS\s+([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\n"
        r"\s*([0-9a-fA-F]+)\s+",
        text,
        re.MULTILINE,
    )
    need(section is not None, ".text section parse failed")
    observed = tuple(int(section.group(index), 16) for index in (1, 2, 3))
    need(observed == (TEXT_START, TEXT_OFFSET, TEXT_SIZE), f".text geometry drift: {observed!r}")
    load = re.search(
        r"^\s*LOAD\s+0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)\s+0x[0-9a-fA-F]+\n"
        r"\s*0x([0-9a-fA-F]+)\s+0x([0-9a-fA-F]+)\s+R E\s+0x([0-9a-fA-F]+)",
        text,
        re.MULTILINE,
    )
    need(load is not None, "executable PT_LOAD parse failed")
    offset, vaddr, filesz, memsz, align = (int(load.group(index), 16) for index in range(1, 6))
    need(vaddr == EXEC_LOAD_START and vaddr + filesz == EXEC_LOAD_END, "executable PT_LOAD drift")
    need(vaddr <= TEXT_START and TEXT_END <= vaddr + filesz, ".text outside executable PT_LOAD")
    return {
        "elf_type": "ET_DYN",
        "pie": True,
        "build_id": BUILD_ID,
        "text": {"start": TEXT_START, "offset": TEXT_OFFSET, "size_bytes": TEXT_SIZE, "end_exclusive": TEXT_END},
        "executable_pt_load": {
            "offset": offset,
            "vaddr": vaddr,
            "filesz": filesz,
            "memsz": memsz,
            "align": align,
            "end_exclusive": vaddr + filesz,
        },
    }


def rule_for_hot(address: int) -> tuple[int, int, str, str, str, bool]:
    matches = [rule for rule in HOT_RULES if rule[0] <= address < rule[1]]
    need(len(matches) == 1, f"hot classification cardinality at {address:#x}: {len(matches)}")
    return matches[0]


def unique_frames(instructions: Iterable[Mapping[str, Any]]) -> list[list[dict[str, str]]]:
    seen: set[bytes] = set()
    values: list[list[dict[str, str]]] = []
    for instruction in instructions:
        frames = instruction.get("frames", [])
        encoded = canonical_json_bytes(frames)
        if encoded not in seen:
            seen.add(encoded)
            values.append(frames)
    return values


def machine_sha(start: int, end: int) -> str:
    need(TEXT_START <= start < end <= TEXT_END, f"range outside .text: {start:#x}..{end:#x}")
    return file_region_sha256(ELF, TEXT_OFFSET + start - TEXT_START, end - start)


def make_range(
    start: int,
    end: int,
    symbol: str,
    bucket: str,
    sub_bucket: str,
    reason: str,
    ambiguous: bool,
    instructions: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    return {
        "elf_sha256": ELF_SHA256,
        "build_id": BUILD_ID,
        "start": start,
        "end_exclusive": end,
        "length_bytes": end - start,
        "symbol": symbol,
        "clone": None,
        "source_inlined_frames": unique_frames(instructions),
        "instruction_count": len(instructions),
        "bucket": bucket,
        "sub_bucket": sub_bucket,
        "classification_reason": reason,
        "machine_bytes_sha256": machine_sha(start, end),
        "ambiguous": ambiguous,
    }


def build_ranges(
    symbols: Mapping[str, Mapping[str, Any]],
    instructions: Mapping[str, Sequence[Mapping[str, Any]]],
    frames: Mapping[int, list[dict[str, str]]],
) -> list[dict[str, Any]]:
    enriched: dict[str, list[dict[str, Any]]] = {}
    for key, values in instructions.items():
        enriched[key] = [{**value, "frames": frames[value["address"]]} for value in values]

    hot_rules = list(HOT_RULES)
    need(hot_rules[0][0] == symbols["hot"]["start"] and hot_rules[-1][1] == symbols["hot"]["end_exclusive"], "hot rules boundary drift")
    for previous, current in zip(hot_rules, hot_rules[1:]):
        need(previous[1] == current[0], f"hot rule gap or overlap: {previous[1]:#x} {current[0]:#x}")
    hot_addresses = {value["address"] for value in enriched["hot"]}
    for start, end, *_ in hot_rules:
        need(start in hot_addresses and (end in hot_addresses or end == symbols["hot"]["end_exclusive"]), f"hot rule not instruction aligned: {start:#x}..{end:#x}")

    mechanism: list[dict[str, Any]] = []
    for start, end, bucket, sub_bucket, reason, ambiguous in hot_rules:
        selected = [value for value in enriched["hot"] if start <= value["address"] < end]
        need(sum(value["size_bytes"] for value in selected) == end - start, f"hot rule byte closure failed: {start:#x}")
        mechanism.append(make_range(start, end, symbols["hot"]["name"], bucket, sub_bucket, reason, ambiguous, selected))

    edge_groups: list[tuple[int, int, str, str, str, bool, list[dict[str, Any]]]] = []
    current: list[dict[str, Any]] = []
    current_class: tuple[str, str, str, bool] | None = None
    for value in enriched["edge"]:
        symbol_decode = any(frame["location"].endswith("v13_typed_peak.rs:602") for frame in value["frames"])
        classification = (
            "DAFSA_DECODE_MEMORY",
            "SYMBOL_DECODE" if symbol_decode else "EDGE_DECODE",
            "edge symbol field load" if symbol_decode else "exclusive packed-edge decoder",
            False,
        )
        if current and classification != current_class:
            edge_groups.append((current[0]["address"], current[-1]["end_exclusive"], *current_class, current))
            current = []
        current.append(value)
        current_class = classification
    if current:
        need(current_class is not None, "missing edge classification")
        edge_groups.append((current[0]["address"], current[-1]["end_exclusive"], *current_class, current))
    for start, end, bucket, sub_bucket, reason, ambiguous, selected in edge_groups:
        mechanism.append(make_range(start, end, symbols["edge"]["name"], bucket, sub_bucket, reason, ambiguous, selected))

    mechanism.append(
        make_range(
            symbols["state"]["start"],
            symbols["state"]["end_exclusive"],
            symbols["state"]["name"],
            "DAFSA_DECODE_MEMORY",
            "STATE_DECODE",
            "exclusive packed-state decoder including cold residual paths",
            False,
            enriched["state"],
        )
    )
    mechanism.sort(key=lambda value: value["start"])
    for left, right in zip(mechanism, mechanism[1:]):
        need(left["end_exclusive"] <= right["start"], "mechanism range overlap")

    ranges: list[dict[str, Any]] = []
    cursor = TEXT_START
    for value in mechanism:
        if cursor < value["start"]:
            ranges.append(
                make_range(
                    cursor,
                    value["start"],
                    "<D2 .text complement>",
                    "OUTSIDE_TRAVERSAL",
                    "OUTSIDE_TRAVERSAL",
                    "full .text complement outside the sealed traversal closure",
                    False,
                    [],
                )
            )
        ranges.append(value)
        cursor = value["end_exclusive"]
    if cursor < TEXT_END:
        ranges.append(
            make_range(
                cursor,
                TEXT_END,
                "<D2 .text complement>",
                "OUTSIDE_TRAVERSAL",
                "OUTSIDE_TRAVERSAL",
                "full .text complement outside the sealed traversal closure",
                False,
                [],
            )
        )

    need(ranges[0]["start"] == TEXT_START and ranges[-1]["end_exclusive"] == TEXT_END, "map text boundary mismatch")
    cursor = TEXT_START
    for value in ranges:
        need(value["start"] == cursor, f"map gap or overlap at {cursor:#x}")
        need(value["machine_bytes_sha256"] == machine_sha(value["start"], value["end_exclusive"]), "machine-byte hash mismatch")
        cursor = value["end_exclusive"]
    need(cursor == TEXT_END, "map coverage end mismatch")
    need(sum(value["length_bytes"] for value in ranges) == TEXT_SIZE, "map byte denominator mismatch")
    return ranges


def consume_marker() -> dict[str, Any]:
    available = STATE / "markers/bucket-map.available"
    consumed = STATE / "markers/bucket-map.consumed-before-exec"
    before = verify_marker(available, BUCKET_MARKER_SHA256, BUCKET_MARKER_SIZE, "BUCKET-MAP")
    need(not consumed.exists(), "bucket-map consumed marker already exists")
    os.rename(available, consumed)
    fsync_directory(available.parent)
    after = row(consumed)
    need(after["sha256"] == before["sha256"] and after["size_bytes"] == before["size_bytes"], "marker rename identity drift")
    return {"before": before, "after": after, "consumed_before_map_commands": True}


def publish_state(verdict: str, receipt_sha256: str | None) -> None:
    write_new_json(
        MAP_STATE,
        {
            "schema": "lay.v10.e1-traversal-d2-primary-only-bucket-map-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "state": verdict,
            "bucket_map_marker_consumed": True,
            "parity_marker_consumed": False,
            "receipt_sha256": receipt_sha256,
            "retry_permitted": False,
        },
        0o400,
    )
    fsync_directory(STATE)


def map_once(payload: Mapping[str, Any]) -> dict[str, Any]:
    admission = verify_admission(payload)
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    marker_consumed = False
    try:
        inputs = stage / "inputs"
        raw = stage / "raw"
        inputs.mkdir(mode=0o700)
        raw.mkdir(mode=0o700)
        local_controller = base64.b64decode(payload.get("local_controller_b64", ""), validate=True)
        remote_controller = base64.b64decode(payload.get("remote_controller_b64", ""), validate=True)
        need(sha256_bytes(local_controller) == payload.get("local_controller_sha256"), "local controller payload drift")
        need(sha256_bytes(remote_controller) == payload.get("remote_controller_sha256"), "remote controller payload drift")
        write_new_bytes(inputs / "local-controller.py", local_controller)
        write_new_bytes(inputs / "remote-controller.py", remote_controller)
        audit_bytes = base64.b64decode(payload["build_audit_receipt_b64"], validate=True)
        write_new_bytes(inputs / "D2_BUILD_AUDIT_RECEIPT.json", audit_bytes)
        correction_bytes = base64.b64decode(payload["reader_correction_v3_b64"], validate=True)
        need(sha256_bytes(correction_bytes) == READER_CORRECTION_V3_SHA256, "reader correction changed after admission")
        write_new_bytes(inputs / "READER_SCOPE_CORRECTION_V3.md", correction_bytes)
        premap = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-premap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "elf_sha256": ELF_SHA256,
            "build_id": BUILD_ID,
            "text_sha256": TEXT_SHA256,
            "text_start": TEXT_START,
            "text_end_exclusive": TEXT_END,
            "hot_rules": HOT_RULES,
            "commands": {
                "readelf": READ_ELF_ARGV,
                "objdump": OBJDUMP_ARGVS,
                "nm": NM_ARGV,
                "addr2line": admission["local_addr2line"],
            },
            "addr2line_execution_scope": "local byte-identical sealed ELF; hash-verified remote ingestion",
            "marker_consumed": False,
            "retry_permitted": False,
        }
        write_new_json(stage / "PREMAP.json", premap)
        fsync_directory(stage)

        # Complete every fallible read-only tool invocation and parser before
        # consuming the one-shot authority. Range construction/publication is
        # still forbidden until the marker has been atomically renamed.
        run_to_files(READ_ELF_ARGV, raw / "READELF.txt", raw / "READELF.stderr")
        objdump_paths = []
        for key, objdump_argv in zip(EXPECTED_SYMBOLS, OBJDUMP_ARGVS, strict=True):
            output = raw / f"OBJDUMP.{key}.txt"
            run_to_files(objdump_argv, output, raw / f"OBJDUMP.{key}.stderr")
            objdump_paths.append(output)
        run_to_files(NM_ARGV, raw / "NM.txt", raw / "NM.stderr")
        elf_metadata = parse_readelf(raw / "READELF.txt")
        symbols = parse_symbols(raw / "NM.txt")
        instructions = parse_target_instructions(objdump_paths, symbols)
        addresses = [value["address"] for key in ("hot", "edge", "state") for value in instructions[key]]
        address_bytes = "".join(f"0x{address:x}\n" for address in addresses).encode()
        need(len(addresses) == INSTRUCTION_COUNT, "remote instruction count drift")
        need(sha256_bytes(address_bytes) == ADDRESS_LIST_SHA256, "remote address-list SHA drift")
        write_new_bytes(stage / "SEALED_ADDRESSES.txt", address_bytes)
        local_addr2line = decode_local_addr2line_payload(payload, expected_addresses=addresses)
        need(local_addr2line["address_bytes"] == address_bytes, "remote/local address-list bytes differ")
        write_new_bytes(raw / "ADDR2LINE.txt", local_addr2line["stdout"])
        write_new_bytes(raw / "ADDR2LINE.stderr", local_addr2line["stderr"])
        write_new_json(
            raw / "ADDR2LINE_PRODUCER.json",
            {
                "schema": "lay.v10.e1-traversal-d2-addr2line-producer.v1",
                "execution_scope": "local byte-identical sealed ELF",
                "remote_addr2line_executed": False,
                **local_addr2line["summary"],
            },
        )
        frames = parse_addr2line(raw / "ADDR2LINE.txt", addresses)

        marker = consume_marker()
        marker_consumed = True
        write_new_json(stage / "MARKER_CONSUMPTION.json", marker)
        ranges = build_ranges(symbols, instructions, frames)

        bucket_bytes = Counter()
        bucket_ranges = Counter()
        sub_bucket_bytes = Counter()
        ambiguous_bytes = 0
        for value in ranges:
            bucket_bytes[value["bucket"]] += value["length_bytes"]
            bucket_ranges[value["bucket"]] += 1
            sub_bucket_bytes[f"{value['bucket']}/{value['sub_bucket']}"] += value["length_bytes"]
            if value["ambiguous"]:
                ambiguous_bytes += value["length_bytes"]
        bucket_map = {
            "schema": "lay.v10.e1-traversal-d2-bucket-map.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "elf_sha256": ELF_SHA256,
            "build_id": BUILD_ID,
            "elf_type": "ET_DYN",
            "address_space": "ELF virtual address",
            "join_key": ["Build ID", "normalized ELF virtual IP"],
            "normalization": {
                "ET_EXEC": "normalized_ip = sampled_ip",
                "ET_DYN": "load_bias = mapping_start - align_down(PT_LOAD.p_vaddr,page_size); normalized_ip = sampled_ip - load_bias",
                "forbidden": "pathname-base or minimum-sample-IP guessing",
            },
            "text": {**elf_metadata["text"], "sha256": TEXT_SHA256},
            "executable_pt_load": elf_metadata["executable_pt_load"],
            "symbols": symbols,
            "addr2line_producer": local_addr2line["summary"],
            "ranges": ranges,
            "range_count": len(ranges),
            "coverage": {
                "text_bytes": TEXT_SIZE,
                "covered_bytes": sum(value["length_bytes"] for value in ranges),
                "gap_bytes": 0,
                "overlap_count": 0,
                "range_outside_exec_pt_load": 0,
                "machine_byte_hash_mismatches": 0,
                "instruction_alignment_mismatches": 0,
            },
            "bucket_bytes": dict(sorted(bucket_bytes.items())),
            "bucket_range_counts": dict(sorted(bucket_ranges.items())),
            "sub_bucket_bytes": dict(sorted(sub_bucket_bytes.items())),
            "ambiguous_bytes": ambiguous_bytes,
            "reserved_absent_sub_buckets": {
                "REDUNDANT_STATE_DECODE": {"status": "ABSENT", "range_count": 0, "length_bytes": 0},
                "TERMINAL_RANK": {"status": "ABSENT_IN_TRACE_FALSE_MACHINE_CODE", "range_count": 0, "length_bytes": 0},
            },
            "post_sample_reassignment_permitted": False,
        }
        write_new_json(stage / "D2_BUCKET_MAP.json", bucket_map)
        map_identity = row(stage / "D2_BUCKET_MAP.json")
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-bucket-map-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_BUCKET_MAP_SEALED",
            "build_audit_receipt_sha256": BUILD_AUDIT_SHA256,
            "reader_correction_v3_sha256": READER_CORRECTION_V3_SHA256,
            "elf": row(ELF),
            "build_id": BUILD_ID,
            "text_sha256": TEXT_SHA256,
            "addr2line_producer": local_addr2line["summary"],
            "map": map_identity,
            "range_count": len(ranges),
            "text_coverage_bytes": TEXT_SIZE,
            "gap_bytes": 0,
            "overlap_count": 0,
            "range_outside_exec_pt_load": 0,
            "machine_byte_hash_mismatches": 0,
            "reserved_redundant_state_decode": "ABSENT",
            "marker": marker,
            "other_available_markers": 9,
            "other_consumed_markers": 1,
            "parity_marker_consumed": False,
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
            "next_action_admitted": "independent read-only bucket-map audit only; parity remains unexecuted",
        }
        write_new_json(stage / "D2_BUCKET_MAP_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, RESULT)
        fsync_directory(PARENT)
        published_receipt = RESULT / "D2_BUCKET_MAP_RECEIPT.json"
        publish_state("D2_BUCKET_MAP_SEALED", sha256_file(published_receipt))
        return {**receipt, "published_receipt_sha256": sha256_file(published_receipt), "remote_result": str(RESULT)}
    except BaseException as error:
        if marker_consumed:
            try:
                if stage.exists():
                    for path in [stage, *stage.rglob("*")]:
                        path.chmod(0o700 if path.is_dir() else 0o600)
                    checksum_manifest = stage / "SHA256SUMS"
                    if checksum_manifest.exists():
                        checksum_manifest.unlink()
                    write_new_json(
                        stage / "FAILURE.json",
                        {
                            "schema": "lay.v10.e1-traversal-d2-primary-only-bucket-map-failure.v1",
                            "task_id": TASK_ID,
                            "transaction_id": TRANSACTION_ID,
                            "verdict": "BLOCKED_BUCKET_MAP",
                            "error": f"{type(error).__name__}: {error}",
                            "marker_consumed": True,
                            "retry_permitted": False,
                            "parity_executed": False,
                        },
                    )
                    write_sha256sums(stage)
                    seal_tree(stage)
                    os.rename(stage, FAILURE)
                    fsync_directory(PARENT)
                publish_state("BLOCKED_BUCKET_MAP", None)
            except BaseException:
                pass
        elif stage.exists():
            shutil.rmtree(stage)
        raise


def main() -> int:
    try:
        need(len(sys.argv) == 2, "expected one base64 payload argument")
        payload = json.loads(base64.b64decode(sys.argv[1], validate=True))
        action = payload.get("action")
        if action == "probe":
            value = verify_admission(payload)
            value["verdict"] = "D2_BUCKET_MAP_REMOTE_PROBE_PASS"
        elif action == "bucket-map-once":
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = map_once(payload)
        else:
            raise MapError(f"unsupported action: {action!r}")
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        return 0
    except Exception as error:
        print(f"D2 BUCKET MAP REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
