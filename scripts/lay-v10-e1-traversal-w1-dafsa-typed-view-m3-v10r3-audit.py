#!/usr/bin/env python3
"""Independent auditor for the V10R3 sealed-ELF TRACE-only recovery."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r3-20260827"
TRANSACTION_ID = "8fa454ad944ce8ba9e6256bc55bdf8ff8231cf26bac7eaab591ad8564515436c"
OLD_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r2-20260827"
OLD_TRANSACTION_ID = "fe5b4741c5d5711b48f356569f3be32a87142edd78979edf0aeb72a9616de7e6"

REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v10r3-{TRANSACTION_ID}"
OLD_REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / OLD_TASK_ID
OLD_REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / OLD_TASK_ID
OLD_REMOTE_BOOTSTRAP = OLD_REMOTE_PARENT / "bootstrap-v1"
OLD_REMOTE_ELF = OLD_REMOTE_PARENT / "build-v1/v10-test-elf"

CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r3.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r3-remote.py"
AUDITOR = pathlib.Path(__file__).resolve()
LEGACY_AUDITOR_PATH = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r2-audit.py"
ADMISSION_SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"

PREFLIGHT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_TRACE_REUSE_CONTROLLER_V1_PREFLIGHT_2026-08-27.json"
IMPLEMENTATION_ROOT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_TRACE_REUSE_CONTROLLER_V1_2026-08-27"
IMPLEMENTATION_RECEIPT = IMPLEMENTATION_ROOT / "IMPLEMENTATION_RECEIPT.json"
PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_SEALED_ELF_QUIET_RECOVERY_V1_2026-08-27.md"
STRUCTURAL_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_SEALED_ELF_QUIET_RECOVERY_V1_ROUTE_RECEIPT_2026-08-27.json"

OLD_JOURNAL_MANIFEST = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R2_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
OLD_BUILD_ROOT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R2_BUILD_AUDIT_V1_2026-08-27"
OLD_BUILD_RECEIPT = OLD_BUILD_ROOT / "BUILD_AUDIT.json"
OLD_BUILD_MANIFEST = OLD_BUILD_ROOT / "SHA256SUMS"
OLD_LOCAL_ELF = OLD_BUILD_ROOT / "REMOTE_BUILD/v10-test-elf"
OLD_QUIET_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R2_QUIET_FAILURE_DIAGNOSIS_2026-08-27.json"

RECEIPTS = ROOT / "docs/structural_gates/receipts"
ADMISSION_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_EXECUTION_ADMISSION_V1_2026-08-27"
BOOTSTRAP_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_BOOTSTRAP_REUSE_AUDIT_V1_2026-08-27"
QUIET_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_QUIET_READINESS_V1_2026-08-27"
TERMINAL_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_TERMINAL_AUDIT_V1_2026-08-27"
ADMISSION_RECEIPT = ADMISSION_ROOT / "EXECUTION_ADMISSION.json"
BOOTSTRAP_RECEIPT = BOOTSTRAP_ROOT / "BOOTSTRAP_AUDIT.json"
QUIET_RECEIPT = QUIET_ROOT / "QUIET_ADMISSION.json"
TERMINAL_RECEIPT = TERMINAL_ROOT / "TERMINAL_AUDIT.json"

ACTIONS = ("self-check", "live-admission", "bootstrap", "quiet", "terminal", "status")
EXPECTED_ELF_SIZE = 320_986_144
EXPECTED_ELF_SHA256 = "0378514225ccec3cadbcfedd21ec77db66518a5eb6789f9acd83525ccf009696"
EXPECTED_ELF_BUILD_ID = "9e2e7c1fef9272f87c14876d7194609df6ac948d"
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)


class V10R3AuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V10R3AuditError(message)


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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


def file_row(path: pathlib.Path) -> dict[str, Any]:
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def load_json(path: pathlib.Path) -> Any:
    value = json.loads(path.read_text())
    need(isinstance(value, dict), f"JSON object required: {path}")
    return value


def write_new(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb") as target:
            target.write(value)
            target.flush()
            os.fsync(target.fileno())
    except BaseException:
        with contextlib.suppress(FileNotFoundError):
            path.unlink()
        raise


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode(), 0o444)


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"manifest absent: {root}")
    count = 0
    for raw in manifest.read_text().splitlines():
        digest, relative = raw.split("  ", 1)
        path = root / relative
        need(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
        count += 1
    return count


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o444 if path.is_file() else 0o555)
    root.chmod(0o555)


def publish_tree(
    destination: pathlib.Path,
    receipt_name: str,
    receipt: Mapping[str, Any],
    copied: Mapping[str, pathlib.Path] | None = None,
) -> dict[str, Any]:
    need(not destination.exists(), f"receipt tree already exists: {destination}")
    stage = destination.with_name(f"{destination.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / receipt_name, canonical(receipt), 0o444)
        for name, source in (copied or {}).items():
            target = stage / name
            if source.is_dir():
                shutil.copytree(source, target)
            else:
                shutil.copy2(source, target)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, destination)
        fsync_dir(destination.parent)
        return load_json(destination / receipt_name)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise


def load_sealed_receipt(root: pathlib.Path, receipt: pathlib.Path, verdicts: Sequence[str]) -> dict[str, Any]:
    need(root.is_dir() and mode_string(root) == "0555", f"sealed receipt root absent: {root}")
    verify_manifest(root)
    need(receipt.is_file() and mode_string(receipt) == "0444", f"sealed receipt absent: {receipt}")
    value = load_json(receipt)
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "receipt namespace drift")
    need(value.get("verdict") in set(verdicts), f"receipt verdict drift: {receipt}")
    return value


def run(argv: Sequence[str], *, timeout: float = 120, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(argv), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if check and result.returncode != 0:
        raise V10R3AuditError(
            f"command failed rc={result.returncode}: {list(argv)!r}; "
            f"stdout={result.stdout[-4096:]!r}; stderr={result.stderr[-4096:]!r}"
        )
    return result


def ssh_python(program: str, arguments: Sequence[str] = (), *, root: bool = True, timeout: float = 3600) -> dict[str, Any]:
    remote_argv = ["/usr/bin/python3", "-c", program, *arguments]
    if root:
        remote_argv = ["/usr/bin/sudo", "-n", *remote_argv]
    result = run([
        "/usr/bin/ssh", "-i", str(SSH_KEY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
        REMOTE, *remote_argv,
    ], timeout=timeout)
    lines = result.stdout.decode().strip().splitlines()
    need(lines, "remote observer returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote observer response is not an object")
    return value


def copy_remote(remote: pathlib.PurePosixPath, destination: pathlib.Path) -> None:
    result = run([
        "/usr/bin/scp", "-i", str(SSH_KEY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
        "-r", f"{REMOTE}:{remote}", str(destination),
    ], timeout=3600, check=False)
    need(result.returncode == 0, f"remote evidence copy failed: {result.stderr[-4096:]!r}")


def legacy() -> Any:
    need(sha256_file(LEGACY_AUDITOR_PATH) == "b83331a49d74d0f890a82750b9e0d9c0b9f073d8059dbeb735de5fb3cf594261", "legacy parser SHA drift")
    spec = importlib.util.spec_from_file_location("lay_v10r2_audit_pinned", LEGACY_AUDITOR_PATH)
    need(spec is not None and spec.loader is not None, "legacy parser import unavailable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def fixed_local(require_implementation: bool = False) -> dict[str, Any]:
    expected = {
        PAPER: (5_058, "5fe73bd9597766dcae830715d56ddf471ae8edb200b29a44c4e123b32c2f8ba9", "0444"),
        STRUCTURAL_RECEIPT: (23_503, "2370843fd63f312e0d638a6a78b9347533a9251a262454beba818135747598fc", "0444"),
        PREFLIGHT: (22_930, "50261f2b14bc45a97ed62e81c8a41149c65815d3d70021ed899c3f32e867c110", "0444"),
        OLD_JOURNAL_MANIFEST: (2_232, "5bf039fa3555c8ed7bb81fd6a1b7703ced12e235e4c53590ae872d64621e3aed", "0444"),
        OLD_BUILD_RECEIPT: (254_342, "edf6a15846f5918c2524299b60d33be41f02fe6ec56e21e04ba3c72cb3626706", "0444"),
        OLD_BUILD_MANIFEST: (661, "6aaf20d21d46535c4520c6daec45cc9b194cf3f591e0e917e992b48a9d17cdbd", "0444"),
        OLD_LOCAL_ELF: (EXPECTED_ELF_SIZE, EXPECTED_ELF_SHA256, "0444"),
        OLD_QUIET_DIAGNOSIS: (2_646, "c2974aca094f09c097aea362fd906b8727f0cb4c6b014beb6f1895c14c8899c8", "0444"),
        LEGACY_AUDITOR_PATH: (82_560, "b83331a49d74d0f890a82750b9e0d9c0b9f073d8059dbeb735de5fb3cf594261", "0555"),
        ADMISSION_SOURCE: (88_326, "6169e6d89a06c9ad3d7aefd467a8147f6d094b962faa25c739ad6d94a364b3dd", "0664"),
    }
    rows = {}
    for path, (size, digest, mode) in expected.items():
        row = file_row(path)
        need((row["size_bytes"], row["sha256"], row["mode"]) == (size, digest, mode), f"fixed local input drift: {path}")
        rows[str(path)] = row
    preflight = load_json(PREFLIGHT)
    need(preflight.get("verdict") == "READY_TO_IMPLEMENT" and preflight.get("safe_to_implement") is True, "implementation preflight not ready")
    need(preflight.get("manifest_sha256") == "10e233ca801e4c747901fa768f311c47097b960b4d732bc56c9f77b124a4d49a", "preflight manifest identity drift")
    verify_manifest(OLD_BUILD_ROOT)
    if require_implementation:
        need(IMPLEMENTATION_RECEIPT.is_file(), "controller implementation receipt absent")
        implementation = load_json(IMPLEMENTATION_RECEIPT)
        need(implementation.get("verdict") == "V10R3_TRACE_REUSE_CONTROLLERS_VERIFIED_UNRUN", "implementation verdict drift")
        need(implementation.get("local_controller_sha256") == sha256_file(CONTROLLER), "local controller SHA drift")
        need(implementation.get("remote_controller_sha256") == sha256_file(REMOTE_CONTROLLER), "remote controller SHA drift")
        need(implementation.get("auditor_sha256") == sha256_file(AUDITOR), "auditor SHA drift")
    return rows


def local_runtime_snapshot() -> dict[str, Any]:
    rows = {}
    for name in ("lay", "lay-daemon", "lay-ibus-engine"):
        path = pathlib.Path("/home/ubu/.local/bin") / name
        if path.exists():
            target = path.resolve()
            rows[name] = {"link": str(path), "target": str(target), "sha256": sha256_file(target), "size_bytes": target.stat().st_size}
    return rows


REMOTE_SNAPSHOT = r'''
import hashlib,json,os,pathlib,re,stat,subprocess,sys,time
new_task,parent_raw,state_raw,cache_raw,old_task,old_parent_raw,old_state_raw=sys.argv[1:]
parent=pathlib.Path(parent_raw); state=pathlib.Path(state_raw); cache=pathlib.Path(cache_raw)
old_parent=pathlib.Path(old_parent_raw); old_state=pathlib.Path(old_state_raw)
old_bootstrap=old_parent/'bootstrap-v1'; old_elf=old_parent/'build-v1/v10-test-elf'
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1048576),b''): h.update(b)
 return h.hexdigest()
def row(path): return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}','size_bytes':path.stat().st_size,'sha256':sha(path)}
def tree(root): return [row(p) for p in sorted(root.rglob('*')) if p.is_file()]
def markers(root):
 m=root/'markers'
 return {'available':sorted(p.name for p in m.glob('*.available')) if m.is_dir() else [],'consumed':sorted(p.name for p in m.glob('*.consumed-before-exec')) if m.is_dir() else []}
def latest(root):
 rows=sorted(root.glob('STATE-*.json')) if root.is_dir() else []
 return json.loads(rows[-1].read_text()) if rows else None
def runtime():
 out=[]
 for root in (pathlib.Path('/home/e/.local/share/lay/nanda_wave/l2'),pathlib.Path('/home/e/.local/share/lay/nanda_wave/l1.1')):
  if root.is_dir():
   for p in sorted(root.iterdir()):
    if p.is_file() and (p.name.startswith('active') or p.suffix in {'.p2m','.p2r'}): out.append(row(p))
 return out
def ancestors():
 out=set(); pid=os.getpid()
 while pid>1 and pid not in out:
  out.add(pid)
  try: pid=int((pathlib.Path('/proc')/str(pid)/'stat').read_text().split()[3])
  except Exception: break
 return out
skip=ancestors(); conflicts=[]
tokens=('perf '+'record','perf '+'stat','cargo '+'test','rustc ', 'm3_end_to_end_physical_proof',str(old_elf))
for p in pathlib.Path('/proc').iterdir():
 if not p.name.isdigit() or int(p.name) in skip: continue
 try: raw=(p/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace').strip()
 except (FileNotFoundError,PermissionError,ProcessLookupError): continue
 if raw and any(x in raw for x in tokens): conflicts.append({'pid':int(p.name),'command':raw})
old_markers=markers(old_state); old_marker_rows={n:row(old_state/'markers'/n) for n in old_markers['available']+old_markers['consumed']}
inputs={}
for n in ('LAY-L2-RU-FULL-v13.bin','slice8b-v7-fixed-13x100.json','LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m','LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r','LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin','l11-proof.json','l11-installed.json'):
 inputs[n]=row(old_bootstrap/'inputs'/n)
notes=subprocess.run(['/usr/bin/readelf','-n',str(old_elf)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,text=True)
m=re.search(r'Build ID:\s*([0-9a-f]+)',notes.stdout)
old_projection={
 'task_id':old_task,'transaction_id':'fe5b4741c5d5711b48f356569f3be32a87142edd78979edf0aeb72a9616de7e6',
 'elf':row(old_elf),'elf_build_id':m.group(1) if m else None,'markers':old_markers,'marker_rows':old_marker_rows,
 'latest_state':latest(old_state),'inputs':inputs,'trace_exists':(old_parent/'trace-v1').exists(),
}
thermal={str(p):int(p.read_text().strip()) for p in pathlib.Path('/sys/devices/system/cpu').glob('cpu*/thermal_throttle/*') if p.read_text().strip().isdigit()}
print(json.dumps({
 'hostname':os.uname().nodename,'kernel':os.uname().release,'machine_id_sha256':sha('/etc/machine-id'),
 'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),
 'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),
 'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),
 'new_paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},
 'new_parent_mode':f'{stat.S_IMODE(parent.stat().st_mode):04o}' if parent.exists() else None,
 'new_state_mode':f'{stat.S_IMODE(state.stat().st_mode):04o}' if state.exists() else None,
 'new_parent_tree':tree(parent) if parent.exists() else [],'new_state_tree':tree(state) if state.exists() else [],
 'new_latest_state':latest(state),'new_markers':markers(state),'new_trace_exists':(parent/'trace-v1').exists(),
 'old_v10r2_projection':old_projection,'active_conflicts':sorted(conflicts,key=lambda x:x['pid']),
 'runtime_projection':runtime(),'thermal':thermal,'loadavg':pathlib.Path('/proc/loadavg').read_text().strip(),
 'free_bytes':os.statvfs('/home/e').f_bavail*os.statvfs('/home/e').f_frsize,'monotonic_ns':time.monotonic_ns(),
},sort_keys=True))
'''


def remote_snapshot() -> dict[str, Any]:
    return ssh_python(REMOTE_SNAPSHOT, [
        TASK_ID, str(REMOTE_PARENT), str(REMOTE_STATE), str(REMOTE_CACHE),
        OLD_TASK_ID, str(OLD_REMOTE_PARENT), str(OLD_REMOTE_STATE),
    ], timeout=3600)


def validate_host(value: Mapping[str, Any]) -> None:
    need(value.get("hostname") == "e-MEGA-MINI-M1-13th", "remote hostname drift")
    need(value.get("kernel") == "6.8.0-124-generic", "remote kernel drift")
    need(value.get("machine_id_sha256") == "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441", "remote machine identity drift")
    need((value.get("online"), value.get("core"), value.get("atom")) == ("0-19", "0-11", "12-19"), "remote topology drift")
    old = value.get("old_v10r2_projection", {})
    need(old.get("task_id") == OLD_TASK_ID and old.get("transaction_id") == OLD_TRANSACTION_ID, "old namespace drift")
    need(old.get("elf", {}).get("size_bytes") == EXPECTED_ELF_SIZE and old.get("elf", {}).get("sha256") == EXPECTED_ELF_SHA256, "old ELF drift")
    need(old.get("elf", {}).get("mode") == "0555" and old.get("elf_build_id") == EXPECTED_ELF_BUILD_ID, "old ELF mode/Build-ID drift")
    need(old.get("markers") == {"available": ["trace.available"], "consumed": ["build.consumed-before-exec"]}, "old marker drift")
    need(old.get("latest_state", {}).get("state") == "BUILD_CREATED_UNAUDITED" and old.get("trace_exists") is False, "old state drift")


def live_admission() -> dict[str, Any]:
    fixed_local(require_implementation=True)
    for path in (ADMISSION_ROOT, BOOTSTRAP_ROOT, QUIET_ROOT, TERMINAL_ROOT):
        need(not path.exists(), f"future receipt already exists: {path}")
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot.get("new_paths") == {"parent": False, "state": False, "cache": False}, "V10R3 remote namespace/cache not absent")
    need(not snapshot.get("active_conflicts"), "conflicting performance process active")
    implementation = load_json(IMPLEMENTATION_RECEIPT)
    receipt = {
        "schema": "lay.v10r3-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R3_EXECUTION_ADMITTED",
        "safe_to_execute": True,
        "local_controller_sha256": implementation["local_controller_sha256"],
        "remote_controller_sha256": implementation["remote_controller_sha256"],
        "auditor_sha256": implementation["auditor_sha256"],
        "controller_implementation_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "controller_preflight_sha256": sha256_file(PREFLIGHT),
        "old_v10r2_build_audit_sha256": sha256_file(OLD_BUILD_RECEIPT),
        "old_v10r2_projection": snapshot["old_v10r2_projection"],
        "local_runtime_before": local_runtime_snapshot(),
        "remote_runtime_before": snapshot["runtime_projection"],
        "host": {key: snapshot[key] for key in ("hostname", "kernel", "machine_id_sha256", "online", "core", "atom")},
        "routes": ["TRACE-REUSE"],
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "bootstrap-reuse only",
    }
    return publish_tree(ADMISSION_ROOT, "EXECUTION_ADMISSION.json", receipt)


def bootstrap_audit() -> dict[str, Any]:
    fixed_local(require_implementation=True)
    admission = load_sealed_receipt(ADMISSION_ROOT, ADMISSION_RECEIPT, ("V10R3_EXECUTION_ADMITTED",))
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot.get("new_paths", {}).get("parent") is True and snapshot.get("new_paths", {}).get("state") is True, "bootstrap namespace absent")
    need(snapshot.get("new_parent_mode") == "0555" and snapshot.get("new_state_mode") == "0700", "bootstrap namespace mode drift")
    need(snapshot.get("new_latest_state", {}).get("state") == "BOOTSTRAP_REUSE_CREATED_UNAUDITED", "bootstrap state drift")
    need(snapshot.get("new_markers") == {"available": [], "consumed": []}, "marker created before quiet gate")
    need(snapshot.get("new_trace_exists") is False and not snapshot.get("active_conflicts"), "bootstrap scientific boundary drift")
    need(snapshot.get("old_v10r2_projection") == admission.get("old_v10r2_projection"), "old V10R2 projection changed")
    need(snapshot.get("runtime_projection") == admission.get("remote_runtime_before"), "remote runtime changed")
    need(local_runtime_snapshot() == admission.get("local_runtime_before"), "local runtime changed")

    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10r3-bootstrap-audit-"))
    try:
        remote_evidence = temporary / "REMOTE_BOOTSTRAP"
        copy_remote(REMOTE_PARENT, remote_evidence)
        entries = verify_manifest(remote_evidence)
        producer = load_json(remote_evidence / "BOOTSTRAP_REUSE_RECEIPT.json")
        need(producer.get("verdict") == "V10R3_BOOTSTRAP_REUSE_CREATED_UNAUDITED", "bootstrap producer verdict drift")
        need(producer.get("old_v10r2_projection") == admission.get("old_v10r2_projection"), "bootstrap producer old projection drift")
        need(producer.get("markers_created") == 0 and producer.get("cargo_invocations") == 0 and producer.get("subject_executions") == 0, "bootstrap execution ledger drift")
        receipt = {
            "schema": "lay.v10r3-bootstrap-reuse-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "V10R3_BOOTSTRAP_REUSE_AUDIT_PASS_QUIET_ADMITTED",
            "local_controller_sha256": admission["local_controller_sha256"],
            "remote_controller_sha256": admission["remote_controller_sha256"],
            "auditor_sha256": admission["auditor_sha256"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "producer_receipt_sha256": sha256_file(remote_evidence / "BOOTSTRAP_REUSE_RECEIPT.json"),
            "remote_manifest_entries": entries,
            "old_v10r2_projection": admission["old_v10r2_projection"],
            "uid_capability": producer.get("uid_capability"),
            "routes": ["TRACE-REUSE"],
            "markers_expected": 1,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "independent bounded quiet readiness only",
        }
        return publish_tree(BOOTSTRAP_ROOT, "BOOTSTRAP_AUDIT.json", receipt, {"REMOTE_BOOTSTRAP": remote_evidence})
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


QUIET_WINDOW = r'''
import json,os,pathlib,time
def cpu():
 out={}
 for line in pathlib.Path('/proc/stat').read_text().splitlines():
  f=line.split()
  if not f or not f[0].startswith('cpu'): continue
  v=[int(x) for x in f[1:]]; out[f[0]]=(sum(v),v[3]+(v[4] if len(v)>4 else 0))
 return out
def thermal(): return {str(p):int(p.read_text().strip()) for p in pathlib.Path('/sys/devices/system/cpu').glob('cpu*/thermal_throttle/*') if p.read_text().strip().isdigit()}
def runtime():
 out=[]
 for root in (pathlib.Path('/home/e/.local/share/lay/nanda_wave/l2'),pathlib.Path('/home/e/.local/share/lay/nanda_wave/l1.1')):
  if root.is_dir():
   for p in sorted(root.iterdir()):
    if p.is_file() and (p.name.startswith('active') or p.suffix in {'.p2m','.p2r'}): out.append((str(p),p.stat().st_size,p.stat().st_mtime_ns))
 return out
def ancestors():
 out=set(); pid=os.getpid()
 while pid>1 and pid not in out:
  out.add(pid)
  try: pid=int((pathlib.Path('/proc')/str(pid)/'stat').read_text().split()[3])
  except Exception: break
 return out
def conflicts():
 out=[]; skip=ancestors(); tokens=('perf '+'record','perf '+'stat','cargo '+'test','rustc ','m3_end_to_end_physical_proof')
 for p in pathlib.Path('/proc').iterdir():
  if not p.name.isdigit() or int(p.name) in skip: continue
  try: raw=(p/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace').strip()
  except (FileNotFoundError,PermissionError,ProcessLookupError): continue
  if raw and any(x in raw for x in tokens): out.append({'pid':int(p.name),'command':raw})
 return sorted(out,key=lambda x:x['pid'])
a=cpu(); ta=thermal(); ra=runtime(); ca=conflicts(); start=time.monotonic_ns(); time.sleep(5); end=time.monotonic_ns(); b=cpu(); tb=thermal(); rb=runtime(); cb=conflicts()
ratios={k:(b[k][1]-a[k][1])/(b[k][0]-a[k][0]) for k in a if k in b and b[k][0]>a[k][0]}
print(json.dumps({'started_monotonic_ns':start,'ended_monotonic_ns':end,'window_seconds':5,'idle_ratios':ratios,'cpu0_idle_ratio':ratios.get('cpu0'),'all_idle_ratio':ratios.get('cpu'),'conflicts_before':ca,'conflicts_after':cb,'thermal_before':ta,'thermal_after':tb,'runtime_before':ra,'runtime_after':rb},sort_keys=True))
'''


def quiet_admission() -> dict[str, Any]:
    fixed_local(require_implementation=True)
    admission = load_sealed_receipt(ADMISSION_ROOT, ADMISSION_RECEIPT, ("V10R3_EXECUTION_ADMITTED",))
    bootstrap = load_sealed_receipt(BOOTSTRAP_ROOT, BOOTSTRAP_RECEIPT, ("V10R3_BOOTSTRAP_REUSE_AUDIT_PASS_QUIET_ADMITTED",))
    before = remote_snapshot()
    validate_host(before)
    need(before.get("new_latest_state", {}).get("state") == "BOOTSTRAP_REUSE_CREATED_UNAUDITED", "quiet predecessor drift")
    need(before.get("new_markers") == {"available": [], "consumed": []} and before.get("new_trace_exists") is False, "marker or TRACE exists before quiet gate")
    need(before.get("old_v10r2_projection") == admission.get("old_v10r2_projection"), "old projection drift before quiet gate")
    need(before.get("runtime_projection") == admission.get("remote_runtime_before"), "runtime drift before quiet gate")
    need(not before.get("active_conflicts"), "conflicting process before quiet gate")

    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10r3-quiet-"))
    observations = temporary / "OBSERVATIONS"
    observations.mkdir(mode=0o700)
    rows = []
    consecutive = 0
    terminal_reason = None
    try:
        for attempt in range(1, 121):
            row = ssh_python(QUIET_WINDOW, root=True, timeout=30)
            conflict = bool(row.get("conflicts_before") or row.get("conflicts_after"))
            thermal = row.get("thermal_before") != row.get("thermal_after")
            runtime = row.get("runtime_before") != row.get("runtime_after")
            cpu0 = float(row.get("cpu0_idle_ratio") or 0.0)
            all_cpu = float(row.get("all_idle_ratio") or 0.0)
            passed = not conflict and not thermal and not runtime and cpu0 >= 0.95 and all_cpu >= 0.90
            row.update({"attempt": attempt, "passed": passed, "consecutive_after": consecutive + 1 if passed else 0})
            rows.append(row)
            write_new(observations / f"WINDOW-{attempt:03d}.json", canonical(row), 0o444)
            consecutive = consecutive + 1 if passed else 0
            if conflict:
                terminal_reason = "conflicting process observed"
                break
            if thermal:
                terminal_reason = "thermal throttle counter changed"
                break
            if runtime:
                terminal_reason = "runtime projection changed during quiet window"
                break
            if consecutive == 3:
                break
        passed = consecutive == 3 and terminal_reason is None
        if not passed and terminal_reason is None:
            terminal_reason = "120 quiet windows exhausted without three consecutive passes"
        after = remote_snapshot()
        validate_host(after)
        stable = (
            after.get("old_v10r2_projection") == admission.get("old_v10r2_projection")
            and after.get("runtime_projection") == admission.get("remote_runtime_before")
            and after.get("new_markers") == {"available": [], "consumed": []}
            and after.get("new_trace_exists") is False
            and not after.get("active_conflicts")
        )
        if not stable:
            passed = False
            terminal_reason = terminal_reason or "post-window provenance projection drift"
        verdict = "V10R3_QUIET_READY_TRACE_ADMITTED" if passed else "BLOCKED_QUIET_BEFORE_MARKER"
        receipt = {
            "schema": "lay.v10r3-quiet-readiness.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": verdict,
            "local_controller_sha256": admission["local_controller_sha256"],
            "remote_controller_sha256": admission["remote_controller_sha256"],
            "auditor_sha256": admission["auditor_sha256"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_RECEIPT),
            "old_v10r2_build_audit_sha256": sha256_file(OLD_BUILD_RECEIPT),
            "old_elf_sha256": EXPECTED_ELF_SHA256,
            "old_elf_build_id": EXPECTED_ELF_BUILD_ID,
            "old_v10r2_projection": admission["old_v10r2_projection"],
            "window_seconds": 5,
            "max_windows": 120,
            "cpu0_idle_threshold": 0.95,
            "all_cpu_idle_threshold": 0.90,
            "attempts": len(rows),
            "consecutive_passes": consecutive,
            "conflict_windows": sum(bool(row.get("conflicts_before") or row.get("conflicts_after")) for row in rows),
            "thermal_drift_windows": sum(row.get("thermal_before") != row.get("thermal_after") for row in rows),
            "runtime_drift_windows": sum(row.get("runtime_before") != row.get("runtime_after") for row in rows),
            "terminal_reason": terminal_reason,
            "host_before": before,
            "host_after": after,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
            "retry_permitted": False,
            "next_action_admitted": "create one TRACE-REUSE marker" if passed else "none",
        }
        return publish_tree(QUIET_ROOT, "QUIET_ADMISSION.json", receipt, {
            "OBSERVATIONS": observations,
            "BOOTSTRAP_AUDIT.json": BOOTSTRAP_RECEIPT,
        })
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def expected_trace_environment() -> dict[str, str]:
    subject = REMOTE_PARENT / "trace-v1/subject"
    inputs = OLD_REMOTE_BOOTSTRAP / "inputs"
    return {
        "HOME": "/home/e",
        "LANG": "C",
        "LAY_L11_RECEIPT": str(inputs / "l11-installed.json"),
        "LAY_L2_FIELD_TRACE": "1",
        "LAY_L2_PACKAGE": str(inputs / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m"),
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(inputs / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_M3_ACTUAL_OWNER_V7": str(inputs / "slice8b-v7-fixed-13x100.json"),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LAY_PROPOSAL_ADMISSION_TRACE": "1",
        "LC_ALL": "C",
        "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
        "TZ": "UTC",
    }


def expected_trace_command() -> list[str]:
    environment = expected_trace_environment()
    return [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0", str(OLD_REMOTE_ELF),
        "--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]


def marker_payload(quiet: Mapping[str, Any]) -> bytes:
    return canonical({
        "schema": "lay.v10r3-one-shot-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": "TRACE-REUSE",
        "local_controller_sha256": quiet["local_controller_sha256"],
        "remote_controller_sha256": quiet["remote_controller_sha256"],
        "auditor_sha256": quiet["auditor_sha256"],
        "quiet_receipt_sha256": sha256_file(QUIET_RECEIPT),
        "old_elf_sha256": EXPECTED_ELF_SHA256,
        "one_shot": True,
        "retry_permitted": False,
    })


def dispatch_failures(failures: Mapping[str, Sequence[str]]) -> str:
    for verdict, key in (
        ("BLOCKED_PROVENANCE", "provenance"),
        ("BLOCKED_BUILD", "build"),
        ("BLOCKED_SEMANTIC", "semantic"),
        ("BLOCKED_CAPABILITY", "capability"),
    ):
        if failures.get(key):
            return verdict
    return "ADMISSION_SUBSTAGES_DECOMPOSED"


def terminal_decision(
    subject: Mapping[str, Any],
    wrapper: Mapping[str, Any],
    build: Mapping[str, Any],
    trace_rows: Sequence[Mapping[str, Any]],
    trace_failures: Mapping[str, Sequence[str]],
    trace_summary: Mapping[str, Any] | None,
) -> tuple[str, dict[str, list[str]]]:
    parser_module = legacy()
    failures = {key: [] for key in ("provenance", "build", "semantic", "capability")}
    if wrapper.get("schema") != "lay.v10r3-trace-wrapper.v1" or wrapper.get("task_id") != TASK_ID or wrapper.get("transaction_id") != TRANSACTION_ID:
        failures["provenance"].append("V10R3 wrapper namespace or schema drift")
    if wrapper.get("command") != expected_trace_command() or wrapper.get("environment") != expected_trace_environment():
        failures["provenance"].append("V10R3 scientific command or environment drift")
    if wrapper.get("verdict") not in {"V10R3_TRACE_REUSE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"}:
        failures["provenance"].append("V10R3 producer wrapper verdict drift")
    if wrapper.get("reused_elf", {}).get("sha256") != EXPECTED_ELF_SHA256 or wrapper.get("reused_elf_build_id") != EXPECTED_ELF_BUILD_ID:
        failures["provenance"].append("reused ELF identity drift")
    if wrapper.get("rustc_compilations") != 0:
        failures["provenance"].append("V10R3 rustc ledger drift")

    legacy_wrapper = dict(wrapper)
    legacy_wrapper.update({
        "schema": "lay.v10r2-trace-wrapper.v1",
        "task_id": OLD_TASK_ID,
        "transaction_id": OLD_TRANSACTION_ID,
        "verdict": "V10R2_TRACE_CREATED_UNAUDITED" if wrapper.get("verdict") == "V10R3_TRACE_REUSE_CREATED_UNAUDITED" else "BLOCKED_PROVENANCE",
        "command": parser_module.expected_trace_command(),
        "environment": parser_module.expected_trace_environment(),
    })
    _, legacy_failures = parser_module.terminal_decision(subject, legacy_wrapper, build, trace_rows, trace_failures, trace_summary)
    for key, values in legacy_failures.items():
        failures[key].extend(str(value) for value in values)
    return dispatch_failures(failures), failures


def terminal_audit() -> dict[str, Any]:
    fixed_local(require_implementation=True)
    admission = load_sealed_receipt(ADMISSION_ROOT, ADMISSION_RECEIPT, ("V10R3_EXECUTION_ADMITTED",))
    bootstrap = load_sealed_receipt(BOOTSTRAP_ROOT, BOOTSTRAP_RECEIPT, ("V10R3_BOOTSTRAP_REUSE_AUDIT_PASS_QUIET_ADMITTED",))
    quiet = load_sealed_receipt(QUIET_ROOT, QUIET_RECEIPT, ("V10R3_QUIET_READY_TRACE_ADMITTED",))
    build = load_json(OLD_BUILD_RECEIPT)
    need(build.get("verdict") == "V10R2_BUILD_AUDIT_PASS_TRACE_ADMITTED", "old build audit no longer admits sealed ELF")
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot.get("new_markers") == {"available": [], "consumed": ["trace.consumed-before-exec"]}, "terminal marker state drift")
    need(snapshot.get("new_latest_state", {}).get("state") in {"TRACE_REUSE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"}, "terminal state drift")
    need(snapshot.get("new_trace_exists") is True and not snapshot.get("active_conflicts"), "terminal TRACE/process boundary drift")
    need(snapshot.get("old_v10r2_projection") == admission.get("old_v10r2_projection"), "old V10R2 history changed")

    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10r3-terminal-"))
    try:
        remote_evidence = temporary / "REMOTE_TRACE"
        copy_remote(REMOTE_PARENT / "trace-v1", remote_evidence)
        entries = verify_manifest(remote_evidence)
        wrapper_path = remote_evidence / "TRACE_WRAPPER.json"
        stderr_path = remote_evidence / "stderr.log"
        wrapper = load_json(wrapper_path) if wrapper_path.is_file() else {}
        stderr = stderr_path.read_bytes() if stderr_path.is_file() else b""
        parser_module = legacy()
        rows, trace_failures, summary = parser_module.parse_trace(stderr)
        subject = wrapper.get("subject_receipt") if isinstance(wrapper.get("subject_receipt"), dict) else {}
        verdict, failures = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
        subject_path = remote_evidence / "subject/SUBJECT_RECEIPT.json"
        if subject_path.is_file():
            if load_json(subject_path) != subject:
                failures["provenance"].append("retained subject receipt disagrees with wrapper")
        elif subject:
            failures["provenance"].append("embedded subject receipt absent from retained evidence")
        expected = marker_payload(quiet)
        marker = wrapper.get("marker", {}) if isinstance(wrapper.get("marker"), dict) else {}
        before_marker = marker.get("before", {}) if isinstance(marker.get("before"), dict) else {}
        after_marker = marker.get("after", {}) if isinstance(marker.get("after"), dict) else {}
        if (
            marker.get("consumed_before_execution") is not True
            or before_marker.get("path") != str(REMOTE_STATE / "markers/trace.available")
            or after_marker.get("path") != str(REMOTE_STATE / "markers/trace.consumed-before-exec")
            or before_marker.get("mode") != "0400"
            or after_marker.get("mode") != "0400"
            or before_marker.get("size_bytes") != len(expected)
            or after_marker.get("size_bytes") != len(expected)
            or before_marker.get("sha256") != sha256_bytes(expected)
            or after_marker.get("sha256") != sha256_bytes(expected)
        ):
            failures["provenance"].append("TRACE-REUSE marker consumption evidence drift")
        if local_runtime_snapshot() != admission.get("local_runtime_before"):
            failures["provenance"].append("local runtime authority changed")
        if snapshot.get("runtime_projection") != admission.get("remote_runtime_before"):
            failures["provenance"].append("remote runtime authority changed")
        verdict = dispatch_failures(failures)
        derived = temporary / "DERIVED_TRACE"
        derived.mkdir(mode=0o700)
        write_new(derived / "TRACE_ROWS.jsonl", b"".join(canonical(row) for row in rows))
        write_new(derived / "TRACE_SUMMARY.json", canonical({
            "schema": "lay.v10r3-admission-substage-derived.v1",
            "parse_failures": trace_failures,
            "summary": summary,
        }))
        receipt = {
            "schema": "lay.v10r3-terminal-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": verdict,
            "positive_verdict": "ADMISSION_SUBSTAGES_DECOMPOSED",
            "failure_priority": ["provenance", "build", "semantic", "capability", "complete_decomposition"],
            "failures": failures,
            "local_controller_sha256": admission["local_controller_sha256"],
            "remote_controller_sha256": admission["remote_controller_sha256"],
            "auditor_sha256": admission["auditor_sha256"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_RECEIPT),
            "quiet_admission_sha256": sha256_file(QUIET_RECEIPT),
            "old_build_audit_sha256": sha256_file(OLD_BUILD_RECEIPT),
            "trace_wrapper_sha256": sha256_file(wrapper_path) if wrapper_path.is_file() else None,
            "stderr_sha256": sha256_file(stderr_path) if stderr_path.is_file() else None,
            "remote_manifest_entries": entries,
            "scientific_receipt": subject,
            "trace": summary,
            "trace_rows": len(rows),
            "v9_trace_rows": wrapper.get("trace_lines", {}).get("v9_aggregate"),
            "v10_trace_rows": wrapper.get("trace_lines", {}).get("v10_substage"),
            "warmup_rows": parser_module.WARMUP_ROWS,
            "measured_rows": parser_module.MEASURED_ROWS,
            "markers_created": 1,
            "markers_consumed": 1,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "historical_reused_build_invocations": 1,
            "subject_executions": 1,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "installed_package_changed": False,
            "runtime_authority_changed": False,
            "production_authority_admitted": False,
            "next_if_pass": "separate mechanism decision from measured substage evidence only",
            "live_projection": snapshot,
        }
        return publish_tree(TERMINAL_ROOT, "TERMINAL_AUDIT.json", receipt, {
            "REMOTE_TRACE": remote_evidence,
            "DERIVED_TRACE": derived,
        })
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def self_check() -> dict[str, Any]:
    fixed_local(require_implementation=False)
    need(ACTIONS == ("self-check", "live-admission", "bootstrap", "quiet", "terminal", "status"), "auditor registry drift")
    parser_module = legacy()
    need(len(parser_module.EXPECTED_STAGE_NAMES) == 36 and len(parser_module.EXPECTED_REASON_NAMES) == 43, "trace registry drift")
    rows, trace_failures, summary = parser_module.parse_trace(parser_module.synthetic_trace())
    need(not any(trace_failures.values()) and summary is not None, "pinned parser synthetic trace failed")
    subject = parser_module.synthetic_subject()
    wrapper = {
        "schema": "lay.v10r3-trace-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R3_TRACE_REUSE_CREATED_UNAUDITED",
        "controller_error": None,
        "command": expected_trace_command(),
        "environment": expected_trace_environment(),
        "exit_code": 101,
        "outputs_complete": True,
        "trace_lines": {"v9_aggregate": parser_module.EXPECTED_TRACE_ROWS, "v10_substage": parser_module.EXPECTED_TRACE_ROWS},
        "reused_elf": {"sha256": EXPECTED_ELF_SHA256},
        "reused_elf_build_id": EXPECTED_ELF_BUILD_ID,
        "thermal_throttle_drift": {},
        "subject_executions": 1,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
    }
    build = load_json(OLD_BUILD_RECEIPT)
    subject["source"]["test_elf_sha256"] = build["elf_sha256"]
    verdict, failures = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
    need(verdict == "ADMISSION_SUBSTAGES_DECOMPOSED" and not any(failures.values()), "positive dispatch model failed")
    subject["fixed_proof"]["semantic"]["candidate_mismatches"] = 1
    wrapper["controller_error"] = "synthetic capability failure"
    verdict, _ = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
    need(verdict == "BLOCKED_SEMANTIC", "failure-priority model failed")
    wrapper["command"] = []
    verdict, _ = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
    need(verdict == "BLOCKED_PROVENANCE", "provenance-priority model failed")
    quiet_model = [
        {"cpu0": 0.96, "all": 0.91},
        {"cpu0": 0.98, "all": 0.93},
        {"cpu0": 0.99, "all": 0.94},
    ]
    need(all(row["cpu0"] >= 0.95 and row["all"] >= 0.90 for row in quiet_model), "quiet positive model failed")
    return {
        "schema": "lay.v10r3-independent-auditor-self-check.v1",
        "verdict": "V10R3_INDEPENDENT_AUDITOR_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "auditor_sha256": sha256_file(AUDITOR),
        "legacy_parser_sha256": sha256_file(LEGACY_AUDITOR_PATH),
        "actions": list(ACTIONS),
        "trace_rows": len(rows),
        "positive_dispatch": "PASS",
        "failure_priority_dispatch": "PASS",
        "provenance_priority_dispatch": "PASS",
        "quiet_three_consecutive_model": "PASS",
        "remote_reads": 0,
        "remote_writes": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "scientific_actions": 0,
    }


def status() -> dict[str, Any]:
    return {
        "schema": "lay.v10r3-auditor-status.v1",
        "verdict": "V10R3_AUDITOR_STATUS",
        "receipts": {
            "execution_admission": ADMISSION_RECEIPT.exists(),
            "bootstrap_audit": BOOTSTRAP_RECEIPT.exists(),
            "quiet_admission": QUIET_RECEIPT.exists(),
            "terminal_audit": TERMINAL_RECEIPT.exists(),
        },
        "remote": remote_snapshot(),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=ACTIONS)
    args = parser.parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "live-admission":
            value = live_admission()
        elif args.action == "bootstrap":
            value = bootstrap_audit()
        elif args.action == "quiet":
            value = quiet_admission()
        elif args.action == "terminal":
            value = terminal_audit()
        else:
            value = status()
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v10r3-auditor-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
