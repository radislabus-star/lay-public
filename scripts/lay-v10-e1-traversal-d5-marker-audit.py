#!/usr/bin/env python3
"""Independent read-only audit of D5 marker creation."""

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
BOOTSTRAP_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_BOOTSTRAP_AUDIT_V1_2026-08-26/D5_BOOTSTRAP_AUDIT_RECEIPT.json"
MARKER_CREATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MARKER_CREATION_V1_2026-08-26/REMOTE_EVIDENCE/D5_MARKER_CREATION_RECEIPT.json"
CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d5-multiworker-tid.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-d5-multiworker-tid-remote.py"
RESULT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MARKER_AUDIT_V1_2026-08-26"
MARKERS = {
    "u4-fixed.available": ("U4-FIXED", "445b573ba817c87abc345d56bb065c27cfc38ed4f9569dbfd1e91803124fabfd", 293),
    "t4-fixed.available": ("T4-FIXED", "c3c3711e77121062613ec1fe252e0f44c8ad41744008dd41fc894e68ce5a3c02", 293),
    "u4-reversed.available": ("U4-REVERSED", "359a675a658f3998c4b5fff181eeb03db67974739471e5b8c100795b86f54c45", 296),
    "t4-reversed.available": ("T4-REVERSED", "3319d302aa0ed4bafdc23de8f04e5138a9352f6f22ca9934131bdc1d696a7cda", 296),
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
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
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


def verify_sums(root: pathlib.Path) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file() and not manifest.is_symlink(), f"manifest missing: {root}")
    listed = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(relative not in listed and path.is_file(), f"manifest member drift: {relative}")
        require(sha256_file(path) == digest, f"manifest hash drift: {relative}")
        listed[relative] = digest
    actual = {
        str(path.relative_to(root))
        for path in root.rglob("*")
        if path.is_file() and path.name != "SHA256SUMS"
    }
    require(set(listed) == actual, f"manifest membership drift: {root}")
    return {"entries": len(listed), "sha256": sha256_file(manifest)}


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
creation=P/"marker-creation-v1"; receipt=creation/"D5_MARKER_CREATION_RECEIPT.json"
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
    require(
        state.get("task_id") == TASK_ID
        and state.get("transaction_id") == TRANSACTION_ID
        and state.get("state") == "D5_MARKERS_CREATED_UNAUDITED"
        and state.get("bootstrap_audit_sha256") == bootstrap_sha
        and state.get("markers_created") == 4
        and state.get("markers_consumed") == 0
        and state.get("retry_permitted") is False,
        "marker state admission drift",
    )
    receipt = value.get("creation_receipt", {}).get("value", {})
    require(
        receipt.get("task_id") == TASK_ID
        and receipt.get("transaction_id") == TRANSACTION_ID
        and receipt.get("verdict") == "D5_MARKERS_CREATED_UNAUDITED"
        and receipt.get("bootstrap_audit_sha256") == bootstrap_sha,
        "marker receipt drift",
    )
    require(receipt.get("markers_expected") == 4 and receipt.get("markers_created") == 4 and receipt.get("markers_consumed") == 0, "marker receipt ledger drift")
    receipt_markers = {item["name"]: item for item in receipt.get("markers", [])}
    require(set(receipt_markers) == set(MARKERS), "marker receipt membership drift")
    for name, (_, digest, size) in MARKERS.items():
        item = receipt_markers[name]
        require(
            item.get("mode") == "0400"
            and item.get("sha256") == digest
            and item.get("size_bytes") == size,
            f"marker receipt identity drift: {name}",
        )
    require(
        value.get("creation_manifest_sha256")
        == sha256_file(MARKER_CREATION.parent / "SHA256SUMS"),
        "remote/local marker creation manifest drift",
    )
    return {"markers": sorted(observed), "markers_created": 4, "markers_consumed": 0, "creation_receipt_sha256": value["creation_receipt"]["sha256"]}


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"marker audit result exists: {RESULT}")
    require(BOOTSTRAP_AUDIT.is_file() and mode(BOOTSTRAP_AUDIT) == "0444", "bootstrap audit missing")
    bootstrap = json.loads(BOOTSTRAP_AUDIT.read_text())
    require(bootstrap.get("verdict") == "D5_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "bootstrap audit verdict drift")
    require(
        bootstrap.get("task_id") == TASK_ID
        and bootstrap.get("transaction_id") == TRANSACTION_ID
        and bootstrap.get("local_controller_sha256") == sha256_file(CONTROLLER)
        and bootstrap.get("remote_controller_sha256") == sha256_file(REMOTE_CONTROLLER),
        "bootstrap audit controller provenance drift",
    )
    require(MARKER_CREATION.is_file() and mode(MARKER_CREATION) == "0444", "marker creation mirror missing")
    marker_manifest = verify_sums(MARKER_CREATION.parent)
    compile(CONTROLLER.read_text(), str(CONTROLLER), "exec")
    compile(REMOTE_CONTROLLER.read_text(), str(REMOTE_CONTROLLER), "exec")
    return {"schema": "lay.v10.e1-traversal-d5-marker-audit-self-check.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D5_MARKER_AUDITOR_VERIFIED_UNRUN", "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT), "marker_creation_receipt_sha256": sha256_file(MARKER_CREATION), "marker_creation_manifest": marker_manifest, "local_controller_sha256": sha256_file(CONTROLLER), "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER), "remote_writes": 0}


def audit() -> dict[str, Any]:
    check = self_check()
    before = live()
    projection = validate(before, check["bootstrap_audit_sha256"])
    after = live()
    require(after == before, "remote marker projection changed during audit")
    receipt = {"schema": "lay.v10.e1-traversal-d5-marker-audit.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D5_MARKER_AUDIT_PASS_U4_FIXED_ADMITTED", "bootstrap_audit_sha256": check["bootstrap_audit_sha256"], "marker_creation_receipt_sha256": check["marker_creation_receipt_sha256"], "local_controller_sha256": check["local_controller_sha256"], "remote_controller_sha256": check["remote_controller_sha256"], "marker_auditor_sha256": sha256_file(AUDITOR), "projection": projection, "projection_sha256": sha256_bytes(canonical(before)), "projection_stable": True, "markers_expected": 4, "markers_created": 4, "markers_consumed": 0, "subject_executions": 0, "perf_record": 0, "perf_stat": 0, "pmu_events_opened": 0, "remote_writes": 0, "runtime_authority_changed": False, "retry_permitted": False, "next_action_admitted": "U4-FIXED only"}
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "D5_MARKER_AUDIT_RECEIPT.json", receipt)
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
    return {**receipt, "receipt_sha256": sha256_file(RESULT / "D5_MARKER_AUDIT_RECEIPT.json"), "audit_result": str(RESULT)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=EXTERNAL_ACTIONS)
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D5 MARKER AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
