#!/usr/bin/env python3
"""One-shot loaded PMU diagnosis for the sealed V10 executor proxy."""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
import time
import types
from typing import Any, Sequence


TASK_ID = "slice8b-v10-loaded-pmu-diagnosis-v4-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_PROVENANCE = pathlib.Path("/home/e/.local/share/lay/provenance")
REMOTE_FINAL = REMOTE_PROVENANCE / TASK_ID
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
REMOTE_EXECUTION = bool(globals().get("REMOTE_EXECUTION", False))
BASE_SOURCE_BYTES = globals().get("BASE_SOURCE_BYTES")
CONTROLLER_SOURCE_BYTES = globals().get("CONTROLLER_SOURCE_BYTES")

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-hardware-dirty.py"
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_V8_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_V8_PREFLIGHT_2026-08-25.json"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_V4_2026-08-25"
)

PARITY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_semantic_parity"
GROUPS = {
    "G2C": [
        "L1-dcache-loads",
        "LLC-loads",
        "LLC-load-misses",
    ],
    "G3": ["dTLB-loads", "dTLB-load-misses"],
}
ORDER = [(group, route) for group in GROUPS for route in ("B5", "B6")]
EXPECTED_PREFLIGHT_MANIFEST = "3d21c997d10ea39adf1c8e5ca52fd4176afe8b71e15a6b1b9fd17e85418b8398"


class ProfileError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ProfileError(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def load_base(source: bytes) -> types.ModuleType:
    module = types.ModuleType("lay_v10_loaded_profile_base")
    module.__file__ = "/opt/lay-evidence/lay-v10-hardware-dirty.py"
    exec(compile(source, module.__file__, "exec"), module.__dict__)
    return module


def open_fifo(path: pathlib.Path, flags: int, deadline: float) -> int:
    while time.monotonic() < deadline:
        try:
            return os.open(path, flags | os.O_NONBLOCK)
        except OSError as error:
            if error.errno not in (6, 11):
                raise
            time.sleep(0.005)
    raise ProfileError(f"FIFO open timeout: {path}")


def read_fifo_line(descriptor: int, deadline: float) -> str:
    value = bytearray()
    while time.monotonic() < deadline:
        try:
            chunk = os.read(descriptor, 4096)
        except BlockingIOError:
            chunk = b""
        if chunk:
            value.extend(chunk)
            if b"\n" in value:
                return bytes(value).decode(errors="replace").strip()
        time.sleep(0.002)
    raise ProfileError("perf control acknowledgement timeout")


def wait_for_file(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        require(process.poll() is None, f"owned process exited before {path.name}")
        time.sleep(0.002)
    raise ProfileError(f"timeout waiting for {path}")


def terminate_owned(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, 15)
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=3)
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, 9)
        process.wait()


def event_pmu(observed: str, expected: str) -> str | None:
    normalized = observed.strip().lower().replace(":u", "")
    expected = expected.lower()
    if normalized == expected:
        return None
    match = re.fullmatch(r"cpu_(core|atom)/([^/]+)/", normalized)
    if match is not None and match.group(2) == expected:
        return match.group(1)
    return "unknown"


def event_matches(observed: str, expected: str) -> bool:
    return event_pmu(observed, expected) != "unknown"


def numeric_counter(value: Any) -> float | None:
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str):
        return None
    compact = value.strip().replace(",", "")
    if compact.startswith("<") or compact in ("", "not counted", "not supported"):
        return None
    try:
        return float(compact)
    except ValueError:
        return None


