#!/usr/bin/env python3
"""Permanently supersede the unrun Clean V2 route before C1."""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import os
import pathlib
import shlex
import socket
import stat
import subprocess
import sys
import tempfile
import time
from typing import Any


CLEAN_TASK_ID = "slice8b-v10-clean-speed-v2-20260825"
SUPERSESSION_TASK_ID = "slice8b-v10-clean-speed-v2-supersession-v1-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_FINAL = pathlib.Path("/home/e/.local/share/lay/provenance") / CLEAN_TASK_ID
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / CLEAN_TASK_ID
REMOTE_EXECUTION = bool(globals().get("REMOTE_EXECUTION", False))

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V3_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_PREFLIGHT_V3_2026-08-25.json"
)
LOCAL_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V1_2026-08-25.json"
)

PREFLIGHT_MANIFEST_SHA256 = "729951e4670a1dd61f4bc29e1350981d5cd84e3bcc609436474a3e228482a90c"
PREFLIGHT_MANIFEST_IDENTITY_SHA256 = "c4fb686565e3d8e7415c4bc675913d2f3ad6952d42cd4ed23964913c3e0a48ca"
PREFLIGHT_RECEIPT_SHA256 = "54e1687c3c5095e9d826a347c2defb6971e93f87d70293d44724f184197bddbd"
CLEAN_CONTROLLER_SHA256 = "10167af258b0412582b17667deb41f14594b5ec12dd88f3aba12094d707383a5"
C1_ORDER_SHA256 = "6cfa8c1156bde7cede5320f6924d3eb5c0e179907c08e82e2121885d7517f406"
C1_CONTRACT_SHA256 = "1a35eeb0f5bb1e83e6785750a3a3857805bc62c93bc25d88d450874ae9f3f3d6"
C1_STRUCTURAL_RECEIPT_SHA256 = "d85d413d81e14b93f3de77b7e598bb766fb3fdd69519a1e225e5a3e2f7f69925"
REMOTE_ABSENCE_AUDIT_SHA256 = "2d196b500cdd26acb25514e2fa90b0b7159566d22a3613c7e9b5fee652a2997d"
ACTIVE_V11_SHA256 = "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b"

PROTECTED = {
    "clean_controller": (
        PROJECT_ROOT / "scripts/lay-v10-hardware-clean.py",
        "0755",
        19_253,
        CLEAN_CONTROLLER_SHA256,
    ),
    "clean_preflight_manifest": (
        PROJECT_ROOT / (
            "docs/structural_gates/preflights/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_2026-08-25.json"
        ),
        "0664",
        11_474,
        "181047ea6b448767a6ff1f66afc1c04e6d9b14062deb395c687d3f5fb2b7632a",
    ),
    "clean_preflight_receipt": (
        PROJECT_ROOT / (
            "docs/structural_gates/receipts/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_PREFLIGHT_V2_2026-08-25.json"
        ),
        "0664",
        4_371,
        "c69302a79844a6d8a60f26bfcd95135c9ab153ac19af91dcc5ac98494a6227a1",
    ),
    "clean_prepared_receipt": (
        PROJECT_ROOT / (
            "docs/structural_gates/receipts/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_PREPARED_V2_2026-08-25.json"
        ),
        "0444",
        9_165,
        "9889209d5a62ebda7e8b5509bac05a8fb4a7c9dd64e67a4a76ce607629ca7ea9",
    ),
    "c1_order": (
        PROJECT_ROOT / (
            "docs/structural_gates/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_ORDER_CORRECTION_V1_2026-08-25.md"
        ),
        "0664",
        4_375,
        C1_ORDER_SHA256,
    ),
    "c1_contract": (
        PROJECT_ROOT / (
            "docs/structural_gates/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRECT_LATENCY_CONTRACT_V1_2026-08-25.md"
        ),
        "0664",
        14_667,
        C1_CONTRACT_SHA256,
    ),
    "c1_structural_receipt": (
        PROJECT_ROOT / (
            "docs/structural_gates/receipts/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_ROUTE_V1_2026-08-25/"
            "HIERARCHICAL_RECEIPT.json"
        ),
        "0664",
        26_573,
        C1_STRUCTURAL_RECEIPT_SHA256,
    ),
    "remote_absence_audit": (
        PROJECT_ROOT / (
            "docs/structural_gates/receipts/"
            "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_"
            "SUPERSESSION_REMOTE_AUDIT_V1_2026-08-25.json"
        ),
        "0664",
        733,
        REMOTE_ABSENCE_AUDIT_SHA256,
    ),
    "active_v11": (
        PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs",
        "0664",
        124_127,
        ACTIVE_V11_SHA256,
    ),
}


