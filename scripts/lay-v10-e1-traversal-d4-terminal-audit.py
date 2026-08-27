#!/usr/bin/env python3
"""Independent read-only terminal audit for D4 single-worker estimator recovery."""

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
TASK_ID = "slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826"
TRANSACTION_ID = "2d3002b7cf615459a4250d7e44eb2094863dc422f908080b7afa59551ba4ee26"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID

BOOTSTRAP_AUDIT_DIR = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_BOOTSTRAP_AUDIT_V1_2026-08-26"
BOOTSTRAP_AUDIT = BOOTSTRAP_AUDIT_DIR / "D4_BOOTSTRAP_AUDIT_RECEIPT.json"
MARKER_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_MARKER_AUDIT_V1_2026-08-26/D4_MARKER_AUDIT_RECEIPT.json"
U3_DIR = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_U3_SINGLE_V1_2026-08-26"
T3_DIR = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_T3_SINGLE_V1_2026-08-26"
U3_RECEIPT = U3_DIR / "REMOTE_EVIDENCE/D4_ROUTE_RECEIPT.json"
T3_RECEIPT = T3_DIR / "REMOTE_EVIDENCE/D4_ROUTE_RECEIPT.json"
CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery-remote.py"
RESULT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TERMINAL_AUDIT_V1_2026-08-26"

BOOTSTRAP_AUDIT_SHA256 = "233fcea000bbdf9edd6325bccf3c5179502a8ba171763d4a566bd75bf0774695"
MARKER_AUDIT_SHA256 = "af9f8f13762d51a427d6bebee6c9c58fddd0741f52e4ceaafd78a2c5c8dcaf57"
U3_RECEIPT_SHA256 = "db2ba1b3d4e11ac2c4edb24e382f93d282b1b5d605fcbb1636f5e346030dc000"
T3_RECEIPT_SHA256 = "dd4e3b7bb49d368fe1461c36fda0968af629293e801500770ca9dc3715a96f09"
CONTROLLER_SHA256 = "2fa0ed312ece2f93e76b62a20bd7b2757f79ef9ecef06c29c184538fb7cfe504"
REMOTE_CONTROLLER_SHA256 = "f6b5010b1f4c38bedcbc0ec0cb466e177af78be4df802eb8b0ff02bdcd461f72"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
MARKERS = {
    "u3-single.consumed-before-exec": ("U3-SINGLE", "4484826fab0137fcc7e41e146891b110a666a865e7594eb008784ddd2c2154e9", 287),
    "t3-single.consumed-before-exec": ("T3-SINGLE", "ff975bfdb78ca675903a6ee123134594cd4ec88391c17be7f41098174d45ffd6", 287),
}
EXTERNAL_ACTIONS = ("self-check", "audit")


class TerminalAuditError(RuntimeError):
    pass


def require(value: bool, message: str) -> None:
    if not value:
        raise TerminalAuditError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def mode(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
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


def sums(root: pathlib.Path) -> None:
    rows = [f"{sha256_file(path)}  {path.relative_to(root)}\n" for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS")]
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def verify_sums(root: pathlib.Path) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"SHA256SUMS missing: {root}")
    listed = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file() and sha256_file(path) == digest, f"manifest drift: {path}")
        listed[relative] = digest
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"}
    require(set(listed) == actual, f"manifest membership drift: {root}")
    return {"entries": len(listed), "sha256": sha256_file(manifest)}


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def run(command: Sequence[str], timeout: int = 1800) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)


def ssh(command: Sequence[str]) -> list[str]:
    return ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", REMOTE, shlex.join(command)]


def runtime() -> dict[str, Any]:
    launcher = pathlib.Path.home() / ".local/bin/lay"
    resolved = launcher.resolve(strict=True)
    return {"launcher": str(launcher), "resolved": str(resolved), "resolved_sha256": sha256_file(resolved)}