def parse_perf_json(
    raw: bytes,
    expected_events: Sequence[str],
    required_pmus: Sequence[str] = (),
) -> dict[str, Any]:
    rows = []
    diagnostics = []
    for line in raw.decode(errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            if line.strip():
                diagnostics.append(line)
            continue
        if isinstance(value, dict) and isinstance(value.get("event"), str):
            rows.append(value)
        elif line.strip():
            diagnostics.append(line)
    require(rows, "perf produced no JSON counter rows")
    counters: dict[str, Any] = {}
    for expected in expected_events:
        matched = [row for row in rows if event_matches(row["event"], expected)]
        require(matched, f"perf event missing: {expected}")
        counted = []
        inactive = []
        for row in matched:
            value = numeric_counter(row.get("counter-value"))
            running = numeric_counter(row.get("pcnt-running"))
            event_runtime = numeric_counter(row.get("event-runtime"))
            pmu = event_pmu(row["event"], expected)
            require(pmu != "unknown", f"unknown PMU event: {row['event']}")
            if value is not None:
                require(running is not None, f"perf running percentage absent: {expected}")
                require(event_runtime is not None and event_runtime > 0, f"perf event runtime absent: {expected}")
                counted.append(
                    {
                        "row": row,
                        "value": value,
                        "running": running,
                        "event_runtime": event_runtime,
                        "pmu": pmu,
                    }
                )
                continue
            counter_text = str(row.get("counter-value", "")).strip().lower()
            require(counter_text == "<not counted>", f"perf event unsupported: {expected}")
            require(event_runtime == 0 and running == 0, f"inactive PMU row has runtime: {expected}")
            inactive.append({"row": row, "pmu": pmu})
        require(counted, f"perf event not counted: {expected}")
        qualified = any(event_pmu(row["event"], expected) is not None for row in matched)
        counted_pmus = sorted({item["pmu"] for item in counted if item["pmu"] is not None})
        if qualified:
            missing_pmus = sorted(set(required_pmus) - set(counted_pmus))
            require(not missing_pmus, f"required PMU not counted for {expected}: {missing_pmus}")
            require(set(counted_pmus) == set(required_pmus), f"unexpected counted PMU for {expected}")
            require(len(counted) == len(counted_pmus), f"duplicate counted PMU row for {expected}")
        perf_reported_value = sum(item["value"] for item in counted)
        runtime_weights: dict[str, float] = {}
        if qualified and len(required_pmus) > 1:
            total_runtime = sum(item["event_runtime"] for item in counted)
            require(total_runtime > 0, f"hybrid runtime absent: {expected}")
            running_sum = sum(item["running"] for item in counted)
            require(98.9 <= running_sum <= 101.1, f"hybrid running partition incomplete: {expected}")
            effective_value = 0.0
            for item in counted:
                weight = item["event_runtime"] / total_runtime
                expected_running = 100.0 * weight
                require(
                    abs(item["running"] - expected_running) <= 1.1,
                    f"hybrid running percentage disagrees with runtime: {expected}",
                )
                runtime_weights[str(item["pmu"])] = weight
                effective_value += item["value"] * weight
            aggregate_method = "hybrid-runtime-weighted"
        else:
            require(
                all(abs(item["running"] - 100.0) <= 0.01 for item in counted),
                f"perf event scaled: {expected}",
            )
            effective_value = perf_reported_value
            if qualified:
                runtime_weights[str(counted[0]["pmu"])] = 1.0
            aggregate_method = "single-pmu-or-unqualified-exact"
        counters[expected] = {
            "value": effective_value,
            "perf_reported_value_sum": perf_reported_value,
            "aggregate_method": aggregate_method,
            "runtime_weights": runtime_weights,
            "rows": matched,
            "counted_rows": [item["row"] for item in counted],
            "inactive_rows": [item["row"] for item in inactive],
            "required_pmus": list(required_pmus) if qualified else [],
            "counted_pmus": counted_pmus,
            "minimum_running_percent": min(item["running"] for item in counted),
        }
    unknown = [
        row["event"]
        for row in rows
        if not any(event_matches(row["event"], expected) for expected in expected_events)
    ]
    require(not unknown, f"perf produced unknown event rows: {unknown}")
    return {"counters": counters, "diagnostics": diagnostics, "raw_rows": rows}


def derived_metrics(group: str, counters: dict[str, Any], requests: int) -> dict[str, Any]:
    values = {key: item["value"] for key, item in counters.items()}
    result = {f"{key}_per_request": value / requests for key, value in values.items()}
    if group == "G2C":
        result["llc_load_miss_rate"] = (
            values["LLC-load-misses"] / values["LLC-loads"] if values["LLC-loads"] else None
        )
    elif group == "G3":
        result["dtlb_load_miss_rate"] = (
            values["dTLB-load-misses"] / values["dTLB-loads"]
            if values["dTLB-loads"]
            else None
        )
    return result


def perf_command(
    events: Sequence[str],
    control_fifo: pathlib.Path,
    ack_fifo: pathlib.Path,
    child: Sequence[str],
) -> list[str]:
    return [
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/perf",
        "stat",
        "--json-output",
        "--no-big-num",
        "--delay=-1",
        f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event",
        ",".join(events),
        "--",
        *child,
    ]


def child_as_e(environment: dict[str, str], command: Sequence[str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, *command]


def consume_marker(base: types.ModuleType, name: str) -> pathlib.Path:
    available = REMOTE_STATE / "markers" / f"{name}.available"
    consumed = REMOTE_STATE / "markers" / f"{name}.consumed-before-exec"
    require(available.is_file() and not consumed.exists(), f"profile marker unavailable: {name}")
    os.rename(available, consumed)
    return consumed


def initialize_state(base: types.ModuleType) -> None:
    require(not REMOTE_FINAL.exists(), "loaded profile final already exists")
    require(not REMOTE_STATE.exists(), "loaded profile state already exists")
    marker_root = REMOTE_STATE / "markers"
    marker_root.mkdir(parents=True, mode=0o700)
    names = ["parity", "capability", *(f"{route.lower()}-{group.lower()}" for group, route in ORDER)]
    for name in names:
        base.write_new_json(
            marker_root / f"{name}.available",
            {"task_id": TASK_ID, "route": name, "available_at": now(), "retry_permitted": False},
        )


def run_parity(base: types.ModuleType, stage: pathlib.Path) -> dict[str, Any]:
    root = stage / "parity"
    root.mkdir(mode=0o700)
    marker = consume_marker(base, "parity")
    receipt_path = root / "PARITY_RECEIPT.json"
    environment = base.subject_environment("B5", root)
    environment["LAY_V10_HW_PARITY_RECEIPT"] = str(receipt_path)
    command = [
        str(base.ELF),
        "--exact",
        PARITY_TEST,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]
    process: subprocess.Popen[bytes] | None = None
    try:
        process = subprocess.Popen(
            command,
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        stdout, stderr = process.communicate(timeout=900)
        base.write_new_bytes(root / "stdout.log", stdout)
        base.write_new_bytes(root / "stderr.log", stderr)
        require(process.returncode == 0, f"parity exited {process.returncode}: {stderr[-2000:]!r}")
        receipt = json.loads(receipt_path.read_text())
        expected = {
            "verdict": "PASS",
            "records": 382,
            "terminal_mismatches": 0,
            "peak_mismatches": 0,
            "completeness_mismatches": 0,
            "work_mismatches": 0,
            "full_row_mismatches": 0,
            "target_form_retained": 382,
            "target_lemma_retained": 382,
            "false_certificates": 0,
            "maximum_product_states": 35_590,
            "maximum_scratch_bytes": 6_656,
        }
        for key, value in expected.items():
            require(receipt.get(key) == value, f"parity mismatch: {key}")
        wrapper = {
            "schema": "lay.v10.loaded-pmu-parity-wrapper.v1",
            "verdict": "PASS",
            "marker": str(marker),
            "command": command,
            "receipt_sha256": base.sha256_file(receipt_path),
            "receipt": receipt,
            "perf_invoked": False,
            "runtime_authority_changed": False,
        }
        base.write_new_json(root / "PARITY_WRAPPER.json", wrapper)
        return wrapper
    except BaseException:
        terminate_owned(process)
        raise


def run_capability(base: types.ModuleType, stage: pathlib.Path) -> dict[str, Any]:
    root = stage / "capability"
    root.mkdir(mode=0o700)
    marker = consume_marker(base, "capability")
    control_fifo = root / "control.fifo"
    ack_fifo = root / "ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    ready = root / "workload-ready"
    go = root / "workload-go"
    done = root / "workload-done"
    stop = root / "workload-stop"
    code = (
        "import pathlib,time,sys;"
        "r=pathlib.Path(sys.argv[1]);g=pathlib.Path(sys.argv[2]);"
        "d=pathlib.Path(sys.argv[3]);s=pathlib.Path(sys.argv[4]);"
        "r.write_text('ready\\n');"
        "\nwhile not g.exists(): time.sleep(0.001);"
        "\nx=0\nfor i in range(2000000): x=(x+i)^((x<<1)&0xffffffff);"
        "\nd.write_text(str(x)+'\\n');"
        "\nwhile not s.exists(): time.sleep(0.001)"
    )
    child = child_as_e(
        {"HOME": "/home/e", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "PATH": "/usr/bin:/bin"},
        ["/usr/bin/taskset", "-c", "0", "/usr/bin/python3", "-c", code, str(ready), str(go), str(done), str(stop)],
    )
    command = perf_command(["cycles", "instructions"], control_fifo, ack_fifo, child)
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        wait_for_file(process, ready, 30.0)
        deadline = time.monotonic() + 5.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 5.0)
        base.atomic_marker(go, b"go\n")
        wait_for_file(process, done, 30.0)
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 5.0)
        base.atomic_marker(stop, b"stop\n")
        stdout, stderr = process.communicate(timeout=20)
        base.write_new_bytes(root / "stdout.log", stdout)
        base.write_new_bytes(root / "perf.raw", stderr)
        require(process.returncode == 0, f"benign perf exited {process.returncode}: {stderr[-2000:]!r}")
        parsed = parse_perf_json(stderr, ["cycles", "instructions"], required_pmus=("core",))
        receipt = {
            "schema": "lay.v10.loaded-pmu-capability.v1",
            "verdict": "PASS",
            "marker": str(marker),
            "command": command,
            "enable_ack": enable_ack,
            "disable_ack": disable_ack,
            "perf_version": subprocess.run(["/usr/bin/perf", "--version"], capture_output=True, text=True).stdout.strip(),
            "perf_event_paranoid": base.read_text(pathlib.Path("/proc/sys/kernel/perf_event_paranoid")),
            "kptr_restrict": base.read_text(pathlib.Path("/proc/sys/kernel/kptr_restrict")),
            "nmi_watchdog": base.read_text(pathlib.Path("/proc/sys/kernel/nmi_watchdog")),
            "counters": parsed["counters"],
            "raw_sha256": base.sha256_file(root / "perf.raw"),
            "pmu_event_opened": True,
            "runtime_authority_changed": False,
        }
        base.write_new_json(root / "CAPABILITY_RECEIPT.json", receipt)
        return receipt
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)
        terminate_owned(process)
        control_fifo.unlink(missing_ok=True)
        ack_fifo.unlink(missing_ok=True)


