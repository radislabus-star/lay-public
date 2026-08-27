#!/usr/bin/env python3
"""Independent read-only audit of the D3 two-marker bootstrap namespace."""

from __future__ import annotations

import argparse
import ast
import base64
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
TASK_ID = "slice8b-v10-e1-traversal-d3-estimator-recovery-v1-20260826"
TRANSACTION_ID = "e88555465ee51b7caed891217e8941ceb0b412ed82981d7c88cde71c3eb452e1"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

LOCAL_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d3-estimator-recovery.py"
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d3-estimator-recovery-remote.py"
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
LOCAL_BOOTSTRAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_BOOTSTRAP_V1_2026-08-26"
)
AUDIT_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_BOOTSTRAP_AUDIT_V1_2026-08-26"
)

PAPER_SHA256 = "ebe80974392a05527bea67944f381cfd2f74fb0be1c5b2ba3bf4a5aba22be11a"
PREFLIGHT_SHA256 = "f9fba59409fe56ae742738d66bbae886b24a165dc34e258529e3f6de6710456e"
PREFLIGHT_RECEIPT_SHA256 = "09a66ecabebd95e1a8b76e459686bc41a0146f7bedbabf18dbc562096e24868d"
D2_TERMINAL_SHA256 = "75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608"

REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
D2_TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
D2_TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
D2_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / D2_TASK_ID

EXTERNAL_ACTIONS = ("self-check", "audit")
ROUTE_ORDER = ("U2-SINGLE", "T2-SINGLE")
D3_MARKERS = {
    "u2-single.available": ("U2-SINGLE", "a212d2edf720551b70f9245d62487fd7834f3796ebfe5d9759a3001fd28f9502", 287),
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


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
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


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def require_file(
    path: pathlib.Path,
    *,
    digest: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    value = row(path)
    if digest is not None:
        require(value["sha256"] == digest, f"SHA drift: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size drift: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode drift: {path}")
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


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o600) -> None:
    write_new_bytes(path, canonical_json_bytes(value), mode)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_sha256sums(root: pathlib.Path) -> None:
    lines = []
    for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS"):
        lines.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(lines).encode())


