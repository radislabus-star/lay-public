#!/usr/bin/env python3
"""Independent live admission audit for the one-shot M3 experiment."""

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
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-v1-20260827"
TRANSACTION_ID = "03ae2d28e6c943c4f20aad58dc4160550314e14bff057ecc4fd60d97c69e35de"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
QUIET_SECONDS = 15
AUDITOR = pathlib.Path(__file__).resolve()
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-{TRANSACTION_ID}"
REMOTE_PROBE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-admission-{TRANSACTION_ID}"

IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_"
    "IMPLEMENTATION_SELF_CHECK_V1_2026-08-27.json"
)
IMPLEMENTATION_RECEIPT_SHA256 = "b4461145fcffa760c904e93838dace7770933a5fd8bab7231623bc2caa3cc4a9"
EXECUTION_ADMISSION = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_"
    "EXECUTION_ADMISSION_V1_2026-08-27.json"
)
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_"
    "LIVE_EXECUTION_ADMISSION_V1_2026-08-27"
)
EXECUTION_JOURNAL = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_"
    "EXECUTION_JOURNAL_V1_2026-08-27"
)
CONTROLLER_FAILURE = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_"
    "CONTROLLER_FAILURE_V1_2026-08-27"
)

SOURCE_EXPECTATIONS = {
    "controller_sha256": (
        ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v1.py",
        "d5baeddfdfe71e0f04f768d848fde43c4a2a59e80897cb2928200c8c3421ecdb",
        "0555",
    ),
    "remote_controller_sha256": (
        ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-remote.py",
        "ff1f3c629aa0525f6efbcabaf6fc78a77123207a38e7059ff578e09fa7a1d306",
        "0555",
    ),
    "bootstrap_auditor_sha256": (
        ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-bootstrap-audit.py",
        "87f526d2cacc7564749d60092b78ce7d28af987446b5a20d20133753e222d584",
        "0555",
    ),
    "build_auditor_sha256": (
        ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-build-audit.py",
        "48454e92c68b4ff172e33f8f5f0a298b93989f0c9f128edc0e582bc216f630d4",
        "0555",
    ),
    "terminal_auditor_sha256": (
        ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-terminal-audit.py",
        "1be1d3733f9b15a1b641936c51dcfdb933cd0bdf89f4a3d6221dc27b7475bc83",
        "0555",
    ),
    "fragment_sha256": (
        ROOT / "scripts/lay_v10_e1_traversal_w1_dafsa_typed_view_m3_test_module.rs.inc",
        "5a2e164c47c88677b74baf44d500c939749c98deff3092a1621086cf6e800875",
        "0444",
    ),
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
    "package": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0a-input-closure-v2/inputs/LAY-L2-RU-FULL-v13.bin",
        "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
        140_556_462,
        "0444",
    ),
    "sidecar": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa",
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
        3_689_884,
        "0444",
    ),
    "v7": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json",
        "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
        1_606_189,
        "0444",
    ),
    "schedule": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0b-schedule-closure-v1/query-schedule.json",
        "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
        174_941,
        "0444",
    ),
    "cargo_toml": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.toml",
        "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
        2_399,
        "0444",
    ),
    "cargo_lock": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.lock",
        "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
        70_770,
        "0444",
    ),
    "cargo_guard": (
        "/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/"
        "b0a-input-closure-v2/inputs/controller/cargo-guard.sh",
        "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
        2_534,
        "0555",
    ),
    "loader": (
        "/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2",
        "8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc",
        240_936,
        "0755",
    ),
}


class AdmissionError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise AdmissionError(message)


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


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def file_row(path: pathlib.Path) -> dict[str, Any]:
    need(path.is_file() and not path.is_symlink(), f"required file absent: {path}")
    return {
        "path": str(path),
        "sha256": sha256_file(path),
        "size_bytes": path.stat().st_size,
        "mode": mode_string(path),
    }


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
    need(not path.exists(), f"immutable publication already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    stage = path.parent / f".{path.name}.stage-{os.getpid()}-{time.time_ns()}"
    try:
        write_new(stage, value, mode)
        os.link(stage, path)
        fsync_directory(path.parent)
    finally:
        stage.unlink(missing_ok=True)


def write_manifest(root: pathlib.Path) -> None:
    rows = [
        f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n"
        for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file() and candidate.name != "SHA256SUMS")
    ]
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish_tree(stage: pathlib.Path, destination: pathlib.Path) -> None:
    need(not destination.exists(), f"immutable evidence tree already exists: {destination}")
    write_manifest(stage)
    seal_tree(stage)
    os.rename(stage, destination)
    fsync_directory(destination.parent)


