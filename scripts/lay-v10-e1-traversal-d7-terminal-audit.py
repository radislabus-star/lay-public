#!/usr/bin/env python3
"""Independent read-only terminal audit for the D7 worker/topology sweep."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import pathlib
import re
import shlex
import stat
import struct
import subprocess
import sys
from collections.abc import Sequence
from typing import Any


TASK_ID = "slice8b-v10-e1-traversal-d7-worker-topology-sweep-v1-20260826"
TRANSACTION_ID = "d0982f48bba3090a155713c32a73bbc71f7ef79a0f5fa1eccbea4423563102e0"
REMOTE = "e@192.168.3.94"
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_RESULT = REMOTE_PARENT / "result-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
ROUTE_CPUS: dict[str, tuple[int, ...]] = {
    "W1": (0,),
    "W6": (0, 2, 4, 6, 8, 10),
    "W12": tuple(range(12)),
    "W14": (0, 2, 4, 6, 8, 10, 12, 13, 14, 15, 16, 17, 18, 19),
    "W20": tuple(range(20)),
}
ROUTE_ORDER = tuple(ROUTE_CPUS)
MARKER_ROUTES = ("build", "parity", *ROUTE_ORDER)
HARDWARE_EVENTS = ("instructions", "cycles", "branches", "branch-misses")
QUERIES = 382
ROUNDS = 20
EDGES_PER_ROUND = 25_145_756
MEASURED_EDGES = EDGES_PER_ROUND * ROUNDS
COMPONENT_SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
EXPECTED = {
    "paper": "090c39efec9916c1a9bea050c6385a17c00b95532e08592730b3b83ede591a23",
    "preflight": "c35e144ed81bd244bbf7d2233f0f8da86fb6fd4aedd1f6d5e25608b9bd0cc700",
    "preflight_receipt": "bb92b6d1059090b0da024cca686202c19386b85b97fc2563adcca84107400c20",
    "fragment": "8c9ff3aaf43942aff6090b1350cef1828e24ea5664d312bde2ebdf29be6687ce",
    "assembled_source": "c9f6400304eed2d84717cbef84b17e5ba1817e08f8d9e68580b5d441d1f13803",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
}


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


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


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode()


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"manifest absent: {manifest}")
    expected: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, relative = line.split("  ", 1)
        require(relative not in expected, f"duplicate manifest path: {relative}")
        expected.add(relative)
        path = root / relative
        require(path.is_file(), f"manifest member absent: {path}")
        require(sha256_file(path) == digest, f"manifest SHA mismatch: {path}")
        require(mode_string(path) == "0444", f"unsealed evidence file: {path}")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path != manifest
    }
    require(actual == expected, "manifest inventory mismatch")
    require(mode_string(manifest) == "0444", "manifest mode drift")
    return len(expected)


def run(command: Sequence[str], *, timeout: float = 3_600) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )
    if result.returncode != 0:
        raise AuditError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            + result.stderr.decode(errors="replace")[-5000:]
        )
    return result


def ssh(command: Sequence[str], *, timeout: float = 3_600) -> bytes:
    return run(
        [
            "ssh",
            "-i",
            "/home/ubu/.ssh/mega-mini-admin",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=6",
            REMOTE,
            shlex.join(list(command)),
        ],
        timeout=timeout,
    ).stdout


def marker_payload(route: str) -> bytes:
    return canonical_json_bytes(
        {
            "schema": "lay.v10.e1-traversal-d7-marker.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "retry_permitted": False,
        }
    )


def independent_remote_probe() -> dict[str, Any]:
    code = r'''
import hashlib,json,os,pathlib,stat,subprocess,sys
task=sys.argv[1]; transaction=sys.argv[2]
parent=pathlib.Path(sys.argv[3]); build=parent/'build-v1'; result=parent/'result-v1'; state=pathlib.Path(sys.argv[4])
routes=('build','parity','W1','W6','W12','W14','W20')
def sha(path):
 d=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''): d.update(b)
 return d.hexdigest()
def mode(path): return f'{stat.S_IMODE(path.stat().st_mode):04o}'
def canonical(value): return (json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':'))+'\n').encode()
def manifest(root):
 lines=(root/'SHA256SUMS').read_text().splitlines(); expected=set()
 for line in lines:
  digest,rel=line.split('  ',1); p=root/rel
  assert rel not in expected and p.is_file() and sha(p)==digest
  expected.add(rel)
 actual={p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file() and p.name!='SHA256SUMS'}
 assert actual==expected
 return len(expected)
markers=state/'markers'; marker_rows={}
for route in routes:
 candidates=[markers/(route.lower()+'.available'),markers/(route.lower()+'.consumed-before-exec')]; present=[p for p in candidates if p.is_file()]
 assert len(present)==1; p=present[0]; name=p.name
 payload={'schema':'lay.v10.e1-traversal-d7-marker.v1','task_id':task,'transaction_id':transaction,'route':route,'retry_permitted':False}
 marker_rows[route]={'name':name,'sha256':sha(p),'size_bytes':p.stat().st_size,'mode':mode(p),'payload_exact':p.read_bytes()==canonical(payload)}
available=sorted(p.name for p in markers.glob('*.available'))
consumed=sorted(p.name for p in markers.glob('*.consumed-before-exec'))
states=[{'name':p.name,'sha256':sha(p),'mode':mode(p),'value':json.loads(p.read_text())} for p in sorted(state.glob('STATE-*.json'))]
machine=pathlib.Path('/etc/machine-id')
ps=subprocess.run(['ps','-eo','pid=,ppid=,args='],stdout=subprocess.PIPE,check=False).stdout.decode(errors='replace').splitlines()
excluded={os.getpid(),os.getppid()}; active=[]
for row in ps:
 fields=row.strip().split(maxsplit=2)
 if len(fields)<3: continue
 pid=int(fields[0]); args=fields[2]
 if pid not in excluded and ('v10_d7_worker_topology_sweep' in args or str(parent) in args): active.append(row.strip())
runtime={}
for p in sorted(pathlib.Path('/home/e/.local/bin').glob('lay*')):
 try:
  resolved=p.resolve(strict=True)
  if resolved.is_file(): runtime[p.name]={'target':str(resolved),'sha256':sha(resolved)}
 except OSError: pass
processes=[]
for row in ps:
 fields=row.strip().split(maxsplit=2)
 if len(fields)<3 or int(fields[0]) in excluded: continue
 args=fields[2]
 if 'lay-daemon' in args or 'lay-ibus-engine' in args or 'ibus-daemon' in args: processes.append(fields[0]+' '+args)
managed=sorted(x for x in processes if 'lay-daemon' in x or 'lay-ibus-engine' in x)
ibus=sorted(x for x in processes if 'ibus-daemon' in x and 'lay-ibus-engine' not in x)
decision=json.loads((result/'D7_DECISION.json').read_text()); build_failure=parent/'build-failure-v1'
print(json.dumps({'hostname':os.uname().nodename,'machine_id_sha256':sha(machine),'parent_mode':mode(parent),'state_mode':mode(state),'markers_mode':mode(markers),'build_exists':build.is_dir(),'build_failure_exists':build_failure.is_dir(),'build_mode':mode(build) if build.is_dir() else None,'result_mode':mode(result),'build_manifest_files':manifest(build) if build.is_dir() else None,'build_failure_manifest_files':manifest(build_failure) if build_failure.is_dir() else None,'result_manifest_files':manifest(result),'build_provenance_sha256':sha(build/'BUILD_PROVENANCE.json') if build.is_dir() else None,'build_manifest_sha256':sha(build/'SHA256SUMS') if build.is_dir() else None,'decision_sha256':sha(result/'D7_DECISION.json'),'decision_verdict':decision.get('verdict'),'available':available,'consumed':consumed,'marker_rows':marker_rows,'state_files':states,'route_lock_mode':mode(state/'route.lock'),'active':active,'runtime':{'installed_lay_hashes':runtime,'managed_lay_process_ids':[x.split(maxsplit=1)[0] for x in managed],'global_ibus_pid':ibus[0].split(maxsplit=1)[0] if ibus else None},'remote_writes':0},sort_keys=True))
'''
    raw = ssh(
        [
            "python3",
            "-c",
            code,
            TASK_ID,
            TRANSACTION_ID,
            str(REMOTE_PARENT),
            str(REMOTE_STATE),
        ]
    )
    return json.loads(raw.decode().splitlines()[-1])


def expected_chunks(workers: int) -> list[tuple[int, int]]:
    size = math.ceil(QUERIES / workers)
    return [
        (worker * size, min((worker + 1) * size, QUERIES))
        for worker in range(workers)
    ]


def close_enough(left: float, right: float, *, relative: float = 1e-12) -> bool:
    return math.isclose(left, right, rel_tol=relative, abs_tol=1e-9)


def audit_samples(root: pathlib.Path, route: str, wrapper: dict[str, Any]) -> dict[str, Any]:
    path = root / route / "subject/component-samples.bin"
    raw = path.read_bytes()
    require(len(raw) == QUERIES * ROUNDS * COMPONENT_SAMPLE.size, f"{route} sample bytes")
    chunks = expected_chunks(len(ROUTE_CPUS[route]))
    owner = {
        query: worker
        for worker, (start, end) in enumerate(chunks)
        for query in range(start, end)
    }
    seen = set()
    traversal_cpu = 0
    outer_cpu = 0
    for offset in range(0, len(raw), COMPONENT_SAMPLE.size):
        values = COMPONENT_SAMPLE.unpack_from(raw, offset)
        query, round_id, worker, flags = values[:4]
        require(0 <= query < QUERIES and 0 <= round_id < ROUNDS, f"{route} coordinate")
        require(worker == owner[query] and flags == 0, f"{route} worker/flags")
        coordinate = (query, round_id)
        require(coordinate not in seen, f"{route} duplicate coordinate")
        seen.add(coordinate)
        outer_cpu += values[5]
        traversal_cpu += values[6 + 3 * 2 + 1]
    require(len(seen) == QUERIES * ROUNDS, f"{route} coordinate coverage")
    statistics = wrapper["statistics"]
    require(statistics["records"] == QUERIES * ROUNDS, f"{route} record denominator")
    require(statistics["measured_edges"] == MEASURED_EDGES, f"{route} edge denominator")
    require(statistics["traversal_thread_cpu_ns"] == traversal_cpu, f"{route} traversal CPU sum")
    require(statistics["outer_thread_cpu_ns"] == outer_cpu, f"{route} outer CPU sum")
    require(
        close_enough(statistics["traversal_ns_per_edge"], traversal_cpu / MEASURED_EDGES),
        f"{route} traversal CPU/edge",
    )
    structure = load_json(root / route / "subject/structure.json")["queries"]
    require(len(structure) == QUERIES, f"{route} structure rows")
    require(
        sum(int(row["examined_edges"]) for row in structure) == EDGES_PER_ROUND,
        f"{route} structure edge denominator",
    )
    return {"records": len(seen), "traversal_thread_cpu_ns": traversal_cpu}


def numeric_counter(value: Any) -> float | None:
    if isinstance(value, (int, float)):
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


def event_identity(event: str) -> tuple[str | None, str]:
    normalized = event.strip().lower().replace(":u", "")
    match = re.fullmatch(r"cpu_(core|atom)/([^/]+)/", normalized)
    if match is not None:
        return match.group(1), match.group(2)
    return None, normalized.strip("/")


def task_clock_ns(row: dict[str, Any], counter: float) -> float:
    unit = str(row.get("unit", "")).strip().lower()
    factors = {
        "msec": 1_000_000,
        "ms": 1_000_000,
        "usec": 1_000,
        "us": 1_000,
        "nsec": 1,
        "ns": 1,
        "sec": 1_000_000_000,
        "s": 1_000_000_000,
    }
    require(unit in factors, f"task-clock unit {unit!r}")
    return counter * factors[unit]


def audit_perf(root: pathlib.Path, route: str, wrapper: dict[str, Any]) -> dict[str, float]:
    perf = wrapper["perf"]
    derived = perf["derived"]
    expected_pmus = {"core"} | ({"atom"} if any(cpu >= 12 for cpu in ROUTE_CPUS[route]) else set())
    rows = []
    for line in (root / route / "perf.raw").read_text(errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and isinstance(value.get("event"), str):
            rows.append(value)
    require(rows == perf["rows"], f"{route} raw perf/projection mismatch")
    effective: dict[str, float] = {}
    for event in HARDWARE_EVENTS:
        matched = [row for row in rows if event_identity(row["event"])[1] == event]
        require(len(matched) == 2, f"{route}/{event} raw row count")
        require({event_identity(row["event"])[0] for row in matched} == {"core", "atom"}, f"{route}/{event} owners")
        active = []
        inactive = []
        for row in matched:
            pmu, _ = event_identity(row["event"])
            value = numeric_counter(row.get("counter-value"))
            runtime = numeric_counter(row.get("event-runtime"))
            running = numeric_counter(row.get("pcnt-running"))
            if value is None:
                require(str(row.get("counter-value", "")).strip().lower() == "<not counted>", f"{route}/{event} unsupported")
                require((runtime or 0) == 0 and (running or 0) == 0, f"{route}/{event} inactive runtime")
                inactive.append(pmu)
            else:
                require(runtime is not None and runtime > 0 and running is not None, f"{route}/{event} active runtime")
                active.append({"pmu": pmu, "counter": value, "runtime": runtime, "running": running})
        require({row["pmu"] for row in active} == expected_pmus, f"{route}/{event} active PMUs")
        require(set(inactive) == {"core", "atom"} - expected_pmus, f"{route}/{event} inactive PMUs")
        runtime_sum = sum(row["runtime"] for row in active)
        require(98.9 <= sum(row["running"] for row in active) <= 101.1, f"{route}/{event} running partition")
        raw_value = sum(row["counter"] * row["runtime"] / runtime_sum for row in active)
        counter = perf["counters"][event]
        parts = counter["parts"]
        require({part["pmu"] for part in parts} == expected_pmus, f"{route}/{event} PMU set")
        runtime = sum(float(part["runtime"]) for part in parts)
        value = sum(float(part["counter"]) * float(part["runtime"]) / runtime for part in parts)
        require(close_enough(raw_value, value), f"{route}/{event} raw aggregate")
        require(close_enough(value, counter["runtime_weighted_value"]), f"{route}/{event} aggregate")
        require(98.9 <= sum(float(part["running"]) for part in parts) <= 101.1, f"{route}/{event} running partition")
        effective[event] = value
    task_rows = [row for row in rows if event_identity(row["event"])[1] == "task-clock"]
    require(len(task_rows) == 1 and event_identity(task_rows[0]["event"])[0] is None, f"{route} task-clock row")
    task_counter = numeric_counter(task_rows[0].get("counter-value"))
    task_running = numeric_counter(task_rows[0].get("pcnt-running"))
    task_runtime = numeric_counter(task_rows[0].get("event-runtime"))
    require(task_counter is not None and task_runtime is not None and task_runtime > 0, f"{route} task-clock counter")
    require(task_running is not None and abs(task_running - 100.0) <= 0.01, f"{route} task-clock scaled")
    task_ns = task_clock_ns(task_rows[0], task_counter)
    require(close_enough(task_ns, float(perf["task_clock"]["value_ns"])), f"{route} task-clock projection")
    expected = {
        "instructions_per_edge": effective["instructions"] / MEASURED_EDGES,
        "cycles_per_edge": effective["cycles"] / MEASURED_EDGES,
        "branches_per_edge": effective["branches"] / MEASURED_EDGES,
        "branch_miss_rate": effective["branch-misses"] / effective["branches"],
        "ipc": effective["instructions"] / effective["cycles"],
        "effective_frequency_ghz": effective["cycles"] / task_ns,
        "task_clock_ns_per_edge": task_ns / MEASURED_EDGES,
    }
    for key, value in expected.items():
        require(close_enough(value, float(derived[key])), f"{route} derived {key}")
    return expected


def calculate_frontiers(routes: dict[str, dict[str, Any]]) -> dict[str, Any]:
    w1 = routes["W1"]["statistics"]["traversal_ns_per_edge"]
    base_throughput = routes["W1"]["statistics"]["aggregate_edges_per_second"]
    points = []
    for route in ROUTE_ORDER:
        statistics = routes[route]["statistics"]
        perf = routes[route]["perf"]["derived"]
        points.append(
            {
                "route": route,
                "workers": len(ROUTE_CPUS[route]),
                "cpus": list(ROUTE_CPUS[route]),
                "traversal_ns_per_edge": statistics["traversal_ns_per_edge"],
                "delta_from_W1_ns_per_edge": statistics["traversal_ns_per_edge"] - w1,
                "inflation_from_W1_percent": (statistics["traversal_ns_per_edge"] / w1 - 1.0) * 100.0,
                "aggregate_edges_per_second": statistics["aggregate_edges_per_second"],
                "throughput_scaling_from_W1": statistics["aggregate_edges_per_second"] / base_throughput,
                "parallel_efficiency_from_W1": statistics["aggregate_edges_per_second"] / base_throughput / len(ROUTE_CPUS[route]),
                **perf,
            }
        )
    candidates = [point for point in points if point["traversal_ns_per_edge"] <= w1 * 1.05]
    latency = max(candidates, key=lambda point: point["workers"]) if candidates else None
    throughput = max(points, key=lambda point: point["aggregate_edges_per_second"])
    pareto = []
    for point in points:
        dominated = any(
            other["route"] != point["route"]
            and other["traversal_ns_per_edge"] <= point["traversal_ns_per_edge"]
            and other["aggregate_edges_per_second"] >= point["aggregate_edges_per_second"]
            and (
                other["traversal_ns_per_edge"] < point["traversal_ns_per_edge"]
                or other["aggregate_edges_per_second"] > point["aggregate_edges_per_second"]
            )
            for other in points
        )
        if not dominated:
            pareto.append(point["route"])
    by_route = {point["route"]: point for point in points}

    def delta(right: str, left: str) -> dict[str, float]:
        return {
            "traversal_ns_per_edge": by_route[right]["traversal_ns_per_edge"] - by_route[left]["traversal_ns_per_edge"],
            "ipc": by_route[right]["ipc"] - by_route[left]["ipc"],
            "effective_frequency_ghz": by_route[right]["effective_frequency_ghz"] - by_route[left]["effective_frequency_ghz"],
            "instructions_per_edge": by_route[right]["instructions_per_edge"] - by_route[left]["instructions_per_edge"],
        }

    return {
        "schema": "lay.v10.e1-traversal-d7-frontiers.v1",
        "points": points,
        "latency_preserving_capacity": latency,
        "throughput_point": throughput,
        "pareto_routes": pareto,
        "topology_interventions": {
            "W1_to_W6_package_and_all_P_cores": delta("W6", "W1"),
            "W6_to_W12_P_core_SMT": delta("W12", "W6"),
            "W6_to_W14_add_E_cores_without_P_SMT": delta("W14", "W6"),
            "W14_to_W20_add_P_SMT_at_full_physical_saturation": delta("W20", "W14"),
        },
        "full_W20_minus_W1_ns_per_edge": by_route["W20"]["traversal_ns_per_edge"] - by_route["W1"]["traversal_ns_per_edge"],
        "historical_target_delta_ns_per_edge": 18.770001603849174,
        "production_policy_admitted": False,
    }


def audit_success(stage: pathlib.Path, decision: dict[str, Any], remote: dict[str, Any]) -> dict[str, Any]:
    root = stage / "REMOTE_RESULT"
    require(decision["verdict"] == "D7_WORKER_TOPOLOGY_SWEEP_CREATED_UNAUDITED", "producer success verdict drift")
    require(remote["build_exists"] and not remote["build_failure_exists"], "live successful build state")
    require(decision["failure"] is None, "producer success contains failure")
    require(decision["build"]["build"]["cargo_invocations"] == 1, "Cargo ledger drift")
    require(decision["parity"]["verdict"] == "PASS", "parity did not PASS")
    require(decision["parity"]["thermal_throttle_drift"] == {}, "parity thermal drift")
    require(set(decision["routes"]) == set(ROUTE_ORDER), "route registry drift")
    ledger = decision["execution_ledger"]
    require(
        ledger
        == {
            "cargo_invocations": 1,
            "subject_executions": 6,
            "perf_stat_invocations": 5,
            "perf_record_invocations": 0,
            "installed_elf_executions": 0,
            "runtime_authority_changed": False,
        },
        "terminal execution ledger drift",
    )
    route_audits = {}
    for route in ROUTE_ORDER:
        wrapper = decision["routes"][route]
        require(wrapper == load_json(root / route / "ROUTE_WRAPPER.json"), f"{route} wrapper projection drift")
        require(wrapper["verdict"] == "PASS" and not wrapper["failures"], f"{route} did not PASS")
        require(wrapper["thermal_throttle_drift"] == {}, f"{route} thermal drift")
        require(wrapper["enable_ack"].startswith("ack") and wrapper["disable_ack"].startswith("ack"), f"{route} perf acknowledgement")
        receipt = wrapper["subject_receipt"]
        require(receipt["workers"] == len(ROUTE_CPUS[route]), f"{route} worker count")
        require(receipt["cpus"] == list(ROUTE_CPUS[route]), f"{route} CPU set")
        chunks = expected_chunks(len(ROUTE_CPUS[route]))
        require(
            receipt["worker_chunks"]
            == [
                {"worker": worker, "start": start, "end": end, "queries": end - start}
                for worker, (start, end) in enumerate(chunks)
            ],
            f"{route} chunk partition",
        )
        require(receipt["worker_affinities"] == [[cpu] for cpu in ROUTE_CPUS[route]], f"{route} affinity")
        require(receipt["worker_migration_deltas"] == [0] * len(ROUTE_CPUS[route]), f"{route} migrations")
        require(receipt["parent_affinity"] == [ROUTE_CPUS[route][0]], f"{route} parent affinity")
        require(receipt["parent_migration_delta"] == 0, f"{route} parent migration")
        sample = audit_samples(root, route, wrapper)
        perf = audit_perf(root, route, wrapper)
        route_audits[route] = {**sample, **perf}
    w1 = decision["routes"]["W1"]["statistics"]["traversal_ns_per_edge"]
    w20 = decision["routes"]["W20"]["statistics"]["traversal_ns_per_edge"]
    require(abs(w1 - 25.96501044152341) / 25.96501044152341 <= 0.05, "W1 baseline gate")
    require(abs(w20 - 44.735012045372585) / 44.735012045372585 <= 0.05, "W20 baseline gate")
    frontiers = calculate_frontiers(decision["routes"])
    require(frontiers == decision["frontiers"], "independent frontier reconstruction mismatch")
    expected_consumed = sorted(f"{route.lower()}.consumed-before-exec" for route in MARKER_ROUTES)
    require(remote["available"] == [] and remote["consumed"] == expected_consumed, "live marker terminal state")
    require(all(row["mode"] == "0400" and row["payload_exact"] for row in remote["marker_rows"].values()), "live marker identity")
    expected_states = [
        "STATE-00-ALL_MARKERS_AVAILABLE.json",
        "STATE-01-BUILD_CREATED.json",
        "STATE-02-PARITY_PASS.json",
        "STATE-03-W1_PASS.json",
        "STATE-04-W6_PASS.json",
        "STATE-05-W12_PASS.json",
        "STATE-06-W14_PASS.json",
        "STATE-07-W20_PASS.json",
    ]
    require([row["name"] for row in remote["state_files"]] == expected_states, "live state sequence")
    for sequence, row in enumerate(remote["state_files"]):
        value = row["value"]
        require(row["mode"] == "0444", f"state {sequence} mode")
        require(value["task_id"] == TASK_ID and value["transaction_id"] == TRANSACTION_ID, f"state {sequence} namespace")
        require(value["sequence"] == sequence, f"state {sequence} ordinal")
        require(len(value["markers"]["consumed"]) == sequence, f"state {sequence} consumed count")
        require(len(value["markers"]["available"]) == len(MARKER_ROUTES) - sequence, f"state {sequence} available count")
    require(
        remote["state_files"][1]["value"]["build_provenance_sha256"]
        == remote["build_provenance_sha256"],
        "build state link",
    )
    require(
        remote["state_files"][2]["value"]["parity_wrapper_sha256"]
        == sha256_file(root / "PARITY/PARITY_WRAPPER.json"),
        "parity state link",
    )
    for sequence, route in enumerate(ROUTE_ORDER, start=3):
        require(
            remote["state_files"][sequence]["value"]["route_wrapper_sha256"]
            == sha256_file(root / route / "ROUTE_WRAPPER.json"),
            f"{route} state link",
        )
    return {"frontiers": frontiers, "route_audits": route_audits}


def audit(stage: pathlib.Path) -> dict[str, Any]:
    require(stage.is_dir(), "audit stage absent")
    result_root = stage / "REMOTE_RESULT"
    manifest_files = verify_manifest(result_root)
    require(sha256_file(stage / "paper.md") == EXPECTED["paper"], "paper SHA drift")
    require(sha256_file(stage / "preflight-v2.json") == EXPECTED["preflight"], "preflight SHA drift")
    require(sha256_file(stage / "preflight-v2-receipt.json") == EXPECTED["preflight_receipt"], "preflight receipt SHA drift")
    require(sha256_file(stage / "fragment.inc") == EXPECTED["fragment"], "fragment SHA drift")
    before = load_json(stage / "LOCAL_RUNTIME_BEFORE.json")
    after = load_json(stage / "LOCAL_RUNTIME_AFTER.json")
    remote_response = load_json(stage / "REMOTE_RESPONSE.json")
    pre_marker_probe = load_json(stage / "REMOTE_PRE_MARKER_PROBE.json")
    producer_status = load_json(stage / "REMOTE_STATUS_AFTER.json")
    decision = load_json(result_root / "D7_DECISION.json")
    identities = load_json(result_root / "INPUT_IDENTITIES.json")
    implementation = load_json(stage / "implementation-self-check.json")
    require(decision["task_id"] == TASK_ID and decision["transaction_id"] == TRANSACTION_ID, "decision namespace")
    require(remote_response == decision, "remote response/decision mismatch")
    require(
        pre_marker_probe["verdict"] == "D7_PRE_MARKER_PROBE_PASS"
        and pre_marker_probe["markers_created"] == 0
        and pre_marker_probe["markers_consumed"] == 0
        and pre_marker_probe["cargo_invocations"] == 0
        and pre_marker_probe["perf_stat_invocations"] == 0
        and pre_marker_probe["subject_executions"] == 0
        and pre_marker_probe["remote_writes"] == 0,
        "pre-marker probe ledger drift",
    )
    require(identities["controller_sha256"] == sha256_file(stage / "controller.py"), "controller identity drift")
    require(identities["fragment_sha256"] == EXPECTED["fragment"], "decision fragment identity")
    require(identities["assembled_source_sha256"] == EXPECTED["assembled_source"], "assembled source identity")
    require(
        implementation["verdict"] == "D7_IMPLEMENTATION_VERIFIED_UNRUN"
        and implementation["controller_sha256"] == sha256_file(stage / "controller.py")
        and implementation["terminal_auditor_sha256"] == sha256_file(stage / "terminal-auditor.py")
        and implementation["fragment_sha256"] == EXPECTED["fragment"]
        and implementation["markers_created"] == 0
        and implementation["markers_consumed"] == 0
        and implementation["cargo_invocations"] == 0
        and implementation["perf_stat_invocations"] == 0
        and implementation["subject_executions"] == 0,
        "sealed implementation closure drift",
    )
    remote = independent_remote_probe()
    require(remote["hostname"] == REMOTE_HOSTNAME, "live remote hostname drift")
    require(remote["machine_id_sha256"] == REMOTE_MACHINE_ID_SHA256, "live remote machine identity drift")
    require(remote["decision_sha256"] == sha256_file(result_root / "D7_DECISION.json"), "live/local decision SHA mismatch")
    require(remote["decision_verdict"] == decision["verdict"], "live/local verdict mismatch")
    require(remote["result_manifest_files"] == manifest_files, "live result manifest incomplete")
    if remote["build_exists"]:
        require(remote["build_manifest_files"] > 0, "live build manifest incomplete")
        require(remote["build_provenance_sha256"] == sha256_file(stage / "REMOTE_BUILD_METADATA/BUILD_PROVENANCE.json"), "build provenance mirror mismatch")
        require(remote["build_manifest_sha256"] == sha256_file(stage / "REMOTE_BUILD_METADATA/SHA256SUMS"), "build manifest mirror mismatch")
    require(not remote["active"], "owned D7 process remains active")
    require(before == after, "local installed Lay authority changed")
    require(decision["remote_runtime_before"] == decision["remote_runtime_after"], "producer remote runtime changed")
    require(remote["runtime"] == decision["remote_runtime_after"], "live remote runtime drift")
    require(producer_status["decision_sha256"] == remote["decision_sha256"], "producer/live status mismatch")
    require(producer_status["remote_writes"] == 0 and remote["remote_writes"] == 0, "audit write ledger drift")
    success = None
    if decision["verdict"] == "D7_WORKER_TOPOLOGY_SWEEP_CREATED_UNAUDITED":
        success = audit_success(stage, decision, remote)
        verdict = "D7_WORKER_TOPOLOGY_SWEEP_COMPLETE"
    else:
        require(decision["verdict"].startswith("BLOCKED_"), "unknown producer verdict")
        verdict = decision["verdict"]
    return {
        "schema": "lay.v10.e1-traversal-d7-independent-terminal-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "producer_verdict": decision["verdict"],
        "producer_decision_sha256": sha256_file(result_root / "D7_DECISION.json"),
        "controller_sha256": sha256_file(stage / "controller.py"),
        "auditor_sha256": sha256_file(pathlib.Path(__file__).resolve()),
        "fragment_sha256": EXPECTED["fragment"],
        "remote_result_manifest_files": manifest_files,
        "remote_build_manifest_files": remote["build_manifest_files"],
        "markers_created": 7,
        "markers_consumed": len(remote["consumed"]),
        "markers_available": len(remote["available"]),
        "cargo_invocations": decision["execution_ledger"]["cargo_invocations"],
        "subject_executions": decision["execution_ledger"]["subject_executions"],
        "perf_stat_invocations": decision["execution_ledger"]["perf_stat_invocations"],
        "perf_record_invocations": 0,
        "runtime_authority_changed": False,
        "remote_writes": 0,
        "scientific": success,
        "production_policy_admitted": False,
        "runtime_edit_admitted": False,
        "next_action_admitted": "paper decision only; no automatic production policy or runtime edit",
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("--stage", type=pathlib.Path, required=True)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        print(json.dumps(audit(arguments.stage.resolve()), ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D7 AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
