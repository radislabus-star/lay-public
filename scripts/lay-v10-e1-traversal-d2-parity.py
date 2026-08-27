#!/usr/bin/env python3
"""Local orchestrator for the one-shot primary-only D2 semantic parity route."""

from __future__ import annotations

import argparse
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


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

CONTROLLER = pathlib.Path(__file__).resolve()
PROJECT_ROOT = CONTROLLER.parents[1]
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d2-parity-remote.py"
MAP_AUDIT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUCKET_MAP_AUDIT_V1_2026-08-26/D2_BUCKET_MAP_AUDIT_RECEIPT.json"
)
D1_PARITY_REFERENCE = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25/"
    "parity/subject/SUBJECT_RECEIPT.json"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "PARITY_V1_2026-08-26"
)
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_RESULT = REMOTE_PARENT / "parity-v1"

MAP_AUDIT_SHA256 = "8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7"
D1_PARITY_SHA256 = "fbc651637522a3619bb12f35af7645cb5b02bda0fabf35b24a8fc8097b730530"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
PARITY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_semantic_parity"
EXTERNAL_ACTIONS = ("self-check", "parity-once")


class ControllerError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ControllerError(message)


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
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def require_file(path: pathlib.Path, *, digest: str | None = None, size: int | None = None, mode: str | None = None) -> dict[str, Any]:
    value = file_identity(path)
    if digest is not None:
        require(value["sha256"] == digest, f"SHA mismatch: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode mismatch: {path}")
    return value


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


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
    write_new_bytes(path, json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n")


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            values.append({"path": path.relative_to(root).as_posix(), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)})
    return values


def write_sha256sums(root: pathlib.Path) -> None:
    values = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(root / "SHA256SUMS", "".join(f"{value['sha256']}  {value['path']}\n" for value in values).encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts and relative not in seen, f"unsafe manifest path: {relative}")
        seen.add(relative)
        require(sha256_file(root / path) == digest, f"manifest mismatch: {relative}")
    actual = {value["path"] for value in inventory(root) if value["path"] != "SHA256SUMS"}
    require(seen == actual, "manifest membership mismatch")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        require(not path.is_symlink(), f"symlink before seal: {path}")
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


def run(command: Sequence[str], *, input_bytes: bytes | None = None, check: bool = True, timeout: int | None = None) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=timeout)
    if check and result.returncode != 0:
        raise ControllerError(f"command failed ({result.returncode}): {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-6000:]}")
    return result


def ssh_argv(command: Sequence[str]) -> list[str]:
    return ["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", REMOTE, shlex.join(command)]


