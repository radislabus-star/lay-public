#!/usr/bin/env python3
"""Independent live-state producer for M2R1 execution admission V1."""

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
QUIET_SECONDS = 15
PROBE_PATH = pathlib.PurePosixPath("/home/e/.cache") / (
    "lay-m2r1-execution-admission-v1-" + TRANSACTION_ID
)

CONTRACT = ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_CONTRACT_2026-08-27.md"
)
ROUTE = ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_ROUTE_2026-08-27.md"
)
STRUCTURAL_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_STRUCTURAL_REVIEW_2026-08-27.json"
)
IMPLEMENTATION_PREFLIGHT = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_IMPLEMENTATION_V1_2026-08-27.json"
)
IMPLEMENTATION_PREFLIGHT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_V1_IMPLEMENTATION_V1_PREFLIGHT_2026-08-27.json"
)
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
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_LIVE_PREFLIGHT_V1_2026-08-27"
)
AUDIT_RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_ADMISSION_AUDIT_V1_2026-08-27"
)
AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-admission-audit.py"
PRODUCER = pathlib.Path(__file__).resolve()

M2_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-v1.py"
M2_REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-remote.py"
M2_BOOTSTRAP_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-bootstrap-audit.py"
M2_BUILD_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-build-audit.py"
M2_TERMINAL_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-terminal-audit.py"
M2_FRAGMENT = ROOT / "scripts/lay_v10_e1_traversal_w1_fused_minimum_m2r1_test_module.rs.inc"
M2_JOURNAL = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_"
    "M2R1_EXECUTION_JOURNAL_V1_2026-08-27"
)

PINNED = {
    CONTRACT: ("16b9f543c5e0f14156621c38488b6f5440ed2256189bebd7fb8a0b807ec1aab5", 4949, "0444"),
    ROUTE: ("57a243c90605580ead96d8bca1bb777cfd6fdf57273087bdba13f82a612e0259", 4842, "0444"),
    STRUCTURAL_RECEIPT: ("dd1ddf46144d1ceb15071af0915f66618253d381cad01a73bbff13abb38b71ac", 27739, "0444"),
    IMPLEMENTATION_PREFLIGHT: ("94994ad58c387546579b2c2ddcf01b7efec9806ec9f434f56185dcba432ac528", 14390, "0444"),
    IMPLEMENTATION_PREFLIGHT_RECEIPT: ("a38f3017862fc696cee03ae484ac0a911a6770b99d135459bfd2a803361a33ae", 9784, "0444"),
    M2_CONTROLLER: ("d3540936c19e45230a70c805b77cf6dab024636457ea0577281a59be2e128106", 67252, "0555"),
    M2_REMOTE_CONTROLLER: ("26bfbbf2a9626b36bad549d6827d7442d1d1b5db7ced0f65da41c099a793de09", 40141, "0555"),
    M2_BOOTSTRAP_AUDITOR: ("c0a7221eeeef460db174682dafad16a7765a970a5623e3641c0657de4bfce84a", 11675, "0555"),
    M2_BUILD_AUDITOR: ("0f943646c0e13c6fe53c406aa542882d68ab9dbe447dc3c660719110ccae6762", 18936, "0555"),
    M2_TERMINAL_AUDITOR: ("37b6d27eaa719ebe6347a97303602fc30b4d4ec7078a0bf49b3ce424037e23fe", 33322, "0555"),
    M2_FRAGMENT: ("a6ea388d5d76f8223511fd4822cff2df9fd0c3394fc200f4fc52db956522ce5b", 156122, "0444"),
    M2R1_IMPLEMENTATION_RECEIPT: ("b888f1d10f4e846bcb1d08afaab3d5f49dbad0deacd4a0916037ae7526d509f4", 23760, "0444"),
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


class PreflightError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise PreflightError(message)


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
    return {
        "path": str(path),
        "sha256": sha256_file(path),
        "size_bytes": path.stat().st_size,
        "mode": mode_string(path),
    }


def verify_file(path: pathlib.Path, expected: tuple[str, int, str]) -> dict[str, Any]:
    need(path.is_file() and not path.is_symlink(), f"required file absent: {path}")
    row = file_row(path)
    need(
        (row["sha256"], row["size_bytes"], row["mode"]) == expected,
        f"file identity drift: {path}",
    )
    return row


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
    lines = []
    for path in sorted(p for p in root.rglob("*") if p.is_file() and p.name != "SHA256SUMS"):
        lines.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(lines).encode())


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


