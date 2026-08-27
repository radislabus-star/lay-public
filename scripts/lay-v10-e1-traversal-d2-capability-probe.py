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
import shutil
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
TASK_ID = "slice8b-v10-e1-traversal-d2-capability-probe-20260825"
REMOTE_FINAL = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_CAPABILITY_PROBE_V2_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_CAPABILITY_PROBE_V2_PREFLIGHT_2026-08-25.json"
)
P0_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/"
    "P0_STATIC_RECEIPT.json"
)
LOCAL_RESULT_ROOT = P0_RECEIPT.parent
LOCAL_REMOTE_EVIDENCE = LOCAL_RESULT_ROOT / "P1_REMOTE_EVIDENCE"
LOCAL_RECEIPT = LOCAL_RESULT_ROOT / "P1_CAPABILITY_PROBE_RECEIPT.json"

EXPECTED = {
    "preflight_manifest": "3ba58e28846e15b53b0e8fd73a6ced836b15293105b36f3e7d435406ac126188",
    "preflight_receipt": "71d98b03f63ed99118f22e2334bb6a0717c68eae1d75110faa08c35de502b399",
    "p0_receipt": "9b6aff3d74d36e7f5868fff596eb17326e819bf45496bbe4f3cf50870a7f71e0",
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
        "id": "T-CAP",
        "cpu": 0,
        "event": "task-clock:u",
        "period": 100_000,
        "type": 1,
        "config": 1,
        "precise_ip": 0,
    },
    {
        "id": "I-CORE-CAP",
        "cpu": 0,
        "event": "cpu_core/event=0xc0/upp",
        "period": 5_000_000,
        "type": 4,
        "config": 0xC0,
        "precise_ip": 2,
    },
    {
        "id": "I-ATOM-CAP",
        "cpu": 12,
        "event": "cpu_atom/event=0xc0/upp",
        "period": 5_000_000,
        "type": 10,
        "config": 0xC0,
        "precise_ip": 2,
    },
)


class ProbeError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ProbeError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def file_identity(path: pathlib.Path, expected_sha256: str | None = None) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
    identity = {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }
    if expected_sha256 is not None:
        require(identity["sha256"] == expected_sha256, f"SHA-256 mismatch: {path}")
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
        raise ProbeError(
            f"command failed {result.returncode}: {command!r}: {result.stderr[-2000:]!r}"
        )
    return result


