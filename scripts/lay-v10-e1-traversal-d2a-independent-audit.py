#!/usr/bin/env python3
"""Independent read-only audit of the sealed primary-only D2-A V2 state."""

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
import tempfile
from typing import Any, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_D2A = REMOTE_PARENT / "d2a-v1"
REMOTE_FAILURE = REMOTE_PARENT / "d2a-failure-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

AUDITOR = pathlib.Path(__file__).resolve()
PROJECT_ROOT = AUDITOR.parents[1]
CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d2-primary-only.py"
D2A_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_V2_2026-08-25"
)
AUDIT_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "D2A_INDEPENDENT_AUDIT_V1_2026-08-25"
)

EXPECTED = {
    "controller": "9329a32b9e4e9edf5d83ddf624e8c9ce6a456494057f4ea3ef5aff6f382d6ec0",
    "controller_size": 64_492,
    "d2a_receipt": "998ca180a976384acb215b9e72a8d956fd830fdf6a1c0641b59eea10cbb00e0f",
    "d2a_receipt_size": 21_910,
    "local_receipt": "e5277fdd7472325d6589bcfabc782d9b824fc1fe20beee2cb1ed4419e2b412bc",
    "local_sha256sums": "0376930d31a9865b49e39966c4d5c050a1a738b70b27e01bb9db6f0239ebfe86",
    "remote_sha256sums": "8d9581f8b4bce2b8cd99683c3b718ef3ea338bf97ab01f3ab14186bda09319c7",
    "assembled_source": "6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181",
    "assembled_source_size": 204_722,
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "production_prefix_size": 39_047,
    "transaction_id": "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7",
    "state": "fb7de0be1dbb7a99c2ddcb2bd1dbc7f469d4fc975b6a564546fd6994c196075a",
    "route_lock": "ddfafcaec3c8068ea0b853cb8a34cf0b40408fbbdc137a6dae3932b5396c3c5d",
    "preflight": "e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a",
}

