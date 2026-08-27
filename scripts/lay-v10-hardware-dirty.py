#!/usr/bin/env python3
"""One-shot dirty B5/B6 wall-time observation using the sealed V10 proxy."""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from typing import Any, Sequence


TASK_ID = "slice8b-v10-dirty-speed-v1-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
HARDWARE_TASK_ID = "slice8b-v10-hardware-b0-b2-v3-20260824"
REMOTE_PROVENANCE = pathlib.Path("/home/e/.local/share/lay/provenance")
REMOTE_HARDWARE = REMOTE_PROVENANCE / HARDWARE_TASK_ID
REMOTE_B0A = REMOTE_HARDWARE / "b0a-input-closure-v2"
REMOTE_BUILD = REMOTE_HARDWARE / "diagnostic-build-v1"
REMOTE_B0B = REMOTE_HARDWARE / "b0b-schedule-closure-v1"
REMOTE_B1 = REMOTE_HARDWARE / "b1-environment-v1"
REMOTE_FINAL = REMOTE_PROVENANCE / TASK_ID
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

ELF = REMOTE_BUILD / "diagnostic-test-elf"
PACKAGE = REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin"
SIDECAR = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa"
V7 = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json"
SCHEDULE = REMOTE_B0B / "query-schedule.json"

EXPECTED = {
    "elf": (20_542_920, "f7bcee37d5dffd583577d66c982c2a28072889b8aac2ccbed22612aa6d4feb09"),
    "build_id": "9829fb05f34bd353877fb6d71f1f8523e084af55",
    "package": (140_556_462, "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"),
    "sidecar": (3_689_884, "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd"),
    "v7": (1_606_189, "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"),
    "schedule": (174_941, "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78"),
    "b0a_receipt": (366_665, "48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3"),
    "b1_receipt": (229_467, "12c591b447f0025548e96272a4e8d5d4f23debc8458ae99fce61b2e751673925"),
}

TEST_NAMES = {
    "B5": "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_b5_proxy",
    "B6": "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_b6_proxy",
}
CPU_LISTS = {"B5": [0], "B6": list(range(20))}
REQUESTS = 382
PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_DIRTY_SPEED_V1_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_DIRTY_SPEED_PREFLIGHT_V1_2026-08-25.json"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_DIRTY_SPEED_V1_2026-08-25"
)