def ssh(argv: list[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    command = " ".join(shlex.quote(item) for item in argv)
    return run(["ssh", "-o", "BatchMode=yes", REMOTE, command], check=check)


def load_json(path: pathlib.Path) -> Any:
    with path.open("r", encoding="utf-8") as source:
        return json.load(source)


def exact_machine_id() -> str:
    return sha256_file(pathlib.Path("/etc/machine-id"))


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace").strip()


def host_projection() -> dict[str, Any]:
    return {
        "hostname": os.uname().nodename,
        "machine_id_exact_file_sha256": exact_machine_id(),
        "kernel": os.uname().release,
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
        "cpu_core_type": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/type")),
        "cpu_core_cpus": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus")),
        "cpu_core_instructions": read_text(
            pathlib.Path("/sys/bus/event_source/devices/cpu_core/events/instructions")
        ),
        "cpu_core_max_precise": read_text(
            pathlib.Path("/sys/bus/event_source/devices/cpu_core/caps/max_precise")
        ),
        "cpu_atom_type": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/type")),
        "cpu_atom_cpus": read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus")),
        "cpu_atom_instructions": read_text(
            pathlib.Path("/sys/bus/event_source/devices/cpu_atom/events/instructions")
        ),
        "cpu_atom_max_precise": read_text(
            pathlib.Path("/sys/bus/event_source/devices/cpu_atom/caps/max_precise")
        ),
        "perf_event_paranoid": read_text(pathlib.Path("/proc/sys/kernel/perf_event_paranoid")),
        "kptr_restrict": read_text(pathlib.Path("/proc/sys/kernel/kptr_restrict")),
        "perf_event_max_sample_rate": read_text(
            pathlib.Path("/proc/sys/kernel/perf_event_max_sample_rate")
        ),
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
    require(value == expected, f"host projection mismatch: {value!r}")


def consume_marker(state: pathlib.Path) -> pathlib.Path:
    available = state / "markers/probe.available"
    consumed = state / "markers/probe.consumed-before-exec"
    require(available.is_file(), "probe marker unavailable")
    require(not consumed.exists(), "probe marker already consumed")
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
    require(mappings, f"no mappings for {expected_path}")
    return mappings


def elf64_load_segments(path: pathlib.Path) -> tuple[int, list[dict[str, int]]]:
    data = path.read_bytes()
    require(data[:4] == b"\x7fELF", "not ELF")
    require(data[4] == 2 and data[5] == 1, "requires little-endian ELF64")
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, 0)
    elf_type = header[1]
    program_offset = header[5]
    program_entry_size = header[9]
    program_count = header[10]
    require(program_entry_size == 56, "unexpected ELF64 program-header size")
    segments = []
    for index in range(program_count):
        offset = program_offset + index * program_entry_size
        values = struct.unpack_from("<IIQQQQQQ", data, offset)
        if values[0] != 1:
            continue
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
    require(segments, "ELF has no PT_LOAD segments")
    return elf_type, segments


def normalize_yes_ips(
    samples: list[dict[str, Any]], maps_text: str
) -> dict[str, Any]:
    mappings = parse_maps(maps_text, YES)
    elf_type, segments = elf64_load_segments(YES)
    require(elf_type == 3, "benign normalization subject is not ET_DYN")
    page_size = os.sysconf("SC_PAGE_SIZE")
    normalized = []
    yes_samples = [sample for sample in samples if sample["dso"].endswith("/yes")]
    require(yes_samples, "no sampled yes DSO IPs")
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
        )
        segment = matching_segments[0]
        aligned_vaddr = segment["vaddr"] // page_size * page_size
        load_bias = mapping["start"] - aligned_vaddr
        normalized_ip = runtime_ip - load_bias
        require(
            segment["vaddr"] <= normalized_ip < segment["vaddr"] + segment["memsz"],
            f"normalized IP outside executable PT_LOAD: {normalized_ip:#x}",
        )
        normalized.append(
            {
                "runtime_ip": f"0x{runtime_ip:x}",
                "normalized_ip": f"0x{normalized_ip:x}",
                "mapping_start": f"0x{mapping['start']:x}",
                "mapping_offset": f"0x{mapping['offset']:x}",
                "pt_load_vaddr": f"0x{segment['vaddr']:x}",
                "load_bias": f"0x{load_bias:x}",
            }
        )
    return {
        "elf_type": "ET_DYN",
        "page_size": page_size,
        "yes_sample_count": len(yes_samples),
        "normalized_count": len(normalized),
        "examples": normalized[:16],
    }


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
    require(match is not None, f"perf evlist lacks {key}")
    return int(match.group(1), 0)


def parser_self_check() -> dict[str, Any]:
    evlist = (
        "cpu_core/event=0xc0/upp: type: 4, config: 0xc0, "
        "{ sample_period, sample_freq }: 5000000, freq: 0, "
        "exclude_kernel: 1, precise_ip: 2"
    )
    require(parse_attr(evlist, "type") == 4, "fixture type parse failed")
    require(parse_attr(evlist, "config") == 0xC0, "fixture config parse failed")
    require(parse_attr(evlist, "sample_period") == 5_000_000, "fixture period parse failed")
    require(parse_attr(evlist, "freq") == 0, "fixture freq parse failed")
    require(parse_attr(evlist, "exclude_kernel") == 1, "fixture privilege parse failed")
    require(parse_attr(evlist, "precise_ip") == 2, "fixture precision parse failed")
    sample_text = (
        "000 cpu_core/event=0xc0/upp: 7f1234567890 /usr/bin/yes\n"
        "[000] cpu_core/event=0xc0/upp: 7f1234567891 /usr/bin/yes\n"
    )
    samples = parse_perf_samples(sample_text)
    require(len(samples) == 2, "fixture sample count mismatch")
    require({sample["cpu"] for sample in samples} == {0}, "fixture CPU parse failed")
    require(
        {sample["event"] for sample in samples} == {"cpu_core/event=0xc0/upp"},
        "fixture event parse failed",
    )
    require({sample["dso"] for sample in samples} == {"/usr/bin/yes"}, "fixture DSO parse failed")
    return {"evlist_union_period": True, "sample_rows": len(samples)}


def run_reader(command: list[str], output: pathlib.Path) -> str:
    result = run(command)
    write_new_bytes(output, result.stdout)
    if result.stderr:
        write_new_bytes(output.with_suffix(output.suffix + ".stderr"), result.stderr)
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
        require(process.poll() is None, "benign yes process exited before sampling")
        with contextlib.suppress(FileNotFoundError, PermissionError):
            if exe_path.resolve() == YES and str(YES) in maps_path.read_text():
                return maps_path.read_text()
        time.sleep(0.01)
    raise ProbeError("benign yes mapping did not become ready")


def stop_process_group(process: subprocess.Popen[bytes]) -> bytes:
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGTERM)
        with contextlib.suppress(subprocess.TimeoutExpired):
            _, stderr = process.communicate(timeout=3)
            return stderr
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
    _, stderr = process.communicate()
    return stderr


