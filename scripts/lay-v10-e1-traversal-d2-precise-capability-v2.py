#!/usr/bin/env python3

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import pathlib
import pwd
import re
import shlex
import signal
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
TASK_ID = "slice8b-v10-e1-traversal-d2-precise-capability-v2-20260825"
REMOTE_FINAL = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRECISE_CAPABILITY_V3_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRECISE_CAPABILITY_V3_PREFLIGHT_2026-08-25.json"
)
TCAP_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_INTERPRETATION_V3_2026-08-25/"
    "T_CAP_INTERPRETATION_RECEIPT.json"
)
P0_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/"
    "P0_STATIC_RECEIPT.json"
)
LOCAL_RESULT_ROOT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRECISE_CAPABILITY_V3_2026-08-25"
)
LOCAL_REMOTE_EVIDENCE = LOCAL_RESULT_ROOT / "REMOTE_EVIDENCE"
LOCAL_RECEIPT = LOCAL_RESULT_ROOT / "PRECISE_CAPABILITY_RECEIPT.json"

EXPECTED = {
    "preflight_manifest": "326e993bda1d5bbab7a34d767b2997f7029c879c9ea6fb7a95333ab750b2d79e",
    "preflight_receipt": "c16ac5eda55a7930cfe56e4776967078af019865cdd71a13a8983ed8cb534d38",
    "tcap_receipt": "f1d572a364312cc6c311ddc49316379b8b63748672c138f5fda50ba615cae2cb",
    "p0_receipt": "9b6aff3d74d36e7f5868fff596eb17326e819bf45496bbe4f3cf50870a7f71e0",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "perf_wrapper": "2d0953085bf720a25efbe24f853e97d27b1f12f18a398255ff82cbafde254dad",
    "perf_resolved": "b0741eb0e6e769ba9ee0ae4e27f0c60909b51be4bc560802aef4bcd91130692e",
    "yes": "ee431b97fb62f59ee94fa698dbc98971001bbb1cbd9c5e32ce4ab4c5530924d8",
    "taskset": "63e52c4b99a688ccd7bab6edbc6df2af1acad124eb852adcb4d20043d28eb2d3",
    "sudo": "1e000f41739201f030cdc588fbe50d5438570f5386104c9521543824827fb985",
}

PERF = pathlib.Path("/usr/lib/linux-tools/6.8.0-124-generic/perf")
YES = pathlib.Path("/usr/bin/yes")
TASKSET = pathlib.Path("/usr/bin/taskset")
EXPECTED_YES_BUILD_ID = "8c99ebc2c856857219acc612c2d9be3172b74be5"

SUBRUNS = (
    {
        "id": "I-CORE-CAP",
        "marker": "i-core",
        "cpu": 0,
        "event": "cpu_core/event=0xc0/upp",
        "period": 5_000_000,
        "type": 4,
        "config": 0xC0,
        "precise_ip": 2,
    },
    {
        "id": "I-ATOM-CAP",
        "marker": "i-atom",
        "cpu": 12,
        "event": "cpu_atom/event=0xc0/upp",
        "period": 5_000_000,
        "type": 10,
        "config": 0xC0,
        "precise_ip": 2,
    },
)


class ProbeError(RuntimeError):
    verdict = "BLOCKED_CAPABILITY"


class ProvenanceError(ProbeError):
    verdict = "BLOCKED_PROVENANCE"


class ControllerProtocolError(ProbeError):
    verdict = "BLOCKED_CONTROLLER_PROTOCOL"


class RecordError(ProbeError):
    verdict = "BLOCKED_RECORD"


class ReaderError(ProbeError):
    verdict = "BLOCKED_READER"


class CapabilityError(ProbeError):
    verdict = "BLOCKED_CAPABILITY"


def require(condition: bool, message: str, error_type: type[ProbeError] = ProbeError) -> None:
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
        raise ProbeError(f"command failed {result.returncode}: {command!r}: {result.stderr[-2000:]!r}")
    return result


