#!/usr/bin/env python3
"""Independent admission and evidence auditor for the M3 V8 remote proof."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import struct
import subprocess
import sys
import tempfile
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827"
TRANSACTION_ID = "7d6455e678c244be3c31dc52c2b64d55f34d0a91338afa1219acf06ff327ffb9"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v8r1-{TRANSACTION_ID}"

LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r1.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r1-remote.py"
AUDITOR = pathlib.Path(__file__).resolve()
V8_SOURCE = ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
V8_PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8_2026-08-27.md"
V8R1_CORRECTION = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_ADMISSION_UID_CONTEXT_CORRECTION_V1_2026-08-27.md"
V8_V1_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8_V1_ADMISSION_FAILURE_DIAGNOSIS_2026-08-27.json"
V8_V1_JOURNAL_MANIFEST = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
SOURCE_PREFLIGHT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_IMPLEMENTATION_V8_V2_PREFLIGHT_2026-08-27.json"
SOURCE_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_IMPLEMENTATION_V8_2026-08-27/IMPLEMENTATION_RECEIPT.json"
CONTROLLER_PREFLIGHT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_REMOTE_CONTROLLER_V8R1_V1_PREFLIGHT_2026-08-27.json"
CONTROLLER_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_REMOTE_CONTROLLER_V8R1_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"

RECEIPTS = ROOT / "docs/structural_gates/receipts"
ADMISSION_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_EXECUTION_ADMISSION_V1_2026-08-27"
BOOTSTRAP_AUDIT_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_BOOTSTRAP_AUDIT_V1_2026-08-27"
BUILD_AUDIT_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_BUILD_AUDIT_V1_2026-08-27"
QUIET_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_QUIET_ADMISSION_V1_2026-08-27"
TERMINAL_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_TERMINAL_AUDIT_V1_2026-08-27"

ADMISSION_RECEIPT = ADMISSION_ROOT / "EXECUTION_ADMISSION.json"
BOOTSTRAP_AUDIT_RECEIPT = BOOTSTRAP_AUDIT_ROOT / "BOOTSTRAP_AUDIT.json"
BUILD_AUDIT_RECEIPT = BUILD_AUDIT_ROOT / "BUILD_AUDIT.json"
QUIET_RECEIPT = QUIET_ROOT / "QUIET_ADMISSION.json"
TERMINAL_RECEIPT = TERMINAL_ROOT / "TERMINAL_AUDIT.json"

ACTIONS = ("self-check", "live-admission", "bootstrap", "build", "quiet", "terminal", "status")
HOSTNAME = "e-MEGA-MINI-M1-13th"
KERNEL = "6.8.0-124-generic"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
EXPECTED_SOURCE_SHA = "28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2"
EXPECTED_V13_SHA = "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"
EXPECTED_V7_SHA = "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"
EXPECTED_PRODUCTIVE_SHA = "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"
EXPECTED_L11_SHA = "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"
SEMANTIC_FIELDS = (
    "candidate_mismatches",
    "certificate_mismatches",
    "structured_certificate_mismatches",
    "schedule_mismatches",
    "completeness_mismatches",
    "lattice_marker_mismatches",
    "emitted_surface_mismatches",
    "gate_mismatches",
    "certificate_collisions",
    "semantic_total",
)


class V8AuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V8AuditError(message)


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
    need(path.is_file() and not path.is_symlink(), f"required file absent: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


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
            rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def verify_manifest(root: pathlib.Path) -> int:
    rows = (root / "SHA256SUMS").read_text().splitlines()
    for row in rows:
        digest, relative = row.split("  ", 1)
        need(sha256_file(root / relative) == digest, f"manifest mismatch: {relative}")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "SHA256SUMS"
    }
    need(actual == {row.split("  ", 1)[1] for row in rows}, "manifest inventory drift")
    return len(rows)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish_tree(destination: pathlib.Path, receipt_name: str, receipt: Mapping[str, Any], copied: Mapping[str, pathlib.Path] | None = None) -> dict[str, Any]:
    need(not destination.exists(), f"audit evidence already exists: {destination}")
    stage = destination.with_name(f"{destination.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / receipt_name, canonical(receipt))
        for name, source in (copied or {}).items():
            target = stage / name
            if source.is_dir():
                shutil.copytree(source, target)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, destination)
        fsync_dir(destination.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return load_json(destination / receipt_name)


def run(
    argv: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: float = 3_600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        stdout = result.stdout[-4000:].decode(errors="replace")
        stderr = result.stderr[-4000:].decode(errors="replace")
        raise V8AuditError(
            f"command failed ({result.returncode}): {list(argv)!r}\n"
            f"stdout:\n{stdout}\nstderr:\n{stderr}"
        )
    return result


def ssh_python(program: str, arguments: Sequence[str] = (), *, root: bool = False, timeout: float = 3_600) -> dict[str, Any]:
    remote_command = ["/usr/bin/python3", "-", *arguments]
    if root:
        remote_command = ["/usr/bin/sudo", "-n", *remote_command]
    result = run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            *remote_command,
        ],
        input_bytes=program.encode(),
        timeout=timeout,
    )
    lines = result.stdout.decode().strip().splitlines()
    need(lines, "remote snapshot returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote snapshot is not an object")
    return value


def copy_remote(remote: pathlib.PurePosixPath, destination: pathlib.Path) -> None:
    destination.mkdir(parents=True, mode=0o700)
    run(
        [
            "/usr/bin/scp",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-q",
            "-p",
            "-r",
            f"{REMOTE}:{remote}/.",
            str(destination),
        ],
        timeout=3_600,
    )


def fixed_local() -> dict[str, Any]:
    expected = {
        V8_SOURCE: EXPECTED_SOURCE_SHA,
        V8_PAPER: "c5f1655ce4ab91f068f0b50aff1fe5a2a01206d64786b1e23dc4396e10b840a7",
        V8R1_CORRECTION: "64eff17ffa27779b3fe79ff4e156dd47c1de1ed43d34a4be432dcff4add1cafe",
        V8_V1_DIAGNOSIS: "e06dac04ae1fb50806d72bdeef01ca46efcbcc1a50a8749bceb78c984a378391",
        V8_V1_JOURNAL_MANIFEST: "9a415838a1f18e972548bdc1b5285042c0ae049f5809b8b4520fd7867020eb81",
        SOURCE_PREFLIGHT: "2b8eda4c0facaf1bde60adb4a99c7ed262d168dffa1ab326e01d3facc23db81b",
        SOURCE_IMPLEMENTATION: "cf4b38d81cc7f9ea9125855194635fafde9c76c9d022f7185635e8bb6c2f29e5",
    }
    rows = {}
    for path, digest in expected.items():
        row = file_row(path)
        need(row["sha256"] == digest, f"fixed local identity drift: {path}")
        rows[path.name] = row
    need(CONTROLLER_IMPLEMENTATION.is_file(), "controller implementation receipt absent")
    implementation = load_json(CONTROLLER_IMPLEMENTATION)
    need(implementation.get("verdict") == "M3_V8R1_REMOTE_CONTROLLERS_VERIFIED_UNRUN", "controller implementation verdict drift")
    for key, path in {
        "local_controller_sha256": LOCAL_CONTROLLER,
        "remote_controller_sha256": REMOTE_CONTROLLER,
        "auditor_sha256": AUDITOR,
    }.items():
        need(implementation.get(key) == sha256_file(path), f"controller implementation binding drift: {key}")
    rows["controller_implementation"] = file_row(CONTROLLER_IMPLEMENTATION)
    rows["controller_preflight"] = file_row(CONTROLLER_PREFLIGHT)
    return {"files": rows, "implementation": implementation}


def local_runtime_snapshot() -> dict[str, Any]:
    names = ("lay", "lay-daemon", "lay-ibus-engine")
    rows = {}
    for name in names:
        path = pathlib.Path("/home/ubu/.local/bin") / name
        if path.exists():
            target = path.resolve()
            rows[name] = {
                "link": str(path),
                "target": str(target),
                "sha256": sha256_file(target),
                "size_bytes": target.stat().st_size,
            }
    return rows


REMOTE_SNAPSHOT = r'''
import hashlib,json,os,pathlib,stat,sys,time
task=sys.argv[1]; parent=pathlib.Path(sys.argv[2]); state=pathlib.Path(sys.argv[3]); cache=pathlib.Path(sys.argv[4])
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1048576),b''): h.update(b)
 return h.hexdigest()
def row(path): return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}','size_bytes':path.stat().st_size,'sha256':sha(path)}
def tree(root): return [row(p) for p in sorted(root.rglob('*')) if p.is_file()]
def runtime():
 roots=[pathlib.Path('/home/e/.local/share/lay/nanda_wave/l2'),pathlib.Path('/home/e/.local/share/lay/nanda_wave/l1.1')]
 out=[]
 for root in roots:
  if root.is_dir():
   for p in sorted(root.iterdir()):
    if p.is_file() and (p.name.startswith('active') or p.suffix in {'.p2m','.p2r'}): out.append(row(p))
 return out
conflicts=[]
for p in pathlib.Path('/proc').iterdir():
 if not p.name.isdigit(): continue
 try: raw=(p/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace').strip()
 except (FileNotFoundError,PermissionError,ProcessLookupError): continue
 if raw and any(x in raw for x in ('perf record','perf stat','cargo test','rustc ', 'm3_end_to_end_physical_proof')):
  conflicts.append({'pid':int(p.name),'command':raw})
states=sorted(state.glob('STATE-*.json')) if state.is_dir() else []
markers=state/'markers'
out={
 'hostname':os.uname().nodename,'kernel':os.uname().release,'machine_id_sha256':sha('/etc/machine-id'),
 'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),
 'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),
 'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),
 'paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},
 'parent_mode':f'{stat.S_IMODE(parent.stat().st_mode):04o}' if parent.exists() else None,
 'state_mode':f'{stat.S_IMODE(state.stat().st_mode):04o}' if state.exists() else None,
 'parent_tree':tree(parent) if parent.exists() else [],
 'state_tree':tree(state) if state.exists() else [],
 'latest_state':json.loads(states[-1].read_text()) if states else None,
 'markers':{'available':sorted(p.name for p in markers.glob('*.available')) if markers.is_dir() else [],'consumed':sorted(p.name for p in markers.glob('*.consumed-before-exec')) if markers.is_dir() else []},
 'conflicting_processes':conflicts,'loadavg':pathlib.Path('/proc/loadavg').read_text().strip(),
 'thermal':{str(p):int(p.read_text().strip()) for p in pathlib.Path('/sys/devices/system/cpu').glob('cpu*/thermal_throttle/*') if p.read_text().strip().isdigit()},
 'runtime_projection':runtime(),'free_bytes':shutil.disk_usage('/home/e').free if False else os.statvfs('/home/e').f_bavail*os.statvfs('/home/e').f_frsize,
 'monotonic_ns':time.monotonic_ns(),
}
print(json.dumps(out,sort_keys=True))
'''


def remote_snapshot() -> dict[str, Any]:
    return ssh_python(
        REMOTE_SNAPSHOT,
        [TASK_ID, str(REMOTE_PARENT), str(REMOTE_STATE), str(REMOTE_CACHE)],
        root=True,
        timeout=3_600,
    )


REMOTE_TOOLCHAIN = r'''
import json,os,subprocess
environment={
 'HOME':'/home/e',
 'LANG':'C.UTF-8',
 'LC_ALL':'C.UTF-8',
 'PATH':'/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
 'RUST_BACKTRACE':'0',
}
def cmd(argv):
 p=subprocess.run(argv,env=environment,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,check=False)
 return {'returncode':p.returncode,'stdout':p.stdout.strip(),'stderr':p.stderr.strip()}
print(json.dumps({
 'execution_uid':os.geteuid(),
 'environment':environment,
 'cargo':cmd(['/home/e/.cargo/bin/cargo','-V']),
 'rustc':cmd(['/home/e/.cargo/bin/rustc','-Vv']),
},sort_keys=True))
'''


def remote_toolchain_snapshot() -> dict[str, Any]:
    return ssh_python(REMOTE_TOOLCHAIN, root=True, timeout=60)


def validate_host(value: Mapping[str, Any]) -> None:
    need(value.get("hostname") == HOSTNAME, "remote hostname drift")
    need(value.get("kernel") == KERNEL, "remote kernel drift")
    need(value.get("machine_id_sha256") == MACHINE_ID_SHA256, "remote machine-id drift")
    need((value.get("online"), value.get("core"), value.get("atom")) == ("0-19", "0-11", "12-19"), "remote topology drift")


def validate_toolchain(value: Mapping[str, Any]) -> None:
    need(value.get("execution_uid") == 0, "toolchain observer UID drift")
    expected_environment = {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
    }
    need(value.get("environment") == expected_environment, "controlled toolchain environment drift")
    cargo = value.get("cargo", {})
    need(cargo.get("returncode") == 0, "remote Cargo query failed")
    need(cargo.get("stdout") == "cargo 1.97.1 (c980f4866 2026-06-30)", "remote Cargo drift")
    rustc_row = value.get("rustc", {})
    need(rustc_row.get("returncode") == 0, "remote rustc query failed")
    rustc = str(rustc_row.get("stdout", ""))
    for token in ("release: 1.97.1", "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452", "LLVM version: 22.1.6"):
        need(token in rustc, "remote rustc drift")


UID_PROBE = r'''
import json,os,pathlib,sys,time
p=pathlib.Path(sys.argv[1]); p.mkdir(mode=0o700)
try:
 a=p/'a'; b=p/'b'; f=open(a,'xb'); f.write(b'm3-v8-admission\n'); f.flush(); os.fsync(f.fileno()); f.close(); os.rename(a,b)
 assert b.read_bytes()==b'm3-v8-admission\n'; b.unlink(); p.rmdir()
 print(json.dumps({'verdict':'PASS','operations':['create','write','fsync','rename','read','unlink'],'probe_absent_after':not p.exists()},sort_keys=True))
except BaseException:
 try:
  for x in p.iterdir(): x.unlink()
  p.rmdir()
 except BaseException: pass
 raise
'''


def live_admission() -> dict[str, Any]:
    local = fixed_local()
    before = remote_snapshot()
    validate_host(before)
    need(before["paths"] == {"parent": False, "state": False, "cache": False}, "V8 remote namespace is not absent")
    need(not before.get("conflicting_processes"), "conflicting remote performance process is active")
    need(int(before.get("free_bytes", 0)) >= 40 * 1024**3, "remote free-space gate failed")
    toolchain = remote_toolchain_snapshot()
    validate_toolchain(toolchain)
    probe_path = f"/home/e/.cache/lay-m3-v8r1-admission-{TRANSACTION_ID}"
    probe = ssh_python(UID_PROBE, [probe_path], root=False)
    need(probe.get("verdict") == "PASS" and probe.get("probe_absent_after") is True, "UID e capability probe failed")
    time.sleep(2)
    after = remote_snapshot()
    validate_host(after)
    need(after["paths"] == before["paths"], "remote namespace changed during admission")
    need(not after.get("conflicting_processes"), "remote host stopped being quiet during admission")
    receipt = {
        "schema": "lay.m3-v8r1-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R1_EXECUTION_ADMITTED",
        "safe_to_execute": True,
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "implementation_receipt_sha256": sha256_file(CONTROLLER_IMPLEMENTATION),
        "source_implementation_receipt_sha256": sha256_file(SOURCE_IMPLEMENTATION),
        "host_before": before,
        "host_after": after,
        "build_toolchain": toolchain,
        "toolchain_version_queries": 2,
        "uid_capability": probe,
        "local_runtime_before": local_runtime_snapshot(),
        "remote_runtime_before": before["runtime_projection"],
        "namespace_absent": True,
        "conflicting_processes": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "fixed_local": local["files"],
    }
    return publish_tree(ADMISSION_ROOT, "EXECUTION_ADMISSION.json", receipt)


def tree_index(value: Mapping[str, Any], key: str) -> dict[str, Mapping[str, Any]]:
    return {str(row["path"]): row for row in value.get(key, [])}


def validate_remote_manifest(snapshot: Mapping[str, Any], root: pathlib.PurePosixPath) -> int:
    index = tree_index(snapshot, "parent_tree")
    manifest_path = str(root / "SHA256SUMS")
    need(manifest_path in index, f"remote manifest absent: {manifest_path}")
    raw = ssh_python(
        "import json,pathlib,sys; print(json.dumps({'text':pathlib.Path(sys.argv[1]).read_text()},sort_keys=True))",
        [manifest_path],
        root=True,
    )["text"]
    rows = str(raw).splitlines()
    for row in rows:
        digest, relative = row.split("  ", 1)
        path = str(root / relative)
        need(path in index and index[path]["sha256"] == digest, f"remote manifest mismatch: {relative}")
    actual = {
        path.removeprefix(str(root) + "/")
        for path in index
        if path != manifest_path and path.startswith(str(root) + "/")
    }
    need(actual == {row.split("  ", 1)[1] for row in rows}, "remote manifest inventory drift")
    return len(rows)


def bootstrap_audit() -> dict[str, Any]:
    fixed_local()
    admission = load_json(ADMISSION_RECEIPT)
    need(admission.get("verdict") == "M3_V8R1_EXECUTION_ADMITTED", "execution admission absent")
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot["paths"]["parent"] and snapshot["paths"]["state"], "remote bootstrap namespace absent")
    need(snapshot.get("parent_mode") == "0555", "remote parent mode drift")
    need(snapshot.get("markers") == {"available": [], "consumed": []}, "markers exist before bootstrap audit")
    need(snapshot.get("latest_state", {}).get("state") == "BOOTSTRAP_CREATED_UNAUDITED", "bootstrap state drift")
    need(not snapshot.get("conflicting_processes"), "conflicting process during bootstrap audit")
    manifest_entries = validate_remote_manifest(snapshot, REMOTE_PARENT)
    parent = tree_index(snapshot, "parent_tree")
    receipt_path = str(REMOTE_PARENT / "BOOTSTRAP_RECEIPT.json")
    need(receipt_path in parent, "remote bootstrap receipt absent")
    receipt = ssh_python(
        "import json,pathlib,sys; print(pathlib.Path(sys.argv[1]).read_text())",
        [receipt_path],
        root=True,
    )
    # ssh_python parsed the receipt itself because the file contains one JSON object.
    need(receipt.get("verdict") == "M3_V8R1_BOOTSTRAP_CREATED_UNAUDITED", "remote bootstrap producer verdict drift")
    need(receipt.get("markers_created") == 0 and receipt.get("cargo_invocations") == 0 and receipt.get("subject_executions") == 0, "bootstrap execution ledger drift")
    need(receipt.get("source_closure", {}).get("files", 0) >= 500, "source closure unexpectedly small")
    l11_receipt_path = REMOTE_PARENT / "bootstrap-v1/inputs/l11-installed.json"
    l11 = ssh_python("import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_text())", [str(l11_receipt_path)], root=True)
    need(l11.get("artifact_sha256") == EXPECTED_L11_SHA and l11.get("runtime_authority") is False, "experiment L1.1 receipt drift")
    receipt_out = {
        "schema": "lay.m3-v8r1-bootstrap-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R1_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED",
        "local_controller_sha256": admission["local_controller_sha256"],
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "auditor_sha256": admission["auditor_sha256"],
        "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
        "remote_bootstrap_receipt_sha256": parent[receipt_path]["sha256"],
        "manifest_entries": manifest_entries,
        "source_files": receipt["source_closure"]["files"],
        "source_bytes": receipt["source_closure"]["bytes"],
        "markers_expected": 2,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "live_projection": snapshot,
    }
    return publish_tree(BOOTSTRAP_AUDIT_ROOT, "BOOTSTRAP_AUDIT.json", receipt_out)


def inspect_elf(path: pathlib.Path) -> dict[str, Any]:
    data = path.read_bytes()
    need(data[:4] == b"\x7fELF" and data[4] == 2 and data[5] == 1, "candidate is not ELF64 little-endian")
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, 0)
    elf_type = header[1]
    section_offset, program_offset = header[6], header[5]
    section_entry_size, section_count, names_index = header[11], header[12], header[13]
    program_entry_size, program_count = header[9], header[10]
    sections = []
    raw_sections = []
    for index in range(section_count):
        row = struct.unpack_from("<IIQQQQIIQQ", data, section_offset + index * section_entry_size)
        raw_sections.append(row)
    names_row = raw_sections[names_index]
    names = data[names_row[4] : names_row[4] + names_row[5]]
    for row in raw_sections:
        start = row[0]
        end = names.find(b"\0", start)
        name = names[start:end].decode(errors="replace")
        sections.append({"name": name, "address": row[3], "offset": row[4], "size": row[5]})
    programs = []
    for index in range(program_count):
        row = struct.unpack_from("<IIQQQQQQ", data, program_offset + index * program_entry_size)
        programs.append({"type": row[0], "flags": row[1], "offset": row[2], "vaddr": row[3], "filesz": row[5], "memsz": row[6]})
    by_name = {row["name"]: row for row in sections}
    for required in (".text", ".symtab", ".strtab", ".debug_info", ".debug_line"):
        need(required in by_name and by_name[required]["size"] > 0, f"ELF section absent: {required}")
    text = by_name[".text"]
    executable_loads = [row for row in programs if row["type"] == 1 and row["flags"] & 1]
    need(any(row["vaddr"] <= text["address"] and text["address"] + text["size"] <= row["vaddr"] + row["memsz"] for row in executable_loads), ".text is outside executable PT_LOAD")
    readelf = run(["/usr/bin/readelf", "-n", str(path)]).stdout.decode(errors="replace")
    match = re.search(r"Build ID:\s*([0-9a-f]+)", readelf)
    need(match is not None, "ELF Build ID absent")
    symbols = run(["/usr/bin/nm", "-C", str(path)], timeout=600).stdout.decode(errors="replace")
    need("m3_end_to_end_physical_proof" in symbols and "m3_end_to_end_pss_helper" in symbols, "V8 test symbols absent")
    return {
        "elf_type": elf_type,
        "et_dyn": elf_type == 3,
        "build_id": match.group(1),
        "text": text,
        "text_sha256": sha256_bytes(data[text["offset"] : text["offset"] + text["size"]]),
        "symtab_present": True,
        "dwarf_info_present": True,
        "dwarf_line_present": True,
        "text_in_executable_load": True,
        "v8_symbols_present": True,
    }


def build_audit() -> dict[str, Any]:
    fixed_local()
    bootstrap = load_json(BOOTSTRAP_AUDIT_RECEIPT)
    need(bootstrap.get("verdict") == "M3_V8R1_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED", "bootstrap audit absent")
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot.get("latest_state", {}).get("state") == "BUILD_CREATED_UNAUDITED", "remote build state drift")
    need(snapshot.get("markers") == {"available": ["e2e.available"], "consumed": ["build.consumed-before-exec"]}, "post-build marker state drift")
    need(not snapshot.get("conflicting_processes"), "owned or conflicting process active after build")
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v8r1-build-audit-"))
    try:
        remote_build = temporary / "REMOTE_BUILD"
        copy_remote(REMOTE_PARENT / "build-v1", remote_build)
        manifest_entries = verify_manifest(remote_build)
        provenance = load_json(remote_build / "BUILD_PROVENANCE.json")
        need(provenance.get("verdict") == "M3_V8R1_BUILD_CREATED_UNAUDITED", "build producer verdict drift")
        expected_tail = ["test", "--offline", "--locked", "--release", "--lib", "--no-run", "m3_v8"]
        command = provenance.get("build", {}).get("command", [])
        need(command[-len(expected_tail) :] == expected_tail, "Cargo argv drift")
        environment = provenance.get("build", {}).get("environment", {})
        expected_environment = {
            "CARGO_BUILD_JOBS": "20",
            "CARGO_INCREMENTAL": "0",
            "CARGO_NET_OFFLINE": "true",
            "CARGO_PROFILE_RELEASE_DEBUG": "2",
            "CARGO_PROFILE_RELEASE_STRIP": "none",
            "RUSTFLAGS": "",
        }
        for key, expected in expected_environment.items():
            need(environment.get(key) == expected, f"build environment drift: {key}")
        need(provenance.get("build", {}).get("cargo_invocations") == 1, "Cargo invocation count drift")
        need(provenance.get("build", {}).get("exit_code") == 0, "Cargo did not succeed")
        elf = remote_build / "m3-v8r1-test-elf"
        elf_row = file_row(elf)
        need(elf_row["mode"] == "0444" or elf_row["mode"] == "0555", "sealed ELF mode drift")
        need(elf_row["sha256"] == provenance.get("executable", {}).get("sha256"), "ELF SHA drift")
        audit = inspect_elf(elf)
        need(audit["et_dyn"], "V8 test ELF is not ET_DYN")
        need(audit["build_id"] == provenance.get("executable", {}).get("build_id"), "ELF Build ID drift")
        receipt = {
            "schema": "lay.m3-v8r1-build-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED",
            "local_controller_sha256": bootstrap["local_controller_sha256"],
            "remote_controller_sha256": bootstrap["remote_controller_sha256"],
            "auditor_sha256": bootstrap["auditor_sha256"],
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
            "build_provenance_sha256": sha256_file(remote_build / "BUILD_PROVENANCE.json"),
            "manifest_entries": manifest_entries,
            "elf_sha256": elf_row["sha256"],
            "elf_size_bytes": elf_row["size_bytes"],
            "elf": audit,
            "source_sha256": provenance["source"]["v13_typed_peak_sha256"],
            "cargo_invocations": 1,
            "rustc_compilations": 1,
            "elf_executed": False,
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
            "live_projection": snapshot,
        }
        return publish_tree(BUILD_AUDIT_ROOT, "BUILD_AUDIT.json", receipt, {"REMOTE_BUILD": remote_build})
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


CPU_IDLE = r'''
import json,pathlib,time
def row():
 out={}
 for line in pathlib.Path('/proc/stat').read_text().splitlines():
  fields=line.split()
  if not fields or not fields[0].startswith('cpu'): continue
  values=[int(x) for x in fields[1:]]; total=sum(values); idle=values[3]+(values[4] if len(values)>4 else 0); out[fields[0]]=(total,idle)
 return out
a=row(); time.sleep(5); b=row(); ratios={k:(b[k][1]-a[k][1])/(b[k][0]-a[k][0]) for k in a if k in b and b[k][0]>a[k][0]}
print(json.dumps({'idle_ratios':ratios,'cpu0_idle_ratio':ratios.get('cpu0'),'all_idle_ratio':ratios.get('cpu')},sort_keys=True))
'''


def quiet_admission() -> dict[str, Any]:
    fixed_local()
    build = load_json(BUILD_AUDIT_RECEIPT)
    need(build.get("verdict") == "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED", "build audit absent")
    before = remote_snapshot()
    validate_host(before)
    need(before.get("latest_state", {}).get("state") == "BUILD_CREATED_UNAUDITED", "quiet preflight state drift")
    need(before.get("markers") == {"available": ["e2e.available"], "consumed": ["build.consumed-before-exec"]}, "quiet preflight markers drift")
    need(not before.get("conflicting_processes"), "conflicting process before E2E")
    idle = ssh_python(CPU_IDLE, root=False, timeout=30)
    need(float(idle.get("cpu0_idle_ratio") or 0.0) >= 0.95, "CPU0 was not at least 95% idle")
    need(float(idle.get("all_idle_ratio") or 0.0) >= 0.90, "host was not at least 90% idle")
    after = remote_snapshot()
    validate_host(after)
    need(not after.get("conflicting_processes"), "conflicting process appeared during quiet preflight")
    need(before.get("thermal") == after.get("thermal"), "thermal throttle counter changed during quiet preflight")
    need(before.get("runtime_projection") == after.get("runtime_projection"), "remote runtime changed during quiet preflight")
    receipt = {
        "schema": "lay.m3-v8r1-quiet-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R1_QUIET_HOST_E2E_ADMITTED",
        "local_controller_sha256": build["local_controller_sha256"],
        "remote_controller_sha256": build["remote_controller_sha256"],
        "auditor_sha256": build["auditor_sha256"],
        "build_audit_sha256": sha256_file(BUILD_AUDIT_RECEIPT),
        "elf_sha256": build["elf_sha256"],
        "host_before": before,
        "host_after": after,
        "idle_observation": idle,
        "quiet_seconds": 5,
        "thermal_throttle_drift": {},
        "conflicting_processes": 0,
        "cargo_invocations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
    }
    return publish_tree(QUIET_ROOT, "QUIET_ADMISSION.json", receipt)


def integer(value: Any, default: int = -1) -> int:
    return value if isinstance(value, int) and not isinstance(value, bool) else default


def terminal_decision(subject: Mapping[str, Any], wrapper: Mapping[str, Any], build: Mapping[str, Any]) -> tuple[str, dict[str, list[str]]]:
    failures = {key: [] for key in ("provenance", "build", "semantic", "capacity", "reload_identity", "rss", "latency", "environment")}
    if wrapper.get("outputs_complete") is not True or subject.get("schema") != "lay.m3-end-to-end-test-owner.v1":
        failures["provenance"].append("subject observation incomplete")
    if wrapper.get("verdict") != "M3_V8R1_E2E_CREATED_UNAUDITED" or wrapper.get("controller_error") is not None:
        failures["provenance"].append("producer wrapper is incomplete or controller-failed")
    positive_subject = subject.get("verdict") == "M3_END_TO_END_TEST_OWNER_PASS"
    if (positive_subject and wrapper.get("exit_code") != 0) or (
        subject.get("verdict") not in (None, "M3_END_TO_END_TEST_OWNER_PASS")
        and wrapper.get("exit_code") in (None, 0)
    ):
        failures["provenance"].append("subject verdict and exit status disagree")
    source = subject.get("source", {})
    if source.get("v13_sha256") != EXPECTED_V13_SHA or source.get("v7_sha256") != EXPECTED_V7_SHA:
        failures["provenance"].append("fixed input identity drift")
    if source.get("productive_v90_sha256") != EXPECTED_PRODUCTIVE_SHA or source.get("l11_sha256") != EXPECTED_L11_SHA:
        failures["provenance"].append("actual-owner package tuple drift")
    if source.get("test_elf_sha256") != build.get("elf_sha256"):
        failures["provenance"].append("test ELF identity drift")
    if build.get("verdict") != "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED":
        failures["build"].append("build audit did not admit E2E")
    fixed = subject.get("fixed_proof", {})
    semantic = fixed.get("semantic", {})
    for field in SEMANTIC_FIELDS:
        if integer(semantic.get(field)) != 0:
            failures["semantic"].append(f"{field} is nonzero or unknown")
    if integer(fixed.get("empty_lane_mismatches")) != 0:
        failures["semantic"].append("empty-lane parity mismatch")
    if integer(semantic.get("capacity_failures")) != 0 or integer(semantic.get("unresolved")) != 0:
        failures["capacity"].append("capacity or unresolved count nonzero")
    if integer(fixed.get("maximum_query_scratch_bytes"), 2**63) > 512 * 1024:
        failures["capacity"].append("maximum query scratch exceeds 512 KiB")
    reload = subject.get("reload", {})
    expected_reload = {
        "reader_identity_mismatches": 0,
        "mixed_generation_observations": 0,
        "stale_a_commits": 0,
        "stale_a_cancellations": 1,
        "current_b_commits": 1,
        "failed_build_publications": 0,
        "rollback_identity_mismatches": 0,
        "typed_materializations": 2,
        "per_request_typed_materializations": 0,
    }
    for field, expected in expected_reload.items():
        if integer(reload.get(field)) != expected:
            failures["reload_identity"].append(f"reload field drift: {field}")
    if reload.get("held_a_survived_publication") is not True:
        failures["reload_identity"].append("held generation A did not survive")
    pss = subject.get("pss", {})
    if integer(pss.get("aggregate_delta_pss_kib"), 2**63) > 40 * 1024:
        failures["rss"].append("aggregate two-process PSS delta exceeds 40 MiB")
    if integer(pss.get("typed_owned_bytes_per_process")) != 3_689_628:
        failures["rss"].append("typed payload byte count drift")
    if integer(pss.get("sidecar_bytes"), 2**63) > 32 * 1024 * 1024 or integer(pss.get("helper_failures")) != 0:
        failures["rss"].append("sidecar or PSS helper gate failed")
    if integer(fixed.get("maximum_round_search_p99_us"), 2**63) > 3_000:
        failures["latency"].append("maximum round search p99 exceeds 3000 us")
    if integer(fixed.get("maximum_round_total_material_p99_us"), 2**63) > 5_000:
        failures["latency"].append("maximum round total p99 exceeds 5000 us")
    if integer(fixed.get("cases")) != 382 or integer(fixed.get("measured_rounds")) != 4 or integer(fixed.get("measured_samples")) != 1_528:
        failures["provenance"].append("request denominator drift")
    if fixed.get("schedule") != ["FORWARD", "REVERSED", "FORWARD", "REVERSED"]:
        failures["provenance"].append("schedule drift")
    if integer(fixed.get("cpu")) != 0 or integer(fixed.get("cpu_mismatches")) != 0 or integer(fixed.get("warmup_cpu_mismatches")) != 0:
        failures["environment"].append("CPU0 execution closure failed")
    if wrapper.get("thermal_throttle_drift") not in ({}, None):
        failures["environment"].append("thermal throttle counters changed")
    if wrapper.get("subject_executions") != 1 or wrapper.get("cargo_invocations") != 0 or wrapper.get("perf_record_invocations") != 0 or wrapper.get("perf_stat_invocations") != 0:
        failures["provenance"].append("E2E execution ledger drift")
    gates = subject.get("gates", {})
    expected_gate_fields = ("semantic", "capacity", "reload_identity", "rss", "latency", "environment")
    if positive_subject and any(gates.get(field) is not True for field in expected_gate_fields):
        failures["provenance"].append("positive subject verdict has an open gate")
    claim = subject.get("claim_boundary", {})
    if (
        claim.get("test_only_generation_owner") is not True
        or claim.get("production_authority_admitted") is not False
        or claim.get("runtime_reload_edit_admitted") is not False
        or subject.get("runtime_authority_changed") is not False
        or subject.get("production_activation_admitted") is not False
    ):
        failures["provenance"].append("scientific claim boundary drift")
    priority = (
        ("BLOCKED_PROVENANCE", "provenance"),
        ("BLOCKED_BUILD", "build"),
        ("BLOCKED_SEMANTIC", "semantic"),
        ("BLOCKED_CAPACITY", "capacity"),
        ("BLOCKED_RELOAD_IDENTITY", "reload_identity"),
        ("BLOCKED_RSS", "rss"),
        ("BLOCKED_LATENCY", "latency"),
        ("BLOCKED_ENVIRONMENT", "environment"),
    )
    for verdict, key in priority:
        if failures[key]:
            return verdict, failures
    return "M3_END_TO_END_TEST_OWNER_PASS", failures


def terminal_audit() -> dict[str, Any]:
    fixed_local()
    admission = load_json(ADMISSION_RECEIPT)
    build = load_json(BUILD_AUDIT_RECEIPT)
    quiet = load_json(QUIET_RECEIPT)
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot.get("markers") == {"available": [], "consumed": ["build.consumed-before-exec", "e2e.consumed-before-exec"]}, "terminal marker state drift")
    need(not snapshot.get("conflicting_processes"), "owned or conflicting process active at terminal audit")
    remote_root = REMOTE_PARENT / "e2e-v1"
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v8r1-terminal-"))
    try:
        remote_evidence = temporary / "REMOTE_E2E"
        copy_remote(remote_root, remote_evidence)
        manifest_entries = verify_manifest(remote_evidence)
        wrapper = load_json(remote_evidence / "E2E_WRAPPER.json")
        subject = wrapper.get("subject_receipt") if isinstance(wrapper.get("subject_receipt"), dict) else {}
        verdict, failures = terminal_decision(subject, wrapper, build)
        positive = verdict == "M3_END_TO_END_TEST_OWNER_PASS"
        need((subject.get("verdict") == verdict) or (not positive and subject.get("verdict") in {verdict, None}), "subject and independent verdict disagree")
        runtime_after = local_runtime_snapshot()
        need(runtime_after == admission.get("local_runtime_before"), "local runtime authority changed")
        need(snapshot.get("runtime_projection") == admission.get("remote_runtime_before"), "remote runtime authority changed")
        receipt = {
            "schema": "lay.m3-v8r1-terminal-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": verdict,
            "positive_verdict": "M3_END_TO_END_TEST_OWNER_PASS",
            "failure_priority": ["provenance", "build", "semantic", "capacity", "reload_identity", "rss", "latency", "environment"],
            "failures": failures,
            "local_controller_sha256": build["local_controller_sha256"],
            "remote_controller_sha256": build["remote_controller_sha256"],
            "auditor_sha256": build["auditor_sha256"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
            "build_audit_sha256": sha256_file(BUILD_AUDIT_RECEIPT),
            "quiet_admission_sha256": sha256_file(QUIET_RECEIPT),
            "e2e_wrapper_sha256": sha256_file(remote_evidence / "E2E_WRAPPER.json"),
            "remote_manifest_entries": manifest_entries,
            "scientific_receipt": subject,
            "summary": {
                "maximum_round_search_p99_us": subject.get("fixed_proof", {}).get("maximum_round_search_p99_us"),
                "maximum_round_total_material_p99_us": subject.get("fixed_proof", {}).get("maximum_round_total_material_p99_us"),
                "maximum_query_scratch_bytes": subject.get("fixed_proof", {}).get("maximum_query_scratch_bytes"),
                "aggregate_two_process_pss_delta_kib": subject.get("pss", {}).get("aggregate_delta_pss_kib"),
                "typed_owned_bytes_per_process": subject.get("pss", {}).get("typed_owned_bytes_per_process"),
                "measured_samples": subject.get("fixed_proof", {}).get("measured_samples"),
            },
            "markers_created": 2,
            "markers_consumed": 2,
            "cargo_invocations": 1,
            "rustc_compilations": 1,
            "subject_executions": 1,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "installed_package_changed": False,
            "runtime_authority_changed": False,
            "production_authority_admitted": False,
            "next_if_pass": "separate production authority decision paper",
            "live_projection": snapshot,
        }
        return publish_tree(TERMINAL_ROOT, "TERMINAL_AUDIT.json", receipt, {"REMOTE_E2E": remote_evidence})
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def self_check() -> dict[str, Any]:
    need(ACTIONS == ("self-check", "live-admission", "bootstrap", "build", "quiet", "terminal", "status"), "auditor registry drift")
    need(SEMANTIC_FIELDS[-1] == "semantic_total" and len(SEMANTIC_FIELDS) == 10, "semantic schema drift")
    need(len({ADMISSION_ROOT, BOOTSTRAP_AUDIT_ROOT, BUILD_AUDIT_ROOT, QUIET_ROOT, TERMINAL_ROOT}) == 5, "audit destination collision")
    need("/home/e/.cargo/bin" not in REMOTE_SNAPSHOT and "subprocess" not in REMOTE_SNAPSHOT, "host snapshot reaches the toolchain")
    need("/home/e/.cargo/bin/cargo" in REMOTE_TOOLCHAIN and "HOME':'/home/e'" in REMOTE_TOOLCHAIN, "controlled toolchain snapshot drift")
    synthetic_subject = {
        "schema": "lay.m3-end-to-end-test-owner.v1",
        "verdict": "M3_END_TO_END_TEST_OWNER_PASS",
        "source": {"v13_sha256": EXPECTED_V13_SHA, "v7_sha256": EXPECTED_V7_SHA, "productive_v90_sha256": EXPECTED_PRODUCTIVE_SHA, "l11_sha256": EXPECTED_L11_SHA, "test_elf_sha256": "a" * 64},
        "fixed_proof": {
            "cases": 382, "measured_rounds": 4, "measured_samples": 1528,
            "schedule": ["FORWARD", "REVERSED", "FORWARD", "REVERSED"],
            "semantic": {**{key: 0 for key in SEMANTIC_FIELDS}, "capacity_failures": 0, "unresolved": 0},
            "empty_lane_mismatches": 0, "maximum_query_scratch_bytes": 1,
            "maximum_round_search_p99_us": 1, "maximum_round_total_material_p99_us": 1,
            "cpu": 0, "cpu_mismatches": 0, "warmup_cpu_mismatches": 0,
        },
        "reload": {
            "reader_identity_mismatches": 0, "mixed_generation_observations": 0,
            "stale_a_commits": 0, "stale_a_cancellations": 1, "current_b_commits": 1,
            "failed_build_publications": 0, "rollback_identity_mismatches": 0,
            "typed_materializations": 2, "per_request_typed_materializations": 0,
            "held_a_survived_publication": True,
        },
        "pss": {"aggregate_delta_pss_kib": 1, "typed_owned_bytes_per_process": 3_689_628, "sidecar_bytes": 1, "helper_failures": 0},
        "gates": {"semantic": True, "capacity": True, "reload_identity": True, "rss": True, "latency": True, "environment": True},
        "claim_boundary": {"test_only_generation_owner": True, "production_authority_admitted": False, "runtime_reload_edit_admitted": False},
        "runtime_authority_changed": False,
        "production_activation_admitted": False,
    }
    wrapper = {
        "verdict": "M3_V8R1_E2E_CREATED_UNAUDITED",
        "controller_error": None,
        "exit_code": 0,
        "outputs_complete": True,
        "thermal_throttle_drift": {},
        "subject_executions": 1,
        "cargo_invocations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
    }
    build = {"verdict": "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED", "elf_sha256": "a" * 64}
    verdict, failures = terminal_decision(synthetic_subject, wrapper, build)
    need(verdict == "M3_END_TO_END_TEST_OWNER_PASS" and not any(failures.values()), "positive dispatch model failed")
    synthetic_subject["fixed_proof"]["semantic"]["candidate_mismatches"] = 1
    synthetic_subject["fixed_proof"]["maximum_round_search_p99_us"] = 9_999
    verdict, _ = terminal_decision(synthetic_subject, wrapper, build)
    need(verdict == "BLOCKED_SEMANTIC", "failure priority model failed")
    return {
        "schema": "lay.m3-v8r1-independent-auditor-self-check.v1",
        "verdict": "M3_V8R1_INDEPENDENT_AUDITOR_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "auditor_sha256": sha256_file(AUDITOR),
        "actions": list(ACTIONS),
        "positive_dispatch": "PASS",
        "failure_priority_dispatch": "PASS",
        "remote_writes": 0,
        "scientific_actions": 0,
    }


def status() -> dict[str, Any]:
    return {
        "schema": "lay.m3-v8r1-auditor-status.v1",
        "verdict": "M3_V8R1_AUDITOR_STATUS",
        "receipts": {
            "execution_admission": ADMISSION_RECEIPT.exists(),
            "bootstrap_audit": BOOTSTRAP_AUDIT_RECEIPT.exists(),
            "build_audit": BUILD_AUDIT_RECEIPT.exists(),
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
        elif args.action == "build":
            value = build_audit()
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
            "schema": "lay.m3-v8r1-auditor-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
