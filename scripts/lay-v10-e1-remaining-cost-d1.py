#!/usr/bin/env python3
"""One-shot exact E1 remaining-cost decomposition on the loaded target host."""

from __future__ import annotations

import argparse
import contextlib
import importlib.util
import json
import math
import os
import pathlib
import re
import signal
import struct
import subprocess
import sys
import tempfile
import time
from typing import Any, Iterable, Sequence


TASK_ID = "slice8b-v10-e1-remaining-cost-d1-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_RESULT = REMOTE_PARENT / "result-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

CONTROLLER = pathlib.Path(__file__).resolve()
BASE_REMOTE = CONTROLLER.with_name("base.py")
REMOTE_BOOTSTRAP = BASE_REMOTE.is_file()
PROJECT_ROOT = (
    pathlib.Path("/home/ubu/projects/lay-l1-exact-peak-search")
    if REMOTE_BOOTSTRAP
    else CONTROLLER.parents[1]
)
BASE_LOCAL = PROJECT_ROOT / "scripts/lay-v10-structural-work.py"
FRAGMENT = CONTROLLER.with_name("fragment.inc") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "scripts/lay_v10_e1_remaining_cost_d1_test_module.rs.inc"
CONTRACT = CONTROLLER.with_name("contract.md") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_CONTRACT_2026-08-25.md"
ROUTE = CONTROLLER.with_name("route.md") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_CODE_ROUTE_V2_2026-08-25.json"
ROUTE_RECEIPT = CONTROLLER.with_name("route-receipt.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_CODE_ROUTE_V2_RECEIPT_2026-08-25.json"
PREFLIGHT = CONTROLLER.with_name("preflight.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_IMPLEMENTATION_V3_2026-08-25.json"
PREFLIGHT_RECEIPT = CONTROLLER.with_name("preflight-receipt.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_IMPLEMENTATION_V3_PREFLIGHT_2026-08-25.json"
M1_DECISION = CONTROLLER.with_name("m1-decision.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_2026-08-25/M1_DECISION.json"
E1_DECISION = CONTROLLER.with_name("e1-decision.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_EXECUTOR_E1_2026-08-25/E1_DECISION.json"
LOCAL_RESULT = PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25"
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"

EXPECTED = {
    "contract": "8b018bd57216748bed4d191b8c369e7b1c0c34e9b488f6dad4a4287b1e81bbb6",
    "route": "4852287e4941a00e26bd2ba3e9de76afd1f78e2adb7587bdcb6d5e68cd021c27",
    "route_receipt": "f771d9ac99df160a657fdd04a5e2ba2ea867b9de3ffec1e07932fac508ff3078",
    "preflight": "e2bd001bb24679f01f140e3710706c19a9a67b22da9573198a536cfcabddf894",
    "preflight_receipt": "063e8d940f784a8246ef42c371166e1652bb32ed2179234826921586300c395f",
    "m1_decision": "f75bdc6995bcdc8553b267ae43e511321bb34fe9d4d9acb14a610104356573a1",
    "e1_decision": "b334c047d29b21c27923fba9b38bbf17bb642cc72c9b112add1c38d8c9b0beab",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
}

TESTS = {
    "P": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_semantic_parity",
    "C-SINGLE": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single",
    "C-TWENTY": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty",
    "PMU": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_twenty_pmu",
}
COMPONENT_ORDER = ("C-SINGLE", "C-FIXED", "C-REVERSED")
PMU_GROUPS = {
    "G0": ("instructions", "cycles", "branches", "branch-misses"),
    "G2": ("L1-dcache-loads", "LLC-loads", "LLC-load-misses"),
    "G3": ("dTLB-loads", "dTLB-load-misses"),
}
PMU_ORDER = tuple(
    f"P-{mapping}-{group}"
    for group in ("G0", "G2", "G3")
    for mapping in ("FIXED", "REVERSED")
)
PHASES = ("oracle", "lanes", "eqmask", "traversal", "merge", "certificate")
COMPONENT_SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
QUERIES = 382
DIAGNOSTIC_ROUNDS = 20


class D1Error(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise D1Error(message)


def load_base() -> Any:
    path = BASE_REMOTE if BASE_REMOTE.is_file() else BASE_LOCAL
    require(path.is_file(), f"missing base controller: {path}")
    spec = importlib.util.spec_from_file_location("lay_v10_d1_base", path)
    require(spec is not None and spec.loader is not None, "cannot load base controller")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    configure_base(module)
    return module


def assemble_source(base: Any, v10: bytes, fragment: bytes) -> bytes:
    require(base.sha256_bytes(v10) == EXPECTED["v10_source"], "V10 source identity mismatch")
    require(v10.endswith(b"}\n"), "V10 terminal brace mismatch")
    require(fragment.startswith(b"\n    const D1_PARITY_TEST"), "D1 fragment prefix mismatch")
    final = v10[:-2] + fragment + b"}\n"
    require(final[:39_047] == v10[:39_047], "V10 production prefix changed")
    require(base.sha256_bytes(final[:39_047]) == EXPECTED["production_prefix"], "production prefix SHA mismatch")
    return final


def initialize_remote_state(base: Any) -> None:
    require(not REMOTE_PARENT.exists(), "D1 remote parent already exists")
    require(not REMOTE_STATE.exists(), "D1 remote state already exists")
    REMOTE_PARENT.mkdir(parents=True, mode=0o700)
    markers = REMOTE_STATE / "markers"
    markers.mkdir(parents=True, mode=0o700)
    for name in ["build", "parity", *COMPONENT_ORDER, *PMU_ORDER]:
        base.write_new_json(markers / f"{name}.available", {
            "task_id": TASK_ID, "route": name, "retry_permitted": False,
        }, 0o400)
    base.write_new_bytes(REMOTE_STATE / "route.lock", b"D1\n", 0o400)
    base.fsync_directory(markers)
    base.fsync_directory(REMOTE_STATE)


def configure_base(base: Any) -> None:
    base.TASK_ID = TASK_ID
    base.REMOTE = REMOTE
    base.REMOTE_HOSTNAME = REMOTE_HOSTNAME
    base.REMOTE_MACHINE_ID_SHA256 = REMOTE_MACHINE_ID_SHA256
    base.REMOTE_PARENT = REMOTE_PARENT
    base.REMOTE_BUILD = REMOTE_BUILD
    base.REMOTE_RESULT = REMOTE_RESULT
    base.REMOTE_STATE = REMOTE_STATE
    base.PROJECT_ROOT = PROJECT_ROOT
    base.FRAGMENT = FRAGMENT
    base.CONTROLLER = CONTROLLER
    base.CONTRACT = CONTRACT
    base.ROUTE = ROUTE
    base.ROUTE_RECEIPT = ROUTE_RECEIPT
    base.PREFLIGHT = PREFLIGHT
    base.PREFLIGHT_RECEIPT = PREFLIGHT_RECEIPT
    base.PMU_RECEIPT = M1_DECISION
    base.LOCAL_RESULT = LOCAL_RESULT
    base.ACTIVE_V11 = ACTIVE_V11
    base.TEST_NAME = TESTS["P"]
    base.EXPECTED.update({
        "contract": EXPECTED["contract"], "route": EXPECTED["route"],
        "route_receipt": EXPECTED["route_receipt"], "preflight": EXPECTED["preflight"],
        "preflight_receipt": EXPECTED["preflight_receipt"],
        "pmu_receipt": EXPECTED["m1_decision"], "v10_source": EXPECTED["v10_source"],
        "production_prefix": EXPECTED["production_prefix"], "active_v11": EXPECTED["active_v11"],
    })
    base.assemble_source = lambda v10, fragment: assemble_source(base, v10, fragment)
    base.initialize_remote_state = lambda: initialize_remote_state(base)


def verify_admission(base: Any) -> dict[str, Any]:
    files = {
        "contract": base.require_file(CONTRACT, sha256=EXPECTED["contract"], mode="0664"),
        "route": base.require_file(ROUTE, sha256=EXPECTED["route"], mode="0664"),
        "route_receipt": base.require_file(ROUTE_RECEIPT, sha256=EXPECTED["route_receipt"], mode="0664"),
        "preflight": base.require_file(PREFLIGHT, sha256=EXPECTED["preflight"], mode="0664"),
        "preflight_receipt": base.require_file(PREFLIGHT_RECEIPT, sha256=EXPECTED["preflight_receipt"], mode="0664"),
        "m1_decision": base.require_file(M1_DECISION, sha256=EXPECTED["m1_decision"], mode="0444"),
        "e1_decision": base.require_file(E1_DECISION, sha256=EXPECTED["e1_decision"], mode="0444"),
        "fragment": base.require_file(FRAGMENT),
        "controller": base.require_file(CONTROLLER),
        "base": base.require_file(BASE_LOCAL),
        "active_v11": base.require_file(ACTIVE_V11, sha256=EXPECTED["active_v11"]),
    }
    route = base.load_json(ROUTE_RECEIPT)
    require(
        route.get("verdict") == "PASS"
        and route.get("ready_for_implementation_preflight") is True
        and route.get("safe_to_edit") is False,
        "D1 route is not implementation-preflight ready",
    )
    preflight = base.load_json(PREFLIGHT_RECEIPT)
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "D1 preflight not ready")
    require(preflight.get("safe_to_implement") is True and not preflight.get("blockers"), "D1 preflight unsafe")
    e1 = base.load_json(E1_DECISION)
    require(e1.get("verdict") == "E1_REJECT", "E1 rejection does not admit D1 diagnosis")
    require(
        e1.get("parity", {}).get("passed") is True
        and e1.get("parity", {}).get("receipt", {}).get("verdict") == "PASS",
        "E1 semantic prerequisite missing",
    )
    require(not LOCAL_RESULT.exists(), "local D1 result already exists")
    return files


def upload_bootstrap(base: Any) -> pathlib.Path:
    temporary = pathlib.Path(base.ssh(["mktemp", "-d", "/tmp/lay-v10-d1.XXXXXX"]).stdout.decode().strip())
    require(str(temporary).startswith("/tmp/lay-v10-d1."), "unexpected remote bootstrap path")
    files = {
        CONTROLLER: "controller.py", BASE_LOCAL: "base.py", FRAGMENT: "fragment.inc",
        CONTRACT: "contract.md", ROUTE: "route.md", ROUTE_RECEIPT: "route-receipt.json",
        PREFLIGHT: "preflight.json", PREFLIGHT_RECEIPT: "preflight-receipt.json",
        M1_DECISION: "m1-decision.json", E1_DECISION: "e1-decision.json",
    }
    for source, name in files.items():
        base.scp(source, f"{temporary}/{name}")
    base.scp(M1_DECISION, f"{temporary}/combined-pmu.json")
    return temporary


def local_self_check(base: Any) -> None:
    files = verify_admission(base)
    v10 = base.P0 / "artifacts/v13_typed_peak.v10.rs"
    source = assemble_source(base, v10.read_bytes(), FRAGMENT.read_bytes())
    rust = FRAGMENT.read_text(encoding="utf-8")
    required = (
        "d1_eq_masks", "d1_equality_window", "d1_u1_advance", "D1PackedNode",
        "d1_search::<false>", "d1_component_search", "d1_thread_cpu_ns",
        "D1_DIAGNOSTIC_ROUNDS: usize = 20", "CLOCK_THREAD_CPUTIME_ID",
        "d1_enumerate_lane_prepared::<false>", "d1_run_twenty_pmu",
        "Barrier::new(D1_WORKERS + 1)", "subject-ready", "controller-disabled",
    )
    for token in required:
        require(token in rust, f"missing D1 source token: {token}")
    forbidden = ("systemctl", "pkill", "killall", "scaling_governor", "LAY_V10_C1_", "V12Executor")
    for token in forbidden:
        require(token not in rust, f"forbidden D1 fragment token: {token}")
    compile(CONTROLLER.read_text(encoding="utf-8"), str(CONTROLLER), "exec")
    with tempfile.NamedTemporaryFile(suffix=".rs") as temporary:
        temporary.write(source)
        temporary.flush()
        formatted = subprocess.run(
            ["rustfmt", "--edition", "2024", "--emit", "stdout", temporary.name],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )
    require(formatted.returncode == 0, "D1 assembled source failed rustfmt parse:\n" + formatted.stderr.decode(errors="replace")[-4000:])
    print(json.dumps({
        "verdict": "PASS", "files": files,
        "fragment_sha256": base.sha256_file(FRAGMENT),
        "controller_sha256": base.sha256_file(CONTROLLER),
        "assembled_source_sha256": base.sha256_bytes(source),
        "production_prefix_sha256": base.sha256_bytes(source[:39_047]),
        "remote_actions": 0, "cargo_invocations": 0, "subject_executions": 0,
    }, sort_keys=True))


def artifacts(base: Any) -> tuple[pathlib.Path, pathlib.Path, pathlib.Path, pathlib.Path]:
    root = base.REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    return (
        base.REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin",
        root / "LAY-L2-RU-FULL-v13.dafsa",
        root / "slice8b-v7-fixed-13x100.json",
        base.REMOTE_B0B / "query-schedule.json",
    )


def subject_environment(base: Any, output: pathlib.Path, run_id: str, cpus: Sequence[int]) -> dict[str, str]:
    package, sidecar, v7, schedule = artifacts(base)
    environment = base.controlled_environment()
    environment.update({
        "LAY_V10_D1_PACKAGE": str(package), "LAY_V10_D1_SIDECAR": str(sidecar),
        "LAY_V10_D1_V7": str(v7), "LAY_V10_D1_SCHEDULE": str(schedule),
        "LAY_V10_D1_OUTPUT": str(output), "LAY_V10_D1_RUN_ID": run_id,
        "LAY_V10_D1_CPUS": ",".join(map(str, cpus)),
    })
    return environment


def subject_command(base: Any, test: str) -> list[str]:
    return [
        str(base.REMOTE_LOADER), str(REMOTE_BUILD / "diagnostic-test-elf"),
        "--exact", test, "--ignored", "--nocapture", "--test-threads=1",
    ]


def child_as_e(environment: dict[str, str], command: Sequence[str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, *command]


def terminate_owned(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGTERM)
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=3)
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def wait_for_file(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        require(process.poll() is None, f"subject exited before {path.name}")
        time.sleep(0.002)
    raise D1Error(f"timeout waiting for {path}")


def open_fifo(path: pathlib.Path, flags: int, deadline: float) -> int:
    while time.monotonic() < deadline:
        try:
            return os.open(path, flags | os.O_NONBLOCK)
        except OSError as error:
            if error.errno not in (6, 11):
                raise
            time.sleep(0.005)
    raise D1Error(f"FIFO open timeout: {path}")


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
    raise D1Error("perf control acknowledgement timeout")


def event_pmu(observed: str, expected: str) -> str | None:
    normalized = observed.strip().lower().replace(":u", "")
    expected = expected.lower()
    if normalized == expected:
        return None
    match = re.fullmatch(r"cpu_(core|atom)/([^/]+)/", normalized)
    if match is not None and match.group(2) == expected:
        return match.group(1)
    return "unknown"


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


def parse_perf(raw: bytes, events: Sequence[str]) -> dict[str, Any]:
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
    for expected in events:
        matched = [row for row in rows if event_pmu(row["event"], expected) != "unknown"]
        require(matched, f"perf event missing: {expected}")
        counted = []
        coverage = set()
        for row in matched:
            counter = numeric_counter(row.get("counter-value"))
            running = numeric_counter(row.get("pcnt-running"))
            runtime = numeric_counter(row.get("event-runtime"))
            pmu = event_pmu(row["event"], expected)
            require(counter is not None, f"unsupported or uncounted perf event: {row}")
            require(pmu in ("core", "atom"), f"D1 PMU row lacks hybrid owner: {row['event']}")
            require(running is not None and abs(running - 100.0) <= 0.01, f"scaled event: {expected}")
            require(runtime is not None and runtime > 0, f"missing runtime: {expected}")
            counted.append((counter, row))
            coverage.add(pmu)
        require(coverage == {"core", "atom"}, f"incomplete hybrid coverage for {expected}: {coverage}")
        require(len(counted) == 2, f"expected core and atom counters for {expected}")
        counters[expected] = {
            "value": sum(value for value, _ in counted),
            "rows": [row for _, row in counted],
            "hybrid_coverage": sorted(coverage),
        }
    return {"counters": counters, "diagnostics": diagnostics, "rows": rows}


def read_text(path: pathlib.Path) -> str | None:
    with contextlib.suppress(OSError):
        return path.read_text(encoding="utf-8", errors="replace").strip()
    return None


def pressure(path: str) -> dict[str, dict[str, float]]:
    result: dict[str, dict[str, float]] = {}
    for line in (read_text(pathlib.Path(path)) or "").splitlines():
        fields = line.split()
        result[fields[0]] = {key: float(value) for key, value in (field.split("=", 1) for field in fields[1:])}
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


def loaded_snapshot(base: Any) -> dict[str, Any]:
    thermal = temperatures()
    topology = base.run(["lscpu", "-J"], check=False).stdout.decode(errors="replace")
    processes = base.run(["ps", "-eo", "pid=,comm=,pcpu=,psr=,args=", "--sort=-pcpu"], check=False).stdout.decode(errors="replace").splitlines()[:40]
    return {
        "monotonic_ns": time.perf_counter_ns(),
        "loadavg": read_text(pathlib.Path("/proc/loadavg")),
        "cpu_pressure": pressure("/proc/pressure/cpu"),
        "io_pressure": pressure("/proc/pressure/io"),
        "memory_pressure": pressure("/proc/pressure/memory"),
        "temperatures": thermal,
        "maximum_temperature_c": max((value["millidegrees_c"] for value in thermal), default=0) / 1000.0,
        "throttle_counters": throttle_counters(),
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
        "topology_lscpu_json": topology,
        "top_processes": processes,
    }


def thermal_drift(before: dict[str, Any], after: dict[str, Any]) -> dict[str, list[int]]:
    result = {}
    left = before["throttle_counters"]
    right = after["throttle_counters"]
    for key in sorted(set(left) | set(right)):
        if left.get(key) != right.get(key):
            result[key] = [left.get(key, 0), right.get(key, 0)]
    return result


def validate_parity(value: dict[str, Any]) -> bool:
    zeros = (
        "terminal_mismatches", "peak_mismatches", "completeness_mismatches",
        "work_mismatches", "rank_prefix_mismatches", "terminal_rank_mismatches",
        "trace_authority_mismatches", "reverse_terminal_mismatches",
        "reverse_peak_mismatches", "reverse_completeness_mismatches",
        "reverse_work_mismatches", "reverse_rank_prefix_mismatches",
        "reverse_terminal_rank_mismatches", "full_row_terminal_mismatches",
        "full_row_peak_mismatches", "full_row_completeness_mismatches",
        "full_row_work_mismatches", "false_certificates",
    )
    checks = [value.get("records") == QUERIES, value.get("schedule_records") == QUERIES]
    checks.extend(value.get(name) == 0 for name in zeros)
    checks.extend([
        value.get("target_form_retained") == QUERIES,
        value.get("target_lemma_retained") == QUERIES,
        value.get("maximum_product_states") == 35_590,
        value.get("e0_maximum_scratch_bytes") == 6_656,
        isinstance(value.get("d1_maximum_scratch_bytes"), int) and value["d1_maximum_scratch_bytes"] <= 6_656,
        value.get("e0_work") == value.get("d1_work"),
        value.get("e0_work", {}).get("expanded_states") == 8_059_788,
        value.get("e0_work", {}).get("examined_edges") == 25_145_756,
        value.get("stress", {}).get("cases") == 714_026,
        value.get("stress", {}).get("transition_mismatches") == 0,
        value.get("stress", {}).get("packed_state_mismatches") == 0,
        value.get("fixtures", {}).get("pass") is True,
    ])
    return value.get("verdict") == "PASS" and all(checks)


def run_parity(base: Any, stage: pathlib.Path) -> dict[str, Any]:
    root = stage / "parity"
    subject = root / "subject"
    subject.mkdir(parents=True, mode=0o700)
    marker = base.consume_marker("parity")
    environment = subject_environment(base, subject, "P", [0])
    command = child_as_e(environment, subject_command(base, TESTS["P"]))
    process = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=3600, check=False)
    base.write_new_bytes(root / "stdout.log", process.stdout)
    base.write_new_bytes(root / "stderr.log", process.stderr)
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    require(receipt_path.is_file(), "D1 parity receipt missing")
    value = base.load_json(receipt_path)
    passed = process.returncode == 0 and validate_parity(value)
    wrapper = {
        "schema": "lay.v10.exact-fused-executor-e1-parity-wrapper.v1",
        "verdict": "PASS" if passed else "FAIL",
        "exit_code": process.returncode, "marker": str(marker),
        "subject_sha256": base.sha256_file(receipt_path),
        "perf_invoked": False, "runtime_authority_changed": False,
    }
    base.write_new_json(root / "PARITY_WRAPPER.json", wrapper)
    return {"passed": passed, "receipt": value, "wrapper": wrapper}


def run_physical(base: Any, stage: pathlib.Path, mode: str) -> dict[str, Any]:
    root = stage / mode.lower()
    subject = root / "subject"
    control = root / "control"
    subject.mkdir(parents=True, mode=0o700)
    control.mkdir(mode=0o700)
    marker = base.consume_marker(mode.lower())
    control_fifo = root / "perf-control.fifo"
    ack_fifo = root / "perf-ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    environment = subject_environment(base, subject, mode, [0])
    environment["LAY_V10_D1_CONTROL_DIR"] = str(control)
    child = child_as_e(environment, ["/usr/bin/taskset", "-c", "0", *subject_command(base, TESTS[mode])])
    command = [
        "/usr/bin/sudo", "-n", "/usr/bin/perf", "stat", "--json-output",
        "--no-big-num", "--delay=-1", f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event", ",".join(EVENTS), "--", *child,
    ]
    before = loaded_snapshot(base)
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        wait_for_file(process, control / "subject-ready", 3600.0)
        deadline = time.monotonic() + 10.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        base.write_new_bytes(control / "controller-enabled", b"enabled\n")
        started_ns = time.perf_counter_ns()
        wait_for_file(process, control / "subject-done", 1200.0)
        ended_ns = time.perf_counter_ns()
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        base.write_new_bytes(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=180)
        base.write_new_bytes(root / "stdout.log", stdout)
        base.write_new_bytes(root / "perf.raw", stderr)
        require(process.returncode == 0, f"{mode} physical exited {process.returncode}: {stderr[-4000:]!r}")
        receipt_path = subject / "SUBJECT_RECEIPT.json"
        subject_value = base.load_json(receipt_path)
        require(subject_value.get("verdict") == "PASS" and subject_value.get("executor") == mode, f"{mode} physical subject failed")
        parsed = parse_perf(stderr)
        values = {event: parsed["counters"][event]["value"] for event in EVENTS}
        after = loaded_snapshot(base)
        drift = thermal_drift(before, after)
        wrapper = {
            "schema": "lay.v10.exact-fused-executor-e1-physical-wrapper.v1",
            "verdict": "BLOCKED_THERMAL" if drift else "PASS", "mode": mode,
            "marker": str(marker), "command": command,
            "enable_ack": enable_ack, "disable_ack": disable_ack,
            "counters": parsed["counters"],
            "derived": {
                "instructions_per_query": values["instructions"] / QUERIES,
                "cycles_per_query": values["cycles"] / QUERIES,
                "branches_per_query": values["branches"] / QUERIES,
                "branch_miss_rate": values["branch-misses"] / values["branches"],
                "ipc": values["instructions"] / values["cycles"],
                "controller_wall_ns_diagnostic": ended_ns - started_ns,
            },
            "environment_before": before, "environment_after": after,
            "thermal_throttle_drift": drift,
            "subject_sha256": base.sha256_file(receipt_path),
            "raw_perf_sha256": base.sha256_file(root / "perf.raw"),
            "environment_intentionally_loaded": True, "pmu_event_opened": True,
            "runtime_authority_changed": False,
        }
        base.write_new_json(root / "PHYSICAL_WRAPPER.json", wrapper)
        return wrapper
    except BaseException:
        terminate_owned(process)
        raise
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)


def parse_samples(path: pathlib.Path) -> list[dict[str, int]]:
    value = path.read_bytes()
    require(len(value) % SAMPLE.size == 0, f"sample width mismatch: {path}")
    records = []
    for offset in range(0, len(value), SAMPLE.size):
        query, round_id, worker, flags, search_us, total_us = SAMPLE.unpack_from(value, offset)
        records.append({
            "query_ordinal": query, "round": round_id, "worker_id": worker,
            "flags": flags, "search_elapsed_us": search_us, "total_elapsed_us": total_us,
        })
    return records


def nearest_rank(values: Iterable[int], percentile: int) -> int:
    ordered = sorted(values)
    require(bool(ordered), "percentile denominator is empty")
    return ordered[math.ceil(len(ordered) * percentile / 100) - 1]


def percentiles(samples: list[dict[str, int]]) -> dict[str, int]:
    require(bool(samples), "sample set empty")
    return {
        "records": len(samples),
        "search_p99_us": nearest_rank((sample["search_elapsed_us"] for sample in samples), 99),
        "total_p99_us": nearest_rank((sample["total_elapsed_us"] for sample in samples), 99),
        "search_max_us": max(sample["search_elapsed_us"] for sample in samples),
        "total_max_us": max(sample["total_elapsed_us"] for sample in samples),
    }


def run_latency_route(base: Any, stage: pathlib.Path, route: str) -> dict[str, Any]:
    kind = route[0]
    cpus = [0] if kind == "S" else list(range(20))
    root = stage / f"run-{route}"
    subject = root / "subject"
    subject.mkdir(parents=True, mode=0o700)
    marker = base.consume_marker(route)
    environment = subject_environment(base, subject, route, cpus)
    command = child_as_e(environment, subject_command(base, TESTS[kind]))
    before = loaded_snapshot(base)
    started_ns = time.perf_counter_ns()
    process = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=2400, check=False)
    ended_ns = time.perf_counter_ns()
    after = loaded_snapshot(base)
    base.write_new_bytes(root / "stdout.log", process.stdout)
    base.write_new_bytes(root / "stderr.log", process.stderr)
    require(process.returncode == 0, f"D1 {route} exited {process.returncode}: {process.stderr[-4000:]!r}")
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    samples_path = subject / "samples.bin"
    require(receipt_path.is_file() and samples_path.is_file(), f"D1 {route} output missing")
    receipt = base.load_json(receipt_path)
    samples = parse_samples(samples_path)
    expected = 38_200 if kind == "S" else 95_500
    require(len(samples) == expected, f"D1 {route} sample denominator mismatch")
    require(receipt.get("samples", {}).get("records") == expected, f"D1 {route} receipt denominator mismatch")
    require(receipt.get("samples", {}).get("errors") == 0, f"D1 {route} errors")
    require(receipt.get("samples", {}).get("unresolved") == 0, f"D1 {route} unresolved")
    drift = thermal_drift(before, after)
    wrapper = {
        "schema": "lay.v10.exact-fused-executor-e1-loaded-latency-route.v1",
        "verdict": "BLOCKED_THERMAL" if drift else "PASS", "route": route,
        "marker": str(marker), "command": command,
        "environment_before": before, "environment_after": after,
        "thermal_throttle_drift": drift,
        "process_wall_ns_diagnostic": ended_ns - started_ns,
        "percentiles": percentiles(samples),
        "subject_sha256": base.sha256_file(receipt_path),
        "samples_sha256": base.sha256_file(samples_path),
        "perf_invoked": False, "pmu_event_opened": False,
        "environment_intentionally_loaded": True, "foreign_process_control": False,
        "runtime_authority_changed": False,
    }
    base.write_new_json(root / "WRAPPER_RECEIPT.json", wrapper)
    return {"wrapper": wrapper, "samples": samples}


def aggregate_latency(routes: dict[str, dict[str, Any]]) -> dict[str, Any]:
    pooled_s: list[dict[str, int]] = []
    pooled_t: list[dict[str, int]] = []
    route_values = {}
    query_samples: dict[int, list[dict[str, int]]] = {index: [] for index in range(QUERIES)}
    fairness = []
    for route in ORDER:
        samples = routes[route]["samples"]
        route_values[route] = percentiles(samples)
        (pooled_s if route.startswith("S") else pooled_t).extend(samples)
        for sample in samples:
            query_samples[sample["query_ordinal"]].append(sample)
        if route.startswith("T"):
            workers = {}
            for worker in range(20):
                worker_samples = [sample for sample in samples if sample["worker_id"] == worker]
                workers[str(worker)] = percentiles(worker_samples)
                fairness.append((route, worker, workers[str(worker)]["total_p99_us"]))
            route_values[route]["workers"] = workers
    require(len(pooled_s) == 191_000 and len(pooled_t) == 477_500, "D1 pooled denominator mismatch")
    pooled = {"S": percentiles(pooled_s), "T": percentiles(pooled_t)}
    per_query = {str(query): percentiles(samples) for query, samples in query_samples.items()}
    worst_query = max(per_query.items(), key=lambda item: item[1]["total_p99_us"])
    worst_worker = max(fairness, key=lambda item: item[2])
    all_samples = [*pooled_s, *pooled_t]
    errors = sum(bool(sample["flags"] & 1) for sample in all_samples)
    unresolved = sum(bool(sample["flags"] & 2) for sample in all_samples)
    hard = {
        "s_pooled_search": pooled["S"]["search_p99_us"] <= 3_000,
        "s_pooled_total": pooled["S"]["total_p99_us"] <= 5_000,
        "s_runs_search": all(route_values[f"S{index}"]["search_p99_us"] <= 3_000 for index in range(1, 6)),
        "s_runs_total": all(route_values[f"S{index}"]["total_p99_us"] <= 5_000 for index in range(1, 6)),
        "t_pooled_total": pooled["T"]["total_p99_us"] <= 5_000,
        "t_runs_total": all(route_values[f"T{index}"]["total_p99_us"] <= 5_000 for index in range(1, 6)),
        "fairness": worst_worker[2] <= 5_000,
        "errors": errors == 0, "unresolved": unresolved == 0,
    }
    return {
        "hard_conjuncts": hard, "all_latency_conjuncts_pass": all(hard.values()),
        "thresholds_us": {"single_search_p99": 3_000, "single_total_p99": 5_000, "twenty_total_p99": 5_000, "fairness_total_p99": 5_000},
        "pooled": pooled, "routes": route_values,
        "fairness": {
            "maximum_run": worst_worker[0], "maximum_worker": worst_worker[1],
            "maximum_total_p99_us": worst_worker[2],
            "minimum_total_p99_us": min(item[2] for item in fairness),
            "spread_us": worst_worker[2] - min(item[2] for item in fairness),
        },
        "query_diagnostics": {
            "per_query": per_query, "worst_query_ordinal": int(worst_query[0]),
            "worst_query_total_p99_us": worst_query[1]["total_p99_us"],
        },
        "sample_counts": {"S": len(pooled_s), "T": len(pooled_t)},
        "errors": errors, "unresolved": unresolved,
    }


def parse_component_samples(path: pathlib.Path) -> list[dict[str, Any]]:
    raw = path.read_bytes()
    require(len(raw) % COMPONENT_SAMPLE.size == 0, f"D1 component sample width mismatch: {path}")
    records = []
    for offset in range(0, len(raw), COMPONENT_SAMPLE.size):
        values = COMPONENT_SAMPLE.unpack_from(raw, offset)
        phases = {}
        cursor = 6
        for phase in PHASES:
            phases[phase] = {"wall_ns": values[cursor], "thread_cpu_ns": values[cursor + 1]}
            cursor += 2
        records.append({
            "query_ordinal": values[0], "round": values[1], "worker_id": values[2],
            "flags": values[3], "outer_wall_ns": values[4], "outer_thread_cpu_ns": values[5],
            "phases": phases,
        })
    return records


def metric_summary(values: Iterable[int]) -> dict[str, Any]:
    observed = list(values)
    require(bool(observed), "D1 metric denominator is empty")
    total = sum(observed)
    return {
        "records": len(observed), "sum": total, "mean": total / len(observed),
        "p50": nearest_rank(observed, 50), "p95": nearest_rank(observed, 95),
        "p99": nearest_rank(observed, 99), "max": max(observed),
    }


def phase_metric(samples: Sequence[dict[str, Any]], phase: str, metric: str) -> list[int]:
    return [sample["phases"][phase][metric] for sample in samples]


def component_statistics(
    samples: list[dict[str, Any]], structures: list[dict[str, Any]], route: str,
) -> dict[str, Any]:
    require(len(samples) == QUERIES * DIAGNOSTIC_ROUNDS, f"D1 {route} denominator mismatch")
    require(len(structures) == QUERIES, f"D1 {route} structure denominator mismatch")
    require(sum(bool(sample["flags"] & 1) for sample in samples) == 0, f"D1 {route} errors")
    require(sum(bool(sample["flags"] & 2) for sample in samples) == 0, f"D1 {route} unresolved")
    outer_wall = metric_summary(sample["outer_wall_ns"] for sample in samples)
    outer_cpu = metric_summary(sample["outer_thread_cpu_ns"] for sample in samples)
    outer_residual = metric_summary(
        max(0, sample["outer_wall_ns"] - sample["outer_thread_cpu_ns"]) for sample in samples
    )
    phases = {}
    for phase in PHASES:
        wall = metric_summary(phase_metric(samples, phase, "wall_ns"))
        cpu = metric_summary(phase_metric(samples, phase, "thread_cpu_ns"))
        residual = metric_summary(
            max(0, sample["phases"][phase]["wall_ns"] - sample["phases"][phase]["thread_cpu_ns"])
            for sample in samples
        )
        phases[phase] = {
            "wall_ns": wall, "thread_cpu_ns": cpu, "wall_minus_cpu_ns": residual,
            "wall_share_of_outer_sum": wall["sum"] / outer_wall["sum"],
            "thread_cpu_share_of_outer_sum": cpu["sum"] / outer_cpu["sum"],
        }
    accounted_wall = sum(value["wall_ns"]["sum"] for value in phases.values())
    accounted_cpu = sum(value["thread_cpu_ns"]["sum"] for value in phases.values())
    by_query = {}
    for query in range(QUERIES):
        subset = [sample for sample in samples if sample["query_ordinal"] == query]
        by_query[str(query)] = {
            "outer_wall_ns": metric_summary(sample["outer_wall_ns"] for sample in subset),
            "outer_thread_cpu_ns": metric_summary(sample["outer_thread_cpu_ns"] for sample in subset),
            "phases": {
                phase: {
                    "wall_p99_ns": nearest_rank(phase_metric(subset, phase, "wall_ns"), 99),
                    "thread_cpu_p99_ns": nearest_rank(phase_metric(subset, phase, "thread_cpu_ns"), 99),
                }
                for phase in PHASES
            },
            "structure": structures[query],
        }
    workers = {}
    worker_ids = sorted({sample["worker_id"] for sample in samples})
    for worker in worker_ids:
        subset = [sample for sample in samples if sample["worker_id"] == worker]
        workers[str(worker)] = {
            "outer_wall_ns": metric_summary(sample["outer_wall_ns"] for sample in subset),
            "outer_thread_cpu_ns": metric_summary(sample["outer_thread_cpu_ns"] for sample in subset),
            "phases": {
                phase: {
                    "wall_p99_ns": nearest_rank(phase_metric(subset, phase, "wall_ns"), 99),
                    "thread_cpu_p99_ns": nearest_rank(phase_metric(subset, phase, "thread_cpu_ns"), 99),
                }
                for phase in PHASES
            },
        }
    structural_totals = {
        field: sum(int(value.get(field, 0)) for value in structures)
        for field in (
            "retrieval_lanes", "eqmask_builds", "expanded_states", "examined_edges",
            "surviving_edges", "pruned_edges", "stack_pushes", "stack_pops",
            "terminal_hits_before_merge", "terminal_refs_after_merge", "certificate_calls",
            "materialized_peaks",
        )
    }
    return {
        "route": route, "samples": len(samples),
        "outer": {
            "wall_ns": outer_wall, "thread_cpu_ns": outer_cpu,
            "wall_minus_cpu_ns": outer_residual,
            "unattributed_wall_sum_ns": max(0, outer_wall["sum"] - accounted_wall),
            "unattributed_thread_cpu_sum_ns": max(0, outer_cpu["sum"] - accounted_cpu),
        },
        "phases": phases,
        "phase_rank_by_thread_cpu_share": sorted(
            ({"phase": phase, "share": values["thread_cpu_share_of_outer_sum"]} for phase, values in phases.items()),
            key=lambda value: value["share"], reverse=True,
        ),
        "per_query": by_query, "per_worker": workers,
        "structural_totals_per_round": structural_totals,
        "errors": 0, "unresolved": 0,
    }


def run_component(base: Any, stage: pathlib.Path, route: str) -> dict[str, Any]:
    cpus = [0] if route == "C-SINGLE" else (
        list(range(20)) if route == "C-FIXED" else list(reversed(range(20)))
    )
    test = TESTS["C-SINGLE"] if route == "C-SINGLE" else TESTS["C-TWENTY"]
    root = stage / route
    subject = root / "subject"
    subject.mkdir(parents=True, mode=0o700)
    marker = base.consume_marker(route)
    environment = subject_environment(base, subject, route, cpus)
    command = child_as_e(environment, subject_command(base, test))
    before = loaded_snapshot(base)
    started_ns = time.perf_counter_ns()
    process = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=3600, check=False)
    ended_ns = time.perf_counter_ns()
    after = loaded_snapshot(base)
    base.write_new_bytes(root / "stdout.log", process.stdout)
    base.write_new_bytes(root / "stderr.log", process.stderr)
    require(process.returncode == 0, f"D1 {route} exited {process.returncode}: {process.stderr[-6000:]!r}")
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    samples_path = subject / "component-samples.bin"
    structure_path = subject / "structure.json"
    require(receipt_path.is_file() and samples_path.is_file() and structure_path.is_file(), f"D1 {route} outputs missing")
    receipt = base.load_json(receipt_path)
    samples = parse_component_samples(samples_path)
    structures = base.load_json(structure_path).get("queries", [])
    require(receipt.get("samples", {}).get("samples") == QUERIES * DIAGNOSTIC_ROUNDS, f"D1 {route} receipt denominator")
    require(receipt.get("samples", {}).get("errors") == 0, f"D1 {route} receipt errors")
    require(receipt.get("samples", {}).get("unresolved") == 0, f"D1 {route} receipt unresolved")
    statistics = component_statistics(samples, structures, route)
    drift = thermal_drift(before, after)
    wrapper = {
        "schema": "lay.v10.e1-remaining-cost-d1-component-wrapper.v1",
        "verdict": "BLOCKED_THERMAL" if drift else "PASS", "route": route,
        "marker": str(marker), "command": command,
        "environment_before": before, "environment_after": after,
        "thermal_throttle_drift": drift, "process_wall_ns_diagnostic": ended_ns - started_ns,
        "subject_receipt": receipt, "statistics": statistics,
        "subject_sha256": base.sha256_file(receipt_path),
        "samples_sha256": base.sha256_file(samples_path),
        "structure_sha256": base.sha256_file(structure_path),
        "perf_invoked": False, "pmu_event_opened": False,
        "environment_intentionally_loaded": True, "loaded_host_is_blocker": False,
        "foreign_process_control": False, "runtime_authority_changed": False,
    }
    base.write_new_json(root / "COMPONENT_WRAPPER.json", wrapper)
    return wrapper


def run_d1_pmu(base: Any, stage: pathlib.Path, route: str) -> dict[str, Any]:
    _, mapping, group = route.split("-", 2)
    cpus = list(range(20)) if mapping == "FIXED" else list(reversed(range(20)))
    events = PMU_GROUPS[group]
    root = stage / route
    subject = root / "subject"
    control = root / "control"
    subject.mkdir(parents=True, mode=0o700)
    control.mkdir(mode=0o700)
    marker = base.consume_marker(route)
    control_fifo = root / "perf-control.fifo"
    ack_fifo = root / "perf-ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    environment = subject_environment(base, subject, route, cpus)
    environment["LAY_V10_D1_CONTROL_DIR"] = str(control)
    child = child_as_e(environment, subject_command(base, TESTS["PMU"]))
    command = [
        "/usr/bin/sudo", "-n", "/usr/bin/perf", "stat", "--json-output",
        "--no-big-num", "--delay=-1", f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event", ",".join(events), "--", *child,
    ]
    before = loaded_snapshot(base)
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        wait_for_file(process, control / "subject-ready", 3600.0)
        deadline = time.monotonic() + 10.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        base.write_new_bytes(control / "controller-enabled", b"enabled\n")
        measured_started = time.perf_counter_ns()
        wait_for_file(process, control / "subject-done", 1200.0)
        measured_ended = time.perf_counter_ns()
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        base.write_new_bytes(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=300)
        base.write_new_bytes(root / "stdout.log", stdout)
        base.write_new_bytes(root / "perf.raw", stderr)
        receipt_path = subject / "SUBJECT_RECEIPT.json"
        require(receipt_path.is_file(), f"D1 {route} subject receipt missing")
        receipt = base.load_json(receipt_path)
        require(receipt.get("verdict") == "PASS", f"D1 {route} subject failed")
        after = loaded_snapshot(base)
        drift = thermal_drift(before, after)
        capability_error = None
        parsed = None
        if process.returncode == 0:
            try:
                parsed = parse_perf(stderr, events)
            except D1Error as error:
                capability_error = str(error)
        else:
            capability_error = f"perf exited {process.returncode}"
        counters = {} if parsed is None else parsed["counters"]
        requests = QUERIES * DIAGNOSTIC_ROUNDS
        edges = 25_145_756 * DIAGNOSTIC_ROUNDS
        derived = {
            event: {
                "per_request": value["value"] / requests,
                "per_examined_edge": value["value"] / edges,
            }
            for event, value in counters.items()
        }
        if group == "G0" and not capability_error:
            derived["ipc"] = counters["instructions"]["value"] / counters["cycles"]["value"]
            derived["branch_miss_rate"] = counters["branch-misses"]["value"] / counters["branches"]["value"]
        wrapper = {
            "schema": "lay.v10.e1-remaining-cost-d1-pmu-wrapper.v1",
            "verdict": "BLOCKED_THERMAL" if drift else ("BLOCKED_CAPABILITY" if capability_error else "PASS"),
            "route": route, "mapping": mapping, "event_group": group, "events": list(events),
            "marker": str(marker), "command": command,
            "enable_ack": enable_ack, "disable_ack": disable_ack,
            "counters": counters, "derived": derived, "capability_error": capability_error,
            "environment_before": before, "environment_after": after,
            "thermal_throttle_drift": drift,
            "controller_measured_wall_ns_diagnostic": measured_ended - measured_started,
            "subject_receipt": receipt, "subject_sha256": base.sha256_file(receipt_path),
            "raw_perf_sha256": base.sha256_file(root / "perf.raw"),
            "environment_intentionally_loaded": True, "loaded_host_is_blocker": False,
            "foreign_process_control": False, "pmu_event_opened": True,
            "runtime_authority_changed": False,
        }
        base.write_new_json(root / "PMU_WRAPPER.json", wrapper)
        return wrapper
    except BaseException:
        terminate_owned(process)
        raise
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)


def ratio(right: float, left: float) -> float | None:
    return right / left if left else None


def aggregate_d1(components: dict[str, dict[str, Any]], pmu: dict[str, dict[str, Any]]) -> dict[str, Any]:
    single = components["C-SINGLE"]["statistics"]
    fixed = components["C-FIXED"]["statistics"]
    reversed_value = components["C-REVERSED"]["statistics"]
    query = "381"
    worker = "19"
    mapping_comparison = {
        "query_381": {
            "fixed_outer_wall_p99_ns": fixed["per_query"][query]["outer_wall_ns"]["p99"],
            "reversed_outer_wall_p99_ns": reversed_value["per_query"][query]["outer_wall_ns"]["p99"],
            "reversed_over_fixed": ratio(
                reversed_value["per_query"][query]["outer_wall_ns"]["p99"],
                fixed["per_query"][query]["outer_wall_ns"]["p99"],
            ),
        },
        "worker_19": {
            "fixed_outer_wall_p99_ns": fixed["per_worker"][worker]["outer_wall_ns"]["p99"],
            "reversed_outer_wall_p99_ns": reversed_value["per_worker"][worker]["outer_wall_ns"]["p99"],
            "reversed_over_fixed": ratio(
                reversed_value["per_worker"][worker]["outer_wall_ns"]["p99"],
                fixed["per_worker"][worker]["outer_wall_ns"]["p99"],
            ),
        },
    }
    pmu_comparison = {}
    for group in PMU_GROUPS:
        left = pmu[f"P-FIXED-{group}"]
        right = pmu[f"P-REVERSED-{group}"]
        events = {}
        if left["verdict"] == right["verdict"] == "PASS":
            for event in PMU_GROUPS[group]:
                events[event] = {
                    "fixed_per_request": left["derived"][event]["per_request"],
                    "reversed_per_request": right["derived"][event]["per_request"],
                    "reversed_over_fixed": ratio(
                        right["derived"][event]["per_request"], left["derived"][event]["per_request"]
                    ),
                }
        pmu_comparison[group] = {"events": events, "fixed_verdict": left["verdict"], "reversed_verdict": right["verdict"]}
    return {
        "component_routes": {route: value["statistics"] for route, value in components.items()},
        "single_phase_rank": single["phase_rank_by_thread_cpu_share"],
        "mapping_comparison": mapping_comparison,
        "pmu_routes": pmu,
        "pmu_fixed_reversed_comparison": pmu_comparison,
    }


def publish_result(base: Any, stage: pathlib.Path, decision: dict[str, Any]) -> None:
    base.write_new_json(stage / "D1_DECISION.json", decision)
    base.write_new_json(stage / "RUN_PROVENANCE.json", {
        "schema": "lay.v10.e1-remaining-cost-d1-run-provenance.v1",
        "task_id": TASK_ID, "controller_sha256": base.sha256_file(CONTROLLER),
        "fragment_sha256": base.sha256_file(FRAGMENT),
        "markers_consumed": sorted(path.name for path in (REMOTE_STATE / "markers").glob("*.consumed-before-exec")),
        "adaptive_rerun": False, "third_loaded_e0_v10_run": False,
        "clean_c1_marker_consumed": False, "full_b_executed": False,
        "foreign_process_control": False, "host_tuning": False,
        "v12_admitted": False, "installed_lay_changed": False,
        "runtime_authority_changed": False,
    })
    base.write_sha256sums(stage)
    base.seal_tree(stage)
    os.rename(stage, REMOTE_RESULT)
    base.fsync_directory(REMOTE_PARENT)
    base.seal_tree(REMOTE_STATE)


def terminal_decision(
    base: Any, stage: pathlib.Path, provenance: dict[str, Any], parity: dict[str, Any],
    components: dict[str, Any] | None, pmu: dict[str, Any] | None,
    comparison: dict[str, Any] | None, verdict: str,
) -> dict[str, Any]:
    decision = {
        "schema": "lay.v10.e1-remaining-cost-d1-decision.v1",
        "task_id": TASK_ID, "verdict": verdict,
        "subject": provenance["executable"],
        "production_prefix_bytes": 39_047,
        "production_prefix_sha256": EXPECTED["production_prefix"],
        "parity": parity, "components": components, "pmu": pmu, "comparison": comparison,
        "environment_intentionally_loaded": True,
        "loaded_host_is_blocker": False,
        "claim_boundary": {
            "diagnostic_only": True,
            "production_integration_admitted": False, "deployment_authority": False,
            "formal_b_pass": False, "v12_admitted": False,
            "runtime_authority_changed": False,
        },
    }
    publish_result(base, stage, decision)
    return decision


def remote_run(base: Any, bootstrap: pathlib.Path) -> None:
    base.remote_machine_identity()
    base.remote_build(bootstrap)
    require(REMOTE_BUILD.is_dir() and not REMOTE_RESULT.exists(), "D1 sealed build missing or result exists")
    base.verify_sha256sums(REMOTE_BUILD)
    provenance = base.load_json(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")
    executable = REMOTE_BUILD / "diagnostic-test-elf"
    require(base.sha256_file(executable) == provenance["executable"]["sha256"], "D1 ELF SHA drift")
    require(base.elf_build_id(executable) == provenance["executable"]["build_id"], "D1 ELF Build ID drift")
    package, sidecar, v7, schedule = artifacts(base)
    for path, digest in ((package, EXPECTED["package"]), (sidecar, EXPECTED["sidecar"]), (v7, EXPECTED["v7"]), (schedule, EXPECTED["schedule"])):
        base.require_file(path, sha256=digest, mode="0444")
    stage = REMOTE_PARENT / f"result-v1.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    try:
        parity = run_parity(base, stage)
        if not parity["passed"]:
            decision = terminal_decision(base, stage, provenance, parity, None, None, None, "D1_REJECT_PARITY")
            print(json.dumps({"state": decision["verdict"], "result": str(REMOTE_RESULT)}))
            return
        components = {}
        for route in COMPONENT_ORDER:
            components[route] = run_component(base, stage, route)
            if components[route]["verdict"] == "BLOCKED_THERMAL":
                decision = terminal_decision(base, stage, provenance, parity, components, None, None, "BLOCKED_THERMAL")
                print(json.dumps({"state": decision["verdict"], "result": str(REMOTE_RESULT)}))
                return
        pmu = {}
        for route in PMU_ORDER:
            pmu[route] = run_d1_pmu(base, stage, route)
            if pmu[route]["verdict"] == "BLOCKED_THERMAL":
                decision = terminal_decision(base, stage, provenance, parity, components, pmu, None, "BLOCKED_THERMAL")
                print(json.dumps({"state": decision["verdict"], "result": str(REMOTE_RESULT)}))
                return
        comparison = aggregate_d1(components, pmu)
        capability_gap = any(value["verdict"] == "BLOCKED_CAPABILITY" for value in pmu.values())
        verdict = "D1_OBSERVED_WITH_CAPABILITY_GAP" if capability_gap else "D1_OBSERVED"
        decision = terminal_decision(base, stage, provenance, parity, components, pmu, comparison, verdict)
        print(json.dumps({"state": decision["verdict"], "result": str(REMOTE_RESULT)}))
    except Exception as error:
        with contextlib.suppress(Exception):
            base.write_new_json(stage / "FAILURE.json", {
                "schema": "lay.v10.e1-remaining-cost-d1-failure.v1",
                "error": str(error), "retry_permitted": False,
                "markers": sorted(path.name for path in (REMOTE_STATE / "markers").glob("*")),
                "runtime_authority_changed": False,
            })
            base.write_sha256sums(stage)
            base.seal_tree(stage)
            os.rename(stage, REMOTE_PARENT / "run-failure-v1")
        raise


def local_run(base: Any) -> None:
    verify_admission(base)
    require(not LOCAL_RESULT.exists(), "local D1 result already exists")
    probe = base.ssh(["python3", "-c", (
        "import hashlib,json,os,pathlib;"
        "m=pathlib.Path('/etc/machine-id');"
        f"p=pathlib.Path('{REMOTE_PARENT}');s=pathlib.Path('{REMOTE_STATE}');"
        "print(json.dumps({'host':os.uname().nodename,'machine':hashlib.sha256(m.read_bytes()).hexdigest(),"
        "'parent':p.exists(),'state':s.exists()}))"
    )])
    status = json.loads(probe.stdout)
    require(status == {"host": REMOTE_HOSTNAME, "machine": REMOTE_MACHINE_ID_SHA256, "parent": False, "state": False}, f"D1 remote pre-run mismatch: {status}")
    before = base.local_runtime_snapshot()
    bootstrap = upload_bootstrap(base)
    try:
        result = base.ssh(["python3", f"{bootstrap}/controller.py", "remote-run", str(bootstrap)], check=False)
        require(result.returncode == 0, result.stderr.decode(errors="replace")[-8000:])
        remote_output = result.stdout.decode(errors="replace").strip().splitlines()
    finally:
        base.ssh(["rm", "-rf", "--", str(bootstrap)], check=False)
    after = base.local_runtime_snapshot()
    require(before == after, f"installed runtime changed: {before} != {after}")
    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    try:
        base.run(["scp", "-q", "-pr", f"{REMOTE}:{REMOTE_RESULT}", str(stage)])
        entries = base.verify_sha256sums(stage)
        decision = base.load_json(stage / "D1_DECISION.json")
        require(decision.get("verdict") in (
            "D1_OBSERVED", "D1_OBSERVED_WITH_CAPABILITY_GAP", "D1_REJECT_PARITY",
            "BLOCKED_THERMAL", "BLOCKED_PROVENANCE",
        ), "invalid D1 decision")
        require(decision.get("claim_boundary", {}).get("v12_admitted") is False, "D1 admitted V12")
        require(not any(path.stat().st_mode & 0o222 for path in stage.rglob("*")), "remote D1 result contains writable objects")
        base.seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        base.fsync_directory(LOCAL_RESULT.parent)
        print(json.dumps({
            "state": decision["verdict"], "remote_output": remote_output[-2:],
            "local_result": str(LOCAL_RESULT), "manifest_entries": entries,
            "runtime_stable": True,
        }, sort_keys=True))
    except Exception:
        base.remove_tree(stage)
        raise


def local_status(base: Any) -> None:
    result = base.ssh(["python3", "-c", (
        "import json,pathlib;"
        f"p=pathlib.Path('{REMOTE_PARENT}');s=pathlib.Path('{REMOTE_STATE}');m=s/'markers';"
        "print(json.dumps({'parent':p.exists(),'build':(p/'build-v1').exists(),"
        "'result':(p/'result-v1').exists(),'failure':(p/'run-failure-v1').exists(),"
        "'state':s.exists(),'markers':sorted(x.name for x in m.glob('*')) if m.is_dir() else []}))"
    )])
    print(result.stdout.decode().strip())


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "run", "status", "remote-run"))
    value.add_argument("argument", nargs="?")
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        base = load_base()
        if arguments.action == "self-check":
            local_self_check(base)
        elif arguments.action == "run":
            local_run(base)
        elif arguments.action == "status":
            local_status(base)
        elif arguments.action == "remote-run":
            require(os.uname().nodename == REMOTE_HOSTNAME, "remote-run on wrong host")
            require(arguments.argument is not None, "remote-run requires bootstrap path")
            remote_run(base, pathlib.Path(arguments.argument))
        return 0
    except Exception as error:
        print(f"D1 ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
