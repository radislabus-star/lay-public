#!/usr/bin/env python3

from __future__ import annotations

import argparse
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
TASK_ID = "slice8b-v10-e1-traversal-d2-tcap-salvage-v2-20260825"
REMOTE_FINAL = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
V1_REMOTE = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-traversal-d2-capability-probe-20260825"
)

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_SALVAGE_V3_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_SALVAGE_V3_PREFLIGHT_2026-08-25.json"
)
REPAIR_CONTRACT = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_CAPABILITY_SHUTDOWN_REPAIR_V2_2026-08-25.md"
)
REPAIR_ROUTE_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_CAPABILITY_SHUTDOWN_REPAIR_V2_ROUTE_RECEIPT_2026-08-25.json"
)
V1_LOCAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/"
    "P1_REMOTE_EVIDENCE"
)
LOCAL_RESULT_ROOT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_SALVAGE_V2_2026-08-25"
)
LOCAL_REMOTE_EVIDENCE = LOCAL_RESULT_ROOT / "REMOTE_EVIDENCE"
LOCAL_RECEIPT = LOCAL_RESULT_ROOT / "T_CAP_RECOVERY_RECEIPT.json"

PERF = pathlib.Path("/usr/lib/linux-tools/6.8.0-124-generic/perf")
YES = pathlib.Path("/usr/bin/yes")
V1_PERF_DATA = V1_REMOTE / "T-CAP/perf.data"
V1_MAPS_BEFORE = V1_REMOTE / "T-CAP/maps-before.txt"
V1_MAPS_DURING = V1_REMOTE / "T-CAP/maps-during.txt"
EXPECTED_YES_BUILD_ID = "8c99ebc2c856857219acc612c2d9be3172b74be5"

EXPECTED = {
    "preflight_manifest": "af1774ded9f0adb309e315aa94c396d5e50d572295f984378f579262b81663d5",
    "preflight_receipt": "df1977a0e531a3064cb908e3559661ffcfeb80b179bfb7023071340af7948764",
    "repair_contract": "e0499e8c435169a66e0e30bfe46b0953b6635010745cdf45302bddfe2a3b6521",
    "repair_route_receipt": "adaf25cb931c7b86f8d0416b4c0d4e0c61179fabf04789a6617601b36c3e810e",
    "v1_manifest": "06c86e7aa849d467c8985ef3b0910090e3d1d8666b576d50d4bc82dc0ab03a55",
    "v1_receipt": "1c41c796458b862813601c8853788675b25b0d221b3447e16237f5f87ed6a8dc",
    "perf_data": "43e67a0b370c14adab60cd229d925f168f7fbaa94930ed2805876d1b01039a99",
    "maps_before": "efee21f8e02424cdd40800356c492fa8730be99b6bbffe3d36d415433ca7df95",
    "maps_during": "efee21f8e02424cdd40800356c492fa8730be99b6bbffe3d36d415433ca7df95",
    "perf": "b0741eb0e6e769ba9ee0ae4e27f0c60909b51be4bc560802aef4bcd91130692e",
    "yes": "ee431b97fb62f59ee94fa698dbc98971001bbb1cbd9c5e32ce4ab4c5530924d8",
}


class SalvageError(RuntimeError):
    verdict = "BLOCKED_TCAP_EVIDENCE"


class ProvenanceError(SalvageError):
    verdict = "BLOCKED_PROVENANCE"


class ReaderError(SalvageError):
    verdict = "BLOCKED_READER"


class CapabilityEvidenceError(SalvageError):
    verdict = "BLOCKED_TCAP_EVIDENCE"


def require(condition: bool, message: str, error_type: type[SalvageError] = SalvageError) -> None:
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


def file_identity(path: pathlib.Path, expected_sha256: str | None = None) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}", ProvenanceError)
    identity = {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }
    if expected_sha256 is not None:
        require(identity["sha256"] == expected_sha256, f"SHA-256 mismatch: {path}", ProvenanceError)
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