def run(
    command: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: float = 180.0,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if result.returncode != 0:
        raise AdmissionError(
            f"command failed ({result.returncode}): {command!r}\n"
            + result.stderr[-5000:].decode(errors="replace")
        )
    return result


def ssh_python(program: str, arguments: Sequence[str] = (), *, root: bool) -> dict[str, Any]:
    remote_command = ["/usr/bin/python3", "-", *arguments]
    if root:
        remote_command = ["/usr/bin/sudo", "-n", *remote_command]
    result = run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_IDENTITY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            *remote_command,
        ],
        input_bytes=program.encode(),
    )
    rows = result.stdout.decode().strip().splitlines()
    need(rows, "empty remote JSON response")
    value = json.loads(rows[-1])
    need(isinstance(value, dict), "remote response is not an object")
    return value


def runtime_snapshot() -> dict[str, Any]:
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    expected = implementation.get("runtime_after", {}).get("installed_lay_hashes", {})
    need(isinstance(expected, dict) and expected, "sealed runtime projection absent")
    observed = {}
    for name, identity in sorted(expected.items()):
        path = pathlib.Path(identity["target"])
        need(path.is_file() and sha256_file(path) == identity["sha256"], f"runtime drift: {name}")
        observed[name] = {"target": str(path), "sha256": identity["sha256"]}
    return {"installed_lay_hashes": observed}


def verify_local() -> dict[str, Any]:
    receipt = file_row(IMPLEMENTATION_RECEIPT)
    need(
        receipt["sha256"] == IMPLEMENTATION_RECEIPT_SHA256 and receipt["mode"] == "0444",
        "M3 implementation receipt identity drift",
    )
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(
        implementation.get("verdict") == "M3_CONTROLLER_VERIFIED_UNRUN"
        and implementation.get("task_id") == TASK_ID
        and implementation.get("transaction_id") == TRANSACTION_ID,
        "M3 implementation receipt content drift",
    )
    sources = {}
    for key, (path, digest, mode) in SOURCE_EXPECTATIONS.items():
        row = file_row(path)
        need(row["sha256"] == digest and row["mode"] == mode, f"sealed M3 source drift: {key}")
        need(implementation.get(key) == digest, f"implementation source binding drift: {key}")
        sources[key] = row
    need(not EXECUTION_ADMISSION.exists(), "M3 execution admission already exists")
    need(not RESULT.exists(), "M3 live admission evidence already exists")
    need(not EXECUTION_JOURNAL.exists(), "M3 execution journal already exists")
    need(not CONTROLLER_FAILURE.exists(), "M3 controller failure evidence already exists")
    for token in ("BOOTSTRAP_AUDIT_V1", "BUILD_AUDIT_V1", "TERMINAL_AUDIT_V1"):
        matches = list((ROOT / "docs/structural_gates/receipts").glob(f"*DAFSA_TYPED_VIEW_M3*{token}*"))
        need(not matches, f"M3 scientific execution evidence already exists: {token}")
    return {"implementation_receipt": receipt, "sources": sources}