def verify_sha256sums(root: pathlib.Path) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"SHA256SUMS missing: {root}")
    listed: dict[str, str] = {}
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        require(relative not in listed, f"duplicate manifest row: {relative}")
        path = root / relative
        require(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
        listed[relative] = digest
    actual = {
        str(path.relative_to(root))
        for path in root.rglob("*")
        if path.is_file() and path.name != "SHA256SUMS"
    }
    require(set(listed) == actual, f"manifest membership drift: {root}")
    return {"manifest": row(manifest), "entries": len(listed), "membership_exact": True}


def writable_paths(root: pathlib.Path) -> list[str]:
    return [
        str(path.relative_to(root)) if path != root else "."
        for path in [root, *root.rglob("*")]
        if path.stat().st_mode & 0o222
    ]


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def run(
    command: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: int = 600,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    require(
        result.returncode == 0,
        f"command failed {result.returncode}: {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-6000:]}",
    )
    return result


def remote_projection_source() -> str:
    constants = "\n".join(
        (
            f"TASK_ID={TASK_ID!r}",
            f"TRANSACTION_ID={TRANSACTION_ID!r}",
            f"D2_TASK_ID={D2_TASK_ID!r}",
            f"D2_TRANSACTION_ID={D2_TRANSACTION_ID!r}",
            f"PARENT=pathlib.Path({str(REMOTE_PARENT)!r})",
            f"STATE=pathlib.Path({str(REMOTE_STATE)!r})",
            f"D2_STATE=pathlib.Path({str(D2_STATE)!r})",
        )
    )
    body = r'''
import hashlib,json,os,pathlib,stat

def need(value,message):
    if not value:
        raise RuntimeError(message)

def sha(path):
    digest=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda:source.read(1024*1024),b''):
            digest.update(block)
    return digest.hexdigest()

def row(path):
    need(path.is_file() and not path.is_symlink(),f'missing or invalid file: {path}')
    return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}',
            'size_bytes':path.stat().st_size,'sha256':sha(path)}

def manifest(root):
    manifest_path=root/'SHA256SUMS'
    need(manifest_path.is_file(),f'missing manifest: {root}')
    listed={}
    for line in manifest_path.read_text().splitlines():
        digest,relative=line.split('  ',1)
        need(relative not in listed,f'duplicate manifest row: {relative}')
        member=root/relative
        need(member.is_file() and sha(member)==digest,f'manifest mismatch: {member}')
        listed[relative]=digest
    actual={str(path.relative_to(root)) for path in root.rglob('*')
            if path.is_file() and path.name!='SHA256SUMS'}
    need(set(listed)==actual,f'manifest membership drift: {root}')
    return {'manifest':row(manifest_path),'entries':len(listed),'membership_exact':True}

need(PARENT.is_dir(),'D3 parent missing')
need(STATE.is_dir(),'D3 state missing')
bootstrap=PARENT/'bootstrap-v1'
need(bootstrap.is_dir(),'D3 bootstrap missing')
markers=STATE/'markers'
need(markers.is_dir(),'D3 marker directory missing')
d3_markers=[]
for path in sorted(markers.iterdir()):
    value=row(path)
    value['name']=path.name
    value['value']=json.loads(path.read_text())
    d3_markers.append(value)
d2_markers=[]
for path in sorted((D2_STATE/'markers').iterdir()):
    value=row(path)
    value['name']=path.name
    value['value']=json.loads(path.read_text())
    d2_markers.append(value)
writable=[]
for path in [bootstrap,*bootstrap.rglob('*')]:
    if path.stat().st_mode & 0o222:
        writable.append(str(path.relative_to(bootstrap)) if path!=bootstrap else '.')
owned=[]
for proc in pathlib.Path('/proc').iterdir():
    if not proc.name.isdigit() or int(proc.name) in {os.getpid(),os.getppid()}:
        continue
    try:
        environment=(proc/'environ').read_bytes()
    except (FileNotFoundError,PermissionError,ProcessLookupError):
        continue
    if b'LAY_V10_D1_RUN_ID=U2-SINGLE\x00' in environment or b'LAY_V10_D1_RUN_ID=T2-SINGLE\x00' in environment:
        owned.append(int(proc.name))
result={
    'hostname':os.uname().nodename,
    'kernel':os.uname().release,
    'machine_id_sha256':sha(pathlib.Path('/etc/machine-id')),
    'sample_rate':pathlib.Path('/proc/sys/kernel/perf_event_max_sample_rate').read_text().strip(),
    'parent_mode':f'{stat.S_IMODE(PARENT.stat().st_mode):04o}',
    'parent_entries':sorted(path.name for path in PARENT.iterdir()),
    'state_entries':sorted(path.name for path in STATE.iterdir()),
    'bootstrap_manifest':manifest(bootstrap),
    'bootstrap_receipt':row(bootstrap/'D3_BOOTSTRAP_RECEIPT.json'),
    'bootstrap_value':json.loads((bootstrap/'D3_BOOTSTRAP_RECEIPT.json').read_text()),
    'bootstrap_inputs':{path.name:row(path) for path in sorted((bootstrap/'inputs').iterdir())},
    'bootstrap_writable_paths':writable,
    'state':row(STATE/'STATE.json'),
    'state_value':json.loads((STATE/'STATE.json').read_text()),
    'route_lock':row(STATE/'route.lock'),
    'route_lock_text':(STATE/'route.lock').read_text(),
    'd3_markers':d3_markers,
    'd2_markers':d2_markers,
    'd2_t_state':row(D2_STATE/'T_SINGLE_STATE.json'),
    'd2_t_state_value':json.loads((D2_STATE/'T_SINGLE_STATE.json').read_text()),
    'owned_route_processes':sorted(owned),
}
print(json.dumps(result,sort_keys=True,separators=(',',':')))
'''
    return "import pathlib\n" + constants + "\n" + body


def live_projection() -> dict[str, Any]:
    source = remote_projection_source().encode()
    encoded = base64.b64encode(source).decode()
    decoder = f"import base64;exec(base64.b64decode({encoded!r}))"
    remote_command = shlex.join(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", decoder])
    result = run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_IDENTITY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            REMOTE,
            remote_command,
        ],
        timeout=120,
    )
    return json.loads(result.stdout)


def verify_controller_graph(local_source: str, remote_source: str) -> dict[str, Any]:
    compile(local_source, str(LOCAL_CONTROLLER), "exec")
    compile(remote_source, str(REMOTE_CONTROLLER), "exec")
    namespace: dict[str, Any] = {"__name__": "d3_remote_audit_registry", "__file__": str(REMOTE_CONTROLLER)}
    exec(compile(remote_source, str(REMOTE_CONTROLLER), "exec"), namespace)
    require(tuple(namespace["ROUTE_ORDER"]) == ROUTE_ORDER, "route order drift")
    require(set(namespace["ROUTES"]) == set(ROUTE_ORDER), "route registry drift")
    require(namespace["U2_EDGES"] == 502_915_120, "U2 denominator drift")
    require(namespace["T2_EDGES"] == 528_060_876, "T2 denominator drift")
    require(tuple(namespace["PERF_RECORD_PREFIX"])[3:]
            == ("record", "--buildid-all", "--sample-cpu", "--timestamp", "--event",
                "task-clock:u", "--count", "200000"), "perf command drift")
    require(namespace["subject_command"]()[0:3]
            == ["/usr/bin/taskset", "--cpu-list", "6"], "staging command drift")
    require(namespace["subject_command"]()[-2:] == ["--nocapture", "--test-threads=1"], "subject tail drift")
    readers = namespace["reader_commands"](pathlib.Path("/tmp/perf.data"))
    require(set(readers) == {"evlist", "samples", "raw-records", "buildids"}, "reader graph drift")
    require(tuple(namespace["U2_DISPATCH"]) == (
        ("provenance", "BLOCKED_PROVENANCE"),
        ("thermal", "BLOCKED_THERMAL"),
        ("semantic", "BLOCKED_SEMANTIC"),
    ), "U2 dispatch drift")
    require(tuple(namespace["T2_DISPATCH"]) == (
        ("provenance", "BLOCKED_PROVENANCE"),
        ("thermal", "BLOCKED_THERMAL"),
        ("capability", "BLOCKED_CAPABILITY"),
        ("bucket_map", "BLOCKED_BUCKET_MAP"),
        ("perturbation", "BLOCKED_PERTURBATION"),
        ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
    ), "T2 dispatch drift")
    tree = ast.parse(remote_source, filename=str(REMOTE_CONTROLLER))
    calls = {
        node.func.attr
        for node in ast.walk(tree)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
    }
    require("kill" not in calls and "killpg" not in calls, "signal lifecycle reachable")
    route_nodes = [
        node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "route_once"
    ]
    require(len(route_nodes) == 1, "route_once function drift")
    stage_modes = []
    for node in ast.walk(route_nodes[0]):
        if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Attribute):
            continue
        if node.func.attr != "mkdir" or not isinstance(node.func.value, ast.Name) or node.func.value.id != "stage":
            continue
        modes = [keyword.value for keyword in node.keywords if keyword.arg == "mode"]
        require(len(modes) == 1, "route stage mkdir mode missing")
        stage_modes.append(ast.literal_eval(modes[0]))
    require(stage_modes == [0o755], f"route stage mode drift: {stage_modes}")
    return {
        "routes": list(ROUTE_ORDER),
        "u2_edges": namespace["U2_EDGES"],
        "t2_edges": namespace["T2_EDGES"],
        "perf_record_routes": ["T2-SINGLE"],
        "perf_stat_routes": [],
        "pid_attach": False,
        "signal_lifecycle": False,
        "route_stage_mode": "0755",
    }


