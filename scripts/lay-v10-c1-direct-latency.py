#!/usr/bin/env python3
"""Fail-closed controller for V10 C1 direct steady-state latency."""

from __future__ import annotations

import argparse
import base64
import contextlib
import datetime as dt
import fcntl
import hashlib
import json
import os
import pathlib
import re
import shlex
import shutil
import signal
import socket
import stat
import struct
import subprocess
import sys
import tempfile
import time
from typing import Any, Iterable


TASK_ID = "slice8b-v10-c1-direct-latency-v1-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PROVENANCE = pathlib.Path("/home/e/.local/share/lay/provenance")
REMOTE_FINAL = REMOTE_PROVENANCE / TASK_ID
REMOTE_WORK = REMOTE_PROVENANCE / f".{TASK_ID}.work"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
REMOTE_B_ROOT = REMOTE_PROVENANCE / "slice8b-v10-hardware-b0-b2-v3-20260824"
REMOTE_B0A = REMOTE_B_ROOT / "b0a-input-closure-v2"
REMOTE_B0B = REMOTE_B_ROOT / "b0b-schedule-closure-v1"
REMOTE_CLEAN_STATE = pathlib.Path("/home/e/.local/state/lay/slice8b-v10-clean-speed-v2-20260825")
REMOTE_EXECUTION = bool(globals().get("REMOTE_EXECUTION", False))
CONTROLLER_SOURCE_BYTES = globals().get("CONTROLLER_SOURCE_BYTES")

V10_SOURCE = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/v13_typed_peak.v10.rs"
CARGO_TOML = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/Cargo.toml"
CARGO_LOCK = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/Cargo.lock"
V7 = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json"
SIDECAR = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa"
PACKAGE = REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin"
SOURCE_CLOSURE = REMOTE_B0A / "inputs/surviving-source-closure"
SCHEDULE = REMOTE_B0B / "query-schedule.json"

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_c1_test_module.rs.inc"
CARGO_GUARD = PROJECT_ROOT / "scripts/cargo-guard.sh"
PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_IMPLEMENTATION_V3_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_IMPLEMENTATION_PREFLIGHT_V3_2026-08-25.json"
)
LOCAL_FINAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRECT_LATENCY_V1_2026-08-25"
)
C1_CONTRACT = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRECT_LATENCY_CONTRACT_V1_2026-08-25.md"
)
C1_ORDER = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_ORDER_CORRECTION_V1_2026-08-25.md"
)
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
CLEAN_SUPERSESSION = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V1_2026-08-25.json"
)

EXPECTED = {
    "preflight_manifest_file": "f84354f849b06264e8f0b2d079502f150ca7155b1e668ff6a592715e4b6e0efd",
    "preflight_manifest_identity": "9fdf017612118e85213c1487ee3bc8409f4df28bde0bb52cf0d1f865badd65ba",
    "preflight_receipt": "82ab93160acf12e71b27a135aec1228bceb24bc708132af1ed5d1473b36f366a",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
    "c1_contract": "1b9c8b2af590c721524b11fffeb54b92e37f5e571457605cb5c27dabc66f7752",
    "reviewed_contract_prefix": "1a35eeb0f5bb1e83e6785750a3a3857805bc62c93bc25d88d450874ae9f3f3d6",
    "c1_order": "02ce34cddff4dc599a9c2bc0daca84e87794f6f6e4838628617bb431a17ef0ff",
    "clean_supersession": "c6a82710d7bf4cafda33bcec21efe784605219c62d1a5fde252ff5c004351ffa",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
}
EXPECTED_SIZES = {
    "v10_source": 91_518,
    "production_prefix": 39_047,
    "package": 140_556_462,
    "sidecar": 3_689_884,
    "v7": 1_606_189,
    "schedule": 174_941,
}
TESTS = {
    "parity": "nanda_wave::l2_field::v13_typed_peak::tests::v10_c1_semantic_parity",
    "S": "nanda_wave::l2_field::v13_typed_peak::tests::v10_c1_single_latency",
    "T": "nanda_wave::l2_field::v13_typed_peak::tests::v10_c1_twenty_worker_latency",
}
ORDER = ["S1", "T1", "T2", "S2", "S3", "T3", "T4", "S4", "S5", "T5"]
S_CPUS = [0]
T_CPUS = list(range(20))
SAMPLES_PER_RUN = {"S": 38_200, "T": 95_500}
SAMPLE_STRUCT = struct.Struct("<HHBBQQ")
SAMPLE_BYTES = 22
THRESHOLDS = {
    "cpu_psi_some_avg10": 2.0,
    "memory_psi_full_avg10": 0.10,
    "io_psi_full_avg10": 0.10,
    "procs_running": 2,
    "temperature_c": 90.0,
    "busy_process_cpu_percent": 1.0,
}
LATENCY_THRESHOLDS = {
    "single_search_p99_us": 3_000,
    "single_total_p99_us": 5_000,
    "twenty_total_p99_us": 5_000,
    "fairness_total_p99_us": 5_000,
}


