#!/usr/bin/env python3
"""Independent read-only audit of D4 marker creation."""

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
BOOTSTRAP_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_BOOTSTRAP_AUDIT_V1_2026-08-26/D4_BOOTSTRAP_AUDIT_RECEIPT.json"
MARKER_CREATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_MARKER_CREATION_V1_2026-08-26/REMOTE_EVIDENCE/D4_MARKER_CREATION_RECEIPT.json"
CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery-remote.py"
RESULT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_MARKER_AUDIT_V1_2026-08-26"
MARKERS = {
    "u3-single.available": ("U3-SINGLE", "4484826fab0137fcc7e41e146891b110a666a865e7594eb008784ddd2c2154e9", 287),
    "t3-single.available": ("T3-SINGLE", "ff975bfdb78ca675903a6ee123134594cd4ec88391c17be7f41098174d45ffd6", 287),
}
EXTERNAL_ACTIONS = ("self-check", "audit")


class MarkerAuditError(RuntimeError):
    pass


def require(value: bool, message: str) -> None:
    if not value:
        raise MarkerAuditError(message)


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


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def sums(root: pathlib.Path) -> None:
    rows = [f"{sha256_file(path)}  {path.relative_to(root)}\n" for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS")]
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def run(command: Sequence[str], timeout: int = 1800) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)


def ssh(command: Sequence[str]) -> list[str]:
    return ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", REMOTE, shlex.join(command)]


def source() -> str:
    return f'''
import hashlib,json,os,pathlib,stat
P=pathlib.Path({str(PARENT)!r}); S=pathlib.Path({str(STATE)!r})
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def row(path):
 s=path.stat(); return {{"name":path.name,"mode":f"{{stat.S_IMODE(s.st_mode):04o}}","uid":s.st_uid,"size_bytes":s.st_size,"sha256":sha(path),"value":json.loads(path.read_text())}}
creation=P/"marker-creation-v1"; receipt=creation/"D4_MARKER_CREATION_RECEIPT.json"
value={{"hostname":os.uname().nodename,"parent_entries":sorted(x.name for x in P.iterdir()),"state_entries":sorted(x.name for x in S.iterdir()),"markers":[row(path) for path in sorted((S/"markers").iterdir())],"marker_state":row(S/"MARKER_STATE.json"),"creation_receipt":row(receipt),"creation_manifest_sha256":sha(creation/"SHA256SUMS"),"bootstrap_manifest_sha256":sha(P/"bootstrap-v1/SHA256SUMS")}}
print(json.dumps(value,sort_keys=True,separators=(",",":")))
'''


def live() -> dict[str, Any]:
    result = run(ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", source()]))
    require(result.returncode == 0, f"remote marker projection failed: {result.stderr.decode(errors='replace')[-4000:]}")
    return json.loads(result.stdout.decode().strip().splitlines()[-1])


def validate(value: Mapping[str, Any], bootstrap_sha: str) -> dict[str, Any]:
    require(value.get("hostname") == "e-MEGA-MINI-M1-13th", "host drift")
    require(value.get("parent_entries") == ["bootstrap-v1", "marker-creation-v1"], "parent membership drift")
    require(value.get("state_entries") == ["MARKER_STATE.json", "STATE.json", "markers", "route.lock"], "state membership drift")
    observed = {item["name"]: item for item in value.get("markers", [])}
    require(set(observed) == set(MARKERS), "marker membership drift")
    for name, (route, digest, size) in MARKERS.items():
        item = observed[name]
        require(item.get("mode") == "0400" and item.get("sha256") == digest and item.get("size_bytes") == size, f"marker identity drift: {name}")
        body = item.get("value", {})
        require(body.get("task_id") == TASK_ID and body.get("transaction_id") == TRANSACTION_ID and body.get("route") == route and body.get("retry_permitted") is False, f"marker body drift: {name}")
    state = value.get("marker_state", {}).get("value", {})
    require(state.get("state") == "D4_MARKERS_CREATED_UNAUDITED" and state.get("bootstrap_audit_sha256") == bootstrap_sha, "marker state admission drift")
    receipt = value.get("creation_receipt", {}).get("value", {})
    require(receipt.get("verdict") == "D4_MARKERS_CREATED_UNAUDITED" and receipt.get("bootstrap_audit_sha256") == bootstrap_sha, "marker receipt drift")
    require(receipt.get("markers_created") == 2 and receipt.get("markers_consumed") == 0, "marker receipt ledger drift")
    return {"markers": sorted(observed), "markers_created": 2, "markers_consumed": 0, "creation_receipt_sha256": value["creation_receipt"]["sha256"]}


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"marker audit result exists: {RESULT}")
    require(BOOTSTRAP_AUDIT.is_file() and mode(BOOTSTRAP_AUDIT) == "0444", "bootstrap audit missing")
    bootstrap = json.loads(BOOTSTRAP_AUDIT.read_text())
    require(bootstrap.get("verdict") == "D4_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "bootstrap audit verdict drift")
    require(MARKER_CREATION.is_file() and mode(MARKER_CREATION) == "0444", "marker creation mirror missing")
    compile(CONTROLLER.read_text(), str(CONTROLLER), "exec")
    compile(REMOTE_CONTROLLER.read_text(), str(REMOTE_CONTROLLER), "exec")
    return {"schema": "lay.v10.e1-traversal-d4-marker-audit-self-check.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D4_MARKER_AUDITOR_VERIFIED_UNRUN", "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT), "marker_creation_receipt_sha256": sha256_file(MARKER_CREATION), "local_controller_sha256": sha256_file(CONTROLLER), "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER), "remote_writes": 0}


def audit() -> dict[str, Any]:
    check = self_check()
    before = live()
    projection = validate(before, check["bootstrap_audit_sha256"])
    after = live()
    require(after == before, "remote marker projection changed during audit")
    receipt = {"schema": "lay.v10.e1-traversal-d4-marker-audit.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D4_MARKER_AUDIT_PASS_U3_ADMITTED", "bootstrap_audit_sha256": check["bootstrap_audit_sha256"], "marker_creation_receipt_sha256": check["marker_creation_receipt_sha256"], "local_controller_sha256": check["local_controller_sha256"], "remote_controller_sha256": check["remote_controller_sha256"], "marker_auditor_sha256": sha256_file(AUDITOR), "projection": projection, "projection_sha256": sha256_bytes(canonical(before)), "projection_stable": True, "markers_expected": 2, "markers_created": 2, "markers_consumed": 0, "subject_executions": 0, "perf_record": 0, "perf_stat": 0, "pmu_events_opened": 0, "remote_writes": 0, "runtime_authority_changed": False, "retry_permitted": False, "next_action_admitted": "U3-SINGLE only"}
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "D4_MARKER_AUDIT_RECEIPT.json", receipt)
        write_json(stage / "SELF_CHECK.json", check)
        write_json(stage / "REMOTE_BEFORE.json", before)
        write_json(stage / "REMOTE_AFTER.json", after)
        write_new(stage / "auditor.py", AUDITOR.read_bytes())
        sums(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return {**receipt, "receipt_sha256": sha256_file(RESULT / "D4_MARKER_AUDIT_RECEIPT.json"), "audit_result": str(RESULT)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=EXTERNAL_ACTIONS)
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D4 MARKER AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