def verify_bootstrap_sources() -> dict[str, Any]:
    require(not LOCAL_BOOTSTRAP.exists(), "unexpected local bootstrap mirror exists after failed scp transport")
    current_local = LOCAL_CONTROLLER.read_bytes()
    current_remote = REMOTE_CONTROLLER.read_bytes()
    current_auditor = AUDITOR.read_bytes()
    command_graph = verify_controller_graph(current_local.decode(), current_remote.decode())
    return {
        "local_controller_sha256": sha256_bytes(current_local),
        "remote_controller_sha256": sha256_bytes(current_remote),
        "auditor_sha256": sha256_bytes(current_auditor),
        "command_graph": command_graph,
        "local_bootstrap_mirror_present": False,
        "transport_defect": "scp as e cannot traverse authoritative root-owned D3 parent mode 0700",
    }


def validate_live_projection(value: Mapping[str, Any]) -> dict[str, Any]:
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote hostname drift")
    require(value.get("kernel") == "6.8.0-124-generic", "remote kernel drift")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote machine identity drift")
    require(value.get("sample_rate") == "8000", "remote sample-rate drift")
    require(value.get("parent_mode") == "0700", "D3 parent mode drift")
    require(value.get("parent_entries") == ["bootstrap-v1"], "D3 parent entry drift")
    require(value.get("state_entries") == ["STATE.json", "markers", "route.lock"], "D3 state entry drift")
    require(value.get("bootstrap_writable_paths") == [], "remote bootstrap evidence writable")
    require(value.get("owned_route_processes") == [], "D3 route process active during audit")
    require(value.get("route_lock_text") == "d3-estimator-recovery-v1\n", "D3 route lock drift")
    require(value.get("route_lock", {}).get("mode") == "0400", "D3 route lock mode drift")
    state = value.get("state_value", {})
    require(state == {
        "markers_created": 2,
        "markers_consumed": 0,
        "markers_expected": 2,
        "retry_permitted": False,
        "schema": "lay.v10.e1-traversal-d3-state.v1",
        "state": "D3_NAMESPACE_CREATED_UNAUDITED",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
    }, "D3 live state drift")
    require(value.get("state", {}).get("mode") == "0400", "D3 state mode drift")
    markers = {item["name"]: item for item in value.get("d3_markers", [])}
    require(set(markers) == set(D3_MARKERS), "D3 live marker set drift")
    for name, (route, digest, size) in D3_MARKERS.items():
        item = markers[name]
        require(item.get("mode") == "0400" and item.get("sha256") == digest
                and item.get("size_bytes") == size, f"D3 live marker identity drift: {name}")
        require(item.get("value") == {
            "one_shot": True,
            "retry_permitted": False,
            "route": route,
            "schema": "lay.v10.e1-traversal-d3-marker.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
        }, f"D3 live marker body drift: {name}")
    receipt = value.get("bootstrap_value", {})
    require(receipt.get("verdict") == "D3_NAMESPACE_CREATED_UNAUDITED", "remote bootstrap verdict drift")
    require(receipt.get("task_id") == TASK_ID and receipt.get("transaction_id") == TRANSACTION_ID,
            "remote bootstrap namespace drift")
    require(receipt.get("markers_created") == 2 and receipt.get("markers_consumed") == 0,
            "remote bootstrap marker ledger drift")
    require(receipt.get("cargo_invocations") == 0 and receipt.get("rustc_compilations") == 0,
            "remote bootstrap build ledger drift")
    require(receipt.get("perf_record") == 0 and receipt.get("perf_stat") == 0
            and receipt.get("pmu_events_opened") == 0 and receipt.get("subject_executions") == 0,
            "remote bootstrap execution ledger drift")
    require(receipt.get("runtime_authority_changed") is False, "remote bootstrap runtime ledger drift")
    receipt_markers = {item["name"]: item for item in receipt.get("markers", [])}
    require(set(receipt_markers) == set(D3_MARKERS), "remote receipt marker set drift")
    for name, (_route, digest, size) in D3_MARKERS.items():
        item = receipt_markers[name]
        require(item.get("mode") == "0400" and item.get("sha256") == digest
                and item.get("size_bytes") == size, f"remote receipt marker identity drift: {name}")
    expected_inputs = {
        "paper.md": PAPER_SHA256,
        "preflight-v2.json": PREFLIGHT_SHA256,
        "preflight-v2-receipt.json": PREFLIGHT_RECEIPT_SHA256,
        "d2-terminal-receipt.json": D2_TERMINAL_SHA256,
        "local-controller.py": receipt.get("payload", {}).get("local_controller_sha256"),
        "remote-controller.py": receipt.get("payload", {}).get("remote_controller_sha256"),
    }
    inputs = value.get("bootstrap_inputs", {})
    require(set(inputs) == set(expected_inputs), "remote bootstrap input set drift")
    for name, digest in expected_inputs.items():
        require(inputs[name].get("mode") == "0444" and inputs[name].get("sha256") == digest,
                f"remote bootstrap input drift: {name}")
    d2_markers = {item["name"]: item for item in value.get("d2_markers", [])}
    require(set(d2_markers) == set(D2_MARKERS), "D2 live marker set drift")
    for name, (digest, size) in D2_MARKERS.items():
        item = d2_markers[name]
        require(item.get("mode") == "0400" and item.get("sha256") == digest
                and item.get("size_bytes") == size, f"D2 marker drift: {name}")
        body = item.get("value", {})
        require(body.get("task_id") == D2_TASK_ID and body.get("transaction_id") == D2_TRANSACTION_ID,
                f"D2 marker namespace drift: {name}")
        require(body.get("retry_permitted") is False, f"D2 marker retry drift: {name}")
    d2_state = value.get("d2_t_state_value", {})
    require(d2_state.get("state") == "BLOCKED_PROVENANCE", "D2 terminal state drift")
    require(d2_state.get("retry_permitted") is False, "D2 retry authority drift")
    require(d2_state.get("receipt_sha256")
            == "afaeb7d3caffb1967dd76021e42b94664803cef5d0ed72ec574fb54526a8fa0d",
            "D2 route receipt drift")
    return {
        "markers_expected": 2,
        "markers_created": 2,
        "markers_consumed": 0,
        "available": sorted(markers),
        "d2_markers": sorted(d2_markers),
        "d2_terminal": d2_state,
        "remote_bootstrap_receipt": value.get("bootstrap_receipt"),
        "remote_bootstrap_manifest": value.get("bootstrap_manifest"),
        "remote_bootstrap_inputs": inputs,
    }


