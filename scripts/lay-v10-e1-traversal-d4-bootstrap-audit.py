#!/usr/bin/env python3
"""Independent read-only UID/scp audit before D4 scientific markers exist."""

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
PROJECT_ROOT = AUDITOR.parents[1]
TASK_ID = "slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826"
TRANSACTION_ID = "2d3002b7cf615459a4250d7e44eb2094863dc422f908080b7afa59551ba4ee26"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID

LOCAL_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery.py"
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery-remote.py"
PAPER = PROJECT_ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TASK_CLOCK_ESTIMATOR_RECOVERY_V1_2026-08-26.md"
PREFLIGHT = PROJECT_ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_2026-08-26.json"
PREFLIGHT_RECEIPT = PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_PREFLIGHT_2026-08-26.json"
D3_TERMINAL = PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_TERMINAL_AUDIT_V1_2026-08-26/D3_TERMINAL_AUDIT_RECEIPT.json"
RESULT = PROJECT_ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_BOOTSTRAP_AUDIT_V1_2026-08-26"

PAPER_SHA256 = "50b5cde0514d356375464f8155905201c395112c88a8d5022c70913b5da4d7b3"
PREFLIGHT_SHA256 = "109d284d0031df29bd1747571690fc410fba72381fe73d4adf647e550ce86a78"
PREFLIGHT_RECEIPT_SHA256 = "21dc062bf9996bd793d9392e46fc747dc89b8c5878275d257e578e0dc501c282"
D3_TERMINAL_SHA256 = "7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
FIXED_PROBE = b"D4 UID E CAPABILITY PROBE V1\n"
EXTERNAL_ACTIONS = ("self-check", "audit")


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing file: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(value)
            output.flush()
            os.fsync(output.fileno())
    finally:
        os.close(descriptor)
    os.chmod(path, mode)


def write_new_json(path: pathlib.Path, value: Any) -> None:
    write_new_bytes(path, canonical_json_bytes(value))


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_sha256sums(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS"):
        rows.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode())


def verify_sha256sums(root: pathlib.Path) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"manifest missing: {root}")
    listed: dict[str, str] = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(relative not in listed and path.is_file(), f"manifest member drift: {relative}")
        require(sha256_file(path) == digest, f"manifest hash drift: {relative}")
        listed[relative] = digest
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"}
    require(set(listed) == actual, "manifest membership drift")
    return {"entries": len(listed), "sha256": sha256_file(manifest), "members": sorted(listed)}


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def run(command: Sequence[str], *, timeout: int = 1800) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)


def ssh(command: Sequence[str]) -> list[str]:
    return ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", REMOTE, shlex.join(command)]


def projection_source() -> str:
    return f'''
import hashlib,json,os,pathlib,stat
P=pathlib.Path({str(REMOTE_PARENT)!r}); S=pathlib.Path({str(REMOTE_STATE)!r})
D2=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
D2S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
D3=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826")
D3S=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826")
def sha(path):
 h=hashlib.sha256()
 with path.open("rb") as f:
  for block in iter(lambda:f.read(1024*1024),b""): h.update(block)
 return h.hexdigest()
def item(path):
 s=path.stat(); return {{"path":str(path),"mode":f"{{stat.S_IMODE(s.st_mode):04o}}","uid":s.st_uid,"gid":s.st_gid,"size_bytes":s.st_size,"sha256":sha(path) if path.is_file() else None}}
def tree(root): return [item(path) for path in sorted(root.rglob("*")) if path.is_file()]
B=P/"bootstrap-v1"; subject=B/"subject"
value={{"hostname":os.uname().nodename,"uid":os.getuid(),"parent":item(P),"state":item(S),"parent_entries":sorted(x.name for x in P.iterdir()),"state_entries":sorted(x.name for x in S.iterdir()),"bootstrap_tree":tree(B),"subject":item(subject),"uid_probe":json.loads((subject/"UID_PROBE.json").read_text()),"uid_probe_bytes":item(subject/"UID_PROBE_BYTES.bin"),"bootstrap_receipt":json.loads((B/"D4_BOOTSTRAP_RECEIPT.json").read_text()),"d2_elf":item(D2/"build-v1/d2-test-elf"),"d2_map":item(D2/"bucket-map-v1/D2_BUCKET_MAP.json"),"d2_markers":tree(D2S/"markers"),"d3_tree":tree(D3),"d3_state_tree":tree(D3S)}}
print(json.dumps(value,sort_keys=True,separators=(",",":")))
'''