def projection_source() -> str:
    return f'''
import hashlib,json,os,pathlib,stat
P=pathlib.Path({str(PARENT)!r}); S=pathlib.Path({str(STATE)!r})
D2=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
D2S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
D3=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826")
D3S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826")
def sha(path):
 h=hashlib.sha256()
 with path.open("rb") as f:
  for block in iter(lambda:f.read(1024*1024),b""): h.update(block)
 return h.hexdigest()
def row(path):
 s=path.stat(); return {{"name":path.name,"mode":f"{{stat.S_IMODE(s.st_mode):04o}}","uid":s.st_uid,"size_bytes":s.st_size,"sha256":sha(path),"value":json.loads(path.read_text()) if path.suffix==".json" or "." in path.name else None}}
def files(root): return [{{"path":str(path.relative_to(root)),"sha256":sha(path),"size_bytes":path.stat().st_size,"mode":f"{{stat.S_IMODE(path.stat().st_mode):04o}}"}} for path in sorted(root.rglob("*")) if path.is_file()]
active=[]
for proc in pathlib.Path("/proc").iterdir():
 if not proc.name.isdigit(): continue
 try: env=(proc/"environ").read_bytes()
 except Exception: continue
 entries=env.split(bytes([0]))
 if b"LAY_V10_D1_RUN_ID=U3-SINGLE" in entries or b"LAY_V10_D1_RUN_ID=T3-SINGLE" in entries: active.append(int(proc.name))
value={{"hostname":os.uname().nodename,"parent_entries":sorted(x.name for x in P.iterdir()),"state_entries":sorted(x.name for x in S.iterdir()),"markers":[row(path) for path in sorted((S/"markers").iterdir())],"u3_state":row(S/"U3_SINGLE_STATE.json"),"t3_state":row(S/"T3_SINGLE_STATE.json"),"u3_manifest_sha256":sha(P/"u3-single-v1/SHA256SUMS"),"t3_manifest_sha256":sha(P/"t3-single-v1/SHA256SUMS"),"u3_receipt":row(P/"u3-single-v1/D4_ROUTE_RECEIPT.json"),"t3_receipt":row(P/"t3-single-v1/D4_ROUTE_RECEIPT.json"),"d2_elf_sha256":sha(D2/"build-v1/d2-test-elf"),"d2_map_sha256":sha(D2/"bucket-map-v1/D2_BUCKET_MAP.json"),"d2_markers":files(D2S/"markers"),"d3_tree":files(D3),"d3_state_tree":files(D3S),"active_subjects":active}}
print(json.dumps(value,sort_keys=True,separators=(",",":")))
'''


