#!/usr/bin/env python3
"""Independent auditor and sole publisher for M2R1 execution admission V1."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
import pathlib
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2r1-v1-20260827"
TRANSACTION_ID = "2dae728a39aecd422995828674d12e311ab6362ebab4013c4f2520b3f6933c5f"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
AUDITOR_QUIET_SECONDS = 5

AUDITOR = pathlib.Path(__file__).resolve()
PRODUCER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-live-preflight.py"
EXPECTED_PRODUCER_SHA256 = "b0f117b7f416f2c111e22c6e2b0809693e1769b8e0d46116d271d1f69bc05566"
IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_IMPLEMENTATION_SELF_CHECK_V1_2026-08-27.json"
)
M2R1_IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_IMPLEMENTATION_SELF_CHECK_V1_2026-08-27.json"
)
LIVE_RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_LIVE_PREFLIGHT_V1_2026-08-27"
)
AUDIT_RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_AUDIT_V1_2026-08-27"
)
EXECUTION_ADMISSION = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_2026-08-27.json"
)
M2_JOURNAL = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_JOURNAL_V1_2026-08-27"
)

M2_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-v1.py"
M2_REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-remote.py"
M2_BOOTSTRAP_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-bootstrap-audit.py"
M2_BUILD_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-build-audit.py"
M2_TERMINAL_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-terminal-audit.py"
M2_FRAGMENT = ROOT / "scripts/lay_v10_e1_traversal_w1_fused_minimum_m2r1_test_module.rs.inc"

EXPECTED_SOURCES = {
    "controller_sha256": (M2_CONTROLLER, "d3540936c19e45230a70c805b77cf6dab024636457ea0577281a59be2e128106"),
    "remote_controller_sha256": (M2_REMOTE_CONTROLLER, "26bfbbf2a9626b36bad549d6827d7442d1d1b5db7ced0f65da41c099a793de09"),
    "bootstrap_auditor_sha256": (M2_BOOTSTRAP_AUDITOR, "c0a7221eeeef460db174682dafad16a7765a970a5623e3641c0657de4bfce84a"),
    "build_auditor_sha256": (M2_BUILD_AUDITOR, "0f943646c0e13c6fe53c406aa542882d68ab9dbe447dc3c660719110ccae6762"),
    "terminal_auditor_sha256": (M2_TERMINAL_AUDITOR, "37b6d27eaa719ebe6347a97303602fc30b4d4ec7078a0bf49b3ce424037e23fe"),
    "fragment_sha256": (M2_FRAGMENT, "a6ea388d5d76f8223511fd4822cff2df9fd0c3394fc200f4fc52db956522ce5b"),
}

REMOTE_EXPECTED = {
    "hostname": "e-MEGA-MINI-M1-13th",
    "machine_id_sha256": "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441",
    "kernel": "6.8.0-124-generic",
    "online": "0-19",
    "core": "0-11",
    "atom": "12-19",
    "cargo": "cargo 1.97.1 (c980f4866 2026-06-30)",
    "rustc_tokens": (
        "release: 1.97.1",
        "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452",
        "host: x86_64-unknown-linux-gnu",
        "LLVM version: 22.1.6",
    ),
    "perf_version": "perf version 6.8.12",
    "perf_sha256": "2d0953085bf720a25efbe24f853e97d27b1f12f18a398255ff82cbafde254dad",
    "perf_event_max_sample_rate": "8000",
    "perf_event_paranoid": "4",
}

REMOTE_INPUTS = {
    "package": ("cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b", 140_556_462, "0444"),
    "sidecar": ("a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd", 3_689_884, "0444"),
    "v7": ("33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4", 1_606_189, "0444"),
    "schedule": ("2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78", 174_941, "0444"),
    "cargo_toml": ("90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b", 2_399, "0444"),
    "cargo_lock": ("e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1", 70_770, "0444"),
    "cargo_guard": ("a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe", 2_534, "0555"),
}


class AuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
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


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def file_row(path: pathlib.Path) -> dict[str, Any]:
    return {"path": str(path), "sha256": sha256_file(path), "size_bytes": path.stat().st_size, "mode": mode_string(path)}


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


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def atomic_publish_file(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    need(not path.exists(), f"publication target exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    stage = path.parent / f".{path.name}.stage-{os.getpid()}-{time.time_ns()}"
    try:
        write_new(stage, value, mode)
        os.link(stage, path)
        fsync_directory(path.parent)
    finally:
        stage.unlink(missing_ok=True)


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(p for p in root.rglob("*") if p.is_file() and p.name != "SHA256SUMS"):
        rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish_tree(stage: pathlib.Path, destination: pathlib.Path) -> None:
    need(not destination.exists(), f"evidence tree exists: {destination}")
    write_manifest(stage)
    seal_tree(stage)
    os.rename(stage, destination)
    fsync_directory(destination.parent)


def verify_manifest(root: pathlib.Path) -> dict[str, str]:
    need(root.is_dir() and mode_string(root) == "0555", f"sealed evidence tree absent: {root}")
    manifest = root / "SHA256SUMS"
    need(manifest.is_file() and mode_string(manifest) == "0444", "evidence manifest absent")
    rows = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        need(path.is_file() and mode_string(path) == "0444", f"manifest file drift: {relative}")
        need(sha256_file(path) == digest, f"manifest SHA drift: {relative}")
        rows[relative] = digest
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"}
    need(set(rows) == actual, "manifest inventory drift")
    return rows


def runtime_snapshot() -> dict[str, Any]:
    m2 = json.loads(M2R1_IMPLEMENTATION_RECEIPT.read_text())
    expected = m2["runtime_after"]["installed_lay_hashes"]
    rows = {}
    for name, identity in sorted(expected.items()):
        path = pathlib.Path(identity["target"])
        need(path.is_file(), f"runtime target absent: {path}")
        digest = sha256_file(path)
        need(digest == identity["sha256"], f"runtime SHA drift: {name}")
        rows[name] = {"target": str(path), "sha256": digest}
    return {"installed_lay_hashes": rows}


REMOTE_AUDIT_PROGRAM = r'''
import hashlib,json,os,pathlib,stat,subprocess,time
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
 return h.hexdigest()
def cmd(argv,env=None): return subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,env=env,text=True)
def mode(path): return f'{stat.S_IMODE(path.stat().st_mode):04o}'
task='slice8b-v10-e1-traversal-w1-fused-minimum-m2r1-v1-20260827'; tx='2dae728a39aecd422995828674d12e311ab6362ebab4013c4f2520b3f6933c5f'
parent=pathlib.Path('/home/e/.local/share/lay/provenance')/task; state=pathlib.Path('/home/e/.local/state/lay')/task
cache=pathlib.Path('/home/e/.cache')/('lay-m2r1-'+tx); probe=pathlib.Path('/home/e/.cache')/('lay-m2r1-execution-admission-v1-'+tx)
ps=cmd(['/usr/bin/ps','-eo','user=,pid=,ppid=,etimes=,psr=,stat=,comm=,args=']).stdout.splitlines(); table='\n'.join(ps)+'\n'
comm_block={'perf','cargo','rustc','rustc_driver'}
tokens=('leg01-v13-all-live-inputs','nando-motif-shape-census','diagnostic-test-elf','v10_m2_fused_minimum','lay-v10-e1-traversal','/target/release/deps/lay-','perf record','perf stat','cargo build','cargo test','cargo bench','cargo check')
conflicts=[]
for line in ps:
 fields=line.strip().split(None,7)
 if len(fields)<8: continue
 user,pid,ppid,etimes,psr,pstate,comm,args=fields; low=args.lower(); reasons=[]
 if comm.lower() in comm_block: reasons.append('blocked-comm')
 reasons.extend('token:'+token for token in tokens if token in low)
 if reasons: conflicts.append({'user':user,'pid':int(pid),'ppid':int(ppid),'etimes':int(etimes),'cpu':psr,'state':pstate,'comm':comm,'argv':args,'reasons':sorted(set(reasons))})
perf_fds=[]
for proc in pathlib.Path('/proc').iterdir():
 if not proc.name.isdigit(): continue
 try:
  comm=(proc/'comm').read_text(errors='replace').strip(); argv=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace').strip()
  for fd in (proc/'fd').iterdir():
   try: target=os.readlink(fd)
   except OSError: continue
   if 'perf_event' in target: perf_fds.append({'pid':int(proc.name),'comm':comm,'argv':argv,'fd':fd.name,'target':target})
 except (OSError,PermissionError): pass
env={'HOME':'/home/e','LANG':'C.UTF-8','LC_ALL':'C.UTF-8','PATH':'/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin'}
def as_e(argv): return cmd(['/usr/bin/sudo','-n','-u','e','/usr/bin/env',*[f'{k}={v}' for k,v in env.items()],*argv])
inputs={
 'package':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/LAY-L2-RU-FULL-v13.bin',
 'sidecar':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa',
 'v7':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json',
 'schedule':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1/query-schedule.json',
 'cargo_toml':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.toml',
 'cargo_lock':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.lock',
 'cargo_guard':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/controller/cargo-guard.sh'}
rows={}
for name,raw in inputs.items():
 p=pathlib.Path(raw); rows[name]={'path':raw,'exists':p.is_file(),'size_bytes':p.stat().st_size if p.is_file() else None,'mode':mode(p) if p.exists() else None,'sha256':sha(p) if p.is_file() else None}
perf=pathlib.Path('/usr/bin/perf'); cargo=as_e(['cargo','-V']); rustc=as_e(['rustc','-Vv']); perf_v=cmd(['/usr/bin/perf','version'])
print(json.dumps({'schema':'lay.m2r1-execution-admission-live-snapshot.v2','hostname':os.uname().nodename,'machine_id_sha256':sha('/etc/machine-id'),'kernel':os.uname().release,'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),'e_uid':int(cmd(['/usr/bin/id','-u','e']).stdout.strip()),'cargo':cargo.stdout.strip(),'cargo_rc':cargo.returncode,'rustc':rustc.stdout.strip(),'rustc_rc':rustc.returncode,'perf_version':perf_v.stdout.strip(),'perf_version_rc':perf_v.returncode,'perf_sha256':sha(perf),'perf_event_max_sample_rate':pathlib.Path('/proc/sys/kernel/perf_event_max_sample_rate').read_text().strip(),'perf_event_paranoid':pathlib.Path('/proc/sys/kernel/perf_event_paranoid').read_text().strip(),'m2r1_paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},'probe_exists':probe.exists(),'conflicting_processes':conflicts,'perf_event_fds':perf_fds,'process_table_sha256':hashlib.sha256(table.encode()).hexdigest(),'process_count':len(ps),'inputs':rows,'snapshot_monotonic_ns':time.monotonic_ns()},sort_keys=True))
'''


def run(command: Sequence[str], *, input_bytes: bytes | None = None, timeout: float = 180.0) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if result.returncode != 0:
        raise AuditError(f"command failed ({result.returncode}): {command!r}\n" + result.stderr[-4000:].decode(errors="replace"))
    return result


def remote_snapshot() -> dict[str, Any]:
    command = ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", REMOTE, "/usr/bin/sudo", "-n", "/usr/bin/python3", "-"]
    result = run(command, input_bytes=REMOTE_AUDIT_PROGRAM.encode())
    lines = result.stdout.decode().strip().splitlines()
    need(lines, "empty remote audit response")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote audit response is not an object")
    return value


def validate_snapshot(value: Mapping[str, Any]) -> None:
    need(value.get("schema") == "lay.m2r1-execution-admission-live-snapshot.v2", "snapshot schema drift")
    for key in ("hostname", "machine_id_sha256", "kernel", "online", "core", "atom", "cargo", "perf_version", "perf_sha256", "perf_event_max_sample_rate", "perf_event_paranoid"):
        need(value.get(key) == REMOTE_EXPECTED[key], f"remote snapshot drift: {key}")
    need(value.get("e_uid") == 1000, "remote subject UID drift")
    need(value.get("cargo_rc") == 0 and value.get("rustc_rc") == 0 and value.get("perf_version_rc") == 0, "tool query failure")
    need(all(token in str(value.get("rustc", "")) for token in REMOTE_EXPECTED["rustc_tokens"]), "rustc identity drift")
    need(value.get("m2r1_paths") == {"parent": False, "state": False, "cache": False}, "M2R1 remote namespace exists")
    need(value.get("probe_exists") is False, "UID probe remains present")
    need(not value.get("conflicting_processes"), "conflicting performance experiment is active")
    need(not value.get("perf_event_fds"), "open perf_event descriptor exists")
    inputs = value.get("inputs")
    need(isinstance(inputs, Mapping) and set(inputs) == set(REMOTE_INPUTS), "remote input inventory drift")
    for name, expected in REMOTE_INPUTS.items():
        row = inputs[name]
        need(row.get("exists") is True and (row.get("sha256"), row.get("size_bytes"), row.get("mode")) == expected, f"remote input drift: {name}")


def verify_implementation() -> dict[str, Any]:
    need(sha256_file(PRODUCER) == EXPECTED_PRODUCER_SHA256, "live producer SHA drift")
    need(IMPLEMENTATION_RECEIPT.is_file() and mode_string(IMPLEMENTATION_RECEIPT) == "0444", "implementation receipt absent")
    value = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(value.get("verdict") == "M2R1_ADMISSION_CONTROLLER_VERIFIED_UNRUN", "implementation verdict drift")
    need(value.get("producer_sha256") == EXPECTED_PRODUCER_SHA256, "implementation producer identity drift")
    need(value.get("auditor_sha256") == sha256_file(AUDITOR), "implementation auditor identity drift")
    for key, (path, digest) in EXPECTED_SOURCES.items():
        need(path.is_file() and sha256_file(path) == digest, f"sealed M2R1 source drift: {key}")
    return value


def verify_live_result() -> tuple[dict[str, Any], dict[str, str]]:
    rows = verify_manifest(LIVE_RESULT)
    expected_files = {"SNAPSHOT-1.json", "SNAPSHOT-2.json", "UID_PROOF.json", "RUNTIME_BEFORE.json", "RUNTIME_AFTER.json", "M2R1_LIVE_PREFLIGHT_RECEIPT.json"}
    need(set(rows) == expected_files, "live preflight evidence inventory drift")
    receipt = json.loads((LIVE_RESULT / "M2R1_LIVE_PREFLIGHT_RECEIPT.json").read_text())
    need(receipt.get("verdict") == "M2R1_LIVE_PREFLIGHT_PASS_UNADMITTED" and receipt.get("safe_to_execute") is False, "live preflight verdict drift")
    need(receipt.get("task_id") == TASK_ID and receipt.get("transaction_id") == TRANSACTION_ID, "live preflight namespace drift")
    need(receipt.get("producer_sha256") == EXPECTED_PRODUCER_SHA256 and receipt.get("auditor_sha256") == sha256_file(AUDITOR), "live preflight source identity drift")
    for name in ("SNAPSHOT-1.json", "SNAPSHOT-2.json"):
        validate_snapshot(json.loads((LIVE_RESULT / name).read_text()))
    proof = json.loads((LIVE_RESULT / "UID_PROOF.json").read_text())
    need(proof.get("verdict") == "PASS" and proof.get("uid") == 1000 and proof.get("before_absent") is True and proof.get("after_absent") is True, "UID proof drift")
    need(len(proof.get("operations", [])) == 10 and proof.get("error") is None, "UID proof incomplete")
    need(proof.get("payload_sha256") == "3b93efe386726592a0440dc8ebe8daa664912a5b9cd2da3340666c1260b2df42", "UID proof payload drift")
    before = json.loads((LIVE_RESULT / "RUNTIME_BEFORE.json").read_text())
    after = json.loads((LIVE_RESULT / "RUNTIME_AFTER.json").read_text())
    need(before == after == runtime_snapshot(), "live preflight runtime projection drift")
    return receipt, rows


def self_check() -> dict[str, Any]:
    need(PRODUCER.is_file() and sha256_file(PRODUCER) == EXPECTED_PRODUCER_SHA256, "producer source identity drift")
    producer_text = PRODUCER.read_text()
    auditor_text = AUDITOR.read_text()
    producer_tree = ast.parse(producer_text)
    auditor_tree = ast.parse(auditor_text)
    producer_admission_names = sum(
        isinstance(node, ast.Name) and node.id == "EXECUTION_ADMISSION"
        for node in ast.walk(producer_tree)
    )
    auditor_publishers = sum(
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "atomic_publish_file"
        and bool(node.args)
        and isinstance(node.args[0], ast.Name)
        and node.args[0].id == "EXECUTION_ADMISSION"
        for node in ast.walk(auditor_tree)
    )
    need(producer_admission_names == 0, "producer owns execution-admission symbol")
    need(auditor_publishers == 1, "admission publisher cardinality drift")
    for path, digest in (item for item in EXPECTED_SOURCES.values()):
        need(path.is_file() and sha256_file(path) == digest, f"sealed source drift: {path}")
    with tempfile.TemporaryDirectory(prefix="m2r1-admission-auditor-check-") as raw:
        root = pathlib.Path(raw)
        destination = root / "admission.json"
        atomic_publish_file(destination, canonical({"safe_to_execute": True}), 0o444)
        need(mode_string(destination) == "0444", "auditor publisher mode drift")
        try:
            atomic_publish_file(destination, b"overwrite\n", 0o444)
        except AuditError:
            publication_fault = "PASS"
        else:
            raise AuditError("auditor publisher overwrote existing admission")
    return {
        "schema": "lay.m2r1-execution-admission-auditor-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "PASS",
        "producer_sha256": EXPECTED_PRODUCER_SHA256,
        "auditor_sha256": sha256_file(AUDITOR),
        "producer_admission_publishers": producer_admission_names,
        "admission_publishers": auditor_publishers,
        "atomic_publication_fault": publication_fault,
        "network_access": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "cargo_compilations": 0,
        "perf_stat": 0,
        "perf_record": 0,
        "subject_executions": 0,
    }


def audit() -> dict[str, Any]:
    implementation = verify_implementation()
    need(not AUDIT_RESULT.exists() and not EXECUTION_ADMISSION.exists(), "admission audit or admission already exists")
    need(not M2_JOURNAL.exists(), "M2R1 execution journal exists before admission")
    live_receipt, manifest = verify_live_result()
    runtime_before = runtime_snapshot()
    time.sleep(AUDITOR_QUIET_SECONDS)
    snapshot = remote_snapshot()
    validate_snapshot(snapshot)
    runtime_after = runtime_snapshot()
    need(runtime_before == runtime_after, "runtime changed during admission audit")

    AUDIT_RESULT.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix=f".{AUDIT_RESULT.name}.stage-", dir=AUDIT_RESULT.parent))
    try:
        write_new(stage / "AUDITOR_SNAPSHOT.json", canonical(snapshot))
        write_new(stage / "RUNTIME_BEFORE.json", canonical(runtime_before))
        write_new(stage / "RUNTIME_AFTER.json", canonical(runtime_after))
        receipt = {
            "schema": "lay.m2r1-execution-admission-independent-audit.v2",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M2R1_EXECUTION_ADMISSION_AUDIT_PASS_READY_TO_PUBLISH",
            "producer_sha256": EXPECTED_PRODUCER_SHA256,
            "auditor_sha256": sha256_file(AUDITOR),
            "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
            "m2r1_implementation_receipt_sha256": sha256_file(M2R1_IMPLEMENTATION_RECEIPT),
            "live_preflight_receipt_sha256": sha256_file(LIVE_RESULT / "M2R1_LIVE_PREFLIGHT_RECEIPT.json"),
            "live_preflight_manifest_sha256": sha256_file(LIVE_RESULT / "SHA256SUMS"),
            "live_preflight_manifest_entries": len(manifest),
            "auditor_snapshot_sha256": sha256_bytes(canonical(snapshot)),
            "auditor_quiet_seconds": AUDITOR_QUIET_SECONDS,
            "conflicting_processes": 0,
            "open_perf_event_descriptors": 0,
            "m2r1_remote_parent_state_cache": 0,
            "execution_journal_present": False,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_compilations": 0,
            "rustc_compilations": 0,
            "perf_stat": 0,
            "perf_record": 0,
            "pmu_events": 0,
            "subject_executions": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "atomic publication of exact M2R1 execution admission only",
        }
        write_new(stage / "M2R1_EXECUTION_ADMISSION_AUDIT_RECEIPT.json", canonical(receipt))
        publish_tree(stage, AUDIT_RESULT)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise

    audit_receipt = AUDIT_RESULT / "M2R1_EXECUTION_ADMISSION_AUDIT_RECEIPT.json"
    admission = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2R1_EXECUTION_ADMITTED",
        "safe_to_execute": True,
        **{key: digest for key, (_, digest) in EXPECTED_SOURCES.items()},
        "implementation_receipt_sha256": sha256_file(M2R1_IMPLEMENTATION_RECEIPT),
        "admission_implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "live_preflight_receipt_sha256": sha256_file(LIVE_RESULT / "M2R1_LIVE_PREFLIGHT_RECEIPT.json"),
        "independent_admission_audit_receipt_sha256": sha256_file(audit_receipt),
        "producer_sha256": EXPECTED_PRODUCER_SHA256,
        "auditor_sha256": sha256_file(AUDITOR),
        "code_route_v1_contract_sha256": "4268d960359e1e6d7cfa00985cafe8a5ea8906be2077dcc824cfe354e76c3948",
        "code_route_v1_receipt_sha256": "339475c9310865b2d2cd799919f918ee37fb06ba7905ebe3f9249638b85a3623",
        "live_snapshots": 3,
        "conflicting_processes": 0,
        "open_perf_event_descriptors": 0,
        "m2r1_remote_namespace_present": False,
        "execution_journal_present": False,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_compilations": 0,
        "rustc_compilations": 0,
        "perf_stat": 0,
        "perf_record": 0,
        "pmu_events": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "one invocation of sealed M2R1 V1 controller run",
    }
    atomic_publish_file(EXECUTION_ADMISSION, canonical(admission), 0o444)
    need(EXECUTION_ADMISSION.read_bytes() == canonical(admission) and mode_string(EXECUTION_ADMISSION) == "0444", "published admission verification failed")
    return {
        **admission,
        "admission": file_row(EXECUTION_ADMISSION),
        "audit_receipt": file_row(audit_receipt),
        "live_preflight_verdict": live_receipt["verdict"],
        "implementation_verdict": implementation["verdict"],
    }


def status() -> dict[str, Any]:
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "implementation_receipt_present": IMPLEMENTATION_RECEIPT.is_file(),
        "live_result_present": LIVE_RESULT.is_dir(),
        "audit_result_present": AUDIT_RESULT.is_dir(),
        "execution_admission": file_row(EXECUTION_ADMISSION) if EXECUTION_ADMISSION.is_file() else None,
        "execution_journal_present": M2_JOURNAL.is_dir(),
        "remote_reads": 0,
        "remote_writes": 0,
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "audit", "status"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            value = self_check()
        elif arguments.action == "audit":
            value = audit()
        else:
            value = status()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2R1 ADMISSION AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