def ssh(argv: list[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    command = " ".join(shlex.quote(item) for item in argv)
    return run(["ssh", "-o", "BatchMode=yes", REMOTE, command], check=check)


def load_json(path: pathlib.Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace").strip()


def exact_machine_id() -> str:
    return sha256_file(pathlib.Path("/etc/machine-id"))


def host_projection() -> dict[str, Any]:
    return {
        "hostname": os.uname().nodename,
        "machine_id_exact_file_sha256": exact_machine_id(),
        "kernel": os.uname().release,
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
        "cpu_core_type": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/type")),
        "cpu_core_cpus": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus")),
        "cpu_core_instructions": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/events/instructions")),
        "cpu_core_max_precise": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/caps/max_precise")),
        "cpu_atom_type": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/type")),
        "cpu_atom_cpus": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus")),
        "cpu_atom_instructions": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/events/instructions")),
        "cpu_atom_max_precise": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/caps/max_precise")),
        "perf_event_paranoid": read_text(pathlib.Path("/proc/sys/kernel/perf_event_paranoid")),
        "kptr_restrict": read_text(pathlib.Path("/proc/sys/kernel/kptr_restrict")),
        "perf_event_max_sample_rate": read_text(pathlib.Path("/proc/sys/kernel/perf_event_max_sample_rate")),
        "perf_wrapper_sha256": sha256_file(pathlib.Path("/usr/bin/perf")),
        "perf_resolved_sha256": sha256_file(PERF),
    }


def verify_host_projection(value: dict[str, Any]) -> None:
    expected = {
        "hostname": REMOTE_HOSTNAME,
        "machine_id_exact_file_sha256": REMOTE_MACHINE_ID_SHA256,
        "kernel": "6.8.0-124-generic",
        "online_cpus": "0-19",
        "cpu_core_type": "4",
        "cpu_core_cpus": "0-11",
        "cpu_core_instructions": "event=0xc0",
        "cpu_core_max_precise": "3",
        "cpu_atom_type": "10",
        "cpu_atom_cpus": "12-19",
        "cpu_atom_instructions": "event=0xc0",
        "cpu_atom_max_precise": "3",
        "perf_event_paranoid": "4",
        "kptr_restrict": "1",
        "perf_event_max_sample_rate": "8000",
        "perf_wrapper_sha256": EXPECTED["perf_wrapper"],
        "perf_resolved_sha256": EXPECTED["perf_resolved"],
    }
    require(value == expected, f"host projection mismatch: {value!r}", ProvenanceError)


def consume_marker(state: pathlib.Path, name: str) -> pathlib.Path:
    available = state / "markers" / f"{name}.available"
    consumed = state / "markers" / f"{name}.consumed-before-exec"
    require(available.is_file(), f"{name} marker unavailable", ProvenanceError)
    require(not consumed.exists(), f"{name} marker already consumed", ProvenanceError)
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
    require(mappings, f"no mappings for {expected_path}", CapabilityError)
    return mappings


def elf64_load_segments(path: pathlib.Path) -> tuple[int, list[dict[str, int]]]:
    data = path.read_bytes()
    require(data[:4] == b"\x7fELF", "benign subject is not ELF", ProvenanceError)
    require(data[4] == 2 and data[5] == 1, "benign subject is not little-endian ELF64", ProvenanceError)
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
    require(segments, "benign subject has no PT_LOAD", ProvenanceError)
    return elf_type, segments


def canonicalize_dso(value: str) -> str:
    require(value.startswith("(") and value.endswith(")"), f"DSO wrapper mismatch: {value!r}", CapabilityError)
    inner = value[1:-1]
    require(inner and "(" not in inner and ")" not in inner, f"nested DSO wrapper: {value!r}", CapabilityError)
    return inner


def parse_perf_samples(text: str) -> list[dict[str, Any]]:
    samples = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        match = re.fullmatch(r"\[(\d{3})\]\s+([^\s]+):\s+([0-9a-fA-F]+)\s+(\S+)", line.strip())
        require(match is not None, f"unparsed sample row {line_number}: {line!r}", CapabilityError)
        samples.append(
            {
                "cpu": int(match.group(1)),
                "event": match.group(2),
                "ip": int(match.group(3), 16),
                "dso": canonicalize_dso(match.group(4)),
            }
        )
    return samples


def normalize_yes_ips(samples: list[dict[str, Any]], maps_text: str) -> dict[str, Any]:
    mappings = parse_maps(maps_text, YES)
    elf_type, segments = elf64_load_segments(YES)
    require(elf_type == 3, "benign subject is not ET_DYN", CapabilityError)
    page_size = os.sysconf("SC_PAGE_SIZE")
    yes_samples = [sample for sample in samples if sample["dso"] == str(YES)]
    require(yes_samples, "no exact yes DSO samples", CapabilityError)
    normalized = []
    for sample in yes_samples:
        runtime_ip = sample["ip"]
        matching_mappings = [
            item for item in mappings if item["start"] <= runtime_ip < item["end"] and "x" in item["permissions"]
        ]
        require(len(matching_mappings) == 1, "sample IP executable mapping is ambiguous", CapabilityError)
        mapping = matching_mappings[0]
        matching_segments = [
            item
            for item in segments
            if item["flags"] & 1 and (item["offset"] // page_size) * page_size == mapping["offset"]
        ]
        require(len(matching_segments) == 1, "mapping executable PT_LOAD is ambiguous", CapabilityError)
        segment = matching_segments[0]
        load_bias = mapping["start"] - (segment["vaddr"] // page_size) * page_size
        normalized_ip = runtime_ip - load_bias
        require(
            segment["vaddr"] <= normalized_ip < segment["vaddr"] + segment["memsz"],
            "normalized IP outside executable PT_LOAD",
            CapabilityError,
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
        "unique_load_biases": sorted({item["load_bias"] for item in normalized}),
        "examples": normalized[:16],
    }


def parse_required_attr(text: str, key: str) -> int:
    match = re.search(rf"\b{re.escape(key)}\s*[:=]\s*(0x[0-9a-fA-F]+|\d+)", text)
    if match is None and key == "sample_period":
        match = re.search(
            r"\{\s*sample_period\s*,\s*sample_freq\s*\}\s*:\s*(0x[0-9a-fA-F]+|\d+)",
            text,
        )
    require(match is not None, f"perf evlist lacks {key}", CapabilityError)
    return int(match.group(1), 0)


def fixed_count_freq(text: str) -> int:
    match = re.search(r"\bfreq\s*[:=]\s*(0x[0-9a-fA-F]+|\d+)", text)
    return int(match.group(1), 0) if match is not None else 0


def parser_self_check() -> dict[str, Any]:
    evlist = (
        "cpu_core/event=0xc0/upp: type: 4, config: 0xc0, "
        "{ sample_period, sample_freq }: 5000000, exclude_kernel: 1, precise_ip: 2"
    )
    require(parse_required_attr(evlist, "type") == 4, "fixture type parse failed")
    require(parse_required_attr(evlist, "config") == 0xC0, "fixture config parse failed")
    require(parse_required_attr(evlist, "sample_period") == 5_000_000, "fixture period parse failed")
    require(fixed_count_freq(evlist) == 0, "fixture fixed-count freq failed")
    require(parse_required_attr(evlist, "exclude_kernel") == 1, "fixture privilege parse failed")
    require(parse_required_attr(evlist, "precise_ip") == 2, "fixture precision parse failed")
    samples = parse_perf_samples("[000] cpu_core/event=0xc0/upp: 7f1234567890 (/usr/bin/yes)\n")
    require(len(samples) == 1 and samples[0]["dso"] == str(YES), "fixture sample parse failed")
    require(controlled_shutdown_returncode(0), "zero shutdown fixture rejected")
    require(controlled_shutdown_returncode(-signal.SIGINT), "SIGINT shutdown fixture rejected")
    require(not controlled_shutdown_returncode(-signal.SIGTERM), "SIGTERM shutdown fixture accepted")
    return {"fixed_count_freq_default": 0, "sample_rows": 1, "accepted_shutdown_codes": [0, -2]}


def controlled_shutdown_returncode(returncode: int | None) -> bool:
    return returncode in {0, -signal.SIGINT}


def run_reader(command: list[str], root: pathlib.Path, name: str, ledger: dict[str, int]) -> str:
    write_new_json(root / f"{name}-command.json", command, 0o400)
    ledger["perf_data_reader_invocations"] += 1
    result = run(command, check=False)
    write_new_bytes(root / f"{name}.stdout", result.stdout, 0o400)
    write_new_bytes(root / f"{name}.stderr", result.stderr, 0o400)
    require(result.returncode == 0, f"reader {name} exited {result.returncode}", ReaderError)
    return result.stdout.decode("utf-8", errors="replace")


def drop_to_e() -> None:
    account = pwd.getpwnam("e")
    os.setgid(account.pw_gid)
    os.initgroups(account.pw_name, account.pw_gid)
    os.setuid(account.pw_uid)


def wait_for_yes_maps(process: subprocess.Popen[bytes]) -> str:
    deadline = time.monotonic() + 5.0
    maps_path = pathlib.Path(f"/proc/{process.pid}/maps")
    exe_path = pathlib.Path(f"/proc/{process.pid}/exe")
    while time.monotonic() < deadline:
        require(process.poll() is None, "benign subject exited before sampling", RecordError)
        with contextlib.suppress(FileNotFoundError, PermissionError):
            if exe_path.resolve() == YES and str(YES) in maps_path.read_text():
                return maps_path.read_text()
        time.sleep(0.01)
    raise RecordError("benign subject mapping did not become ready")


def stop_subject(process: subprocess.Popen[bytes]) -> tuple[int, bytes]:
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGTERM)
        try:
            _, stderr = process.communicate(timeout=3)
            return process.returncode, stderr
        except subprocess.TimeoutExpired:
            pass
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
    _, stderr = process.communicate()
    return process.returncode, stderr


def run_subrun(spec: dict[str, Any], stage: pathlib.Path, ledger: dict[str, int]) -> dict[str, Any]:
    root = stage / spec["id"]
    root.mkdir(mode=0o700)
    data_path = root / "perf.data"
    ledger["yes_execution_invocations"] += 1
    yes_process = subprocess.Popen(
        [str(TASKSET), "-c", str(spec["cpu"]), str(YES)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        preexec_fn=drop_to_e,
        start_new_session=True,
    )
    record_process: subprocess.Popen[bytes] | None = None
    shutdown = {
        "fixed_interval_seconds": 2.0,
        "fixed_interval_completed": False,
        "perf_alive_before_shutdown": False,
        "subject_alive_before_shutdown": False,
        "affinity_unchanged_before_shutdown": False,
        "shutdown_requested": False,
        "shutdown_signal": None,
        "shutdown_signal_count": 0,
        "returncode": None,
        "controlled_shutdown_valid": False,
    }
    maps_before = ""
    maps_during = ""
    maps_after = ""
    affinity_before: list[int] = []
    affinity_during: list[int] = []
    affinity_after: list[int] = []
    try:
        maps_before = wait_for_yes_maps(yes_process)
        write_new_bytes(root / "maps-before.txt", maps_before.encode(), 0o400)
        affinity_before = sorted(os.sched_getaffinity(yes_process.pid))
        require(affinity_before == [spec["cpu"]], f"subject affinity mismatch: {affinity_before}", ProvenanceError)
        record_command = [
            str(PERF),
            "record",
            "--buildid-all",
            "--sample-cpu",
            "--timestamp",
            "--event",
            spec["event"],
            "--count",
            str(spec["period"]),
            "--pid",
            str(yes_process.pid),
            "--output",
            str(data_path),
        ]
        write_new_json(root / "record-command.json", record_command, 0o400)
        ledger["perf_record_invocations"] += 1
        ledger["pmu_event_attempts"] += 1
        record_process = subprocess.Popen(
            record_command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        interval_started = time.monotonic()
        time.sleep(2.0)
        shutdown["fixed_interval_completed"] = time.monotonic() - interval_started >= 2.0
        shutdown["perf_alive_before_shutdown"] = record_process.poll() is None
        shutdown["subject_alive_before_shutdown"] = yes_process.poll() is None
        require(shutdown["fixed_interval_completed"], "fixed interval did not complete", ControllerProtocolError)
        require(shutdown["perf_alive_before_shutdown"], "perf exited before controller shutdown", RecordError)
        require(shutdown["subject_alive_before_shutdown"], "subject exited before controller shutdown", RecordError)
        maps_during = pathlib.Path(f"/proc/{yes_process.pid}/maps").read_text()
        write_new_bytes(root / "maps-during.txt", maps_during.encode(), 0o400)
        affinity_during = sorted(os.sched_getaffinity(yes_process.pid))
        shutdown["affinity_unchanged_before_shutdown"] = affinity_during == affinity_before == [spec["cpu"]]
        require(shutdown["affinity_unchanged_before_shutdown"], "subject affinity drift", ProvenanceError)
        shutdown["shutdown_requested"] = True
        shutdown["shutdown_signal"] = "SIGINT"
        shutdown["shutdown_signal_count"] = 1
        os.killpg(record_process.pid, signal.SIGINT)
        try:
            record_stdout, record_stderr = record_process.communicate(timeout=15)
        except subprocess.TimeoutExpired as failure:
            raise ControllerProtocolError("perf did not finalize after controller SIGINT") from failure
        shutdown["returncode"] = record_process.returncode
        shutdown["controlled_shutdown_valid"] = controlled_shutdown_returncode(record_process.returncode)
        write_new_bytes(root / "record-stdout.log", record_stdout, 0o400)
        write_new_bytes(root / "record-stderr.log", record_stderr, 0o400)
        write_new_json(root / "shutdown.json", shutdown, 0o400)
        require(
            shutdown["controlled_shutdown_valid"],
            f"unexpected controller-requested shutdown code: {record_process.returncode}",
            ControllerProtocolError,
        )
        require(yes_process.poll() is None, "subject exited before post-record snapshot", RecordError)
        maps_after = pathlib.Path(f"/proc/{yes_process.pid}/maps").read_text()
        write_new_bytes(root / "maps-after.txt", maps_after.encode(), 0o400)
        affinity_after = sorted(os.sched_getaffinity(yes_process.pid))
        require(affinity_after == affinity_during == affinity_before, "subject affinity changed", ProvenanceError)
    finally:
        if record_process is not None and record_process.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(record_process.pid, signal.SIGTERM)
            with contextlib.suppress(subprocess.TimeoutExpired):
                record_process.wait(timeout=3)
            if record_process.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(record_process.pid, signal.SIGKILL)
            cleanup_stdout, cleanup_stderr = record_process.communicate()
            if not (root / "record-stdout.log").exists():
                write_new_bytes(root / "record-stdout.log", cleanup_stdout, 0o400)
                write_new_bytes(root / "record-stderr.log", cleanup_stderr, 0o400)
        if not (root / "shutdown.json").exists():
            if record_process is not None:
                shutdown["returncode"] = record_process.returncode
            write_new_json(root / "shutdown.json", shutdown, 0o400)
        yes_returncode, yes_stderr = stop_subject(yes_process)
        write_new_json(root / "subject-stop.json", {"returncode": yes_returncode, "signal": "SIGTERM"}, 0o400)
        write_new_bytes(root / "yes-stderr.log", yes_stderr, 0o400)

    require(data_path.is_file() and data_path.stat().st_size > 0, "perf.data missing or empty", RecordError)
    evlist = run_reader([str(PERF), "evlist", "-v", "-i", str(data_path)], root, "evlist", ledger)
    samples_text = run_reader(
        [str(PERF), "script", "-i", str(data_path), "-F", "cpu,event,ip,dso"],
        root,
        "samples",
        ledger,
    )
    raw_text = run_reader([str(PERF), "script", "-D", "-i", str(data_path)], root, "raw-records", ledger)
    buildids = run_reader([str(PERF), "buildid-list", "-i", str(data_path)], root, "buildids", ledger)

    samples = parse_perf_samples(samples_text)
    require(samples, "perf script emitted no sample IPs", CapabilityError)
    require({sample["cpu"] for sample in samples} == {spec["cpu"]}, "sample CPU mismatch", CapabilityError)
    require({sample["event"] for sample in samples} == {spec["event"]}, "sample event mismatch", CapabilityError)
    require(parse_required_attr(evlist, "type") == spec["type"], "event type mismatch", CapabilityError)
    require(parse_required_attr(evlist, "config") == spec["config"], "event config mismatch", CapabilityError)
    require(parse_required_attr(evlist, "sample_period") == spec["period"], "sample period mismatch", CapabilityError)
    require(fixed_count_freq(evlist) == 0, "adaptive frequency observed", CapabilityError)
    require(parse_required_attr(evlist, "precise_ip") == spec["precise_ip"], "precise_ip mismatch", CapabilityError)
    require(parse_required_attr(evlist, "exclude_kernel") == 1, "kernel samples not excluded", CapabilityError)

    lost = len(re.findall(r"PERF_RECORD_LOST(?:_SAMPLES)?\b", raw_text))
    throttle = len(re.findall(r"PERF_RECORD_THROTTLE\b", raw_text))
    unthrottle = len(re.findall(r"PERF_RECORD_UNTHROTTLE\b", raw_text))
    require((lost, throttle, unthrottle) == (0, 0, 0), "lost or throttle records observed", CapabilityError)
    build_id_pattern = rf"(?m)^{EXPECTED_YES_BUILD_ID}\s+{re.escape(str(YES))}$"
    require(re.search(build_id_pattern, buildids) is not None, "yes Build ID mismatch", CapabilityError)
    mappings_before = parse_maps(maps_before, YES)
    mappings_during = parse_maps(maps_during, YES)
    mappings_after = parse_maps(maps_after, YES)
    require(mappings_before == mappings_during == mappings_after, "subject mappings changed", CapabilityError)
    normalization = normalize_yes_ips(samples, maps_during)
    require(normalization["normalized_count"] == normalization["yes_sample_count"], "normalization count mismatch")
    ledger["validated_pmu_events"] += 1
    return {
        "id": spec["id"],
        "verdict": "PASS",
        "cpu": spec["cpu"],
        "event": spec["event"],
        "period": spec["period"],
        "event_type": spec["type"],
        "event_config": spec["config"],
        "freq": 0,
        "precise_ip": spec["precise_ip"],
        "exclude_kernel": 1,
        "sample_count": len(samples),
        "sample_cpus": sorted({sample["cpu"] for sample in samples}),
        "sample_events": sorted({sample["event"] for sample in samples}),
        "affinity_before": affinity_before,
        "affinity_during": affinity_during,
        "affinity_after": affinity_after,
        "shutdown": shutdown,
        "lost_records": lost,
        "throttle_records": throttle,
        "unthrottle_records": unthrottle,
        "perf_data_sha256": sha256_file(data_path),
        "yes_build_id": EXPECTED_YES_BUILD_ID,
        "normalization": normalization,
        "perf_record_invocations": 1,
        "perf_data_reader_invocations": 4,
    }


def write_sha256sums(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        relative = path.relative_to(root)
        if relative == pathlib.Path("SHA256SUMS"):
            continue
        rows.append(f"{sha256_file(path)}  {relative}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode(), 0o400)


def verify_sha256sums(root: pathlib.Path) -> None:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing manifest: {manifest}", ProvenanceError)
    for line in manifest.read_text().splitlines():
        expected, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest path missing: {relative}", ProvenanceError)
        require(sha256_file(path) == expected, f"manifest mismatch: {relative}", ProvenanceError)


def seal_tree(root: pathlib.Path) -> None:
    write_sha256sums(root)
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)
    verify_sha256sums(root)


def source_scan() -> dict[str, Any]:
    source = pathlib.Path(__file__).read_text()
    scanned_source, excluded_count = re.subn(
        r"\ndef source_scan\(\) -> dict\[str, Any\]:\n.*?\n\ndef local_runtime_projection\(",
        "\ndef source_scan() -> dict[str, Any]:\n<SELF_SCAN_BODY_EXCLUDED>\n\ndef local_runtime_projection(",
        source,
        flags=re.DOTALL,
    )
    require(excluded_count == 1, "source-scan exclusion mismatch", ControllerProtocolError)
    patterns = {
        "software_tcap_event": r"task-clock|[\"']T-CAP[\"']",
        "perf_stat": r"[\"']stat[\"']|perf\s+stat",
        "third_precise_event": r"cpu_(?:core|atom)/event=(?!0xc0)",
        "cargo_or_rustc": r"subprocess[^\n]{0,240}(?:cargo|rustc)",
        "foreign_control": r"(?i)(pkill|killall|renice|systemctl).*(nando|btop|k1)",
        "host_tuning": r"(?i)(scaling_governor|energy_performance_preference|intel_pstate).*(write|echo|set)",
    }
    matches = {name: bool(re.search(pattern, scanned_source)) for name, pattern in patterns.items()}
    require(not any(matches.values()), f"forbidden precise-controller source match: {matches}", ControllerProtocolError)
    return {"patterns": patterns, "matches": matches, "self_scan_bodies_excluded": excluded_count}


def local_runtime_projection() -> dict[str, Any]:
    version = run([str(pathlib.Path.home() / ".local/bin/lay"), "--version"]).stdout.decode().strip()
    pids = {}
    for name in ("ibus-daemon", "lay-daemon", "lay-ibus-engine"):
        pids[name] = int(run(["pgrep", "-xo", name]).stdout.decode().strip())
    active = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
    return {"lay_version": version, "active_v11_sha256": sha256_file(active), "pids": pids}


def self_check() -> dict[str, Any]:
    file_identity(PREFLIGHT_MANIFEST, EXPECTED["preflight_manifest"])
    file_identity(PREFLIGHT_RECEIPT, EXPECTED["preflight_receipt"])
    file_identity(TCAP_RECEIPT, EXPECTED["tcap_receipt"])
    file_identity(P0_RECEIPT, EXPECTED["p0_receipt"])
    preflight = load_json(PREFLIGHT_RECEIPT)
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "precise preflight not ready", ProvenanceError)
    require(preflight.get("safe_to_implement") is True, "precise preflight unsafe", ProvenanceError)
    require(load_json(TCAP_RECEIPT).get("verdict") == "T_CAP_RECOVERED_FROM_SEALED_EVIDENCE", "T-CAP recovery absent", ProvenanceError)
    require([spec["id"] for spec in SUBRUNS] == ["I-CORE-CAP", "I-ATOM-CAP"], "precise route set drift")
    scan = source_scan()
    parser = parser_self_check()
    with tempfile.TemporaryDirectory(prefix="lay-d2-precise-self-check-") as temporary:
        root = pathlib.Path(temporary)
        (root / "markers").mkdir()
        write_new_bytes(root / "markers/i-core.available", b"available\n", 0o400)
        write_new_bytes(root / "markers/i-atom.available", b"available\n", 0o400)
        core = consume_marker(root, "i-core")
        require(core.is_file() and (root / "markers/i-atom.available").is_file(), "core marker fixture failed")
        atom = consume_marker(root, "i-atom")
        require(atom.is_file(), "atom marker fixture failed")
    result = {"verdict": "PASS_UNRUN", "source_scan": scan, "parser": parser, "routes": [spec["id"] for spec in SUBRUNS]}
    print(json.dumps(result, sort_keys=True))
    return result


def remote_run(expected_controller_sha256: str) -> None:
    require(os.geteuid() == 0, "remote precise controller must run as root", ProvenanceError)
    controller = pathlib.Path(__file__).resolve()
    file_identity(controller, expected_controller_sha256)
    static_check = {"source_scan": source_scan(), "parser": parser_self_check()}
    require(not REMOTE_FINAL.exists(), "remote precise final exists", ProvenanceError)
    require(not REMOTE_STATE.exists(), "remote precise state exists", ProvenanceError)
    file_identity(pathlib.Path("/usr/bin/perf"), EXPECTED["perf_wrapper"])
    file_identity(PERF, EXPECTED["perf_resolved"])
    file_identity(YES, EXPECTED["yes"])
    file_identity(TASKSET, EXPECTED["taskset"])
    file_identity(pathlib.Path("/usr/bin/sudo"), EXPECTED["sudo"])
    before = host_projection()
    verify_host_projection(before)

    stage = pathlib.Path(f"{REMOTE_FINAL}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    (stage / "inputs").mkdir(mode=0o700)
    write_new_bytes(stage / "inputs/controller.py", controller.read_bytes(), 0o400)
    preserved_controller = file_identity(stage / "inputs/controller.py")
    preserved_controller["path"] = str(REMOTE_FINAL / "inputs/controller.py")
    REMOTE_STATE.mkdir(parents=True, mode=0o700)
    (REMOTE_STATE / "markers").mkdir(mode=0o700)
    write_new_bytes(REMOTE_STATE / "route.lock", f"{TASK_ID}\n".encode(), 0o400)
    for marker in ("i-core", "i-atom"):
        write_new_bytes(REMOTE_STATE / "markers" / f"{marker}.available", b"available\n", 0o400)

    ledger = {
        "perf_record_invocations": 0,
        "perf_data_reader_invocations": 0,
        "pmu_event_attempts": 0,
        "validated_pmu_events": 0,
        "yes_execution_invocations": 0,
    }
    subruns = []
    error: str | None = None
    failure_verdict: str | None = None
    try:
        core_marker = consume_marker(REMOTE_STATE, "i-core")
        core_result = run_subrun(SUBRUNS[0], stage, ledger)
        subruns.append(core_result)
        require(core_result["verdict"] == "PASS", "I-CORE did not pass", CapabilityError)
        atom_marker = consume_marker(REMOTE_STATE, "i-atom")
        atom_result = run_subrun(SUBRUNS[1], stage, ledger)
        subruns.append(atom_result)
        require(atom_result["verdict"] == "PASS", "I-ATOM did not pass", CapabilityError)
    except ProbeError as failure:
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
            error = f"post-probe host projection failed: {after['projection_error']}"
            failure_verdict = "BLOCKED_PROVENANCE"
    if not host_stable and error is None:
        error = "host projection drift"
        failure_verdict = "BLOCKED_PROVENANCE"
    verdict = (
        "D2_CAPABILITY_READY"
        if error is None and len(subruns) == 2 and ledger["validated_pmu_events"] == 2 and host_stable
        else (failure_verdict or "BLOCKED_CAPABILITY")
    )
    marker_names = sorted(path.name for path in (REMOTE_STATE / "markers").iterdir())
    receipt = {
        "schema": "lay.v10.e1-traversal-d2-precise-capability.v2",
        "task_id": TASK_ID,
        "verdict": verdict,
        "error": error,
        "controller": file_identity(controller),
        "preserved_controller": preserved_controller,
        "static_check": static_check,
        "host_before": before,
        "host_after": after,
        "host_stable": host_stable,
        "markers": marker_names,
        "i_core_consumed_before_exec": "i-core.consumed-before-exec" in marker_names,
        "i_atom_consumed_only_after_core_pass": "i-atom.consumed-before-exec" in marker_names and len(subruns) >= 1,
        "subruns_required": [spec["id"] for spec in SUBRUNS],
        "subruns": subruns,
        "execution_ledger": ledger,
        "tcap_record_invocations": 0,
        "tcap_reader_invocations": 0,
        "software_events_opened": 0,
        "perf_stat_invocations": 0,
        "benign_subject": file_identity(YES, EXPECTED["yes"]),
        "yes_build_id": EXPECTED_YES_BUILD_ID,
        "retry_permitted": False,
        "d2_final_implementation_preflight_may_be_created": verdict == "D2_CAPABILITY_READY",
        "d2_implementation_admitted": False,
        "d2_elf_built": False,
        "d2_subject_executed": False,
        "cargo_build_check_test": 0,
        "rustc_compilation": 0,
        "foreign_process_control": False,
        "host_policy_tuning": False,
        "full_b_admitted": False,
        "v12_admitted": False,
        "runtime_integration_admitted": False,
        "runtime_authority_changed": False,
    }
    write_new_json(stage / "PRECISE_CAPABILITY_RECEIPT.json", receipt, 0o400)
    seal_tree(stage)
    os.replace(stage, REMOTE_FINAL)
    print(json.dumps({"verdict": verdict, "receipt": str(REMOTE_FINAL / "PRECISE_CAPABILITY_RECEIPT.json")}, sort_keys=True))


def local_run() -> None:
    local_static_check = self_check()
    require(not LOCAL_RESULT_ROOT.exists(), "local precise result exists", ProvenanceError)
    before_runtime = local_runtime_projection()
    expected_runtime = {
        "lay_version": "lay 1.0.43",
        "active_v11_sha256": EXPECTED["active_v11"],
        "pids": {"ibus-daemon": 2076194, "lay-daemon": 3410795, "lay-ibus-engine": 3410820},
    }
    require(before_runtime == expected_runtime, f"local runtime drift: {before_runtime}", ProvenanceError)
    controller_sha256 = sha256_file(pathlib.Path(__file__))
    status_code = (
        "import hashlib,json,os,pathlib;"
        "h=hashlib.sha256(pathlib.Path('/etc/machine-id').read_bytes()).hexdigest();"
        f"print(json.dumps({{'host':os.uname().nodename,'machine':h,'final':pathlib.Path('{REMOTE_FINAL}').exists(),'state':pathlib.Path('{REMOTE_STATE}').exists()}}))"
    )
    status = json.loads(ssh(["/usr/bin/python3", "-c", status_code]).stdout)
    require(
        status == {"host": REMOTE_HOSTNAME, "machine": REMOTE_MACHINE_ID_SHA256, "final": False, "state": False},
        f"remote pre-run mismatch: {status}",
        ProvenanceError,
    )
    bootstrap = pathlib.Path(ssh(["mktemp", "-d", "/tmp/lay-d2-precise.XXXXXX"]).stdout.decode().strip())
    remote_controller = bootstrap / "controller.py"
    try:
        run(["scp", "-q", str(pathlib.Path(__file__)), f"{REMOTE}:{remote_controller}"])
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
        require(result.returncode == 0, f"remote precise publication failed: {result.stderr[-2000:]!r}")
        remote_summary = json.loads(result.stdout.decode().splitlines()[-1])
    finally:
        ssh(["rm", "-rf", "--", str(bootstrap)], check=False)

    LOCAL_RESULT_ROOT.mkdir(mode=0o775)
    copy_stage = LOCAL_RESULT_ROOT / f".remote-evidence.stage-{os.getpid()}"
    copy_stage.mkdir(mode=0o700)
    copy_result = run(["scp", "-q", "-r", f"{REMOTE}:{REMOTE_FINAL}/.", str(copy_stage)], check=False)
    require(copy_result.returncode == 0, f"remote evidence copy failed: {copy_result.stderr[-2000:]!r}")
    verify_sha256sums(copy_stage)
    copy_stage.rename(LOCAL_REMOTE_EVIDENCE)
    for path in sorted(LOCAL_REMOTE_EVIDENCE.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    LOCAL_REMOTE_EVIDENCE.chmod(0o555)
    verify_sha256sums(LOCAL_REMOTE_EVIDENCE)

    remote_receipt = load_json(LOCAL_REMOTE_EVIDENCE / "PRECISE_CAPABILITY_RECEIPT.json")
    require(remote_receipt["verdict"] == remote_summary["verdict"], "remote summary mismatch", ProvenanceError)
    after_runtime = local_runtime_projection()
    require(after_runtime == before_runtime, f"local runtime changed: {after_runtime}", ProvenanceError)
    local_receipt = dict(remote_receipt)
    local_receipt.update(
        {
            "schema": "lay.v10.e1-traversal-d2-precise-capability-local.v2",
            "remote_evidence_path": str(LOCAL_REMOTE_EVIDENCE),
            "remote_evidence_manifest_sha256": sha256_file(LOCAL_REMOTE_EVIDENCE / "SHA256SUMS"),
            "preflight_manifest_sha256": EXPECTED["preflight_manifest"],
            "preflight_receipt_sha256": EXPECTED["preflight_receipt"],
            "tcap_receipt_sha256": EXPECTED["tcap_receipt"],
            "local_static_check": local_static_check,
            "local_runtime_before": before_runtime,
            "local_runtime_after": after_runtime,
            "runtime_authority_changed": False,
        }
    )
    write_new_json(LOCAL_RECEIPT, local_receipt, 0o400)
    print(json.dumps({"verdict": local_receipt["verdict"], "receipt": str(LOCAL_RECEIPT)}, sort_keys=True))


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
        raise ControllerProtocolError(f"unknown command: {arguments.command}")


if __name__ == "__main__":
    try:
        main()
    except ProbeError as error:
        print(json.dumps({"verdict": error.verdict, "error": str(error)}), file=sys.stderr)
        raise SystemExit(1)
