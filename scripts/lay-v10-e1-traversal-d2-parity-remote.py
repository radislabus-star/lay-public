#!/usr/bin/env python3
"""Remote one-shot semantic parity producer for primary-only D2."""

from __future__ import annotations

import base64
import fcntl
import hashlib
import json
import os
import pathlib
import shutil
import stat
import subprocess
import sys
import time
from typing import Any, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
ELF = PARENT / "build-v1/d2-test-elf"
MAP_RESULT = PARENT / "bucket-map-v1"
RESULT = PARENT / "parity-v1"
FAILURE = PARENT / "parity-failure-v1"
PARITY_STATE = STATE / "PARITY_STATE.json"
LOCK = STATE / "route.lock"
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")

B0A = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2"
)
B0B = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1"
)
PACKAGE = B0A / "inputs/LAY-L2-RU-FULL-v13.bin"
ARTIFACTS = B0A / "inputs/slice8b-v10-f6178f/artifacts"
SIDECAR = ARTIFACTS / "LAY-L2-RU-FULL-v13.dafsa"
V7 = ARTIFACTS / "slice8b-v7-fixed-13x100.json"
SCHEDULE = B0B / "query-schedule.json"

ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
ELF_SIZE = 317_706_232
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
MAP_RECEIPT_SHA256 = "8197087fff1853b7e4c167e92ae4900eb57821c298b918bfba7dee4c477f3430"
MAP_AUDIT_SHA256 = "8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7"
D1_PARITY_SHA256 = "fbc651637522a3619bb12f35af7645cb5b02bda0fabf35b24a8fc8097b730530"
INPUTS = {
    PACKAGE: ("cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b", 140_556_462),
    SIDECAR: ("a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd", 3_689_884),
    V7: ("33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4", 1_606_189),
    SCHEDULE: ("2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78", 174_941),
}
PARITY_MARKER_SHA256 = "ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55"
PARITY_MARKER_SIZE = 479
PARITY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_semantic_parity"
SUBJECT_COMMAND = (
    str(LOADER),
    str(ELF),
    "--exact",
    PARITY_TEST,
    "--ignored",
    "--nocapture",
    "--test-threads=1",
)

MARKERS_BEFORE = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.consumed-before-exec": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.available": (PARITY_MARKER_SHA256, PARITY_MARKER_SIZE),
    "u-single.available": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.available": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.available": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.available": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.available": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
    "t-single.available": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}