def descendants(root_pid: int) -> list[dict[str, Any]]:
    records: dict[int, tuple[int, dict[str, Any]]] = {}
    for path in pathlib.Path("/proc").glob("[0-9]*"):
        try:
            status = (path / "status").read_text()
            parent = re.search(r"^PPid:\s*(\d+)$", status, re.MULTILINE)
            allowed = re.search(r"^Cpus_allowed_list:\s*(.+)$", status, re.MULTILINE)
            record = {
                "pid": int(path.name),
                "comm": (path / "comm").read_text().strip(),
                "cmdline": (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip(),
                "cpus_allowed_list": allowed.group(1).strip() if allowed else None,
            }
            records[record["pid"]] = (int(parent.group(1)) if parent else 0, record)
        except (FileNotFoundError, ProcessLookupError, PermissionError, ValueError):
            continue
    selected = {root_pid}
    changed = True
    while changed:
        changed = False
        for pid, (parent, _) in records.items():
            if parent in selected and pid not in selected:
                selected.add(pid)
                changed = True
    return [records[pid][1] for pid in sorted(selected) if pid in records]


def run_profile_window(
    base: types.ModuleType,
    stage: pathlib.Path,
    group: str,
    route: str,
) -> dict[str, Any]:
    name = f"{route.lower()}-{group.lower()}"
    root = stage / name
    control = root / "control"
    control.mkdir(parents=True, mode=0o700)
    marker = consume_marker(base, name)
    control_fifo = root / "perf-control.fifo"
    ack_fifo = root / "perf-ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    environment = base.subject_environment(route, root)
    child = child_as_e(
        environment,
        [
            str(base.ELF),
            "--exact",
            base.TEST_NAMES[route],
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ],
    )
    command = perf_command(GROUPS[group], control_fifo, ack_fifo, child)
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    stdout = b""
    stderr = b""
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        wait_for_file(process, control / "subject-ready", 180.0)
        process_tree_at_ready = descendants(process.pid)
        before_processes = base.tracked_processes()
        throttles_before = base.throttle_counters()
        samples = [base.light_sample()]
        deadline = time.monotonic() + 5.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 5.0)
        enabled = control / "controller-enabled"
        temporary = base.prepare_marker(enabled, b"enabled\n")
        started_ns = time.perf_counter_ns()
        base.publish_marker(temporary, enabled)
        next_sample = time.monotonic()
        deadline = time.monotonic() + 180.0
        while not (control / "subject-done").is_file():
            require(process.poll() is None, f"{name} subject exited during executor window")
            current = time.monotonic()
            if current >= next_sample:
                samples.append(base.light_sample())
                next_sample = current + 0.02
            require(current < deadline, f"{name} executor window timeout")
            time.sleep(0.0005)
        ended_ns = time.perf_counter_ns()
        samples.append(base.light_sample())
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 5.0)
        base.atomic_marker(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=60)
        base.write_new_bytes(root / "stdout.log", stdout)
        base.write_new_bytes(root / "perf.raw", stderr)
        require(process.returncode == 0, f"{name} perf exited {process.returncode}: {stderr[-3000:]!r}")
        subject_path = root / "subject-receipt.json"
        subject = json.loads(subject_path.read_text())
        base.validate_subject(route, subject)
        required_pmus = ("core",) if route == "B5" else ("core", "atom")
        parsed = parse_perf_json(stderr, GROUPS[group], required_pmus=required_pmus)
        after_processes = base.tracked_processes()
        throttles_after = base.throttle_counters()
        wall_seconds = (ended_ns - started_ns) / 1_000_000_000.0
        receipt = {
            "schema": "lay.v10.loaded-pmu-profile-window.v1",
            "verdict": "LOADED_PMU_WINDOW_OBSERVED",
            "route": route,
            "group": group,
            "events": GROUPS[group],
            "marker": str(marker),
            "command": command,
            "enable_ack": enable_ack,
            "disable_ack": disable_ack,
            "requests": base.REQUESTS,
            "workers": 1 if route == "B5" else 20,
            "executor_window_wall_ns_diagnostic": ended_ns - started_ns,
            "process_tree_at_ready": process_tree_at_ready,
            "counters": parsed["counters"],
            "derived": derived_metrics(group, parsed["counters"], base.REQUESTS),
            "sample_summary": base.summarize_samples(samples),
            "background_process_cpu_deltas": base.process_deltas(before_processes, after_processes, wall_seconds),
            "throttle_counters_unchanged": throttles_before == throttles_after,
            "subject_receipt_sha256": base.sha256_file(subject_path),
            "raw_perf_sha256": base.sha256_file(root / "perf.raw"),
            "pmu_event_opened": True,
            "environment_intentionally_loaded": True,
            "formal_b_pass": False,
            "runtime_authority_changed": False,
        }
        require(receipt["throttle_counters_unchanged"], f"thermal throttle drift during {name}")
        base.write_new_json(root / "PROFILE_WINDOW_RECEIPT.json", receipt)
        return receipt
    except BaseException:
        if stdout and not (root / "stdout.log").exists():
            with contextlib.suppress(Exception):
                base.write_new_bytes(root / "stdout.log", stdout)
        if stderr and not (root / "perf.raw").exists():
            with contextlib.suppress(Exception):
                base.write_new_bytes(root / "perf.raw", stderr)
        raise
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)
        terminate_owned(process)
        control_fifo.unlink(missing_ok=True)
        ack_fifo.unlink(missing_ok=True)