def run_subrun(spec: dict[str, Any], stage: pathlib.Path) -> dict[str, Any]:
    root = stage / spec["id"]
    root.mkdir(mode=0o700)
    data_path = root / "perf.data"
    yes_process = subprocess.Popen(
        [str(TASKSET), "-c", str(spec["cpu"]), str(YES)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        preexec_fn=drop_to_e,
        start_new_session=True,
    )
    record_process: subprocess.Popen[bytes] | None = None
    yes_stderr = b""
    try:
        maps_before = wait_for_yes_maps(yes_process)
        write_new_bytes(root / "maps-before.txt", maps_before.encode())
        affinity_before = sorted(os.sched_getaffinity(yes_process.pid))
        require(affinity_before == [spec["cpu"]], f"benign PID affinity mismatch: {affinity_before}")
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
        write_new_json(root / "record-command.json", record_command)
        record_process = subprocess.Popen(
            record_command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        time.sleep(2.0)
        require(record_process.poll() is None, "perf record exited before fixed probe duration")
        maps_during = pathlib.Path(f"/proc/{yes_process.pid}/maps").read_text()
        write_new_bytes(root / "maps-during.txt", maps_during.encode())
        affinity_during = sorted(os.sched_getaffinity(yes_process.pid))
        require(affinity_during == [spec["cpu"]], f"benign PID affinity drift: {affinity_during}")
        os.killpg(record_process.pid, signal.SIGINT)
        record_stdout, record_stderr = record_process.communicate(timeout=15)
        write_new_bytes(root / "record-stdout.log", record_stdout)
        write_new_bytes(root / "record-stderr.log", record_stderr)
        require(record_process.returncode == 0, f"perf record exited {record_process.returncode}")
        maps_after = pathlib.Path(f"/proc/{yes_process.pid}/maps").read_text()
        write_new_bytes(root / "maps-after.txt", maps_after.encode())
        affinity_after = sorted(os.sched_getaffinity(yes_process.pid))
        require(affinity_after == [spec["cpu"]], f"benign PID affinity drift: {affinity_after}")
    finally:
        if record_process is not None and not (root / "record-stdout.log").exists():
            if record_process.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(record_process.pid, signal.SIGTERM)
                with contextlib.suppress(subprocess.TimeoutExpired):
                    record_process.wait(timeout=3)
                if record_process.poll() is None:
                    with contextlib.suppress(ProcessLookupError):
                        os.killpg(record_process.pid, signal.SIGKILL)
            cleanup_stdout, cleanup_stderr = record_process.communicate()
            write_new_bytes(root / "record-stdout.log", cleanup_stdout)
            write_new_bytes(root / "record-stderr.log", cleanup_stderr)
        yes_stderr = stop_process_group(yes_process)
        write_new_bytes(root / "yes-stderr.log", yes_stderr)

    require(data_path.is_file() and data_path.stat().st_size > 0, "perf.data missing or empty")
    evlist = run_reader([str(PERF), "evlist", "-v", "-i", str(data_path)], root / "evlist.txt")
    sample_text = run_reader(
        [str(PERF), "script", "-i", str(data_path), "-F", "cpu,event,ip,dso"],
        root / "samples.txt",
    )
    raw_text = run_reader([str(PERF), "script", "-D", "-i", str(data_path)], root / "raw-records.txt")
    buildids = run_reader([str(PERF), "buildid-list", "-i", str(data_path)], root / "buildids.txt")

    samples = parse_perf_samples(sample_text)
    require(samples, "perf script emitted no sample IPs")
    require({sample["cpu"] for sample in samples} == {spec["cpu"]}, "sample CPU mismatch")
    require({sample["event"] for sample in samples} == {spec["event"]}, "sample event mismatch")
    require(parse_attr(evlist, "type") == spec["type"], "perf event type mismatch")
    require(parse_attr(evlist, "config") == spec["config"], "perf event config mismatch")
    require(parse_attr(evlist, "sample_period") == spec["period"], "sample period mismatch")
    require(parse_attr(evlist, "freq") == 0, "adaptive frequency observed")
    require(parse_attr(evlist, "precise_ip") == spec["precise_ip"], "precise_ip mismatch")
    require(parse_attr(evlist, "exclude_kernel") == 1, "kernel samples are not excluded")

    lost_records = len(re.findall(r"PERF_RECORD_LOST(?:_SAMPLES)?\b", raw_text))
    throttle_records = len(re.findall(r"PERF_RECORD_THROTTLE\b", raw_text))
    unthrottle_records = len(re.findall(r"PERF_RECORD_UNTHROTTLE\b", raw_text))
    require(lost_records == 0, f"lost records observed: {lost_records}")
    require(throttle_records == 0, f"throttle records observed: {throttle_records}")
    require(unthrottle_records == 0, f"unthrottle records observed: {unthrottle_records}")
    build_id_pattern = rf"(?im)^{EXPECTED_YES_BUILD_ID}\s+{re.escape(str(YES))}$"
    require(re.search(build_id_pattern, buildids) is not None, "yes Build ID absent or mismatched")
    mappings_before = parse_maps(maps_before, YES)
    mappings_during = parse_maps(maps_during, YES)
    mappings_after = parse_maps(maps_after, YES)
    require(
        mappings_before == mappings_during == mappings_after,
        "benign executable mappings changed during probe",
    )
    normalization = normalize_yes_ips(samples, maps_during)
    return {
        "id": spec["id"],
        "verdict": "PASS",
        "cpu": spec["cpu"],
        "event": spec["event"],
        "period": spec["period"],
        "event_type": spec["type"],
        "event_config": spec["config"],
        "precise_ip": spec["precise_ip"],
        "sample_count": len(samples),
        "sample_cpus": sorted({sample["cpu"] for sample in samples}),
        "sample_events": sorted({sample["event"] for sample in samples}),
        "pid_affinity_before": affinity_before,
        "pid_affinity_during": affinity_during,
        "pid_affinity_after": affinity_after,
        "lost_records": lost_records,
        "throttle_records": throttle_records,
        "unthrottle_records": unthrottle_records,
        "perf_data_sha256": sha256_file(data_path),
        "evlist_sha256": sha256_file(root / "evlist.txt"),
        "samples_sha256": sha256_file(root / "samples.txt"),
        "raw_records_sha256": sha256_file(root / "raw-records.txt"),
        "buildids_sha256": sha256_file(root / "buildids.txt"),
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
    require(manifest.is_file(), f"missing manifest: {manifest}")
    for line in manifest.read_text().splitlines():
        expected, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest path missing: {relative}")
        require(sha256_file(path) == expected, f"manifest mismatch: {relative}")


def seal_tree(root: pathlib.Path) -> None:
    write_sha256sums(root)
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)
    verify_sha256sums(root)


def remote_run(expected_controller_sha256: str) -> None:
    require(os.geteuid() == 0, "remote probe controller must run as root")
    controller = pathlib.Path(__file__).resolve()
    file_identity(controller, expected_controller_sha256)
    remote_static_check = {
        "source_scan": source_scan(),
        "parser": parser_self_check(),
    }
    require(os.uname().nodename == REMOTE_HOSTNAME, "wrong probe host")
    require(exact_machine_id() == REMOTE_MACHINE_ID_SHA256, "wrong probe machine")
    require(not REMOTE_FINAL.exists(), "remote probe final already exists")
    require(not REMOTE_STATE.exists(), "remote probe state already exists")
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
    write_new_bytes(REMOTE_STATE / "markers/probe.available", b"available\n", 0o400)
    consumed = consume_marker(REMOTE_STATE)
    subruns = []
    error: str | None = None
    try:
        for spec in SUBRUNS:
            subruns.append(run_subrun(spec, stage))
    except Exception as failure:
        error = f"{type(failure).__name__}: {failure}"

    try:
        after: dict[str, Any] = host_projection()
        host_stable = before == after
    except Exception as failure:
        after = {"projection_error": f"{type(failure).__name__}: {failure}"}
        host_stable = False
        if error is None:
            error = f"post-run host projection failed: {after['projection_error']}"
    verdict = "PASS" if error is None and len(subruns) == len(SUBRUNS) and host_stable else "BLOCKED_CAPABILITY"
    if not host_stable and error is None:
        error = "host projection drift"
    receipt = {
        "schema": "lay.v10.e1-traversal-d2-benign-capability-probe.v1",
        "task_id": TASK_ID,
        "verdict": verdict,
        "error": error,
        "controller": file_identity(controller),
        "preserved_controller": preserved_controller,
        "remote_static_check": remote_static_check,
        "host_before": before,
        "host_after": after,
        "host_stable": host_stable,
        "probe_marker": str(consumed),
        "probe_marker_consumed_before_exec": consumed.is_file(),
        "retry_permitted": False,
        "subruns_required": [spec["id"] for spec in SUBRUNS],
        "subruns": subruns,
        "perf_record_invocations": sum(item["perf_record_invocations"] for item in subruns),
        "perf_stat_invocations": 0,
        "perf_data_reader_invocations": sum(
            item["perf_data_reader_invocations"] for item in subruns
        ),
        "pmu_events_opened": sum(1 for item in subruns if item["event_type"] in {4, 10}),
        "software_events_opened": sum(1 for item in subruns if item["event_type"] == 1),
        "benign_subject": file_identity(YES, EXPECTED["yes"]),
        "yes_build_id": EXPECTED_YES_BUILD_ID,
        "d2_elf_built": False,
        "d2_subject_executed": False,
        "v10_v13_package_sidecar_schedule_loaded": False,
        "cargo_build_check_test": 0,
        "rustc_compilation": 0,
        "foreign_process_control": False,
        "host_policy_tuning": False,
        "full_b_admitted": False,
        "v12_admitted": False,
        "runtime_integration_admitted": False,
        "runtime_authority_changed": False,
        "final_d2_implementation_ready": False,
    }
    write_new_json(stage / "PROBE_RECEIPT.json", receipt, 0o400)
    seal_tree(stage)
    os.replace(stage, REMOTE_FINAL)
    print(json.dumps({"verdict": verdict, "receipt": str(REMOTE_FINAL / "PROBE_RECEIPT.json")}))


def source_scan() -> dict[str, Any]:
    source = pathlib.Path(__file__).read_text()
    scanned_source, excluded_count = re.subn(
        r"\ndef source_scan\(\) -> dict\[str, Any\]:\n.*?\n\ndef local_runtime_projection\(",
        "\ndef source_scan() -> dict[str, Any]:\n<SELF_SCAN_BODY_EXCLUDED>\n\ndef local_runtime_projection(",
        source,
        flags=re.DOTALL,
    )
    require(excluded_count == 1, f"source-scan exclusion count mismatch: {excluded_count}")
    patterns = {
        "network": r"urllib\.|requests\.|https?://|/usr/bin/(curl|wget)",
        "perf_stat": r"[\"']stat[\"']|perf\s+stat",
        "cargo_or_rustc": r"subprocess[^\n]{0,200}(cargo|rustc)",
        "foreign_control": r"(?i)(pkill|killall|renice|systemctl).*(nando|btop|k1)",
        "host_tuning": r"(?i)(scaling_governor|energy_performance_preference|intel_pstate).*(write|echo|set)",
    }
    matches = {
        name: bool(re.search(pattern, scanned_source)) for name, pattern in patterns.items()
    }
    require(not any(matches.values()), f"forbidden probe-controller source match: {matches}")
    return {
        "patterns": patterns,
        "matches": matches,
        "self_scan_bodies_excluded": excluded_count,
    }


def local_runtime_projection() -> dict[str, Any]:
    version = run([str(pathlib.Path.home() / ".local/bin/lay"), "--version"]).stdout.decode().strip()
    pids = {}
    for name in ("ibus-daemon", "lay-daemon", "lay-ibus-engine"):
        result = run(["pgrep", "-xo", name])
        pids[name] = int(result.stdout.decode().strip())
    active = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
    return {"lay_version": version, "active_v11_sha256": sha256_file(active), "pids": pids}


def self_check() -> dict[str, Any]:
    file_identity(PREFLIGHT_MANIFEST, EXPECTED["preflight_manifest"])
    file_identity(PREFLIGHT_RECEIPT, EXPECTED["preflight_receipt"])
    file_identity(P0_RECEIPT, EXPECTED["p0_receipt"])
    receipt = load_json(PREFLIGHT_RECEIPT)
    require(receipt.get("verdict") == "READY_TO_IMPLEMENT", "probe preflight is not ready")
    require(receipt.get("safe_to_implement") is True, "probe preflight is unsafe")
    scan = source_scan()
    parser = parser_self_check()
    with tempfile.TemporaryDirectory(prefix="lay-d2-probe-self-check-") as temporary:
        root = pathlib.Path(temporary)
        (root / "markers").mkdir()
        write_new_bytes(root / "markers/probe.available", b"available\n", 0o400)
        consumed = consume_marker(root)
        require(consumed.is_file(), "marker fault check did not retain consumed marker")
        require(not (root / "markers/probe.available").exists(), "available marker survived")
    result = {"verdict": "PASS_UNRUN", "source_scan": scan, "parser": parser}
    print(json.dumps(result))
    return result


def local_run() -> None:
    local_static_check = self_check()
    require(not LOCAL_REMOTE_EVIDENCE.exists(), "local remote evidence already exists")
    require(not LOCAL_RECEIPT.exists(), "local probe receipt already exists")
    before_runtime = local_runtime_projection()
    expected_runtime = {
        "lay_version": "lay 1.0.43",
        "active_v11_sha256": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
        "pids": {
            "ibus-daemon": 2076194,
            "lay-daemon": 3410795,
            "lay-ibus-engine": 3410820,
        },
    }
    require(before_runtime == expected_runtime, f"local runtime baseline drift: {before_runtime}")
    controller_sha256 = sha256_file(pathlib.Path(__file__))

    status_code = (
        "import hashlib,json,os,pathlib;"
        "h=hashlib.sha256(pathlib.Path('/etc/machine-id').read_bytes()).hexdigest();"
        f"print(json.dumps({{'host':os.uname().nodename,'machine':h,'final':pathlib.Path('{REMOTE_FINAL}').exists(),'state':pathlib.Path('{REMOTE_STATE}').exists()}}))"
    )
    status = json.loads(ssh(["/usr/bin/python3", "-c", status_code]).stdout)
    require(
        status
        == {
            "host": REMOTE_HOSTNAME,
            "machine": REMOTE_MACHINE_ID_SHA256,
            "final": False,
            "state": False,
        },
        f"remote pre-run mismatch: {status}",
    )

    bootstrap = pathlib.Path(
        ssh(["mktemp", "-d", "/tmp/lay-d2-capability-probe.XXXXXX"]).stdout.decode().strip()
    )
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
        require(result.returncode == 0, f"remote probe publication failed: {result.stderr[-2000:]!r}")
        remote_summary = json.loads(result.stdout.decode().splitlines()[-1])
    finally:
        ssh(["rm", "-rf", "--", str(bootstrap)], check=False)

    with tempfile.TemporaryDirectory(prefix="lay-d2-probe-copy-", dir=str(LOCAL_RESULT_ROOT)) as temporary:
        temporary_root = pathlib.Path(temporary)
        run(["scp", "-q", "-r", f"{REMOTE}:{REMOTE_FINAL}", str(temporary_root)])
        copied = temporary_root / REMOTE_FINAL.name
        verify_sha256sums(copied)
        copied.rename(LOCAL_REMOTE_EVIDENCE)

    remote_receipt = load_json(LOCAL_REMOTE_EVIDENCE / "PROBE_RECEIPT.json")
    require(remote_receipt["verdict"] == remote_summary["verdict"], "remote summary mismatch")
    after_runtime = local_runtime_projection()
    require(after_runtime == before_runtime, f"local runtime changed: {after_runtime}")
    local_receipt = dict(remote_receipt)
    local_receipt.update(
        {
            "schema": "lay.v10.e1-traversal-d2-benign-capability-probe-local.v1",
            "remote_evidence_path": str(LOCAL_REMOTE_EVIDENCE),
            "remote_evidence_manifest_sha256": sha256_file(LOCAL_REMOTE_EVIDENCE / "SHA256SUMS"),
            "probe_preflight_manifest_sha256": EXPECTED["preflight_manifest"],
            "probe_preflight_receipt_sha256": EXPECTED["preflight_receipt"],
            "p0_static_receipt_sha256": EXPECTED["p0_receipt"],
            "local_static_check": local_static_check,
            "local_runtime_before": before_runtime,
            "local_runtime_after": after_runtime,
            "runtime_authority_changed": False,
        }
    )
    temporary_receipt = LOCAL_RESULT_ROOT / f".{LOCAL_RECEIPT.name}.tmp-{os.getpid()}"
    write_new_json(temporary_receipt, local_receipt, 0o400)
    os.replace(temporary_receipt, LOCAL_RECEIPT)
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
        raise ProbeError(f"unknown command: {arguments.command}")


if __name__ == "__main__":
    try:
        main()
    except ProbeError as error:
        print(json.dumps({"verdict": "ERROR", "error": str(error)}), file=sys.stderr)
        raise SystemExit(1)