REMOTE_BOOTSTRAP = (
    "import base64,hashlib,json,sys\n"
    "envelope=json.loads(sys.stdin.buffer.read())\n"
    "source=base64.b64decode(envelope['remote_controller'],validate=True)\n"
    "payload=base64.b64decode(envelope['payload'],validate=True)\n"
    "assert hashlib.sha256(source).hexdigest()==sys.argv[1]\n"
    "assert hashlib.sha256(payload).hexdigest()==sys.argv[2]\n"
    "sys.argv=['lay-v10-e1-traversal-d2-parity-remote.py',base64.b64encode(payload).decode()]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-e1-traversal-d2-parity-remote.py>'}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def payload(action: str) -> bytes:
    local_source = CONTROLLER.read_bytes()
    remote_source = REMOTE_CONTROLLER.read_bytes()
    return canonical_json_bytes(
        {
            "action": action,
            "map_audit_receipt_b64": base64.b64encode(MAP_AUDIT.read_bytes()).decode(),
            "d1_parity_receipt_b64": base64.b64encode(D1_PARITY_REFERENCE.read_bytes()).decode(),
            "local_controller_sha256": sha256_bytes(local_source),
            "local_controller_b64": base64.b64encode(local_source).decode(),
            "remote_controller_sha256": sha256_bytes(remote_source),
            "remote_controller_b64": base64.b64encode(remote_source).decode(),
        }
    )


def remote_call(action: str, *, timeout: int) -> subprocess.CompletedProcess[bytes]:
    remote_source = REMOTE_CONTROLLER.read_bytes()
    request = payload(action)
    envelope = canonical_json_bytes({"remote_controller": base64.b64encode(remote_source).decode(), "payload": base64.b64encode(request).decode()})
    command = ["/usr/bin/python3", "-c", REMOTE_BOOTSTRAP, sha256_bytes(remote_source), sha256_bytes(request)]
    return run(ssh_argv(command), input_bytes=envelope, check=False, timeout=timeout)


def parse_last_json(result: subprocess.CompletedProcess[bytes], action: str) -> dict[str, Any]:
    require(result.returncode == 0, f"remote {action} failed ({result.returncode}):\n{result.stderr.decode(errors='replace')[-8000:]}")
    lines = result.stdout.decode(errors="replace").strip().splitlines()
    require(bool(lines), f"remote {action} produced no output")
    return json.loads(lines[-1])


def local_runtime_snapshot() -> dict[str, Any]:
    launcher = pathlib.Path.home() / ".local/bin/lay"
    resolved = launcher.resolve(strict=True)
    return {"launcher": str(launcher), "resolved": str(resolved), "resolved_sha256": sha256_file(resolved)}


def verify_local_admission() -> dict[str, Any]:
    require(not LOCAL_RESULT.exists(), "local parity result already exists")
    require_file(SSH_IDENTITY, mode="0600")
    audit = require_file(MAP_AUDIT, digest=MAP_AUDIT_SHA256, size=4_363, mode="0444")
    audit_value = json.loads(MAP_AUDIT.read_text())
    require(audit_value.get("verdict") == "D2_BUCKET_MAP_AUDITED", "map audit verdict drift")
    require(audit_value.get("map", {}).get("map", {}).get("sha256") == MAP_SHA256, "map audit map identity drift")
    reference = require_file(D1_PARITY_REFERENCE, digest=D1_PARITY_SHA256, size=1_905, mode="0444")
    return {"map_audit": audit, "d1_parity_reference": reference, "controller": file_identity(CONTROLLER), "remote_controller": file_identity(REMOTE_CONTROLLER)}


def verify_command_graph() -> dict[str, Any]:
    source = REMOTE_CONTROLLER.read_text()
    compile(source, str(REMOTE_CONTROLLER), "exec")
    for forbidden in ("perf record", "perf stat", "cargo ", "rustc ", "--pid", "SIGINT"):
        require(forbidden not in source, f"forbidden route token in parity controller: {forbidden}")
    require(source.count("subprocess.run(SUBJECT_COMMAND") == 1, "parity subject call cardinality drift")
    require(source.count("os.rename(available, consumed)") == 1, "parity marker rename cardinality drift")
    return {
        "external_actions": list(EXTERNAL_ACTIONS),
        "remote_actions": ["probe-before", "parity-once", "probe-after"],
        "subject_routes": ["PARITY"],
        "subject_test": PARITY_TEST,
        "cargo_routes": [],
        "perf_routes": [],
        "pmu_routes": [],
        "marker_mutation_routes": ["parity-once"],
    }


def verify_probe(value: Mapping[str, Any], *, post: bool) -> None:
    expected = "D2_PARITY_REMOTE_POST_PROBE_PASS" if post else "D2_PARITY_REMOTE_PROBE_PASS"
    require(value.get("verdict") == expected, "remote parity probe verdict drift")
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote host drift")
    require(value.get("elf", {}).get("sha256") == ELF_SHA256, "remote parity ELF drift")
    require(value.get("map", {}).get("sha256") == MAP_SHA256, "remote parity map drift")
    markers = {row["name"]: row for row in value.get("markers", [])}
    expected_name = "parity.consumed-before-exec" if post else "parity.available"
    require(expected_name in markers and len(markers) == 11, "remote parity marker projection drift")


def self_check() -> dict[str, Any]:
    compile(CONTROLLER.read_text(), str(CONTROLLER), "exec")
    admission = verify_local_admission()
    graph = verify_command_graph()
    probe = parse_last_json(remote_call("probe-before", timeout=180), "probe-before")
    verify_probe(probe, post=False)
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-parity-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D2_PARITY_CONTROLLER_VERIFIED_UNRUN",
        "admission": admission,
        "command_graph": graph,
        "remote_probe_sha256": sha256_bytes(canonical_json_bytes(probe)),
        "marker_mutations": 0,
        "subject_executions": 0,
        "cargo_invocations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
    }