class SupersessionError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SupersessionError(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


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


def write_new_atomic(path: pathlib.Path, content: bytes, mode: int) -> None:
    require(not path.exists(), f"refusing to replace existing file: {path}")
    temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}-{time.time_ns()}")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as output:
            output.write(content)
            output.flush()
            os.fsync(output.fileno())
    except BaseException:
        raise
    os.chmod(temporary, mode)
    os.replace(temporary, path)
    directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def maybe_fault(point: str | None, expected: str) -> None:
    if point == expected:
        raise SupersessionError(f"injected fault: {point}")


def inspect_state(state: pathlib.Path, final: pathlib.Path) -> dict[str, Any]:
    result: dict[str, Any] = {
        "hostname": socket.gethostname(),
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        "state_path": str(state),
        "state_exists": state.exists(),
        "final_path": str(final),
        "final_exists": final.exists(),
    }
    if not state.exists():
        return result
    require(state.is_dir(), f"state path is not a directory: {state}")
    entries = sorted(path.name for path in state.iterdir())
    writable = sum(1 for path in [state, *state.iterdir()] if mode_string(path)[-1] in "2367")
    result.update(
        {
            "state_mode": mode_string(state),
            "entries": entries,
            "file_count": sum(path.is_file() for path in state.iterdir()),
            "writable_objects": writable,
        }
    )
    tombstone_path = state / "SUPERSEDED_UNRUN.json"
    sums_path = state / "SHA256SUMS"
    if tombstone_path.is_file():
        tombstone_bytes = tombstone_path.read_bytes()
        result["tombstone_mode"] = mode_string(tombstone_path)
        result["tombstone_sha256"] = sha256_bytes(tombstone_bytes)
        result["tombstone"] = json.loads(tombstone_bytes)
    if sums_path.is_file():
        sums_bytes = sums_path.read_bytes()
        result["sha256sums_mode"] = mode_string(sums_path)
        result["sha256sums_sha256"] = sha256_bytes(sums_bytes)
        result["sha256sums"] = sums_bytes.decode()
        expected = result.get("tombstone_sha256")
        result["sha256sums_verified"] = sums_bytes == (
            f"{expected}  SUPERSEDED_UNRUN.json\n".encode()
        )
    return result


def publish_state(
    state: pathlib.Path,
    final: pathlib.Path,
    tombstone: dict[str, Any],
    fault_after: str | None = None,
) -> dict[str, Any]:
    require(not final.exists(), f"Clean V2 result already exists: {final}")
    require(not state.exists(), f"Clean V2 state already exists: {state}")
    require(state.parent.is_dir(), f"state parent is absent: {state.parent}")
    maybe_fault(fault_after, "before_mkdir")

    state.mkdir(mode=0o700)
    maybe_fault(fault_after, "after_mkdir")

    tombstone_bytes = canonical_json(tombstone)
    write_new_atomic(state / "SUPERSEDED_UNRUN.json", tombstone_bytes, 0o600)
    maybe_fault(fault_after, "after_tombstone")

    sums = f"{sha256_bytes(tombstone_bytes)}  SUPERSEDED_UNRUN.json\n".encode()
    write_new_atomic(state / "SHA256SUMS", sums, 0o600)
    maybe_fault(fault_after, "after_manifest")

    require((state / "SUPERSEDED_UNRUN.json").read_bytes() == tombstone_bytes, "tombstone drift")
    require((state / "SHA256SUMS").read_bytes() == sums, "SHA256SUMS drift")
    os.chmod(state / "SUPERSEDED_UNRUN.json", 0o444)
    os.chmod(state / "SHA256SUMS", 0o444)
    maybe_fault(fault_after, "after_files_sealed")

    os.chmod(state, 0o555)
    maybe_fault(fault_after, "after_tree_sealed")
    result = inspect_state(state, final)
    require(result["entries"] == ["SHA256SUMS", "SUPERSEDED_UNRUN.json"], "unexpected state entries")
    require(result["state_mode"] == "0555", "state tree is not sealed")
    require(result["tombstone_mode"] == "0444", "tombstone is writable")
    require(result["sha256sums_mode"] == "0444", "manifest is writable")
    require(result["writable_objects"] == 0, "writable remote objects remain")
    require(result["sha256sums_verified"] is True, "remote SHA256SUMS mismatch")
    require(result["final_exists"] is False, "Clean V2 final path was created")
    return result