def remote_run() -> None:
    require(REMOTE_EXECUTION, "remote-run is not a local action")
    require(isinstance(BASE_SOURCE_BYTES, bytes), "base controller bytes absent")
    base = load_base(BASE_SOURCE_BYTES)
    require(base.sha256_bytes(BASE_SOURCE_BYTES) == "f0aa4b55a803ef204d1d898cec70eb282757009a71ef5d9310417b2ac17a45e8", "base controller SHA mismatch")
    require(not REMOTE_FINAL.exists(), "remote loaded profile already exists")
    stage = REMOTE_PROVENANCE / f".{TASK_ID}.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    initialize_state(base)
    result: dict[str, Any] = {
        "schema": "lay.v10.loaded-pmu-diagnosis.v4",
        "task_id": TASK_ID,
        "started_at": now(),
        "verdict": "ERROR",
        "claim": "LOADED_EXECUTOR_PROXY_DIAGNOSTIC_ONLY",
        "environment_intentionally_loaded": True,
        "third_dirty_latency_run": False,
        "formal_b_pass": False,
        "v12_admitted": False,
        "runtime_authority_changed": False,
    }
    failure: BaseException | None = None
    try:
        result["inputs"] = base.verify_remote_inputs()
        result["stable_host_before"] = base.stable_host_projection()
        result["background_before"] = list(base.tracked_processes().values())
        result["parity"] = run_parity(base, stage)
        result["capability"] = run_capability(base, stage)
        windows = {}
        for group, route in ORDER:
            windows[f"{route}-{group}"] = run_profile_window(base, stage, group, route)
        result["windows"] = windows
        result["background_after"] = list(base.tracked_processes().values())
        result["stable_host_after"] = base.stable_host_projection()
        result["stable_host_unchanged"] = result["stable_host_before"] == result["stable_host_after"]
        require(result["stable_host_unchanged"], "stable host projection changed")
        result["verdict"] = "LOADED_PMU_DIAGNOSTIC_OBSERVED"
    except BaseException as error:
        failure = error
        result["error"] = str(error)
    result["completed_at"] = now()
    base.write_new_json(stage / "PROFILE_RESULT.json", result)
    base.publish(stage, REMOTE_FINAL)
    base.seal_tree(REMOTE_STATE)
    print(json.dumps({"verdict": result["verdict"], "error": result.get("error"), "remote_final": str(REMOTE_FINAL)}, sort_keys=True))
    if failure is not None:
        raise SystemExit(1)