def verify_pinned() -> dict[str, Any]:
    rows = {str(path.relative_to(ROOT)): verify_file(path, expected) for path, expected in PINNED.items()}
    preflight = json.loads(IMPLEMENTATION_PREFLIGHT_RECEIPT.read_text())
    need(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True,
        "admission implementation preflight is not READY",
    )
    m2 = json.loads(M2R1_IMPLEMENTATION_RECEIPT.read_text())
    need(m2.get("verdict") == "M2R1_CONTROLLER_VERIFIED_UNRUN", "M2R1 receipt verdict drift")
    need(m2.get("task_id") == TASK_ID and m2.get("transaction_id") == TRANSACTION_ID, "M2R1 namespace drift")
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


REMOTE_SNAPSHOT_PROGRAM = r'''
import hashlib,json,os,pathlib,stat,subprocess,time
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
 return h.hexdigest()
def cmd(argv,env=None):
 return subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False,env=env,text=True)
def mode(path): return f'{stat.S_IMODE(path.stat().st_mode):04o}'
task='slice8b-v10-e1-traversal-w1-fused-minimum-m2r1-v1-20260827'
tx='2dae728a39aecd422995828674d12e311ab6362ebab4013c4f2520b3f6933c5f'
parent=pathlib.Path('/home/e/.local/share/lay/provenance')/task
state=pathlib.Path('/home/e/.local/state/lay')/task
cache=pathlib.Path('/home/e/.cache')/('lay-m2r1-'+tx)
probe=pathlib.Path('/home/e/.cache')/('lay-m2r1-execution-admission-v1-'+tx)
ps=cmd(['/usr/bin/ps','-eo','user=,pid=,ppid=,etimes=,psr=,stat=,comm=,args=']).stdout.splitlines()
table='\n'.join(ps)+'\n'
comm_block={'perf','cargo','rustc','rustc_driver'}
tokens=(
 'leg01-v13-all-live-inputs','nando-motif-shape-census','diagnostic-test-elf',
 'v10_m2_fused_minimum','lay-v10-e1-traversal','/target/release/deps/lay-',
 'perf record','perf stat','cargo build','cargo test','cargo bench','cargo check'
)
conflicts=[]
for line in ps:
 fields=line.strip().split(None,7)
 if len(fields)<8: continue
 user,pid,ppid,etimes,psr,pstate,comm,args=fields
 low=args.lower()
 reasons=[]
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
def as_e(argv): return cmd(['/usr/bin/sudo','-n','-u','e','/usr/bin/env',*[f'{k}={v}' for k,v in env.items()],*argv])
inputs={
 'package':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/LAY-L2-RU-FULL-v13.bin',
 'sidecar':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa',
 'v7':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json',
 'schedule':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1/query-schedule.json',
 'cargo_toml':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.toml',
 'cargo_lock':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/slice8b-v10-f6178f/artifacts/Cargo.lock',
 'cargo_guard':'/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2/inputs/controller/cargo-guard.sh',
}
rows={}
for name,raw in inputs.items():
 p=pathlib.Path(raw)
 rows[name]={'path':raw,'exists':p.is_file(),'size_bytes':p.stat().st_size if p.is_file() else None,'mode':mode(p) if p.exists() else None,'sha256':sha(p) if p.is_file() else None}
perf=pathlib.Path('/usr/bin/perf')
cargo=as_e(['cargo','-V'])
rustc=as_e(['rustc','-Vv'])
perf_v=cmd(['/usr/bin/perf','version'])
print(json.dumps({
 'schema':'lay.m2r1-execution-admission-live-snapshot.v2',
 'hostname':os.uname().nodename,'machine_id_sha256':sha('/etc/machine-id'),'kernel':os.uname().release,
 'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),
 'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),
 'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),
 'e_uid':int(cmd(['/usr/bin/id','-u','e']).stdout.strip()),
 'cargo':cargo.stdout.strip(),'cargo_rc':cargo.returncode,
 'rustc':rustc.stdout.strip(),'rustc_rc':rustc.returncode,
 'perf_version':perf_v.stdout.strip(),'perf_version_rc':perf_v.returncode,
 'perf_sha256':sha(perf),
 'perf_event_max_sample_rate':pathlib.Path('/proc/sys/kernel/perf_event_max_sample_rate').read_text().strip(),
 'perf_event_paranoid':pathlib.Path('/proc/sys/kernel/perf_event_paranoid').read_text().strip(),
 'm2r1_paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},
 'probe_exists':probe.exists(),'conflicting_processes':conflicts,'perf_event_fds':perf_fds,
 'process_table_sha256':hashlib.sha256(table.encode()).hexdigest(),'process_count':len(ps),
 'inputs':rows,'snapshot_monotonic_ns':time.monotonic_ns(),
},sort_keys=True))
'''