REMOTE_SNAPSHOT_PROGRAM = r'''
import hashlib,json,os,pathlib,stat,subprocess,time
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as source:
  for block in iter(lambda:source.read(1048576),b''): h.update(block)
 return h.hexdigest()
def cmd(argv,env=None): return subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,env=env,text=True)
def mode(path): return f'{stat.S_IMODE(path.stat().st_mode):04o}'
task='slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-v1-20260827'
tx='03ae2d28e6c943c4f20aad58dc4160550314e14bff057ecc4fd60d97c69e35de'
parent=pathlib.Path('/home/e/.local/share/lay/provenance')/task
state=pathlib.Path('/home/e/.local/state/lay')/task
cache=pathlib.Path('/home/e/.cache')/('lay-m3-'+tx)
probe=pathlib.Path('/home/e/.cache')/('lay-m3-admission-'+tx)
ps=cmd(['/usr/bin/ps','-eo','user=,pid=,ppid=,etimes=,psr=,stat=,comm=,args=']).stdout.splitlines()
table='\n'.join(ps)+'\n'
comm_block={'perf','cargo','rustc','rustc_driver'}
tokens=(
 'leg01-v13-all-live-inputs','nando-motif-shape-census','diagnostic-test-elf',
 'v10_m3_dafsa_typed_view','v10_m2_fused_minimum','/target/release/deps/lay-',
 'perf record','perf stat','cargo build','cargo test','cargo bench','cargo check'
)
conflicts=[]
for line in ps:
 fields=line.strip().split(None,7)
 if len(fields)<8: continue
 user,pid,ppid,etimes,psr,pstate,comm,args=fields
 low=args.lower(); reasons=[]
 if comm.lower() in comm_block: reasons.append('blocked-comm')
 reasons.extend('token:'+token for token in tokens if token in low)
 if reasons:
  conflicts.append({'user':user,'pid':int(pid),'ppid':int(ppid),'etimes':int(etimes),'cpu':psr,'state':pstate,'comm':comm,'argv':args,'reasons':sorted(set(reasons))})
perf_fds=[]
for proc in pathlib.Path('/proc').iterdir():
 if not proc.name.isdigit(): continue
 try:
  comm=(proc/'comm').read_text(errors='replace').strip()
  argv=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace').strip()
  for fd in (proc/'fd').iterdir():
   try: target=os.readlink(fd)
   except OSError: continue
   if 'perf_event' in target:
    perf_fds.append({'pid':int(proc.name),'comm':comm,'argv':argv,'fd':fd.name,'target':target})
 except (OSError,PermissionError): pass
env={'HOME':'/home/e','LANG':'C.UTF-8','LC_ALL':'C.UTF-8','PATH':'/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin'}
def as_e(argv): return cmd(['/usr/bin/sudo','-n','-u','e','/usr/bin/env',*[f'{key}={value}' for key,value in env.items()],*argv])
inputs={
 'package':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/LAY-L2-RU-FULL-v13.bin',
 'sidecar':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa',
 'v7':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json',
 'schedule':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1/query-schedule.json',
 'cargo_toml':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.toml',
 'cargo_lock':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.lock',
 'cargo_guard':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/controller/cargo-guard.sh',
 'loader':'/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2',
}
rows={}
for name,raw in inputs.items():
 path=pathlib.Path(raw)
 rows[name]={'path':raw,'exists':path.is_file(),'size_bytes':path.stat().st_size if path.is_file() else None,'mode':mode(path) if path.exists() else None,'sha256':sha(path) if path.is_file() else None}
cargo=as_e(['cargo','-V']); rustc=as_e(['rustc','-Vv']); perf=pathlib.Path('/usr/bin/perf'); perf_v=cmd(['/usr/bin/perf','version'])
print(json.dumps({
 'schema':'lay.m3-live-execution-admission-snapshot.v1',
 'hostname':os.uname().nodename,'machine_id_sha256':sha('/etc/machine-id'),'kernel':os.uname().release,
 'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),
 'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),
 'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),
 'e_uid':int(cmd(['/usr/bin/id','-u','e']).stdout.strip()),
 'cargo':cargo.stdout.strip(),'cargo_rc':cargo.returncode,
 'rustc':rustc.stdout.strip(),'rustc_rc':rustc.returncode,
 'perf_version':perf_v.stdout.strip(),'perf_version_rc':perf_v.returncode,'perf_sha256':sha(perf),
 'perf_event_max_sample_rate':pathlib.Path('/proc/sys/kernel/perf_event_max_sample_rate').read_text().strip(),
 'perf_event_paranoid':pathlib.Path('/proc/sys/kernel/perf_event_paranoid').read_text().strip(),
 'm3_paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},
 'probe_exists':probe.exists(),'conflicting_processes':conflicts,'perf_event_fds':perf_fds,
 'process_table_sha256':hashlib.sha256(table.encode()).hexdigest(),'process_count':len(ps),
 'inputs':rows,'snapshot_monotonic_ns':time.monotonic_ns(),
},sort_keys=True))
'''


