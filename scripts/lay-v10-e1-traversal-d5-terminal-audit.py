#!/usr/bin/env python3
"""Independent read-only terminal audit for D5 multiworker attribution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shlex
import shutil
import stat
import subprocess
import sys
import time
from typing import Any, Mapping, Sequence


AUDITOR = pathlib.Path(__file__).resolve()
ROOT = AUDITOR.parents[1]
TASK_ID = "slice8b-v10-e1-traversal-d5-multiworker-tid-estimator-v1-20260826"
TRANSACTION_ID = "3ee46e2c915677e1b2d3cd6bcc9709e0232252dbc120745b097d736537779036"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID

CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d5-multiworker-tid.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d5-multiworker-tid-remote.py"
BOOTSTRAP_AUDIT_DIR = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_BOOTSTRAP_AUDIT_V1_2026-08-26"
BOOTSTRAP_AUDIT = BOOTSTRAP_AUDIT_DIR / "D5_BOOTSTRAP_AUDIT_RECEIPT.json"
MARKER_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MARKER_AUDIT_V1_2026-08-26/D5_MARKER_AUDIT_RECEIPT.json"
D4_TERMINAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TERMINAL_AUDIT_V1_2026-08-26/D4_TERMINAL_AUDIT_RECEIPT.json"
RESULT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_TERMINAL_AUDIT_V1_2026-08-26"

ROUTE_ORDER = ("U4-FIXED", "T4-FIXED", "U4-REVERSED", "T4-REVERSED")
ROUTE_RESULTS = {
    route: ROOT
    / "docs/structural_gates/receipts"
    / f"LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_{route.replace('-', '_')}_V1_2026-08-26"
    for route in ROUTE_ORDER
}
ROUTE_PASS = {
    "U4-FIXED": "U4_FIXED_PASS",
    "T4-FIXED": "T4_FIXED_PASS",
    "U4-REVERSED": "U4_REVERSED_PASS",
    "T4-REVERSED": "D5_MULTIWORKER_ATTRIBUTION_PASS",
}
MARKER_SPECS = {
    "U4-FIXED": ("u4-fixed", "445b573ba817c87abc345d56bb065c27cfc38ed4f9569dbfd1e91803124fabfd", 293),
    "T4-FIXED": ("t4-fixed", "c3c3711e77121062613ec1fe252e0f44c8ad41744008dd41fc894e68ce5a3c02", 293),
    "U4-REVERSED": ("u4-reversed", "359a675a658f3998c4b5fff181eeb03db67974739471e5b8c100795b86f54c45", 296),
    "T4-REVERSED": ("t4-reversed", "3319d302aa0ed4bafdc23de8f04e5138a9352f6f22ca9934131bdc1d696a7cda", 296),
}
BLOCKED_VERDICTS = {
    "BLOCKED_PROVENANCE",
    "BLOCKED_THERMAL",
    "BLOCKED_SEMANTIC",
    "BLOCKED_CAPABILITY",
    "BLOCKED_BUCKET_MAP",
    "BLOCKED_PERTURBATION",
    "BLOCKED_SAMPLE_COVERAGE",
}
DISPATCH_PRIORITY = {
    "U": (
        ("provenance", "BLOCKED_PROVENANCE"),
        ("thermal", "BLOCKED_THERMAL"),
        ("semantic", "BLOCKED_SEMANTIC"),
    ),
    "T": (
        ("provenance", "BLOCKED_PROVENANCE"),
        ("thermal", "BLOCKED_THERMAL"),
        ("capability", "BLOCKED_CAPABILITY"),
        ("bucket_map", "BLOCKED_BUCKET_MAP"),
        ("perturbation", "BLOCKED_PERTURBATION"),
        ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
    ),
}

# Filled only after the immutable terminal route prefix exists.
BOOTSTRAP_AUDIT_SHA256 = "8d3be4ec3d7f44acf01bc9c10bff04b076eede4ac5869795b3502dee00d42049"
MARKER_AUDIT_SHA256 = "8bd945422cbc14dc8f91f440e3c1c0fd253208daea1acf825acc8cae141cd32f"
CONTROLLER_SHA256 = "6e5ea4a68d4541043ad95eda12b94e0e9efa0d21e3d4ba2c62a80565ddc626d0"
REMOTE_CONTROLLER_SHA256 = "767e2cfd907527f92bcea1db54b69a9763f29b04a8f76701940abc2336cd2714"
# Executed routes are pinned to 64 hex characters. Routes after an early
# terminal verdict are pinned to None and must remain absent.
ROUTE_RECEIPT_SHA256: dict[str, str | None] = {
    "U4-FIXED": "229b901d65516d7eb6041668d2974409a721e601a6acb03d52d66929895903e4",
    "T4-FIXED": "d337dcddcd74e95e8009e520347f6e3cf6c1319c46bf7e896862200af8f5cbbf",
    "U4-REVERSED": None,
    "T4-REVERSED": None,
}
D4_TERMINAL_SHA256 = "f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
PERIOD = 200_000
T4_EDGES = 528_060_876
EXTERNAL_ACTIONS = ("self-check", "audit")


class TerminalAuditError(RuntimeError):
    pass


def require(value: bool, message: str) -> None:
    if not value:
        raise TerminalAuditError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def mode(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing file: {path}")
    return {"path": str(path), "mode": mode(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def write_new(path: pathlib.Path, value: bytes) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        os.write(descriptor, value)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_json(path: pathlib.Path, value: Any) -> None:
    write_new(path, canonical(value))


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def sums(root: pathlib.Path) -> None:
    lines = [
        f"{sha256_file(path)}  {path.relative_to(root)}\n"
        for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS")
    ]
    write_new(root / "SHA256SUMS", "".join(lines).encode())


def verify_sums(root: pathlib.Path) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"manifest missing: {root}")
    listed = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(relative not in listed and path.is_file(), f"manifest member drift: {relative}")
        require(sha256_file(path) == digest, f"manifest hash drift: {relative}")
        listed[relative] = digest
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"}
    require(set(listed) == actual, f"manifest membership drift: {root}")
    return {"entries": len(listed), "sha256": sha256_file(manifest)}


def run(command: Sequence[str], timeout: int = 3600) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)


def ssh(command: Sequence[str]) -> list[str]:
    return ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", REMOTE, shlex.join(command)]


def projection_source() -> str:
    routes = {route: route.lower() for route in ROUTE_ORDER}
    return f'''
import hashlib,json,os,pathlib,stat
P=pathlib.Path({str(PARENT)!r}); S=pathlib.Path({str(STATE)!r})
D2=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
D2S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
D3=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826")
D3S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826")
D4=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826")
D4S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826")
ROUTES={routes!r}
def sha(path):
 h=hashlib.sha256()
 with path.open("rb") as source:
  for block in iter(lambda:source.read(1024*1024),b""): h.update(block)
 return h.hexdigest()
def item(path):
 value=path.stat(); return {{"path":str(path),"mode":f"{{stat.S_IMODE(value.st_mode):04o}}","size_bytes":value.st_size,"sha256":sha(path)}}
def body(path): return {{**item(path),"name":path.name,"value":json.loads(path.read_text())}}
def tree(root): return [item(path) for path in sorted(root.rglob("*")) if path.is_file()]
route_rows={{}}
for route,slug in ROUTES.items():
 state_path=S/f"{{route.replace('-','_')}}_STATE.json"
 result=P/f"{{slug}}-v1"; failure=P/f"{{slug}}-failure-v1"
 row={{"state_exists":state_path.exists(),"result_exists":result.exists(),"failure_exists":failure.exists()}}
 if state_path.is_file(): row["state"]=body(state_path)
 if result.is_dir():
  receipt=result/"D5_ROUTE_RECEIPT.json"
  row["receipt"]=body(receipt); row["manifest_sha256"]=sha(result/"SHA256SUMS")
 if failure.is_dir():
  row["failure_tree"]=tree(failure); row["failure_manifest_sha256"]=sha(failure/"SHA256SUMS")
 route_rows[route]=row
active=[]
route_env={{f"LAY_V10_D1_RUN_ID={{route}}".encode() for route in ROUTES}}
for proc in pathlib.Path("/proc").iterdir():
 if not proc.name.isdigit(): continue
 try: entries=set((proc/"environ").read_bytes().split(bytes([0])))
 except Exception: continue
 if entries & route_env: active.append(int(proc.name))
value={{"hostname":os.uname().nodename,"sample_rate":pathlib.Path("/proc/sys/kernel/perf_event_max_sample_rate").read_text().strip(),"parent_mode":f"{{stat.S_IMODE(P.stat().st_mode):04o}}","state_mode":f"{{stat.S_IMODE(S.stat().st_mode):04o}}","parent_entries":sorted(path.name for path in P.iterdir()),"state_entries":sorted(path.name for path in S.iterdir()),"markers":[body(path) for path in sorted((S/"markers").iterdir())],"routes":route_rows,"d2_elf_sha256":sha(D2/"build-v1/d2-test-elf"),"d2_map_sha256":sha(D2/"bucket-map-v1/D2_BUCKET_MAP.json"),"d2_markers":tree(D2S/"markers"),"d2_state_tree":tree(D2S),"d3_tree":tree(D3),"d3_state_tree":tree(D3S),"d4_tree":tree(D4),"d4_state_tree":tree(D4S),"active_subjects":sorted(active)}}
print(json.dumps(value,sort_keys=True,separators=(",",":")))
'''


def live() -> dict[str, Any]:
    result = run(ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", projection_source()]))
    require(result.returncode == 0, f"remote terminal projection failed: {result.stderr.decode(errors='replace')[-4000:]}")
    lines = result.stdout.decode().strip().splitlines()
    require(lines, "remote terminal projection empty")
    return json.loads(lines[-1])


def normalized_tree(rows: Sequence[Mapping[str, Any]], token: str) -> list[tuple[str, str, int, str]]:
    result = []
    for value in rows:
        path = str(value["path"])
        require(token in path, f"projection path lacks root token: {path}")
        relative = path.split(token, 1)[1].lstrip("/")
        result.append((relative, str(value["sha256"]), int(value["size_bytes"]), str(value["mode"])))
    return sorted(result)


def validate_dispatch(route: str, receipt: Mapping[str, Any]) -> dict[str, Any]:
    dispatch = receipt.get("dispatch")
    require(isinstance(dispatch, Mapping), f"dispatch missing: {route}")
    priority = DISPATCH_PRIORITY[route[0]]
    violations = dispatch.get("all_violations")
    require(
        isinstance(violations, Mapping)
        and set(violations) == {cause for cause, _ in priority},
        f"dispatch schema drift: {route}",
    )
    selected = None
    for rank, (cause, verdict) in enumerate(priority):
        reasons = violations[cause]
        require(
            isinstance(reasons, list)
            and all(isinstance(reason, str) and reason for reason in reasons),
            f"dispatch reason schema drift: {route}/{cause}",
        )
        if reasons and selected is None:
            selected = (rank, cause, verdict, reasons[0])
    if selected is None:
        expected_verdict = ROUTE_PASS[route]
        require(
            dispatch.get("selected_cause") is None
            and dispatch.get("selected_rank") is None
            and dispatch.get("verdict") == expected_verdict
            and receipt.get("verdict") == expected_verdict,
            f"PASS dispatch drift: {route}",
        )
        return {"blocked": False, "verdict": expected_verdict, "cause": None}
    rank, cause, verdict, reason = selected
    require(
        dispatch.get("selected_cause") == cause
        and dispatch.get("selected_rank") == rank
        and dispatch.get("verdict") == verdict
        and dispatch.get("reason") == reason
        and receipt.get("verdict") == verdict,
        f"blocked dispatch priority drift: {route}",
    )
    return {"blocked": True, "verdict": verdict, "cause": cause}


def load_routes() -> tuple[dict[str, dict[str, Any]], dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    terminal: dict[str, Any] | None = None
    saw_absent = False
    for route in ROUTE_ORDER:
        expected = ROUTE_RECEIPT_SHA256[route]
        route_root = ROUTE_RESULTS[route]
        if expected is None:
            saw_absent = True
            require(not route_root.exists(), f"unpinned future route result exists: {route}")
            continue
        require(not saw_absent, f"executed route appears after an absent route: {route}")
        require(
            isinstance(expected, str) and len(expected) == 64,
            f"route receipt SHA not pinned: {route}",
        )
        verify_sums(route_root)
        verify_sums(route_root / "REMOTE_EVIDENCE")
        receipt_path = route_root / "REMOTE_EVIDENCE/D5_ROUTE_RECEIPT.json"
        require(
            mode(receipt_path) == "0444" and sha256_file(receipt_path) == expected,
            f"route receipt identity drift: {route}",
        )
        value = json.loads(receipt_path.read_text())
        require(
            value.get("task_id") == TASK_ID
            and value.get("transaction_id") == TRANSACTION_ID
            and value.get("route") == route
            and value.get("retry_permitted") is False
            and value.get("marker", {}).get("consumed_before_effect") is True,
            f"route namespace or authority drift: {route}",
        )
        remote_evidence = route_root / "REMOTE_EVIDENCE"
        require(
            sha256_file(remote_evidence / "inputs/local-controller.py") == CONTROLLER_SHA256
            and sha256_file(remote_evidence / "inputs/remote-controller.py")
            == REMOTE_CONTROLLER_SHA256
            and value.get("bootstrap_audit_sha256") == BOOTSTRAP_AUDIT_SHA256
            and value.get("marker_audit_sha256") == MARKER_AUDIT_SHA256,
            f"route controller or admission provenance drift: {route}",
        )
        dispatch = validate_dispatch(route, value)
        result[route] = value
        if dispatch["blocked"]:
            require(terminal is None, "multiple blocked routes executed")
            terminal = {"route": route, **dispatch}
            saw_absent = True
    require(result, "no D5 scientific route receipt exists")
    executed = list(result)
    require(executed == list(ROUTE_ORDER[: len(executed)]), "executed route prefix drift")
    if terminal is None:
        require(
            executed == list(ROUTE_ORDER),
            "route prefix ended at PASS without a terminal route",
        )
        terminal = {
            "route": ROUTE_ORDER[-1],
            "blocked": False,
            "verdict": "D5_MULTIWORKER_ATTRIBUTION_PASS",
            "cause": None,
        }
    else:
        require(terminal["route"] == executed[-1], "a route executed after terminal BLOCKED verdict")
        require(terminal["verdict"] in BLOCKED_VERDICTS, "unknown terminal blocked verdict")
    return result, terminal


def validate_pass_observation(route: str, receipt: Mapping[str, Any]) -> dict[str, Any]:
    observation = receipt.get("observation")
    require(
        isinstance(observation, Mapping)
        and observation.get("route") == route
        and observation.get("complete") is True,
        f"PASS observation incomplete: {route}",
    )
    subject = observation.get("subject")
    require(isinstance(subject, Mapping), f"subject validation missing: {route}")
    checks = subject.get("checks")
    require(
        isinstance(checks, Mapping)
        and checks
        and all(value is True for value in checks.values())
        and subject.get("violations") == [],
        f"subject validation drift: {route}",
    )
    expected_mapping = "FIXED" if route.endswith("FIXED") else "REVERSED"
    require(subject.get("mapping") == expected_mapping, f"subject mapping drift: {route}")
    if route.startswith("U4-"):
        require(
            observation.get("perf_record_invocations") == 0
            and observation.get("perf_reader_invocations") == 0
            and observation.get("pmu_events_opened") == 0
            and subject.get("u4_denominator_edges") == 502_915_120,
            f"U4 execution or denominator drift: {route}",
        )
        return {
            "cpu_per_edge_ns": float(subject["traversal_thread_cpu_per_edge_ns"]),
            "denominator_edges": int(subject["u4_denominator_edges"]),
        }
    attribution = observation.get("attribution")
    event = observation.get("event_validation")
    raw = observation.get("raw_records")
    require(isinstance(attribution, Mapping) and isinstance(event, Mapping) and isinstance(raw, Mapping), f"T4 evidence missing: {route}")
    require(
        event
        == {
            "line": event["line"],
            "type": 1,
            "config": 1,
            "sample_period": PERIOD,
            "exclude_kernel": 1,
            "inherit": 1,
            "freq": 0,
            "precise_ip": 0,
        },
        f"event identity drift: {route}",
    )
    require(
        observation.get("perf_record_invocations") == 1
        and observation.get("perf_reader_invocations") == 4
        and observation.get("pmu_events_opened") == 1,
        f"T4 execution ledger drift: {route}",
    )
    require(
        raw["lost_records"] == 0
        and raw["throttle_records"] == 0
        and raw["unthrottle_records"] == 0,
        f"loss or throttle drift: {route}",
    )
    require(attribution["accepted_traversal_samples"] >= 50_000, f"sample coverage drift: {route}")
    require(attribution["unattributed_percent"] <= 5.0, f"unattributed drift: {route}")
    require(attribution["sampled_vs_paired_u4_delta_percent"] <= 5.0, f"paired perturbation drift: {route}")
    require(attribution["t4_denominator_edges"] == T4_EDGES, f"T4 denominator drift: {route}")
    tid_graph = attribution["tid_graph"]
    require(tid_graph["worker_count"] == 20 and len(tid_graph["worker_tids"]) == 20, f"worker cardinality drift: {route}")
    require(sorted(tid_graph["worker_sample_cpus"].values()) == list(range(20)), f"worker CPU closure drift: {route}")
    require(
        len(attribution["accepted_samples_by_tid"]) == 20
        and all(int(value) > 0 for value in attribution["accepted_samples_by_tid"].values()),
        f"worker accepted-sample closure drift: {route}",
    )
    require(attribution.get("normalization_unique") is True, f"normalization drift: {route}")
    require(attribution.get("dso_mismatches") == [], f"DSO mapping drift: {route}")
    require(
        attribution.get("map_check", {}).get("machine_byte_mismatches") == [],
        f"machine-byte map drift: {route}",
    )
    return {
        "cpu_per_edge_ns": float(attribution["sampled_traversal_cpu_per_edge_ns"]),
        "denominator_edges": int(attribution["t4_denominator_edges"]),
        "accepted_samples": int(attribution["accepted_traversal_samples"]),
        "unattributed_percent": float(attribution["unattributed_percent"]),
        "bucket_counts": {str(key): int(value) for key, value in attribution["accepted_bucket_counts"].items()},
        "paired_delta_percent": float(attribution["sampled_vs_paired_u4_delta_percent"]),
    }


def validate_blocked_observation(
    route: str, receipt: Mapping[str, Any], cause: str
) -> dict[str, Any]:
    observation = receipt.get("observation")
    require(
        isinstance(observation, Mapping) and observation.get("route") == route,
        f"blocked observation missing: {route}",
    )
    violations = observation.get("violations", {})
    require(violations.get(cause), f"selected blocked cause lacks a violation: {route}/{cause}")
    if cause == "provenance":
        require(observation.get("complete") is False, f"provenance block marked complete: {route}")
    elif cause == "thermal":
        require(bool(observation.get("thermal_throttle_drift")), f"thermal block lacks counter drift: {route}")
    elif cause == "semantic":
        status = observation.get("status", {})
        subject = observation.get("subject", {})
        require(
            status.get("timed_out") is True
            or status.get("returncode") != 0
            or bool(subject.get("violations")),
            f"semantic block lacks failed subject predicate: {route}",
        )
    elif cause == "capability":
        record = observation.get("record_status", {})
        readers = observation.get("reader_status", {})
        event = observation.get("event_validation", {})
        expected_event = (
            {
                "line": event.get("line"),
                "type": 1,
                "config": 1,
                "sample_period": PERIOD,
                "exclude_kernel": 1,
                "inherit": 1,
                "freq": 0,
                "precise_ip": 0,
            }
            if event
            else None
        )
        require(
            record.get("timed_out") is True
            or record.get("returncode") != 0
            or observation.get("perf_data") is None
            or not observation.get("event_validation")
            or event != expected_event
            or observation.get("sample_count") == 0
            or any(
                value.get("timed_out") is True
                or value.get("returncode") != 0
                or value.get("not_run") is True
                for value in readers.values()
            ),
            f"capability block lacks failed perf predicate: {route}",
        )
    elif cause == "bucket_map":
        attribution = observation.get("attribution", {})
        require(
            bool(attribution.get("dso_mismatches"))
            or bool(
                attribution.get("map_check", {}).get("machine_byte_mismatches")
            )
            or any(
                token in " ".join(violations[cause]).lower()
                for token in ("build id", "normalization", "map", "mapping", "dso", "machine byte")
            ),
            f"bucket-map block lacks failed map predicate: {route}",
        )
    elif cause == "perturbation":
        require(
            float(
                observation.get("attribution", {}).get(
                    "sampled_vs_paired_u4_delta_percent", -1.0
                )
            )
            > 5.0,
            f"perturbation block lacks >5% delta: {route}",
        )
    elif cause == "sample_coverage":
        raw = observation.get("raw_records", {})
        attribution = observation.get("attribution", {})
        require(
            int(raw.get("lost_records", 0)) > 0
            or int(raw.get("throttle_records", 0)) > 0
            or int(raw.get("unthrottle_records", 0)) > 0
            or int(attribution.get("accepted_traversal_samples", 0)) < 50_000
            or float(attribution.get("unattributed_percent", 100.0)) > 5.0
            or any(
                token in " ".join(violations[cause]).lower()
                for token in ("period", "event", "attribution unavailable", "worker tid")
            ),
            f"sample-coverage block lacks failed coverage predicate: {route}",
        )
    else:
        raise TerminalAuditError(f"unknown blocked cause: {route}/{cause}")
    return {
        "route": route,
        "verdict": receipt["verdict"],
        "cause": cause,
        "observation_complete": observation.get("complete"),
        "violation_count": len(violations[cause]),
    }


def validate_science(
    routes: Mapping[str, Mapping[str, Any]], terminal: Mapping[str, Any]
) -> dict[str, Any]:
    d4 = json.loads(D4_TERMINAL.read_text())
    require(d4.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS", "D4 terminal verdict drift")
    d4_counts = {str(key): int(value) for key, value in d4["accepted_bucket_counts"].items()}
    d4_total = float(d4["t3_cpu_per_edge_ns"])
    d4_u = float(d4["u3_cpu_per_edge_ns"])
    route_metrics = {
        route: validate_pass_observation(route, receipt)
        for route, receipt in routes.items()
        if receipt.get("verdict") == ROUTE_PASS[route]
    }
    route_summaries = {
        route: {
            "verdict": receipt["verdict"],
            "selected_cause": receipt["dispatch"]["selected_cause"],
            "observation_complete": receipt["observation"].get("complete"),
            "perf_record": receipt.get("perf_record"),
            "perf_readers": receipt.get("perf_readers"),
            "subject_executions": receipt.get("subject_executions"),
        }
        for route, receipt in routes.items()
    }
    if terminal["blocked"]:
        blocked_validation = validate_blocked_observation(
            terminal["route"], routes[terminal["route"]], str(terminal["cause"])
        )
        return {
            "claim_valid": False,
            "terminal": dict(terminal),
            "executed_routes": list(routes),
            "route_summaries": route_summaries,
            "passed_route_metrics": route_metrics,
            "blocked_validation": blocked_validation,
            "bucket_inflation_claim": None,
        }
    require(set(route_metrics) == set(ROUTE_ORDER), "positive D5 terminal lacks four PASS routes")
    comparisons = {}
    for mapping in ("FIXED", "REVERSED"):
        u_route = f"U4-{mapping}"
        t_route = f"T4-{mapping}"
        attribution = routes[t_route]["observation"]["attribution"]
        counts = route_metrics[t_route]["bucket_counts"]
        total = route_metrics[t_route]["cpu_per_edge_ns"]
        buckets = sorted(set(d4_counts) | set(counts))
        deltas = {
            bucket: (counts.get(bucket, 0) - d4_counts.get(bucket, 0)) * PERIOD / T4_EDGES
            for bucket in buckets
        }
        inflation = total - d4_total
        require(abs(sum(deltas.values()) - inflation) <= 1e-12, f"bucket inflation reconciliation drift: {t_route}")
        route_metrics[t_route]["bucket_percent"] = {
            key: 100.0 * value / attribution["accepted_traversal_samples"]
            for key, value in counts.items()
        }
        route_metrics[t_route]["bucket_ns_per_edge"] = {
            key: value * PERIOD / T4_EDGES for key, value in counts.items()
        }
        comparisons[mapping] = {"u4_vs_d4_u3_inflation_ns_per_edge": route_metrics[u_route]["cpu_per_edge_ns"] - d4_u, "t4_vs_d4_t3_inflation_ns_per_edge": inflation, "bucket_ns_per_edge_delta_vs_d4": deltas, "bucket_delta_sum_ns_per_edge": sum(deltas.values()), "reconciled": True}
    return {"claim_valid": True, "terminal": dict(terminal), "d4": {"u3_cpu_per_edge_ns": d4_u, "t3_cpu_per_edge_ns": d4_total, "bucket_counts": d4_counts}, "routes": route_metrics, "route_summaries": route_summaries, "comparisons": comparisons}


def validate_live(
    value: Mapping[str, Any],
    routes: Mapping[str, Mapping[str, Any]],
    terminal: Mapping[str, Any],
) -> dict[str, Any]:
    baseline = json.loads((BOOTSTRAP_AUDIT_DIR / "REMOTE_BEFORE.json").read_text())
    require(value.get("hostname") == "e-MEGA-MINI-M1-13th" and value.get("sample_rate") == "8000", "host projection drift")
    require(value.get("parent_mode") == "0755" and value.get("state_mode") == "0755", "D5 parent mode drift")
    executed = list(routes)
    expected_parent = sorted(
        ["bootstrap-v1", "marker-creation-v1"]
        + [f"{route.lower()}-v1" for route in executed]
    )
    expected_state = sorted(
        ["MARKER_STATE.json", "STATE.json", "markers", "route.lock"]
        + [f"{route.replace('-', '_')}_STATE.json" for route in executed]
    )
    require(value.get("parent_entries") == expected_parent, "terminal parent membership drift")
    require(value.get("state_entries") == expected_state, "terminal state membership drift")
    observed = {item["name"]: item for item in value["markers"]}
    expected_markers = {}
    for index, route in enumerate(ROUTE_ORDER):
        marker, digest, size = MARKER_SPECS[route]
        suffix = "consumed-before-exec" if index < len(executed) else "available"
        expected_markers[f"{marker}.{suffix}"] = (route, digest, size)
    require(set(observed) == set(expected_markers), "terminal marker membership drift")
    for name, (route, digest, size) in expected_markers.items():
        item = observed[name]
        require(item["mode"] == "0400" and item["sha256"] == digest and item["size_bytes"] == size, f"terminal marker identity drift: {name}")
        require(
            item["value"].get("task_id") == TASK_ID
            and item["value"].get("transaction_id") == TRANSACTION_ID
            and item["value"].get("route") == route
            and item["value"].get("one_shot") is True
            and item["value"].get("retry_permitted") is False,
            f"terminal marker body drift: {name}",
        )
    for route in executed:
        remote = value["routes"][route]
        require(
            remote.get("state_exists") is True
            and remote.get("result_exists") is True
            and remote.get("failure_exists") is False,
            f"executed remote route projection drift: {route}",
        )
        require(remote["receipt"]["sha256"] == ROUTE_RECEIPT_SHA256[route], f"remote receipt drift: {route}")
        require(remote["state"]["value"]["state"] == routes[route]["verdict"], f"remote state drift: {route}")
        require(remote["state"]["value"]["receipt_sha256"] == ROUTE_RECEIPT_SHA256[route], f"remote state receipt drift: {route}")
        require(remote["receipt"]["value"]["verdict"] == routes[route]["verdict"], f"remote/local verdict drift: {route}")
    for route in ROUTE_ORDER[len(executed) :]:
        remote = value["routes"][route]
        require(
            remote.get("state_exists") is False
            and remote.get("result_exists") is False
            and remote.get("failure_exists") is False,
            f"future route artifact exists after terminal verdict: {route}",
        )
    require(value["d2_elf_sha256"] == ELF_SHA256 and value["d2_map_sha256"] == MAP_SHA256, "D2 identity drift")
    for key, token in (("d2_markers", "/markers/"), ("d2_state_tree", "/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825/"), ("d3_tree", "/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826/"), ("d3_state_tree", "/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826/"), ("d4_tree", "/slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826/"), ("d4_state_tree", "/slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826/")):
        require(normalized_tree(value[key], token) == normalized_tree(baseline[key], token), f"predecessor drift: {key}")
    require(value.get("active_subjects") == [], "D5 subject or perf process still active")
    require(routes[executed[-1]]["verdict"] == terminal["verdict"], "terminal route verdict drift")
    return {"markers_created": 4, "markers_consumed": len(executed), "markers_available": 4 - len(executed), "executed_routes": executed, "terminal_route": terminal["route"], "terminal_verdict": terminal["verdict"], "active_subjects": 0, "predecessor_drift": 0, "runtime_authority_changed": False}


def exact_ledger_sum(routes: Mapping[str, Mapping[str, Any]], key: str) -> int | None:
    values = [route.get(key) for route in routes.values()]
    if any(not isinstance(value, int) or isinstance(value, bool) for value in values):
        return None
    return sum(values)


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"terminal audit result exists: {RESULT}")
    fixed = ((BOOTSTRAP_AUDIT, BOOTSTRAP_AUDIT_SHA256), (MARKER_AUDIT, MARKER_AUDIT_SHA256), (CONTROLLER, CONTROLLER_SHA256), (REMOTE_CONTROLLER, REMOTE_CONTROLLER_SHA256), (D4_TERMINAL, D4_TERMINAL_SHA256))
    for path, digest in fixed:
        require(len(digest) == 64 and path.is_file() and sha256_file(path) == digest, f"pinned input drift: {path}")
    verify_sums(BOOTSTRAP_AUDIT_DIR)
    verify_sums(MARKER_AUDIT.parent)
    bootstrap = json.loads(BOOTSTRAP_AUDIT.read_text())
    marker_audit = json.loads(MARKER_AUDIT.read_text())
    require(
        bootstrap.get("verdict") == "D5_UID_ACCESS_AUDIT_PASS_MARKER_CREATION"
        and bootstrap.get("local_controller_sha256") == CONTROLLER_SHA256
        and bootstrap.get("remote_controller_sha256") == REMOTE_CONTROLLER_SHA256,
        "bootstrap audit provenance drift",
    )
    require(
        marker_audit.get("verdict") == "D5_MARKER_AUDIT_PASS_U4_FIXED_ADMITTED"
        and marker_audit.get("bootstrap_audit_sha256") == BOOTSTRAP_AUDIT_SHA256
        and marker_audit.get("local_controller_sha256") == CONTROLLER_SHA256
        and marker_audit.get("remote_controller_sha256") == REMOTE_CONTROLLER_SHA256,
        "marker audit provenance drift",
    )
    saw_absent = False
    pinned_routes = []
    for route in ROUTE_ORDER:
        digest = ROUTE_RECEIPT_SHA256[route]
        if digest is None:
            saw_absent = True
            continue
        require(not saw_absent and len(digest) == 64, f"route receipt SHA pin sequence drift: {route}")
        pinned_routes.append(route)
    require(pinned_routes, "no terminal route receipt SHA pinned")
    compile(CONTROLLER.read_text(), str(CONTROLLER), "exec")
    compile(REMOTE_CONTROLLER.read_text(), str(REMOTE_CONTROLLER), "exec")
    return {"schema": "lay.v10.e1-traversal-d5-terminal-audit-self-check.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D5_TERMINAL_AUDITOR_VERIFIED_UNRUN", "auditor": row(AUDITOR), "controller_sha256": CONTROLLER_SHA256, "remote_controller_sha256": REMOTE_CONTROLLER_SHA256, "pinned_routes": pinned_routes, "remote_writes": 0}


def audit() -> dict[str, Any]:
    check = self_check()
    routes, terminal = load_routes()
    science = validate_science(routes, terminal)
    before = live()
    live_terminal = validate_live(before, routes, terminal)
    after = live()
    require(after == before, "remote projection changed during terminal audit")
    receipt = {"schema": "lay.v10.e1-traversal-d5-terminal-audit.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": terminal["verdict"], "terminal_scope": "D5 fixed/reversed twenty-worker TID estimator terminal closure", "terminal_route": terminal["route"], "terminal_cause": terminal["cause"], "bootstrap_audit_sha256": BOOTSTRAP_AUDIT_SHA256, "marker_audit_sha256": MARKER_AUDIT_SHA256, "controller_sha256": CONTROLLER_SHA256, "remote_controller_sha256": REMOTE_CONTROLLER_SHA256, "route_receipt_sha256": ROUTE_RECEIPT_SHA256, "terminal_projection": live_terminal, "science": science, "claim_valid": not terminal["blocked"], "markers_created": 4, "markers_consumed": len(routes), "markers_available": 4 - len(routes), "subject_executions": exact_ledger_sum(routes, "subject_executions"), "perf_record": exact_ledger_sum(routes, "perf_record"), "perf_readers": exact_ledger_sum(routes, "perf_readers"), "pmu_events_opened": exact_ledger_sum(routes, "pmu_events_opened"), "perf_stat": 0, "cargo_invocations": 0, "rustc_compilations": 0, "runtime_authority_changed": False, "optimization_authority": False, "retry_permitted": False, "next_action_admitted": ("separate paper optimization decision only; no rewrite, build, integration, install, restart or deployment" if not terminal["blocked"] else "none; terminal D5 blocked verdict and no route retry")}
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "D5_TERMINAL_AUDIT_RECEIPT.json", receipt)
        write_json(stage / "SELF_CHECK.json", check)
        write_json(stage / "REMOTE_BEFORE.json", before)
        write_json(stage / "REMOTE_AFTER.json", after)
        write_new(stage / "terminal-auditor.py", AUDITOR.read_bytes())
        for route in routes:
            write_json(stage / f"{route.replace('-', '_')}_SCIENTIFIC_RECEIPT.json", routes[route])
        sums(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return {**receipt, "receipt_sha256": sha256_file(RESULT / "D5_TERMINAL_AUDIT_RECEIPT.json"), "audit_result": str(RESULT)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=EXTERNAL_ACTIONS)
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D5 TERMINAL AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
