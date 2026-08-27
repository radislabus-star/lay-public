#!/usr/bin/env python3
"""Independent read-only terminal audit of the blocked D3 U2 route."""

from __future__ import annotations

import argparse
import ast
import base64
import hashlib
import io
import json
import os
import pathlib
import shlex
import shutil
import stat
import subprocess
import sys
import tarfile
import time
from typing import Any, Mapping, Sequence


AUDITOR = pathlib.Path(__file__).resolve()
PROJECT_ROOT = AUDITOR.parents[1]
TASK_ID = "slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826"
TRANSACTION_ID = "e88555465ee51b7caed891217e8941ceb0b412ed82981d7c88cde71c3eb452e1"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

PAPER = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_TASK_CLOCK_ESTIMATOR_RECOVERY_V1_2026-08-26.md"
)
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_2026-08-26.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_PREFLIGHT_2026-08-26.json"
)
D2_TERMINAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "T_SINGLE_TERMINAL_AUDIT_V1_2026-08-26/T_SINGLE_TERMINAL_AUDIT_RECEIPT.json"
)
BOOTSTRAP_AUDIT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_BOOTSTRAP_AUDIT_V1_2026-08-26/"
    "D3_BOOTSTRAP_AUDIT_RECEIPT.json"
)
CORRECTION = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_"
    "BOOTSTRAP_EXECUTION_ADMISSION_CORRECTION_V2_2026-08-26.md"
)
LOCAL_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d3-estimator-recovery.py"
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d3-estimator-recovery-remote.py"
BOOTSTRAP_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d3-bootstrap-audit.py"
RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_TERMINAL_AUDIT_V1_2026-08-26"
)

PAPER_SHA256 = "ebe80974392a05527bea67944f381cfd2f74fb0be1c5b2ba3bf4a5aba22be11a"
PREFLIGHT_SHA256 = "f9fba59409fe56ae742738d66bbae886b24a165dc34e258529e3f6de6710456e"
PREFLIGHT_RECEIPT_SHA256 = "09a66ecabebd95e1a8b76e459686bc41a0146f7bedbabf18dbc562096e24868d"
D2_TERMINAL_SHA256 = "75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608"
BOOTSTRAP_AUDIT_SHA256 = "0c5a34b1809dbbfa8b3744b65d86f3dfa1d0c1bfb00a623a0bbe5089669b7bb1"
CORRECTION_SHA256 = "b963c0059fe6efaca746fad2ad9dc4784c7a6d4b9ea524c9faa18c6858197519"
LOCAL_CONTROLLER_SHA256 = "8f48f4ac95a288d3c58cf60946a7e176f5b527d7a11c51d22680efd89c22acb2"
REMOTE_CONTROLLER_SHA256 = "3540c797130f537330c18b3739baddacfac5eba1f8fdaa201e3af1f30e3a6e85"
BOOTSTRAP_AUDITOR_SHA256 = "59f963d6327f00baf561113e8ff98b781b2eae22d83c68cbfa0e2d2c56610db6"
BOOTSTRAP_RECEIPT_SHA256 = "a7b921799751f38f745a2945ff8b7222428ff16c54e42c35e0e0d99019468529"
U2_RECEIPT_SHA256 = "7c74a689079b8c40442c8065ce73a5deb69d990daf3a6404b43461808744888e"
U2_OBSERVATION_SHA256 = "a85d128fed922a0317206ff7d716c59ea6e6e876320453a37af4430996ea74c9"
U2_STDERR_SHA256 = "42929422fb0590c9083660e3558ec3643cde54dcf13b2bb485e0ac142934a964"
U2_MANIFEST_SHA256 = "76469bd56d932f27cfc10610c8829f4864d74450d17f95f5cc9fb55699ec071d"
U2_STATE_SHA256 = "b7be4bc41ee05b4afd9d29d24caf45d170018a42301c5c2c44840de626c202cc"

REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_U2 = REMOTE_PARENT / "u2-single-v1"
D2_TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
D2_TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
D2_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / D2_TASK_ID