UID_PROBE_PROGRAM = r'''
import hashlib,json,os,pathlib,shutil,sys
p=pathlib.Path(sys.argv[1]); parent=p.parent; payload=b'm2r1-admission-uid-proof-v1\n'
before=not p.exists(); operations=[]; error=None
try:
 if not before: raise RuntimeError('probe path already exists')
 p.mkdir(mode=0o700); operations.append('mkdir')
 a=p/'a'; b=p/'b'
 fd=os.open(a,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 try:
  os.write(fd,payload); os.fsync(fd); operations.extend(['create','write','fsync-file'])
 finally: os.close(fd)
 os.rename(a,b); operations.append('rename')
 d=os.open(p,os.O_RDONLY|os.O_DIRECTORY); os.fsync(d); os.close(d); operations.append('fsync-directory')
 if b.read_bytes()!=payload: raise RuntimeError('probe bytes mismatch')
 operations.append('reopen-read')
 b.unlink(); operations.append('unlink')
 d=os.open(p,os.O_RDONLY|os.O_DIRECTORY); os.fsync(d); os.close(d)
 p.rmdir(); operations.append('rmdir')
 d=os.open(parent,os.O_RDONLY|os.O_DIRECTORY); os.fsync(d); os.close(d); operations.append('fsync-parent')
except BaseException as exc:
 error=f'{type(exc).__name__}: {exc}'
 if before and p.exists():
  shutil.rmtree(p,ignore_errors=True)
after=not p.exists()
print(json.dumps({'schema':'lay.m2r1-execution-admission-uid-proof.v2','uid':os.getuid(),'gid':os.getgid(),'path':str(p),'before_absent':before,'after_absent':after,'operations':operations,'payload_sha256':hashlib.sha256(payload).hexdigest(),'error':error,'verdict':'PASS' if error is None and after else 'BLOCKED_PROVENANCE'},sort_keys=True))
'''


