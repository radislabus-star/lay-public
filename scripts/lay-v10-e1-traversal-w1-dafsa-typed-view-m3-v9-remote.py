#!/usr/bin/env python3
"""Fail-closed remote producer for the M3 V9 materialization trace."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import json
import os
import pathlib
import re
import shutil
import signal
import stat
import subprocess
import sys
import time
from collections.abc import Mapping, Sequence
from typing import Any


TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-20260827"
TRANSACTION_ID = "ed21c54906eebc5a9a99afc873b3a38b8a6ca5e6003b539d019539403aa2ffb1"
V8R1_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827"
V8R1_TRANSACTION_ID = "7d6455e678c244be3c31dc52c2b64d55f34d0a91338afa1219acf06ff327ffb9"
V8R2_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r2-direct-exec-20260827"
V8R2_TRANSACTION_ID = "59694b7b9f0327d78896b5bc4797671f54478674186558e338e4a1b0d9ef7813"
V8R3_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r3-terminal-projection-20260827"
V8R3_TRANSACTION_ID = "a33732116bdf8a1ccf1216e99958750ad33b0ccc6a3c7bbd4226454b20bfd66f"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
KERNEL = "6.8.0-124-generic"

PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
BOOTSTRAP = PARENT / "bootstrap-v1"
TRACE = PARENT / "trace-v1"
V8R1_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / V8R1_TASK_ID
V8R1_STATE = pathlib.Path("/home/e/.local/state/lay") / V8R1_TASK_ID
V8R1_ELF = V8R1_PARENT / "build-v1/m3-v8r1-test-elf"
V8R1_INPUTS = V8R1_PARENT / "bootstrap-v1/inputs"
V8R1_WRAPPER = V8R1_PARENT / "e2e-v1/E2E_WRAPPER.json"
V8R2_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / V8R2_TASK_ID
V8R2_STATE = pathlib.Path("/home/e/.local/state/lay") / V8R2_TASK_ID
V8R2_CACHE = pathlib.Path("/home/e/.cache") / f"lay-m3-v8r2-{V8R2_TRANSACTION_ID}"
V8R3_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / V8R3_TASK_ID
V8R3_STATE = pathlib.Path("/home/e/.local/state/lay") / V8R3_TASK_ID
V8R3_ELF = V8R3_PARENT / "bootstrap-v1/m3-v8r3-test-elf"
V8R3_WRAPPER = V8R3_PARENT / "e2e-v1/E2E_WRAPPER.json"
EXECUTABLE = V8R3_ELF

ACTIONS = ("self-check", "status", "bootstrap", "create-marker", "trace-once")
ROUTES = ("TRACE",)
MARKER_NAME = "trace"
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)
EXPECTED_ELF_SIZE = 320_613_368
EXPECTED_ELF_SHA256 = "0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea"
EXPECTED_BUILD_ID = "c6ddac7181428a303cbc51be61dd3bb115677562"
EXPECTED_BUILD_AUDIT_SHA256 = "d7d5e7110171e5c6546016ff0c9374c323804014ef8cfa7a690ad7d1d11c8340"
EXPECTED_TERMINAL_AUDIT_SHA256 = "04d0e17158a63a49088e8c8ff9dc25df67e50ac6a97b770ea3fcf1a73d67ec91"
EXPECTED_DIAGNOSIS_SHA256 = "9b05af87d83c937dcc1e4eab0e398ab3d93ef49ac3e0bfb8089a58ba3d64bae0"
EXPECTED_V8R1_WRAPPER_SHA256 = "1edc2c195b67485d007c1ed9354db14cf6f9907eaa1ffc3d77fa5f07b13f291b"
EXPECTED_V8R2_DIAGNOSIS_SHA256 = "a74b51570143f56a8186435acfac80ab4dac5ec4140f105ac82df960508ed3dd"
EXPECTED_V8R2_JOURNAL_SHA256 = "298ebe81c77f48edca3d9df1fd514b14d6bcd7abf15f41ac42752cf27029d785"
EXPECTED_V8R2_CONTROLLER_RECEIPT_SHA256 = "0c8e792d88e532b5cfb18c65a4ba15c031cc0956e6c9798a0b4de8c30b450b34"
EXPECTED_V8R2_LOCAL_CONTROLLER_SHA256 = "7a77aaa1a1b42e77e3c654e12953c6f02e70ecee71046a50befe6ffaea7a446a"
EXPECTED_V8R2_REMOTE_CONTROLLER_SHA256 = "f40183fb405a0ab52ac86dc1493fc38ae4ca14aef0830e46188b49cb4e21081a"
EXPECTED_V8R2_AUDITOR_SHA256 = "72b9071d75bd0cfb15545c1820dae453ff961d207f7cd346c17ae1881f61592f"
EXPECTED_V8R3_TERMINAL_SHA256 = "2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc"
EXPECTED_V8R3_SUBJECT_SHA256 = "65cd8a6f08d77c192ae0eb24fa3df106ee5030e7a8bbdfdf44d08429f7d9bfd5"
EXPECTED_V8R3_WRAPPER_SHA256 = "e2aaefc1490df86cb1abae176035efab22b290cc72afe23fefeeb52e96d42ece"
EXPECTED_LATENCY_DECISION_SHA256 = "45e2e279997f7a93072bcfd74ad11d2566f55b442685d1be2a75e905dd543a8a"
EXPECTED_V9_PAPER_SHA256 = "98000de5d6a502d4bf1b2005deca476bfdd12539f02528a4b450f240f3d9ed27"
EXPECTED_V9_ROUTE_SHA256 = "7270e79ccd5c64f33dc5d1a3f95ff4dbea9a8f13baebc81aa97254a80d7fe860"
EXPECTED_V9_ROUTE_RECEIPT_SHA256 = "84171ed96027ae674e50618398af2d881f44b83fd7265a4b5e5cd92d8b9be00f"
EXPECTED_V9_PREFLIGHT_MANIFEST_SHA256 = "efdbd7adc388492656d529ed28e1854b400d4be9ddf205cdb577624b91963082"
EXPECTED_V9_PREFLIGHT_RECEIPT_SHA256 = "43e5f2e849a7ccdeab2fbc371f12fc0950e11e4c3b5c8fef043d01f464223587"
TRACE_PREFIX = "productive_v90_materialization_trace "
TRACE_PATTERN = re.compile(
    r"^productive_v90_materialization_trace "
    r"surfaces=(\d+) emitted=(\d+) setup_us=(\d+) projection_us=(\d+) "
    r"classify_us=(\d+) gate_us=(\d+) evidence_us=(\d+)$"
)
EXPECTED_TRACE_ROWS = 1_910
WARMUP_ROWS = 382
MEASURED_ROWS = 1_528
TAIL_ROWS = 16
EXPECTED_INPUTS = {
    "LAY-L2-RU-FULL-v13.bin": (140_556_462, "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"),
    "slice8b-v7-fixed-13x100.json": (1_606_189, "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m": (17_309_944, "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r": (2_123_112, "de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e"),
    "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin": (77_962_328, "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"),
    "l11-proof.json": (539_536, "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d"),
}


class V9RemoteError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V9RemoteError(message)


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


def require_file(
    path: pathlib.Path,
    *,
    size: int | None = None,
    digest: str | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    row = file_row(path)
    if size is not None:
        need(row["size_bytes"] == size, f"size drift: {path}")
    if digest is not None:
        need(row["sha256"] == digest, f"SHA drift: {path}")
    if mode is not None:
        need(row["mode"] == mode, f"mode drift: {path}")
    return row


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


def write_json(path: pathlib.Path, value: Any, mode: int = 0o600) -> None:
    write_new(path, canonical(value), mode)


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def run(
    argv: Sequence[str],
    *,
    env: Mapping[str, str] | None = None,
    timeout: float = 3_600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        env=dict(env) if env is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        raise V9RemoteError(
            f"command failed ({result.returncode}): {list(argv)!r}\n"
            f"{result.stderr[-4000:].decode(errors='replace')}"
        )
    return result


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
    }


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def seal_tree(root: pathlib.Path, executable: pathlib.Path | None = None) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_dir():
            path.chmod(0o555)
        else:
            path.chmod(0o555 if executable is not None and path == executable else 0o444)
    root.chmod(0o555)


def verify_inventory(root: pathlib.Path, payload: Mapping[str, Any]) -> dict[str, Any]:
    inventory = payload.get("files")
    need(isinstance(inventory, dict) and inventory, "bootstrap payload inventory absent")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "PAYLOAD.json"
    }
    need(actual == set(inventory), "bootstrap payload file set drift")
    rows = {}
    for relative, expected in sorted(inventory.items()):
        need(isinstance(expected, dict), f"invalid inventory row: {relative}")
        rows[relative] = require_file(
            root / relative,
            size=int(expected.get("size_bytes", -1)),
            digest=str(expected.get("sha256", "")),
        )
    return rows


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1).decode()


def marker_inventory() -> dict[str, list[str]]:
    markers = STATE / "markers"
    if not markers.is_dir():
        return {"available": [], "consumed": []}
    return {
        "available": sorted(path.name for path in markers.glob("*.available")),
        "consumed": sorted(path.name for path in markers.glob("*.consumed-before-exec")),
    }


def latest_state(root: pathlib.Path = STATE) -> dict[str, Any] | None:
    if not root.is_dir():
        return None
    rows = sorted(root.glob("STATE-*.json"))
    return load_json(rows[-1]) if rows else None


def append_state(sequence: int, state: str, **extra: Any) -> None:
    write_json(
        STATE / f"STATE-{sequence:02d}-{state}.json",
        {
            "schema": "lay.m3-v9-remote-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "sequence": sequence,
            "state": state,
            "markers": marker_inventory(),
            **extra,
        },
        0o444,
    )
    fsync_dir(STATE)


def verify_host() -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "remote hostname drift")
    need(os.uname().release == KERNEL, "remote kernel drift")
    machine = require_file(pathlib.Path("/etc/machine-id"))
    need(machine["sha256"] == MACHINE_ID_SHA256, "remote machine-id drift")
    online = pathlib.Path("/sys/devices/system/cpu/online").read_text().strip()
    core = pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus").read_text().strip()
    atom = pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus").read_text().strip()
    need((online, core, atom) == ("0-19", "0-11", "12-19"), "remote CPU topology drift")
    return {
        "hostname": HOSTNAME,
        "kernel": KERNEL,
        "machine_id_sha256": machine["sha256"],
        "online": online,
        "core": core,
        "atom": atom,
    }


def input_paths() -> dict[str, pathlib.Path]:
    return {
        "v13": V8R1_INPUTS / "LAY-L2-RU-FULL-v13.bin",
        "v7": V8R1_INPUTS / "slice8b-v7-fixed-13x100.json",
        "productive": V8R1_INPUTS / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m",
        "recovery": V8R1_INPUTS / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r",
        "l11": V8R1_INPUTS / "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin",
        "l11_proof": V8R1_INPUTS / "l11-proof.json",
        "l11_receipt": V8R1_INPUTS / "l11-installed.json",
    }


def verify_v8r1_predecessor(payload_root: pathlib.Path) -> dict[str, Any]:
    build_audit_path = payload_root / "V8R1_BUILD_AUDIT.json"
    terminal_audit_path = payload_root / "V8R1_TERMINAL_AUDIT.json"
    diagnosis_path = payload_root / "V8R1_DIAGNOSIS.json"
    require_file(build_audit_path, digest=EXPECTED_BUILD_AUDIT_SHA256, mode="0444")
    require_file(terminal_audit_path, digest=EXPECTED_TERMINAL_AUDIT_SHA256, mode="0444")
    require_file(diagnosis_path, digest=EXPECTED_DIAGNOSIS_SHA256, mode="0444")
    build = load_json(build_audit_path)
    terminal = load_json(terminal_audit_path)
    diagnosis = load_json(diagnosis_path)
    need(build.get("verdict") == "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED", "V8R1 build audit verdict drift")
    need(terminal.get("verdict") == "BLOCKED_PROVENANCE", "V8R1 terminal verdict drift")
    need(terminal.get("e2e_wrapper_sha256") == EXPECTED_V8R1_WRAPPER_SHA256, "V8R1 terminal wrapper binding drift")
    need(diagnosis.get("verdict") == "V8R1_LOADER_CURRENT_EXE_DEFECT_CONFIRMED", "V8R1 diagnosis verdict drift")
    original = require_file(
        V8R1_ELF,
        size=EXPECTED_ELF_SIZE,
        digest=EXPECTED_ELF_SHA256,
        mode="0444",
    )
    need(elf_build_id(V8R1_ELF) == EXPECTED_BUILD_ID, "V8R1 Build ID drift")
    v8r1_markers = V8R1_STATE / "markers"
    consumed = sorted(path.name for path in v8r1_markers.glob("*.consumed-before-exec"))
    available = sorted(path.name for path in v8r1_markers.glob("*.available"))
    need(consumed == ["build.consumed-before-exec", "e2e.consumed-before-exec"] and not available, "V8R1 marker history drift")
    producer_state = latest_state(V8R1_STATE)
    need(isinstance(producer_state, dict), "V8R1 producer state absent")
    need(
        producer_state.get("task_id") == V8R1_TASK_ID
        and producer_state.get("transaction_id") == V8R1_TRANSACTION_ID
        and producer_state.get("state") == "E2E_CREATED_UNAUDITED"
        and producer_state.get("e2e_wrapper_sha256") == EXPECTED_V8R1_WRAPPER_SHA256,
        "V8R1 remote producer projection drift",
    )
    wrapper = require_file(V8R1_WRAPPER, digest=EXPECTED_V8R1_WRAPPER_SHA256, mode="0444")
    terminal_projection = terminal.get("live_projection", {}).get("latest_state", {})
    need(
        terminal_projection.get("state") == producer_state["state"]
        and terminal_projection.get("e2e_wrapper_sha256") == EXPECTED_V8R1_WRAPPER_SHA256,
        "V8R1 independent terminal projection drift",
    )
    inputs = {}
    for name, (size, digest) in EXPECTED_INPUTS.items():
        inputs[name] = require_file(V8R1_INPUTS / name, size=size, digest=digest, mode="0444")
    l11_receipt = load_json(V8R1_INPUTS / "l11-installed.json")
    need(l11_receipt.get("artifact_sha256") == EXPECTED_INPUTS["LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin"][1], "L1.1 receipt drift")
    return {
        "build_audit_sha256": sha256_file(build_audit_path),
        "terminal_audit_sha256": sha256_file(terminal_audit_path),
        "diagnosis_sha256": sha256_file(diagnosis_path),
        "remote_producer_state": producer_state,
        "remote_wrapper": wrapper,
        "independent_terminal": {
            "verdict": terminal["verdict"],
            "receipt_sha256": sha256_file(terminal_audit_path),
            "e2e_wrapper_sha256": terminal["e2e_wrapper_sha256"],
        },
        "elf": original,
        "build_id": EXPECTED_BUILD_ID,
        "inputs": inputs,
        "markers": {"available": available, "consumed": consumed},
    }


def verify_v8r2_predecessor(payload_root: pathlib.Path) -> dict[str, Any]:
    expected = {
        "V8R2_DIAGNOSIS.json": EXPECTED_V8R2_DIAGNOSIS_SHA256,
        "V8R2_JOURNAL_SHA256SUMS": EXPECTED_V8R2_JOURNAL_SHA256,
        "V8R2_CONTROLLER_IMPLEMENTATION.json": EXPECTED_V8R2_CONTROLLER_RECEIPT_SHA256,
        "V8R2_LOCAL_CONTROLLER.py": EXPECTED_V8R2_LOCAL_CONTROLLER_SHA256,
        "V8R2_REMOTE_CONTROLLER.py": EXPECTED_V8R2_REMOTE_CONTROLLER_SHA256,
        "V8R2_AUDITOR.py": EXPECTED_V8R2_AUDITOR_SHA256,
    }
    files = {
        name: require_file(payload_root / name, digest=digest, mode="0444")
        for name, digest in expected.items()
    }
    diagnosis = load_json(payload_root / "V8R2_DIAGNOSIS.json")
    need(
        diagnosis.get("task_id") == V8R2_TASK_ID
        and diagnosis.get("transaction_id") == V8R2_TRANSACTION_ID
        and diagnosis.get("verdict") == "V8R2_REMOTE_TERMINAL_PROJECTION_DEFECT_CONFIRMED"
        and diagnosis.get("execution_verdict") == "BLOCKED_PROVENANCE",
        "V8R2 diagnosis drift",
    )
    need(
        diagnosis.get("execution_journal_sha256sums_sha256") == EXPECTED_V8R2_JOURNAL_SHA256
        and diagnosis.get("controller_implementation_sha256") == EXPECTED_V8R2_CONTROLLER_RECEIPT_SHA256,
        "V8R2 diagnosis binding drift",
    )
    need(
        not V8R2_PARENT.exists() and not V8R2_STATE.exists() and not V8R2_CACHE.exists(),
        "V8R2 remote namespace must remain absent",
    )
    return {
        "diagnosis_verdict": diagnosis["verdict"],
        "execution_verdict": diagnosis["execution_verdict"],
        "pending_intent": diagnosis.get("failed_action", {}).get("intent_durable") is True
        and diagnosis.get("failed_action", {}).get("completion_present") is False,
        "remote_paths_absent": {"parent": True, "state": True, "cache": True},
        "files": files,
    }


def verify_v9_contract(payload_root: pathlib.Path) -> dict[str, Any]:
    expected = {
        "LATENCY_DECISION.md": EXPECTED_LATENCY_DECISION_SHA256,
        "V9_PAPER.md": EXPECTED_V9_PAPER_SHA256,
        "V9_ROUTE.md": EXPECTED_V9_ROUTE_SHA256,
        "V9_ROUTE_RECEIPT.json": EXPECTED_V9_ROUTE_RECEIPT_SHA256,
        "PREFLIGHT_MANIFEST.json": EXPECTED_V9_PREFLIGHT_MANIFEST_SHA256,
        "PREFLIGHT_RECEIPT.json": EXPECTED_V9_PREFLIGHT_RECEIPT_SHA256,
        "V8R3_TERMINAL_AUDIT.json": EXPECTED_V8R3_TERMINAL_SHA256,
        "V8R3_SUBJECT_RECEIPT.json": EXPECTED_V8R3_SUBJECT_SHA256,
    }
    rows = {
        name: require_file(payload_root / name, digest=digest, mode="0444")
        for name, digest in expected.items()
    }
    route = load_json(payload_root / "V9_ROUTE_RECEIPT.json")
    preflight = load_json(payload_root / "PREFLIGHT_RECEIPT.json")
    terminal = load_json(payload_root / "V8R3_TERMINAL_AUDIT.json")
    subject = load_json(payload_root / "V8R3_SUBJECT_RECEIPT.json")
    need(route.get("verdict") == "PASS", "V9 structural route verdict drift")
    need(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True,
        "V9 implementation preflight drift",
    )
    need(
        terminal.get("task_id") == V8R3_TASK_ID
        and terminal.get("transaction_id") == V8R3_TRANSACTION_ID
        and terminal.get("verdict") == "BLOCKED_LATENCY",
        "V8R3 terminal predecessor drift",
    )
    need(
        subject.get("schema") == "lay.m3-end-to-end-test-owner.v1"
        and subject.get("verdict") == "BLOCKED_LATENCY",
        "V8R3 subject predecessor drift",
    )
    return {"files": rows, "route_verdict": "PASS", "preflight_verdict": "READY_TO_IMPLEMENT"}


def verify_v8r3_predecessor() -> dict[str, Any]:
    executable = require_file(
        V8R3_ELF,
        size=EXPECTED_ELF_SIZE,
        digest=EXPECTED_ELF_SHA256,
        mode="0555",
    )
    need(elf_build_id(V8R3_ELF) == EXPECTED_BUILD_ID, "V8R3 Build ID drift")
    wrapper = require_file(V8R3_WRAPPER, digest=EXPECTED_V8R3_WRAPPER_SHA256, mode="0444")
    state = latest_state(V8R3_STATE)
    need(isinstance(state, dict), "V8R3 state absent")
    need(
        state.get("task_id") == V8R3_TASK_ID
        and state.get("transaction_id") == V8R3_TRANSACTION_ID
        and state.get("state") == "E2E_CREATED_UNAUDITED"
        and state.get("e2e_wrapper_sha256") == EXPECTED_V8R3_WRAPPER_SHA256,
        "V8R3 live state drift",
    )
    markers = V8R3_STATE / "markers"
    available = sorted(path.name for path in markers.glob("*.available"))
    consumed = sorted(path.name for path in markers.glob("*.consumed-before-exec"))
    need(not available and consumed == ["e2e.consumed-before-exec"], "V8R3 marker history drift")
    return {
        "executable": executable,
        "build_id": EXPECTED_BUILD_ID,
        "wrapper": wrapper,
        "state": state,
        "markers": {"available": available, "consumed": consumed},
    }


def verify_execution_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V9_EXECUTION_ADMITTED", "execution admission verdict drift")
    need(value.get("safe_to_execute") is True, "execution admission is not safe")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "execution admission namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "execution admission controller SHA drift")
    return value


def uid_capability_probe(stage_parent: pathlib.Path) -> dict[str, Any]:
    probe = stage_parent / "uid-capability"
    probe.mkdir(mode=0o700)
    shutil.chown(probe, user="e", group="e")
    required = [EXECUTABLE, *input_paths().values()]
    code = (
        "import os,pathlib; p=pathlib.Path(os.environ['P']); "
        "required=[pathlib.Path(os.environ[k]) for k in sorted(k for k in os.environ if k.startswith('X'))]; "
        "[(q.stat(), q.open('rb').read(1)) for q in required]; a=p/'a'; b=p/'b'; "
        "f=open(a,'xb'); f.write(b'm3-v9-uid-proof\\n'); f.flush(); os.fsync(f.fileno()); f.close(); "
        "os.rename(a,b); assert b.read_bytes()==b'm3-v9-uid-proof\\n'; b.unlink()"
    )
    environment = [f"P={probe}", *[f"X{index:02d}={item}" for index, item in enumerate(required)]]
    result = run(
        ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *environment, "/usr/bin/python3", "-c", code],
        check=False,
    )
    need(result.returncode == 0 and not any(probe.iterdir()), "UID e capability proof failed")
    probe.rmdir()
    return {
        "uid": int(run(["/usr/bin/id", "-u", "e"]).stdout.strip()),
        "required_reads": [str(path) for path in required],
        "operations": ["traverse", "stat", "read", "create", "write", "fsync", "rename", "reopen", "unlink"],
        "verdict": "PASS",
    }


def bootstrap_once(path: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root")
    need(not PARENT.exists() and not STATE.exists(), "V9 remote namespace already exists")
    payload = load_json(path / "PAYLOAD.json")
    need(payload.get("schema") == "lay.m3-v9-bootstrap-payload.v1", "bootstrap payload schema drift")
    need(payload.get("task_id") == TASK_ID and payload.get("transaction_id") == TRANSACTION_ID, "bootstrap payload namespace drift")
    controller_sha = sha256_file(path / "remote-controller.py")
    admission = verify_execution_admission(path / "EXECUTION_ADMISSION.json", controller_sha)
    inventory = verify_inventory(path, payload)
    host = verify_host()
    contract = verify_v9_contract(path)
    predecessor = verify_v8r1_predecessor(path)
    v8r2_predecessor = verify_v8r2_predecessor(path)
    v8r3_predecessor = verify_v8r3_predecessor()

    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o700)
    try:
        copied = stage / "bootstrap-v1"
        shutil.copytree(path, copied)
        uid = uid_capability_probe(stage)
        write_new(state_stage / "route.lock", b"m3-v9-route-lock\n", 0o600)
        receipt = {
            "schema": "lay.m3-v9-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M3_V9_BOOTSTRAP_CREATED_UNAUDITED",
            "controller_sha256": controller_sha,
            "execution_admission_sha256": sha256_file(path / "EXECUTION_ADMISSION.json"),
            "execution_admission_verdict": admission["verdict"],
            "host": host,
            "v9_contract": contract,
            "v8r1_predecessor": predecessor,
            "v8r2_predecessor": v8r2_predecessor,
            "v8r3_predecessor": v8r3_predecessor,
            "executable": v8r3_predecessor["executable"],
            "executable_copied": False,
            "executable_build_id": EXPECTED_BUILD_ID,
            "input_bindings": {key: str(value) for key, value in input_paths().items()},
            "uploaded_files": len(inventory),
            "uid_capability": uid,
            "markers_expected": 1,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
        }
        write_json(stage / "BOOTSTRAP_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        stage.chmod(0o755)
        os.rename(stage, PARENT)
        os.rename(state_stage, STATE)
        fsync_dir(PARENT.parent)
        fsync_dir(STATE.parent)
        append_state(0, "BOOTSTRAP_CREATED_UNAUDITED")
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def verify_bootstrap_audit(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V9_BOOTSTRAP_AUDIT_PASS_MARKER_ADMITTED", "bootstrap audit did not admit marker")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "bootstrap audit controller SHA drift")
    for key in ("local_controller_sha256", "remote_controller_sha256", "auditor_sha256"):
        need(re.fullmatch(r"[0-9a-f]{64}", str(value.get(key, ""))) is not None, f"bootstrap audit lacks {key}")
    return value


def marker_payload(admission: Mapping[str, Any]) -> bytes:
    return canonical({
        "schema": "lay.m3-v9-one-shot-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": "TRACE",
        "local_controller_sha256": admission["local_controller_sha256"],
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "auditor_sha256": admission["auditor_sha256"],
        "one_shot": True,
        "retry_permitted": False,
    })


def create_marker(audit_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "marker creation requires root")
    audit = verify_bootstrap_audit(audit_path, controller_sha)
    markers = STATE / "markers"
    need(not markers.exists(), "V9 marker tree already exists")
    stage = STATE / f"markers.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    try:
        marker = stage / "trace.available"
        write_new(marker, marker_payload(audit), 0o400)
        row = file_row(marker)
        os.rename(stage, markers)
        fsync_dir(STATE)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    append_state(1, "TRACE_MARKER_AVAILABLE", bootstrap_audit_sha256=sha256_file(audit_path))
    return {
        "schema": "lay.m3-v9-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_TRACE_MARKER_AVAILABLE",
        "markers": marker_inventory(),
        "marker_row": row,
        "markers_created": 1,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
    }


def verify_quiet_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V9_QUIET_HOST_TRACE_ADMITTED", "quiet admission verdict drift")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "quiet admission namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "quiet admission controller SHA drift")
    need(value.get("elf_sha256") == EXPECTED_ELF_SHA256, "quiet admission ELF drift")
    return value


def consume_marker(admission: Mapping[str, Any]) -> dict[str, Any]:
    markers = STATE / "markers"
    available = markers / "trace.available"
    consumed = markers / "trace.consumed-before-exec"
    expected = marker_payload(admission)
    before = require_file(available, digest=sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), "TRACE marker state drift")
    os.rename(available, consumed)
    fsync_dir(markers)
    after = require_file(consumed, size=before["size_bytes"], digest=before["sha256"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


def throttle_counters() -> dict[str, int]:
    rows = {}
    for path in pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*"):
        try:
            value = path.read_text().strip()
        except OSError:
            continue
        if value.isdigit():
            rows[str(path)] = int(value)
    return rows


def throttle_drift(before: Mapping[str, int], after: Mapping[str, int]) -> dict[str, list[int]]:
    return {
        key: [int(before.get(key, -1)), int(after.get(key, -1))]
        for key in sorted(set(before) | set(after))
        if before.get(key) != after.get(key)
    }


def terminate(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGKILL)
    with contextlib.suppress(Exception):
        process.wait(timeout=10)


def scientific_environment(subject: pathlib.Path) -> dict[str, str]:
    paths = input_paths()
    environment = controlled_environment()
    environment.update({
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(paths["v13"]),
        "LAY_M3_ACTUAL_OWNER_V7": str(paths["v7"]),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LAY_L2_PACKAGE": str(paths["v13"]),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(paths["productive"]),
        "LAY_L11_RECEIPT": str(paths["l11_receipt"]),
        "LAY_L2_FIELD_TRACE": "1",
    })
    return environment


def scientific_command(subject: pathlib.Path) -> list[str]:
    environment = scientific_environment(subject)
    return [
        "/usr/bin/sudo",
        "-n",
        "-u",
        "e",
        "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset",
        "-c",
        "0",
        str(EXECUTABLE),
        "--ignored",
        "--exact",
        SCIENTIFIC_TEST,
        "--nocapture",
        "--test-threads=1",
    ]


def observe_direct_parent(process: subprocess.Popen[bytes], timeout: float = 5.0) -> dict[str, Any]:
    deadline = time.monotonic() + timeout
    target = str(EXECUTABLE.resolve())
    while time.monotonic() < deadline:
        rows = []
        for path in pathlib.Path("/proc").iterdir():
            if not path.name.isdigit():
                continue
            try:
                executable = str((path / "exe").resolve())
                command = (path / "cmdline").read_bytes().split(b"\0")
                argv = [item.decode(errors="replace") for item in command if item]
            except (FileNotFoundError, PermissionError, ProcessLookupError):
                continue
            if executable == target and SCIENTIFIC_TEST in argv:
                rows.append({"pid": int(path.name), "executable": executable, "argv": argv})
        if rows:
            rows.sort(key=lambda row: row["pid"])
            return {"observed": True, "target": target, "processes": rows}
        if process.poll() is not None:
            break
        time.sleep(0.01)
    return {"observed": False, "target": target, "processes": []}


TRACE_STAGE_NAMES = ("setup_us", "projection_us", "classify_us", "gate_us", "evidence_us")


def nearest_rank(values: Sequence[int], percentile: int) -> int:
    need(values, "cannot summarize an empty trace distribution")
    ordered = sorted(int(value) for value in values)
    rank = max(1, (percentile * len(ordered) + 99) // 100)
    return ordered[rank - 1]


def distribution(rows: Sequence[Mapping[str, Any]], field: str) -> dict[str, int]:
    values = [int(row[field]) for row in rows]
    need(values, f"empty trace distribution: {field}")
    return {
        "count": len(values),
        "p50_us": nearest_rank(values, 50),
        "p99_us": nearest_rank(values, 99),
        "max_us": max(values),
        "sum_us": sum(values),
    }


def summarize_trace(rows: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    need(len(rows) == EXPECTED_TRACE_ROWS, "trace row count drift")
    warmup = list(rows[:WARMUP_ROWS])
    measured = list(rows[WARMUP_ROWS:])
    need(len(warmup) == WARMUP_ROWS and len(measured) == MEASURED_ROWS, "trace phase cardinality drift")
    fields = (*TRACE_STAGE_NAMES, "traced_total_us")
    pooled = {field: distribution(measured, field) for field in fields}
    rounds = []
    for round_index in range(4):
        start = round_index * WARMUP_ROWS
        subset = measured[start : start + WARMUP_ROWS]
        rounds.append({
            "round": round_index + 1,
            "schedule": ("FORWARD", "REVERSED", "FORWARD", "REVERSED")[round_index],
            "stages": {field: distribution(subset, field) for field in fields},
        })
    tail = sorted(measured, key=lambda row: (-int(row["traced_total_us"]), int(row["ordinal"])))[:TAIL_ROWS]
    tail_total = sum(int(row["traced_total_us"]) for row in tail)
    need(tail_total > 0, "trace tail total is zero")
    stage_rows = {}
    dominant_candidates = []
    for stage in TRACE_STAGE_NAMES:
        stage_sum = sum(int(row[stage]) for row in tail)
        largest_count = 0
        for row in tail:
            largest = max(int(row[name]) for name in TRACE_STAGE_NAMES)
            winners = [name for name in TRACE_STAGE_NAMES if int(row[name]) == largest]
            largest_count += int(winners == [stage])
        share = stage_sum / tail_total
        stage_rows[stage] = {
            "sum_us": stage_sum,
            "share": share,
            "largest_stage_rows": largest_count,
        }
        if share >= 0.80 and largest_count >= 15:
            dominant_candidates.append(stage)
    dominant = dominant_candidates[0] if len(dominant_candidates) == 1 else None
    return {
        "schema": "lay.m3-v9-materialization-trace.v1",
        "trace_rows": len(rows),
        "warmup_rows": len(warmup),
        "measured_rows": len(measured),
        "pooled": pooled,
        "rounds": rounds,
        "tail": {
            "rows": len(tail),
            "ordinals": [int(row["ordinal"]) for row in tail],
            "traced_total_us": tail_total,
            "stages": stage_rows,
            "dominant_stage": dominant,
            "dominance_candidates": dominant_candidates,
            "thresholds": {"aggregate_share": 0.80, "largest_stage_rows": 15},
        },
        "claim_boundary": {
            "outer_per_request_join": False,
            "v8r3_latency_reinterpreted": False,
            "production_authority_admitted": False,
        },
    }


def parse_trace(stderr: bytes) -> tuple[list[dict[str, Any]], list[str], dict[str, Any] | None]:
    rows = []
    errors = []
    schedules = ("FORWARD", "REVERSED", "FORWARD", "REVERSED")
    for line_number, line in enumerate(stderr.decode("utf-8", errors="replace").splitlines(), start=1):
        if not line.startswith(TRACE_PREFIX):
            continue
        match = TRACE_PATTERN.fullmatch(line)
        if match is None:
            errors.append(f"line {line_number}: malformed trace row")
            continue
        values = [int(value) for value in match.groups()]
        ordinal = len(rows)
        if ordinal < WARMUP_ROWS:
            phase = "WARMUP"
            round_index = 0
            schedule = "FORWARD"
            case_ordinal = ordinal
        else:
            measured_ordinal = ordinal - WARMUP_ROWS
            round_index = measured_ordinal // WARMUP_ROWS + 1
            case_ordinal = measured_ordinal % WARMUP_ROWS
            schedule = schedules[min(round_index - 1, len(schedules) - 1)]
            phase = "MEASURED"
        row = {
            "ordinal": ordinal,
            "stderr_line": line_number,
            "phase": phase,
            "round": round_index,
            "schedule": schedule,
            "case_ordinal": case_ordinal,
            "surfaces": values[0],
            "emitted": values[1],
            "setup_us": values[2],
            "projection_us": values[3],
            "classify_us": values[4],
            "gate_us": values[5],
            "evidence_us": values[6],
        }
        row["traced_total_us"] = sum(int(row[name]) for name in TRACE_STAGE_NAMES)
        rows.append(row)
    if len(rows) != EXPECTED_TRACE_ROWS:
        errors.append(f"trace row count {len(rows)} != {EXPECTED_TRACE_ROWS}")
    summary = None
    if not errors:
        try:
            summary = summarize_trace(rows)
        except BaseException as error:
            errors.append(f"{type(error).__name__}: {error}")
    return rows, errors, summary


def trace_once(quiet_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "TRACE requires root controller")
    quiet = verify_quiet_admission(quiet_path, controller_sha)
    state = latest_state()
    need(state is not None and state.get("state") == "TRACE_MARKER_AVAILABLE", "TRACE state predecessor drift")
    need(not TRACE.exists() and not (PARENT / "trace-failure-v1").exists(), "V9 TRACE evidence already exists")
    require_file(EXECUTABLE, size=EXPECTED_ELF_SIZE, digest=EXPECTED_ELF_SHA256, mode="0555")
    need(elf_build_id(EXECUTABLE) == EXPECTED_BUILD_ID, "V9 executable Build ID drift")

    stage = PARENT / f"trace-v1.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    final_subject = TRACE / "subject"
    evidence = subject / "evidence"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    shutil.chown(evidence, user="e", group="e")
    environment = scientific_environment(final_subject)
    command = scientific_command(final_subject)
    executable_index = command.index(str(EXECUTABLE))
    need(command[executable_index - 3 : executable_index] == ["/usr/bin/taskset", "-c", "0"], "direct command prefix drift")
    need(not any("ld-linux" in token for token in command), "loader became reachable")
    write_json(stage / "PRE_TRACE.json", {
        "schema": "lay.m3-v9-pre-trace.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "remote_controller_sha256": controller_sha,
        "quiet_admission_sha256": sha256_file(quiet_path),
        "elf_sha256": EXPECTED_ELF_SHA256,
        "elf_build_id": EXPECTED_BUILD_ID,
        "command": command,
        "environment": environment,
        "subject_started": False,
        "retry_permitted": False,
    })
    fsync_dir(stage)
    marker = consume_marker(quiet)
    write_json(stage / "TRACE_MARKER_CONSUMED.json", marker)
    os.rename(stage, TRACE)
    fsync_dir(PARENT)

    thermal_before = throttle_counters()
    process: subprocess.Popen[bytes] | None = None
    stdout = b""
    stderr = b""
    controller_error = None
    direct_identity: dict[str, Any] = {"observed": False, "target": str(EXECUTABLE), "processes": []}
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        direct_identity = observe_direct_parent(process)
        need(direct_identity["observed"] is True, "direct executable identity was not observed")
        stdout, stderr = process.communicate(timeout=10_800)
    except BaseException as error:
        controller_error = f"{type(error).__name__}: {error}"
        terminate(process)
        if process is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
    thermal_after = throttle_counters()
    thermal_change = throttle_drift(thermal_before, thermal_after)
    write_new(TRACE / "stdout.log", stdout)
    write_new(TRACE / "stderr.log", stderr)
    rows, trace_errors, trace_summary = parse_trace(stderr)
    write_json(TRACE / "TRACE_ROWS.json", {
        "schema": "lay.m3-v9-materialization-trace-rows.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "rows": rows,
        "parse_errors": trace_errors,
    })
    if trace_summary is not None:
        write_json(TRACE / "TRACE_SUMMARY.json", trace_summary)
    receipt_path = TRACE / "subject/SUBJECT_RECEIPT.json"
    receipt = None
    if receipt_path.is_file():
        try:
            receipt = load_json(receipt_path)
        except BaseException as error:
            controller_error = controller_error or f"{type(error).__name__}: {error}"
    complete = isinstance(receipt, dict) and receipt.get("schema") == "lay.m3-end-to-end-test-owner.v1"
    subject_verdict = receipt.get("verdict") if complete else None
    exit_code = process.returncode if process is not None else None
    exit_consistent = (subject_verdict == "M3_END_TO_END_TEST_OWNER_PASS" and exit_code == 0) or (
        complete and subject_verdict != "M3_END_TO_END_TEST_OWNER_PASS" and exit_code == 101
    )
    producer_complete = (
        controller_error is None
        and direct_identity["observed"] is True
        and complete
        and exit_consistent
        and not trace_errors
        and trace_summary is not None
    )
    wrapper = {
        "schema": "lay.m3-v9-trace-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_TRACE_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        "controller_error": controller_error,
        "marker": marker,
        "command": command,
        "environment": environment,
        "direct_exec_identity": direct_identity,
        "exit_code": exit_code,
        "subject_receipt": receipt,
        "outputs_complete": complete,
        "exit_pair_consistent": exit_consistent,
        "trace_rows": len(rows),
        "trace_parse_errors": trace_errors,
        "trace_summary": trace_summary,
        "thermal_before": thermal_before,
        "thermal_after": thermal_after,
        "thermal_throttle_drift": thermal_change,
        "subject_executions": int(process is not None),
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
    }
    write_json(TRACE / "TRACE_WRAPPER.json", wrapper)
    write_manifest(TRACE)
    seal_tree(TRACE)
    append_state(
        2,
        "TRACE_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        trace_wrapper_sha256=sha256_file(TRACE / "TRACE_WRAPPER.json"),
    )
    return wrapper


def owned_processes() -> list[dict[str, Any]]:
    processes = []
    for path in pathlib.Path("/proc").iterdir():
        if not path.name.isdigit():
            continue
        try:
            command = (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip()
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if command and any(token in command for token in (TASK_ID, SCIENTIFIC_TEST, "perf record", "perf stat")):
            processes.append({"pid": int(path.name), "command": command})
    return sorted(processes, key=lambda row: row["pid"])


def status() -> dict[str, Any]:
    return {
        "schema": "lay.m3-v9-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_STATUS",
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "bootstrap_exists": BOOTSTRAP.exists(),
        "executable_exists": EXECUTABLE.exists(),
        "trace_exists": TRACE.exists(),
        "latest_state": latest_state(),
        "markers": marker_inventory(),
        "active_owned_processes": owned_processes(),
    }


def self_check() -> dict[str, Any]:
    command = scientific_command(TRACE / "subject")
    executable_index = command.index(str(EXECUTABLE))
    need(ACTIONS == ("self-check", "status", "bootstrap", "create-marker", "trace-once"), "action registry drift")
    need(ROUTES == ("TRACE",) and MARKER_NAME == "trace", "route registry drift")
    need(command[executable_index - 3 : executable_index] == ["/usr/bin/taskset", "-c", "0"], "direct command drift")
    need(command[executable_index + 1 :] == ["--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1"], "scientific argv tail drift")
    need(not any("ld-linux" in token for token in command), "loader route became reachable")
    need("LAY_L2_FIELD_TRACE=1" in command, "trace environment absent")
    need(EXECUTABLE == V8R3_ELF and not str(EXECUTABLE).startswith(str(PARENT)), "sealed V8R3 ELF binding drift")
    need(SCIENTIFIC_TEST.endswith("m3_end_to_end_physical_proof"), "scientific test drift")
    need(len({V8R1_TASK_ID, V8R2_TASK_ID, V8R3_TASK_ID, TASK_ID}) == 4, "namespace registry drift")
    sample = (
        "productive_v90_materialization_trace surfaces=4 emitted=4 setup_us=1 "
        "projection_us=2 classify_us=3 gate_us=100 evidence_us=4\n"
    ).encode() * EXPECTED_TRACE_ROWS
    rows, errors, summary = parse_trace(sample)
    need(not errors and len(rows) == EXPECTED_TRACE_ROWS and summary is not None, "trace parser self-check failed")
    need(summary["tail"]["dominant_stage"] == "gate_us", "trace dominance estimator drift")
    return {
        "schema": "lay.m3-v9-remote-self-check.v1",
        "verdict": "M3_V9_REMOTE_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(pathlib.Path(__file__)),
        "actions": list(ACTIONS),
        "routes": list(ROUTES),
        "markers": ["trace.available"],
        "bootstrap_parent_mode": "0755",
        "trace_argv": command,
        "trace_rows": EXPECTED_TRACE_ROWS,
        "measured_rows": MEASURED_ROWS,
        "tail_rows": TAIL_ROWS,
        "synthetic_dominant_stage": summary["tail"]["dominant_stage"],
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "production_authority_admitted": False,
    }


def with_route_lock(callback: Any) -> dict[str, Any]:
    lock_path = STATE / "route.lock"
    need(lock_path.is_file(), "route lock absent")
    with lock_path.open("rb") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        return callback()


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=ACTIONS)
    value.add_argument("--bootstrap", type=pathlib.Path)
    value.add_argument("--audit", type=pathlib.Path)
    value.add_argument("--quiet", type=pathlib.Path)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        controller_sha = sha256_file(pathlib.Path(__file__))
        if args.action == "self-check":
            value = self_check()
        elif args.action == "status":
            value = status()
        elif args.action == "bootstrap":
            need(args.bootstrap is not None, "--bootstrap is required")
            value = bootstrap_once(args.bootstrap)
        elif args.action == "create-marker":
            need(args.audit is not None, "--audit is required")
            value = with_route_lock(lambda: create_marker(args.audit, controller_sha))
        else:
            need(args.quiet is not None, "--quiet is required")
            value = with_route_lock(lambda: trace_once(args.quiet, controller_sha))
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.m3-v9-remote-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