class DirtyError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise DirtyError(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def file_identity(path: pathlib.Path) -> dict[str, Any]:
    return {
        "path": str(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
        "mode": mode_string(path),
    }


def verify_file(path: pathlib.Path, size: int, digest: str) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
    identity = file_identity(path)
    require(identity["size_bytes"] == size, f"size mismatch: {path}")
    require(identity["sha256"] == digest, f"SHA-256 mismatch: {path}")
    return identity


def write_new_bytes(path: pathlib.Path, data: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any) -> None:
    write_new_bytes(path, (json.dumps(value, indent=2, sort_keys=True) + "\n").encode())


def prepare_marker(path: pathlib.Path, data: bytes) -> pathlib.Path:
    temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}-{time.time_ns()}")
    write_new_bytes(temporary, data)
    return temporary


def publish_marker(temporary: pathlib.Path, path: pathlib.Path) -> None:
    try:
        os.link(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def atomic_marker(path: pathlib.Path, data: bytes = b"ready\n") -> None:
    publish_marker(prepare_marker(path, data), path)


def consume_marker(route: str) -> pathlib.Path:
    REMOTE_STATE.mkdir(parents=True, exist_ok=True, mode=0o700)
    marker = REMOTE_STATE / f"{route.lower()}-consumed"
    write_new_json(
        marker,
        {
            "task_id": TASK_ID,
            "route": route,
            "consumed_at": now(),
            "retry_permitted": False,
        },
    )
    return marker


def seal_tree(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        if path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode())
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish(stage: pathlib.Path, final: pathlib.Path) -> None:
    require(not final.exists(), f"final output already exists: {final}")
    seal_tree(stage)
    os.rename(stage, final)


def read_text(path: pathlib.Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8", errors="replace").strip()
    except (FileNotFoundError, PermissionError, OSError):
        return None


def pressure(path: pathlib.Path) -> dict[str, dict[str, float]]:
    result: dict[str, dict[str, float]] = {}
    for line in (read_text(path) or "").splitlines():
        fields = line.split()
        result[fields[0]] = {
            key: float(value) for key, value in (field.split("=", 1) for field in fields[1:])
        }
    return result


def temperatures() -> list[dict[str, Any]]:
    values = []
    for path in sorted(pathlib.Path("/sys/class/hwmon").glob("hwmon*/temp*_input")):
        raw = read_text(path)
        if raw and raw.lstrip("-").isdigit():
            label = read_text(path.with_name(path.name.replace("_input", "_label")))
            values.append({"path": str(path), "label": label, "millidegrees_c": int(raw)})
    return values


def throttle_counters() -> dict[str, int]:
    values = {}
    for path in sorted(pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*")):
        raw = read_text(path)
        if raw and raw.isdigit():
            values[str(path)] = int(raw)
    return values


def procs_running() -> int | None:
    for line in (read_text(pathlib.Path("/proc/stat")) or "").splitlines():
        if line.startswith("procs_running "):
            return int(line.split()[1])
    return None


def light_sample() -> dict[str, Any]:
    thermal = temperatures()
    return {
        "monotonic_ns": time.perf_counter_ns(),
        "cpu_pressure": pressure(pathlib.Path("/proc/pressure/cpu")),
        "memory_pressure": pressure(pathlib.Path("/proc/pressure/memory")),
        "io_pressure": pressure(pathlib.Path("/proc/pressure/io")),
        "procs_running": procs_running(),
        "maximum_temperature_c": max((value["millidegrees_c"] for value in thermal), default=0)
        / 1000.0,
    }


def process_record(path: pathlib.Path) -> dict[str, Any] | None:
    try:
        stat_text = (path / "stat").read_text()
        tail = stat_text[stat_text.rfind(")") + 2 :].split()
        status = (path / "status").read_text()
        comm = (path / "comm").read_text().strip()
        cmdline = (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip()
        allowed = re.search(r"^Cpus_allowed_list:\s*(.+)$", status, re.MULTILINE)
        rss = re.search(r"^VmRSS:\s*(\d+)", status, re.MULTILINE)
        ticks = os.sysconf(os.sysconf_names["SC_CLK_TCK"])
        return {
            "pid": int(path.name),
            "comm": comm,
            "cmdline": cmdline,
            "cpu_seconds": (int(tail[11]) + int(tail[12])) / ticks,
            "rss_bytes": int(rss.group(1)) * 1024 if rss else None,
            "cpus_allowed_list": allowed.group(1).strip() if allowed else None,
        }
    except (FileNotFoundError, ProcessLookupError, PermissionError, IndexError, ValueError):
        return None


def tracked_processes(subject_pid: int | None = None) -> dict[int, dict[str, Any]]:
    records = {}
    for path in pathlib.Path("/proc").iterdir():
        if not path.name.isdigit():
            continue
        record = process_record(path)
        if record is None:
            continue
        text = f"{record['comm']} {record['cmdline']}".lower()
        if "nando" in text or record["comm"] == "btop" or record["pid"] == subject_pid:
            records[record["pid"]] = record
    return records


def process_deltas(
    before: dict[int, dict[str, Any]], after: dict[int, dict[str, Any]], wall_seconds: float
) -> list[dict[str, Any]]:
    values = []
    for pid in sorted(before.keys() & after.keys()):
        delta = max(0.0, after[pid]["cpu_seconds"] - before[pid]["cpu_seconds"])
        values.append(
            {
                "pid": pid,
                "comm": after[pid]["comm"],
                "cmdline": after[pid]["cmdline"],
                "cpu_seconds_delta": delta,
                "cpu_percent_during_window": delta / wall_seconds * 100.0 if wall_seconds else None,
            }
        )
    return sorted(values, key=lambda item: item["cpu_seconds_delta"], reverse=True)


def task_affinities(pid: int) -> list[dict[str, Any]]:
    values = []
    for task in sorted(pathlib.Path(f"/proc/{pid}/task").glob("[0-9]*")):
        try:
            status = (task / "status").read_text()
            allowed = re.search(r"^Cpus_allowed_list:\s*(.+)$", status, re.MULTILINE)
            values.append(
                {
                    "tid": int(task.name),
                    "comm": (task / "comm").read_text().strip(),
                    "cpus_allowed_list": allowed.group(1).strip() if allowed else None,
                }
            )
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return values


def stable_host_projection() -> dict[str, Any]:
    policies = []
    for root in sorted(pathlib.Path("/sys/devices/system/cpu/cpufreq").glob("policy*")):
        policies.append(
            {
                "policy": root.name,
                "affected_cpus": read_text(root / "affected_cpus"),
                "driver": read_text(root / "scaling_driver"),
                "governor": read_text(root / "scaling_governor"),
                "epp": read_text(root / "energy_performance_preference"),
                "min": read_text(root / "scaling_min_freq"),
                "max": read_text(root / "scaling_max_freq"),
            }
        )
    pstate_root = pathlib.Path("/sys/devices/system/cpu/intel_pstate")
    return {
        "hostname": os.uname().nodename,
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        "boot_id": read_text(pathlib.Path("/proc/sys/kernel/random/boot_id")),
        "kernel": os.uname().release,
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
        "smt_active": read_text(pathlib.Path("/sys/devices/system/cpu/smt/active")),
        "frequency_policy": policies,
        "pstate": {
            name: read_text(pstate_root / name)
            for name in ("status", "min_perf_pct", "max_perf_pct", "no_turbo")
        },
    }


def build_id(path: pathlib.Path) -> str:
    result = subprocess.run(
        ["/usr/bin/readelf", "-n", str(path)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(result.returncode == 0, f"readelf failed: {result.stderr[-1000:]!r}")
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", result.stdout)
    require(match is not None, "ELF Build ID is absent")
    return match.group(1).decode()


def verify_remote_inputs() -> dict[str, Any]:
    require(os.uname().nodename == REMOTE_HOSTNAME, "unexpected remote hostname")
    require(
        sha256_file(pathlib.Path("/etc/machine-id")) == REMOTE_MACHINE_ID_SHA256,
        "unexpected remote machine identity",
    )
    identities = {
        "elf": verify_file(ELF, *EXPECTED["elf"]),
        "package": verify_file(PACKAGE, *EXPECTED["package"]),
        "sidecar": verify_file(SIDECAR, *EXPECTED["sidecar"]),
        "v7": verify_file(V7, *EXPECTED["v7"]),
        "schedule": verify_file(SCHEDULE, *EXPECTED["schedule"]),
        "b0a_receipt": verify_file(REMOTE_B0A / "INPUT_CLOSURE.json", *EXPECTED["b0a_receipt"]),
        "b1_receipt": verify_file(REMOTE_B1 / "ENVIRONMENT_RECEIPT.json", *EXPECTED["b1_receipt"]),
    }
    require(identities["elf"]["mode"] == "0555", "diagnostic ELF mode mismatch")
    identities["elf"]["build_id"] = build_id(ELF)
    require(identities["elf"]["build_id"] == EXPECTED["build_id"], "diagnostic Build ID mismatch")
    closure = json.loads((REMOTE_B0B / "SCHEDULE_CLOSURE.json").read_text())
    require(closure.get("schedule_sha256") == EXPECTED["schedule"][1], "B0b schedule binding mismatch")
    require(closure.get("entries") == REQUESTS, "B0b entry count mismatch")
    require(closure.get("diagnostic_executable_sha256") == EXPECTED["elf"][1], "B0b ELF binding mismatch")
    b1 = json.loads((REMOTE_B1 / "ENVIRONMENT_RECEIPT.json").read_text())
    require(b1.get("verdict") == "BLOCKED_ENVIRONMENT", "B1 is not the preserved blocked receipt")
    return identities


def subject_environment(route: str, route_root: pathlib.Path) -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "LOGNAME": "e",
        "PATH": "/home/e/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "USER": "e",
        "RUST_TEST_THREADS": "1",
        "LAY_V10_HW_V13_PACKAGE": str(PACKAGE),
        "LAY_V10_HW_SIDECAR": str(SIDECAR),
        "LAY_V10_HW_V7": str(V7),
        "LAY_V10_HW_SCHEDULE": str(SCHEDULE),
        "LAY_V10_HW_CONTROL_DIR": str(route_root / "control"),
        "LAY_V10_HW_CPU_LIST": ",".join(str(cpu) for cpu in CPU_LISTS[route]),
        "LAY_V10_HW_SUBJECT_RECEIPT": str(route_root / "subject-receipt.json"),
    }


def terminate_owned(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, 15)
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=3)
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, 9)
        process.wait()


def wait_for_ready(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        require(process.poll() is None, f"subject exited before ready with {process.returncode}")
        time.sleep(0.002)
    raise DirtyError("subject-ready timeout")


def validate_subject(route: str, value: dict[str, Any]) -> None:
    require("error" not in value, f"{route} subject error: {value.get('error')}")
    require(value.get("route") == route, f"{route} route mismatch")
    require(value.get("schedule_sha256") == EXPECTED["schedule"][1], f"{route} schedule mismatch")
    require(value.get("requests") == REQUESTS, f"{route} request count mismatch")
    require(value.get("in_process_latency_observed") is False, f"{route} latency flag mismatch")
    if route == "B5":
        require(len(value.get("queries", [])) == REQUESTS, "B5 query output count mismatch")
    else:
        workers = value.get("worker_queries", [])
        require(len(workers) == 20, "B6 worker count mismatch")
        require(sum(len(worker) for worker in workers) == REQUESTS, "B6 query output count mismatch")


def summarize_samples(samples: list[dict[str, Any]]) -> dict[str, Any]:
    def values(pointer: Sequence[str]) -> list[float]:
        result = []
        for sample in samples:
            current: Any = sample
            for key in pointer:
                current = current.get(key, {}) if isinstance(current, dict) else {}
            if isinstance(current, (int, float)):
                result.append(float(current))
        return result

    def span(pointer: Sequence[str]) -> dict[str, float | None]:
        observed = values(pointer)
        return {"minimum": min(observed) if observed else None, "maximum": max(observed) if observed else None}

    return {
        "samples": len(samples),
        "cpu_psi_some_avg10": span(("cpu_pressure", "some", "avg10")),
        "memory_psi_full_avg10": span(("memory_pressure", "full", "avg10")),
        "io_psi_full_avg10": span(("io_pressure", "full", "avg10")),
        "procs_running": span(("procs_running",)),
        "maximum_temperature_c": span(("maximum_temperature_c",)),
    }


def run_route(route: str, stage: pathlib.Path) -> dict[str, Any]:
    route_root = stage / route.lower()
    control = route_root / "control"
    control.mkdir(parents=True, mode=0o700)
    marker = consume_marker(route)
    command = [
        str(ELF),
        "--exact",
        TEST_NAMES[route],
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]
    process: subprocess.Popen[bytes] | None = None
    stdout = b""
    stderr = b""
    try:
        process = subprocess.Popen(
            command,
            env=subject_environment(route, route_root),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        wait_for_ready(process, control / "subject-ready", 180.0)
        affinities = task_affinities(process.pid)
        pinned = sorted(
            int(item["cpus_allowed_list"])
            for item in affinities
            if item["cpus_allowed_list"] and item["cpus_allowed_list"].isdigit()
        )
        require(pinned == CPU_LISTS[route], f"{route} pinned task affinity mismatch: {pinned}")
        processes_before = tracked_processes(process.pid)
        throttles_before = throttle_counters()
        samples = [light_sample()]
        enabled_path = control / "controller-enabled"
        enabled_temporary = prepare_marker(enabled_path, b"enabled\n")
        start_ns = time.perf_counter_ns()
        publish_marker(enabled_temporary, enabled_path)
        next_sample = time.monotonic()
        deadline = time.monotonic() + 120.0
        while not (control / "subject-done").is_file():
            require(process.poll() is None, f"{route} subject exited during measured window")
            current = time.monotonic()
            if current >= next_sample:
                samples.append(light_sample())
                next_sample = current + 0.02
            require(current < deadline, f"{route} measured window timeout")
            time.sleep(0.0005)
        end_ns = time.perf_counter_ns()
        samples.append(light_sample())
        processes_after = tracked_processes(process.pid)
        throttles_after = throttle_counters()
        atomic_marker(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=60)
        require(process.returncode == 0, f"{route} subject exited {process.returncode}: {stderr[-2000:]!r}")
        write_new_bytes(route_root / "stdout.log", stdout)
        write_new_bytes(route_root / "stderr.log", stderr)
        subject = json.loads((route_root / "subject-receipt.json").read_text())
        validate_subject(route, subject)
        wall_ns = end_ns - start_ns
        wall_seconds = wall_ns / 1_000_000_000.0
        background_before = {pid: value for pid, value in processes_before.items() if pid != process.pid}
        background_after = {pid: value for pid, value in processes_after.items() if pid != process.pid}
        subject_delta = process_deltas(
            {process.pid: processes_before[process.pid]},
            {process.pid: processes_after[process.pid]},
            wall_seconds,
        )
        route_receipt = {
            "schema": "lay.v10.dirty-speed-route.v1",
            "task_id": TASK_ID,
            "route": route,
            "verdict": "DIRTY_DIAGNOSTIC_OBSERVED",
            "measured_at": now(),
            "command": command,
            "marker": str(marker),
            "cpus": CPU_LISTS[route],
            "task_affinities_at_ready": affinities,
            "pinned_subject_task_cpus": pinned,
            "requests": REQUESTS,
            "workers": 1 if route == "B5" else 20,
            "wall_window_ns": wall_ns,
            "wall_window_ms": wall_ns / 1_000_000.0,
            "effective_wall_ms_per_request": wall_ns / 1_000_000.0 / REQUESTS,
            "throughput_requests_per_second": REQUESTS / wall_seconds,
            "subject_proc_stat_cpu_delta": subject_delta[0] if subject_delta else None,
            "controller_start_sync_poll_upper_bound_ms": 10.0,
            "controller_done_observation_poll_upper_bound_ms": 0.5,
            "per_query_latency_observed": False,
            "per_query_p99_observed": False,
            "sample_summary": summarize_samples(samples),
            "samples": samples,
            "background_process_cpu_deltas": process_deltas(
                background_before, background_after, wall_seconds
            ),
            "background_processes_before": list(background_before.values()),
            "background_processes_after": list(background_after.values()),
            "throttle_counters_unchanged": throttles_before == throttles_after,
            "throttle_counters_before": throttles_before,
            "throttle_counters_after": throttles_after,
            "subject_receipt_sha256": sha256_file(route_root / "subject-receipt.json"),
            "formal_b_pass": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        }
        write_new_json(route_root / "DIRTY_ROUTE_RECEIPT.json", route_receipt)
        return route_receipt
    except Exception:
        if process is not None:
            terminate_owned(process)
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
            with contextlib.suppress(Exception):
                write_new_bytes(route_root / "stdout.log", stdout)
            with contextlib.suppress(Exception):
                write_new_bytes(route_root / "stderr.log", stderr)
        raise


def remote_run() -> None:
    require(not REMOTE_FINAL.exists(), f"dirty result already exists: {REMOTE_FINAL}")
    stage = REMOTE_PROVENANCE / f".{TASK_ID}.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    result: dict[str, Any] = {
        "schema": "lay.v10.dirty-speed.v1",
        "task_id": TASK_ID,
        "started_at": now(),
        "verdict": "ERROR",
        "claim": "DIRTY_DIAGNOSTIC_ONLY",
        "environment_intentionally_dirty": True,
        "formal_b_pass": False,
        "b1_unchanged": True,
        "b1_state": "BLOCKED_ENVIRONMENT",
        "b2_executed": False,
        "b3_executed": False,
        "proxy_semantic_parity_executed": False,
        "quality": "UNKNOWN",
        "v12_admitted": False,
        "user_product_decision_required": True,
        "perf_invoked": False,
        "pmu_event_opened": False,
        "cargo_invoked": False,
        "host_policy_modified": False,
        "foreign_process_stopped": False,
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    failure: Exception | None = None
    try:
        result["inputs"] = verify_remote_inputs()
        result["stable_host_before"] = stable_host_projection()
        result["throttle_counters_before"] = throttle_counters()
        result["background_before"] = list(tracked_processes().values())
        result["routes"] = {
            "B5": run_route("B5", stage),
            "B6": run_route("B6", stage),
        }
        result["background_after"] = list(tracked_processes().values())
        result["throttle_counters_after"] = throttle_counters()
        result["stable_host_after"] = stable_host_projection()
        result["stable_host_unchanged"] = result["stable_host_before"] == result["stable_host_after"]
        result["throttle_counters_unchanged"] = (
            result["throttle_counters_before"] == result["throttle_counters_after"]
        )
        require(result["stable_host_unchanged"], "stable host projection changed")
        require(result["throttle_counters_unchanged"], "thermal throttle counters changed")
        result["verdict"] = "DIRTY_DIAGNOSTIC_OBSERVED"
    except Exception as error:
        failure = error
        result["error"] = str(error)
    result["completed_at"] = now()
    write_new_json(stage / "DIRTY_SPEED_RECEIPT.json", result)
    publish(stage, REMOTE_FINAL)
    print(
        json.dumps(
            {
                "verdict": result["verdict"],
                "remote_receipt": str(REMOTE_FINAL / "DIRTY_SPEED_RECEIPT.json"),
                "error": result.get("error"),
            },
            sort_keys=True,
        )
    )
    if failure is not None:
        raise SystemExit(1)


def run_local(command: Sequence[str], *, input_bytes: bytes | None = None) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        list(command),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def local_runtime_snapshot() -> dict[str, Any]:
    active_v11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
    processes = []
    result = run_local(["/usr/bin/pgrep", "-a", "-f", "^(ibus-daemon|lay-daemon|lay-ibus-engine)( |$)"])
    if result.returncode in (0, 1):
        processes = result.stdout.decode(errors="replace").splitlines()
    version = run_local([str(pathlib.Path.home() / ".local/bin/lay"), "--version"])
    return {
        "active_v11_sha256": sha256_file(active_v11),
        "lay_version_stdout": version.stdout.decode(errors="replace").strip(),
        "lay_version_stderr": version.stderr.decode(errors="replace").strip(),
        "lay_processes": processes,
    }


def run_remote_and_publish_local() -> None:
    require(PREFLIGHT.is_file() and PREFLIGHT_RECEIPT.is_file(), "dirty preflight evidence is absent")
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "dirty preflight did not pass")
    require(preflight.get("manifest_sha256") == "7af194e7ce8b3c790095178d332b2ec80aa38983eb3dc86d90f4332959aca58d", "dirty preflight identity mismatch")
    require(not LOCAL_RESULT.exists(), f"local dirty result already exists: {LOCAL_RESULT}")
    source = pathlib.Path(__file__).read_bytes()
    controller_sha256 = sha256_bytes(source)
    local_before = local_runtime_snapshot()
    process = run_local(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/python3", "-", "remote-run"],
        input_bytes=source,
    )
    remote_receipt_result = run_local(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/cat", str(REMOTE_FINAL / "DIRTY_SPEED_RECEIPT.json")]
    )
    require(remote_receipt_result.returncode == 0, f"remote receipt unavailable: {remote_receipt_result.stderr[-2000:]!r}")
    remote_receipt = json.loads(remote_receipt_result.stdout)
    local_after = local_runtime_snapshot()
    require(local_before == local_after, "local Lay runtime or active V11 changed during remote dirty run")
    require(process.returncode == 0, f"remote dirty run failed: {process.stdout[-2000:]!r} {process.stderr[-2000:]!r}")
    require(remote_receipt.get("verdict") == "DIRTY_DIAGNOSTIC_OBSERVED", "remote dirty run did not complete")
    stage = LOCAL_RESULT.with_name(f".{LOCAL_RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    write_new_bytes(stage / "REMOTE_RECEIPT.json", remote_receipt_result.stdout)
    write_new_json(
        stage / "LOCAL_RUN_RECEIPT.json",
        {
            "schema": "lay.v10.dirty-speed-local-index.v1",
            "task_id": TASK_ID,
            "recorded_at": now(),
            "controller_sha256": controller_sha256,
            "preflight_manifest_sha256": preflight["manifest_sha256"],
            "remote_receipt_path": str(REMOTE_FINAL / "DIRTY_SPEED_RECEIPT.json"),
            "remote_receipt_sha256": sha256_bytes(remote_receipt_result.stdout),
            "ssh_stdout": process.stdout.decode(errors="replace"),
            "ssh_stderr": process.stderr.decode(errors="replace"),
            "local_runtime_before": local_before,
            "local_runtime_after": local_after,
            "formal_b_pass": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        },
    )
    publish(stage, LOCAL_RESULT)
    compact = {
        route: {
            key: remote_receipt["routes"][route][key]
            for key in (
                "wall_window_ms",
                "effective_wall_ms_per_request",
                "throughput_requests_per_second",
                "sample_summary",
                "background_process_cpu_deltas",
            )
        }
        for route in ("B5", "B6")
    }
    print(json.dumps({"verdict": remote_receipt["verdict"], "routes": compact}, indent=2, sort_keys=True))


def self_check() -> None:
    require(set(TEST_NAMES) == {"B5", "B6"}, "unexpected route matrix")
    require(CPU_LISTS["B5"] == [0] and CPU_LISTS["B6"] == list(range(20)), "CPU matrix drift")
    require(REQUESTS == 382, "request denominator drift")
    source = pathlib.Path(__file__).read_text()
    forbidden = [
        "/usr/bin/" + "perf",
        "cargo" + " build",
        "cargo" + " test",
        "cargo" + " check",
        "system" + "ctl",
        "cpu" + "power",
        "pk" + "ill",
        "kill" + "all",
    ]
    require(not any(token in source for token in forbidden), "forbidden command token in controller")
    fake_b5 = {
        "route": "B5",
        "schedule_sha256": EXPECTED["schedule"][1],
        "requests": REQUESTS,
        "queries": [{}] * REQUESTS,
        "in_process_latency_observed": False,
    }
    fake_b6 = {
        "route": "B6",
        "schedule_sha256": EXPECTED["schedule"][1],
        "requests": REQUESTS,
        "worker_queries": [[{}] * 20 for _ in range(19)] + [[{}] * 2],
        "in_process_latency_observed": False,
    }
    validate_subject("B5", fake_b5)
    validate_subject("B6", fake_b6)
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        global REMOTE_STATE
        original_state = REMOTE_STATE
        REMOTE_STATE = root / "state"
        try:
            consume_marker("B5")
            try:
                consume_marker("B5")
            except FileExistsError:
                pass
            else:
                raise DirtyError("one-run marker accepted a second attempt")
        finally:
            REMOTE_STATE = original_state
        stage = root / "stage"
        final = root / "final"
        stage.mkdir()
        write_new_bytes(stage / "value", b"ok\n")
        publish(stage, final)
        require(final.is_dir() and mode_string(final) == "0555", "atomic publication self-check failed")
        sleeper = subprocess.Popen(["/usr/bin/sleep", "30"], start_new_session=True)
        terminate_owned(sleeper)
        require(sleeper.poll() is not None, "owned-child cleanup self-check failed")
    print(json.dumps({"verdict": "PASS", "checks": 8, "remote_execution": False}, sort_keys=True))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "run", "remote-run"))
    args = parser.parse_args()
    if args.action == "self-check":
        self_check()
    elif args.action == "run":
        run_remote_and_publish_local()
    else:
        remote_run()


if __name__ == "__main__":
    main()
