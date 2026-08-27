#!/usr/bin/env python3
"""Independent bootstrap audit for the M2 fused-minimum experiment."""

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
from collections.abc import Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2-v1-20260826"
TRANSACTION_ID = "c760eea52b6416b3529f9d684c315147b5a1140522114642c417d7db4065102c"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
AUDITOR = pathlib.Path(__file__).resolve()
LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-remote.py"
FRAGMENT = ROOT / "scripts/lay_v10_e1_traversal_w1_fused_minimum_m2_test_module.rs.inc"
IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "IMPLEMENTATION_SELF_CHECK_V2_2026-08-26.json"
)
EXECUTION_ADMISSION = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "EXECUTION_ADMISSION_V1_2026-08-26.json"
)
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BOOTSTRAP_AUDIT_V1_2026-08-26"
)
EXPECTED_FRAGMENT = "b0a775420edf9e9d6e7f0b59f9ad840e4822cd0fdd0adc2429ea22a3e9e3a175"


class BootstrapAuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise BootstrapAuditError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def canonical_json_bytes(value: Any) -> bytes:
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
    write_new(path, canonical_json_bytes(value))


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def run(command: Sequence[str], *, timeout: float = 3_600) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if result.returncode != 0:
        raise BootstrapAuditError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            + result.stderr.decode(errors="replace")[-5000:]
        )
    return result


def ssh(command: Sequence[str]) -> bytes:
    return run([
        "/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
        REMOTE, shlex.join(list(command)),
    ]).stdout


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def sums(root: pathlib.Path) -> None:
    rows = [
        f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n"
        for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file() and candidate.name != "SHA256SUMS")
    ]
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def fixed_inputs() -> dict[str, Any]:
    need(IMPLEMENTATION_RECEIPT.is_file(), "sealed M2 implementation receipt absent")
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(implementation.get("verdict") == "M2_CONTROLLER_VERIFIED_UNRUN", "implementation verdict drift")
    need(sha256_file(FRAGMENT) == EXPECTED_FRAGMENT, "M2 fragment drift")
    for name, path in {
        "local_controller": LOCAL_CONTROLLER,
        "remote_controller": REMOTE_CONTROLLER,
        "auditor": AUDITOR,
    }.items():
        need(path.is_file(), f"{name} absent")
        compile(path.read_text(), str(path), "exec")
    return {
        "implementation": implementation,
        "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "fragment_sha256": sha256_file(FRAGMENT),
    }


def self_check() -> dict[str, Any]:
    values = fixed_inputs()
    need(not RESULT.exists(), "bootstrap audit result already exists")
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-bootstrap-audit-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2_BOOTSTRAP_AUDITOR_VERIFIED_UNRUN",
        **{key: value for key, value in values.items() if key != "implementation"},
        "network_access": 0,
        "remote_writes": 0,
        "markers_created": 0,
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "subject_executions": 0,
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
def tree(root): return [row(p) for p in sorted(root.rglob('*')) if p.is_file()]
receipt=parent/'M2_BOOTSTRAP_RECEIPT.json'
print(json.dumps({'hostname':os.uname().nodename,'uid':os.geteuid(),'parent_mode':f'{stat.S_IMODE(parent.stat().st_mode):04o}','state_mode':f'{stat.S_IMODE(state.stat().st_mode):04o}','parent_entries':sorted(p.name for p in parent.iterdir()),'state_entries':sorted(p.name for p in state.iterdir()),'bootstrap_receipt':json.loads(receipt.read_text()),'bootstrap_receipt_row':row(receipt),'parent_tree':tree(parent),'state_tree':tree(state),'markers_exists':(state/'markers').exists(),'remote_writes':0},sort_keys=True))
'''
    raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", code, str(REMOTE_PARENT), str(REMOTE_STATE)])
    return json.loads(raw.decode().strip().splitlines()[-1])


def validate_live(value: dict[str, Any], check: dict[str, Any]) -> dict[str, Any]:
    need(value.get("hostname") == "e-MEGA-MINI-M1-13th" and value.get("uid") == 0, "remote identity drift")
    need(value.get("parent_mode") == "0555" and value.get("state_mode") == "0755", "remote root mode drift")
    need(value.get("parent_entries") == ["M2_BOOTSTRAP_RECEIPT.json", "SHA256SUMS", "bootstrap-v1", "uid-probe"], "bootstrap membership drift")
    need(value.get("state_entries") == ["STATE-00-BOOTSTRAP_CREATED_UNAUDITED.json", "route.lock"], "pre-marker state membership drift")
    need(value.get("markers_exists") is False, "M2 markers exist before bootstrap audit")
    receipt = value.get("bootstrap_receipt", {})
    need(receipt.get("verdict") == "M2_BOOTSTRAP_CREATED_UNAUDITED", "bootstrap producer verdict drift")
    need(receipt.get("task_id") == TASK_ID and receipt.get("transaction_id") == TRANSACTION_ID, "bootstrap producer namespace drift")
    need(receipt.get("uid_capability", {}).get("verdict") == "PASS", "UID capability proof absent")
    need(receipt.get("markers_created") == 0 and receipt.get("cargo_invocations") == 0 and receipt.get("perf_stat_invocations") == 0, "bootstrap execution ledger drift")
    payload = receipt.get("bootstrap", {}).get("payload", {})
    need(payload.get("files", {}).get("local-controller.py") == check["local_controller_sha256"], "bootstrap local controller drift")
    need(payload.get("files", {}).get("remote-controller.py") == check["remote_controller_sha256"], "bootstrap remote controller drift")
    return {"bootstrap_receipt_sha256": value["bootstrap_receipt_row"]["sha256"], "uid_capability": receipt["uid_capability"], "markers_created": 0, "markers_consumed": 0}


def audit() -> dict[str, Any]:
    check = self_check()
    need(EXECUTION_ADMISSION.is_file(), "execution admission absent")
    admission = json.loads(EXECUTION_ADMISSION.read_text())
    need(admission.get("verdict") == "M2_EXECUTION_ADMITTED", "execution admission verdict drift")
    before = live_projection()
    validated = validate_live(before, check)
    after = live_projection()
    need(after == before, "remote bootstrap changed during read-only audit")
    receipt = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-bootstrap-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED",
        "local_controller_sha256": check["local_controller_sha256"],
        "remote_controller_sha256": check["remote_controller_sha256"],
        "bootstrap_auditor_sha256": check["auditor_sha256"],
        "implementation_receipt_sha256": check["implementation_receipt_sha256"],
        "execution_admission_sha256": sha256_file(EXECUTION_ADMISSION),
        "producer": validated,
        "live_projection_stable": True,
        "markers_expected": 8,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "subject_executions": 0,
        "remote_writes": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "one atomic eight-marker M2 creation action only",
    }
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "M2_BOOTSTRAP_AUDIT_RECEIPT.json", receipt)
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
    return {**receipt, "receipt_sha256": sha256_file(RESULT / "M2_BOOTSTRAP_AUDIT_RECEIPT.json")}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2 BOOTSTRAP AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