def verify_local_baselines() -> None:
    require(sha256_file(PREFLIGHT_MANIFEST) == PREFLIGHT_MANIFEST_SHA256, "preflight manifest drift")
    require(sha256_file(PREFLIGHT_RECEIPT) == PREFLIGHT_RECEIPT_SHA256, "preflight receipt drift")
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "preflight did not pass")
    require(preflight.get("safe_to_implement") is True, "preflight did not admit implementation")
    require(
        preflight.get("manifest_sha256") == PREFLIGHT_MANIFEST_IDENTITY_SHA256,
        "preflight canonical identity mismatch",
    )
    for name, (path, mode, size, digest) in PROTECTED.items():
        require(path.is_file(), f"protected file absent: {name}")
        require(mode_string(path) == mode, f"protected mode drift: {name}")
        require(path.stat().st_size == size, f"protected size drift: {name}")
        require(sha256_file(path) == digest, f"protected SHA-256 drift: {name}")

    clean_source = PROTECTED["clean_controller"][0].read_text()
    require(f'REMOTE_STATE_TEXT = f"/home/e/.local/state/lay/{{TASK_ID}}"' in clean_source, "state path definition drift")
    require('require(not state.exists(), f"clean attempt already consumed: {state}")' in clean_source, "run guard drift")
    require('"measurement_attempt_consumed": state.exists()' in clean_source, "readiness observation drift")


def tombstone(controller_sha256: str) -> dict[str, Any]:
    return {
        "schema": "lay.v10.clean-speed-v2-superseded-unrun.v1",
        "task_id": CLEAN_TASK_ID,
        "supersession_task_id": SUPERSESSION_TASK_ID,
        "status": "SUPERSEDED_UNRUN_BY_C1",
        "published_at": now(),
        "subject_executed": False,
        "measurement_produced": False,
        "clean_result_published": False,
        "old_route_physically_runnable": False,
        "rollback": "RETAIN_TOMBSTONE",
        "remote_state_path": str(REMOTE_STATE),
        "remote_final_path": str(REMOTE_FINAL),
        "identities": {
            "supersession_controller_sha256": controller_sha256,
            "clean_v2_controller_sha256": CLEAN_CONTROLLER_SHA256,
            "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
            "preflight_manifest_identity_sha256": PREFLIGHT_MANIFEST_IDENTITY_SHA256,
            "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "c1_order_sha256": C1_ORDER_SHA256,
            "c1_contract_sha256": C1_CONTRACT_SHA256,
            "c1_structural_receipt_sha256": C1_STRUCTURAL_RECEIPT_SHA256,
            "remote_absence_audit_sha256": REMOTE_ABSENCE_AUDIT_SHA256,
            "active_v11_sha256": ACTIVE_V11_SHA256,
        },
        "scope": {
            "b_matrix": "HOLD_UNTIL_C1_FAIL_PAPER_DECISION",
            "c1_implementation": "NOT_STARTED",
            "v12": "NOT_ADMITTED",
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        },
    }