def live_projection() -> dict[str, Any]:
    result = run(ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", projection_source()]))
    require(result.returncode == 0, f"remote projection failed: {result.stderr.decode(errors='replace')[-4000:]}")
    lines = result.stdout.decode().strip().splitlines()
    require(lines, "remote projection empty")
    return json.loads(lines[-1])


def copy_bootstrap(destination: pathlib.Path) -> dict[str, Any]:
    result = run(["/usr/bin/scp", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", "-q", "-p", "-r", f"{REMOTE}:{REMOTE_PARENT}/bootstrap-v1", str(destination)], timeout=3600)
    require(result.returncode == 0, f"scp as e failed: {result.stderr.decode(errors='replace')[-4000:]}")
    return verify_sha256sums(destination)


def validate(value: Mapping[str, Any]) -> dict[str, Any]:
    require(value.get("hostname") == REMOTE_HOSTNAME and value.get("uid") == 0, "remote audit identity drift")
    require(value.get("parent", {}).get("mode") == "0755" and value.get("parent", {}).get("uid") == 0, "D4 parent access drift")
    require(value.get("state", {}).get("mode") == "0755" and value.get("state", {}).get("uid") == 0, "D4 state access drift")
    require(value.get("parent_entries") == ["bootstrap-v1"], "pre-marker parent membership drift")
    require(value.get("state_entries") == ["STATE.json", "route.lock"], "scientific marker exists before audit")
    require(value.get("subject", {}).get("mode") == "0555" and value.get("subject", {}).get("uid") == 1000, "subject seal drift")
    probe = value.get("uid_probe", {})
    require(probe.get("verdict") == "D4_UID_E_CAPABILITY_PROOF_PASS" and probe.get("uid") == 1000, "UID proof drift")
    require(probe.get("elf_sha256") == ELF_SHA256 and probe.get("map_sha256") == MAP_SHA256, "UID input read drift")
    require(value.get("uid_probe_bytes", {}).get("sha256") == sha256_bytes(FIXED_PROBE), "UID probe byte drift")
    receipt = value.get("bootstrap_receipt", {})
    require(receipt.get("verdict") == "D4_UID_PROOF_CREATED_UNAUDITED", "bootstrap verdict drift")
    require(receipt.get("markers_created") == 0 and receipt.get("markers_consumed") == 0, "bootstrap marker ledger drift")
    require(value.get("d2_elf", {}).get("sha256") == ELF_SHA256 and value.get("d2_map", {}).get("sha256") == MAP_SHA256, "D2 identity drift")
    return {"uid_operations": probe.get("operations"), "markers_created": 0, "markers_consumed": 0, "parent_mode": "0755", "subject_uid": 1000}


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"audit result exists: {RESULT}")
    for path, digest, mode in ((PAPER, PAPER_SHA256, "0444"), (PREFLIGHT, PREFLIGHT_SHA256, "0444"), (PREFLIGHT_RECEIPT, PREFLIGHT_RECEIPT_SHA256, "0444"), (D3_TERMINAL, D3_TERMINAL_SHA256, "0444")):
        require(path.is_file() and mode_string(path) == mode and sha256_file(path) == digest, f"local input drift: {path}")
    require(SSH_IDENTITY.is_file() and mode_string(SSH_IDENTITY) == "0600", "SSH identity drift")
    compile(LOCAL_CONTROLLER.read_text(), str(LOCAL_CONTROLLER), "exec")
    compile(REMOTE_CONTROLLER.read_text(), str(REMOTE_CONTROLLER), "exec")
    return {"schema": "lay.v10.e1-traversal-d4-bootstrap-audit-self-check.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D4_BOOTSTRAP_AUDITOR_VERIFIED_UNRUN", "auditor": row(AUDITOR), "local_controller_sha256": sha256_file(LOCAL_CONTROLLER), "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER), "remote_writes": 0, "markers_created": 0, "subject_executions": 0, "perf_record": 0, "perf_stat": 0}


def audit() -> dict[str, Any]:
    check = self_check()
    before = live_projection()
    live = validate(before)
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        copied = copy_bootstrap(stage / "REMOTE_BOOTSTRAP")
        copied_receipt = json.loads((stage / "REMOTE_BOOTSTRAP/D4_BOOTSTRAP_RECEIPT.json").read_text())
        require(copied_receipt == before["bootstrap_receipt"], "scp receipt differs from live projection")
        require((stage / "REMOTE_BOOTSTRAP/subject/UID_PROBE_BYTES.bin").read_bytes() == FIXED_PROBE, "scp UID bytes drift")
        after = live_projection()
        require(after == before, "remote projection changed during audit")
        receipt_sha = next(item["sha256"] for item in before["bootstrap_tree"] if item["path"].endswith("/D4_BOOTSTRAP_RECEIPT.json"))
        receipt = {"schema": "lay.v10.e1-traversal-d4-bootstrap-audit.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "D4_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "local_controller_sha256": check["local_controller_sha256"], "remote_controller_sha256": check["remote_controller_sha256"], "bootstrap_auditor_sha256": sha256_file(AUDITOR), "bootstrap_receipt_sha256": receipt_sha, "live_projection": live, "live_projection_sha256": sha256_bytes(canonical_json_bytes(before)), "live_projection_stable": True, "scp_as_e_pass": True, "scp_manifest": copied, "scp_receipt_byte_identical": True, "markers_expected": 2, "markers_created": 0, "markers_consumed": 0, "cargo_invocations": 0, "rustc_compilations": 0, "perf_record": 0, "perf_stat": 0, "pmu_events_opened": 0, "subject_executions": 0, "remote_writes": 0, "runtime_authority_changed": False, "retry_permitted": False, "next_action_admitted": "one D4 marker creation action only"}
        write_new_json(stage / "D4_BOOTSTRAP_AUDIT_RECEIPT.json", receipt)
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "REMOTE_BEFORE.json", before)
        write_new_json(stage / "REMOTE_AFTER.json", after)
        write_new_bytes(stage / "auditor.py", AUDITOR.read_bytes())
        write_new_bytes(stage / "local-controller.py", LOCAL_CONTROLLER.read_bytes())
        write_new_bytes(stage / "remote-controller.py", REMOTE_CONTROLLER.read_bytes())
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, RESULT)
        fsync_directory(RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return {**receipt, "receipt_sha256": sha256_file(RESULT / "D4_BOOTSTRAP_AUDIT_RECEIPT.json"), "audit_result": str(RESULT)}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=EXTERNAL_ACTIONS)
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D4 BOOTSTRAP AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