def run(
    command: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: float = 120.0,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        raise PreflightError(
            f"command failed ({result.returncode}): {command!r}\n"
            + result.stderr[-4000:].decode(errors="replace")
        )
    return result


def ssh_python(program: str, arguments: Sequence[str] = (), *, root: bool) -> dict[str, Any]:
    remote_command = ["/usr/bin/python3", "-", *arguments]
    if root:
        remote_command = ["/usr/bin/sudo", "-n", *remote_command]
    command = [
        "/usr/bin/ssh",
        "-i",
        str(SSH_IDENTITY),
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=10",
        REMOTE,
        *remote_command,
    ]
    result = run(command, input_bytes=program.encode(), timeout=180.0)
    lines = result.stdout.decode().strip().splitlines()
    need(lines, "empty remote JSON response")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote response is not an object")
    return value


def remote_snapshot() -> dict[str, Any]:
    return ssh_python(REMOTE_SNAPSHOT_PROGRAM, root=True)


def validate_snapshot(value: Mapping[str, Any], *, require_clean: bool) -> list[str]:
    need(value.get("schema") == "lay.m2r1-execution-admission-live-snapshot.v2", "snapshot schema drift")
    for key in ("hostname", "machine_id_sha256", "kernel", "online", "core", "atom", "cargo", "perf_version", "perf_sha256", "perf_event_max_sample_rate", "perf_event_paranoid"):
        need(value.get(key) == REMOTE_EXPECTED[key], f"remote snapshot drift: {key}")
    need(value.get("e_uid") == 1000, "remote subject UID drift")
    need(value.get("cargo_rc") == 0 and value.get("rustc_rc") == 0 and value.get("perf_version_rc") == 0, "tool version query failed")
    rustc = str(value.get("rustc", ""))
    need(all(token in rustc for token in REMOTE_EXPECTED["rustc_tokens"]), "rustc identity drift")
    need(value.get("m2r1_paths") == {"parent": False, "state": False, "cache": False}, "M2R1 remote namespace exists")
    need(value.get("probe_exists") is False, "UID probe path is not absent")
    inputs = value.get("inputs")
    need(isinstance(inputs, Mapping) and set(inputs) == set(REMOTE_INPUTS), "remote input inventory drift")
    for name, expected in REMOTE_INPUTS.items():
        row = inputs[name]
        observed = (row.get("sha256"), row.get("size_bytes"), row.get("mode"))
        need(row.get("exists") is True and observed == expected, f"remote input drift: {name}")
    conflicts = list(value.get("conflicting_processes", []))
    perf_fds = list(value.get("perf_event_fds", []))
    if require_clean:
        need(not conflicts, "conflicting remote experiment is active")
        need(not perf_fds, "open remote perf_event descriptor exists")
    return [*(f"process:{row.get('pid')}:{row.get('comm')}" for row in conflicts), *(f"perf-fd:{row.get('pid')}:{row.get('fd')}" for row in perf_fds)]


def uid_probe() -> dict[str, Any]:
    value = ssh_python(UID_PROBE_PROGRAM, (str(PROBE_PATH),), root=False)
    validate_uid_proof(value)
    return value


def validate_uid_proof(value: Mapping[str, Any]) -> None:
    expected = ["mkdir", "create", "write", "fsync-file", "rename", "fsync-directory", "reopen-read", "unlink", "rmdir", "fsync-parent"]
    need(value.get("verdict") == "PASS", "UID capability probe failed")
    need(value.get("uid") == 1000 and value.get("before_absent") is True and value.get("after_absent") is True, "UID capability identity drift")
    need(value.get("operations") == expected and value.get("error") is None, "UID capability operation drift")
    need(value.get("payload_sha256") == "3b93efe386726592a0440dc8ebe8daa664912a5b9cd2da3340666c1260b2df42", "UID capability payload drift")


def admission_candidates() -> list[pathlib.Path]:
    root = ROOT / "docs/structural_gates/preflights"
    return sorted(root.glob("*FUSED_MINIMUM_M2R1_EXECUTION_ADMISSION_V1_2026-08-27.json"))


def verify_unrun() -> None:
    need(not RESULT.exists() and not AUDIT_RESULT.exists(), "admission evidence already exists")
    need(not IMPLEMENTATION_RECEIPT.exists(), "admission implementation receipt already exists")
    need(not M2_JOURNAL.exists(), "M2R1 execution journal exists")
    need(not admission_candidates(), "M2R1 execution admission already exists")
    for token in ("BOOTSTRAP_AUDIT_V1", "BUILD_AUDIT_V1", "TERMINAL_AUDIT_V1", "CONTROLLER_FAILURE_V1"):
        matches = list((ROOT / "docs/structural_gates/receipts").glob(f"*FUSED_MINIMUM_M2R1*{token}*"))
        need(not matches, f"M2R1 execution evidence exists: {token}")


def implementation_receipt() -> dict[str, Any]:
    need(IMPLEMENTATION_RECEIPT.is_file(), "admission implementation receipt absent")
    value = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(value.get("verdict") == "M2R1_ADMISSION_CONTROLLER_VERIFIED_UNRUN", "admission implementation verdict drift")
    need(value.get("producer_sha256") == sha256_file(PRODUCER), "producer changed after seal")
    need(value.get("auditor_sha256") == sha256_file(AUDITOR), "auditor changed after seal")
    return value


def static_source_check() -> dict[str, Any]:
    need(AUDITOR.is_file(), "independent admission auditor absent")
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
    need(auditor_publishers == 1, "auditor admission publisher cardinality drift")
    for text, label in ((producer_text, "producer"), (auditor_text, "auditor")):
        for forbidden in ("perf\", \"stat", "perf\", \"record", "cargo\", \"test", "cargo\", \"build", "--exact\", PHYSICAL_TEST"):
            need(forbidden not in text, f"scientific command in {label}: {forbidden}")
    for tree, label in ((producer_tree, "producer"), (auditor_tree, "auditor")):
        subprocess_runs = sum(
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "subprocess"
            and node.func.attr == "run"
            for node in ast.walk(tree)
        )
        forbidden_process_calls = [
            node
            for node in ast.walk(tree)
            if isinstance(node, ast.Call)
            and (
                isinstance(node.func, ast.Attribute)
                and isinstance(node.func.value, ast.Name)
                and node.func.value.id in {"subprocess", "os"}
                and node.func.attr in {"Popen", "system", "popen"}
            )
        ]
        need(subprocess_runs == 1 and not forbidden_process_calls, f"process command graph drift in {label}")
    return {
        "producer_ast": "PASS",
        "auditor_ast": "PASS",
        "producer_admission_publishers": producer_admission_names,
        "auditor_admission_publishers": auditor_publishers,
    }


def synthetic_snapshot() -> dict[str, Any]:
    inputs = {}
    for name, expected in REMOTE_INPUTS.items():
        inputs[name] = {"exists": True, "sha256": expected[0], "size_bytes": expected[1], "mode": expected[2]}
    return {
        "schema": "lay.m2r1-execution-admission-live-snapshot.v2",
        **{key: value for key, value in REMOTE_EXPECTED.items() if key != "rustc_tokens"},
        "e_uid": 1000,
        "cargo_rc": 0,
        "rustc": "\n".join(REMOTE_EXPECTED["rustc_tokens"]),
        "rustc_rc": 0,
        "perf_version_rc": 0,
        "m2r1_paths": {"parent": False, "state": False, "cache": False},
        "probe_exists": False,
        "inputs": inputs,
        "conflicting_processes": [],
        "perf_event_fds": [],
    }


def self_check() -> dict[str, Any]:
    pinned = verify_pinned()
    source = static_source_check()
    validate_snapshot(synthetic_snapshot(), require_clean=True)
    conflict = synthetic_snapshot()
    conflict["conflicting_processes"] = [{"pid": 7, "comm": "perf"}]
    need(validate_snapshot(conflict, require_clean=False) == ["process:7:perf"], "conflict projection test failed")
    try:
        validate_snapshot(conflict, require_clean=True)
    except PreflightError:
        conflict_veto = "PASS"
    else:
        raise PreflightError("conflicting process did not veto admission")
    snapshot_faults = 0
    mutations = (
        ("schema", "unknown"),
        ("hostname", "foreign-host"),
        ("probe_exists", True),
        ("m2r1_paths", {"parent": True, "state": False, "cache": False}),
        ("perf_event_fds", [{"pid": 9, "fd": "4"}]),
        ("inputs", {}),
    )
    for key, replacement in mutations:
        broken = synthetic_snapshot()
        broken[key] = replacement
        try:
            validate_snapshot(broken, require_clean=True)
        except PreflightError:
            snapshot_faults += 1
        else:
            raise PreflightError(f"snapshot mutation did not fail closed: {key}")
    proof = {
        "verdict": "PASS",
        "uid": 1000,
        "before_absent": True,
        "after_absent": True,
        "operations": ["mkdir", "create", "write", "fsync-file", "rename", "fsync-directory", "reopen-read", "unlink", "rmdir", "fsync-parent"],
        "payload_sha256": "3b93efe386726592a0440dc8ebe8daa664912a5b9cd2da3340666c1260b2df42",
        "error": None,
    }
    validate_uid_proof(proof)
    uid_faults = 0
    for key, replacement in (("uid", 0), ("after_absent", False), ("operations", []), ("payload_sha256", "0" * 64), ("error", "lost")):
        broken = dict(proof)
        broken[key] = replacement
        try:
            validate_uid_proof(broken)
        except PreflightError:
            uid_faults += 1
        else:
            raise PreflightError(f"UID proof mutation did not fail closed: {key}")
    with tempfile.TemporaryDirectory(prefix="m2r1-admission-self-check-") as raw:
        root = pathlib.Path(raw)
        destination = root / "receipt.json"
        atomic_publish_file(destination, canonical({"verdict": "PASS"}), 0o444)
        need(destination.is_file() and mode_string(destination) == "0444", "atomic publisher test failed")
        try:
            atomic_publish_file(destination, b"replacement\n", 0o444)
        except PreflightError:
            publication_fault = "PASS"
        else:
            raise PreflightError("atomic publisher overwrote existing evidence")
    return {
        "schema": "lay.m2r1-execution-admission-implementation-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2R1_ADMISSION_CONTROLLER_VERIFIED_UNRUN",
        "pinned": pinned,
        "source": source,
        "fault_injection": {
            "conflicting_process_veto": conflict_veto,
            "snapshot_faults_passed": snapshot_faults,
            "snapshot_faults_expected": len(mutations),
            "uid_faults_passed": uid_faults,
            "uid_faults_expected": 5,
            "atomic_publication_no_overwrite": publication_fault,
        },
        "network_access": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "cargo_compilations": 0,
        "rustc_compilations": 0,
        "perf_stat": 0,
        "perf_record": 0,
        "pmu_events": 0,
        "subject_executions": 0,
        "markers_created": 0,
        "markers_consumed": 0,
        "runtime_authority_changed": False,
    }


def seal_self_check() -> dict[str, Any]:
    verify_unrun()
    receipt = self_check()
    receipt.update(
        {
            "producer_sha256": sha256_file(PRODUCER),
            "auditor_sha256": sha256_file(AUDITOR),
            "implementation_preflight_sha256": sha256_file(IMPLEMENTATION_PREFLIGHT),
            "implementation_preflight_receipt_sha256": sha256_file(IMPLEMENTATION_PREFLIGHT_RECEIPT),
            "next_action_admitted": "live preflight producer only; M2R1 execution remains forbidden",
        }
    )
    atomic_publish_file(IMPLEMENTATION_RECEIPT, canonical(receipt), 0o444)
    return {**receipt, "receipt": file_row(IMPLEMENTATION_RECEIPT)}


def produce() -> dict[str, Any]:
    implementation_receipt()
    need(not RESULT.exists() and not AUDIT_RESULT.exists() and not M2_JOURNAL.exists(), "M2R1 admission or execution evidence exists")
    need(not admission_candidates(), "M2R1 execution admission already exists")
    runtime_before = runtime_snapshot()
    snapshot_one = remote_snapshot()
    conflicts_one = validate_snapshot(snapshot_one, require_clean=False)
    if conflicts_one:
        return {
            "verdict": "WAIT_CONFLICTING_EXPERIMENT",
            "snapshot": 1,
            "conflicts": conflicts_one,
            "remote_writes": 0,
            "admission_created": False,
        }
    proof = uid_probe()
    time.sleep(QUIET_SECONDS)
    snapshot_two = remote_snapshot()
    conflicts_two = validate_snapshot(snapshot_two, require_clean=False)
    if conflicts_two:
        return {
            "verdict": "WAIT_CONFLICTING_EXPERIMENT",
            "snapshot": 2,
            "conflicts": conflicts_two,
            "uid_probe_completed_and_cleaned": True,
            "remote_writes": 1,
            "admission_created": False,
        }
    validate_snapshot(snapshot_one, require_clean=True)
    validate_snapshot(snapshot_two, require_clean=True)
    runtime_after = runtime_snapshot()
    need(runtime_before == runtime_after, "local runtime changed during live preflight")

    RESULT.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix=f".{RESULT.name}.stage-", dir=RESULT.parent))
    try:
        write_new(stage / "SNAPSHOT-1.json", canonical(snapshot_one))
        write_new(stage / "UID_PROOF.json", canonical(proof))
        write_new(stage / "SNAPSHOT-2.json", canonical(snapshot_two))
        write_new(stage / "RUNTIME_BEFORE.json", canonical(runtime_before))
        write_new(stage / "RUNTIME_AFTER.json", canonical(runtime_after))
        receipt = {
            "schema": "lay.m2r1-live-execution-preflight.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M2R1_LIVE_PREFLIGHT_PASS_UNADMITTED",
            "safe_to_execute": False,
            "producer_sha256": sha256_file(PRODUCER),
            "auditor_sha256": sha256_file(AUDITOR),
            "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
            "m2r1_implementation_receipt_sha256": sha256_file(M2R1_IMPLEMENTATION_RECEIPT),
            "snapshot_1_sha256": sha256_bytes(canonical(snapshot_one)),
            "snapshot_2_sha256": sha256_bytes(canonical(snapshot_two)),
            "uid_proof_sha256": sha256_bytes(canonical(proof)),
            "quiet_seconds": QUIET_SECONDS,
            "conflicting_processes": 0,
            "open_perf_event_descriptors": 0,
            "m2r1_remote_parent_state_cache": 0,
            "uid_probe_operations": len(proof["operations"]),
            "uid_probe_cleaned": True,
            "remote_reads": 2,
            "remote_disposable_probe_transactions": 1,
            "cargo_version_queries": 2,
            "rustc_version_queries": 2,
            "cargo_compilations": 0,
            "rustc_compilations": 0,
            "perf_version_queries": 2,
            "perf_stat": 0,
            "perf_record": 0,
            "pmu_events": 0,
            "subject_executions": 0,
            "markers_created": 0,
            "markers_consumed": 0,
            "runtime_authority_changed": False,
            "execution_admission_created": False,
            "next_action_admitted": "independent admission audit only; M2R1 execution remains forbidden",
        }
        write_new(stage / "M2R1_LIVE_PREFLIGHT_RECEIPT.json", canonical(receipt))
        publish_tree(stage, RESULT)
        return {**receipt, "receipt": file_row(RESULT / "M2R1_LIVE_PREFLIGHT_RECEIPT.json")}
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise


def status() -> dict[str, Any]:
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "implementation_receipt_present": IMPLEMENTATION_RECEIPT.is_file(),
        "live_preflight_result_present": RESULT.is_dir(),
        "admission_audit_result_present": AUDIT_RESULT.is_dir(),
        "admission_candidates": [str(path) for path in admission_candidates()],
        "execution_journal_present": M2_JOURNAL.is_dir(),
        "remote_reads": 0,
        "remote_writes": 0,
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "seal-self-check", "produce", "status"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            value = self_check()
        elif arguments.action == "seal-self-check":
            value = seal_self_check()
        elif arguments.action == "produce":
            value = produce()
        else:
            value = status()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2R1 LIVE PREFLIGHT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