def remote_verify() -> None:
    require(REMOTE_EXECUTION, "remote-verify is not a local action")
    require(isinstance(BASE_SOURCE_BYTES, bytes), "base controller bytes absent")
    base = load_base(BASE_SOURCE_BYTES)
    final_count = verify_manifest(base, REMOTE_FINAL)
    state_count = verify_manifest(base, REMOTE_STATE)
    writable = [str(path) for root in (REMOTE_FINAL, REMOTE_STATE) for path in root.rglob("*") if path.stat().st_mode & 0o222]
    result = json.loads((REMOTE_FINAL / "PROFILE_RESULT.json").read_text())
    require(result.get("verdict") == "LOADED_PMU_DIAGNOSTIC_OBSERVED", "remote result did not pass")
    require(not writable, "remote evidence contains writable objects")
    print(json.dumps({"verdict": "PASS", "final_manifest_entries": final_count, "state_manifest_entries": state_count, "writable_objects": 0}, sort_keys=True))


def verify_manifest(base: types.ModuleType, root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"manifest absent: {manifest}")
    rows = [line for line in manifest.read_text().splitlines() if line]
    for row in rows:
        digest, relative = row.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest path absent: {path}")
        require(base.sha256_file(path) == digest, f"manifest digest mismatch: {path}")
    return len(rows)