EXTERNAL_ACTIONS = ("self-check", "audit")
D3_MARKERS = {
    "u2-single.consumed-before-exec": ("U2-SINGLE", "a212d2edf720551b70f9245d62487fd7834f3796ebfe5d9759a3001fd28f9502", 287),
    "t2-single.available": ("T2-SINGLE", "19e3cffbac9de01ed6ddbffbb5c26bbd148e33d45017e568e7c2ab1af3453695", 287),
}
D2_MARKERS = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.consumed-before-exec": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.consumed-before-exec": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "u-single.consumed-before-exec": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.consumed-before-exec": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.consumed-before-exec": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.consumed-before-exec": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.consumed-before-exec": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
    "t-single.consumed-before-exec": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}


class TerminalAuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise TerminalAuditError(message)


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


def row(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def require_file(path: pathlib.Path, digest: str, mode: str = "0444") -> dict[str, Any]:
    value = row(path)
    require(value["sha256"] == digest and value["mode"] == mode, f"file identity drift: {path}")
    return value


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
    listed = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        require(relative not in listed, f"duplicate manifest row: {relative}")
        path = root / relative
        require(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
        listed[relative] = digest
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"}
    require(set(listed) == actual, f"manifest membership drift: {root}")
    return {"manifest": row(manifest), "entries": len(listed), "membership_exact": True}


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def run(command: Sequence[str], *, timeout: int = 600) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=timeout)
    require(result.returncode == 0, f"command failed {result.returncode}: {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-6000:]}")
    return result


def ssh_command(command: Sequence[str]) -> list[str]:
    return [
        "/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8",
        REMOTE, shlex.join(command),
    ]


def remote_projection_source() -> str:
    constants = "\n".join((
        f"PARENT=pathlib.Path({str(REMOTE_PARENT)!r})",
        f"STATE=pathlib.Path({str(REMOTE_STATE)!r})",
        f"U2=pathlib.Path({str(REMOTE_U2)!r})",
        f"D2_STATE=pathlib.Path({str(D2_STATE)!r})",
    ))
    body = r'''
import hashlib,json,os,pathlib,pwd,stat
def need(value,message):
    if not value: raise RuntimeError(message)
def sha(path):
    digest=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda:source.read(1024*1024),b''): digest.update(block)
    return digest.hexdigest()
def row(path):
    need(path.is_file() and not path.is_symlink(),f'missing or invalid file: {path}')
    return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}','size_bytes':path.stat().st_size,'sha256':sha(path)}
def manifest(root):
    path=root/'SHA256SUMS'; need(path.is_file(),f'manifest missing: {root}'); listed={}
    for line in path.read_text().splitlines():
        digest,relative=line.split('  ',1); member=root/relative
        need(relative not in listed and member.is_file() and sha(member)==digest,f'manifest mismatch: {member}')
        listed[relative]=digest
    actual={str(item.relative_to(root)) for item in root.rglob('*') if item.is_file() and item.name!='SHA256SUMS'}
    need(set(listed)==actual,f'manifest membership drift: {root}')
    return {'manifest':row(path),'entries':len(listed),'membership_exact':True}
need(PARENT.is_dir() and STATE.is_dir() and U2.is_dir(),'D3 terminal tree missing')
markers=[]
for path in sorted((STATE/'markers').iterdir()):
    value=row(path); value['name']=path.name; value['value']=json.loads(path.read_text()); markers.append(value)
d2_markers=[]
for path in sorted((D2_STATE/'markers').iterdir()):
    value=row(path); value['name']=path.name; value['value']=json.loads(path.read_text()); d2_markers.append(value)
owned=[]
for proc in pathlib.Path('/proc').iterdir():
    if not proc.name.isdigit() or int(proc.name) in {os.getpid(),os.getppid()}: continue
    try: environment=(proc/'environ').read_bytes()
    except (FileNotFoundError,PermissionError,ProcessLookupError): continue
    if b'LAY_V10_D1_RUN_ID=U2-SINGLE\x00' in environment or b'LAY_V10_D1_RUN_ID=T2-SINGLE\x00' in environment:
        owned.append(int(proc.name))
u2_receipt=U2/'D3_ROUTE_RECEIPT.json'; u2_observation=U2/'OBSERVATION.json'; u2_stderr=U2/'subject.stderr'
u2_state=STATE/'U2_SINGLE_STATE.json'; subject=U2/'subject'
result={
 'hostname':os.uname().nodename,'kernel':os.uname().release,'machine_id_sha256':sha(pathlib.Path('/etc/machine-id')),
 'sample_rate':pathlib.Path('/proc/sys/kernel/perf_event_max_sample_rate').read_text().strip(),
 'parent_mode':f'{stat.S_IMODE(PARENT.stat().st_mode):04o}','parent_entries':sorted(path.name for path in PARENT.iterdir()),
 'state_entries':sorted(path.name for path in STATE.iterdir()),'markers':markers,'owned_route_processes':sorted(owned),
 'bootstrap_manifest':manifest(PARENT/'bootstrap-v1'),'bootstrap_receipt':row(PARENT/'bootstrap-v1/D3_BOOTSTRAP_RECEIPT.json'),
 'u2_manifest':manifest(U2),'u2_receipt':row(u2_receipt),'u2_receipt_value':json.loads(u2_receipt.read_text()),
 'u2_observation':row(u2_observation),'u2_observation_value':json.loads(u2_observation.read_text()),
 'u2_stderr':row(u2_stderr),'u2_stderr_text':u2_stderr.read_text(),'u2_files':sorted(str(path.relative_to(U2)) for path in U2.rglob('*') if path.is_file()),
 'u2_subject':{'mode':f'{stat.S_IMODE(subject.stat().st_mode):04o}','uid':subject.stat().st_uid,'gid':subject.stat().st_gid,'entries':sorted(path.name for path in subject.iterdir())},
 'u2_state':row(u2_state),'u2_state_value':json.loads(u2_state.read_text()),
 't2_result_exists':(PARENT/'t2-single-v1').exists(),'t2_failure_exists':(PARENT/'t2-single-failure-v1').exists(),
 't2_state_exists':(STATE/'T2_SINGLE_STATE.json').exists(),
 'd2_markers':d2_markers,'d2_t_state':row(D2_STATE/'T_SINGLE_STATE.json'),'d2_t_state_value':json.loads((D2_STATE/'T_SINGLE_STATE.json').read_text()),
}
print(json.dumps(result,sort_keys=True,separators=(',',':')))
'''
    return "import pathlib\n" + constants + "\n" + body


def live_projection() -> dict[str, Any]:
    encoded = base64.b64encode(remote_projection_source().encode()).decode()
    decoder = f"import base64;exec(base64.b64decode({encoded!r}))"
    return json.loads(run(ssh_command(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", decoder]), timeout=120).stdout)


def copy_remote_u2(destination: pathlib.Path) -> dict[str, Any]:
    archive = run(
        ssh_command(["/usr/bin/sudo", "-n", "/usr/bin/tar", "--format=posix", "-C", str(REMOTE_U2), "-cf", "-", "."]),
        timeout=600,
    ).stdout
    destination.mkdir(mode=0o700)
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as source:
        members = source.getmembers()
        for member in members:
            relative = pathlib.PurePosixPath(member.name)
            parts = [part for part in relative.parts if part not in ("", ".")]
            require(not relative.is_absolute() and ".." not in parts, f"unsafe archive path: {member.name}")
            require(member.isdir() or member.isfile(), f"unsupported archive member: {member.name}")
            target = destination.joinpath(*parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True, mode=0o700)
            elif parts:
                target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
                stream = source.extractfile(member)
                require(stream is not None, f"archive member unreadable: {member.name}")
                write_new_bytes(target, stream.read())
    manifest = verify_sha256sums(destination)
    require(manifest["manifest"]["sha256"] == U2_MANIFEST_SHA256 and manifest["entries"] == 13,
            "copied U2 manifest drift")
    return {"archive_sha256": sha256_bytes(archive), "archive_size_bytes": len(archive), "manifest": manifest}


def local_runtime_snapshot() -> dict[str, Any]:
    launcher = pathlib.Path.home() / ".local/bin/lay"
    resolved = launcher.resolve(strict=True)
    return {"launcher": str(launcher), "resolved": str(resolved), "resolved_sha256": sha256_file(resolved)}


def verify_static_contract() -> dict[str, Any]:
    identities = {
        "paper": require_file(PAPER, PAPER_SHA256),
        "preflight": require_file(PREFLIGHT, PREFLIGHT_SHA256),
        "preflight_receipt": require_file(PREFLIGHT_RECEIPT, PREFLIGHT_RECEIPT_SHA256),
        "d2_terminal": require_file(D2_TERMINAL, D2_TERMINAL_SHA256),
        "bootstrap_audit": require_file(BOOTSTRAP_AUDIT, BOOTSTRAP_AUDIT_SHA256),
        "correction": require_file(CORRECTION, CORRECTION_SHA256),
        "local_controller": require_file(LOCAL_CONTROLLER, LOCAL_CONTROLLER_SHA256, "0755"),
        "remote_controller": require_file(REMOTE_CONTROLLER, REMOTE_CONTROLLER_SHA256, "0755"),
        "bootstrap_auditor": require_file(BOOTSTRAP_AUDITOR, BOOTSTRAP_AUDITOR_SHA256, "0755"),
        "terminal_auditor": row(AUDITOR),
    }
    bootstrap = json.loads(BOOTSTRAP_AUDIT.read_text())
    require(bootstrap.get("verdict") == "D3_BOOTSTRAP_AUDIT_PASS_EXECUTION_ADMITTED", "historical bootstrap audit drift")
    require(bootstrap.get("bootstrap_receipt_sha256") == BOOTSTRAP_RECEIPT_SHA256, "bootstrap receipt link drift")
    require(bootstrap.get("local_transport_defect")
            == "scp as e cannot traverse authoritative root-owned D3 parent mode 0700", "recorded transport defect drift")
    remote_source = REMOTE_CONTROLLER.read_text()
    compile(remote_source, str(REMOTE_CONTROLLER), "exec")
    tree = ast.parse(remote_source, filename=str(REMOTE_CONTROLLER))
    functions = {node.name: node for node in tree.body if isinstance(node, ast.FunctionDef)}
    require({"bootstrap", "route_once", "child_as_e"}.issubset(functions), "controller function graph drift")
    bootstrap_modes = []
    for node in ast.walk(functions["bootstrap"]):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "mkdir":
            if isinstance(node.func.value, ast.Name) and node.func.value.id == "parent_stage":
                bootstrap_modes.extend(ast.literal_eval(keyword.value) for keyword in node.keywords if keyword.arg == "mode")
    require(bootstrap_modes == [0o700], f"bootstrap parent mode source drift: {bootstrap_modes}")
    return {
        "identities": identities,
        "historical_bootstrap_audit_verdict": bootstrap["verdict"],
        "effective_bootstrap_admission": "INVALID",
        "bootstrap_parent_mode_from_source": "0700",
        "subject_account": "e",
        "correction_verdict": "BLOCKED_PROVENANCE",
    }


def validate_live(value: Mapping[str, Any]) -> dict[str, Any]:
    require(value.get("hostname") == REMOTE_HOSTNAME and value.get("kernel") == "6.8.0-124-generic", "host drift")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256 and value.get("sample_rate") == "8000", "host identity drift")
    require(value.get("parent_mode") == "0700", "D3 parent mode drift")
    require(value.get("parent_entries") == ["bootstrap-v1", "u2-single-v1"], "D3 result membership drift")
    require(value.get("state_entries") == ["STATE.json", "U2_SINGLE_STATE.json", "markers", "route.lock"], "D3 state membership drift")
    require(value.get("owned_route_processes") == [], "D3 route process still active")
    require(value.get("bootstrap_receipt", {}).get("sha256") == BOOTSTRAP_RECEIPT_SHA256, "bootstrap receipt drift")
    require(value.get("u2_manifest", {}).get("manifest", {}).get("sha256") == U2_MANIFEST_SHA256
            and value.get("u2_manifest", {}).get("entries") == 13, "U2 manifest drift")
    require(value.get("u2_receipt", {}).get("sha256") == U2_RECEIPT_SHA256, "U2 receipt drift")
    require(value.get("u2_observation", {}).get("sha256") == U2_OBSERVATION_SHA256, "U2 observation drift")
    require(value.get("u2_stderr", {}).get("sha256") == U2_STDERR_SHA256, "U2 stderr drift")
    require("Permission denied (os error 13)" in value.get("u2_stderr_text", ""), "U2 permission failure absent")
    receipt = value.get("u2_receipt_value", {})
    require(receipt.get("verdict") == "BLOCKED_PROVENANCE" and receipt.get("retry_permitted") is False, "U2 verdict drift")
    dispatch = receipt.get("dispatch", {})
    require(dispatch.get("selected_cause") == "provenance" and dispatch.get("selected_rank") == 0,
            "U2 dispatch drift")
    require(dispatch.get("reason") == "subject evidence unavailable: D3Error: subject evidence incomplete",
            "U2 dispatch reason drift")
    observation = value.get("u2_observation_value", {})
    require(observation.get("status") == {"returncode": 101, "timed_out": False}, "U2 process status drift")
    require(observation.get("perf_record_invocations") == 0 and observation.get("perf_stat_invocations") == 0
            and observation.get("pmu_events_opened") == 0 and observation.get("subject_executions") == 1,
            "U2 execution ledger drift")
    require(value.get("u2_subject", {}).get("entries") == [], "unexpected U2 subject artifacts")
    state = value.get("u2_state_value", {})
    require(value.get("u2_state", {}).get("sha256") == U2_STATE_SHA256 and value.get("u2_state", {}).get("mode") == "0400",
            "U2 state identity drift")
    require(state.get("state") == "BLOCKED_PROVENANCE" and state.get("receipt_sha256") == U2_RECEIPT_SHA256
            and state.get("retry_permitted") is False, "U2 terminal state drift")
    markers = {item["name"]: item for item in value.get("markers", [])}
    require(set(markers) == set(D3_MARKERS), "D3 terminal marker set drift")
    for name, (route, digest, size) in D3_MARKERS.items():
        item = markers[name]
        require(item.get("mode") == "0400" and item.get("sha256") == digest and item.get("size_bytes") == size,
                f"D3 marker identity drift: {name}")
        require(item.get("value", {}).get("route") == route and item.get("value", {}).get("retry_permitted") is False,
                f"D3 marker body drift: {name}")
    require(not value.get("t2_result_exists") and not value.get("t2_failure_exists") and not value.get("t2_state_exists"),
            "T2 was touched after blocked U2")
    d2_markers = {item["name"]: item for item in value.get("d2_markers", [])}
    require(set(d2_markers) == set(D2_MARKERS), "D2 marker membership drift")
    for name, (digest, size) in D2_MARKERS.items():
        item = d2_markers[name]
        require(item.get("mode") == "0400" and item.get("sha256") == digest and item.get("size_bytes") == size,
                f"D2 marker drift: {name}")
        require(item.get("value", {}).get("task_id") == D2_TASK_ID
                and item.get("value", {}).get("transaction_id") == D2_TRANSACTION_ID,
                f"D2 marker namespace drift: {name}")
    d2_state = value.get("d2_t_state_value", {})
    require(d2_state.get("state") == "BLOCKED_PROVENANCE" and d2_state.get("retry_permitted") is False,
            "D2 terminal state drift")
    return {
        "verdict": "BLOCKED_PROVENANCE",
        "selected_cause": "provenance",
        "u2_marker_consumed": True,
        "t2_marker_retired_unconsumed": True,
        "u2_subject_executions": 1,
        "t2_subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "d2_markers_unchanged": 11,
    }


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"terminal audit result exists: {RESULT}")
    require_file(SSH_IDENTITY, sha256_file(SSH_IDENTITY), "0600")
    static = verify_static_contract()
    projection = ast.parse(remote_projection_source(), filename="<d3-terminal-audit-remote-projection>")
    mutating = {
        node.func.attr for node in ast.walk(projection)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        and node.func.attr in {"chmod", "mkdir", "rename", "replace", "rmdir", "touch", "unlink", "write_bytes", "write_text"}
    }
    require(not mutating, f"remote projection mutation reachable: {sorted(mutating)}")
    return {
        "schema": "lay.v10.e1-traversal-d3-terminal-audit-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D3_TERMINAL_AUDITOR_VERIFIED_UNRUN",
        "static_contract": static,
        "remote_projection_mutations": 0,
        "remote_writes": 0,
        "marker_mutations": 0,
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
    }


def audit() -> dict[str, Any]:
    check = self_check()
    runtime_before = local_runtime_snapshot()
    remote_before = live_projection()
    terminal = validate_live(remote_before)
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        copied = copy_remote_u2(stage / "REMOTE_U2_EVIDENCE")
        require(sha256_file(stage / "REMOTE_U2_EVIDENCE/D3_ROUTE_RECEIPT.json") == U2_RECEIPT_SHA256,
                "copied U2 receipt drift")
        remote_after = live_projection()
        validate_live(remote_after)
        require(remote_after == remote_before, "remote terminal projection changed during audit")
        runtime_after = local_runtime_snapshot()
        require(runtime_after == runtime_before, "installed runtime changed during terminal audit")
        receipt = {
            "schema": "lay.v10.e1-traversal-d3-terminal-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "terminal_scope": "D3 single-worker estimator recovery",
            "selected_cause": "provenance",
            "selected_rank": 0,
            "reason": "root-owned D3 parent mode 0700 prevented subject user e from publishing required U2 evidence",
            "historical_bootstrap_audit": {
                "verdict": "D3_BOOTSTRAP_AUDIT_PASS_EXECUTION_ADMITTED",
                "receipt_sha256": BOOTSTRAP_AUDIT_SHA256,
                "effective_execution_admission": "INVALID",
                "receipt_modified": False,
            },
            "correction": check["static_contract"]["identities"]["correction"],
            "u2": {
                "receipt_sha256": U2_RECEIPT_SHA256,
                "observation_sha256": U2_OBSERVATION_SHA256,
                "stderr_sha256": U2_STDERR_SHA256,
                "remote_manifest_sha256": U2_MANIFEST_SHA256,
                "remote_manifest_entries": 13,
                "process_exit": 101,
                "marker_consumed_before_effect": True,
                "subject_executions": 1,
                "retry_permitted": False,
            },
            "t2": {
                "marker": "t2-single.available",
                "status": "RETIRED_UNCONSUMED_BY_TERMINAL_D3",
                "subject_executions": 0,
                "perf_record": 0,
                "pmu_events_opened": 0,
            },
            "terminal_projection": terminal,
            "remote_projection_sha256": sha256_bytes(canonical_json_bytes(remote_before)),
            "remote_projection_stable": True,
            "copied_remote_evidence": copied,
            "d2_marker_mutations": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "runtime_authority_changed": False,
            "optimization_authority": False,
            "retry_permitted": False,
            "next_action_admitted": "none within D3; any future measurement requires a new paper namespace",
        }
        write_new_json(stage / "D3_TERMINAL_AUDIT_RECEIPT.json", receipt)
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "REMOTE_BEFORE.json", remote_before)
        write_new_json(stage / "REMOTE_AFTER.json", remote_after)
        write_new_bytes(stage / "terminal-auditor.py", AUDITOR.read_bytes())
        write_new_bytes(stage / "correction-v2.md", CORRECTION.read_bytes())
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, RESULT)
        fsync_directory(RESULT.parent)
    except BaseException:
        if stage.exists(): shutil.rmtree(stage)
        raise
    return {
        **receipt,
        "receipt_sha256": sha256_file(RESULT / "D3_TERMINAL_AUDIT_RECEIPT.json"),
        "result": str(RESULT),
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=EXTERNAL_ACTIONS)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D3 TERMINAL AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
