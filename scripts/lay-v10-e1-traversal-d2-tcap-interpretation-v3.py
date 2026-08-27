#!/usr/bin/env python3

from __future__ import annotations

import argparse
import ast
import contextlib
import hashlib
import json
import os
import pathlib
import re
import shlex
import stat
import struct
import subprocess
import sys
import tempfile
import time
from typing import Any


REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_YES = pathlib.PurePosixPath("/usr/bin/yes")
EXPECTED_YES_SHA256 = "ee431b97fb62f59ee94fa698dbc98971001bbb1cbd9c5e32ce4ab4c5530924d8"
EXPECTED_YES_BUILD_ID = "8c99ebc2c856857219acc612c2d9be3172b74be5"

TASK_ID = "slice8b-v10-e1-traversal-d2-tcap-interpretation-v3-20260825"
PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
STATE_ROOT = pathlib.Path.home() / ".local/state/lay" / TASK_ID
RESULT_ROOT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_INTERPRETATION_V3_2026-08-25"
)

PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_INTERPRETATION_V3_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_INTERPRETATION_V3_PREFLIGHT_2026-08-25.json"
)
CORRECTION = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_SALVAGE_INTERPRETATION_CORRECTION_V3_2026-08-25.md"
)
CORRECTION_ROUTE_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_SALVAGE_INTERPRETATION_CORRECTION_V3_ROUTE_RECEIPT_2026-08-25.json"
)
V2_ROOT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_SALVAGE_V2_2026-08-25"
)
V2_REMOTE = V2_ROOT / "REMOTE_EVIDENCE"
V1_TCAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/"
    "P1_REMOTE_EVIDENCE/T-CAP"
)
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"

EXPECTED = {
    "preflight_manifest": "bd5fafde6828e13257b178c2c5af02a62e31b6b3f3579a36ff9ff937e276e317",
    "preflight_receipt": "95c7cf9f914ad860489684476ba7568b7dd99ef7b772381012ddcad136644f0a",
    "correction": "57c4e7fab83e36067fc383e4c30545c76a4eb7deec6bf3b2c0b5bf57cc4d1ac8",
    "correction_route_receipt": "62d8e2a2266492404b33c10a499581f283cdf683d4cc753d855f496f78c8e872",
    "v2_local_receipt": "805fb3316b6a64b507e94b515e533ecbf18852d61d8b20d86cadfd5ea0039ef9",
    "v2_manifest": "23cbcaeb9021b8693ccbcdbbe1df2efa2b6d6883dfa806656926bc29bd892764",
    "v2_remote_receipt": "7b2348e795b5392449871056983b129dd2cfcf02f0584b2bd7cda0eb046abfff",
    "evlist": "8c281ab1b257e02c31ad8b5d42da3127dc34bad85dbc31656f9bd8ecd6c84b43",
    "samples": "997d8c64b482509959b0f728e8c12df16a0e994aff1e6386d0cd3a42b28e7f4a",
    "raw_records": "fef73b5591319fdf137fea05a8839fe1691f1ab4c8ddd4c7886e7fe9ebc647a5",
    "buildids": "a2c4d26d06f0d008176a2774611cef6ee840b1ecda2f12198bb12db3db16e489",
    "maps_before": "efee21f8e02424cdd40800356c492fa8730be99b6bbffe3d36d415433ca7df95",
    "maps_during": "efee21f8e02424cdd40800356c492fa8730be99b6bbffe3d36d415433ca7df95",
    "record_command": "d2aa8bec8fcdcb0bf0fbc702c08592d0e14bd0285a93569b03d36bf9f5f423b3",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
}

EXPECTED_RECORD_COMMAND = [
    "/usr/lib/linux-tools/6.8.0-124-generic/perf",
    "record",
    "--buildid-all",
    "--sample-cpu",
    "--timestamp",
    "--event",
    "task-clock:u",
    "--count",
    "100000",
    "--pid",
    "2888545",
    "--output",
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-traversal-d2-capability-probe-20260825.stage-2888541-1787671831776878395/"
    "T-CAP/perf.data",
]