def run_local(command: Sequence[str], input_bytes: bytes | None = None) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(list(command), input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)


def local_runtime_snapshot() -> dict[str, Any]:
    version = run_local([str(pathlib.Path.home() / ".local/bin/lay"), "--version"])
    processes = run_local(["/usr/bin/pgrep", "-af", r"(^|/)(ibus-daemon|lay-daemon|lay-ibus-engine)( |$)"])
    return {
        "lay_version": version.stdout.decode(errors="replace").strip(),
        "active_v11_sha256": load_base(BASE_CONTROLLER.read_bytes()).sha256_file(
            PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
        ),
        "expected_process_presence": processes.stdout.decode(errors="replace").splitlines(),
    }


def remote_program(action: str) -> bytes:
    source = pathlib.Path(__file__).read_bytes()
    base = BASE_CONTROLLER.read_bytes()
    controller_file = "/opt/lay-evidence/lay-v10-hardware-loaded-profile.py"
    return (
        b"import sys\n"
        + b"controller_file=" + repr(controller_file).encode() + b"\n"
        + b"controller_source=" + repr(source).encode() + b"\n"
        + b"scope={'__name__':'__main__','__file__':controller_file,'REMOTE_EXECUTION':True,"
        + b"'BASE_SOURCE_BYTES':" + repr(base).encode() + b",'CONTROLLER_SOURCE_BYTES':controller_source}\n"
        + b"sys.argv=[controller_file," + repr(action).encode() + b"]\n"
        + b"exec(compile(controller_source,controller_file,'exec'),scope)\n"
    )