MARKER_ROUTES = {
    "build.available": "BUILD",
    "bucket-map.available": "BUCKET-MAP",
    "parity.available": "PARITY",
    "u-single.available": "U-SINGLE",
    "u-fixed.available": "U-FIXED",
    "u-reversed.available": "U-REVERSED",
    "v-fixed-instr.available": "V-FIXED-INSTR",
    "v-reversed-instr.available": "V-REVERSED-INSTR",
    "t-single.available": "T-SINGLE",
    "t-fixed.available": "T-FIXED",
    "t-reversed.available": "T-REVERSED",
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


def file_identity(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
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
    row = file_identity(path)
    if digest is not None:
        require(row["sha256"] == digest, f"SHA-256 mismatch: {path}")
    if size is not None:
        require(row["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(row["mode"] == mode, f"mode mismatch: {path}")
    return row


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    ).encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new_bytes(
        path,
        json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n",
        mode,
    )


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def remove_tree(root: pathlib.Path) -> None:
    if not root.exists():
        return
    for path in sorted(root.rglob("*")):
        path.chmod(0o700 if path.is_dir() else 0o600)
    root.chmod(0o700)
    shutil.rmtree(root)


def manifest_rows(root: pathlib.Path, excluded: set[str] | None = None) -> list[dict[str, Any]]:
    skip = excluded or set()
    rows = []
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
        relative = path.relative_to(root).as_posix()
        if relative in skip:
            continue
        row = file_identity(path)
        row["path"] = relative
        rows.append(row)
    return rows


def write_sha256sums(root: pathlib.Path) -> None:
    rows = manifest_rows(root, {"SHA256SUMS"})
    value = "".join(f"{row['sha256']}  {row['path']}\n" for row in rows).encode()
    write_new_bytes(root / "SHA256SUMS", value, 0o444)


def verify_sha256sums(root: pathlib.Path) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    listed: dict[str, str] = {}
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, relative = line.split("  ", 1)
        require(relative not in listed, f"duplicate manifest path: {relative}")
        listed[relative] = digest
        path = root / relative
        require(path.is_file(), f"manifest member missing: {path}")
        require(sha256_file(path) == digest, f"manifest SHA mismatch: {path}")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path != manifest
    }
    require(set(listed) == actual, f"manifest membership mismatch: {root}")
    return {
        "manifest": file_identity(manifest),
        "entries": len(listed),
        "membership_exact": True,
        "all_sha256_match": True,
    }


def writable_paths(root: pathlib.Path) -> list[str]:
    paths = [root, *root.rglob("*")]
    return [str(path.relative_to(root) or ".") for path in paths if path.stat().st_mode & 0o222]


def run(command: Sequence[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False
    )
    if check and result.returncode != 0:
        detail = result.stderr.decode(errors="replace")[-4000:]
        raise AuditError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n{detail}"
        )
    return result


def remote_probe_source() -> str:
    constants = "\n".join(
        (
            f"PARENT=pathlib.Path({json.dumps(str(REMOTE_PARENT))})",
            f"D2A=pathlib.Path({json.dumps(str(REMOTE_D2A))})",
            f"FAILURE=pathlib.Path({json.dumps(str(REMOTE_FAILURE))})",
            f"STATE=pathlib.Path({json.dumps(str(REMOTE_STATE))})",
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
    need(path.is_file(),f'missing file: {path}')
    return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}',
            'size_bytes':path.stat().st_size,'sha256':sha(path)}

def manifest(root):
    path=root/'SHA256SUMS'
    need(path.is_file(),f'missing manifest: {path}')
    listed={}
    for line in path.read_text().splitlines():
        digest,relative=line.split('  ',1)
        need(relative not in listed,f'duplicate: {relative}')
        member=root/relative
        need(member.is_file(),f'missing: {member}')
        need(sha(member)==digest,f'hash mismatch: {member}')
        listed[relative]=digest
    actual={str(path.relative_to(root)) for path in root.rglob('*')
            if path.is_file() and path != root/'SHA256SUMS'}
    need(set(listed)==actual,'manifest membership mismatch')
    return {'manifest':row(path),'entries':len(listed),'membership_exact':True,
            'all_sha256_match':True}

machine=pathlib.Path('/etc/machine-id')
need(PARENT.is_dir(),'D2-A parent missing')
need(D2A.is_dir(),'D2-A tree missing')
need(not FAILURE.exists(),'D2-A failure tree exists')
need(STATE.is_dir(),'D2-A state missing')
markers=STATE/'markers'
need(markers.is_dir(),'marker directory missing')
marker_names=sorted(path.name for path in markers.iterdir())
marker_rows=[]
for name in marker_names:
    path=markers/name
    identity=row(path)
    identity['value']=json.loads(path.read_text())
    marker_rows.append(identity)
writable=[]
for path in [D2A,*D2A.rglob('*')]:
    if path.stat().st_mode & 0o222:
        writable.append(str(path.relative_to(D2A) or '.'))
elf_files=[]
for path in PARENT.rglob('*'):
    if path.is_file():
        with path.open('rb') as source:
            if source.read(4)==b'\x7fELF':
                elf_files.append(str(path))
result={
    'hostname':os.uname().nodename,
    'machine_id_exact_file_sha256':sha(machine),
    'parent_entries':sorted(path.name for path in PARENT.iterdir()),
    'd2a_present':D2A.is_dir(),
    'failure_present':FAILURE.exists(),
    'state_present':STATE.is_dir(),
    'd2a_manifest':manifest(D2A),
    'd2a_receipt':row(D2A/'D2A_RECEIPT.json'),
    'd2a_receipt_value':json.loads((D2A/'D2A_RECEIPT.json').read_text()),
    'zero_execution_ledger':row(D2A/'ZERO_EXECUTION_LEDGER.json'),
    'zero_execution_value':json.loads((D2A/'ZERO_EXECUTION_LEDGER.json').read_text()),
    'state_entries':sorted(path.name for path in STATE.iterdir()),
    'state':row(STATE/'STATE.json'),
    'state_value':json.loads((STATE/'STATE.json').read_text()),
    'route_lock':row(STATE/'route.lock'),
    'route_lock_value':json.loads((STATE/'route.lock').read_text()),
    'marker_names':marker_names,
    'markers':marker_rows,
    'consumed_names':sorted(path.name for path in markers.glob('*.consumed*')),
    'd2a_writable_paths':writable,
    'elf_files_under_task_parent':elf_files,
}
print(json.dumps(result,sort_keys=True,separators=(',',':')))
'''
    return constants + "\n" + body


def remote_projection() -> dict[str, Any]:
    result = run(
        [
            "ssh",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            shlex.join(["python3", "-c", remote_probe_source()]),
        ]
    )
    return json.loads(result.stdout)


def check_row(
    checks: list[dict[str, Any]],
    check_id: str,
    condition: bool,
    detail: Any,
) -> None:
    require(condition, f"{check_id}: {detail}")
    checks.append({"id": check_id, "verdict": "PASS", "detail": detail})


def local_evidence_audit(checks: list[dict[str, Any]]) -> dict[str, Any]:
    controller = require_file(
        CONTROLLER,
        digest=EXPECTED["controller"],
        size=EXPECTED["controller_size"],
        mode="0755",
    )
    check_row(checks, "L01-controller-identity", True, controller)
    d2a_receipt_row = require_file(
        D2A_RESULT / "D2A_RECEIPT.json",
        digest=EXPECTED["d2a_receipt"],
        size=EXPECTED["d2a_receipt_size"],
        mode="0444",
    )
    check_row(checks, "L02-authoritative-receipt-identity", True, d2a_receipt_row)
    local_receipt_row = require_file(
        D2A_RESULT / "LOCAL_D2A_RECEIPT.json",
        digest=EXPECTED["local_receipt"],
        mode="0444",
    )
    check_row(checks, "L03-local-receipt-identity", True, local_receipt_row)
    local_manifest = verify_sha256sums(D2A_RESULT)
    check_row(
        checks,
        "L04-local-sha256sums",
        local_manifest["manifest"]["sha256"] == EXPECTED["local_sha256sums"],
        local_manifest,
    )
    check_row(
        checks,
        "L05-local-evidence-immutable",
        writable_paths(D2A_RESULT) == [],
        {"writable_paths": writable_paths(D2A_RESULT)},
    )
    remote_copy = D2A_RESULT / "REMOTE_EVIDENCE"
    remote_manifest = verify_sha256sums(remote_copy)
    check_row(
        checks,
        "L06-copied-remote-sha256sums",
        remote_manifest["manifest"]["sha256"] == EXPECTED["remote_sha256sums"],
        remote_manifest,
    )
    authoritative = (D2A_RESULT / "D2A_RECEIPT.json").read_bytes()
    copied = (remote_copy / "D2A_RECEIPT.json").read_bytes()
    check_row(
        checks,
        "L07-receipt-copy-byte-parity",
        authoritative == copied,
        {"sha256": sha256_bytes(copied)},
    )
    receipt = json.loads(authoritative)
    check_row(
        checks,
        "L08-verdict-and-controller-state",
        receipt.get("verdict") == "D2A_CLOSED_ALL_MARKERS_AVAILABLE"
        and receipt.get("controller_state") == "PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN",
        {"verdict": receipt.get("verdict"), "controller_state": receipt.get("controller_state")},
    )
    check_row(
        checks,
        "L09-transaction-identity",
        receipt.get("transaction_id") == EXPECTED["transaction_id"],
        receipt.get("transaction_id"),
    )
    marker_rows = {row["marker"]: row for row in receipt["markers"]}
    check_row(
        checks,
        "L10-marker-ledger",
        receipt.get("markers_expected") == 11
        and receipt.get("markers_created") == 11
        and receipt.get("markers_consumed") == 0,
        {
            "expected": receipt.get("markers_expected"),
            "created": receipt.get("markers_created"),
            "consumed": receipt.get("markers_consumed"),
        },
    )
    check_row(
        checks,
        "L11-marker-name-route-set",
        {name: row["route_id"] for name, row in marker_rows.items()} == MARKER_ROUTES,
        {name: row["route_id"] for name, row in marker_rows.items()},
    )
    check_row(
        checks,
        "L12-marker-receipt-modes",
        {row["mode"] for row in marker_rows.values()} == {"0400"},
        sorted({row["mode"] for row in marker_rows.values()}),
    )
    assembled = remote_copy / "inputs/assembled_d2_source.rs"
    assembled_row = require_file(
        assembled,
        digest=EXPECTED["assembled_source"],
        size=EXPECTED["assembled_source_size"],
        mode="0444",
    )
    check_row(checks, "L13-assembled-source", True, assembled_row)
    check_row(
        checks,
        "L14-production-prefix",
        sha256_bytes(assembled.read_bytes()[: EXPECTED["production_prefix_size"]])
        == EXPECTED["production_prefix"],
        {
            "size_bytes": EXPECTED["production_prefix_size"],
            "sha256": EXPECTED["production_prefix"],
        },
    )
    check_row(
        checks,
        "L15-source-not-compiled",
        receipt["source_closure"].get("compiled") is False,
        receipt["source_closure"].get("compiled"),
    )
    zero_fields = {
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "d2_subject": 0,
        "u_routes_executed": 0,
        "v_routes_executed": 0,
        "t_routes_executed": 0,
    }
    check_row(
        checks,
        "L16-zero-execution-counters",
        all(receipt.get(key) == value for key, value in zero_fields.items()),
        {key: receipt.get(key) for key in zero_fields},
    )
    false_fields = (
        "d2_elf_created",
        "bucket_map_created",
        "parity_executed",
        "runtime_authority_changed",
        "installed_lay_changed",
        "foreign_process_control",
        "host_tuning",
    )
    check_row(
        checks,
        "L17-zero-execution-booleans",
        all(receipt.get(key) is False for key in false_fields),
        {key: receipt.get(key) for key in false_fields},
    )
    zero_ledger = load_json(remote_copy / "ZERO_EXECUTION_LEDGER.json")
    check_row(
        checks,
        "L18-zero-ledger-parity",
        all(zero_ledger.get(key) == value for key, value in zero_fields.items() if key in zero_ledger)
        and zero_ledger.get("bucket_map_created") is False
        and zero_ledger.get("runtime_authority_changed") is False
        and zero_ledger.get("markers_consumed") == 0,
        zero_ledger,
    )
    local_receipt = load_json(D2A_RESULT / "LOCAL_D2A_RECEIPT.json")
    check_row(
        checks,
        "L19-runtime-stability",
        local_receipt.get("runtime_stable") is True
        and local_receipt.get("runtime_before") == local_receipt.get("runtime_after")
        and local_receipt.get("runtime_authority_changed") is False,
        {
            "runtime_stable": local_receipt.get("runtime_stable"),
            "runtime_before": local_receipt.get("runtime_before"),
            "runtime_after": local_receipt.get("runtime_after"),
        },
    )
    copied_controller = require_file(
        remote_copy / "inputs/controller.py",
        digest=EXPECTED["controller"],
        size=EXPECTED["controller_size"],
        mode="0444",
    )
    check_row(checks, "L20-sealed-controller-copy", True, copied_controller)
    check_row(
        checks,
        "L21-next-action-boundary",
        receipt.get("next_action_admitted")
        == "independent D2-A audit only; build remains unexecuted",
        receipt.get("next_action_admitted"),
    )
    return {
        "controller": controller,
        "authoritative_receipt": d2a_receipt_row,
        "local_receipt": local_receipt_row,
        "local_manifest": local_manifest,
        "copied_remote_manifest": remote_manifest,
        "assembled_source": assembled_row,
        "receipt": receipt,
    }


def remote_evidence_audit(
    projection: Mapping[str, Any],
    local: Mapping[str, Any],
    checks: list[dict[str, Any]],
) -> None:
    receipt = local["receipt"]
    check_row(
        checks,
        "R01-host-identity",
        projection.get("hostname") == REMOTE_HOSTNAME
        and projection.get("machine_id_exact_file_sha256") == REMOTE_MACHINE_ID_SHA256,
        {
            "hostname": projection.get("hostname"),
            "machine": projection.get("machine_id_exact_file_sha256"),
        },
    )
    check_row(
        checks,
        "R02-tree-projection",
        projection.get("parent_entries") == ["d2a-v1"]
        and projection.get("d2a_present") is True
        and projection.get("failure_present") is False
        and projection.get("state_present") is True,
        {
            "parent_entries": projection.get("parent_entries"),
            "d2a": projection.get("d2a_present"),
            "failure": projection.get("failure_present"),
            "state": projection.get("state_present"),
        },
    )
    check_row(
        checks,
        "R03-live-remote-sha256sums",
        projection["d2a_manifest"]["manifest"]["sha256"]
        == EXPECTED["remote_sha256sums"]
        and projection["d2a_manifest"]["all_sha256_match"] is True
        and projection["d2a_manifest"]["membership_exact"] is True,
        projection["d2a_manifest"],
    )
    check_row(
        checks,
        "R04-live-receipt-identity",
        projection["d2a_receipt"]["sha256"] == EXPECTED["d2a_receipt"]
        and projection["d2a_receipt_value"] == receipt,
        projection["d2a_receipt"],
    )
    check_row(
        checks,
        "R05-live-evidence-immutable",
        projection.get("d2a_writable_paths") == [],
        projection.get("d2a_writable_paths"),
    )
    check_row(
        checks,
        "R06-state-identity",
        projection["state"]["sha256"] == EXPECTED["state"]
        and projection["state"]["mode"] == "0400"
        and projection.get("state_entries") == ["STATE.json", "markers", "route.lock"],
        {"state": projection["state"], "entries": projection.get("state_entries")},
    )
    state = projection["state_value"]
    check_row(
        checks,
        "R07-state-ledger",
        state.get("task_id") == TASK_ID
        and state.get("transaction_id") == EXPECTED["transaction_id"]
        and state.get("state") == "D2A_CLOSED_ALL_MARKERS_AVAILABLE"
        and state.get("markers_expected") == 11
        and state.get("markers_created") == 11
        and state.get("markers_consumed") == 0
        and state.get("retry_permitted") is False,
        {
            key: state.get(key)
            for key in (
                "task_id",
                "transaction_id",
                "state",
                "markers_expected",
                "markers_created",
                "markers_consumed",
                "retry_permitted",
            )
        },
    )
    check_row(
        checks,
        "R08-route-lock",
        projection["route_lock"]["sha256"] == EXPECTED["route_lock"]
        and projection["route_lock"]["mode"] == "0400"
        and projection["route_lock_value"].get("state") == "unlocked"
        and projection["route_lock_value"].get("transaction_id")
        == EXPECTED["transaction_id"],
        {
            "identity": projection["route_lock"],
            "value": projection["route_lock_value"],
        },
    )
    check_row(
        checks,
        "R09-live-marker-name-set",
        projection.get("marker_names") == sorted(MARKER_ROUTES),
        projection.get("marker_names"),
    )
    receipt_markers = {row["marker"]: row for row in receipt["markers"]}
    live_markers = {pathlib.Path(row["path"]).name: row for row in projection["markers"]}
    marker_checks = []
    for name, route_id in MARKER_ROUTES.items():
        live = live_markers[name]
        value = live["value"]
        sealed = receipt_markers[name]
        marker_checks.append(
            live["sha256"] == sealed["sha256"]
            and live["size_bytes"] == sealed["size_bytes"]
            and live["mode"] == sealed["mode"] == "0400"
            and value.get("schema") == "lay.v10.e1-traversal-d2-primary-only-marker.v1"
            and value.get("task_id") == TASK_ID
            and value.get("transaction_id") == EXPECTED["transaction_id"]
            and value.get("route_id") == route_id
            and value.get("state") == "available"
            and value.get("retry_permitted") is False
            and value.get("controller_sha256") == EXPECTED["controller"]
            and value.get("preflight_sha256") == EXPECTED["preflight"]
        )
    check_row(
        checks,
        "R10-live-marker-identities-and-values",
        all(marker_checks),
        {"markers_checked": len(marker_checks), "all_exact": all(marker_checks)},
    )
    check_row(
        checks,
        "R11-no-consumed-marker",
        projection.get("consumed_names") == [],
        projection.get("consumed_names"),
    )
    zero = projection["zero_execution_value"]
    check_row(
        checks,
        "R12-live-zero-execution-ledger",
        zero
        == {
            "bucket_map_created": False,
            "cargo_invocations": 0,
            "d2_subject": 0,
            "markers_consumed": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "runtime_authority_changed": False,
            "rustc_compilations": 0,
        },
        zero,
    )
    check_row(
        checks,
        "R13-no-d2-elf-under-task-parent",
        projection.get("elf_files_under_task_parent") == [],
        projection.get("elf_files_under_task_parent"),
    )


def self_check() -> dict[str, Any]:
    require(not AUDIT_RESULT.exists(), "independent audit result already exists")
    compile(AUDITOR.read_text(encoding="utf-8"), str(AUDITOR), "exec")
    controller = require_file(
        CONTROLLER,
        digest=EXPECTED["controller"],
        size=EXPECTED["controller_size"],
        mode="0755",
    )
    receipt = require_file(
        D2A_RESULT / "D2A_RECEIPT.json",
        digest=EXPECTED["d2a_receipt"],
        size=EXPECTED["d2a_receipt_size"],
        mode="0444",
    )
    probe = remote_probe_source()
    forbidden_probe_effects = (
        "write_text",
        "write_bytes",
        "unlink(",
        "rename(",
        "chmod(",
        "mkdir(",
        "rmtree",
        "subprocess",
        "os.system",
        "os.kill",
    )
    require(
        all(token not in probe for token in forbidden_probe_effects),
        "remote projection contains a write or process-control effect",
    )
    return {
        "schema": "lay.v10.e1-traversal-d2a-independent-auditor-self-check.v1",
        "verdict": "D2A_INDEPENDENT_AUDITOR_VERIFIED_UNRUN",
        "auditor": file_identity(AUDITOR),
        "producer_controller_read_only": controller,
        "authoritative_d2a_receipt": receipt,
        "producer_controller_imported": False,
        "command_graph": {
            "remote-projection": [
                "ssh",
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=10",
                REMOTE,
                "python3 -c <read-only-projection>",
            ]
        },
        "remote_command_kind": "ssh read-only python projection",
        "remote_writes": 0,
        "marker_mutations": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "d2_subject": 0,
        "d2_elf_created": False,
    }


def publish_result(
    verdict: str,
    *,
    self_check_value: Mapping[str, Any],
    checks: list[dict[str, Any]],
    before: Mapping[str, Any] | None,
    after: Mapping[str, Any] | None,
    local: Mapping[str, Any] | None,
    error: str | None,
) -> dict[str, Any]:
    stage = pathlib.Path(f"{AUDIT_RESULT}.stage-{os.getpid()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        shutil.copy2(AUDITOR, stage / "auditor.py")
        if before is not None:
            write_new_json(stage / "LIVE_REMOTE_BEFORE.json", before)
        if after is not None:
            write_new_json(stage / "LIVE_REMOTE_AFTER.json", after)
        if local is not None:
            local_summary = {key: value for key, value in local.items() if key != "receipt"}
            write_new_json(stage / "LOCAL_EVIDENCE_AUDIT.json", local_summary)
        receipt = {
            "schema": "lay.v10.e1-traversal-d2a-independent-audit.v1",
            "task_id": TASK_ID,
            "verdict": verdict,
            "error": error,
            "auditor": file_identity(stage / "auditor.py"),
            "producer_controller": self_check_value["producer_controller_read_only"],
            "authoritative_d2a_receipt": self_check_value["authoritative_d2a_receipt"],
            "checks_passed": len(checks),
            "checks": checks,
            "live_projection_repeated": before is not None and after is not None,
            "live_projection_stable": before == after if before is not None and after is not None else False,
            "build_admitted": verdict == "D2A_AUDIT_PASS_BUILD_ADMISSION",
            "build_executed": False,
            "build_marker_state": "available" if verdict == "D2A_AUDIT_PASS_BUILD_ADMISSION" else "unknown",
            "markers_expected": 11,
            "markers_created": 11,
            "markers_consumed": 0,
            "remote_writes": 0,
            "marker_mutations": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "d2_subject": 0,
            "d2_elf_created": False,
            "bucket_map_created": False,
            "parity_executed": False,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": (
                "separate post-D2A controller may verify and consume build.available before one Cargo build"
                if verdict == "D2A_AUDIT_PASS_BUILD_ADMISSION"
                else "none"
            ),
            "stop_before_build": True,
        }
        write_new_json(stage / "D2A_INDEPENDENT_AUDIT_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, AUDIT_RESULT)
        descriptor = os.open(AUDIT_RESULT.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
        return receipt
    except Exception:
        remove_tree(stage)
        raise


def audit() -> dict[str, Any]:
    check = self_check()
    checks: list[dict[str, Any]] = []
    before: dict[str, Any] | None = None
    after: dict[str, Any] | None = None
    local: dict[str, Any] | None = None
    try:
        before = remote_projection()
        local = local_evidence_audit(checks)
        remote_evidence_audit(before, local, checks)
        after = remote_projection()
        check_row(
            checks,
            "R14-live-projection-stable",
            before == after,
            {"before_sha256": sha256_bytes(canonical_json_bytes(before)),
             "after_sha256": sha256_bytes(canonical_json_bytes(after))},
        )
        return publish_result(
            "D2A_AUDIT_PASS_BUILD_ADMISSION",
            self_check_value=check,
            checks=checks,
            before=before,
            after=after,
            local=local,
            error=None,
        )
    except Exception as error:
        if not AUDIT_RESULT.exists():
            publish_result(
                "BLOCKED_PROVENANCE",
                self_check_value=check,
                checks=checks,
                before=before,
                after=after,
                local=local,
                error=str(error),
            )
        raise


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "audit"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        result = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(result, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2-A INDEPENDENT AUDIT ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