class InterpretationError(RuntimeError):
    verdict = "BLOCKED_TCAP_EVIDENCE"


class ProvenanceError(InterpretationError):
    verdict = "BLOCKED_PROVENANCE"


class ControllerError(InterpretationError):
    verdict = "BLOCKED_CONTROLLER_PROTOCOL"


def require(
    condition: bool,
    message: str,
    error_type: type[InterpretationError] = InterpretationError,
) -> None:
    if not condition:
        raise error_type(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def file_identity(
    path: pathlib.Path,
    expected_sha256: str | None = None,
    expected_mode: str | None = None,
) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}", ProvenanceError)
    identity = {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }
    if expected_sha256 is not None:
        require(identity["sha256"] == expected_sha256, f"SHA-256 mismatch: {path}", ProvenanceError)
    if expected_mode is not None:
        require(identity["mode"] == expected_mode, f"mode mismatch: {path}", ProvenanceError)
    return identity


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def write_new_bytes(path: pathlib.Path, data: bytes, mode: int = 0o600) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
    except Exception:
        with contextlib.suppress(FileNotFoundError):
            path.unlink()
        raise


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o600) -> None:
    write_new_bytes(path, canonical_json_bytes(value), mode)


def read_json(path: pathlib.Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_sha256sums(root: pathlib.Path) -> None:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing manifest: {manifest}", ProvenanceError)
    for line in manifest.read_text(encoding="utf-8").splitlines():
        expected, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest path missing: {relative}", ProvenanceError)
        require(sha256_file(path) == expected, f"manifest mismatch: {relative}", ProvenanceError)


def write_sha256sums(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        relative = path.relative_to(root)
        if relative == pathlib.Path("SHA256SUMS"):
            continue
        rows.append(f"{sha256_file(path)}  {relative}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode(), 0o400)


def seal_tree(root: pathlib.Path) -> None:
    write_sha256sums(root)
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)
    verify_sha256sums(root)


def ssh_read(argv: list[str]) -> bytes:
    command = " ".join(shlex.quote(item) for item in argv)
    result = subprocess.run(
        ["ssh", "-o", "BatchMode=yes", REMOTE, command],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(result.returncode == 0, f"remote read failed: {result.stderr[-2000:]!r}", ProvenanceError)
    return result.stdout


def copy_remote_yes(destination: pathlib.Path) -> dict[str, Any]:
    hostname = ssh_read(["hostname"]).decode().strip()
    require(hostname == REMOTE_HOSTNAME, f"remote hostname mismatch: {hostname}", ProvenanceError)
    machine_id_output = ssh_read(["sha256sum", "--", "/etc/machine-id"]).decode().strip()
    machine_id_sha256 = machine_id_output.split()[0]
    require(
        machine_id_sha256 == REMOTE_MACHINE_ID_SHA256,
        "remote machine-id mismatch",
        ProvenanceError,
    )
    yes_output = ssh_read(["sha256sum", "--", str(REMOTE_YES)]).decode().strip()
    remote_yes_sha256 = yes_output.split()[0]
    require(remote_yes_sha256 == EXPECTED_YES_SHA256, "remote yes SHA mismatch", ProvenanceError)
    result = subprocess.run(
        ["scp", "-q", f"{REMOTE}:{REMOTE_YES}", str(destination)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(result.returncode == 0, f"remote yes copy failed: {result.stderr[-2000:]!r}", ProvenanceError)
    destination.chmod(0o400)
    copied = file_identity(destination, EXPECTED_YES_SHA256, "0400")
    return {
        "remote": REMOTE,
        "hostname": hostname,
        "machine_id_sha256": machine_id_sha256,
        "path": str(REMOTE_YES),
        "pre_copy_sha256": remote_yes_sha256,
        "copied": copied,
    }


def input_snapshot() -> dict[str, dict[str, Any]]:
    paths = {
        "preflight_manifest": (PREFLIGHT_MANIFEST, EXPECTED["preflight_manifest"], "0664"),
        "preflight_receipt": (PREFLIGHT_RECEIPT, EXPECTED["preflight_receipt"], "0664"),
        "correction": (CORRECTION, EXPECTED["correction"], "0664"),
        "correction_route_receipt": (
            CORRECTION_ROUTE_RECEIPT,
            EXPECTED["correction_route_receipt"],
            "0444",
        ),
        "v2_local_receipt": (V2_ROOT / "T_CAP_RECOVERY_RECEIPT.json", EXPECTED["v2_local_receipt"], "0400"),
        "v2_manifest": (V2_REMOTE / "SHA256SUMS", EXPECTED["v2_manifest"], "0444"),
        "v2_remote_receipt": (
            V2_REMOTE / "T_CAP_RECOVERY_RECEIPT.json",
            EXPECTED["v2_remote_receipt"],
            "0444",
        ),
        "evlist": (V2_REMOTE / "evlist.stdout", EXPECTED["evlist"], "0444"),
        "samples": (V2_REMOTE / "samples.stdout", EXPECTED["samples"], "0444"),
        "raw_records": (V2_REMOTE / "raw-records.stdout", EXPECTED["raw_records"], "0444"),
        "buildids": (V2_REMOTE / "buildids.stdout", EXPECTED["buildids"], "0444"),
        "maps_before": (V1_TCAP / "maps-before.txt", EXPECTED["maps_before"], "0444"),
        "maps_during": (V1_TCAP / "maps-during.txt", EXPECTED["maps_during"], "0444"),
        "record_command": (V1_TCAP / "record-command.json", EXPECTED["record_command"], "0444"),
    }
    return {
        name: file_identity(path, expected_sha, expected_mode)
        for name, (path, expected_sha, expected_mode) in paths.items()
    }


def canonicalize_dso(value: str) -> str:
    require(value.startswith("(") and value.endswith(")"), f"DSO wrapper mismatch: {value!r}")
    inner = value[1:-1]
    require(inner != "", "empty DSO wrapper")
    require("(" not in inner and ")" not in inner, f"nested or unmatched DSO wrapper: {value!r}")
    return inner


def parse_samples(text: str) -> list[dict[str, Any]]:
    samples = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        match = re.fullmatch(
            r"\[(\d{3})\]\s+([^\s]+):\s+([0-9a-fA-F]+)\s+(\S+)",
            line.strip(),
        )
        require(match is not None, f"unparsed sample row {line_number}: {line!r}")
        samples.append(
            {
                "cpu": int(match.group(1)),
                "event": match.group(2),
                "runtime_ip": int(match.group(3), 16),
                "rendered_dso": match.group(4),
                "canonical_dso": canonicalize_dso(match.group(4)),
            }
        )
    return samples


def exact_event_line(text: str) -> str:
    lines = [line for line in text.splitlines() if line.startswith("task-clock:u:")]
    require(len(lines) == 1, f"task-clock evlist line count: {len(lines)}")
    return lines[0]


def parse_required_attr(line: str, key: str) -> int:
    match = re.search(rf"\b{re.escape(key)}\s*:\s*(0x[0-9a-fA-F]+|\d+)", line)
    if match is None and key == "sample_period":
        match = re.search(
            r"\{\s*sample_period\s*,\s*sample_freq\s*\}\s*:\s*(0x[0-9a-fA-F]+|\d+)",
            line,
        )
    require(match is not None, f"sealed task-clock line lacks {key}")
    return int(match.group(1), 0)


def parse_maps(text: str, expected_path: str) -> list[dict[str, Any]]:
    mappings = []
    for line in text.splitlines():
        fields = line.split(maxsplit=5)
        if len(fields) < 6 or fields[5] != expected_path:
            continue
        start_text, end_text = fields[0].split("-", 1)
        mappings.append(
            {
                "start": int(start_text, 16),
                "end": int(end_text, 16),
                "permissions": fields[1],
                "offset": int(fields[2], 16),
                "path": fields[5],
            }
        )
    require(mappings, f"no mappings for {expected_path}")
    return mappings


def align4(value: int) -> int:
    return (value + 3) & ~3


def elf64_info(path: pathlib.Path) -> dict[str, Any]:
    data = path.read_bytes()
    require(data[:4] == b"\x7fELF", "copied yes is not ELF", ProvenanceError)
    require(data[4] == 2 and data[5] == 1, "copied yes is not little-endian ELF64", ProvenanceError)
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, 0)
    elf_type = header[1]
    program_offset = header[5]
    program_entry_size = header[9]
    program_count = header[10]
    require(program_entry_size == 56, "unexpected ELF64 program-header size", ProvenanceError)
    segments = []
    note_regions = []
    for index in range(program_count):
        values = struct.unpack_from("<IIQQQQQQ", data, program_offset + index * program_entry_size)
        segment_type, flags, offset, vaddr, _paddr, filesz, memsz, alignment = values
        require(offset + filesz <= len(data), f"program segment {index} exceeds ELF", ProvenanceError)
        if segment_type == 1:
            segments.append(
                {
                    "flags": flags,
                    "offset": offset,
                    "vaddr": vaddr,
                    "filesz": filesz,
                    "memsz": memsz,
                    "align": alignment,
                }
            )
        elif segment_type == 4:
            note_regions.append((offset, filesz))
    require(elf_type == 3, f"copied yes ELF type is {elf_type}, expected ET_DYN", ProvenanceError)
    require(segments, "copied yes has no PT_LOAD", ProvenanceError)
    build_ids = []
    for offset, size in note_regions:
        cursor = offset
        limit = offset + size
        while cursor + 12 <= limit:
            namesz, descsz, note_type = struct.unpack_from("<III", data, cursor)
            cursor += 12
            name = data[cursor : cursor + namesz]
            cursor += align4(namesz)
            description = data[cursor : cursor + descsz]
            cursor += align4(descsz)
            require(cursor <= limit, "ELF note exceeds PT_NOTE", ProvenanceError)
            if note_type == 3 and name.rstrip(b"\x00") == b"GNU":
                build_ids.append(description.hex())
    require(build_ids == [EXPECTED_YES_BUILD_ID], f"copied yes Build IDs mismatch: {build_ids}", ProvenanceError)
    return {"elf_type": "ET_DYN", "build_id": build_ids[0], "pt_load": segments}


def normalize_yes_ips(
    yes_samples: list[dict[str, Any]],
    mappings: list[dict[str, Any]],
    segments: list[dict[str, int]],
) -> dict[str, Any]:
    page_size = os.sysconf("SC_PAGE_SIZE")
    rows = []
    for sample_index, sample in enumerate(yes_samples):
        runtime_ip = sample["runtime_ip"]
        matching_mappings = [
            mapping
            for mapping in mappings
            if mapping["start"] <= runtime_ip < mapping["end"] and "x" in mapping["permissions"]
        ]
        require(
            len(matching_mappings) == 1,
            f"yes sample {sample_index} executable mapping count is {len(matching_mappings)}",
        )
        mapping = matching_mappings[0]
        matching_segments = [
            segment
            for segment in segments
            if segment["flags"] & 1
            and (segment["offset"] // page_size) * page_size == mapping["offset"]
        ]
        require(
            len(matching_segments) == 1,
            f"yes sample {sample_index} executable PT_LOAD count is {len(matching_segments)}",
        )
        segment = matching_segments[0]
        load_bias = mapping["start"] - (segment["vaddr"] // page_size) * page_size
        normalized_ip = runtime_ip - load_bias
        require(
            segment["vaddr"] <= normalized_ip < segment["vaddr"] + segment["memsz"],
            f"yes sample {sample_index} normalized IP outside PT_LOAD",
        )
        rows.append(
            {
                "sample_index": sample_index,
                "runtime_ip": f"0x{runtime_ip:x}",
                "normalized_ip": f"0x{normalized_ip:x}",
                "load_bias": f"0x{load_bias:x}",
                "mapping_start": f"0x{mapping['start']:x}",
                "mapping_offset": f"0x{mapping['offset']:x}",
                "pt_load_vaddr": f"0x{segment['vaddr']:x}",
            }
        )
    return {
        "schema": "lay.v10.e1-traversal-d2-tcap-ip-normalization.v3",
        "identity_key": {
            "build_id": EXPECTED_YES_BUILD_ID,
            "dso": str(REMOTE_YES),
        },
        "normalization": "runtime_ip - load_bias, with PT_LOAD page geometry",
        "page_size": page_size,
        "sample_count": len(rows),
        "unique_load_biases": sorted({row["load_bias"] for row in rows}),
        "rows": rows,
    }


def local_runtime_projection() -> dict[str, Any]:
    version = subprocess.run(
        [str(pathlib.Path.home() / ".local/bin/lay"), "--version"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    ).stdout.decode().strip()
    pids = {}
    for name in ("ibus-daemon", "lay-daemon", "lay-ibus-engine"):
        result = subprocess.run(
            ["pgrep", "-xo", name],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        pids[name] = int(result.stdout.decode().strip())
    return {
        "lay_version": version,
        "active_v11_sha256": sha256_file(ACTIVE_V11),
        "pids": pids,
    }


def source_scan() -> dict[str, Any]:
    source = pathlib.Path(__file__).read_text(encoding="utf-8")
    tree = ast.parse(source)
    subprocess_calls = []
    remote_read_calls = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            function = ast.unparse(node.func)
            if function in {"subprocess.run", "subprocess.Popen", "os.system", "os.posix_spawn"}:
                subprocess_calls.append(ast.get_source_segment(source, node) or function)
            if function == "ssh_read":
                remote_read_calls.append(ast.unparse(node.args[0]))
    require("subprocess.Popen" not in subprocess_calls, "Popen is forbidden", ControllerError)
    require("os.system" not in subprocess_calls, "os.system is forbidden", ControllerError)
    require("os.posix_spawn" not in subprocess_calls, "posix_spawn is forbidden", ControllerError)
    command_surface = "\n".join(subprocess_calls)
    require(
        re.search(r"(?i)\b(?:perf|cargo|rustc|taskset|pkill|killall|renice|systemctl)\b", command_surface)
        is None,
        "forbidden subprocess route present",
        ControllerError,
    )
    require(
        sorted(remote_read_calls)
        == sorted(
            [
                "['hostname']",
                "['sha256sum', '--', '/etc/machine-id']",
                "['sha256sum', '--', str(REMOTE_YES)]",
            ]
        ),
        f"remote read allowlist drift: {remote_read_calls}",
        ControllerError,
    )
    return {
        "subprocess_call_count": len(subprocess_calls),
        "allowed_external_programs": ["hostname", "lay --version", "pgrep", "scp", "sha256sum", "ssh"],
        "perf_invocation_route": False,
        "subject_execution_route": False,
        "cargo_or_rustc_route": False,
        "foreign_process_control_route": False,
    }


def parser_self_check() -> dict[str, Any]:
    sample = parse_samples("[000] task-clock:u: 62cefc9d977e (/usr/bin/yes)\n")
    require(len(sample) == 1, "sample parser fixture count")
    require(sample[0]["canonical_dso"] == "/usr/bin/yes", "sample DSO fixture")
    fixtures_rejected = 0
    for value in ("/usr/bin/yes", "((/usr/bin/yes))", "(/usr/bin/yes", "/usr/bin/yes)"):
        try:
            canonicalize_dso(value)
        except InterpretationError:
            fixtures_rejected += 1
    require(fixtures_rejected == 4, "DSO rejection fixture mismatch")
    line = (
        "task-clock:u: type: 1 (software), config: 0x1, "
        "{ sample_period, sample_freq }: 100000, exclude_kernel: 1"
    )
    require(parse_required_attr(line, "type") == 1, "type parser fixture")
    require(parse_required_attr(line, "config") == 1, "config parser fixture")
    require(parse_required_attr(line, "sample_period") == 100_000, "period parser fixture")
    return {"sample_rows": 1, "dso_invalid_fixtures_rejected": 4, "fixed_period_union": True}


def self_check() -> dict[str, Any]:
    before = input_snapshot()
    verify_sha256sums(V2_REMOTE)
    preflight = read_json(PREFLIGHT_RECEIPT)
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "interpretation preflight not ready")
    require(preflight.get("safe_to_implement") is True, "interpretation preflight unsafe")
    require(preflight.get("blockers") == [], "interpretation preflight blockers present")
    require(read_json(V2_ROOT / "T_CAP_RECOVERY_RECEIPT.json")["verdict"] == "BLOCKED_TCAP_EVIDENCE", "V2 verdict drift")
    require(read_json(V1_TCAP / "record-command.json") == EXPECTED_RECORD_COMMAND, "record command drift")
    for name in ("evlist", "samples", "raw-records", "buildids"):
        stderr = V2_REMOTE / f"{name}.stderr"
        require(stderr.is_file() and stderr.stat().st_size == 0, f"nonempty or missing {name} stderr")
    scan = source_scan()
    parser = parser_self_check()
    require(input_snapshot() == before, "self-check mutated sealed inputs", ProvenanceError)
    result = {"verdict": "PASS_UNRUN", "source_scan": scan, "parser": parser}
    print(json.dumps(result, sort_keys=True))
    return result


def consume_marker() -> pathlib.Path:
    require(not STATE_ROOT.exists(), f"interpretation state already exists: {STATE_ROOT}", ProvenanceError)
    marker_root = STATE_ROOT / "markers"
    marker_root.mkdir(parents=True, mode=0o700)
    write_new_bytes(STATE_ROOT / "route.lock", f"{TASK_ID}\n".encode(), 0o400)
    available = marker_root / "interpretation.available"
    consumed = marker_root / "interpretation.consumed-before-exec"
    write_new_bytes(available, b"available\n", 0o400)
    available.rename(consumed)
    require(consumed.is_file() and not available.exists(), "interpretation marker consumption failed", ControllerError)
    return consumed


def interpret(stage: pathlib.Path) -> tuple[dict[str, Any], dict[str, Any]]:
    remote_yes = stage / "REMOTE_YES.elf"
    remote_identity = copy_remote_yes(remote_yes)
    elf = elf64_info(remote_yes)

    evlist_text = (V2_REMOTE / "evlist.stdout").read_text(encoding="utf-8")
    sample_text = (V2_REMOTE / "samples.stdout").read_text(encoding="utf-8")
    raw_text = (V2_REMOTE / "raw-records.stdout").read_text(encoding="utf-8")
    buildids_text = (V2_REMOTE / "buildids.stdout").read_text(encoding="utf-8")
    maps_before_text = (V1_TCAP / "maps-before.txt").read_text(encoding="utf-8")
    maps_during_text = (V1_TCAP / "maps-during.txt").read_text(encoding="utf-8")

    samples = parse_samples(sample_text)
    require(len(samples) == 4_578, f"sample count mismatch: {len(samples)}")
    require({sample["cpu"] for sample in samples} == {0}, "sample CPU mismatch")
    require({sample["event"] for sample in samples} == {"task-clock:u"}, "sample event mismatch")
    yes_samples = [sample for sample in samples if sample["canonical_dso"] == str(REMOTE_YES)]
    require(len(yes_samples) == 927, f"yes sample count mismatch: {len(yes_samples)}")

    event_line = exact_event_line(evlist_text)
    require(parse_required_attr(event_line, "type") == 1, "task-clock type mismatch")
    require(parse_required_attr(event_line, "config") == 1, "task-clock config mismatch")
    require(parse_required_attr(event_line, "sample_period") == 100_000, "task-clock period mismatch")
    require(parse_required_attr(event_line, "exclude_kernel") == 1, "task-clock privilege mismatch")
    require(re.search(r"\bfreq\s*:", event_line) is None, "unexpected explicit freq flag")
    require(re.search(r"\bprecise_ip\s*:", event_line) is None, "unexpected explicit precise_ip flag")

    raw_samples = len(re.findall(r"PERF_RECORD_SAMPLE\b", raw_text))
    lost = len(re.findall(r"PERF_RECORD_LOST(?:_SAMPLES)?\b", raw_text))
    throttle = len(re.findall(r"PERF_RECORD_THROTTLE\b", raw_text))
    unthrottle = len(re.findall(r"PERF_RECORD_UNTHROTTLE\b", raw_text))
    require(raw_samples == 4_578, f"raw sample count mismatch: {raw_samples}")
    require((lost, throttle, unthrottle) == (0, 0, 0), "lost or throttle records observed")
    build_id_pattern = rf"(?m)^{EXPECTED_YES_BUILD_ID}\s+{re.escape(str(REMOTE_YES))}$"
    require(re.search(build_id_pattern, buildids_text) is not None, "sealed yes Build ID mismatch")

    mappings_before = parse_maps(maps_before_text, str(REMOTE_YES))
    mappings_during = parse_maps(maps_during_text, str(REMOTE_YES))
    require(mappings_before == mappings_during, "yes maps-before/maps-during mismatch")
    normalization = normalize_yes_ips(yes_samples, mappings_during, elf["pt_load"])
    require(normalization["sample_count"] == 927, "normalization denominator mismatch")
    write_new_json(stage / "YES_IP_NORMALIZATION.json", normalization, 0o400)

    validation = {
        "v2_reader_invocations": 4,
        "v3_reader_invocations": 0,
        "reader_stderr_bytes": {name: 0 for name in ("evlist", "samples", "raw-records", "buildids")},
        "sample_count": len(samples),
        "yes_sample_count": len(yes_samples),
        "sample_cpus": [0],
        "sample_events": ["task-clock:u"],
        "event_type": 1,
        "event_config": 1,
        "sample_period": 100_000,
        "freq": 0,
        "freq_derivation": "absent fixed-count optional flag defaults to zero",
        "exclude_kernel": 1,
        "precise_ip": 0,
        "precise_ip_derivation": "absent task-clock optional flag defaults to zero",
        "raw_sample_records": raw_samples,
        "lost_records": lost,
        "throttle_records": throttle,
        "unthrottle_records": unthrottle,
        "yes_build_id": EXPECTED_YES_BUILD_ID,
        "maps_before_equal_maps_during": True,
        "canonical_yes_dso": str(REMOTE_YES),
        "normalized_yes_ips": normalization["sample_count"],
        "normalization_unique_load_biases": normalization["unique_load_biases"],
    }
    return validation, {"remote_identity": remote_identity, "elf": elf}


def run_interpretation() -> None:
    static_check = self_check()
    require(not RESULT_ROOT.exists(), f"interpretation result already exists: {RESULT_ROOT}", ProvenanceError)
    before_inputs = input_snapshot()
    before_runtime = local_runtime_projection()
    require(before_runtime["lay_version"] == "lay 1.0.43", "Lay version drift", ProvenanceError)
    require(before_runtime["active_v11_sha256"] == EXPECTED["active_v11"], "active V11 drift", ProvenanceError)
    marker = consume_marker()
    stage = RESULT_ROOT.parent / f".{RESULT_ROOT.name}.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    write_new_bytes(stage / "controller.py", pathlib.Path(__file__).read_bytes(), 0o400)

    error: str | None = None
    failure_verdict: str | None = None
    validation: dict[str, Any] | None = None
    yes_evidence: dict[str, Any] | None = None
    try:
        validation, yes_evidence = interpret(stage)
    except InterpretationError as failure:
        error = f"{type(failure).__name__}: {failure}"
        failure_verdict = failure.verdict
    except Exception as failure:
        error = f"{type(failure).__name__}: {failure}"
        failure_verdict = "BLOCKED_CONTROLLER_PROTOCOL"

    after_runtime = local_runtime_projection()
    after_inputs = input_snapshot()
    if after_inputs != before_inputs and error is None:
        error = "sealed input identity drift"
        failure_verdict = "BLOCKED_PROVENANCE"
    if after_runtime != before_runtime and error is None:
        error = f"local runtime projection changed: {after_runtime}"
        failure_verdict = "BLOCKED_PROVENANCE"
    verdict = (
        "T_CAP_RECOVERED_FROM_SEALED_EVIDENCE"
        if error is None and validation is not None and yes_evidence is not None
        else (failure_verdict or "BLOCKED_TCAP_EVIDENCE")
    )
    receipt = {
        "schema": "lay.v10.e1-traversal-d2-tcap-interpretation.v3",
        "task_id": TASK_ID,
        "verdict": verdict,
        "error": error,
        "historical_v1_verdict_unchanged": "BLOCKED_CAPABILITY",
        "historical_v2_verdict_unchanged": "BLOCKED_TCAP_EVIDENCE",
        "effective_tcap_capability": "PASS_FROM_SEALED_EVIDENCE" if verdict.startswith("T_CAP_RECOVERED") else "UNKNOWN",
        "controller": file_identity(pathlib.Path(__file__)),
        "static_check": static_check,
        "marker": str(marker),
        "marker_consumed_before_interpretation": marker.is_file(),
        "inputs_before": before_inputs,
        "inputs_after_equal": after_inputs == before_inputs,
        "yes_evidence": yes_evidence,
        "validation": validation,
        "execution_counts": {
            "perf_executable_invocations": 0,
            "perf_reader_invocations": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "event_open_invocations": 0,
            "yes_execution_invocations": 0,
            "d2_subject_executions": 0,
            "cargo_build_check_test_invocations": 0,
            "rustc_invocations": 0,
        },
        "runtime_before": before_runtime,
        "runtime_after": after_runtime,
        "runtime_authority_changed": False,
        "precise_only_preflight_admitted": verdict == "T_CAP_RECOVERED_FROM_SEALED_EVIDENCE",
        "precise_event_admitted_by_this_receipt": False,
        "d2_implementation_admitted": False,
        "full_b_admitted": False,
        "v12_admitted": False,
        "runtime_integration_admitted": False,
        "retry_permitted": False,
    }
    write_new_json(stage / "T_CAP_INTERPRETATION_RECEIPT.json", receipt, 0o400)
    seal_tree(stage)
    os.replace(stage, RESULT_ROOT)
    print(json.dumps({"verdict": verdict, "receipt": str(RESULT_ROOT / "T_CAP_INTERPRETATION_RECEIPT.json")}, sort_keys=True))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("self-check")
    subparsers.add_parser("run")
    return parser.parse_args()


def main() -> None:
    arguments = parse_args()
    if arguments.command == "self-check":
        self_check()
    elif arguments.command == "run":
        run_interpretation()
    else:
        raise ControllerError(f"unknown command: {arguments.command}")


if __name__ == "__main__":
    try:
        main()
    except InterpretationError as error:
        print(json.dumps({"verdict": error.verdict, "error": str(error)}), file=sys.stderr)
        raise SystemExit(1)