def copy_remote_evidence(destination: pathlib.Path) -> None:
    result = run(["/usr/bin/scp", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", "-q", "-p", "-r", f"{REMOTE}:{REMOTE_RESULT}", str(destination)], check=False, timeout=900)
    require(result.returncode == 0, result.stderr.decode(errors="replace")[-6000:])


def parity_once() -> dict[str, Any]:
    check = self_check()
    runtime_before = local_runtime_snapshot()
    remote_receipt = parse_last_json(remote_call("parity-once", timeout=4000), "parity-once")
    require(remote_receipt.get("verdict") == "D2_PARITY_PASS", "remote parity verdict drift")
    require(remote_receipt.get("semantic_mismatch_count") == 0, "remote parity mismatch drift")
    require(remote_receipt.get("perf_record") == remote_receipt.get("perf_stat") == 0, "perf invoked during parity")
    post = parse_last_json(remote_call("probe-after", timeout=180), "probe-after")
    verify_probe(post, post=True)
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime projection changed during parity")

    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        remote_evidence = stage / "REMOTE_EVIDENCE"
        copy_remote_evidence(remote_evidence)
        entries = verify_sha256sums(remote_evidence)
        receipt_path = remote_evidence / "D2_PARITY_RECEIPT.json"
        copied = json.loads(receipt_path.read_text())
        require(copied.get("verdict") == "D2_PARITY_PASS" and copied.get("semantic_mismatch_count") == 0, "copied parity receipt drift")
        subject = json.loads((remote_evidence / "subject/SUBJECT_RECEIPT.json").read_text())
        require(subject.get("verdict") == "PASS" and subject.get("records") == 382, "copied subject parity drift")
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "RUNTIME_BEFORE.json", runtime_before)
        write_new_json(stage / "RUNTIME_AFTER.json", runtime_after)
        write_new_bytes(stage / "local-controller.py", CONTROLLER.read_bytes())
        write_new_bytes(stage / "remote-controller.py", REMOTE_CONTROLLER.read_bytes())
        write_new_bytes(stage / "D2_BUCKET_MAP_AUDIT_RECEIPT.json", MAP_AUDIT.read_bytes())
        local_receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-local-parity.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_PARITY_PASS",
            "map_audit_receipt_sha256": MAP_AUDIT_SHA256,
            "remote_manifest_entries": entries,
            "remote_receipt_sha256": sha256_file(receipt_path),
            "subject_receipt_sha256": sha256_file(remote_evidence / "subject/SUBJECT_RECEIPT.json"),
            "semantic_mismatch_count": 0,
            "parity_marker_consumed": True,
            "remaining_available_markers": 8,
            "total_consumed_markers": 3,
            "cargo_invocations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "runtime_authority_changed": False,
            "next_action_admitted": "U-SINGLE only",
        }
        write_new_json(stage / "LOCAL_PARITY_RECEIPT.json", local_receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        fsync_directory(LOCAL_RESULT.parent)
    except BaseException:
        remove_owned_tree(stage)
        raise
    return {"verdict": "D2_PARITY_PASS", "local_result": str(LOCAL_RESULT), "remote_receipt_sha256": sha256_file(LOCAL_RESULT / "REMOTE_EVIDENCE/D2_PARITY_RECEIPT.json"), "runtime_authority_changed": False, "next_action_admitted": "U-SINGLE only"}


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=EXTERNAL_ACTIONS)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else parity_once()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2 PARITY ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