def run_remote_and_publish_local() -> None:
    require(PREFLIGHT.is_file() and PREFLIGHT_RECEIPT.is_file(), "loaded profile preflight absent")
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "loaded profile preflight did not pass")
    require(preflight.get("safe_to_implement") is True, "loaded profile preflight is not safe")
    require(preflight.get("manifest_sha256") == EXPECTED_PREFLIGHT_MANIFEST, "preflight identity mismatch")
    require(not LOCAL_RESULT.exists(), "local loaded profile result already exists")
    before = local_runtime_snapshot()
    execution = run_local(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/python3", "-", "remote-run"],
        remote_program("remote-run"),
    )
    verification = run_local(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/python3", "-", "remote-verify"],
        remote_program("remote-verify"),
    )
    remote_result = run_local(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/cat", str(REMOTE_FINAL / "PROFILE_RESULT.json")]
    )
    after = local_runtime_snapshot()
    require(before == after, "local runtime stable projection changed")
    require(execution.returncode == 0, f"remote profile failed: {execution.stdout[-3000:]!r} {execution.stderr[-3000:]!r}")
    require(verification.returncode == 0, f"remote verification failed: {verification.stdout[-3000:]!r} {verification.stderr[-3000:]!r}")
    require(remote_result.returncode == 0, "remote profile result unavailable")
    result_value = json.loads(remote_result.stdout)
    require(result_value.get("verdict") == "LOADED_PMU_DIAGNOSTIC_OBSERVED", "remote profile verdict mismatch")
    stage = LOCAL_RESULT.with_name(f".{LOCAL_RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    base = load_base(BASE_CONTROLLER.read_bytes())
    base.write_new_bytes(stage / "REMOTE_RESULT.json", remote_result.stdout)
    base.write_new_bytes(stage / "REMOTE_VERIFICATION.json", verification.stdout)
    base.write_new_json(
        stage / "LOCAL_INDEX.json",
        {
            "schema": "lay.v10.loaded-pmu-local-index.v4",
            "task_id": TASK_ID,
            "recorded_at": now(),
            "controller_sha256": base.sha256_file(pathlib.Path(__file__)),
            "base_controller_sha256": base.sha256_file(BASE_CONTROLLER),
            "preflight_manifest_sha256": preflight["manifest_sha256"],
            "remote_result_sha256": base.sha256_bytes(remote_result.stdout),
            "remote_verification": json.loads(verification.stdout),
            "local_runtime_before": before,
            "local_runtime_after": after,
            "formal_b_pass": False,
            "v12_admitted": False,
            "runtime_authority_changed": False,
        },
    )
    base.publish(stage, LOCAL_RESULT)
    compact = {
        key: value["derived"]
        for key, value in result_value["windows"].items()
    }
    print(json.dumps({"verdict": result_value["verdict"], "windows": compact, "local_result": str(LOCAL_RESULT)}, indent=2, sort_keys=True))


def self_check() -> None:
    require(list(GROUPS) == ["G2C", "G3"], "counter group order drift")
    require(ORDER == [(group, route) for group in GROUPS for route in ("B5", "B6")], "profile order drift")
    require(PARITY_TEST.endswith("v10_hardware_semantic_parity"), "parity test drift")
    compile(remote_program("remote-run"), "/opt/lay-evidence/loaded-profile-bootstrap.py", "exec")
    forbidden_latency_token = "c1" + "_single"
    require(forbidden_latency_token not in pathlib.Path(__file__).read_text().lower(), "C1 single latency route leaked")
    sample = b'not-json\n{"counter-value":"1000","unit":"","event":"cycles","event-runtime":1,"pcnt-running":100.00}\n{"counter-value":"2000","unit":"","event":"instructions","event-runtime":1,"pcnt-running":100.00}\n'
    parsed = parse_perf_json(sample, ["cycles", "instructions"])
    require(parsed["counters"]["cycles"]["value"] == 1000.0, "perf parser value mismatch")
    hybrid = (
        b'{"counter-value":"<not counted>","event":"cpu_atom/cycles/","event-runtime":0,"pcnt-running":0.0}\n'
        b'{"counter-value":"1000","event":"cpu_core/cycles/","event-runtime":1,"pcnt-running":100.0}\n'
    )
    hybrid_parsed = parse_perf_json(hybrid, ["cycles"], required_pmus=("core",))
    require(hybrid_parsed["counters"]["cycles"]["value"] == 1000.0, "hybrid aggregate mismatch")
    require(hybrid_parsed["counters"]["cycles"]["counted_pmus"] == ["core"], "hybrid PMU mismatch")
    require(len(hybrid_parsed["counters"]["cycles"]["inactive_rows"]) == 1, "inactive PMU row lost")
    partition = (
        b'{"counter-value":"1000","event":"cpu_core/cycles/","event-runtime":60,"pcnt-running":60.0}\n'
        b'{"counter-value":"2000","event":"cpu_atom/cycles/","event-runtime":40,"pcnt-running":40.0}\n'
    )
    partition_parsed = parse_perf_json(partition, ["cycles"], required_pmus=("core", "atom"))
    require(partition_parsed["counters"]["cycles"]["value"] == 1400.0, "hybrid weight mismatch")
    require(
        partition_parsed["counters"]["cycles"]["aggregate_method"] == "hybrid-runtime-weighted",
        "hybrid aggregate method mismatch",
    )
    for bad in (
        b'{"counter-value":"<not counted>","event":"cycles","pcnt-running":100.0}\n',
        b'{"counter-value":"1","event":"cycles","event-runtime":1,"pcnt-running":99.0}\n',
        b'{"counter-value":"1","event":"cpu_core/cycles/","event-runtime":1,"pcnt-running":100.0}\n{"counter-value":"<not counted>","event":"cpu_atom/cycles/","event-runtime":0,"pcnt-running":0.0}\n',
        b'{"counter-value":"1","event":"cpu_core/cycles/","event-runtime":1,"pcnt-running":100.0}\n{"counter-value":"<not supported>","event":"cpu_atom/cycles/","event-runtime":0,"pcnt-running":0.0}\n',
        b'{"counter-value":"1","event":"cpu_core/cycles/","event-runtime":1,"pcnt-running":100.0}\n{"counter-value":"<not counted>","event":"cpu_atom/cycles/","event-runtime":1,"pcnt-running":0.0}\n',
        b'{"counter-value":"1","event":"cpu_core/cycles/","event-runtime":60,"pcnt-running":70.0}\n{"counter-value":"1","event":"cpu_atom/cycles/","event-runtime":40,"pcnt-running":30.0}\n',
    ):
        try:
            required = ("core", "atom") if b"cpu_core" in bad else ()
            parse_perf_json(bad, ["cycles"], required_pmus=required)
        except ProfileError:
            pass
        else:
            raise ProfileError("perf parser accepted invalid evidence")
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        global REMOTE_STATE
        original_state = REMOTE_STATE
        REMOTE_STATE = root / "state"
        marker_root = REMOTE_STATE / "markers"
        marker_root.mkdir(parents=True)
        base = load_base(BASE_CONTROLLER.read_bytes())
        base.write_new_json(marker_root / "b5-g0.available", {"retry_permitted": False})
        consume_marker(base, "b5-g0")
        try:
            consume_marker(base, "b5-g0")
        except ProfileError:
            pass
        else:
            raise ProfileError("profile marker accepted retry")
        REMOTE_STATE = original_state
        sleeper = subprocess.Popen(["/usr/bin/sleep", "30"], start_new_session=True)
        terminate_owned(sleeper)
        require(sleeper.poll() is not None, "owned process cleanup failed")
    print(json.dumps({"verdict": "PASS", "checks": 18, "remote_execution": False}, sort_keys=True))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "run", "remote-run", "remote-verify"))
    arguments = parser.parse_args()
    if arguments.action == "self-check":
        self_check()
    elif arguments.action == "run":
        run_remote_and_publish_local()
    elif arguments.action == "remote-run":
        remote_run()
    else:
        remote_verify()


if __name__ == "__main__":
    main()