class ParityError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ParityError(message)


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
    need(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            need(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new_bytes(path, json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n", mode)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        need(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            values.append({"path": path.relative_to(root).as_posix(), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)})
    return values


def write_sha256sums(root: pathlib.Path) -> None:
    values = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(root / "SHA256SUMS", "".join(f"{value['sha256']}  {value['path']}\n" for value in values).encode())


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        need(not path.is_symlink(), f"symlink before seal: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def marker_projection() -> list[dict[str, Any]]:
    values = []
    for path in sorted((STATE / "markers").iterdir()):
        values.append({**row(path), "name": path.name, "value": json.loads(path.read_text())})
    return values


def verify_markers(*, post_parity: bool) -> list[dict[str, Any]]:
    expected = dict(MARKERS_BEFORE)
    if post_parity:
        expected["parity.consumed-before-exec"] = expected.pop("parity.available")
    values = marker_projection()
    observed = {value["name"]: value for value in values}
    need(set(observed) == set(expected), f"marker membership drift: {sorted(observed)}")
    for name, (digest, size) in expected.items():
        value = observed[name]
        need(value["sha256"] == digest and value["size_bytes"] == size and value["mode"] == "0400", f"marker identity drift: {name}")
        body = value["value"]
        need(body.get("task_id") == TASK_ID and body.get("transaction_id") == TRANSACTION_ID, f"marker provenance drift: {name}")
        need(body.get("retry_permitted") is False, f"marker retry drift: {name}")
    return values


def validate_parity(value: Mapping[str, Any]) -> bool:
    zeros = (
        "terminal_mismatches", "peak_mismatches", "completeness_mismatches", "work_mismatches",
        "rank_prefix_mismatches", "terminal_rank_mismatches", "trace_authority_mismatches",
        "reverse_terminal_mismatches", "reverse_peak_mismatches", "reverse_completeness_mismatches",
        "reverse_work_mismatches", "reverse_rank_prefix_mismatches", "reverse_terminal_rank_mismatches",
        "full_row_terminal_mismatches", "full_row_peak_mismatches", "full_row_completeness_mismatches",
        "full_row_work_mismatches", "false_certificates",
    )
    checks = [value.get("schema") == "lay.v10.e1-semantic-parity.v1", value.get("test") == PARITY_TEST]
    checks.extend([value.get("records") == 382, value.get("schedule_records") == 382])
    checks.extend(value.get(name) == 0 for name in zeros)
    checks.extend(
        [
            value.get("target_form_retained") == 382,
            value.get("target_lemma_retained") == 382,
            value.get("maximum_product_states") == 35_590,
            value.get("e0_maximum_scratch_bytes") == 6_656,
            isinstance(value.get("d1_maximum_scratch_bytes"), int) and value["d1_maximum_scratch_bytes"] <= 6_656,
            value.get("e0_work") == value.get("d1_work"),
            value.get("e0_work", {}).get("expanded_states") == 8_059_788,
            value.get("e0_work", {}).get("examined_edges") == 25_145_756,
            value.get("stress", {}).get("cases") == 714_026,
            value.get("stress", {}).get("transition_mismatches") == 0,
            value.get("stress", {}).get("packed_state_mismatches") == 0,
            value.get("fixtures", {}).get("pass") is True,
            value.get("runtime_authority_changed") is False,
        ]
    )
    return value.get("verdict") == "PASS" and all(checks)


def decode_input(payload: Mapping[str, Any], key: str, digest: str) -> bytes:
    value = base64.b64decode(payload.get(key, ""), validate=True)
    need(sha256_bytes(value) == digest, f"payload SHA drift: {key}")
    return value


def verify_common(payload: Mapping[str, Any], *, post_parity: bool) -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(sha256_file(pathlib.Path("/etc/machine-id")) == MACHINE_ID_SHA256, "machine identity drift")
    audit_bytes = decode_input(payload, "map_audit_receipt_b64", MAP_AUDIT_SHA256)
    audit = json.loads(audit_bytes)
    need(audit.get("verdict") == "D2_BUCKET_MAP_AUDITED", "map-audit verdict drift")
    need(audit.get("map", {}).get("map", {}).get("sha256") == MAP_SHA256, "map-audit map SHA drift")
    reference = json.loads(decode_input(payload, "d1_parity_receipt_b64", D1_PARITY_SHA256))
    need(validate_parity(reference), "parity validator rejected sealed D1 reference")
    need(row(ELF)["sha256"] == ELF_SHA256 and ELF.stat().st_size == ELF_SIZE and mode_string(ELF) == "0555", "D2 ELF drift")
    need(row(MAP_RESULT / "D2_BUCKET_MAP.json")["sha256"] == MAP_SHA256, "remote map drift")
    need(row(MAP_RESULT / "D2_BUCKET_MAP_RECEIPT.json")["sha256"] == MAP_RECEIPT_SHA256, "remote map receipt drift")
    map_state = json.loads((STATE / "BUCKET_MAP_STATE.json").read_text())
    need(map_state.get("state") == "D2_BUCKET_MAP_SEALED" and map_state.get("receipt_sha256") == MAP_RECEIPT_SHA256, "remote map state drift")
    for path, (digest, size) in INPUTS.items():
        value = row(path)
        need(value["sha256"] == digest and value["size_bytes"] == size and value["mode"] == "0444", f"input drift: {path}")
    need(LOADER.is_file(), "ELF loader missing")
    if post_parity:
        need(RESULT.is_dir() and not FAILURE.exists() and PARITY_STATE.is_file(), "parity result projection drift")
    else:
        need(not RESULT.exists() and not FAILURE.exists() and not PARITY_STATE.exists(), "parity terminal evidence already exists")
    markers = verify_markers(post_parity=post_parity)
    return {
        "hostname": os.uname().nodename,
        "machine_id_sha256": MACHINE_ID_SHA256,
        "elf": row(ELF),
        "map": row(MAP_RESULT / "D2_BUCKET_MAP.json"),
        "map_audit_receipt_sha256": MAP_AUDIT_SHA256,
        "markers": markers,
        "parent_entries": sorted(path.name for path in PARENT.iterdir()),
        "state_entries": sorted(path.name for path in STATE.iterdir()),
        "remote_writes": 0,
    }


def controlled_environment(output: pathlib.Path) -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "PATH": "/home/e/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "Europe/Tallinn",
        "LAY_V10_D1_PACKAGE": str(PACKAGE),
        "LAY_V10_D1_SIDECAR": str(SIDECAR),
        "LAY_V10_D1_V7": str(V7),
        "LAY_V10_D1_SCHEDULE": str(SCHEDULE),
        "LAY_V10_D1_OUTPUT": str(output),
        "LAY_V10_D1_RUN_ID": "PARITY",
        "LAY_V10_D1_CPUS": "0",
    }


def consume_marker() -> dict[str, Any]:
    available = STATE / "markers/parity.available"
    consumed = STATE / "markers/parity.consumed-before-exec"
    before = row(available)
    need(before["sha256"] == PARITY_MARKER_SHA256 and before["size_bytes"] == PARITY_MARKER_SIZE, "parity marker drift")
    need(not consumed.exists(), "parity marker already consumed")
    os.rename(available, consumed)
    fsync_directory(available.parent)
    after = row(consumed)
    need(after["sha256"] == before["sha256"] and after["size_bytes"] == before["size_bytes"], "parity marker rename drift")
    return {"before": before, "after": after, "consumed_before_subject": True}


def publish_state(verdict: str, receipt_sha256: str | None) -> None:
    write_new_json(
        PARITY_STATE,
        {
            "schema": "lay.v10.e1-traversal-d2-primary-only-parity-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "state": verdict,
            "parity_marker_consumed": True,
            "receipt_sha256": receipt_sha256,
            "retry_permitted": False,
        },
        0o400,
    )
    fsync_directory(STATE)


def parity_once(payload: Mapping[str, Any]) -> dict[str, Any]:
    admission = verify_common(payload, post_parity=False)
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    marker_consumed = False
    try:
        inputs = stage / "inputs"
        subject = stage / "subject"
        inputs.mkdir(mode=0o700)
        subject.mkdir(mode=0o700)
        local_controller = decode_input(payload, "local_controller_b64", payload["local_controller_sha256"])
        remote_controller = decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"])
        write_new_bytes(inputs / "local-controller.py", local_controller)
        write_new_bytes(inputs / "remote-controller.py", remote_controller)
        write_new_bytes(inputs / "D2_BUCKET_MAP_AUDIT_RECEIPT.json", decode_input(payload, "map_audit_receipt_b64", MAP_AUDIT_SHA256))
        write_new_bytes(inputs / "D1_PARITY_REFERENCE.json", decode_input(payload, "d1_parity_receipt_b64", D1_PARITY_SHA256))
        environment = controlled_environment(subject)
        write_new_json(
            stage / "PREPARITY.json",
            {
                "schema": "lay.v10.e1-traversal-d2-primary-only-preparity.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "command": SUBJECT_COMMAND,
                "environment": environment,
                "admission": admission,
                "marker_consumed": False,
                "retry_permitted": False,
            },
        )
        fsync_directory(stage)
        marker = consume_marker()
        marker_consumed = True
        write_new_json(stage / "MARKER_CONSUMPTION.json", marker)
        started_ns = time.perf_counter_ns()
        process = subprocess.run(SUBJECT_COMMAND, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=3600)
        ended_ns = time.perf_counter_ns()
        write_new_bytes(stage / "stdout.log", process.stdout)
        write_new_bytes(stage / "stderr.log", process.stderr)
        receipt_path = subject / "SUBJECT_RECEIPT.json"
        need(receipt_path.is_file(), "D2 parity subject receipt missing")
        subject_receipt = json.loads(receipt_path.read_text())
        passed = process.returncode == 0 and validate_parity(subject_receipt)
        need(passed, f"D2 semantic parity failed with exit {process.returncode}")
        wrapper = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-parity-wrapper.v1",
            "verdict": "PASS",
            "command": list(SUBJECT_COMMAND),
            "environment": environment,
            "exit_code": process.returncode,
            "process_wall_ns_diagnostic": ended_ns - started_ns,
            "subject_receipt": subject_receipt,
            "subject_receipt_sha256": sha256_file(receipt_path),
            "marker": marker,
            "cargo_invocations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "runtime_authority_changed": False,
        }
        write_new_json(stage / "PARITY_WRAPPER.json", wrapper)
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-parity-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_PARITY_PASS",
            "map_audit_receipt_sha256": MAP_AUDIT_SHA256,
            "elf": row(ELF),
            "map": row(MAP_RESULT / "D2_BUCKET_MAP.json"),
            "subject_receipt": row(receipt_path),
            "semantic_mismatch_count": 0,
            "marker": marker,
            "remaining_available_markers": 8,
            "total_consumed_markers": 3,
            "cargo_invocations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "U-SINGLE only",
        }
        write_new_json(stage / "D2_PARITY_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, RESULT)
        fsync_directory(PARENT)
        published = RESULT / "D2_PARITY_RECEIPT.json"
        publish_state("D2_PARITY_PASS", sha256_file(published))
        return {**receipt, "published_receipt_sha256": sha256_file(published), "remote_result": str(RESULT)}
    except BaseException as error:
        if marker_consumed:
            try:
                for path in [stage, *stage.rglob("*")]:
                    path.chmod(0o700 if path.is_dir() else 0o600)
                checksum = stage / "SHA256SUMS"
                if checksum.exists():
                    checksum.unlink()
                write_new_json(stage / "FAILURE.json", {"verdict": "BLOCKED_SEMANTIC", "error": f"{type(error).__name__}: {error}", "marker_consumed": True, "retry_permitted": False})
                write_sha256sums(stage)
                seal_tree(stage)
                os.rename(stage, FAILURE)
                fsync_directory(PARENT)
                publish_state("BLOCKED_SEMANTIC", None)
            except BaseException:
                pass
        elif stage.exists():
            shutil.rmtree(stage)
        raise


def main() -> int:
    try:
        need(len(sys.argv) == 2, "expected one base64 payload")
        payload = json.loads(base64.b64decode(sys.argv[1], validate=True))
        action = payload.get("action")
        if action == "probe-before":
            value = {**verify_common(payload, post_parity=False), "verdict": "D2_PARITY_REMOTE_PROBE_PASS"}
        elif action == "probe-after":
            value = {**verify_common(payload, post_parity=True), "verdict": "D2_PARITY_REMOTE_POST_PROBE_PASS", "receipt": row(RESULT / "D2_PARITY_RECEIPT.json")}
        elif action == "parity-once":
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = parity_once(payload)
        else:
            raise ParityError(f"unsupported action: {action!r}")
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        return 0
    except Exception as error:
        print(f"D2 PARITY REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