UID_PROBE_PROGRAM = r'''
import hashlib,json,os,pathlib,shutil,sys
probe=pathlib.Path(sys.argv[1]); inputs=[pathlib.Path(value) for value in sys.argv[2:]]
payload=b'm3-admission-uid-proof-v1\n'; operations=[]; error=None; before=not probe.exists()
try:
 if not before: raise RuntimeError('probe path already exists')
 for root in ('/home/e/.cache','/home/e/.local/share/lay/provenance','/home/e/.local/state/lay'):
  descriptor=os.open(root,os.O_RDONLY|os.O_DIRECTORY); os.close(descriptor); operations.append('traverse:'+root)
 for path in inputs:
  descriptor=os.open(path,os.O_RDONLY); first=os.read(descriptor,1); os.close(descriptor)
  if not first: raise RuntimeError('empty required input: '+str(path))
  operations.append('read:'+path.name)
 probe.mkdir(mode=0o700); operations.append('mkdir')
 source=probe/'a'; destination=probe/'b'
 descriptor=os.open(source,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 try:
  os.write(descriptor,payload); os.fsync(descriptor); operations.extend(['create','write','fsync-file'])
 finally: os.close(descriptor)
 os.rename(source,destination); operations.append('rename')
 descriptor=os.open(probe,os.O_RDONLY|os.O_DIRECTORY); os.fsync(descriptor); os.close(descriptor); operations.append('fsync-directory')
 if destination.read_bytes()!=payload: raise RuntimeError('probe byte mismatch')
 operations.append('reopen-read'); destination.unlink(); operations.append('unlink')
 probe.rmdir(); operations.append('rmdir')
 descriptor=os.open(probe.parent,os.O_RDONLY|os.O_DIRECTORY); os.fsync(descriptor); os.close(descriptor); operations.append('fsync-parent')
except BaseException as exception:
 error=f'{type(exception).__name__}: {exception}'
 if before and probe.exists(): shutil.rmtree(probe,ignore_errors=True)
print(json.dumps({'schema':'lay.m3-live-execution-admission-uid-proof.v1','uid':os.getuid(),'gid':os.getgid(),'path':str(probe),'before_absent':before,'after_absent':not probe.exists(),'inputs_read':len(inputs),'operations':operations,'payload_sha256':hashlib.sha256(payload).hexdigest(),'error':error,'verdict':'PASS' if error is None and not probe.exists() else 'BLOCKED_PROVENANCE'},sort_keys=True))
'''


def remote_snapshot() -> dict[str, Any]:
    return ssh_python(REMOTE_SNAPSHOT_PROGRAM, root=True)


def validate_snapshot(value: Mapping[str, Any]) -> None:
    need(value.get("schema") == "lay.m3-live-execution-admission-snapshot.v1", "snapshot schema drift")
    for key in (
        "hostname",
        "machine_id_sha256",
        "kernel",
        "online",
        "core",
        "atom",
        "cargo",
        "perf_version",
        "perf_sha256",
        "perf_event_max_sample_rate",
        "perf_event_paranoid",
    ):
        need(value.get(key) == REMOTE_EXPECTED[key], f"remote snapshot drift: {key}")
    need(value.get("e_uid") == 1000, "remote subject UID drift")
    need(
        value.get("cargo_rc") == 0 and value.get("rustc_rc") == 0 and value.get("perf_version_rc") == 0,
        "remote tool query failed",
    )
    need(
        all(token in str(value.get("rustc", "")) for token in REMOTE_EXPECTED["rustc_tokens"]),
        "remote rustc identity drift",
    )
    need(value.get("m3_paths") == {"parent": False, "state": False, "cache": False}, "M3 remote namespace is not absent")
    need(value.get("probe_exists") is False, "M3 admission probe remains present")
    need(not value.get("conflicting_processes"), "conflicting remote performance process is active")
    need(not value.get("perf_event_fds"), "open remote perf_event descriptor exists")
    inputs = value.get("inputs")
    need(isinstance(inputs, Mapping) and set(inputs) == set(REMOTE_INPUTS), "remote input inventory drift")
    for name, (_, digest, size, mode) in REMOTE_INPUTS.items():
        row = inputs[name]
        need(
            row.get("exists") is True
            and (row.get("sha256"), row.get("size_bytes"), row.get("mode")) == (digest, size, mode),
            f"remote input identity drift: {name}",
        )


