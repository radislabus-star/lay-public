#!/usr/bin/env python3
"""Recover U-SINGLE validity from sealed evidence without another execution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shlex
import shutil
import stat
import struct
import subprocess
import time
from typing import Any, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
CONTROLLER = pathlib.Path(__file__).resolve()
PROJECT_ROOT = CONTROLLER.parents[1]
CORRECTION = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "U_SINGLE_WORKER_SENTINEL_CORRECTION_V1_2026-08-26.md"
)
FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_e1_remaining_cost_d1_test_module.rs.inc"
D1_ROOT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25"
)
U_ROOT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "U_SINGLE_V1_2026-08-26"
)
RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "U_SINGLE_SALVAGE_V1_2026-08-26"
)

HISTORICAL_RECEIPT_SHA256 = "46d52ac863e25da861f803096a6918a47a1f4b7138c0167c1f4724ad7b26dac8"
FRAGMENT_SHA256 = "bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665"
D1_SAMPLES_SHA256 = "b520bcd979449e60f6a03ce477375e98e774a1999eff02d22840a0e8b07832b9"
D2_SAMPLES_SHA256 = "acbb10629e9a259d6d81bc8dad78caf55e102589293556559e04f930c1f18735"
STRUCTURE_SHA256 = "90d24adee563be803c390b41b18b41624b999db37b34c26650cb362f03d06712"
SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
RECORDS = 7_640
REMOTE_STATE = pathlib.PurePosixPath(
    "/home/e/.local/state/lay/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
)
REMOTE_RESULT = pathlib.PurePosixPath(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825/uv-u-single-v1"
)


class SalvageError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SalvageError(message)


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


def write_new_json(path: pathlib.Path, value: Any) -> None:
    write_new_bytes(path, json.dumps(value, sort_keys=True, indent=2).encode() + b"\n")


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            values.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return values


def write_sha256sums(root: pathlib.Path) -> None:
    values = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{value['sha256']}  {value['path']}\n" for value in values).encode(),
    )


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        require(not pure.is_absolute() and ".." not in pure.parts and relative not in seen, f"unsafe manifest row: {relative}")
        seen.add(relative)
        require(sha256_file(root / pure) == digest, f"manifest mismatch: {relative}")
    actual = {value["path"] for value in inventory(root) if value["path"] != "SHA256SUMS"}
    require(seen == actual, "manifest membership mismatch")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def remove_owned_tree(root: pathlib.Path) -> None:
    if not root.exists():
        return
    for path in [root, *root.rglob("*")]:
        path.chmod(0o700 if path.is_dir() else 0o600)
    shutil.rmtree(root)


def run(command: Sequence[str], *, check: bool = True, timeout: int = 120) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=timeout)
    if check and result.returncode != 0:
        raise SalvageError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            f"{result.stderr.decode(errors='replace')[-4000:]}"
        )
    return result


def verify_sample_stream(path: pathlib.Path, expected_sha256: str) -> dict[str, Any]:
    require(sha256_file(path) == expected_sha256, f"sample SHA drift: {path}")
    raw = path.read_bytes()
    require(len(raw) == RECORDS * SAMPLE.size, f"sample denominator drift: {path}")
    workers = set()
    errors = 0
    unresolved = 0
    traversal_cpu_ns = 0
    for offset in range(0, len(raw), SAMPLE.size):
        values = SAMPLE.unpack_from(raw, offset)
        workers.add(values[2])
        errors += int(bool(values[3] & 1))
        unresolved += int(bool(values[3] & 2))
        traversal_cpu_ns += values[13]
    return {
        **row(path),
        "records": len(raw) // SAMPLE.size,
        "record_width_bytes": SAMPLE.size,
        "worker_ids": sorted(workers),
        "errors": errors,
        "unresolved": unresolved,
        "traversal_thread_cpu_ns": traversal_cpu_ns,
    }


REMOTE_PROJECTION = r'''import hashlib,json,pathlib,stat
s=pathlib.Path("/home/e/.local/state/lay/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825")
r=pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825/uv-u-single-v1")
def sha(p):
 h=hashlib.sha256()
 with p.open("rb") as f:
  for b in iter(lambda:f.read(1048576),b""): h.update(b)
 return h.hexdigest()
state=json.loads((s/"U_SINGLE_STATE.json").read_text())
markers=[]
for p in sorted((s/"markers").iterdir()):
 markers.append({"name":p.name,"mode":f"{stat.S_IMODE(p.stat().st_mode):04o}","size":p.stat().st_size,"sha256":sha(p)})
print(json.dumps({"state":state,"receipt_sha256":sha(r/"D2_UV_ROUTE_RECEIPT.json"),"markers":markers},sort_keys=True,separators=(",",":")))'''


def remote_projection() -> dict[str, Any]:
    remote_command = shlex.join(["/usr/bin/python3", "-c", REMOTE_PROJECTION])
    command = [
        "/usr/bin/ssh",
        "-i",
        str(SSH_IDENTITY),
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        REMOTE,
        remote_command,
    ]
    result = run(command)
    return json.loads(result.stdout)


def validate_projection(value: dict[str, Any]) -> None:
    state = value.get("state", {})
    require(state.get("state") == "BLOCKED_SEMANTIC", "historical state drift")
    require(state.get("route") == "U-SINGLE", "historical route drift")
    require(state.get("receipt_sha256") == HISTORICAL_RECEIPT_SHA256, "historical state receipt drift")
    require(value.get("receipt_sha256") == HISTORICAL_RECEIPT_SHA256, "remote receipt drift")
    markers = {item["name"]: item for item in value.get("markers", [])}
    require(len(markers) == 11, "marker count drift")
    require("u-single.consumed-before-exec" in markers, "U-SINGLE consumed marker missing")
    require("u-single.available" not in markers, "U-SINGLE marker recreated")
    require("u-fixed.available" in markers, "U-FIXED marker missing")
    require(all(item["mode"] == "0400" for item in markers.values()), "marker mode drift")


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), "salvage result already exists")
    require_file = row
    require_file(CONTROLLER)
    correction = require_file(CORRECTION)
    fragment = require_file(FRAGMENT)
    require(fragment["sha256"] == FRAGMENT_SHA256, "D1 fragment drift")
    source = FRAGMENT.read_text()
    require("const D1_SINGLE_WORKER_SENTINEL: u8 = u8::MAX;" in source, "single sentinel definition missing")
    require("D1_SINGLE_WORKER_SENTINEL," in source, "single sentinel producer use missing")
    require(mode_string(U_ROOT) == "0555", "historical result tree mode drift")
    local_entries = verify_sha256sums(U_ROOT)
    remote_entries = verify_sha256sums(U_ROOT / "REMOTE_EVIDENCE")
    receipt_file = U_ROOT / "REMOTE_EVIDENCE/D2_UV_ROUTE_RECEIPT.json"
    require(sha256_file(receipt_file) == HISTORICAL_RECEIPT_SHA256, "historical receipt drift")
    receipt = json.loads(receipt_file.read_text())
    require(receipt.get("verdict") == "BLOCKED_SEMANTIC", "historical verdict drift")
    dispatch = receipt.get("dispatch", {})
    require(dispatch.get("selected_cause") == "semantic", "historical dispatch cause drift")
    require(
        dispatch.get("all_violations", {}).get("semantic") == ["component worker coverage mismatch"],
        "historical violation set drift",
    )
    return {
        "controller": row(CONTROLLER),
        "correction": correction,
        "fragment": fragment,
        "historical_receipt": row(receipt_file),
        "historical_local_manifest_entries": local_entries,
        "historical_remote_manifest_entries": remote_entries,
        "verdict": "U_SINGLE_SALVAGE_CONTROLLER_VERIFIED_UNRUN",
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "remote_writes": 0,
    }


def salvage() -> dict[str, Any]:
    check = self_check()
    before = remote_projection()
    validate_projection(before)
    d1 = verify_sample_stream(
        D1_ROOT / "C-SINGLE/subject/component-samples.bin", D1_SAMPLES_SHA256
    )
    d2 = verify_sample_stream(
        U_ROOT / "REMOTE_EVIDENCE/subject/component-samples.bin", D2_SAMPLES_SHA256
    )
    require(d1["worker_ids"] == d2["worker_ids"] == [255], "single worker sentinel mismatch")
    require(d1["errors"] == d1["unresolved"] == d2["errors"] == d2["unresolved"] == 0, "sample semantic flags nonzero")
    structure = row(U_ROOT / "REMOTE_EVIDENCE/subject/structure.json")
    require(structure["sha256"] == STRUCTURE_SHA256, "U-SINGLE structure drift")
    historical = json.loads(
        (U_ROOT / "REMOTE_EVIDENCE/D2_UV_ROUTE_RECEIPT.json").read_text()
    )
    observation = historical["observation"]
    require(observation.get("complete") is True and observation.get("exit_code") == 0, "historical observation incomplete")
    require(observation.get("violations", {}).get("thermal") == [], "thermal drift present")
    require(observation.get("violations", {}).get("perturbation") == [], "perturbation violation present")
    delta = observation.get("details", {}).get("absolute_delta_percent")
    require(isinstance(delta, (int, float)) and delta <= 5.0, "CPU/edge delta exceeds 5%")
    after = remote_projection()
    validate_projection(after)
    require(before == after, "live remote projection changed during salvage")

    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "REMOTE_BEFORE.json", before)
        write_new_json(stage / "REMOTE_AFTER.json", after)
        write_new_bytes(stage / "controller.py", CONTROLLER.read_bytes())
        write_new_bytes(stage / "correction.md", CORRECTION.read_bytes())
        write_new_bytes(stage / "d1-fragment.inc", FRAGMENT.read_bytes())
        write_new_bytes(
            stage / "HISTORICAL_U_SINGLE_RECEIPT.json",
            (U_ROOT / "REMOTE_EVIDENCE/D2_UV_ROUTE_RECEIPT.json").read_bytes(),
        )
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-u-single-sealed-evidence-salvage.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "U_SINGLE_RECOVERED_FROM_SEALED_EVIDENCE",
            "effective_route_state": "U_SINGLE_PASS",
            "historical_execution_verdict": "BLOCKED_SEMANTIC",
            "historical_receipt_sha256": HISTORICAL_RECEIPT_SHA256,
            "historical_state_unchanged": True,
            "historical_marker_consumed": True,
            "retry_permitted": False,
            "correction_scope": "single worker ID 255 is the frozen D1 sentinel, not semantic mismatch",
            "d1_samples": d1,
            "d2_samples": d2,
            "structure": structure,
            "cpu_per_edge_delta_percent": delta,
            "thermal_throttle_drift": {},
            "semantic_errors_unresolved": "0/0",
            "remote_projection_unchanged": True,
            "new_subject_execution": False,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "remote_writes": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "U-FIXED only through a controller that pins this exact receipt",
        }
        write_new_json(stage / "U_SINGLE_SALVAGE_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, RESULT)
        fsync_directory(RESULT.parent)
    except BaseException:
        remove_owned_tree(stage)
        raise
    return {
        "verdict": "U_SINGLE_RECOVERED_FROM_SEALED_EVIDENCE",
        "effective_route_state": "U_SINGLE_PASS",
        "receipt_sha256": sha256_file(RESULT / "U_SINGLE_SALVAGE_RECEIPT.json"),
        "result": str(RESULT),
        "next_action_admitted": "U-FIXED only",
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "salvage"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else salvage()
        print(json.dumps(value, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2 U-SINGLE SALVAGE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