def self_check() -> dict[str, Any]:
    require(not AUDIT_RESULT.exists(), f"audit result already exists: {AUDIT_RESULT}")
    require_file(PAPER, digest=PAPER_SHA256, size=9_473, mode="0444")
    require_file(PREFLIGHT, digest=PREFLIGHT_SHA256, size=27_120, mode="0444")
    require_file(PREFLIGHT_RECEIPT, digest=PREFLIGHT_RECEIPT_SHA256, size=11_073, mode="0444")
    terminal = require_file(D2_TERMINAL, digest=D2_TERMINAL_SHA256, size=14_907, mode="0444")
    terminal_value = json.loads(D2_TERMINAL.read_text())
    require(terminal_value.get("verdict") == "BLOCKED_PROVENANCE", "D2 terminal verdict drift")
    require_file(SSH_IDENTITY, mode="0600")
    sources = verify_bootstrap_sources()
    projection_tree = ast.parse(remote_projection_source(), filename="<d3-bootstrap-audit-remote-projection>")
    mutating = {
        node.func.attr
        for node in ast.walk(projection_tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and node.func.attr in {"chmod", "mkdir", "rename", "replace", "rmdir", "touch", "unlink", "write_bytes", "write_text"}
    }
    require(not mutating, f"remote projection mutation calls: {sorted(mutating)}")
    return {
        "schema": "lay.v10.e1-traversal-d3-bootstrap-audit-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D3_BOOTSTRAP_AUDITOR_VERIFIED_UNRUN",
        "auditor": row(AUDITOR),
        "d2_terminal": terminal,
        "bootstrap_sources": sources,
        "remote_projection_mutations": 0,
        "remote_writes": 0,
        "marker_mutations": 0,
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
    }


def audit() -> dict[str, Any]:
    check = self_check()
    before = live_projection()
    live = validate_live_projection(before)
    sources = check["bootstrap_sources"]
    payload = before.get("bootstrap_value", {}).get("payload", {})
    require(payload.get("local_controller_sha256") == sources["local_controller_sha256"],
            "bootstrap/current local controller SHA drift")
    require(payload.get("remote_controller_sha256") == sources["remote_controller_sha256"],
            "bootstrap/current remote controller SHA drift")
    after = live_projection()
    validate_live_projection(after)
    require(after == before, "remote projection changed during read-only audit")
    receipt = {
        "schema": "lay.v10.e1-traversal-d3-bootstrap-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D3_BOOTSTRAP_AUDIT_PASS_EXECUTION_ADMITTED",
        "local_controller_sha256": sources["local_controller_sha256"],
        "remote_controller_sha256": sources["remote_controller_sha256"],
        "bootstrap_auditor_sha256": sources["auditor_sha256"],
        "bootstrap_receipt_sha256": before["bootstrap_receipt"]["sha256"],
        "remote_bootstrap_manifest": before["bootstrap_manifest"],
        "command_graph": sources["command_graph"],
        "local_bootstrap_mirror_present": False,
        "local_transport_defect": sources["transport_defect"],
        "authoritative_remote_bootstrap_complete": True,
        "live_projection": live,
        "live_projection_sha256": sha256_bytes(canonical_json_bytes(before)),
        "live_projection_stable": True,
        "markers_expected": 2,
        "markers_created": 2,
        "markers_consumed": 0,
        "markers_available": ["u2-single.available", "t2-single.available"],
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "subject_executions": 0,
        "d2_marker_mutations": 0,
        "remote_writes": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "next_action_admitted": "U2-SINGLE only; T2-SINGLE remains gated by exact U2_SINGLE_PASS",
    }
    stage = pathlib.Path(f"{AUDIT_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new_json(stage / "D3_BOOTSTRAP_AUDIT_RECEIPT.json", receipt)
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "REMOTE_BEFORE.json", before)
        write_new_json(stage / "REMOTE_AFTER.json", after)
        write_new_bytes(stage / "auditor.py", AUDITOR.read_bytes())
        write_new_bytes(stage / "local-controller.py", LOCAL_CONTROLLER.read_bytes())
        write_new_bytes(stage / "remote-controller.py", REMOTE_CONTROLLER.read_bytes())
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, AUDIT_RESULT)
        fsync_directory(AUDIT_RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return {
        **receipt,
        "receipt_sha256": sha256_file(AUDIT_RESULT / "D3_BOOTSTRAP_AUDIT_RECEIPT.json"),
        "audit_result": str(AUDIT_RESULT),
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
        print(f"D3 BOOTSTRAP AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
