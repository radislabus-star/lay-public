#!/usr/bin/env python3
"""Resume-safe local orchestrator for the V11 paired target-host comparison."""

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
CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_CONTROLLER = ROOT / "scripts/lay-v11-admission-lexical-fact-reuse-remote.py"
SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"
PREFLIGHT_MANIFEST = ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_EXECUTION_V3_2026-08-27.json"
PREFLIGHT_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_EXECUTION_V3_PREFLIGHT_2026-08-27.json"
IMPLEMENTATION_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_LEXICAL_FACT_REUSE_V11_IMPLEMENTATION_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"

TASK_ID = "slice8b-v11-admission-lexical-fact-reuse-paired-v3-20260827"
TRANSACTION_ID = "edc8266912d2d41ce112997f32dfe1f0748ae2699f777e70af55a54f55e0461e"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_UPLOAD = pathlib.PurePosixPath("/home/e/.cache") / f"lay-v11-upload-{TRANSACTION_ID}"
REMOTE_FINAL_CONTROLLER = REMOTE_PARENT / "bootstrap/remote-controller.py"
LOCAL_EVIDENCE = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_EXECUTION_V1_2026-08-27"

PREFLIGHT_MANIFEST_SHA256 = "78f1f5146acbf53e025922f805994534ee5ef6edbe5b19660d10d4eb926dd5e2"
PREFLIGHT_RECEIPT_SHA256 = "bcb47347eda5fc9f3667a4fb4cfbc795e108ad40b322432db6684206f5181eeb"
IMPLEMENTATION_RECEIPT_SHA256 = "e3e05ffbf1d25d8d2b0d2b6095e268769737c11867fd71b688db5c1d63b6a9f9"
SOURCE_SIZE = 119_643
SOURCE_SHA256 = "e8a6a182753084659e00ccd5e20238d585d859437824609a987ea03ce6edca72"

ACTIONS = ("self-check", "status", "run")
REMOTE_ACTIONS = ("self-check", "status", "bootstrap", "build-once", "run-once", "terminal")
REMOTE_ROUTES = ("BUILD", "B0", "B1")
TERMINAL_STATES = {
    "V11_PAIRED_COMPARISON_COMPLETE",
    "BLOCKED_CONTROLLER",
    "BLOCKED_BOOTSTRAP",
    "BLOCKED_BUILD",
    "BLOCKED_PROVENANCE",
    "BLOCKED_SEMANTIC",
}


class ControllerError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ControllerError(message)


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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