REMOTE_BOOTSTRAP = (
    "import hashlib,sys\n"
    "source=sys.stdin.buffer.read()\n"
    "expected=sys.argv[1]\n"
    "action=sys.argv[2]\n"
    "payload=sys.argv[3]\n"
    "assert hashlib.sha256(source).hexdigest()==expected, 'controller source SHA mismatch'\n"
    "sys.argv=['lay-v10-c1-supersede-clean-v2.py',action,payload]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-c1-supersede-clean-v2.py>',"
    "'REMOTE_EXECUTION':True}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def remote_call(action: str, payload: dict[str, Any] | None = None) -> subprocess.CompletedProcess[bytes]:
    source = pathlib.Path(__file__).read_bytes()
    encoded = ""
    if payload is not None:
        encoded = base64.b64encode(canonical_json(payload)).decode()
    command = shlex.join(
        ["/usr/bin/python3", "-c", REMOTE_BOOTSTRAP, sha256_bytes(source), action, encoded]
    )
    return subprocess.run(
        [
            "/usr/bin/ssh",
            "-i",
            str(pathlib.Path.home() / ".ssh/mega-mini-admin"),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            REMOTE,
            command,
        ],
        input=source,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def parse_remote_result(result: subprocess.CompletedProcess[bytes], action: str) -> dict[str, Any]:
    require(result.returncode == 0, f"remote {action} failed: {result.stderr[-2000:]!r}")
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise SupersessionError(f"remote {action} returned invalid JSON: {error}") from error
    require(isinstance(value, dict), f"remote {action} returned a non-object")
    return value


def validate_remote_identity(value: dict[str, Any]) -> None:
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote machine identity mismatch")
    require(value.get("state_path") == str(REMOTE_STATE), "remote state path mismatch")
    require(value.get("final_path") == str(REMOTE_FINAL), "remote final path mismatch")


def remote_status() -> None:
    require(REMOTE_EXECUTION, "remote-status is not a local action")
    value = inspect_state(REMOTE_STATE, REMOTE_FINAL)
    print(json.dumps(value, sort_keys=True))


def remote_publish(encoded: str) -> None:
    require(REMOTE_EXECUTION, "remote-publish is not a local action")
    value = json.loads(base64.b64decode(encoded, validate=True))
    require(isinstance(value, dict), "tombstone payload is not an object")
    require(socket.gethostname() == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(sha256_file(pathlib.Path("/etc/machine-id")) == REMOTE_MACHINE_ID_SHA256, "remote machine identity mismatch")
    try:
        result = publish_state(REMOTE_STATE, REMOTE_FINAL, value)
    except BaseException as error:
        state_exists = REMOTE_STATE.exists()
        failure = {
            "verdict": "SUPERSESSION_PARTIAL_FAIL_CLOSED" if state_exists else "SUPERSESSION_NOT_STARTED",
            "state_exists": state_exists,
            "final_exists": REMOTE_FINAL.exists(),
            "error": str(error),
        }
        print(json.dumps(failure, sort_keys=True))
        raise
    print(json.dumps({"verdict": "REMOTE_TOMBSTONE_SEALED", "state": result}, sort_keys=True))


def self_check() -> None:
    verify_local_baselines()
    source = pathlib.Path(__file__).read_text()
    forbidden = [
        "/usr/bin/" + "perf",
        "cargo-" + "guard",
        "system" + "ctl",
        "pk" + "ill",
        "kill" + "all",
        "cpu" + "power",
        "gnome-" + "extensions",
    ]
    require(not any(token in source for token in forbidden), "forbidden command token in controller")

    controller_sha = sha256_bytes(source.encode())
    payload = tombstone(controller_sha)
    fault_points = [
        "before_mkdir",
        "after_mkdir",
        "after_tombstone",
        "after_manifest",
        "after_files_sealed",
        "after_tree_sealed",
    ]
    for fault in fault_points:
        with tempfile.TemporaryDirectory(prefix="lay-c1-supersession-") as directory:
            root = pathlib.Path(directory)
            parent = root / "state"
            parent.mkdir()
            state = parent / CLEAN_TASK_ID
            final = root / "final" / CLEAN_TASK_ID
            try:
                publish_state(state, final, payload, fault)
            except SupersessionError:
                pass
            else:
                raise SupersessionError(f"fault did not fire: {fault}")
            require(state.exists() is (fault != "before_mkdir"), f"fault state mismatch: {fault}")
            require(not final.exists(), f"fault created final output: {fault}")

    with tempfile.TemporaryDirectory(prefix="lay-c1-supersession-") as directory:
        root = pathlib.Path(directory)
        parent = root / "state"
        parent.mkdir()
        state = parent / CLEAN_TASK_ID
        final = root / "final" / CLEAN_TASK_ID
        observed = publish_state(state, final, payload)
        require(observed["sha256sums_verified"] is True, "self-check manifest mismatch")
        before = {path.name: sha256_file(path) for path in state.iterdir()}
        try:
            publish_state(state, final, payload)
        except SupersessionError:
            pass
        else:
            raise SupersessionError("second publication was accepted")
        after = {path.name: sha256_file(path) for path in state.iterdir()}
        require(before == after, "second publication changed sealed state")

    print(
        json.dumps(
            {
                "verdict": "PASS",
                "checks": 21,
                "fault_points": len(fault_points),
                "ssh_invoked": False,
                "remote_writes": False,
                "subject_executed": False,
            },
            sort_keys=True,
        )
    )


def local_status() -> None:
    verify_local_baselines()
    result = parse_remote_result(remote_call("remote-status"), "status")
    validate_remote_identity(result)
    print(json.dumps(result, indent=2, sort_keys=True))


def local_run() -> None:
    verify_local_baselines()
    require(not LOCAL_RECEIPT.exists(), f"local supersession receipt exists: {LOCAL_RECEIPT}")
    controller_path = pathlib.Path(__file__)
    controller_sha = sha256_file(controller_path)

    before = parse_remote_result(remote_call("remote-status"), "precondition status")
    validate_remote_identity(before)
    require(before.get("state_exists") is False, "Clean V2 state is no longer absent")
    require(before.get("final_exists") is False, "Clean V2 result exists; UNRUN cannot be claimed")

    payload = tombstone(controller_sha)
    publication = remote_call("remote-publish", payload)
    if publication.returncode != 0:
        classification = remote_call("remote-status")
        message = {
            "verdict": "SUPERSESSION_REMOTE_FAILURE_NO_RETRY",
            "publication_stdout": publication.stdout.decode(errors="replace")[-2000:],
            "publication_stderr": publication.stderr.decode(errors="replace")[-2000:],
            "classification_stdout": classification.stdout.decode(errors="replace")[-4000:],
            "classification_stderr": classification.stderr.decode(errors="replace")[-2000:],
        }
        print(json.dumps(message, indent=2, sort_keys=True))
        raise SystemExit(1)

    published = parse_remote_result(publication, "publication")
    require(published.get("verdict") == "REMOTE_TOMBSTONE_SEALED", "remote publication verdict mismatch")
    after = parse_remote_result(remote_call("remote-status"), "post-publication status")
    validate_remote_identity(after)
    require(after.get("state_exists") is True, "remote state path absent after publication")
    require(after.get("final_exists") is False, "Clean V2 final path exists after supersession")
    require(after.get("entries") == ["SHA256SUMS", "SUPERSEDED_UNRUN.json"], "remote entries mismatch")
    require(after.get("state_mode") == "0555", "remote state mode mismatch")
    require(after.get("tombstone_mode") == "0444", "remote tombstone mode mismatch")
    require(after.get("sha256sums_mode") == "0444", "remote manifest mode mismatch")
    require(after.get("writable_objects") == 0, "remote state has writable objects")
    require(after.get("sha256sums_verified") is True, "remote manifest did not verify")
    require(after.get("tombstone") == payload, "remote tombstone payload mismatch")

    local_receipt = {
        "schema": "lay.v10.clean-speed-v2-supersession-local-index.v1",
        "task_id": SUPERSESSION_TASK_ID,
        "recorded_at": now(),
        "verdict": "CLEAN_V2_SUPERSEDED_UNRUN_INDEXED",
        "status": "SUPERSEDED_UNRUN_BY_C1",
        "controller": {
            "path": str(controller_path),
            "mode": mode_string(controller_path),
            "size_bytes": controller_path.stat().st_size,
            "sha256": controller_sha,
        },
        "preflight": {
            "manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
            "manifest_identity_sha256": PREFLIGHT_MANIFEST_IDENTITY_SHA256,
            "receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "verdict": "READY_TO_IMPLEMENT",
        },
        "remote_before": before,
        "remote_after": after,
        "remote_tombstone_sha256": after["tombstone_sha256"],
        "remote_sha256sums_sha256": after["sha256sums_sha256"],
        "subject_executed": False,
        "measurement_produced": False,
        "clean_result_published": False,
        "old_route_physically_runnable": False,
        "rollback": "RETAIN_TOMBSTONE",
        "b_matrix": "HOLD_UNTIL_C1_FAIL_PAPER_DECISION",
        "c1_implementation": "NOT_STARTED",
        "v12": "NOT_ADMITTED",
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
        "documentation_updated": False,
    }
    write_new_atomic(LOCAL_RECEIPT, canonical_json(local_receipt), 0o444)
    require(mode_string(LOCAL_RECEIPT) == "0444", "local receipt is writable")
    print(
        json.dumps(
            {
                "verdict": local_receipt["verdict"],
                "status": local_receipt["status"],
                "remote_state": str(REMOTE_STATE),
                "remote_final_exists": False,
                "local_receipt": str(LOCAL_RECEIPT),
            },
            indent=2,
            sort_keys=True,
        )
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "action",
        choices=("self-check", "status", "run", "remote-status", "remote-publish"),
    )
    parser.add_argument("payload", nargs="?", default="")
    args = parser.parse_args()
    if args.action == "self-check":
        self_check()
    elif args.action == "status":
        local_status()
    elif args.action == "run":
        local_run()
    elif args.action == "remote-status":
        remote_status()
    else:
        remote_publish(args.payload)


if __name__ == "__main__":
    try:
        main()
    except SupersessionError as error:
        print(json.dumps({"verdict": "ERROR", "error": str(error)}, sort_keys=True), file=sys.stderr)
        raise SystemExit(1)