def run(command: list[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if check and result.returncode != 0:
        raise ReaderError(
            f"command failed {result.returncode}: {command!r}: {result.stderr[-2000:]!r}"
        )
    return result


def ssh(argv: list[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    command = " ".join(shlex.quote(item) for item in argv)
    result = subprocess.run(
        ["ssh", "-o", "BatchMode=yes", REMOTE, command],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and result.returncode != 0:
        raise SalvageError(f"ssh failed {result.returncode}: {result.stderr[-2000:]!r}")
    return result


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace").strip()


def host_projection() -> dict[str, Any]:
    return {
        "hostname": os.uname().nodename,
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        "kernel": os.uname().release,
        "perf_sha256": sha256_file(PERF),
        "v1_manifest_sha256": sha256_file(V1_REMOTE / "SHA256SUMS"),
        "v1_perf_data_sha256": sha256_file(V1_PERF_DATA),
        "v1_maps_before_sha256": sha256_file(V1_MAPS_BEFORE),
        "v1_maps_during_sha256": sha256_file(V1_MAPS_DURING),
    }


def verify_host_projection(value: dict[str, Any]) -> None:
    require(
        value
        == {
            "hostname": REMOTE_HOSTNAME,
            "machine_id_sha256": REMOTE_MACHINE_ID_SHA256,
            "kernel": "6.8.0-124-generic",
            "perf_sha256": EXPECTED["perf"],
            "v1_manifest_sha256": EXPECTED["v1_manifest"],
            "v1_perf_data_sha256": EXPECTED["perf_data"],
            "v1_maps_before_sha256": EXPECTED["maps_before"],
            "v1_maps_during_sha256": EXPECTED["maps_during"],
        },
        f"host or V1 projection mismatch: {value!r}",
        ProvenanceError,
    )


def verify_sha256sums(root: pathlib.Path) -> None:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing manifest: {manifest}", ProvenanceError)
    for line in manifest.read_text().splitlines():
        expected, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest path missing: {relative}", ProvenanceError)
        require(sha256_file(path) == expected, f"manifest mismatch: {relative}", ProvenanceError)


def consume_marker(state: pathlib.Path) -> pathlib.Path:
    available = state / "markers/readers.available"
    consumed = state / "markers/readers.consumed-before-exec"
    require(available.is_file(), "reader marker unavailable", ProvenanceError)
    require(not consumed.exists(), "reader marker already consumed", ProvenanceError)
    available.rename(consumed)
    return consumed


def parse_maps(text: str, expected_path: pathlib.Path) -> list[dict[str, Any]]:
    mappings = []
    for line in text.splitlines():
        fields = line.split(maxsplit=5)
        if len(fields) < 6 or fields[5] != str(expected_path):
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
    require(mappings, f"no mappings for {expected_path}", CapabilityEvidenceError)
    return mappings


def elf64_load_segments(path: pathlib.Path) -> tuple[int, list[dict[str, int]]]:
    data = path.read_bytes()
    require(data[:4] == b"\x7fELF", "not ELF", ProvenanceError)
    require(data[4] == 2 and data[5] == 1, "requires little-endian ELF64", ProvenanceError)
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, 0)
    elf_type = header[1]
    program_offset = header[5]
    program_entry_size = header[9]
    program_count = header[10]
    require(program_entry_size == 56, "unexpected ELF64 program-header size", ProvenanceError)
    segments = []
    for index in range(program_count):
        values = struct.unpack_from("<IIQQQQQQ", data, program_offset + index * program_entry_size)
        if values[0] == 1:
            segments.append(
                {
                    "flags": values[1],
                    "offset": values[2],
                    "vaddr": values[3],
                    "filesz": values[5],
                    "memsz": values[6],
                    "align": values[7],
                }
            )
    require(segments, "ELF has no PT_LOAD segments", ProvenanceError)
    return elf_type, segments


def parse_perf_samples(text: str) -> list[dict[str, Any]]:
    samples = []
    for line in text.splitlines():
        fields = line.split()
        if len(fields) < 4:
            continue
        cpu_text = fields[0].strip("[]")
        ip_text = fields[-2].removeprefix("0x")
        if not cpu_text.isdigit() or not re.fullmatch(r"[0-9a-fA-F]+", ip_text):
            continue
        samples.append(
            {
                "cpu": int(cpu_text),
                "event": " ".join(fields[1:-2]).rstrip(":"),
                "ip": int(ip_text, 16),
                "dso": fields[-1],
            }
        )
    return samples


def parse_attr(text: str, key: str) -> int:
    match = re.search(rf"\b{re.escape(key)}\s*[:=]\s*(0x[0-9a-fA-F]+|\d+)", text)
    if match is None and key == "sample_period":
        match = re.search(
            r"\{\s*sample_period\s*,\s*sample_freq\s*\}\s*:\s*"
            r"(0x[0-9a-fA-F]+|\d+)",
            text,
        )
    require(match is not None, f"perf evlist lacks {key}", CapabilityEvidenceError)
    return int(match.group(1), 0)


def normalize_yes_ips(samples: list[dict[str, Any]], maps_text: str) -> dict[str, Any]:
    mappings = parse_maps(maps_text, YES)
    elf_type, segments = elf64_load_segments(YES)
    require(elf_type == 3, "yes is not ET_DYN", CapabilityEvidenceError)
    page_size = os.sysconf("SC_PAGE_SIZE")
    yes_samples = [sample for sample in samples if sample["dso"].endswith("/yes")]
    require(yes_samples, "no sampled yes DSO IPs", CapabilityEvidenceError)
    normalized = []
    for sample in yes_samples:
        runtime_ip = sample["ip"]
        matching_mappings = [
            item
            for item in mappings
            if item["start"] <= runtime_ip < item["end"] and "x" in item["permissions"]
        ]
        require(
            len(matching_mappings) == 1,
            f"sample IP executable mapping count is {len(matching_mappings)}: {runtime_ip:#x}",
            CapabilityEvidenceError,
        )
        mapping = matching_mappings[0]
        matching_segments = [
            item
            for item in segments
            if item["flags"] & 1
            and item["offset"] // page_size * page_size == mapping["offset"]
        ]
        require(
            len(matching_segments) == 1,
            f"mapping executable PT_LOAD count is {len(matching_segments)}",
            CapabilityEvidenceError,
        )
        segment = matching_segments[0]
        load_bias = mapping["start"] - segment["vaddr"] // page_size * page_size
        normalized_ip = runtime_ip - load_bias
        require(
            segment["vaddr"] <= normalized_ip < segment["vaddr"] + segment["memsz"],
            f"normalized IP outside executable PT_LOAD: {normalized_ip:#x}",
            CapabilityEvidenceError,
        )
        normalized.append(
            {
                "runtime_ip": f"0x{runtime_ip:x}",
                "normalized_ip": f"0x{normalized_ip:x}",
                "load_bias": f"0x{load_bias:x}",
                "mapping_offset": f"0x{mapping['offset']:x}",
                "pt_load_vaddr": f"0x{segment['vaddr']:x}",
            }
        )
    return {
        "elf_type": "ET_DYN",
        "page_size": page_size,
        "yes_sample_count": len(yes_samples),
        "normalized_count": len(normalized),
        "examples": normalized[:16],
    }


def reader_commands() -> list[tuple[str, list[str]]]:
    source = str(V1_PERF_DATA)
    return [
        ("evlist", [str(PERF), "evlist", "-v", "-i", source]),
        ("samples", [str(PERF), "script", "-i", source, "-F", "cpu,event,ip,dso"]),
        ("raw-records", [str(PERF), "script", "-D", "-i", source]),
        ("buildids", [str(PERF), "buildid-list", "-i", source]),
    ]


def run_reader(name: str, command: list[str], stage: pathlib.Path) -> str:
    write_new_json(stage / f"{name}-command.json", command, 0o400)
    result = run(command, check=False)
    write_new_bytes(stage / f"{name}.stdout", result.stdout, 0o400)
    write_new_bytes(stage / f"{name}.stderr", result.stderr, 0o400)
    require(result.returncode == 0, f"reader {name} exited {result.returncode}", ReaderError)
    return result.stdout.decode("utf-8", errors="replace")


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


def source_scan() -> dict[str, Any]:
    source = pathlib.Path(__file__).read_text()
    scanned_source, excluded_count = re.subn(
        r"\ndef source_scan\(\) -> dict\[str, Any\]:\n.*?\n\ndef parser_self_check\(",
        "\ndef source_scan() -> dict[str, Any]:\n<SELF_SCAN_BODY_EXCLUDED>\n\ndef parser_self_check(",
        source,
        flags=re.DOTALL,
    )
    require(excluded_count == 1, f"source-scan exclusion count mismatch: {excluded_count}")
    patterns = {
        "process_spawn": r"\bPopen\b|os\.system|posix_spawn",
        "perf_mutator": r"\[str\(PERF\),\s*[\"'](?:record|stat)[\"']",
        "subject_execution": r"subprocess[^\n]{0,240}(?:YES|/usr/bin/yes|taskset)",
        "cargo_or_rustc": r"subprocess[^\n]{0,240}(?:cargo|rustc)",
        "foreign_control": r"(?i)(pkill|killall|renice|systemctl).*(nando|btop|k1)",
        "host_tuning": r"(?i)(scaling_governor|energy_performance_preference|intel_pstate).*(write|echo|set)",
    }
    matches = {name: bool(re.search(pattern, scanned_source)) for name, pattern in patterns.items()}
    require(not any(matches.values()), f"forbidden salvage source match: {matches}")
    return {"patterns": patterns, "matches": matches, "self_scan_bodies_excluded": excluded_count}


def parser_self_check() -> dict[str, Any]:
    evlist = (
        "task-clock:u: type: 1, config: 0x1, "
        "{ sample_period, sample_freq }: 100000, freq: 0, "
        "exclude_kernel: 1, precise_ip: 0"
    )
    require(parse_attr(evlist, "type") == 1, "fixture type parse failed")
    require(parse_attr(evlist, "config") == 1, "fixture config parse failed")
    require(parse_attr(evlist, "sample_period") == 100_000, "fixture period parse failed")
    require(parse_attr(evlist, "freq") == 0, "fixture freq parse failed")
    require(parse_attr(evlist, "exclude_kernel") == 1, "fixture privilege parse failed")
    require(parse_attr(evlist, "precise_ip") == 0, "fixture precision parse failed")
    samples = parse_perf_samples("000 task-clock:u: 7f1234567890 /usr/bin/yes\n")
    require(len(samples) == 1 and samples[0]["event"] == "task-clock:u", "sample fixture failed")
    return {"evlist_union_period": True, "sample_rows": 1}


def remote_run(expected_controller_sha256: str) -> None:
    require(os.geteuid() == 0, "remote salvage controller must run as root", ProvenanceError)
    controller = pathlib.Path(__file__).resolve()
    file_identity(controller, expected_controller_sha256)
    static_check = {"source_scan": source_scan(), "parser": parser_self_check()}
    require(not REMOTE_FINAL.exists(), "salvage remote final already exists", ProvenanceError)
    require(not REMOTE_STATE.exists(), "salvage remote state already exists", ProvenanceError)
    file_identity(PERF, EXPECTED["perf"])
    file_identity(YES, EXPECTED["yes"])
    file_identity(V1_REMOTE / "SHA256SUMS", EXPECTED["v1_manifest"])
    file_identity(V1_REMOTE / "PROBE_RECEIPT.json", EXPECTED["v1_receipt"])
    file_identity(V1_PERF_DATA, EXPECTED["perf_data"])
    file_identity(V1_MAPS_BEFORE, EXPECTED["maps_before"])
    file_identity(V1_MAPS_DURING, EXPECTED["maps_during"])
    verify_sha256sums(V1_REMOTE)
    before = host_projection()
    verify_host_projection(before)

    stage = pathlib.Path(f"{REMOTE_FINAL}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    (stage / "inputs").mkdir(mode=0o700)
    write_new_bytes(stage / "inputs/controller.py", controller.read_bytes(), 0o400)
    write_new_json(stage / "reader-commands.json", reader_commands(), 0o400)
    REMOTE_STATE.mkdir(parents=True, mode=0o700)
    (REMOTE_STATE / "markers").mkdir(mode=0o700)
    write_new_bytes(REMOTE_STATE / "route.lock", f"{TASK_ID}\n".encode(), 0o400)
    write_new_bytes(REMOTE_STATE / "markers/readers.available", b"available\n", 0o400)
    consumed = consume_marker(REMOTE_STATE)

    reader_invocations = 0
    error: str | None = None
    failure_verdict: str | None = None
    validation: dict[str, Any] | None = None
    try:
        outputs: dict[str, str] = {}
        for name, command in reader_commands():
            reader_invocations += 1
            outputs[name] = run_reader(name, command, stage)

        samples = parse_perf_samples(outputs["samples"])
        require(samples, "perf script emitted no sample IPs", CapabilityEvidenceError)
        require({sample["cpu"] for sample in samples} == {0}, "sample CPU mismatch", CapabilityEvidenceError)
        require(
            {sample["event"] for sample in samples} == {"task-clock:u"},
            "sample event mismatch",
            CapabilityEvidenceError,
        )
        evlist = outputs["evlist"]
        require(parse_attr(evlist, "type") == 1, "event type mismatch", CapabilityEvidenceError)
        require(parse_attr(evlist, "config") == 1, "event config mismatch", CapabilityEvidenceError)
        require(parse_attr(evlist, "sample_period") == 100_000, "sample period mismatch", CapabilityEvidenceError)
        require(parse_attr(evlist, "freq") == 0, "adaptive frequency observed", CapabilityEvidenceError)
        require(parse_attr(evlist, "exclude_kernel") == 1, "kernel samples not excluded", CapabilityEvidenceError)
        require(parse_attr(evlist, "precise_ip") == 0, "unexpected precise_ip", CapabilityEvidenceError)

        raw = outputs["raw-records"]
        lost_records = len(re.findall(r"PERF_RECORD_LOST(?:_SAMPLES)?\b", raw))
        throttle_records = len(re.findall(r"PERF_RECORD_THROTTLE\b", raw))
        unthrottle_records = len(re.findall(r"PERF_RECORD_UNTHROTTLE\b", raw))
        require(lost_records == 0, f"lost records observed: {lost_records}", CapabilityEvidenceError)
        require(throttle_records == 0, f"throttle records observed: {throttle_records}", CapabilityEvidenceError)
        require(unthrottle_records == 0, f"unthrottle records observed: {unthrottle_records}", CapabilityEvidenceError)
        build_id_pattern = rf"(?im)^{EXPECTED_YES_BUILD_ID}\s+{re.escape(str(YES))}$"
        require(
            re.search(build_id_pattern, outputs["buildids"]) is not None,
            "yes Build ID absent or mismatched",
            CapabilityEvidenceError,
        )
        mappings_before = parse_maps(V1_MAPS_BEFORE.read_text(), YES)
        mappings_during = parse_maps(V1_MAPS_DURING.read_text(), YES)
        require(mappings_before == mappings_during, "yes mappings changed", CapabilityEvidenceError)
        normalization = normalize_yes_ips(samples, V1_MAPS_DURING.read_text())
        validation = {
            "sample_count": len(samples),
            "sample_cpus": sorted({sample["cpu"] for sample in samples}),
            "sample_events": sorted({sample["event"] for sample in samples}),
            "event_type": 1,
            "event_config": 1,
            "sample_period": 100_000,
            "freq": 0,
            "exclude_kernel": 1,
            "precise_ip": 0,
            "lost_records": lost_records,
            "throttle_records": throttle_records,
            "unthrottle_records": unthrottle_records,
            "yes_build_id": EXPECTED_YES_BUILD_ID,
            "normalization": normalization,
        }
    except SalvageError as failure:
        error = f"{type(failure).__name__}: {failure}"
        failure_verdict = failure.verdict
    except Exception as failure:
        error = f"{type(failure).__name__}: {failure}"
        failure_verdict = "BLOCKED_CONTROLLER_PROTOCOL"

    try:
        after: dict[str, Any] = host_projection()
        host_stable = before == after
    except Exception as failure:
        after = {"projection_error": f"{type(failure).__name__}: {failure}"}
        host_stable = False
        if error is None:
            error = f"post-reader host projection failed: {after['projection_error']}"
            failure_verdict = "BLOCKED_PROVENANCE"
    if not host_stable and error is None:
        error = "host projection drift"
        failure_verdict = "BLOCKED_PROVENANCE"

    verdict = (
        "T_CAP_RECOVERED_FROM_SEALED_EVIDENCE"
        if error is None and validation is not None and reader_invocations == 4 and host_stable
        else (failure_verdict or "BLOCKED_TCAP_EVIDENCE")
    )
    preserved_controller = file_identity(stage / "inputs/controller.py")
    preserved_controller["path"] = str(REMOTE_FINAL / "inputs/controller.py")
    receipt = {
        "schema": "lay.v10.e1-traversal-d2-tcap-offline-salvage.v2",
        "task_id": TASK_ID,
        "verdict": verdict,
        "error": error,
        "controller": file_identity(controller),
        "preserved_controller": preserved_controller,
        "static_check": static_check,
        "v1_inputs": {
            "manifest": file_identity(V1_REMOTE / "SHA256SUMS", EXPECTED["v1_manifest"]),
            "receipt": file_identity(V1_REMOTE / "PROBE_RECEIPT.json", EXPECTED["v1_receipt"]),
            "perf_data": file_identity(V1_PERF_DATA, EXPECTED["perf_data"]),
            "maps_before": file_identity(V1_MAPS_BEFORE, EXPECTED["maps_before"]),
            "maps_during": file_identity(V1_MAPS_DURING, EXPECTED["maps_during"]),
        },
        "host_before": before,
        "host_after": after,
        "host_stable": host_stable,
        "reader_marker": str(consumed),
        "reader_marker_consumed_before_exec": consumed.is_file(),
        "reader_invocations": reader_invocations,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "event_open_invocations": 0,
        "yes_execution_invocations": 0,
        "d2_subject_executed": False,
        "validation": validation,
        "retry_permitted": False,
        "precise_probe_admitted": verdict == "T_CAP_RECOVERED_FROM_SEALED_EVIDENCE",
        "d2_implementation_admitted": False,
        "full_b_admitted": False,
        "v12_admitted": False,
        "runtime_authority_changed": False,
    }
    write_new_json(stage / "T_CAP_RECOVERY_RECEIPT.json", receipt, 0o400)
    seal_tree(stage)
    os.replace(stage, REMOTE_FINAL)
    print(json.dumps({"verdict": verdict, "receipt": str(REMOTE_FINAL / "T_CAP_RECOVERY_RECEIPT.json")}))


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
            ["pgrep", "-xo", name], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True
        )
        pids[name] = int(result.stdout.decode().strip())
    active = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
    return {"lay_version": version, "active_v11_sha256": sha256_file(active), "pids": pids}


def self_check() -> dict[str, Any]:
    file_identity(PREFLIGHT_MANIFEST, EXPECTED["preflight_manifest"])
    file_identity(PREFLIGHT_RECEIPT, EXPECTED["preflight_receipt"])
    file_identity(REPAIR_CONTRACT, EXPECTED["repair_contract"])
    file_identity(REPAIR_ROUTE_RECEIPT, EXPECTED["repair_route_receipt"])
    file_identity(V1_LOCAL / "SHA256SUMS", EXPECTED["v1_manifest"])
    file_identity(V1_LOCAL / "PROBE_RECEIPT.json", EXPECTED["v1_receipt"])
    file_identity(V1_LOCAL / "T-CAP/perf.data", EXPECTED["perf_data"])
    file_identity(V1_LOCAL / "T-CAP/maps-before.txt", EXPECTED["maps_before"])
    file_identity(V1_LOCAL / "T-CAP/maps-during.txt", EXPECTED["maps_during"])
    verify_sha256sums(V1_LOCAL)
    receipt = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(receipt.get("verdict") == "READY_TO_IMPLEMENT", "salvage preflight not ready")
    require(receipt.get("safe_to_implement") is True, "salvage preflight unsafe")
    scan = source_scan()
    parser = parser_self_check()
    with tempfile.TemporaryDirectory(prefix="lay-d2-tcap-salvage-self-check-") as temporary:
        root = pathlib.Path(temporary)
        (root / "markers").mkdir()
        write_new_bytes(root / "markers/readers.available", b"available\n", 0o400)
        consumed = consume_marker(root)
        require(consumed.is_file(), "marker self-check failed")
        require(not (root / "markers/readers.available").exists(), "available marker survived")
    result = {"verdict": "PASS_UNRUN", "source_scan": scan, "parser": parser}
    print(json.dumps(result))
    return result


def local_run() -> None:
    static_check = self_check()
    require(not LOCAL_RESULT_ROOT.exists(), "local salvage result already exists", ProvenanceError)
    before_runtime = local_runtime_projection()
    expected_runtime = {
        "lay_version": "lay 1.0.43",
        "active_v11_sha256": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
        "pids": {"ibus-daemon": 2076194, "lay-daemon": 3410795, "lay-ibus-engine": 3410820},
    }
    require(before_runtime == expected_runtime, f"local runtime baseline drift: {before_runtime}")
    controller_sha256 = sha256_file(pathlib.Path(__file__))
    status_code = (
        "import json,os,pathlib;"
        f"print(json.dumps({{'host':os.uname().nodename,'final':pathlib.Path('{REMOTE_FINAL}').exists(),"
        f"'state':pathlib.Path('{REMOTE_STATE}').exists()}}))"
    )
    status = json.loads(ssh(["/usr/bin/python3", "-c", status_code]).stdout)
    require(
        status == {"host": REMOTE_HOSTNAME, "final": False, "state": False},
        f"remote pre-run mismatch: {status}",
        ProvenanceError,
    )

    bootstrap = pathlib.Path(
        ssh(["mktemp", "-d", "/tmp/lay-d2-tcap-salvage.XXXXXX"]).stdout.decode().strip()
    )
    remote_controller = bootstrap / "controller.py"
    try:
        copy_result = subprocess.run(
            ["scp", "-q", str(pathlib.Path(__file__)), f"{REMOTE}:{remote_controller}"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        require(copy_result.returncode == 0, f"controller scp failed: {copy_result.stderr!r}")
        result = ssh(
            [
                "/usr/bin/sudo",
                "-n",
                "/usr/bin/python3",
                str(remote_controller),
                "remote-run",
                "--controller-sha256",
                controller_sha256,
            ],
            check=False,
        )
        require(result.returncode == 0, f"remote salvage publication failed: {result.stderr[-2000:]!r}")
        remote_summary = json.loads(result.stdout.decode().splitlines()[-1])
    finally:
        ssh(["rm", "-rf", "--", str(bootstrap)], check=False)

    LOCAL_RESULT_ROOT.mkdir(mode=0o775)
    temporary_evidence = LOCAL_RESULT_ROOT / f".remote-evidence.tmp-{os.getpid()}"
    temporary_evidence.mkdir(mode=0o700)
    copy_result = subprocess.run(
        ["scp", "-q", "-r", f"{REMOTE}:{REMOTE_FINAL}/.", str(temporary_evidence)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(copy_result.returncode == 0, f"evidence scp failed: {copy_result.stderr!r}")
    verify_sha256sums(temporary_evidence)
    temporary_evidence.rename(LOCAL_REMOTE_EVIDENCE)
    for path in sorted(LOCAL_REMOTE_EVIDENCE.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    LOCAL_REMOTE_EVIDENCE.chmod(0o555)
    verify_sha256sums(LOCAL_REMOTE_EVIDENCE)

    remote_receipt = json.loads((LOCAL_REMOTE_EVIDENCE / "T_CAP_RECOVERY_RECEIPT.json").read_text())
    require(remote_receipt["verdict"] == remote_summary["verdict"], "remote summary mismatch")
    after_runtime = local_runtime_projection()
    require(after_runtime == before_runtime, f"local runtime changed: {after_runtime}")
    local_receipt = dict(remote_receipt)
    local_receipt.update(
        {
            "schema": "lay.v10.e1-traversal-d2-tcap-offline-salvage-local.v2",
            "remote_evidence_path": str(LOCAL_REMOTE_EVIDENCE),
            "remote_evidence_manifest_sha256": sha256_file(LOCAL_REMOTE_EVIDENCE / "SHA256SUMS"),
            "preflight_manifest_sha256": EXPECTED["preflight_manifest"],
            "preflight_receipt_sha256": EXPECTED["preflight_receipt"],
            "repair_contract_sha256": EXPECTED["repair_contract"],
            "local_static_check": static_check,
            "local_runtime_before": before_runtime,
            "local_runtime_after": after_runtime,
            "runtime_authority_changed": False,
        }
    )
    write_new_json(LOCAL_RECEIPT, local_receipt, 0o400)
    print(json.dumps({"verdict": local_receipt["verdict"], "receipt": str(LOCAL_RECEIPT)}))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("self-check")
    subparsers.add_parser("run")
    remote_parser = subparsers.add_parser("remote-run")
    remote_parser.add_argument("--controller-sha256", required=True)
    return parser.parse_args()


def main() -> None:
    arguments = parse_args()
    if arguments.command == "self-check":
        self_check()
    elif arguments.command == "run":
        local_run()
    elif arguments.command == "remote-run":
        remote_run(arguments.controller_sha256)
    else:
        raise SalvageError(f"unknown command: {arguments.command}")


if __name__ == "__main__":
    try:
        main()
    except SalvageError as error:
        print(json.dumps({"verdict": error.verdict, "error": str(error)}), file=sys.stderr)
        raise SystemExit(1)