class C1Error(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise C1Error(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def canonical_json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def pretty_json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n"


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


def write_all(descriptor: int, value: bytes) -> None:
    remaining = memoryview(value)
    while remaining:
        written = os.write(descriptor, remaining)
        require(written > 0, "short write")
        remaining = remaining[written:]


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        write_all(descriptor, value)
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new_bytes(path, pretty_json(value), mode)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    records = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink is forbidden: {path}")
        if path.is_file():
            records.append(
                {
                    "path": str(path.relative_to(root)),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return records


def write_sha256sums(root: pathlib.Path) -> None:
    rows = [f"{record['sha256']}  {record['path']}\n" for record in inventory(root)]
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and re.fullmatch(r"[0-9a-f]{64}", digest) is not None, "bad manifest row")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts, "unsafe manifest path")
        require(relative not in seen and relative != "SHA256SUMS", "duplicate manifest path")
        seen.add(relative)
        require(sha256_file(root / path) == digest, f"manifest mismatch: {relative}")
    actual = {record["path"] for record in inventory(root) if record["path"] != "SHA256SUMS"}
    require(seen == actual, "manifest inventory mismatch")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        require(not path.is_symlink(), f"symlink is forbidden: {path}")
        if path.is_dir():
            path.chmod(0o555)
        else:
            path.chmod(0o555 if path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def make_tree_writable(root: pathlib.Path) -> None:
    if not root.exists():
        return
    if root.is_dir():
        root.chmod(0o755)
    for path in sorted(root.rglob("*")):
        if path.is_dir():
            path.chmod(0o755)
        else:
            path.chmod(0o755 if path.stat().st_mode & 0o111 else 0o644)


def remove_owned_tree(root: pathlib.Path) -> None:
    if root.exists():
        make_tree_writable(root)
        shutil.rmtree(root)


def atomic_publish(stage: pathlib.Path, final: pathlib.Path) -> None:
    require(stage.parent == final.parent, "stage and final must share a parent")
    require(stage.is_dir() and not final.exists(), "publication precondition failed")
    for path in [stage, *stage.rglob("*")]:
        require(not path.is_symlink(), f"symlink before publication: {path}")
        require(stat.S_IMODE(path.stat().st_mode) & 0o222 == 0, f"writable publication object: {path}")
    fsync_directory(stage)
    os.rename(stage, final)
    fsync_directory(final.parent)


def require_file(
    path: pathlib.Path,
    *,
    digest: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    require(path.is_file(), f"required file absent: {path}")
    if digest is not None:
        require(sha256_file(path) == digest, f"SHA-256 mismatch: {path}")
    if size is not None:
        require(path.stat().st_size == size, f"size mismatch: {path}")
    if mode is not None:
        require(mode_string(path) == mode, f"mode mismatch: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def assemble_source(v10: bytes, fragment: bytes) -> bytes:
    require(sha256_bytes(v10) == EXPECTED["v10_source"], "V10 source identity mismatch")
    require(v10.endswith(b"}\n"), "V10 terminal brace mismatch")
    require(fragment.startswith(b"\n    const C1_PARITY_TEST"), "C1 fragment prefix mismatch")
    final = v10[:-2] + fragment + b"}\n"
    require(len(v10.splitlines(keepends=True)[:1148]) > 0, "V10 line boundary absent")
    prefix = b"".join(v10.splitlines(keepends=True)[:1148])
    require(len(prefix) == EXPECTED_SIZES["production_prefix"], "production prefix size mismatch")
    require(final[: len(prefix)] == prefix, "production prefix bytes changed")
    require(sha256_bytes(prefix) == EXPECTED["production_prefix"], "production prefix SHA mismatch")
    return final


def nearest_rank(values: Iterable[int], percentile: int) -> int:
    ordered = sorted(values)
    require(ordered, "percentile denominator is empty")
    index = (len(ordered) * percentile + 99) // 100 - 1
    return ordered[index]


def parse_samples(path: pathlib.Path) -> list[dict[str, int]]:
    value = path.read_bytes()
    require(len(value) % SAMPLE_BYTES == 0, f"sample file width mismatch: {path}")
    records = []
    for offset in range(0, len(value), SAMPLE_BYTES):
        query, round_id, worker, flags, search_us, total_us = SAMPLE_STRUCT.unpack_from(value, offset)
        records.append(
            {
                "query_ordinal": query,
                "round": round_id,
                "worker_id": worker,
                "flags": flags,
                "search_elapsed_us": search_us,
                "total_elapsed_us": total_us,
            }
        )
    return records


def verify_local_admission() -> dict[str, Any]:
    manifest = require_file(PREFLIGHT_MANIFEST, digest=EXPECTED["preflight_manifest_file"], mode="0664")
    receipt = require_file(PREFLIGHT_RECEIPT, digest=EXPECTED["preflight_receipt"], mode="0664")
    receipt_value = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(receipt_value.get("verdict") == "READY_TO_IMPLEMENT", "C1 preflight did not pass")
    require(receipt_value.get("safe_to_implement") is True, "C1 preflight did not admit implementation")
    require(receipt_value.get("manifest_sha256") == EXPECTED["preflight_manifest_identity"], "C1 preflight identity mismatch")
    require_file(C1_CONTRACT, digest=EXPECTED["c1_contract"], mode="0664")
    require(sha256_bytes(C1_CONTRACT.read_bytes()[:14_667]) == EXPECTED["reviewed_contract_prefix"], "reviewed C1 contract prefix drift")
    require_file(C1_ORDER, digest=EXPECTED["c1_order"], mode="0664")
    require_file(CLEAN_SUPERSESSION, digest=EXPECTED["clean_supersession"], mode="0444")
    require_file(ACTIVE_V11, digest=EXPECTED["active_v11"], mode="0664")
    require_file(CARGO_GUARD, digest=EXPECTED["cargo_guard"], mode="0775")
    return {"manifest": manifest, "receipt": receipt}


def local_runtime_snapshot() -> dict[str, Any]:
    def process(name: str) -> list[dict[str, Any]]:
        result = subprocess.run(["/usr/bin/pgrep", "-x", name], stdout=subprocess.PIPE, check=False)
        records = []
        for raw in result.stdout.split():
            pid = int(raw)
            root = pathlib.Path("/proc") / str(pid)
            with contextlib.suppress(Exception):
                fields = (root / "stat").read_text().split()
                records.append(
                    {
                        "pid": pid,
                        "starttime": fields[21],
                        "argv": (root / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip(),
                    }
                )
        return records

    version = subprocess.run([str(pathlib.Path.home() / ".local/bin/lay"), "--version"], stdout=subprocess.PIPE, check=True).stdout.decode().strip()
    return {
        "lay_version": version,
        "active_v11_sha256": sha256_file(ACTIVE_V11),
        "ibus_daemon": process("ibus-daemon"),
        "lay_daemon": process("lay-daemon"),
        "lay_ibus_engine": process("lay-ibus-engine"),
    }


def process_record(path: pathlib.Path) -> dict[str, Any] | None:
    try:
        fields = (path / "stat").read_text().split()
        return {
            "pid": int(path.name),
            "ppid": int(fields[3]),
            "comm": (path / "comm").read_text().strip(),
            "cmdline": (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip(),
            "cpu_seconds": (int(fields[13]) + int(fields[14])) / os.sysconf("SC_CLK_TCK"),
        }
    except (FileNotFoundError, PermissionError, ProcessLookupError, ValueError, IndexError):
        return None


def all_processes() -> dict[int, dict[str, Any]]:
    records = {}
    for path in pathlib.Path("/proc").iterdir():
        if path.name.isdigit():
            record = process_record(path)
            if record is not None:
                records[record["pid"]] = record
    return records


def ancestor_pids(records: dict[int, dict[str, Any]], pid: int) -> set[int]:
    output = {pid}
    while pid in records and records[pid]["ppid"] not in output and records[pid]["ppid"] > 0:
        pid = records[pid]["ppid"]
        output.add(pid)
    return output


def read_pressure(name: str) -> dict[str, dict[str, float]]:
    value = {}
    for line in (pathlib.Path("/proc/pressure") / name).read_text().splitlines():
        fields = line.split()
        value[fields[0]] = {key: float(raw) for key, raw in (item.split("=", 1) for item in fields[1:]) if key != "total"}
    return value


def maximum_temperature() -> float:
    values = []
    for path in pathlib.Path("/sys/class/thermal").glob("thermal_zone*/temp"):
        with contextlib.suppress(ValueError, OSError):
            value = float(path.read_text().strip()) / 1000.0
            if 0.0 < value < 200.0:
                values.append(value)
    for path in pathlib.Path("/sys/class/hwmon").glob("hwmon*/temp*_input"):
        with contextlib.suppress(ValueError, OSError):
            value = float(path.read_text().strip()) / 1000.0
            if 0.0 < value < 200.0:
                values.append(value)
    return max(values, default=0.0)


def throttle_counters() -> dict[str, str]:
    output = {}
    patterns = [
        "/sys/devices/system/cpu/cpu*/thermal_throttle/*_throttle_count",
        "/sys/class/thermal/thermal_zone*/trip_point_*_temp",
    ]
    for pattern in patterns:
        for path in sorted(pathlib.Path("/").glob(pattern.removeprefix("/"))):
            with contextlib.suppress(OSError):
                output[str(path)] = path.read_text().strip()
    return output


def stable_host_projection() -> dict[str, Any]:
    projection: dict[str, Any] = {}
    paths = [
        pathlib.Path("/sys/devices/system/cpu/online"),
        pathlib.Path("/sys/devices/system/cpu/smt/active"),
        pathlib.Path("/sys/devices/system/cpu/intel_pstate/no_turbo"),
        pathlib.Path("/sys/devices/system/cpu/cpufreq/boost"),
    ]
    paths.extend(sorted(pathlib.Path("/sys/devices/system/cpu/cpufreq").glob("policy*/scaling_governor")))
    paths.extend(sorted(pathlib.Path("/sys/devices/system/cpu/cpufreq").glob("policy*/energy_performance_preference")))
    for path in paths:
        if path.is_file():
            projection[str(path)] = path.read_text().strip()
    topology = {}
    for cpu in range(20):
        root = pathlib.Path(f"/sys/devices/system/cpu/cpu{cpu}/topology")
        topology[str(cpu)] = {
            name: (root / name).read_text().strip()
            for name in ("core_id", "physical_package_id", "thread_siblings_list")
        }
    projection["topology"] = topology
    return projection


def quiet_sample() -> dict[str, Any]:
    before = all_processes()
    excluded = ancestor_pids(before, os.getpid())
    started = time.monotonic()
    time.sleep(0.25)
    elapsed = time.monotonic() - started
    after = all_processes()
    busy = []
    for pid in sorted(before.keys() & after.keys()):
        if pid in excluded or not after[pid]["cmdline"]:
            continue
        cpu = (after[pid]["cpu_seconds"] - before[pid]["cpu_seconds"]) / elapsed * 100.0
        if cpu > THRESHOLDS["busy_process_cpu_percent"]:
            busy.append({"pid": pid, "comm": after[pid]["comm"], "cmdline": after[pid]["cmdline"], "cpu_percent": cpu})
    stat_line = next(line for line in pathlib.Path("/proc/stat").read_text().splitlines() if line.startswith("procs_running"))
    return {
        "observed_at": now(),
        "cpu_pressure": read_pressure("cpu"),
        "memory_pressure": read_pressure("memory"),
        "io_pressure": read_pressure("io"),
        "procs_running": int(stat_line.split()[1]),
        "maximum_temperature_c": maximum_temperature(),
        "busy_processes": busy,
    }


def sample_failures(sample: dict[str, Any]) -> list[str]:
    failures = []
    if sample["cpu_pressure"].get("some", {}).get("avg10", float("inf")) > THRESHOLDS["cpu_psi_some_avg10"]:
        failures.append("cpu_psi")
    if sample["memory_pressure"].get("full", {}).get("avg10", float("inf")) > THRESHOLDS["memory_psi_full_avg10"]:
        failures.append("memory_psi")
    if sample["io_pressure"].get("full", {}).get("avg10", float("inf")) > THRESHOLDS["io_psi_full_avg10"]:
        failures.append("io_psi")
    if sample["procs_running"] > THRESHOLDS["procs_running"]:
        failures.append("procs_running")
    temperature = sample["maximum_temperature_c"]
    if not (0.0 < temperature < THRESHOLDS["temperature_c"]):
        failures.append("temperature")
    if sample["busy_processes"]:
        failures.append("busy_processes")
    return failures


def environment_admission() -> dict[str, Any]:
    stable_before = stable_host_projection()
    throttle_before = throttle_counters()
    samples = [quiet_sample() for _ in range(3)]
    stable_after = stable_host_projection()
    throttle_after = throttle_counters()
    failures = sorted({failure for sample in samples for failure in sample_failures(sample)})
    if stable_before != stable_after:
        failures.append("stable_host_projection_drift")
    if throttle_before != throttle_after:
        failures.append("thermal_throttle_counter_drift")
    return {
        "schema": "lay.v10.c1-environment-admission.v1",
        "verdict": "PASS" if not failures else "BLOCKED_ENVIRONMENT",
        "failures": sorted(set(failures)),
        "samples": samples,
        "thresholds": THRESHOLDS,
        "stable_before": stable_before,
        "stable_after": stable_after,
        "throttle_before": throttle_before,
        "throttle_after": throttle_after,
        "remote_writes": False,
    }


def remote_input_identities() -> dict[str, Any]:
    require(socket.gethostname() == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(sha256_file(pathlib.Path("/etc/machine-id")) == REMOTE_MACHINE_ID_SHA256, "remote machine identity mismatch")
    identities = {
        "v10_source": require_file(V10_SOURCE, digest=EXPECTED["v10_source"], size=EXPECTED_SIZES["v10_source"], mode="0444"),
        "cargo_toml": require_file(CARGO_TOML, digest=EXPECTED["cargo_toml"], mode="0444"),
        "cargo_lock": require_file(CARGO_LOCK, digest=EXPECTED["cargo_lock"], mode="0444"),
        "package": require_file(PACKAGE, digest=EXPECTED["package"], size=EXPECTED_SIZES["package"], mode="0444"),
        "sidecar": require_file(SIDECAR, digest=EXPECTED["sidecar"], size=EXPECTED_SIZES["sidecar"], mode="0444"),
        "v7": require_file(V7, digest=EXPECTED["v7"], size=EXPECTED_SIZES["v7"], mode="0444"),
        "schedule": require_file(SCHEDULE, digest=EXPECTED["schedule"], size=EXPECTED_SIZES["schedule"], mode="0444"),
    }
    require(SOURCE_CLOSURE.is_dir(), "source closure is absent")
    require(REMOTE_CLEAN_STATE.is_dir(), "Clean V2 supersession state is absent")
    require_file(REMOTE_CLEAN_STATE / "SUPERSEDED_UNRUN.json", digest="76c7bc00053e642fc83b089e2bcfd088d1788f99d5e38bbe7aa6ccdd55e9f35d", mode="0444")
    require(not (REMOTE_PROVENANCE / "slice8b-v10-clean-speed-v2-20260825").exists(), "Clean V2 final result exists")
    return identities


def marker_path(name: str, state: str) -> pathlib.Path:
    return REMOTE_STATE / "markers" / f"{name}.{state}"


def consume_marker(name: str) -> pathlib.Path:
    available = marker_path(name, "available")
    consumed = marker_path(name, "consumed-before-exec")
    require(available.is_file() and not consumed.exists(), f"marker is not available: {name}")
    os.rename(available, consumed)
    fsync_directory(available.parent)
    return consumed


@contextlib.contextmanager
def route_lock() -> Iterable[None]:
    lock = REMOTE_STATE / "route.lock"
    require_file(lock, mode="0400")
    descriptor = os.open(lock, os.O_RDONLY)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise C1Error("another C1 owner holds the route lock") from error
        yield
    finally:
        with contextlib.suppress(OSError):
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def remote_status_value() -> dict[str, Any]:
    value: dict[str, Any] = {
        "hostname": socket.gethostname(),
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        "final_exists": REMOTE_FINAL.exists(),
        "work_exists": REMOTE_WORK.exists(),
        "state_exists": REMOTE_STATE.exists(),
        "paths": {"final": str(REMOTE_FINAL), "work": str(REMOTE_WORK), "state": str(REMOTE_STATE)},
    }
    if REMOTE_STATE.is_dir():
        value["markers"] = sorted(path.name for path in (REMOTE_STATE / "markers").iterdir())
    if (REMOTE_WORK / "build-v1/EXECUTABLE_PROVENANCE.json").is_file():
        value["build"] = json.loads((REMOTE_WORK / "build-v1/EXECUTABLE_PROVENANCE.json").read_text())
    if (REMOTE_WORK / "parity-v1/subject/SUBJECT_RECEIPT.json").is_file():
        value["parity"] = json.loads((REMOTE_WORK / "parity-v1/subject/SUBJECT_RECEIPT.json").read_text())
    return value


def remote_prepare(payload: dict[str, Any]) -> None:
    require(REMOTE_EXECUTION, "remote prepare is not a local action")
    inputs = remote_input_identities()
    require(not REMOTE_FINAL.exists() and not REMOTE_WORK.exists() and not REMOTE_STATE.exists(), "C1 remote path already exists")
    fragment = base64.b64decode(payload["fragment_b64"], validate=True)
    controller = CONTROLLER_SOURCE_BYTES
    require(isinstance(controller, bytes), "controller source bytes are absent")
    require(sha256_bytes(fragment) == payload["fragment_sha256"], "fragment transport mismatch")
    require(sha256_bytes(controller) == payload["controller_sha256"], "controller transport mismatch")
    assembled = assemble_source(V10_SOURCE.read_bytes(), fragment)

    REMOTE_STATE.mkdir(mode=0o700)
    try:
        REMOTE_WORK.mkdir(mode=0o700)
        inputs_dir = REMOTE_WORK / "inputs"
        inputs_dir.mkdir(mode=0o700)
        write_new_bytes(inputs_dir / "lay_v10_c1_test_module.rs.inc", fragment)
        write_new_bytes(inputs_dir / "lay-v10-c1-direct-latency.py", controller, 0o555)
        write_new_json(
            REMOTE_WORK / "PREPARATION.json",
            {
                "schema": "lay.v10.c1-preparation.v1",
                "task_id": TASK_ID,
                "prepared_at": now(),
                "controller_sha256": payload["controller_sha256"],
                "fragment_sha256": payload["fragment_sha256"],
                "assembled_source_sha256": sha256_bytes(assembled),
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "inputs": inputs,
                "route_order": ORDER,
                "subject_executed": False,
                "build_executed": False,
                "perf_invoked": False,
                "pmu_event_opened": False,
                "runtime_authority_changed": False,
                "installed_lay_changed": False,
            },
        )
        markers = REMOTE_STATE / "markers"
        markers.mkdir(mode=0o700)
        write_new_bytes(REMOTE_STATE / "route.lock", b"c1\n", 0o400)
        for name in ["build", "parity", *ORDER]:
            write_new_bytes(marker_path(name, "available"), pretty_json({"route": name, "available_at": now()}), 0o400)
        fsync_directory(markers)
        fsync_directory(REMOTE_STATE)
        fsync_directory(REMOTE_WORK)
    except BaseException:
        raise
    print(json.dumps({"verdict": "C1_PREPARED_BUILD_AVAILABLE", "status": remote_status_value()}, sort_keys=True))


def find_test_executable(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if path.is_file() and os.access(path, os.X_OK) and not path.name.endswith((".d", ".rlib", ".rmeta")):
            with path.open("rb") as source:
                if source.read(4) == b"\x7fELF":
                    candidates.append(path)
    require(len(candidates) == 1, f"expected one test ELF, found {candidates}")
    return candidates[0]


def elf_build_id(path: pathlib.Path) -> str:
    result = subprocess.run(["/usr/bin/readelf", "-n", str(path)], stdout=subprocess.PIPE, check=True)
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", result.stdout)
    require(match is not None, "ELF Build ID is absent")
    return match.group(1).decode()


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "LOGNAME": "e",
        "PATH": "/home/e/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "USER": "e",
        "RUST_TEST_THREADS": "1",
    }


def remote_build() -> None:
    require(REMOTE_EXECUTION, "remote build is not a local action")
    remote_input_identities()
    with route_lock():
        require(REMOTE_WORK.is_dir() and REMOTE_STATE.is_dir(), "C1 is not prepared")
        require(not (REMOTE_WORK / "build-v1").exists(), "C1 build already exists")
        fragment = (REMOTE_WORK / "inputs/lay_v10_c1_test_module.rs.inc").read_bytes()
        source = assemble_source(V10_SOURCE.read_bytes(), fragment)
        workspace = pathlib.Path(f"/home/e/.cache/lay-c1-build-{os.getpid()}-{time.time_ns()}")
        stage = REMOTE_WORK / f".build-v1.stage-{os.getpid()}-{time.time_ns()}"
        workspace.mkdir(parents=True, mode=0o700)
        stage.mkdir(mode=0o700)
        marker_consumed = False
        try:
            shutil.copytree(SOURCE_CLOSURE, workspace, dirs_exist_ok=True)
            make_tree_writable(workspace)
            shutil.copyfile(CARGO_TOML, workspace / "Cargo.toml")
            shutil.copyfile(CARGO_LOCK, workspace / "Cargo.lock")
            (workspace / "scripts").mkdir(exist_ok=True)
            shutil.copyfile(REMOTE_WORK / "inputs/lay-v10-c1-direct-latency.py", stage / "controller.py")
            source_guard = REMOTE_B0A / "inputs/controller/cargo-guard.sh"
            shutil.copyfile(source_guard, workspace / "scripts/cargo-guard.sh")
            require(sha256_file(workspace / "scripts/cargo-guard.sh") == EXPECTED["cargo_guard"], "remote cargo guard mismatch")
            (workspace / "scripts/cargo-guard.sh").chmod(0o775)
            source_path = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
            source_path.parent.mkdir(parents=True, exist_ok=True)
            source_path.write_bytes(source)
            source_path.chmod(0o444)
            write_new_bytes(stage / "c1-source.rs", source)
            prebuild = {
                "schema": "lay.v10.c1-prebuild.v1",
                "task_id": TASK_ID,
                "prepared_at": now(),
                "v10_source_sha256": EXPECTED["v10_source"],
                "production_prefix_bytes": EXPECTED_SIZES["production_prefix"],
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "fragment_sha256": sha256_bytes(fragment),
                "final_source_sha256": sha256_bytes(source),
                "cargo_toml_sha256": EXPECTED["cargo_toml"],
                "cargo_lock_sha256": EXPECTED["cargo_lock"],
                "cargo_guard_sha256": EXPECTED["cargo_guard"],
                "cargo_started": False,
            }
            write_new_json(stage / "PREBUILD.json", prebuild)
            consume_marker("build")
            marker_consumed = True
            environment = controlled_environment()
            environment.update(
                {
                    "CARGO_BUILD_JOBS": "20",
                    "CARGO_INCREMENTAL": "0",
                    "CARGO_NET_OFFLINE": "true",
                    "CARGO_TARGET_DIR": str(workspace / "target"),
                    "RUSTFLAGS": "",
                }
            )
            command = [
                str(workspace / "scripts/cargo-guard.sh"),
                "test",
                "--offline",
                "--locked",
                "--release",
                "--lib",
                "--no-run",
                "nanda_wave::l2_field::v13_typed_peak::tests",
            ]
            with (stage / "cargo.log").open("wb") as log:
                process = subprocess.Popen(
                    command,
                    cwd=workspace,
                    env=environment,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    start_new_session=True,
                )
                try:
                    exit_code = process.wait(timeout=1800)
                except BaseException:
                    with contextlib.suppress(ProcessLookupError):
                        os.killpg(process.pid, signal.SIGTERM)
                    with contextlib.suppress(subprocess.TimeoutExpired):
                        process.wait(timeout=10)
                    raise
                log.flush()
                os.fsync(log.fileno())
            require(exit_code == 0, f"C1 guarded build exited {exit_code}")
            executable = find_test_executable(workspace / "target")
            shutil.copyfile(executable, stage / "c1-test-elf")
            (stage / "c1-test-elf").chmod(0o555)
            provenance = {
                "schema": "lay.v10.c1-executable-provenance.v1",
                "task_id": TASK_ID,
                "source": prebuild,
                "build": {
                    "hostname": socket.gethostname(),
                    "command": command,
                    "cargo_incremental": "0",
                    "cargo_net_offline": "true",
                    "profile": "release",
                    "build_marker_consumed_before_cargo": True,
                    "retry_permitted": False,
                },
                "executable": {
                    "sha256": sha256_file(stage / "c1-test-elf"),
                    "size_bytes": (stage / "c1-test-elf").stat().st_size,
                    "build_id": elf_build_id(stage / "c1-test-elf"),
                    "tests": TESTS,
                },
                "executed": False,
                "perf_invoked": False,
                "pmu_event_opened": False,
                "runtime_authority_changed": False,
                "installed_lay_changed": False,
            }
            write_new_json(stage / "EXECUTABLE_PROVENANCE.json", provenance)
            write_sha256sums(stage)
            seal_tree(stage)
            atomic_publish(stage, REMOTE_WORK / "build-v1")
            print(json.dumps({"verdict": "C1_EXECUTABLE_SEALED_BUILD_CONSUMED", "executable": provenance["executable"]}, sort_keys=True))
        except BaseException as error:
            if marker_consumed and stage.exists():
                with contextlib.suppress(Exception):
                    write_new_json(stage / "FAILURE.json", {"verdict": "C1_BUILD_FAILED_CONSUMED_NO_RETRY", "error": str(error)})
                    write_sha256sums(stage)
                    seal_tree(stage)
                    atomic_publish(stage, REMOTE_WORK / f"build-v1.failed-{time.time_ns()}")
            raise
        finally:
            with contextlib.suppress(Exception):
                remove_owned_tree(workspace)


def subject_environment(output: pathlib.Path, route: str) -> dict[str, str]:
    environment = controlled_environment()
    cpus = S_CPUS if route.startswith("S") else T_CPUS
    environment.update(
        {
            "LAY_V10_C1_PACKAGE": str(PACKAGE),
            "LAY_V10_C1_SIDECAR": str(SIDECAR),
            "LAY_V10_C1_V7": str(V7),
            "LAY_V10_C1_SCHEDULE": str(SCHEDULE),
            "LAY_V10_C1_OUTPUT": str(output),
            "LAY_V10_C1_RUN_ID": route,
            "LAY_V10_C1_CPUS": ",".join(map(str, cpus)),
        }
    )
    return environment


def execute_subject(executable: pathlib.Path, test: str, route: str, stage: pathlib.Path) -> tuple[int, int]:
    output = stage / "subject"
    output.mkdir(mode=0o700)
    cpus = S_CPUS if route.startswith("S") else T_CPUS
    command = [
        "/usr/bin/taskset",
        "-c",
        ",".join(map(str, cpus)),
        str(executable),
        "--ignored",
        "--exact",
        test,
        "--nocapture",
        "--test-threads=1",
    ]
    with (stage / "stdout.log").open("xb") as stdout, (stage / "stderr.log").open("xb") as stderr:
        started = time.perf_counter_ns()
        process = subprocess.Popen(
            command,
            env=subject_environment(output, route),
            stdout=stdout,
            stderr=stderr,
            start_new_session=True,
        )
        try:
            exit_code = process.wait(timeout=1800)
        except BaseException:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(process.pid, signal.SIGTERM)
            with contextlib.suppress(subprocess.TimeoutExpired):
                process.wait(timeout=10)
            raise
        elapsed = time.perf_counter_ns() - started
        stdout.flush(); os.fsync(stdout.fileno())
        stderr.flush(); os.fsync(stderr.fileno())
    return exit_code, elapsed


def remote_parity() -> None:
    require(REMOTE_EXECUTION, "remote parity is not a local action")
    remote_input_identities()
    with route_lock():
        build = REMOTE_WORK / "build-v1"
        require(build.is_dir(), "C1 build is absent")
        verify_sha256sums(build)
        stage = REMOTE_WORK / f".parity-v1.stage-{os.getpid()}-{time.time_ns()}"
        stage.mkdir(mode=0o700)
        marker_consumed = False
        try:
            consume_marker("parity")
            marker_consumed = True
            executable = build / "c1-test-elf"
            provenance = json.loads((build / "EXECUTABLE_PROVENANCE.json").read_text())
            require_file(executable, digest=provenance["executable"]["sha256"], size=provenance["executable"]["size_bytes"], mode="0555")
            exit_code, elapsed_ns = execute_subject(executable, TESTS["parity"], "PARITY", stage)
            receipt = json.loads((stage / "subject/SUBJECT_RECEIPT.json").read_text())
            expected = {
                "verdict": "PASS",
                "records": 382,
                "schedule_records": 382,
                "terminal_mismatches": 0,
                "peak_mismatches": 0,
                "completeness_mismatches": 0,
                "work_mismatches": 0,
                "target_form_retained": 382,
                "target_lemma_retained": 382,
                "false_certificates": 0,
                "maximum_product_states": 35_590,
                "maximum_scratch_bytes": 6_656,
            }
            require(exit_code == 0, f"C1 parity exited {exit_code}")
            for key, value in expected.items():
                require(receipt.get(key) == value, f"C1 parity mismatch {key}: {receipt.get(key)!r}")
            write_new_json(
                stage / "WRAPPER_RECEIPT.json",
                {
                    "schema": "lay.v10.c1-parity-wrapper.v1",
                    "verdict": "PASS",
                    "test": TESTS["parity"],
                    "process_wall_ns_diagnostic": elapsed_ns,
                    "latency_authority": False,
                    "parity_marker_consumed_before_exec": True,
                    "retry_permitted": False,
                    "perf_invoked": False,
                    "pmu_event_opened": False,
                    "runtime_authority_changed": False,
                    "installed_lay_changed": False,
                },
            )
            write_sha256sums(stage)
            seal_tree(stage)
            atomic_publish(stage, REMOTE_WORK / "parity-v1")
            print(json.dumps({"verdict": "C1_PARITY_PASS_LATENCY_AVAILABLE", "parity": receipt}, sort_keys=True))
        except BaseException as error:
            if marker_consumed and stage.exists():
                with contextlib.suppress(Exception):
                    write_new_json(stage / "FAILURE.json", {"verdict": "C1_PARITY_FAILED_CONSUMED_LATENCY_BLOCKED", "error": str(error)})
                    write_sha256sums(stage)
                    seal_tree(stage)
                    atomic_publish(stage, REMOTE_WORK / f"parity-v1.failed-{time.time_ns()}")
            raise


def route_kind(route: str) -> str:
    return route[0]


def validate_subject_route(route: str, stage: pathlib.Path) -> tuple[dict[str, Any], list[dict[str, int]]]:
    receipt = json.loads((stage / "subject/SUBJECT_RECEIPT.json").read_text())
    samples_path = stage / "subject/samples.bin"
    samples = parse_samples(samples_path)
    kind = route_kind(route)
    require(receipt.get("run_id") == route, "subject run identity mismatch")
    require(receipt.get("samples", {}).get("records") == SAMPLES_PER_RUN[kind], "subject sample count mismatch")
    require(len(samples) == SAMPLES_PER_RUN[kind], "raw sample count mismatch")
    require(receipt.get("samples_sha256") == sha256_file(samples_path), "subject sample SHA mismatch")
    require(receipt.get("samples", {}).get("errors") == 0, "subject errors")
    require(receipt.get("samples", {}).get("unresolved") == 0, "subject unresolved")
    if kind == "S":
        require(receipt.get("thread_migrations_delta") == 0, "S thread migrated")
        require(receipt.get("sample_capacity") == 38_200, "S capacity mismatch")
    else:
        require(receipt.get("worker_capacities") == [5_000] * 19 + [500], "T worker capacities mismatch")
        require(receipt.get("worker_migration_deltas") == [0] * 20, "T worker migrated")
        require(receipt.get("start_barriers") == 251 and receipt.get("end_barriers") == 251, "T barrier count mismatch")
    return receipt, samples


def route_percentiles(samples: list[dict[str, int]]) -> dict[str, int]:
    return {
        "search_p99_us": nearest_rank((sample["search_elapsed_us"] for sample in samples), 99),
        "total_p99_us": nearest_rank((sample["total_elapsed_us"] for sample in samples), 99),
        "search_max_us": max(sample["search_elapsed_us"] for sample in samples),
        "total_max_us": max(sample["total_elapsed_us"] for sample in samples),
    }


def active_foreign_cpu(before: dict[int, dict[str, Any]], after: dict[int, dict[str, Any]], elapsed: float) -> list[dict[str, Any]]:
    excluded = ancestor_pids(after, os.getpid())
    busy = []
    for pid in sorted(before.keys() & after.keys()):
        if pid in excluded or not after[pid]["cmdline"]:
            continue
        cpu = (after[pid]["cpu_seconds"] - before[pid]["cpu_seconds"]) / max(elapsed, 0.001) * 100.0
        if cpu > THRESHOLDS["busy_process_cpu_percent"]:
            busy.append({"pid": pid, "comm": after[pid]["comm"], "cmdline": after[pid]["cmdline"], "cpu_percent": cpu})
    return busy


def publish_terminal(verdict: str, detail: dict[str, Any]) -> dict[str, Any]:
    require(REMOTE_WORK.is_dir() and not REMOTE_FINAL.exists(), "C1 publication path mismatch")
    result = {
        "schema": "lay.v10.c1-direct-steady-state-latency.v1",
        "task_id": TASK_ID,
        "published_at": now(),
        "verdict": verdict,
        **detail,
        "claim": "V10_DERIVED_STEADY_STATE_PRODUCT_LATENCY_ONLY",
        "historical_v10_replay": False,
        "full_b_executed": False,
        "v12_admitted": False,
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    write_new_json(REMOTE_WORK / "C1_RESULT.json", result)
    write_sha256sums(REMOTE_WORK)
    seal_tree(REMOTE_WORK)
    atomic_publish(REMOTE_WORK, REMOTE_FINAL)
    seal_tree(REMOTE_STATE)
    return result


def aggregate_matrix() -> dict[str, Any]:
    routes: dict[str, Any] = {}
    pooled_s: list[dict[str, int]] = []
    pooled_t: list[dict[str, int]] = []
    query_samples: dict[int, list[dict[str, int]]] = {index: [] for index in range(382)}
    fairness = []
    for route in ORDER:
        root = REMOTE_WORK / f"run-{route}"
        verify_sha256sums(root)
        samples = parse_samples(root / "subject/samples.bin")
        kind = route_kind(route)
        routes[route] = route_percentiles(samples)
        (pooled_s if kind == "S" else pooled_t).extend(samples)
        for sample in samples:
            query_samples[sample["query_ordinal"]].append(sample)
        if kind == "T":
            workers = {}
            for worker in range(20):
                worker_samples = [sample for sample in samples if sample["worker_id"] == worker]
                workers[str(worker)] = route_percentiles(worker_samples)
                fairness.append((route, worker, workers[str(worker)]["total_p99_us"]))
            routes[route]["workers"] = workers
    require(len(pooled_s) == 191_000, "pooled S denominator mismatch")
    require(len(pooled_t) == 477_500, "pooled T denominator mismatch")
    pooled = {"S": route_percentiles(pooled_s), "T": route_percentiles(pooled_t)}
    per_query = {
        str(query): route_percentiles(samples)
        for query, samples in query_samples.items()
    }
    worst_query = max(per_query.items(), key=lambda item: item[1]["total_p99_us"])
    worst_worker = max(fairness, key=lambda item: item[2])
    errors = sum(sample["flags"] & 1 != 0 for sample in [*pooled_s, *pooled_t])
    unresolved = sum(sample["flags"] & 2 != 0 for sample in [*pooled_s, *pooled_t])
    hard = {
        "s_pooled_search": pooled["S"]["search_p99_us"] <= 3_000,
        "s_pooled_total": pooled["S"]["total_p99_us"] <= 5_000,
        "s_runs_search": all(routes[f"S{index}"]["search_p99_us"] <= 3_000 for index in range(1, 6)),
        "s_runs_total": all(routes[f"S{index}"]["total_p99_us"] <= 5_000 for index in range(1, 6)),
        "t_pooled_total": pooled["T"]["total_p99_us"] <= 5_000,
        "t_runs_total": all(routes[f"T{index}"]["total_p99_us"] <= 5_000 for index in range(1, 6)),
        "fairness": worst_worker[2] <= 5_000,
        "errors": errors == 0,
        "unresolved": unresolved == 0,
    }
    return {
        "verdict": "C1_PASS" if all(hard.values()) else "C1_FAIL",
        "hard_conjuncts": hard,
        "thresholds_us": LATENCY_THRESHOLDS,
        "pooled": pooled,
        "routes": routes,
        "fairness": {
            "maximum_run": worst_worker[0],
            "maximum_worker": worst_worker[1],
            "maximum_total_p99_us": worst_worker[2],
            "minimum_total_p99_us": min(item[2] for item in fairness),
            "spread_us": worst_worker[2] - min(item[2] for item in fairness),
        },
        "query_diagnostics": {
            "per_query": per_query,
            "worst_query_ordinal": int(worst_query[0]),
            "worst_query_total_p99_us": worst_query[1]["total_p99_us"],
        },
        "sample_counts": {"S": len(pooled_s), "T": len(pooled_t)},
        "errors": errors,
        "unresolved": unresolved,
    }


def remote_run_matrix() -> None:
    require(REMOTE_EXECUTION, "remote matrix is not a local action")
    remote_input_identities()
    with route_lock():
        require((REMOTE_WORK / "parity-v1/SUBJECT_RECEIPT.json").is_file(), "C1 parity is absent")
        parity = json.loads((REMOTE_WORK / "parity-v1/SUBJECT_RECEIPT.json").read_text())
        require(parity.get("verdict") == "PASS", "C1 parity did not pass")
        executable = REMOTE_WORK / "build-v1/c1-test-elf"
        provenance = json.loads((REMOTE_WORK / "build-v1/EXECUTABLE_PROVENANCE.json").read_text())
        require_file(executable, digest=provenance["executable"]["sha256"], size=provenance["executable"]["size_bytes"], mode="0555")
        for route in ORDER:
            require(marker_path(route, "available").is_file(), f"route marker unavailable: {route}")
            admission = environment_admission()
            if admission["verdict"] != "PASS":
                os.rename(marker_path(route, "available"), marker_path(route, "blocked-before-exec"))
                write_new_json(REMOTE_WORK / f"{route}_BLOCKED_ENVIRONMENT.json", admission)
                result = publish_terminal(
                    "BLOCKED_ENVIRONMENT",
                    {
                        "blocked_route": route,
                        "environment": admission,
                        "completed_routes": [prior for prior in ORDER if (REMOTE_WORK / f"run-{prior}").is_dir()],
                        "replacement_process_admitted": False,
                    },
                )
                print(json.dumps(result, sort_keys=True))
                return
            stable_before = stable_host_projection()
            throttle_before = throttle_counters()
            processes_before = all_processes()
            stage = REMOTE_WORK / f".run-{route}.stage-{os.getpid()}-{time.time_ns()}"
            stage.mkdir(mode=0o700)
            consumed = False
            try:
                consume_marker(route)
                consumed = True
                started = time.monotonic()
                exit_code, wall_ns = execute_subject(executable, TESTS[route_kind(route)], route, stage)
                elapsed = time.monotonic() - started
                processes_after = all_processes()
                stable_after = stable_host_projection()
                throttle_after = throttle_counters()
                foreign = active_foreign_cpu(processes_before, processes_after, elapsed)
                receipt, samples = validate_subject_route(route, stage)
                environment_ok = stable_before == stable_after and throttle_before == throttle_after and not foreign
                require(exit_code == 0, f"C1 {route} exited {exit_code}")
                require(environment_ok, f"C1 {route} environment changed")
                wrapper = {
                    "schema": "lay.v10.c1-route-wrapper.v1",
                    "route": route,
                    "verdict": "PASS",
                    "admission": admission,
                    "stable_before": stable_before,
                    "stable_after": stable_after,
                    "throttle_before": throttle_before,
                    "throttle_after": throttle_after,
                    "foreign_busy_processes": foreign,
                    "process_wall_ns_diagnostic": wall_ns,
                    "percentiles": route_percentiles(samples),
                    "marker_consumed_before_exec": True,
                    "replacement_process_admitted": False,
                    "perf_invoked": False,
                    "pmu_event_opened": False,
                    "runtime_authority_changed": False,
                    "installed_lay_changed": False,
                }
                write_new_json(stage / "WRAPPER_RECEIPT.json", wrapper)
                write_sha256sums(stage)
                seal_tree(stage)
                atomic_publish(stage, REMOTE_WORK / f"run-{route}")
            except BaseException as error:
                if consumed and stage.exists():
                    with contextlib.suppress(Exception):
                        write_new_json(stage / "FAILURE.json", {"verdict": "C1_MATRIX_TERMINAL_NO_REPLACEMENT", "route": route, "error": str(error)})
                        write_sha256sums(stage)
                        seal_tree(stage)
                        atomic_publish(stage, REMOTE_WORK / f"run-{route}.failed-{time.time_ns()}")
                raise
        aggregation = aggregate_matrix()
        result = publish_terminal(aggregation.pop("verdict"), aggregation)
        print(json.dumps(result, sort_keys=True))


REMOTE_BOOTSTRAP = (
    "import hashlib,sys\n"
    "source=sys.stdin.buffer.read()\n"
    "assert hashlib.sha256(source).hexdigest()==sys.argv[1], 'controller SHA mismatch'\n"
    "action=sys.argv[2]\n"
    "payload=sys.argv[3]\n"
    "sys.argv=['lay-v10-c1-direct-latency.py',action,payload]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-c1-direct-latency.py>',"
    "'REMOTE_EXECUTION':True,'CONTROLLER_SOURCE_BYTES':source}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def remote_call(action: str, payload: dict[str, Any] | None = None, timeout: int = 30) -> subprocess.CompletedProcess[bytes]:
    source = pathlib.Path(__file__).read_bytes()
    encoded = "" if payload is None else base64.b64encode(pretty_json(payload)).decode()
    command = shlex.join(["/usr/bin/python3", "-c", REMOTE_BOOTSTRAP, sha256_bytes(source), action, encoded])
    return subprocess.run(
        [
            "/usr/bin/ssh",
            "-i",
            str(pathlib.Path.home() / ".ssh/mega-mini-admin"),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            REMOTE,
            command,
        ],
        input=source,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )


def remote_json(action: str, payload: dict[str, Any] | None = None, timeout: int = 30) -> dict[str, Any]:
    result = remote_call(action, payload, timeout)
    require(result.returncode == 0, f"remote {action} failed: {result.stderr[-4000:]!r}")
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise C1Error(f"remote {action} returned invalid JSON: {result.stdout[-2000:]!r}") from error
    require(isinstance(value, dict), f"remote {action} returned a non-object")
    return value


def self_check() -> None:
    verify_local_admission()
    v10 = pathlib.Path("/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/artifacts/v13_typed_peak.v10.rs").read_bytes()
    fragment = FRAGMENT.read_bytes()
    assembled = assemble_source(v10, fragment)
    source = pathlib.Path(__file__).read_text()
    rust = fragment.decode()
    sample_region = rust.split("fn c1_sample(", 1)[1].split("fn c1_warmup", 1)[0]
    for forbidden in ("serde_json", "Sha256", "full_row", "target_surface", "Vec::with_capacity"):
        require(forbidden not in sample_region, f"measured sample region contains {forbidden}")
    require("result.search_elapsed_us" in sample_region and "result.total_elapsed_us" in sample_region, "production timings absent")
    require("std::hint::black_box(&outcome)" in sample_region, "result observability absent")
    require("C1_SINGLE_ROUNDS: usize = 100" in rust and "C1_TWENTY_ROUNDS: usize = 250" in rust, "round constants drift")
    require("Barrier::new(C1_WORKERS + 1)" in rust, "T barriers absent")
    forbidden_controller = ["/usr/bin/" + "perf", "perf_event_" + "open", "system" + "ctl", "pk" + "ill", "kill" + "all", "cpu" + "power"]
    require(not any(token in source for token in forbidden_controller), "forbidden controller token")
    require(nearest_rank(range(1, 501), 99) == 495, "nearest-rank n=500 mismatch")
    fixture = SAMPLE_STRUCT.pack(7, 9, 255, 0, 123, 456)
    with tempfile.TemporaryDirectory(prefix="lay-c1-self-check-") as directory:
        path = pathlib.Path(directory) / "samples.bin"
        path.write_bytes(fixture)
        records = parse_samples(path)
        require(records == [{"query_ordinal": 7, "round": 9, "worker_id": 255, "flags": 0, "search_elapsed_us": 123, "total_elapsed_us": 456}], "sample parser mismatch")
        stage = pathlib.Path(directory) / "stage"
        stage.mkdir()
        write_new_json(stage / "receipt.json", {"pass": True})
        write_sha256sums(stage)
        seal_tree(stage)
        verify_sha256sums(stage)
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "checks": 31,
                "production_prefix_sha256": sha256_bytes(assembled[:39_047]),
                "final_source_sha256": sha256_bytes(assembled),
                "fragment_sha256": sha256_bytes(fragment),
                "tests": TESTS,
                "remote_actions": 0,
                "cargo_invocations": 0,
                "subject_executions": 0,
                "perf_invocations": 0,
            },
            sort_keys=True,
        )
    )


def local_status() -> None:
    verify_local_admission()
    print(json.dumps(remote_json("remote-status"), indent=2, sort_keys=True))


def local_prepare() -> None:
    verify_local_admission()
    require(not LOCAL_FINAL.exists(), "local C1 final already exists")
    before = remote_json("remote-status")
    require(before.get("final_exists") is False and before.get("work_exists") is False and before.get("state_exists") is False, "C1 remote path is not absent")
    fragment = FRAGMENT.read_bytes()
    source = pathlib.Path(__file__).read_bytes()
    value = remote_json(
        "remote-prepare",
        {"fragment_b64": base64.b64encode(fragment).decode(), "fragment_sha256": sha256_bytes(fragment), "controller_sha256": sha256_bytes(source)},
        timeout=60,
    )
    print(json.dumps(value, indent=2, sort_keys=True))


def local_build() -> None:
    verify_local_admission()
    value = remote_json("remote-build", timeout=1900)
    print(json.dumps(value, indent=2, sort_keys=True))


def local_parity() -> None:
    verify_local_admission()
    value = remote_json("remote-parity", timeout=600)
    print(json.dumps(value, indent=2, sort_keys=True))


def local_ready() -> None:
    verify_local_admission()
    value = remote_json("remote-ready", timeout=60)
    print(json.dumps(value, indent=2, sort_keys=True))


def fetch_remote_final() -> dict[str, Any]:
    command = shlex.join(["/usr/bin/python3", "-c", "import pathlib,sys;sys.stdout.buffer.write(pathlib.Path(sys.argv[1]).read_bytes())", str(REMOTE_FINAL / "C1_RESULT.json")])
    result = subprocess.run(
        ["/usr/bin/ssh", "-i", str(pathlib.Path.home() / ".ssh/mega-mini-admin"), "-o", "BatchMode=yes", REMOTE, command],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(result.returncode == 0, f"cannot fetch remote C1 result: {result.stderr[-2000:]!r}")
    return json.loads(result.stdout)


def local_index(remote_result: dict[str, Any], runtime_before: dict[str, Any], runtime_after: dict[str, Any]) -> None:
    require(not LOCAL_FINAL.exists(), "local C1 index already exists")
    require(runtime_before == runtime_after, "installed runtime changed during C1")
    stage = LOCAL_FINAL.with_name(f".{LOCAL_FINAL.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    write_new_json(stage / "REMOTE_C1_RESULT.json", remote_result)
    write_new_json(
        stage / "LOCAL_INDEX.json",
        {
            "schema": "lay.v10.c1-local-index.v1",
            "task_id": TASK_ID,
            "recorded_at": now(),
            "remote_verdict": remote_result["verdict"],
            "remote_result_sha256": sha256_bytes(pretty_json(remote_result)),
            "controller_sha256": sha256_file(pathlib.Path(__file__)),
            "fragment_sha256": sha256_file(FRAGMENT),
            "preflight_manifest_sha256": EXPECTED["preflight_manifest_file"],
            "preflight_receipt_sha256": EXPECTED["preflight_receipt"],
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        },
    )
    write_sha256sums(stage)
    seal_tree(stage)
    atomic_publish(stage, LOCAL_FINAL)


def local_run() -> None:
    verify_local_admission()
    runtime_before = local_runtime_snapshot()
    value = remote_json("remote-run", timeout=7200)
    remote_result = fetch_remote_final()
    require(value.get("verdict") == remote_result.get("verdict"), "remote run/final verdict mismatch")
    runtime_after = local_runtime_snapshot()
    local_index(remote_result, runtime_before, runtime_after)
    print(json.dumps({"verdict": remote_result["verdict"], "local_index": str(LOCAL_FINAL)}, indent=2, sort_keys=True))


def remote_dispatch(action: str, encoded: str) -> None:
    require(REMOTE_EXECUTION, f"{action} is not a local action")
    payload = json.loads(base64.b64decode(encoded, validate=True)) if encoded else {}
    if action == "remote-status":
        print(json.dumps(remote_status_value(), sort_keys=True))
    elif action == "remote-prepare":
        remote_prepare(payload)
    elif action == "remote-build":
        remote_build()
    elif action == "remote-parity":
        remote_parity()
    elif action == "remote-ready":
        remote_input_identities()
        status = remote_status_value()
        require(status.get("parity", {}).get("verdict") == "PASS", "C1 parity is not ready")
        print(json.dumps({"verdict": "READY_FOR_S1" if (admission := environment_admission())["verdict"] == "PASS" else "BLOCKED_ENVIRONMENT", "admission": admission, "remote_writes": False}, sort_keys=True))
    elif action == "remote-run":
        remote_run_matrix()
    else:
        raise C1Error(f"unknown remote action: {action}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "status", "prepare", "build", "parity", "ready", "run", "remote-status", "remote-prepare", "remote-build", "remote-parity", "remote-ready", "remote-run"))
    parser.add_argument("payload", nargs="?", default="")
    args = parser.parse_args()
    if args.action == "self-check":
        self_check()
    elif args.action == "status":
        local_status()
    elif args.action == "prepare":
        local_prepare()
    elif args.action == "build":
        local_build()
    elif args.action == "parity":
        local_parity()
    elif args.action == "ready":
        local_ready()
    elif args.action == "run":
        local_run()
    else:
        remote_dispatch(args.action, args.payload)


if __name__ == "__main__":
    try:
        main()
    except C1Error as error:
        print(json.dumps({"verdict": "ERROR", "error": str(error)}, sort_keys=True), file=sys.stderr)
        raise SystemExit(1)