def uid_probe() -> dict[str, Any]:
    paths = [value[0] for value in REMOTE_INPUTS.values()]
    proof = ssh_python(UID_PROBE_PROGRAM, (str(REMOTE_PROBE), *paths), root=False)
    need(
        proof.get("verdict") == "PASS"
        and proof.get("uid") == 1000
        and proof.get("before_absent") is True
        and proof.get("after_absent") is True
        and proof.get("inputs_read") == len(REMOTE_INPUTS)
        and proof.get("error") is None,
        "UID e capability proof failed",
    )
    need(
        proof.get("payload_sha256") == sha256_bytes(b"m3-admission-uid-proof-v1\n"),
        "UID capability payload drift",
    )
    return proof


def synthetic_snapshot() -> dict[str, Any]:
    return {
        "schema": "lay.m3-live-execution-admission-snapshot.v1",
        **{key: value for key, value in REMOTE_EXPECTED.items() if key != "rustc_tokens"},
        "e_uid": 1000,
        "cargo_rc": 0,
        "rustc": "\n".join(REMOTE_EXPECTED["rustc_tokens"]),
        "rustc_rc": 0,
        "perf_version_rc": 0,
        "m3_paths": {"parent": False, "state": False, "cache": False},
        "probe_exists": False,
        "conflicting_processes": [],
        "perf_event_fds": [],
        "inputs": {
            name: {"exists": True, "sha256": digest, "size_bytes": size, "mode": mode}
            for name, (_, digest, size, mode) in REMOTE_INPUTS.items()
        },
    }


def self_check() -> dict[str, Any]:
    local = verify_local()
    source = AUDITOR.read_text()
    tree = ast.parse(source)
    subprocess_runs = [
        node
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == "subprocess"
        and node.func.attr == "run"
    ]
    forbidden_calls = [
        node
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id in {"subprocess", "os"}
        and node.func.attr in {"Popen", "system", "popen"}
    ]
    publishers = [
        node
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "atomic_publish_file"
        and node.args
        and isinstance(node.args[0], ast.Name)
        and node.args[0].id == "EXECUTION_ADMISSION"
    ]
    need(len(subprocess_runs) == 1 and not forbidden_calls, "admission process graph drift")
    need(len(publishers) == 1, "execution admission publisher cardinality drift")
    remote_programs = REMOTE_SNAPSHOT_PROGRAM + UID_PROBE_PROGRAM
    for forbidden in (
        '"--exact"',
        '"run-route"',
        '"build-once"',
        '"parity-once"',
        '"test", "--offline"',
    ):
        need(forbidden not in remote_programs, f"scientific execution token in admission programs: {forbidden}")
    validate_snapshot(synthetic_snapshot())
    with tempfile.TemporaryDirectory(prefix="lay-m3-admission-publication-") as raw:
        target = pathlib.Path(raw) / "admission.json"
        atomic_publish_file(target, canonical({"safe_to_execute": True}))
        need(mode_string(target) == "0444", "admission publication mode drift")
        try:
            atomic_publish_file(target, b"overwrite\n")
        except AdmissionError:
            overwrite_blocked = True
        else:
            overwrite_blocked = False
        need(overwrite_blocked, "execution admission could be overwritten")
    return {
        "schema": "lay.m3-live-execution-admission-auditor-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_LIVE_ADMISSION_AUDITOR_VERIFIED_UNRUN",
        "auditor_sha256": sha256_file(AUDITOR),
        "implementation_receipt_sha256": IMPLEMENTATION_RECEIPT_SHA256,
        "local_closure": local,
        "subprocess_run_owners": 1,
        "execution_admission_publishers": 1,
        "atomic_overwrite_blocked": True,
        "network_access": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "subject_executions": 0,
    }


