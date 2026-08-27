#!/usr/bin/env python3
"""Resume-safe local orchestrator for the no-build V11 R1 repair."""

from __future__ import annotations

import argparse
import ast
import importlib.util
import json
import os
import pathlib
import shutil
import sys
import tempfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_CONTROLLER = ROOT / "scripts/lay-v11-admission-lexical-fact-reuse-r1-remote.py"
BASE_REMOTE_CONTROLLER = ROOT / "scripts/lay-v11-admission-lexical-fact-reuse-remote.py"
BASE_LOCAL_PATH = ROOT / "scripts/lay-v11-admission-lexical-fact-reuse.py"
SPEC = importlib.util.spec_from_file_location("lay_v11_local_base", BASE_LOCAL_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load V11 local controller: {BASE_LOCAL_PATH}")
base = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(base)

PREFLIGHT_MANIFEST = ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_R1_EXECUTION_V1_2026-08-27.json"
PREFLIGHT_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_R1_EXECUTION_V1_PREFLIGHT_2026-08-27.json"
FAILURE_ROOT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_EXECUTION_V1_FAILURE_2026-08-27"
FAILURE_RECEIPT = FAILURE_ROOT / "FAILURE_RECEIPT.json"
FAILED_LOCAL_ELF = FAILURE_ROOT / "REMOTE_EVIDENCE/build-v1/m3-v11-test-elf"

TASK_ID = "slice8b-v11-admission-lexical-fact-reuse-paired-r1-v1-20260827"
TRANSACTION_ID = "a6f54d6a2cc8295c139e65a09aa4fdd070a3b913e8b367d31afc7010615cae4d"
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_UPLOAD = pathlib.PurePosixPath("/home/e/.cache") / f"lay-v11-r1-upload-{TRANSACTION_ID}"
REMOTE_FINAL_CONTROLLER = REMOTE_PARENT / "bootstrap/remote-controller.py"
LOCAL_EVIDENCE = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V11_ADMISSION_LEXICAL_FACT_REUSE_PAIRED_R1_EXECUTION_V1_2026-08-27"

PREFLIGHT_MANIFEST_SHA256 = "a02baf5e09ae8e9802add6cf300581448c14021807f1ba889408eaab8315cf56"
PREFLIGHT_RECEIPT_SHA256 = "b8fdc7c20dd772be0494228a98a54cdb8966c8e35ee8e4998ad0d994ff022071"
FAILURE_RECEIPT_SHA256 = "b1f2fc1305169e25e5801594f87ec76e660e4e54df29d7d10c7a86e8b853a506"
ELF_SIZE = 321_129_832
ELF_SHA256 = "dbd5feb315e9537b0797cb98d6f38dd66fdc6ee3e562bf9db479ed2c9f34b51a"
SOURCE_SHA256 = "e8a6a182753084659e00ccd5e20238d585d859437824609a987ea03ce6edca72"

ACTIONS = ("self-check", "status", "run")
REMOTE_ACTIONS = ("self-check", "status", "bootstrap", "run-once", "terminal")
TERMINAL_STATES = {"V11_R1_PAIRED_COMPARISON_COMPLETE", "BLOCKED_PROVENANCE", "BLOCKED_SEMANTIC"}


class RepairError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise RepairError(message)


def remote_exists(path: pathlib.PurePosixPath) -> bool:
    return base.ssh(["/usr/bin/test", "-e", str(path)], timeout=30, check=False).returncode == 0


def controller_path() -> pathlib.PurePosixPath:
    return REMOTE_FINAL_CONTROLLER if remote_exists(REMOTE_FINAL_CONTROLLER) else REMOTE_UPLOAD / "remote-controller.py"


def remote_call(action: str, *arguments: str, timeout: float = 3600) -> dict[str, Any]:
    need(action in REMOTE_ACTIONS, f"unknown remote action: {action}")
    result = base.ssh(
        ["/usr/bin/sudo", "-n", "/usr/bin/python3", str(controller_path()), action, *arguments],
        timeout=timeout,
        check=False,
    )
    return base.parse_json_output(result, f"remote R1 {action}")


def literal(tree: ast.Module, name: str) -> Any:
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id == name:
                    return ast.literal_eval(node.value)
    raise RepairError(f"literal absent: {name}")


def self_check() -> dict[str, Any]:
    base.require_file(PREFLIGHT_MANIFEST, size=9_474, digest=PREFLIGHT_MANIFEST_SHA256, mode="0444")
    base.require_file(PREFLIGHT_RECEIPT, size=3_977, digest=PREFLIGHT_RECEIPT_SHA256, mode="0444")
    base.require_file(FAILURE_RECEIPT, size=2_686, digest=FAILURE_RECEIPT_SHA256, mode="0444")
    base.require_file(FAILED_LOCAL_ELF, size=ELF_SIZE, digest=ELF_SHA256, mode="0444")
    need(base.load_json(PREFLIGHT_RECEIPT).get("verdict") == "READY_TO_IMPLEMENT", "R1 preflight is not READY")
    need(base.load_json(FAILURE_RECEIPT).get("verdict") == "BLOCKED_PROVENANCE", "failed route interpretation drift")
    tree = ast.parse(REMOTE_CONTROLLER.read_text(), filename=str(REMOTE_CONTROLLER))
    need(tuple(literal(tree, "ACTIONS")) == REMOTE_ACTIONS, "R1 action registry drift")
    need(tuple(literal(tree, "ROUTES")) == ("B0R", "B1R"), "R1 route registry drift")
    result = base.run_command([sys.executable, str(REMOTE_CONTROLLER), "self-check"])
    remote = base.parse_json_output(result, "local R1 remote-controller self-check")
    need(remote.get("verdict") == "V11_R1_REMOTE_CONTROLLER_STATIC_PASS", "R1 remote self-check failed")
    need(all(remote.get(key) is False for key in ("cargo_reachable", "rustc_reachable", "perf_reachable", "new_elf_bytes_reachable")), "forbidden R1 route reachable")
    return {
        "schema": "lay.v11-paired-r1-local-self-check.v1",
        "verdict": "V11_R1_LOCAL_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "actions": list(ACTIONS),
        "remote": remote,
        "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "failure_receipt_sha256": FAILURE_RECEIPT_SHA256,
        "elf_sha256": ELF_SHA256,
        "remote_writes": 0,
        "cargo_invocations": 0,
        "subject_executions": 0,
    }


def make_bundle() -> pathlib.Path:
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-v11-r1-bundle-"))
    shutil.copyfile(REMOTE_CONTROLLER, temporary / "remote-controller.py")
    shutil.copyfile(BASE_REMOTE_CONTROLLER, temporary / "v11-base.py")
    admission = {
        "schema": "lay.v11-paired-r1-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V11_R1_EXECUTION_ADMITTED",
        "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "failure_receipt_sha256": FAILURE_RECEIPT_SHA256,
        "elf_size_bytes": ELF_SIZE,
        "elf_sha256": ELF_SHA256,
        "source_sha256": SOURCE_SHA256,
        "local_controller_sha256": base.sha256_file(CONTROLLER),
        "remote_controller_sha256": base.sha256_file(REMOTE_CONTROLLER),
        "base_controller_sha256": base.sha256_file(BASE_REMOTE_CONTROLLER),
        "routes": ["B0R", "B1R"],
        "modes": {"B0R": "UNCACHED", "B1R": "REUSE"},
        "cargo_admitted": False,
        "same_elf": True,
        "small_performance_miss_terminal": False,
        "runtime_authority_admitted": False,
    }
    base.write_json(temporary / "ADMISSION.json", admission, 0o444)
    return temporary


def ensure_upload() -> dict[str, Any]:
    if remote_exists(REMOTE_PARENT):
        return {"state": "namespace-present", "path": str(REMOTE_PARENT)}
    if not remote_exists(REMOTE_UPLOAD):
        bundle = make_bundle()
        try:
            base.scp_to(bundle, REMOTE_UPLOAD, recursive=True)
        finally:
            shutil.rmtree(bundle, ignore_errors=True)
    need(remote_exists(REMOTE_UPLOAD / "ADMISSION.json"), "R1 upload incomplete")
    check = remote_call("self-check")
    need(check.get("verdict") == "V11_R1_REMOTE_CONTROLLER_STATIC_PASS", "uploaded R1 controller failed")
    return {"state": "upload-ready", "path": str(REMOTE_UPLOAD), "remote_self_check": check}


def verify_remote_manifests(root: pathlib.Path) -> dict[str, int]:
    manifests = sorted(root.rglob("SHA256SUMS"))
    need(len(manifests) >= 4, "R1 remote evidence manifests incomplete")
    entries = 0
    for manifest in manifests:
        for line in manifest.read_text().splitlines():
            digest, relative = line.split("  ", 1)
            path = manifest.parent / relative
            need(path.is_file() and base.sha256_file(path) == digest, f"manifest mismatch: {path}")
            entries += 1
    return {"manifests": len(manifests), "entries": entries}


def publish_local() -> dict[str, Any]:
    if LOCAL_EVIDENCE.exists():
        return base.load_json(LOCAL_EVIDENCE / "LOCAL_RECEIPT.json")
    LOCAL_EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(tempfile.mkdtemp(prefix=f"{LOCAL_EVIDENCE.name}.stage-", dir=LOCAL_EVIDENCE.parent))
    try:
        base.scp_from(REMOTE_PARENT, stage, recursive=True)
        copied = stage / TASK_ID
        need(copied.is_dir(), "R1 remote evidence copy absent")
        remote_evidence = stage / "REMOTE_EVIDENCE"
        os.rename(copied, remote_evidence)
        manifests = verify_remote_manifests(remote_evidence)
        terminal_path = remote_evidence / "terminal-v1/TERMINAL.json"
        terminal = base.load_json(terminal_path) if terminal_path.is_file() else None
        copied_elf = remote_evidence / "bootstrap/m3-v11-test-elf"
        need(copied_elf.is_file() and base.sha256_file(copied_elf) == ELF_SHA256, "copied R1 ELF drift")
        inputs = stage / "LOCAL_INPUTS"
        inputs.mkdir(mode=0o700)
        for name, source in (
            ("local-controller.py", CONTROLLER),
            ("remote-controller.py", REMOTE_CONTROLLER),
            ("v11-base.py", BASE_REMOTE_CONTROLLER),
            ("preflight-manifest.json", PREFLIGHT_MANIFEST),
            ("preflight-receipt.json", PREFLIGHT_RECEIPT),
            ("failure-receipt.json", FAILURE_RECEIPT),
        ):
            shutil.copyfile(source, inputs / name)
        receipt = {
            "schema": "lay.v11-paired-r1-local-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": terminal.get("verdict") if isinstance(terminal, dict) else "BLOCKED_PROVENANCE",
            "remote_terminal_sha256": base.sha256_file(terminal_path) if terminal_path.is_file() else None,
            "remote_manifests": manifests,
            "local_controller_sha256": base.sha256_file(CONTROLLER),
            "remote_controller_sha256": base.sha256_file(REMOTE_CONTROLLER),
            "base_controller_sha256": base.sha256_file(BASE_REMOTE_CONTROLLER),
            "elf_sha256": ELF_SHA256,
            "elf_local_storage": "byte-identical remote evidence copy verified before local sealing",
            "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "terminal": terminal,
            "cargo_invocations": 0,
            "subject_executions": 2 if isinstance(terminal, dict) else None,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "pmu_events": 0,
            "runtime_authority_changed": False,
            "production_authority_admitted": False,
        }
        base.write_json(stage / "LOCAL_RECEIPT.json", receipt)
        base.write_manifest(stage)
        base.seal_tree(stage)
        os.rename(stage, LOCAL_EVIDENCE)
        base.fsync_dir(LOCAL_EVIDENCE.parent)
        return receipt
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def remote_status() -> dict[str, Any]:
    if not remote_exists(REMOTE_PARENT):
        return {
            "schema": "lay.v11-paired-r1-local-status.v1",
            "verdict": "V11_R1_REMOTE_NAMESPACE_ABSENT",
            "task_id": TASK_ID,
            "upload_exists": remote_exists(REMOTE_UPLOAD),
            "local_evidence_exists": LOCAL_EVIDENCE.exists(),
        }
    return remote_call("status", timeout=60)


def run_pipeline() -> dict[str, Any]:
    local = self_check()
    if LOCAL_EVIDENCE.exists():
        receipt = base.load_json(LOCAL_EVIDENCE / "LOCAL_RECEIPT.json")
        return {"verdict": receipt["verdict"], "resumed": True, "local_receipt": receipt}
    ensure_upload()
    if not remote_exists(REMOTE_PARENT):
        result = remote_call("bootstrap", "--bundle", str(REMOTE_UPLOAD), timeout=1800)
        need(result.get("verdict") == "V11_R1_MARKERS_AVAILABLE", f"R1 bootstrap failed: {result}")
    while True:
        status = remote_call("status", timeout=60)
        state = (status.get("latest_state") or {}).get("state")
        if state == "R1_MARKERS_AVAILABLE":
            result = remote_call("run-once", "--route", "B0R", timeout=10_800)
            need(result.get("verdict") == "V11_B0R_CREATED", f"B0R ended terminally: {result}")
        elif state == "B0R_CREATED":
            result = remote_call("run-once", "--route", "B1R", timeout=10_800)
            need(result.get("verdict") == "V11_B1R_CREATED", f"B1R ended terminally: {result}")
        elif state == "B1R_CREATED":
            remote_call("terminal", timeout=600)
        elif state in TERMINAL_STATES:
            receipt = publish_local()
            return {
                "schema": "lay.v11-paired-r1-run-result.v1",
                "verdict": receipt["verdict"],
                "local_self_check": local["verdict"],
                "local_evidence": str(LOCAL_EVIDENCE),
                "terminal": receipt.get("terminal"),
            }
        else:
            raise RepairError(f"unknown R1 state: {state!r}")


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
            "schema": "lay.v11-paired-r1-local-error.v1",
            "verdict": "ERROR",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "error": f"{type(error).__name__}: {error}",
        }, ensure_ascii=False, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