def live() -> dict[str, Any]:
    result = run(ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", projection_source()]), timeout=3600)
    require(result.returncode == 0, f"remote terminal projection failed: {result.stderr.decode(errors='replace')[-4000:]}")
    return json.loads(result.stdout.decode().strip().splitlines()[-1])


def validate_route_receipts() -> dict[str, Any]:
    require(sha256_file(U3_RECEIPT) == U3_RECEIPT_SHA256 and sha256_file(T3_RECEIPT) == T3_RECEIPT_SHA256, "route receipt SHA drift")
    u3 = json.loads(U3_RECEIPT.read_text())
    t3 = json.loads(T3_RECEIPT.read_text())
    require(u3.get("verdict") == "U3_SINGLE_PASS" and t3.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS", "route verdict drift")
    require(all(not values for values in u3["dispatch"]["all_violations"].values()), "U3 dispatch violation")
    require(all(not values for values in t3["dispatch"]["all_violations"].values()), "T3 dispatch violation")
    observation = t3["observation"]
    event = observation["event_validation"]
    raw = observation["raw_records"]
    attribution = observation["attribution"]
    require(event == {**event, "sample_period": 200000, "freq": 0, "exclude_kernel": 1, "precise_ip": 0, "inherit": 1}, "event identity drift")
    require(raw["lost_records"] == 0 and raw["throttle_records"] == 0 and raw["unthrottle_records"] == 0, "lost/throttle drift")
    require(observation["host_before"]["perf_event_max_sample_rate"] == "8000" and observation["host_after"]["perf_event_max_sample_rate"] == "8000", "sample-rate drift")
    require(observation["thermal_throttle_drift"] == {}, "thermal drift")
    require(attribution["accepted_traversal_samples"] >= 50000, "sample coverage drift")
    require(attribution["unattributed_percent"] <= 5.0, "unattributed drift")
    require(attribution["sampled_vs_paired_u3_delta_percent"] <= 5.0, "paired perturbation drift")
    require(attribution["normalization_unique"] is True and attribution["map_check"]["machine_byte_mismatches"] == [], "map identity drift")
    return {"u3": u3, "t3": t3, "event": event, "raw_records": raw, "attribution": attribution}


def validate_live(value: Mapping[str, Any], local: Mapping[str, Any]) -> dict[str, Any]:
    require(value.get("hostname") == "e-MEGA-MINI-M1-13th", "host drift")
    require(value.get("parent_entries") == ["bootstrap-v1", "marker-creation-v1", "t3-single-v1", "u3-single-v1"], "terminal parent membership drift")
    require(value.get("state_entries") == ["MARKER_STATE.json", "STATE.json", "T3_SINGLE_STATE.json", "U3_SINGLE_STATE.json", "markers", "route.lock"], "terminal state membership drift")
    observed = {item["name"]: item for item in value["markers"]}
    require(set(observed) == set(MARKERS), "terminal marker membership drift")
    for name, (route, digest, size) in MARKERS.items():
        item = observed[name]
        require(item["mode"] == "0400" and item["sha256"] == digest and item["size_bytes"] == size, f"terminal marker identity drift: {name}")
        require(item["value"]["route"] == route and item["value"]["retry_permitted"] is False, f"terminal marker body drift: {name}")
    require(value["u3_state"]["value"]["state"] == "U3_SINGLE_PASS" and value["u3_state"]["value"]["receipt_sha256"] == U3_RECEIPT_SHA256, "U3 state drift")
    require(value["t3_state"]["value"]["state"] == "D4_SINGLE_ESTIMATOR_PASS" and value["t3_state"]["value"]["receipt_sha256"] == T3_RECEIPT_SHA256, "T3 state drift")
    require(value["u3_receipt"]["sha256"] == U3_RECEIPT_SHA256 and value["t3_receipt"]["sha256"] == T3_RECEIPT_SHA256, "remote route receipt drift")
    require(value["u3_manifest_sha256"] == local["u3_manifest"]["sha256"] and value["t3_manifest_sha256"] == local["t3_manifest"]["sha256"], "remote/local manifest drift")
    require(value["d2_elf_sha256"] == ELF_SHA256 and value["d2_map_sha256"] == MAP_SHA256, "D2 identity drift")
    require(value["active_subjects"] == [], "D4 subject remains active")
    baseline = json.loads((BOOTSTRAP_AUDIT_DIR / "REMOTE_BEFORE.json").read_text())
    require(value["d2_markers"] == [{"path": item["path"].split("/markers/", 1)[1], "sha256": item["sha256"], "size_bytes": item["size_bytes"], "mode": item["mode"]} for item in baseline["d2_markers"]], "D2 markers changed during D4")
    require(value["d3_tree"] == [{"path": item["path"].split("/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826/", 1)[1], "sha256": item["sha256"], "size_bytes": item["size_bytes"], "mode": item["mode"]} for item in baseline["d3_tree"]], "D3 evidence changed during D4")
    require(value["d3_state_tree"] == [{"path": item["path"].split("/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826/", 1)[1], "sha256": item["sha256"], "size_bytes": item["size_bytes"], "mode": item["mode"]} for item in baseline["d3_state_tree"]], "D3 state changed during D4")
    return {"markers_consumed": 2, "active_subjects": 0, "predecessor_drift": 0, "runtime_authority_changed": False}


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"terminal audit result exists: {RESULT}")
    for path, digest in ((BOOTSTRAP_AUDIT, BOOTSTRAP_AUDIT_SHA256), (MARKER_AUDIT, MARKER_AUDIT_SHA256), (U3_RECEIPT, U3_RECEIPT_SHA256), (T3_RECEIPT, T3_RECEIPT_SHA256), (CONTROLLER, CONTROLLER_SHA256), (REMOTE_CONTROLLER, REMOTE_CONTROLLER_SHA256)):
        require(path.is_file() and sha256_file(path) == digest, f"terminal input drift: {path}")
    local = {"bootstrap_audit": row(BOOTSTRAP_AUDIT), "marker_audit": row(MARKER_AUDIT), "u3_receipt": row(U3_RECEIPT), "t3_receipt": row(T3_RECEIPT), "controller": row(CONTROLLER), "remote_controller": row(REMOTE_CONTROLLER), "u3_manifest": verify_sums(U3_DIR / "REMOTE_EVIDENCE"), "t3_manifest": verify_sums(T3_DIR / "REMOTE_EVIDENCE")}
    return {"schema": "lay.v10.e1-traversal-d4-terminal-audit-self-check.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D4_TERMINAL_AUDITOR_VERIFIED_UNRUN", "local": local, "remote_writes": 0, "subject_executions": 0, "perf_record": 0, "perf_stat": 0}


def audit() -> dict[str, Any]:
    check = self_check()
    routes = validate_route_receipts()
    runtime_before = runtime()
    before = live()
    terminal = validate_live(before, check["local"])
    after = live()
    require(after == before, "remote projection changed during terminal audit")
    runtime_after = runtime()
    require(runtime_after == runtime_before, "installed runtime changed during terminal audit")
    attribution = routes["attribution"]
    total = sum(attribution["accepted_bucket_counts"].values())
    bucket_percent = {name: count * 100.0 / total for name, count in attribution["accepted_bucket_counts"].items()}
    receipt = {"schema": "lay.v10.e1-traversal-d4-terminal-audit.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D4_SINGLE_ESTIMATOR_PASS", "terminal_scope": "D4 single-worker task-clock estimator recovery", "bootstrap_audit_sha256": BOOTSTRAP_AUDIT_SHA256, "marker_audit_sha256": MARKER_AUDIT_SHA256, "u3_receipt_sha256": U3_RECEIPT_SHA256, "t3_receipt_sha256": T3_RECEIPT_SHA256, "controller_sha256": CONTROLLER_SHA256, "remote_controller_sha256": REMOTE_CONTROLLER_SHA256, "terminal_projection": terminal, "event_validation": routes["event"], "lost_records": routes["raw_records"]["lost_records"], "throttle_records": routes["raw_records"]["throttle_records"], "unthrottle_records": routes["raw_records"]["unthrottle_records"], "accepted_traversal_samples": attribution["accepted_traversal_samples"], "unattributed_samples": attribution["unattributed_samples"], "unattributed_percent": attribution["unattributed_percent"], "staging_traversal_samples_excluded": attribution["staging_traversal_samples_excluded"], "u3_cpu_per_edge_ns": attribution["paired_u3_cpu_per_edge_ns"], "t3_cpu_per_edge_ns": attribution["sampled_traversal_cpu_per_edge_ns"], "sampled_vs_u3_delta_percent": attribution["sampled_vs_paired_u3_delta_percent"], "accepted_bucket_counts": attribution["accepted_bucket_counts"], "accepted_bucket_percent": bucket_percent, "accepted_sub_bucket_counts": attribution["accepted_sub_bucket_counts"], "normalization_unique": attribution["normalization_unique"], "machine_byte_mismatches": len(attribution["map_check"]["machine_byte_mismatches"]), "markers_created": 2, "markers_consumed": 2, "u3_subject_executions": 1, "t3_subject_executions": 1, "perf_record": 1, "perf_readers": 4, "pmu_events_opened": 1, "cargo_invocations": 0, "rustc_compilations": 0, "runtime_before": runtime_before, "runtime_after": runtime_after, "runtime_authority_changed": False, "optimization_authority": False, "retry_permitted": False, "next_action_admitted": "paper decision only; no optimization, build, integration, install, restart or deployment"}
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "D4_TERMINAL_AUDIT_RECEIPT.json", receipt)
        write_json(stage / "SELF_CHECK.json", check)
        write_json(stage / "REMOTE_BEFORE.json", before)
        write_json(stage / "REMOTE_AFTER.json", after)
        write_json(stage / "U3_SCIENTIFIC_RECEIPT.json", routes["u3"])
        write_json(stage / "T3_SCIENTIFIC_RECEIPT.json", routes["t3"])
        write_new(stage / "terminal-auditor.py", AUDITOR.read_bytes())
        sums(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return {**receipt, "receipt_sha256": sha256_file(RESULT / "D4_TERMINAL_AUDIT_RECEIPT.json"), "audit_result": str(RESULT)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=EXTERNAL_ACTIONS)
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D4 TERMINAL AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
