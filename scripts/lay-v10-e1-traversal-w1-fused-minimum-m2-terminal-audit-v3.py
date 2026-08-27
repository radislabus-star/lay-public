#!/usr/bin/env python3
"""V3 independent terminal scientific audit for W1 fused-minimum M2."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import pathlib
import re
import shlex
import shutil
import stat
import struct
import subprocess
import sys
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2-v1-20260826"
TRANSACTION_ID = "c760eea52b6416b3529f9d684c315147b5a1140522114642c417d7db4065102c"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
AUDITOR = pathlib.Path(__file__).resolve()
LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-v3.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-remote.py"
IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "IMPLEMENTATION_SELF_CHECK_V3_2026-08-26.json"
)
BOOTSTRAP_AUDIT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BOOTSTRAP_AUDIT_V2_2026-08-26/M2_BOOTSTRAP_AUDIT_RECEIPT.json"
)
BUILD_AUDIT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BUILD_AUDIT_V2_2026-08-26/M2_BUILD_AUDIT_RECEIPT.json"
)
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "TERMINAL_AUDIT_V2_2026-08-26"
)
ROUTE_ORDER = (
    "B0-ITERATOR",
    "G0-M1-GUARDED",
    "I0-INTERLEAVED",
    "I1-INTERLEAVED",
    "G1-M1-GUARDED",
    "B1-ITERATOR",
)
ROUTE_DIRS = {
    route: f"route-{index + 1:02d}-{route.lower().replace('-', '_')}"
    for index, route in enumerate(ROUTE_ORDER)
}
PAIR_ROUTES = {
    "B": ("B0-ITERATOR", "B1-ITERATOR"),
    "G": ("G0-M1-GUARDED", "G1-M1-GUARDED"),
    "I": ("I0-INTERLEAVED", "I1-INTERLEAVED"),
}
QUERIES = 382
ROUNDS = 20
RECORDS = QUERIES * ROUNDS
EDGES_PER_ROUND = 25_145_756
MEASURED_EDGES = EDGES_PER_ROUND * ROUNDS
SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
W1_TRAVERSAL_BASELINE = 25.923669775527927
W1_INSTRUCTION_BASELINE = 361.20658023962375
HARDWARE_EVENTS = ("instructions", "cycles", "branches", "branch-misses")


class TerminalAuditError(RuntimeError):
    pass


class RouteAuditIssue(TerminalAuditError):
    def __init__(self, verdict: str, detail: str) -> None:
        super().__init__(detail)
        self.verdict = verdict
        self.detail = detail


class FailureSet:
    def __init__(self) -> None:
        self.provenance: list[str] = []
        self.thermal: list[str] = []
        self.capability: list[str] = []
        self.measurement: list[str] = []
        self.perturbation: list[str] = []

    def dispatch(self) -> tuple[str | None, list[str]]:
        for verdict, rows in (
            ("BLOCKED_PROVENANCE", self.provenance),
            ("BLOCKED_THERMAL", self.thermal),
            ("BLOCKED_CAPABILITY", self.capability),
            ("BLOCKED_MEASUREMENT", self.measurement),
            ("BLOCKED_PERTURBATION", self.perturbation),
        ):
            if rows:
                return verdict, rows
        return None, []

    def value(self) -> dict[str, list[str]]:
        return {name: list(getattr(self, name)) for name in ("provenance", "thermal", "capability", "measurement", "perturbation")}


def need(condition: bool, message: str) -> None:
    if not condition:
        raise TerminalAuditError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def write_new(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(value)
            output.flush()
            os.fsync(output.fileno())
    finally:
        os.close(descriptor)
    os.chmod(path, mode)


def write_json(path: pathlib.Path, value: Any) -> None:
    write_new(path, canonical(value))


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def run(command: Sequence[str], *, timeout: float = 3_600) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if result.returncode != 0:
        raise TerminalAuditError(f"command failed ({result.returncode}): {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-5000:]}")
    return result


def ssh(command: Sequence[str]) -> bytes:
    return run(["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", REMOTE, shlex.join(list(command))]).stdout


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"manifest absent: {root}")
    expected = set()
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        need(relative not in expected and path.is_file() and sha256_file(path) == digest, f"manifest drift: {relative}")
        expected.add(relative)
    actual = {path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file() and path != manifest}
    need(actual == expected, f"manifest inventory drift: {root}")
    return len(expected)


def sums(root: pathlib.Path) -> None:
    rows = [f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n" for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file() and candidate.name != "SHA256SUMS")]
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def fixed_inputs() -> dict[str, Any]:
    for path in (IMPLEMENTATION_RECEIPT, BOOTSTRAP_AUDIT):
        need(path.is_file(), f"terminal predecessor absent: {path}")
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    bootstrap = json.loads(BOOTSTRAP_AUDIT.read_text())
    need(implementation.get("verdict") == "M2_CONTROLLER_V3_VERIFIED_UNRUN", "implementation verdict drift")
    need(bootstrap.get("verdict") == "M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED", "bootstrap audit verdict drift")
    build = json.loads(BUILD_AUDIT.read_text()) if BUILD_AUDIT.is_file() else None
    if build is not None:
        need(
            build.get("verdict")
            in {"M2_BUILD_AUDITED_PARITY_ADMITTED", "BLOCKED_BUILD", "BLOCKED_PROVENANCE"},
            "build audit verdict drift",
        )
    for path in (AUDITOR, LOCAL_CONTROLLER, REMOTE_CONTROLLER):
        need(path.is_file(), f"source absent: {path}")
        compile(path.read_text(), str(path), "exec")
    return {
        "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT),
        "build_audit_sha256": sha256_file(BUILD_AUDIT) if BUILD_AUDIT.is_file() else None,
        "build_audit_verdict": build.get("verdict") if build is not None else None,
        "build_audit_failure_causes": build.get("failure_causes", []) if build is not None else [],
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
    }


def self_check() -> dict[str, Any]:
    values = fixed_inputs()
    need(not RESULT.exists(), "terminal audit result already exists")
    need(SAMPLE.size == 118 and RECORDS == 7_640 and MEASURED_EDGES == 502_915_120, "M2 denominator constants drift")
    need(tuple(PAIR_ROUTES["B"] + PAIR_ROUTES["G"] + PAIR_ROUTES["I"]) != ROUTE_ORDER, "pair registry accidentally used as physical order")
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-terminal-audit-self-check.v3",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2_TERMINAL_AUDITOR_VERIFIED_UNRUN",
        **values,
        "route_order": list(ROUTE_ORDER),
        "pair_routes": {key: list(value) for key, value in PAIR_ROUTES.items()},
        "remote_writes": 0,
    }


def live_projection() -> dict[str, Any]:
    code = r'''
import hashlib,json,os,pathlib,stat,sys
parent=pathlib.Path(sys.argv[1]); state=pathlib.Path(sys.argv[2])
def sha(p):
 d=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''): d.update(b)
 return d.hexdigest()
def row(p): return {'path':str(p),'mode':f'{stat.S_IMODE(p.stat().st_mode):04o}','size_bytes':p.stat().st_size,'sha256':sha(p)}
states=sorted(state.glob('STATE-*.json')); markers=sorted((state/'markers').iterdir())
ps=os.popen("ps -eo pid=,args=").read().splitlines(); active=[x.strip() for x in ps if ('v10_m2_fused_minimum' in x or str(parent) in x) and 'python3 -c' not in x]
print(json.dumps({'hostname':os.uname().nodename,'uid':os.geteuid(),'parent_mode':f'{stat.S_IMODE(parent.stat().st_mode):04o}','state_mode':f'{stat.S_IMODE(state.stat().st_mode):04o}','parent_entries':sorted(p.name for p in parent.iterdir()),'state_rows':[{'row':row(p),'value':json.loads(p.read_text())} for p in states],'markers':[{'name':p.name,'row':row(p),'value':json.loads(p.read_text())} for p in markers],'active_subjects':active,'remote_writes':0},sort_keys=True))
'''
    return json.loads(ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", code, str(REMOTE_PARENT), str(REMOTE_STATE)]).decode().strip().splitlines()[-1])


def copy_remote(destination: pathlib.Path) -> None:
    run(["/usr/bin/scp", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-q", "-p", "-r", f"{REMOTE}:{REMOTE_PARENT}", str(destination)], timeout=3_600)


def parse_samples(path: pathlib.Path) -> list[dict[str, Any]]:
    raw = path.read_bytes()
    need(len(raw) == RECORDS * SAMPLE.size, "component sample byte denominator drift")
    rows = []
    for offset in range(0, len(raw), SAMPLE.size):
        values = SAMPLE.unpack_from(raw, offset)
        rows.append({
            "query": values[0],
            "round": values[1],
            "worker": values[2],
            "flags": values[3],
            "outer_wall_ns": values[4],
            "outer_cpu_ns": values[5],
            "phases": [{"wall_ns": values[6 + index * 2], "cpu_ns": values[7 + index * 2]} for index in range(6)],
        })
    return rows


def numeric(value: Any) -> float | None:
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return float(value)
    if not isinstance(value, str):
        return None
    compact = value.strip().replace(",", "")
    if not compact or compact.startswith("<"):
        return None
    try:
        return float(compact)
    except ValueError:
        return None


def event_identity(value: str) -> tuple[str | None, str]:
    normalized = value.strip().lower().replace(":u", "")
    match = re.fullmatch(r"cpu_(core|atom)/([^/]+)/", normalized)
    if match:
        return match.group(1), match.group(2)
    return None, normalized.strip("/")


def task_ns(row: Mapping[str, Any], counter: float) -> float:
    unit = str(row.get("unit", "")).strip().lower()
    scales = {"msec": 1_000_000.0, "ms": 1_000_000.0, "usec": 1_000.0, "us": 1_000.0, "nsec": 1.0, "ns": 1.0, "sec": 1_000_000_000.0, "s": 1_000_000_000.0}
    need(unit in scales, f"task-clock unit drift: {unit!r}")
    return counter * scales[unit]


def parse_perf(path: pathlib.Path) -> dict[str, Any]:
    rows = []
    for line in path.read_text(errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and isinstance(value.get("event"), str):
            rows.append(value)
    need(rows, "perf JSON rows absent")
    counters = {}
    for event in HARDWARE_EVENTS:
        matched = [row for row in rows if event_identity(row["event"])[1] == event]
        need(len(matched) == 2 and {event_identity(row["event"])[0] for row in matched} == {"core", "atom"}, f"hybrid row closure drift: {event}")
        core = next(row for row in matched if event_identity(row["event"])[0] == "core")
        atom = next(row for row in matched if event_identity(row["event"])[0] == "atom")
        core_value = numeric(core.get("counter-value"))
        core_runtime = numeric(core.get("event-runtime"))
        core_running = numeric(core.get("pcnt-running"))
        need(core_value is not None and core_runtime is not None and core_runtime > 0 and core_running is not None and abs(core_running - 100.0) <= 0.01, f"core event incomplete or scaled: {event}")
        need(numeric(atom.get("counter-value")) is None and (numeric(atom.get("event-runtime")) or 0) == 0, f"atom event unexpectedly active: {event}")
        counters[event] = core_value
    task_rows = [row for row in rows if event_identity(row["event"])[1] == "task-clock"]
    need(len(task_rows) == 1, "task-clock row cardinality drift")
    task_value = numeric(task_rows[0].get("counter-value"))
    running = numeric(task_rows[0].get("pcnt-running"))
    need(task_value is not None and running is not None and abs(running - 100.0) <= 0.01, "task-clock incomplete or scaled")
    task_clock = task_ns(task_rows[0], task_value)
    need(task_clock > 0 and counters["cycles"] > 0 and counters["instructions"] > 0, "physical denominator absent")
    return {
        "rows": rows,
        "instructions_per_edge": counters["instructions"] / MEASURED_EDGES,
        "cycles_per_edge": counters["cycles"] / MEASURED_EDGES,
        "branches_per_edge": counters["branches"] / MEASURED_EDGES,
        "branch_miss_rate": counters["branch-misses"] / counters["branches"],
        "task_clock_ns_per_edge": task_clock / MEASURED_EDGES,
        "effective_frequency_ghz": counters["cycles"] / task_clock,
        "ipc": counters["instructions"] / counters["cycles"],
    }


def validate_route(root: pathlib.Path, route: str) -> dict[str, Any]:
    try:
        verify_manifest(root)
        wrapper = json.loads((root / "ROUTE_WRAPPER.json").read_text())
        need(wrapper.get("route") == route and wrapper.get("verdict") == "PASS_UNAUDITED", f"route producer verdict drift: {route}")
        need(wrapper.get("perf_stat_invocations") == 1 and wrapper.get("perf_record_invocations") == 0 and wrapper.get("subject_executions") == 1, f"route ledger drift: {route}")
    except Exception as error:
        raise RouteAuditIssue("BLOCKED_PROVENANCE", f"{route}: {error}") from error
    try:
        receipt = wrapper.get("subject_receipt", {})
        need(receipt.get("verdict") == "PASS" and receipt.get("route") == route, f"subject receipt drift: {route}")
        need(receipt.get("queries") == QUERIES and receipt.get("rounds") == ROUNDS and receipt.get("workers") == 1, f"subject denominator drift: {route}")
        need(
            receipt.get("warmup_rounds") == 1
            and receipt.get("start_barriers") == ROUNDS + 1
            and receipt.get("end_barriers") == ROUNDS + 1
            and receipt.get("cpus") == [0]
            and receipt.get("worker_affinity") == [0]
            and receipt.get("worker_migration_delta") == 0
            and receipt.get("parent_affinity") == [0]
            and receipt.get("parent_migration_delta") == 0,
            f"subject envelope drift: {route}",
        )
        samples = parse_samples(root / "subject/component-samples.bin")
        seen = set()
        for row in samples:
            need(0 <= row["query"] < QUERIES and 0 <= row["round"] < ROUNDS and row["worker"] == 0 and row["flags"] == 0, f"component record invalid: {route}")
            coordinate = (row["query"], row["round"])
            need(coordinate not in seen, f"duplicate component coordinate: {route}")
            seen.add(coordinate)
        need(len(seen) == RECORDS, f"component coordinate closure drift: {route}")
        structures = json.loads((root / "subject/structure.json").read_text()).get("queries", [])
        need(len(structures) == QUERIES and sum(int(row.get("examined_edges", 0)) for row in structures) == EDGES_PER_ROUND, f"structure denominator drift: {route}")
        traversal_cpu = sum(row["phases"][3]["cpu_ns"] for row in samples)
        need(traversal_cpu > 0, f"traversal CPU absent: {route}")
    except Exception as error:
        raise RouteAuditIssue("BLOCKED_MEASUREMENT", f"{route}: {error}") from error
    try:
        perf = parse_perf(root / "perf.raw")
    except Exception as error:
        raise RouteAuditIssue("BLOCKED_CAPABILITY", f"{route}: {error}") from error
    return {
        "route": route,
        "candidate": receipt.get("candidate"),
        "records": len(samples),
        "measured_edges": MEASURED_EDGES,
        "traversal_thread_cpu_ns": traversal_cpu,
        "traversal_ns_per_edge": traversal_cpu / MEASURED_EDGES,
        "perf": perf,
        "thermal_throttle_drift": wrapper.get("thermal_throttle_drift"),
        "wrapper_sha256": sha256_file(root / "ROUTE_WRAPPER.json"),
    }


def pair_spread(left: float, right: float) -> float:
    mean = (left + right) / 2.0
    need(mean > 0 and math.isfinite(mean), "pair mean invalid")
    return abs(left - right) / mean


def pair_metrics(routes: Mapping[str, Mapping[str, Any]], key: str) -> dict[str, Any]:
    left_name, right_name = PAIR_ROUTES[key]
    left, right = routes[left_name], routes[right_name]
    fields = {
        "traversal_ns_per_edge": (left["traversal_ns_per_edge"], right["traversal_ns_per_edge"]),
        "cycles_per_edge": (left["perf"]["cycles_per_edge"], right["perf"]["cycles_per_edge"]),
        "instructions_per_edge": (left["perf"]["instructions_per_edge"], right["perf"]["instructions_per_edge"]),
        "effective_frequency_ghz": (left["perf"]["effective_frequency_ghz"], right["perf"]["effective_frequency_ghz"]),
    }
    return {
        "routes": [left_name, right_name],
        "means": {name: sum(values) / 2.0 for name, values in fields.items()},
        "spreads": {name: pair_spread(*values) for name, values in fields.items() if name != "effective_frequency_ghz"},
        "raw": {name: list(values) for name, values in fields.items()},
    }


def decide(routes: Mapping[str, Mapping[str, Any]]) -> dict[str, Any]:
    failures = FailureSet()
    for route, value in routes.items():
        if value.get("thermal_throttle_drift") not in ({}, None):
            failures.thermal.append(f"{route}: thermal throttle drift")
    pairs = {key: pair_metrics(routes, key) for key in ("B", "G", "I")}
    for key, value in pairs.items():
        for metric, spread in value["spreads"].items():
            if spread > 0.02:
                failures.perturbation.append(f"{key} {metric} pair spread {spread:.9%} exceeds 2%")
    baseline = pairs["B"]["means"]
    traversal_delta = abs(baseline["traversal_ns_per_edge"] - W1_TRAVERSAL_BASELINE) / W1_TRAVERSAL_BASELINE
    instruction_delta = abs(baseline["instructions_per_edge"] - W1_INSTRUCTION_BASELINE) / W1_INSTRUCTION_BASELINE
    if traversal_delta > 0.05:
        failures.perturbation.append(f"B traversal baseline delta {traversal_delta:.9%} exceeds 5%")
    if instruction_delta > 0.01:
        failures.perturbation.append(f"B instruction baseline delta {instruction_delta:.9%} exceeds 1%")
    blocked, causes = failures.dispatch()
    candidates = {}
    for key in ("G", "I"):
        mean = pairs[key]["means"]
        cpu_gain = (baseline["traversal_ns_per_edge"] - mean["traversal_ns_per_edge"]) / baseline["traversal_ns_per_edge"]
        cycle_gain = (baseline["cycles_per_edge"] - mean["cycles_per_edge"]) / baseline["cycles_per_edge"]
        instruction_change = (mean["instructions_per_edge"] - baseline["instructions_per_edge"]) / baseline["instructions_per_edge"]
        frequency_delta = abs(mean["effective_frequency_ghz"] - baseline["effective_frequency_ghz"]) / baseline["effective_frequency_ghz"]
        passed = blocked is None and cpu_gain >= 0.05 and cycle_gain >= 0.05 and instruction_change <= 0.01 and frequency_delta <= 0.03
        candidates[key] = {"candidate": "M1_GUARDED_CHAIN" if key == "G" else "INTERLEAVED_RUNNING_MIN", "cpu_gain": cpu_gain, "cycle_gain": cycle_gain, "instruction_delta": instruction_change, "frequency_delta": frequency_delta, "pass": passed}
    selected = None
    passed = [key for key, value in candidates.items() if value["pass"]]
    if len(passed) == 1:
        selected = passed[0]
    elif len(passed) == 2:
        g_cpu = pairs["G"]["means"]["traversal_ns_per_edge"]
        i_cpu = pairs["I"]["means"]["traversal_ns_per_edge"]
        faster = min(g_cpu, i_cpu)
        if abs(g_cpu - i_cpu) / faster > 0.01:
            selected = "G" if g_cpu < i_cpu else "I"
        else:
            g_instructions = pairs["G"]["means"]["instructions_per_edge"]
            i_instructions = pairs["I"]["means"]["instructions_per_edge"]
            lower = min(g_instructions, i_instructions)
            selected = ("G" if g_instructions < i_instructions else "I") if abs(g_instructions - i_instructions) / lower > 0.01 else "G"
    verdict = blocked or ("W1_FUSED_MINIMUM_MECHANISM_PASS" if selected else "W1_FUSED_MINIMUM_MECHANISM_REJECTED")
    return {
        "verdict": verdict,
        "failure_causes": causes,
        "all_failure_sets": failures.value(),
        "pairs": pairs,
        "baseline_validity": {"traversal_delta": traversal_delta, "instruction_delta": instruction_delta},
        "candidates": candidates,
        "selected": candidates[selected]["candidate"] if selected else None,
    }


def validate_live(value: dict[str, Any], controller_sha: str, build_audit_verdict: str | None) -> dict[str, Any]:
    need(value.get("hostname") == "e-MEGA-MINI-M1-13th" and value.get("uid") == 0, "live identity drift")
    need(value.get("active_subjects") == [], "M2 process remains active")
    states = value.get("state_rows", [])
    need(3 <= len(states) <= 10 and [row["value"]["sequence"] for row in states] == list(range(len(states))), "state sequence drift")
    latest = states[-1]["value"].get("state")
    build_terminal = len(states) == 3
    if build_terminal:
        need(
            latest == "BLOCKED_BUILD"
            or (
                latest == "BUILD_CREATED_UNAUDITED"
                and build_audit_verdict in {"BLOCKED_BUILD", "BLOCKED_PROVENANCE"}
            ),
            "early build terminal state drift",
        )
        blocked = "BLOCKED_BUILD" if latest == "BLOCKED_BUILD" else build_audit_verdict
        parity_executed = False
        executed_count = 0
    else:
        blocked = latest if isinstance(latest, str) and latest.startswith("BLOCKED_") else None
        parity_executed = True
        executed_count = len(states) - 4
        need(blocked is not None or (executed_count == 6 and latest == "B1-ITERATOR_PASS"), "terminal producer state drift")
    executed_routes = list(ROUTE_ORDER[:executed_count])
    markers = value.get("markers", [])
    need(len(markers) == 8, "terminal marker cardinality drift")
    expected_routes = ("BUILD", "PARITY", *ROUTE_ORDER)
    need({row["value"].get("route") for row in markers} == set(expected_routes), "marker route set drift")
    for row in markers:
        marker = row["value"]
        need(row["row"]["mode"] == "0400" and marker.get("task_id") == TASK_ID and marker.get("transaction_id") == TRANSACTION_ID and marker.get("controller_sha256") == controller_sha and marker.get("retry_permitted") is False, f"marker identity drift: {row['name']}")
        route = marker.get("route")
        consumed_routes = {"BUILD", *executed_routes}
        if parity_executed:
            consumed_routes.add("PARITY")
        consumed = route in consumed_routes
        expected_suffix = ".consumed-before-exec" if consumed else ".available"
        need(row["name"].endswith(expected_suffix), f"marker consumption projection drift: {route}")
    consumed_count = 1 + int(parity_executed) + executed_count
    return {
        "states": len(states),
        "markers_created": 8,
        "markers_consumed": consumed_count,
        "markers_available": 8 - consumed_count,
        "executed_routes": executed_routes,
        "parity_executed": parity_executed,
        "build_terminal": build_terminal,
        "latest_state": latest,
        "blocked_verdict": blocked,
        "active_subjects": 0,
    }


def blocked_science(verdict: str, causes: Sequence[str]) -> dict[str, Any]:
    return {
        "verdict": verdict,
        "failure_causes": list(causes),
        "all_failure_sets": {
            "provenance": list(causes) if verdict == "BLOCKED_PROVENANCE" else [],
            "build": list(causes) if verdict == "BLOCKED_BUILD" else [],
            "parity": list(causes) if verdict == "BLOCKED_PARITY" else [],
            "thermal": list(causes) if verdict == "BLOCKED_THERMAL" else [],
            "capability": list(causes) if verdict == "BLOCKED_CAPABILITY" else [],
            "measurement": list(causes) if verdict == "BLOCKED_MEASUREMENT" else [],
            "perturbation": list(causes) if verdict == "BLOCKED_PERTURBATION" else [],
        },
        "pairs": None,
        "baseline_validity": None,
        "candidates": None,
        "selected": None,
    }


def audit() -> dict[str, Any]:
    check = self_check()
    before = live_projection()
    live = validate_live(before, check["local_controller_sha256"], check["build_audit_verdict"])
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        copy_remote(stage / "REMOTE_EVIDENCE")
        remote = stage / "REMOTE_EVIDENCE"
        routes: dict[str, Any] = {}
        route_issues: list[RouteAuditIssue] = []
        cargo_invocations = 1
        if live["build_terminal"]:
            if live["latest_state"] == "BLOCKED_BUILD":
                failure_root = remote / "build-failure-v1"
                verify_manifest(failure_root)
                failure = json.loads((failure_root / "BUILD_FAILURE.json").read_text())
                need(failure.get("verdict") == "BLOCKED_BUILD", "remote build failure verdict drift")
                cargo_invocations = int(failure.get("cargo_invocations", 0))
                scientific = blocked_science("BLOCKED_BUILD", [str(failure.get("error", "M2 build failed"))])
            else:
                verify_manifest(remote / "build-v1")
                need(
                    check["build_audit_verdict"] in {"BLOCKED_BUILD", "BLOCKED_PROVENANCE"},
                    "blocked build audit predecessor absent",
                )
                scientific = blocked_science(
                    check["build_audit_verdict"],
                    check["build_audit_failure_causes"] or ["independent M2 build audit blocked"],
                )
        else:
            verify_manifest(remote / "build-v1")
            verify_manifest(remote / "parity-v1")
            parity = json.loads((remote / "parity-v1/PARITY_WRAPPER.json").read_text())
            need(
                parity.get("verdict") in {"PASS", "BLOCKED_PARITY", "BLOCKED_PROVENANCE"},
                "sealed parity verdict drift",
            )
            for route in live["executed_routes"]:
                route_root = remote / ROUTE_DIRS[route]
                try:
                    verify_manifest(route_root)
                    wrapper = json.loads((route_root / "ROUTE_WRAPPER.json").read_text())
                except Exception as error:
                    route_issues.append(RouteAuditIssue("BLOCKED_PROVENANCE", f"{route}: {error}"))
                    continue
                if wrapper.get("verdict") == "PASS_UNAUDITED":
                    try:
                        routes[route] = validate_route(route_root, route)
                    except RouteAuditIssue as issue:
                        route_issues.append(issue)
                else:
                    need(wrapper.get("verdict") in {"BLOCKED_PROVENANCE", "BLOCKED_THERMAL"}, f"unknown producer route verdict: {route}")
            if parity.get("verdict") == "BLOCKED_PARITY":
                scientific = blocked_science("BLOCKED_PARITY", ["M2 semantic parity failed"])
            elif parity.get("verdict") == "BLOCKED_PROVENANCE":
                scientific = blocked_science(
                    "BLOCKED_PROVENANCE",
                    [f"parity controller failure: {parity.get('controller_error') or 'incomplete parity observation'}"],
                )
            elif live["blocked_verdict"] is not None:
                scientific = blocked_science(live["blocked_verdict"], [f"producer stopped at {live['executed_routes'][-1]}"])
            elif route_issues:
                priority = {"BLOCKED_PROVENANCE": 0, "BLOCKED_THERMAL": 1, "BLOCKED_CAPABILITY": 2, "BLOCKED_MEASUREMENT": 3, "BLOCKED_PERTURBATION": 4}
                verdict = min((issue.verdict for issue in route_issues), key=priority.__getitem__)
                scientific = blocked_science(verdict, [issue.detail for issue in route_issues if issue.verdict == verdict])
            else:
                need(set(routes) == set(ROUTE_ORDER), "complete M2 route set absent without blocked verdict")
                scientific = decide(routes)
        after = live_projection()
        need(after == before, "remote state changed during terminal audit")
        receipt = {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-terminal-audit.v3",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": scientific["verdict"],
            "local_controller_sha256": check["local_controller_sha256"],
            "remote_controller_sha256": check["remote_controller_sha256"],
            "terminal_auditor_sha256": check["auditor_sha256"],
            "implementation_receipt_sha256": check["implementation_receipt_sha256"],
            "bootstrap_audit_sha256": check["bootstrap_audit_sha256"],
            "build_audit_sha256": check["build_audit_sha256"],
            "live_terminal": live,
            "route_order": list(ROUTE_ORDER),
            "routes": routes,
            "scientific": scientific,
            "markers_created": 8,
            "markers_consumed": live["markers_consumed"],
            "markers_available": live["markers_available"],
            "cargo_invocations": cargo_invocations,
            "perf_stat_invocations": len(live["executed_routes"]),
            "perf_record_invocations": 0,
            "subject_executions": int(live["parity_executed"]) + len(live["executed_routes"]),
            "runtime_authority_changed": False,
            "production_edit_admitted": False,
            "retry_permitted": False,
            "next_action_admitted": (
                "separate test-only source decision paper only"
                if scientific["verdict"] == "W1_FUSED_MINIMUM_MECHANISM_PASS"
                else (
                    "new DAFSA-decode paper only; minimum lowering closed"
                    if scientific["verdict"] == "W1_FUSED_MINIMUM_MECHANISM_REJECTED"
                    else "none; terminal M2 blocked verdict"
                )
            ),
        }
        write_json(stage / "M2_TERMINAL_AUDIT_RECEIPT.json", receipt)
        write_json(stage / "SELF_CHECK.json", check)
        write_json(stage / "REMOTE_BEFORE.json", before)
        write_json(stage / "REMOTE_AFTER.json", after)
        write_new(stage / "terminal-auditor.py", AUDITOR.read_bytes())
        sums(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
        return {**receipt, "receipt_sha256": sha256_file(RESULT / "M2_TERMINAL_AUDIT_RECEIPT.json")}
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2 TERMINAL AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