def audit() -> dict[str, Any]:
    need(mode_string(AUDITOR) == "0555", "admission auditor is not sealed executable")
    check = self_check()
    local = check["local_closure"]
    runtime_before = runtime_snapshot()
    snapshots = []
    snapshot_one = remote_snapshot()
    validate_snapshot(snapshot_one)
    snapshots.append(snapshot_one)
    proof = uid_probe()
    time.sleep(QUIET_SECONDS)
    snapshot_two = remote_snapshot()
    validate_snapshot(snapshot_two)
    snapshots.append(snapshot_two)
    snapshot_three = remote_snapshot()
    validate_snapshot(snapshot_three)
    snapshots.append(snapshot_three)
    runtime_after = runtime_snapshot()
    need(runtime_before == runtime_after, "installed runtime changed during live admission")

    RESULT.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix=f".{RESULT.name}.stage-", dir=RESULT.parent))
    try:
        for index, snapshot in enumerate(snapshots, 1):
            write_new(stage / f"SNAPSHOT-{index}.json", canonical(snapshot))
        write_new(stage / "UID_PROOF.json", canonical(proof))
        write_new(stage / "RUNTIME_BEFORE.json", canonical(runtime_before))
        write_new(stage / "RUNTIME_AFTER.json", canonical(runtime_after))
        receipt = {
            "schema": "lay.m3-live-execution-admission-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M3_LIVE_EXECUTION_ADMISSION_PASS",
            "auditor_sha256": sha256_file(AUDITOR),
            "auditor_self_check_sha256": sha256_bytes(canonical(check)),
            "implementation_receipt_sha256": IMPLEMENTATION_RECEIPT_SHA256,
            "source_identities": {key: digest for key, (_, digest, _) in SOURCE_EXPECTATIONS.items()},
            "live_snapshots": len(snapshots),
            "quiet_seconds": QUIET_SECONDS,
            "uid_capability": "PASS",
            "remote_namespace_present": False,
            "conflicting_processes": 0,
            "open_perf_event_descriptors": 0,
            "execution_journal_present": False,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "perf_stat_invocations": 0,
            "perf_record_invocations": 0,
            "subject_executions": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "atomic publication of exact M3 execution admission only",
        }
        write_new(stage / "M3_LIVE_EXECUTION_ADMISSION_AUDIT_RECEIPT.json", canonical(receipt))
        write_new(stage / "SELF_CHECK.json", canonical(check))
        write_new(stage / "auditor.py", AUDITOR.read_bytes())
        publish_tree(stage, RESULT)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise

    audit_receipt = RESULT / "M3_LIVE_EXECUTION_ADMISSION_AUDIT_RECEIPT.json"
    admission = {
        "schema": "lay.v10.e1-traversal-w1-dafsa-typed-view-m3-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_EXECUTION_ADMITTED",
        "safe_to_execute": True,
        **{key: digest for key, (_, digest, _) in SOURCE_EXPECTATIONS.items()},
        "implementation_receipt_sha256": IMPLEMENTATION_RECEIPT_SHA256,
        "live_admission_auditor_sha256": sha256_file(AUDITOR),
        "live_admission_audit_receipt_sha256": sha256_file(audit_receipt),
        "live_admission_manifest_sha256": sha256_file(RESULT / "SHA256SUMS"),
        "live_snapshots": len(snapshots),
        "conflicting_processes": 0,
        "open_perf_event_descriptors": 0,
        "m3_remote_namespace_present": False,
        "execution_journal_present": False,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_stat_invocations": 0,
        "perf_record_invocations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "one invocation of sealed M3 controller run",
    }
    atomic_publish_file(EXECUTION_ADMISSION, canonical(admission), 0o444)
    need(
        EXECUTION_ADMISSION.read_bytes() == canonical(admission)
        and mode_string(EXECUTION_ADMISSION) == "0444",
        "published M3 execution admission verification failed",
    )
    return {
        **admission,
        "admission": file_row(EXECUTION_ADMISSION),
        "audit_receipt": file_row(audit_receipt),
        "manifest": file_row(RESULT / "SHA256SUMS"),
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "local_closure": local,
    }


def status() -> dict[str, Any]:
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "auditor": file_row(AUDITOR),
        "result_present": RESULT.is_dir(),
        "execution_admission": file_row(EXECUTION_ADMISSION) if EXECUTION_ADMISSION.is_file() else None,
        "execution_journal_present": EXECUTION_JOURNAL.is_dir(),
        "controller_failure_present": CONTROLLER_FAILURE.is_dir(),
        "remote_reads": 0,
        "remote_writes": 0,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit", "status"))
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit() if arguments.action == "audit" else status()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M3 LIVE ADMISSION ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