def run_command(
    argv: Sequence[str],
    *,
    timeout: float = 3600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(argv), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if check and result.returncode != 0:
        raise ControllerError(
            f"command failed rc={result.returncode}: {list(argv)!r}; "
            f"stdout={result.stdout[-3000:].decode(errors='replace')!r}; "
            f"stderr={result.stderr[-3000:].decode(errors='replace')!r}"
        )
    return result


def ssh(argv: Sequence[str], *, timeout: float = 3600, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return run_command(
        [
            "/usr/bin/ssh", "-i", str(SSH_KEY), "-o", "BatchMode=yes",
            "-o", "ConnectTimeout=10", REMOTE, *argv,
        ],
        timeout=timeout,
        check=check,
    )


def scp_to(local: pathlib.Path, remote: pathlib.PurePosixPath, *, recursive: bool = False) -> None:
    argv = ["/usr/bin/scp", "-q"]
    if recursive:
        argv.append("-r")
    argv.extend(["-i", str(SSH_KEY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", str(local), f"{REMOTE}:{remote}"])
    run_command(argv, timeout=3600)


def scp_from(remote: pathlib.PurePosixPath, local: pathlib.Path, *, recursive: bool = False) -> None:
    argv = ["/usr/bin/scp", "-q"]
    if recursive:
        argv.append("-r")
    argv.extend(["-i", str(SSH_KEY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", f"{REMOTE}:{remote}", str(local)])
    run_command(argv, timeout=3600)


def remote_exists(path: pathlib.PurePosixPath) -> bool:
    return ssh(["/usr/bin/test", "-e", str(path)], timeout=30, check=False).returncode == 0


def parse_json_output(result: subprocess.CompletedProcess[bytes], context: str) -> dict[str, Any]:
    lines = [line for line in result.stdout.decode(errors="replace").splitlines() if line.strip()]
    need(lines, f"{context} returned no JSON")
    try:
        value = json.loads(lines[-1])
    except json.JSONDecodeError as error:
        raise ControllerError(f"{context} returned invalid JSON: {error}") from error
    need(isinstance(value, dict), f"{context} response is not an object")
    if result.returncode != 0:
        raise ControllerError(f"{context} failed: {value}")
    return value


def controller_path() -> pathlib.PurePosixPath:
    return REMOTE_FINAL_CONTROLLER if remote_exists(REMOTE_FINAL_CONTROLLER) else REMOTE_UPLOAD / "remote-controller.py"


def remote_call(action: str, *arguments: str, timeout: float = 3600) -> dict[str, Any]:
    need(action in REMOTE_ACTIONS, f"unknown remote action: {action}")
    path = controller_path()
    result = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(path), action, *arguments], timeout=timeout, check=False)
    return parse_json_output(result, f"remote {action}")


def literal(tree: ast.Module, name: str) -> Any:
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id == name:
                    return ast.literal_eval(node.value)
    raise ControllerError(f"literal absent: {name}")


def self_check() -> dict[str, Any]:
    require_file(PREFLIGHT_MANIFEST, size=13_815, digest=PREFLIGHT_MANIFEST_SHA256, mode="0444")
    require_file(PREFLIGHT_RECEIPT, size=6_767, digest=PREFLIGHT_RECEIPT_SHA256, mode="0444")
    require_file(IMPLEMENTATION_RECEIPT, size=4_722, digest=IMPLEMENTATION_RECEIPT_SHA256, mode="0444")
    require_file(SOURCE, size=SOURCE_SIZE, digest=SOURCE_SHA256, mode="0664")
    need(load_json(PREFLIGHT_RECEIPT).get("verdict") == "READY_TO_IMPLEMENT", "effective preflight is not READY")
    need(load_json(IMPLEMENTATION_RECEIPT).get("verdict") == "V11_MECHANISM_IMPLEMENTED_UNRUN", "implementation predecessor drift")
    tree = ast.parse(REMOTE_CONTROLLER.read_text(), filename=str(REMOTE_CONTROLLER))
    need(tuple(literal(tree, "ACTIONS")) == REMOTE_ACTIONS, "remote action registry drift")
    need(tuple(literal(tree, "ROUTES")) == REMOTE_ROUTES, "remote route registry drift")
    need(literal(tree, "MODES") == {"B0": "UNCACHED", "B1": "REUSE"}, "remote mode registry drift")
    result = run_command([sys.executable, str(REMOTE_CONTROLLER), "self-check"])
    remote_static = parse_json_output(result, "local remote-controller self-check")
    need(remote_static.get("verdict") == "V11_REMOTE_CONTROLLER_STATIC_PASS", "remote controller static failure")
    need(remote_static.get("perf_reachable") is False and remote_static.get("runtime_install_reachable") is False, "forbidden route became reachable")
    return {
        "schema": "lay.v11-paired-local-self-check.v1",
        "verdict": "V11_LOCAL_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "actions": list(ACTIONS),
        "remote": remote_static,
        "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "implementation_receipt_sha256": IMPLEMENTATION_RECEIPT_SHA256,
        "source_sha256": SOURCE_SHA256,
        "remote_writes": 0,
        "builds": 0,
        "subjects": 0,
    }


def make_bundle() -> pathlib.Path:
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-v11-paired-bundle-"))
    shutil.copyfile(REMOTE_CONTROLLER, temporary / "remote-controller.py")
    shutil.copyfile(SOURCE, temporary / "proposal_admission.rs")
    admission = {
        "schema": "lay.v11-paired-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V11_PAIRED_EXECUTION_ADMITTED",
        "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "implementation_receipt_sha256": IMPLEMENTATION_RECEIPT_SHA256,
        "source_size_bytes": SOURCE_SIZE,
        "source_sha256": SOURCE_SHA256,
        "local_controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "routes": ["BUILD", "B0", "B1"],
        "modes": {"B0": "UNCACHED", "B1": "REUSE"},
        "one_build": True,
        "same_elf": True,
        "small_performance_miss_terminal": False,
        "runtime_authority_admitted": False,
    }
    write_json(temporary / "ADMISSION.json", admission, 0o444)
    return temporary


def ensure_upload() -> dict[str, Any]:
    if remote_exists(REMOTE_PARENT):
        return {"state": "namespace-present", "path": str(REMOTE_PARENT)}
    if not remote_exists(REMOTE_UPLOAD):
        bundle = make_bundle()
        try:
            scp_to(bundle, REMOTE_UPLOAD, recursive=True)
        finally:
            shutil.rmtree(bundle, ignore_errors=True)
    need(remote_exists(REMOTE_UPLOAD / "ADMISSION.json"), "remote upload is incomplete")
    check = remote_call("self-check")
    need(check.get("verdict") == "V11_REMOTE_CONTROLLER_STATIC_PASS", "uploaded controller self-check failed")
    return {"state": "upload-ready", "path": str(REMOTE_UPLOAD), "remote_self_check": check}


def verify_remote_manifests(root: pathlib.Path) -> dict[str, int]:
    manifests = sorted(root.rglob("SHA256SUMS"))
    need(len(manifests) >= 5, "remote evidence manifests incomplete")
    entries = 0
    for manifest in manifests:
        for line in manifest.read_text().splitlines():
            digest, relative = line.split("  ", 1)
            path = manifest.parent / relative
            need(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
            entries += 1
    return {"manifests": len(manifests), "entries": entries}


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode(), 0o444)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish_local() -> dict[str, Any]:
    if LOCAL_EVIDENCE.exists():
        return load_json(LOCAL_EVIDENCE / "LOCAL_RECEIPT.json")
    LOCAL_EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix=f"{LOCAL_EVIDENCE.name}.stage-", dir=LOCAL_EVIDENCE.parent))
    try:
        scp_from(REMOTE_PARENT, stage, recursive=True)
        copied = stage / TASK_ID
        need(copied.is_dir(), "remote evidence copy absent")
        remote_evidence = stage / "REMOTE_EVIDENCE"
        os.rename(copied, remote_evidence)
        manifests = verify_remote_manifests(remote_evidence)
        terminal_path = remote_evidence / "terminal-v1/TERMINAL.json"
        terminal = load_json(terminal_path) if terminal_path.is_file() else None
        latest_states = sorted((remote_evidence.parent / "REMOTE_STATE").glob("STATE-*.json")) if (remote_evidence.parent / "REMOTE_STATE").is_dir() else []
        inputs = stage / "LOCAL_INPUTS"
        inputs.mkdir(mode=0o700)
        for name, source in (
            ("local-controller.py", CONTROLLER),
            ("remote-controller.py", REMOTE_CONTROLLER),
            ("preflight-manifest.json", PREFLIGHT_MANIFEST),
            ("preflight-receipt.json", PREFLIGHT_RECEIPT),
            ("implementation-receipt.json", IMPLEMENTATION_RECEIPT),
            ("proposal_admission.rs", SOURCE),
        ):
            shutil.copyfile(source, inputs / name)
        receipt = {
            "schema": "lay.v11-paired-local-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": terminal.get("verdict") if isinstance(terminal, dict) else "BLOCKED_PROVENANCE",
            "remote_terminal_sha256": sha256_file(terminal_path) if terminal_path.is_file() else None,
            "remote_manifests": manifests,
            "local_controller_sha256": sha256_file(CONTROLLER),
            "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
            "source_sha256": SOURCE_SHA256,
            "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "terminal": terminal,
            "one_build": True,
            "subject_executions": 2 if isinstance(terminal, dict) else None,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "pmu_events": 0,
            "runtime_authority_changed": False,
            "production_authority_admitted": False,
        }
        write_json(stage / "LOCAL_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, LOCAL_EVIDENCE)
        fsync_dir(LOCAL_EVIDENCE.parent)
        return receipt
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def remote_status() -> dict[str, Any]:
    if not remote_exists(REMOTE_PARENT):
        return {
            "schema": "lay.v11-paired-local-status.v1",
            "verdict": "V11_REMOTE_NAMESPACE_ABSENT",
            "task_id": TASK_ID,
            "upload_exists": remote_exists(REMOTE_UPLOAD),
            "local_evidence_exists": LOCAL_EVIDENCE.exists(),
        }
    return remote_call("status", timeout=60)


def run_pipeline() -> dict[str, Any]:
    local = self_check()
    if LOCAL_EVIDENCE.exists():
        receipt = load_json(LOCAL_EVIDENCE / "LOCAL_RECEIPT.json")
        return {"verdict": receipt["verdict"], "resumed": True, "local_receipt": receipt}
    ensure_upload()
    if not remote_exists(REMOTE_PARENT):
        result = remote_call("bootstrap", "--bundle", str(REMOTE_UPLOAD), timeout=1800)
        need(result.get("verdict") == "V11_ALL_MARKERS_AVAILABLE", f"bootstrap failed: {result}")
    while True:
        status = remote_call("status", timeout=60)
        latest = status.get("latest_state") or {}
        state = latest.get("state")
        if state == "ALL_MARKERS_AVAILABLE":
            result = remote_call("build-once", timeout=10_800)
            need(result.get("verdict") == "V11_BUILD_CREATED", f"build ended terminally: {result}")
        elif state == "BUILD_CREATED":
            result = remote_call("run-once", "--route", "B0", timeout=10_800)
            need(result.get("verdict") == "V11_B0_CREATED", f"B0 ended terminally: {result}")
        elif state == "B0_CREATED":
            result = remote_call("run-once", "--route", "B1", timeout=10_800)
            need(result.get("verdict") == "V11_B1_CREATED", f"B1 ended terminally: {result}")
        elif state == "B1_CREATED":
            remote_call("terminal", timeout=600)
        elif state in TERMINAL_STATES:
            receipt = publish_local()
            return {
                "schema": "lay.v11-paired-run-result.v1",
                "verdict": receipt["verdict"],
                "local_self_check": local["verdict"],
                "local_evidence": str(LOCAL_EVIDENCE),
                "terminal": receipt.get("terminal"),
            }
        else:
            raise ControllerError(f"unknown remote state: {state!r}")


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=ACTIONS)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "status":
            value = remote_status()
        else:
            value = run_pipeline()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v11-paired-local-error.v1",
            "verdict": "ERROR",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "error": f"{type(error).__name__}: {error}",
        }, ensure_ascii=False, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
