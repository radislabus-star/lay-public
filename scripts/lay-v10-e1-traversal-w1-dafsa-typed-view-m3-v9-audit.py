#!/usr/bin/env python3
"""Independent read-only auditor for the M3 V9 materialization trace."""

from __future__ import annotations

import argparse
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
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-20260827"
TRANSACTION_ID = "ed21c54906eebc5a9a99afc873b3a38b8a6ca5e6003b539d019539403aa2ffb1"
V8R1_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827"
V8R1_TRANSACTION_ID = "7d6455e678c244be3c31dc52c2b64d55f34d0a91338afa1219acf06ff327ffb9"
V8R2_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r2-direct-exec-20260827"
V8R2_TRANSACTION_ID = "59694b7b9f0327d78896b5bc4797671f54478674186558e338e4a1b0d9ef7813"
V8R3_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r3-terminal-projection-20260827"
V8R3_TRANSACTION_ID = "a33732116bdf8a1ccf1216e99958750ad33b0ccc6a3c7bbd4226454b20bfd66f"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v9-{TRANSACTION_ID}"
V8R3_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / V8R3_TASK_ID
V8R3_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / V8R3_TASK_ID
REMOTE_EXECUTABLE = V8R3_PARENT / "bootstrap-v1/m3-v8r3-test-elf"
V8R3_WRAPPER = V8R3_PARENT / "e2e-v1/E2E_WRAPPER.json"
V8R1_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / V8R1_TASK_ID
V8R1_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / V8R1_TASK_ID
V8R1_ELF = V8R1_PARENT / "build-v1/m3-v8r1-test-elf"
V8R1_INPUTS = V8R1_PARENT / "bootstrap-v1/inputs"
V8R1_WRAPPER = V8R1_PARENT / "e2e-v1/E2E_WRAPPER.json"
V8R2_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / V8R2_TASK_ID
V8R2_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / V8R2_TASK_ID
V8R2_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v8r2-{V8R2_TRANSACTION_ID}"

LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v9.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v9-remote.py"
AUDITOR = pathlib.Path(__file__).resolve()
V8_SOURCE = ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
TRACE_SOURCE = ROOT / "src/nanda_wave/l2_field/productive_v1/live.rs"
PROPOSAL_SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"
LATENCY_DECISION = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_LATENCY_FAILURE_DECISION_V1_2026-08-27.md"
V9_PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_2026-08-27.md"
V9_ROUTE = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_ROUTE_V2_2026-08-27.md"
V9_ROUTE_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_ROUTE_V2_RECEIPT_2026-08-27.json"
V9_PREFLIGHT_MANIFEST = ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_IMPLEMENTATION_V1_2026-08-27.json"
V9_PREFLIGHT_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_IMPLEMENTATION_V1_PREFLIGHT_2026-08-27.json"
V8R3_TERMINAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"
V8R3_SUBJECT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_TERMINAL_AUDIT_V1_2026-08-27/REMOTE_E2E/subject/SUBJECT_RECEIPT.json"
V8R3_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
V8R1_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_PSS_HELPER_LIFECYCLE_DIAGNOSIS_2026-08-27.json"
V8R1_BUILD_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_BUILD_AUDIT_V1_2026-08-27/BUILD_AUDIT.json"
V8R1_TERMINAL_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"
V8R1_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
V8R1_CONTROLLER = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_REMOTE_CONTROLLER_V8R1_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V8R1_LOCAL_ELF = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_BUILD_AUDIT_V1_2026-08-27/REMOTE_BUILD/m3-v8r1-test-elf"
V8R2_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R2_LIVE_ADMISSION_FAILURE_DIAGNOSIS_2026-08-27.json"
V8R2_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R2_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
V8R2_CONTROLLER = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_REMOTE_CONTROLLER_V8R2_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V8R2_LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r2.py"
V8R2_REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r2-remote.py"
V8R2_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r2-audit.py"
V8_SOURCE_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_IMPLEMENTATION_V8_2026-08-27/IMPLEMENTATION_RECEIPT.json"
CONTROLLER_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_CONTROLLER_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"

RECEIPTS = ROOT / "docs/structural_gates/receipts"
ADMISSION_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_EXECUTION_ADMISSION_V1_2026-08-27"
BOOTSTRAP_AUDIT_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_BOOTSTRAP_AUDIT_V1_2026-08-27"
QUIET_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_QUIET_ADMISSION_V1_2026-08-27"
TERMINAL_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_TERMINAL_AUDIT_V1_2026-08-27"
ADMISSION_RECEIPT = ADMISSION_ROOT / "EXECUTION_ADMISSION.json"
BOOTSTRAP_AUDIT_RECEIPT = BOOTSTRAP_AUDIT_ROOT / "BOOTSTRAP_AUDIT.json"
QUIET_RECEIPT = QUIET_ROOT / "QUIET_ADMISSION.json"
TERMINAL_RECEIPT = TERMINAL_ROOT / "TERMINAL_AUDIT.json"

ACTIONS = ("self-check", "live-admission", "bootstrap", "quiet", "terminal", "status")
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)
HOSTNAME = "e-MEGA-MINI-M1-13th"
KERNEL = "6.8.0-124-generic"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
EXPECTED_ELF_SIZE = 320_613_368
EXPECTED_ELF_SHA256 = "0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea"
EXPECTED_BUILD_ID = "c6ddac7181428a303cbc51be61dd3bb115677562"
EXPECTED_V8R1_WRAPPER_SHA256 = "1edc2c195b67485d007c1ed9354db14cf6f9907eaa1ffc3d77fa5f07b13f291b"
EXPECTED_SOURCE_SHA = "28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2"
EXPECTED_V13_SHA = "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"
EXPECTED_V7_SHA = "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"
EXPECTED_PRODUCTIVE_SHA = "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"
EXPECTED_L11_SHA = "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"
EXPECTED_V8R3_TERMINAL_SHA = "2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc"
EXPECTED_V8R3_SUBJECT_SHA = "65cd8a6f08d77c192ae0eb24fa3df106ee5030e7a8bbdfdf44d08429f7d9bfd5"
EXPECTED_V8R3_WRAPPER_SHA = "e2aaefc1490df86cb1abae176035efab22b290cc72afe23fefeeb52e96d42ece"
EXPECTED_V8R3_JOURNAL_SHA = "1c176cb2b9e986011fba901996260ad34bf59d71d864ae663045800a8bf9cdfc"
EXPECTED_LATENCY_DECISION_SHA = "45e2e279997f7a93072bcfd74ad11d2566f55b442685d1be2a75e905dd543a8a"
EXPECTED_V9_PAPER_SHA = "98000de5d6a502d4bf1b2005deca476bfdd12539f02528a4b450f240f3d9ed27"
EXPECTED_V9_ROUTE_SHA = "7270e79ccd5c64f33dc5d1a3f95ff4dbea9a8f13baebc81aa97254a80d7fe860"
EXPECTED_V9_ROUTE_RECEIPT_SHA = "84171ed96027ae674e50618398af2d881f44b83fd7265a4b5e5cd92d8b9be00f"
EXPECTED_V9_PREFLIGHT_MANIFEST_SHA = "efdbd7adc388492656d529ed28e1854b400d4be9ddf205cdb577624b91963082"
EXPECTED_V9_PREFLIGHT_RECEIPT_SHA = "43e5f2e849a7ccdeab2fbc371f12fc0950e11e4c3b5c8fef043d01f464223587"
EXPECTED_V9_NANDA_MANIFEST_SHA = "2a34631475e65f7b75f165b8f2224c36a490499848414011a38b75c242a067ce"
EXPECTED_TRACE_SOURCE_SHA = "87180990b6883641483a46886074e5350f35e351454d734f0c3c9da723d758bd"
EXPECTED_PROPOSAL_SOURCE_SHA = "dd4a37a8c0430c9ff145f9ae9cbbbc735164ece833a143a19af644ac7ad835ca"
EXPECTED_TRACE_ROWS = 1_910
WARMUP_ROWS = 382
MEASURED_ROWS = 1_528
TAIL_ROWS = 16
TRACE_PREFIX = "productive_v90_materialization_trace "
TRACE_PATTERN = __import__("re").compile(
    r"^productive_v90_materialization_trace surfaces=(\d+) emitted=(\d+) setup_us=(\d+) "
    r"projection_us=(\d+) classify_us=(\d+) gate_us=(\d+) evidence_us=(\d+)$"
)
TRACE_STAGE_NAMES = ("setup_us", "projection_us", "classify_us", "gate_us", "evidence_us")
EXPECTED_INPUTS = {
    "LAY-L2-RU-FULL-v13.bin": (140_556_462, EXPECTED_V13_SHA),
    "slice8b-v7-fixed-13x100.json": (1_606_189, EXPECTED_V7_SHA),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m": (17_309_944, EXPECTED_PRODUCTIVE_SHA),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r": (2_123_112, "de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e"),
    "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin": (77_962_328, EXPECTED_L11_SHA),
    "l11-proof.json": (539_536, "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d"),
}
SEMANTIC_FIELDS = (
    "candidate_mismatches",
    "certificate_mismatches",
    "structured_certificate_mismatches",
    "schedule_mismatches",
    "completeness_mismatches",
    "lattice_marker_mismatches",
    "emitted_surface_mismatches",
    "gate_mismatches",
    "certificate_collisions",
    "semantic_total",
)


class V9AuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V9AuditError(message)


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


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"manifest absent: {root}")
    rows = manifest.read_text().splitlines()
    expected = set()
    for row in rows:
        digest, relative = row.split("  ", 1)
        path = root / relative
        need(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {relative}")
        expected.add(relative)
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "SHA256SUMS"
    }
    need(actual == expected, f"manifest inventory drift: {root}")
    return len(rows)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish_tree(
    destination: pathlib.Path,
    receipt_name: str,
    receipt: Mapping[str, Any],
    copied: Mapping[str, pathlib.Path] | None = None,
) -> dict[str, Any]:
    need(not destination.exists(), f"audit receipt already exists: {destination}")
    stage = destination.with_name(f"{destination.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / receipt_name, canonical(receipt))
        for name, source in (copied or {}).items():
            shutil.copytree(source, stage / name)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, destination)
        fsync_dir(destination.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return load_json(destination / receipt_name)


def run(
    argv: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: float = 3_600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        raise V9AuditError(
            f"command failed ({result.returncode}): {list(argv)!r}\n"
            f"stdout:\n{result.stdout[-4000:].decode(errors='replace')}\n"
            f"stderr:\n{result.stderr[-4000:].decode(errors='replace')}"
        )
    return result


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout
    match = __import__("re").search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1).decode()


def ssh_python(
    program: str,
    arguments: Sequence[str] = (),
    *,
    root: bool = False,
    timeout: float = 3_600,
) -> dict[str, Any]:
    remote_command = ["/usr/bin/python3", "-", *arguments]
    if root:
        remote_command = ["/usr/bin/sudo", "-n", *remote_command]
    result = run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            *remote_command,
        ],
        input_bytes=program.encode(),
        timeout=timeout,
    )
    lines = result.stdout.decode().strip().splitlines()
    need(lines, "remote operation returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote operation is not an object")
    return value


def copy_remote(remote: pathlib.PurePosixPath, destination: pathlib.Path) -> None:
    destination.mkdir(parents=True, mode=0o700)
    run(
        [
            "/usr/bin/scp",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-q",
            "-p",
            "-r",
            f"{REMOTE}:{remote}/.",
            str(destination),
        ],
        timeout=3_600,
    )


def fixed_local() -> dict[str, Any]:
    expected = {
        LATENCY_DECISION: EXPECTED_LATENCY_DECISION_SHA,
        V9_PAPER: EXPECTED_V9_PAPER_SHA,
        V9_ROUTE: EXPECTED_V9_ROUTE_SHA,
        V9_ROUTE_RECEIPT: EXPECTED_V9_ROUTE_RECEIPT_SHA,
        V9_PREFLIGHT_MANIFEST: EXPECTED_V9_PREFLIGHT_MANIFEST_SHA,
        V9_PREFLIGHT_RECEIPT: EXPECTED_V9_PREFLIGHT_RECEIPT_SHA,
        V8R3_TERMINAL: EXPECTED_V8R3_TERMINAL_SHA,
        V8R3_SUBJECT: EXPECTED_V8R3_SUBJECT_SHA,
        V8R3_JOURNAL: EXPECTED_V8R3_JOURNAL_SHA,
        V8R2_DIAGNOSIS: "a74b51570143f56a8186435acfac80ab4dac5ec4140f105ac82df960508ed3dd",
        V8R2_JOURNAL: "298ebe81c77f48edca3d9df1fd514b14d6bcd7abf15f41ac42752cf27029d785",
        V8R2_CONTROLLER: "0c8e792d88e532b5cfb18c65a4ba15c031cc0956e6c9798a0b4de8c30b450b34",
        V8R2_LOCAL_CONTROLLER: "7a77aaa1a1b42e77e3c654e12953c6f02e70ecee71046a50befe6ffaea7a446a",
        V8R2_REMOTE_CONTROLLER: "f40183fb405a0ab52ac86dc1493fc38ae4ca14aef0830e46188b49cb4e21081a",
        V8R2_AUDITOR: "72b9071d75bd0cfb15545c1820dae453ff961d207f7cd346c17ae1881f61592f",
        V8R1_DIAGNOSIS: "9b05af87d83c937dcc1e4eab0e398ab3d93ef49ac3e0bfb8089a58ba3d64bae0",
        V8R1_BUILD_AUDIT: "d7d5e7110171e5c6546016ff0c9374c323804014ef8cfa7a690ad7d1d11c8340",
        V8R1_TERMINAL_AUDIT: "04d0e17158a63a49088e8c8ff9dc25df67e50ac6a97b770ea3fcf1a73d67ec91",
        V8R1_JOURNAL: "c6c9d648bdbc02ee8f10639099fc4add6141488600f771188e2b6f69445d91a4",
        V8R1_CONTROLLER: "4c16d6c34409f8395354796c48d4364319bbc52c6dbbb309600b3443bdd2f99c",
        V8R1_LOCAL_ELF: EXPECTED_ELF_SHA256,
        V8_SOURCE: EXPECTED_SOURCE_SHA,
        TRACE_SOURCE: EXPECTED_TRACE_SOURCE_SHA,
        PROPOSAL_SOURCE: EXPECTED_PROPOSAL_SOURCE_SHA,
        V8_SOURCE_IMPLEMENTATION: "cf4b38d81cc7f9ea9125855194635fafde9c76c9d022f7185635e8bb6c2f29e5",
    }
    rows = {}
    for path, digest in expected.items():
        row = file_row(path)
        need(row["sha256"] == digest, f"fixed local identity drift: {path}")
        rows[str(path.relative_to(ROOT))] = row
    need(file_row(V8R1_LOCAL_ELF)["mode"] == "0444", "local sealed ELF mode drift")
    need(elf_build_id(V8R1_LOCAL_ELF) == EXPECTED_BUILD_ID, "local sealed ELF Build ID drift")
    for path in (V8R2_LOCAL_CONTROLLER, V8R2_REMOTE_CONTROLLER, V8R2_AUDITOR):
        need(mode_string(path) == "0555", f"sealed V8R2 controller mode drift: {path}")
    for path in (
        LATENCY_DECISION,
        V9_PAPER,
        V9_ROUTE,
        V9_ROUTE_RECEIPT,
        V9_PREFLIGHT_MANIFEST,
        V9_PREFLIGHT_RECEIPT,
        V8R3_TERMINAL,
        V8R3_SUBJECT,
        V8R3_JOURNAL,
        V8R2_DIAGNOSIS,
        V8R2_JOURNAL,
        V8R2_CONTROLLER,
    ):
        need(mode_string(path) == "0444", f"sealed predecessor mode drift: {path}")
    validate_local_predecessor_pair()
    preflight = load_json(V9_PREFLIGHT_RECEIPT)
    need(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True
        and preflight.get("manifest_sha256") == EXPECTED_V9_NANDA_MANIFEST_SHA,
        "effective V9 preflight drift",
    )
    need(CONTROLLER_IMPLEMENTATION.is_file(), "controller implementation receipt absent")
    implementation = load_json(CONTROLLER_IMPLEMENTATION)
    need(implementation.get("verdict") == "M3_V9_CONTROLLERS_VERIFIED_UNRUN", "controller implementation verdict drift")
    for key, path in {
        "local_controller_sha256": LOCAL_CONTROLLER,
        "remote_controller_sha256": REMOTE_CONTROLLER,
        "auditor_sha256": AUDITOR,
    }.items():
        need(implementation.get(key) == sha256_file(path), f"controller implementation binding drift: {key}")
    rows["controller_implementation"] = file_row(CONTROLLER_IMPLEMENTATION)
    return {"files": rows, "implementation": implementation}


def local_runtime_snapshot() -> dict[str, Any]:
    rows = {}
    for name in ("lay", "lay-daemon", "lay-ibus-engine"):
        path = pathlib.Path("/home/ubu/.local/bin") / name
        if path.exists():
            target = path.resolve()
            rows[name] = {
                "link": str(path),
                "target": str(target),
                "sha256": sha256_file(target),
                "size_bytes": target.stat().st_size,
            }
    return rows


REMOTE_SNAPSHOT = r'''
import hashlib,json,os,pathlib,stat,sys,time
task=sys.argv[1]; parent=pathlib.Path(sys.argv[2]); state=pathlib.Path(sys.argv[3]); cache=pathlib.Path(sys.argv[4])
v8r3_parent=pathlib.Path(sys.argv[5]); v8r3_state=pathlib.Path(sys.argv[6]); v8r3_elf=pathlib.Path(sys.argv[7]); v8r3_wrapper=pathlib.Path(sys.argv[8])
v8r1_inputs=pathlib.Path(sys.argv[9]); v8r2_parent=pathlib.Path(sys.argv[10]); v8r2_state=pathlib.Path(sys.argv[11]); v8r2_cache=pathlib.Path(sys.argv[12])
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1048576),b''): h.update(b)
 return h.hexdigest()
def row(path): return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}','size_bytes':path.stat().st_size,'sha256':sha(path)}
def tree(root): return [row(p) for p in sorted(root.rglob('*')) if p.is_file()]
def latest(root):
 rows=sorted(root.glob('STATE-*.json')) if root.is_dir() else []
 return json.loads(rows[-1].read_text()) if rows else None
def markers(root):
 p=root/'markers'
 return {'available':sorted(x.name for x in p.glob('*.available')) if p.is_dir() else [],'consumed':sorted(x.name for x in p.glob('*.consumed-before-exec')) if p.is_dir() else []}
def runtime():
 roots=[pathlib.Path('/home/e/.local/share/lay/nanda_wave/l2'),pathlib.Path('/home/e/.local/share/lay/nanda_wave/l1.1')]
 out=[]
 for root in roots:
  if root.is_dir():
   for p in sorted(root.iterdir()):
    if p.is_file() and (p.name.startswith('active') or p.suffix in {'.p2m','.p2r'}): out.append(row(p))
 return out
conflicts=[]
for p in pathlib.Path('/proc').iterdir():
 if not p.name.isdigit(): continue
 try: raw=(p/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace').strip()
 except (FileNotFoundError,PermissionError,ProcessLookupError): continue
 if raw and any(x in raw for x in ('perf record','perf stat','cargo test','rustc ', 'm3_end_to_end_physical_proof')):
  conflicts.append({'pid':int(p.name),'command':raw})
input_names=['LAY-L2-RU-FULL-v13.bin','slice8b-v7-fixed-13x100.json','LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m','LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r','LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin','l11-proof.json']
out={
 'hostname':os.uname().nodename,'kernel':os.uname().release,'machine_id_sha256':sha('/etc/machine-id'),
 'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),
 'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),
 'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),
 'paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},
 'parent_mode':f'{stat.S_IMODE(parent.stat().st_mode):04o}' if parent.exists() else None,
 'parent_uid':parent.stat().st_uid if parent.exists() else None,'parent_gid':parent.stat().st_gid if parent.exists() else None,
 'state_mode':f'{stat.S_IMODE(state.stat().st_mode):04o}' if state.exists() else None,
 'parent_tree':tree(parent) if parent.exists() else [],'state_tree':tree(state) if state.exists() else [],
 'latest_state':latest(state),'markers':markers(state),
 'v8r3':{'parent':v8r3_parent.exists(),'state':v8r3_state.exists(),'elf':row(v8r3_elf) if v8r3_elf.is_file() else None,'wrapper':row(v8r3_wrapper) if v8r3_wrapper.is_file() else None,'latest_state':latest(v8r3_state),'markers':markers(v8r3_state)},
 'v8r1_inputs':{name:row(v8r1_inputs/name) for name in input_names if (v8r1_inputs/name).is_file()},
 'v8r2_paths':{'parent':v8r2_parent.exists(),'state':v8r2_state.exists(),'cache':v8r2_cache.exists()},
 'conflicting_processes':conflicts,'loadavg':pathlib.Path('/proc/loadavg').read_text().strip(),
 'thermal':{str(p):int(p.read_text().strip()) for p in pathlib.Path('/sys/devices/system/cpu').glob('cpu*/thermal_throttle/*') if p.read_text().strip().isdigit()},
 'runtime_projection':runtime(),'free_bytes':os.statvfs('/home/e').f_bavail*os.statvfs('/home/e').f_frsize,'monotonic_ns':time.monotonic_ns(),
}
print(json.dumps(out,sort_keys=True))
'''


def remote_snapshot() -> dict[str, Any]:
    return ssh_python(
        REMOTE_SNAPSHOT,
        [
            TASK_ID,
            str(REMOTE_PARENT),
            str(REMOTE_STATE),
            str(REMOTE_CACHE),
            str(V8R3_PARENT),
            str(V8R3_STATE),
            str(REMOTE_EXECUTABLE),
            str(V8R3_WRAPPER),
            str(V8R1_INPUTS),
            str(V8R2_PARENT),
            str(V8R2_STATE),
            str(V8R2_CACHE),
        ],
        root=True,
        timeout=3_600,
    )


def validate_host(value: Mapping[str, Any]) -> None:
    need(value.get("hostname") == HOSTNAME, "remote hostname drift")
    need(value.get("kernel") == KERNEL, "remote kernel drift")
    need(value.get("machine_id_sha256") == MACHINE_ID_SHA256, "remote machine-id drift")
    need((value.get("online"), value.get("core"), value.get("atom")) == ("0-19", "0-11", "12-19"), "remote topology drift")


def validate_v8r2_absent(value: Mapping[str, Any]) -> None:
    need(value.get("v8r2_paths") == {"parent": False, "state": False, "cache": False}, "V8R2 remote namespace must remain absent")


def validate_local_predecessor_pair() -> dict[str, Any]:
    terminal = load_json(V8R3_TERMINAL)
    subject = load_json(V8R3_SUBJECT)
    producer = terminal.get("live_projection", {}).get("latest_state", {})
    need(
        terminal.get("task_id") == V8R3_TASK_ID
        and terminal.get("transaction_id") == V8R3_TRANSACTION_ID
        and terminal.get("verdict") == "BLOCKED_LATENCY"
        and terminal.get("e2e_wrapper_sha256") == EXPECTED_V8R3_WRAPPER_SHA
        and producer.get("state") == "E2E_CREATED_UNAUDITED"
        and producer.get("e2e_wrapper_sha256") == EXPECTED_V8R3_WRAPPER_SHA,
        "local V8R3 producer/terminal pair drift",
    )
    need(
        subject.get("schema") == "lay.m3-end-to-end-test-owner.v1"
        and subject.get("verdict") == "BLOCKED_LATENCY"
        and subject.get("gates") == {
            "capacity": True,
            "environment": True,
            "latency": False,
            "reload_identity": True,
            "rss": True,
            "semantic": True,
        },
        "local V8R3 subject projection drift",
    )
    return {
        "remote_producer_state": producer["state"],
        "independent_terminal_verdict": terminal["verdict"],
        "subject_verdict": subject["verdict"],
        "e2e_wrapper_sha256": EXPECTED_V8R3_WRAPPER_SHA,
    }


def validate_v8r3(value: Mapping[str, Any]) -> None:
    old = value.get("v8r3", {})
    need(old.get("parent") is True and old.get("state") is True, "V8R3 namespace absent")
    elf = old.get("elf", {})
    need(elf.get("mode") == "0555" and elf.get("size_bytes") == EXPECTED_ELF_SIZE and elf.get("sha256") == EXPECTED_ELF_SHA256, "V8R3 ELF drift")
    producer = old.get("latest_state", {})
    need(
        producer.get("task_id") == V8R3_TASK_ID
        and producer.get("transaction_id") == V8R3_TRANSACTION_ID
        and producer.get("state") == "E2E_CREATED_UNAUDITED"
        and producer.get("e2e_wrapper_sha256") == EXPECTED_V8R3_WRAPPER_SHA,
        "V8R3 producer state drift",
    )
    wrapper = old.get("wrapper", {})
    need(wrapper.get("mode") == "0444" and wrapper.get("sha256") == EXPECTED_V8R3_WRAPPER_SHA, "V8R3 remote wrapper drift")
    need(old.get("markers") == {"available": [], "consumed": ["e2e.consumed-before-exec"]}, "V8R3 marker history drift")
    inputs = value.get("v8r1_inputs", {})
    for name, (size, digest) in EXPECTED_INPUTS.items():
        row = inputs.get(name, {})
        need(row.get("mode") == "0444" and row.get("size_bytes") == size and row.get("sha256") == digest, f"V8R1 input drift: {name}")
    validate_v8r2_absent(value)


UID_PROBE = r'''
import json,os,pathlib,sys
p=pathlib.Path(sys.argv[1]); p.mkdir(mode=0o700)
try:
 a=p/'a'; b=p/'b'; f=open(a,'xb'); f.write(b'm3-v9-admission\n'); f.flush(); os.fsync(f.fileno()); f.close(); os.rename(a,b)
 assert b.read_bytes()==b'm3-v9-admission\n'; b.unlink(); p.rmdir()
 print(json.dumps({'verdict':'PASS','operations':['create','write','fsync','rename','read','unlink'],'probe_absent_after':not p.exists()},sort_keys=True))
except BaseException:
 try:
  for x in p.iterdir(): x.unlink()
  p.rmdir()
 except BaseException: pass
 raise
'''


def live_admission() -> dict[str, Any]:
    local = fixed_local()
    before = remote_snapshot()
    validate_host(before)
    validate_v8r3(before)
    need(before["paths"] == {"parent": False, "state": False, "cache": False}, "V9 remote namespace is not absent")
    need(not before.get("conflicting_processes"), "conflicting remote performance process is active")
    need(int(before.get("free_bytes", 0)) >= 5 * 1024**3, "remote free-space gate failed")
    probe_path = f"/home/e/.cache/lay-m3-v9-admission-{TRANSACTION_ID}"
    probe = ssh_python(UID_PROBE, [probe_path], root=False)
    need(probe.get("verdict") == "PASS" and probe.get("probe_absent_after") is True, "UID e capability probe failed")
    time.sleep(2)
    after = remote_snapshot()
    validate_host(after)
    validate_v8r3(after)
    need(after["paths"] == before["paths"], "remote namespace changed during admission")
    need(not after.get("conflicting_processes"), "remote host stopped being quiet during admission")
    receipt = {
        "schema": "lay.m3-v9-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_EXECUTION_ADMITTED",
        "safe_to_execute": True,
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "implementation_receipt_sha256": sha256_file(CONTROLLER_IMPLEMENTATION),
        "v8r3_terminal_audit_sha256": sha256_file(V8R3_TERMINAL),
        "v8r3_subject_receipt_sha256": sha256_file(V8R3_SUBJECT),
        "v8r3_journal_sha256sums_sha256": sha256_file(V8R3_JOURNAL),
        "host_before": before,
        "host_after": after,
        "uid_capability": probe,
        "local_runtime_before": local_runtime_snapshot(),
        "remote_runtime_before": before["runtime_projection"],
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "fixed_local": local,
    }
    return publish_tree(ADMISSION_ROOT, "EXECUTION_ADMISSION.json", receipt)


def tree_index(value: Mapping[str, Any], key: str) -> dict[str, Mapping[str, Any]]:
    return {str(row["path"]): row for row in value.get(key, [])}


def remote_text(path: pathlib.PurePosixPath) -> str:
    return str(ssh_python(
        "import json,pathlib,sys; print(json.dumps({'text':pathlib.Path(sys.argv[1]).read_text()},sort_keys=True))",
        [str(path)],
        root=True,
    )["text"])


def validate_remote_manifest(snapshot: Mapping[str, Any], root: pathlib.PurePosixPath) -> int:
    index = tree_index(snapshot, "parent_tree")
    manifest_path = str(root / "SHA256SUMS")
    need(manifest_path in index, f"remote manifest absent: {manifest_path}")
    rows = remote_text(root / "SHA256SUMS").splitlines()
    for row in rows:
        digest, relative = row.split("  ", 1)
        path = str(root / relative)
        need(path in index and index[path]["sha256"] == digest, f"remote manifest mismatch: {relative}")
    actual = {
        path.removeprefix(str(root) + "/")
        for path in index
        if path != manifest_path and path.startswith(str(root) + "/")
    }
    need(actual == {row.split("  ", 1)[1] for row in rows}, "remote manifest inventory drift")
    return len(rows)


def bootstrap_audit() -> dict[str, Any]:
    fixed_local()
    admission = load_json(ADMISSION_RECEIPT)
    need(admission.get("verdict") == "M3_V9_EXECUTION_ADMITTED", "execution admission absent")
    snapshot = remote_snapshot()
    validate_host(snapshot)
    validate_v8r3(snapshot)
    need(snapshot["paths"]["parent"] and snapshot["paths"]["state"], "remote bootstrap namespace absent")
    need(snapshot.get("parent_mode") == "0755" and snapshot.get("parent_uid") == 0 and snapshot.get("parent_gid") == 0, "remote parent authority drift")
    need(snapshot.get("state_mode") == "0700", "remote state mode drift")
    need(snapshot.get("markers") == {"available": [], "consumed": []}, "markers exist before bootstrap audit")
    need(snapshot.get("latest_state", {}).get("state") == "BOOTSTRAP_CREATED_UNAUDITED", "bootstrap state drift")
    need(not snapshot.get("conflicting_processes"), "conflicting process during bootstrap audit")
    manifest_entries = validate_remote_manifest(snapshot, REMOTE_PARENT)
    index = tree_index(snapshot, "parent_tree")
    original = snapshot["v8r3"]["elf"]
    need(original.get("mode") == "0555", "V8R3 source ELF mode changed")
    need(str(REMOTE_EXECUTABLE) not in index, "V9 bootstrap unexpectedly copied the executable")
    bootstrap = json.loads(remote_text(REMOTE_PARENT / "BOOTSTRAP_RECEIPT.json"))
    need(bootstrap.get("verdict") == "M3_V9_BOOTSTRAP_CREATED_UNAUDITED", "bootstrap producer verdict drift")
    need(bootstrap.get("executable_build_id") == EXPECTED_BUILD_ID, "bootstrap Build ID drift")
    need(bootstrap.get("executable") == original and bootstrap.get("executable_copied") is False, "bootstrap executable binding drift")
    need(bootstrap.get("markers_expected") == 1 and bootstrap.get("markers_created") == 0, "bootstrap marker ledger drift")
    need(
        bootstrap.get("v8r1_predecessor", {}).get("remote_producer_state", {}).get("state") == "E2E_CREATED_UNAUDITED"
        and bootstrap.get("v8r1_predecessor", {}).get("independent_terminal", {}).get("verdict") == "BLOCKED_PROVENANCE",
        "bootstrap V8R1 predecessor pair drift",
    )
    need(
        bootstrap.get("v8r2_predecessor", {}).get("diagnosis_verdict") == "V8R2_REMOTE_TERMINAL_PROJECTION_DEFECT_CONFIRMED"
        and bootstrap.get("v8r2_predecessor", {}).get("remote_paths_absent") == {"parent": True, "state": True, "cache": True},
        "bootstrap V8R2 predecessor closure drift",
    )
    v8r3 = bootstrap.get("v8r3_predecessor", {})
    need(
        v8r3.get("executable") == original
        and v8r3.get("build_id") == EXPECTED_BUILD_ID
        and v8r3.get("wrapper") == snapshot["v8r3"]["wrapper"]
        and v8r3.get("state") == snapshot["v8r3"]["latest_state"]
        and v8r3.get("markers") == snapshot["v8r3"]["markers"],
        "bootstrap V8R3 predecessor closure drift",
    )
    uid = bootstrap.get("uid_capability", {})
    need(
        uid.get("verdict") == "PASS"
        and isinstance(uid.get("uid"), int)
        and uid["uid"] >= 0
        and uid.get("operations") == ["traverse", "stat", "read", "create", "write", "fsync", "rename", "reopen", "unlink"]
        and str(REMOTE_EXECUTABLE) in uid.get("required_reads", []),
        "bootstrap UID capability closure drift",
    )
    need(bootstrap.get("cargo_invocations") == 0 and bootstrap.get("rustc_compilations") == 0 and bootstrap.get("subject_executions") == 0, "bootstrap execution ledger drift")
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v9-bootstrap-mirror-"))
    try:
        mirror = temporary / "REMOTE_BOOTSTRAP"
        copy_remote(REMOTE_PARENT, mirror)
        mirror_manifest_entries = verify_manifest(mirror)
        need(mirror_manifest_entries == manifest_entries, "SSH evidence mirror manifest drift")
        mirror_manifest_sha256 = sha256_file(mirror / "SHA256SUMS")
    finally:
        shutil.rmtree(temporary, ignore_errors=True)
    receipt = {
        "schema": "lay.m3-v9-bootstrap-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_BOOTSTRAP_AUDIT_PASS_MARKER_ADMITTED",
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
        "bootstrap_receipt_sha256": index[str(REMOTE_PARENT / "BOOTSTRAP_RECEIPT.json")]["sha256"],
        "remote_manifest_entries": manifest_entries,
        "v8r3_source_elf": original,
        "v9_executable_copy_present": False,
        "elf_build_id": EXPECTED_BUILD_ID,
        "uid_capability": uid,
        "ssh_evidence_mirror": {
            "verdict": "PASS",
            "manifest_entries": mirror_manifest_entries,
            "sha256sums_sha256": mirror_manifest_sha256,
        },
        "markers_expected": 1,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "live_projection": snapshot,
    }
    return publish_tree(BOOTSTRAP_AUDIT_ROOT, "BOOTSTRAP_AUDIT.json", receipt)


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


def quiet_admission() -> dict[str, Any]:
    fixed_local()
    bootstrap = load_json(BOOTSTRAP_AUDIT_RECEIPT)
    need(bootstrap.get("verdict") == "M3_V9_BOOTSTRAP_AUDIT_PASS_MARKER_ADMITTED", "bootstrap audit absent")
    before = remote_snapshot()
    validate_host(before)
    validate_v8r3(before)
    need(before.get("latest_state", {}).get("state") == "TRACE_MARKER_AVAILABLE", "quiet preflight state drift")
    need(before.get("markers") == {"available": ["trace.available"], "consumed": []}, "quiet marker state drift")
    need(not before.get("conflicting_processes"), "conflicting process before TRACE")
    marker = tree_index(before, "state_tree").get(str(REMOTE_STATE / "markers/trace.available"), {})
    expected = marker_payload(bootstrap)
    need(marker.get("mode") == "0400" and marker.get("size_bytes") == len(expected) and marker.get("sha256") == sha256_bytes(expected), "canonical marker drift")
    time.sleep(5)
    after = remote_snapshot()
    validate_host(after)
    validate_v8r3(after)
    need(after.get("latest_state") == before.get("latest_state") and after.get("markers") == before.get("markers"), "remote state changed during quiet audit")
    need(not after.get("conflicting_processes"), "remote host stopped being quiet")
    thermal_drift = {
        key: [before["thermal"].get(key), after["thermal"].get(key)]
        for key in sorted(set(before["thermal"]) | set(after["thermal"]))
        if before["thermal"].get(key) != after["thermal"].get(key)
    }
    need(not thermal_drift, "thermal throttle changed during quiet audit")
    receipt = {
        "schema": "lay.m3-v9-quiet-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_QUIET_HOST_TRACE_ADMITTED",
        "local_controller_sha256": bootstrap["local_controller_sha256"],
        "remote_controller_sha256": bootstrap["remote_controller_sha256"],
        "auditor_sha256": bootstrap["auditor_sha256"],
        "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
        "elf_sha256": EXPECTED_ELF_SHA256,
        "elf_build_id": EXPECTED_BUILD_ID,
        "host_before": before,
        "host_after": after,
        "quiet_seconds": 5,
        "thermal_throttle_drift": thermal_drift,
        "conflicting_processes": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
    }
    return publish_tree(QUIET_ROOT, "QUIET_ADMISSION.json", receipt)


def integer(value: Any, default: int = -1) -> int:
    return value if isinstance(value, int) and not isinstance(value, bool) else default


def expected_command() -> list[str]:
    subject = REMOTE_PARENT / "trace-v1/subject"
    environment = {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(V8R1_INPUTS / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_M3_ACTUAL_OWNER_V7": str(V8R1_INPUTS / "slice8b-v7-fixed-13x100.json"),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LAY_L2_PACKAGE": str(V8R1_INPUTS / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(V8R1_INPUTS / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m"),
        "LAY_L11_RECEIPT": str(V8R1_INPUTS / "l11-installed.json"),
        "LAY_L2_FIELD_TRACE": "1",
    }
    return [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0", str(REMOTE_EXECUTABLE),
        "--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]


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
    schedules = ("FORWARD", "REVERSED", "FORWARD", "REVERSED")
    for round_index, schedule in enumerate(schedules):
        start = round_index * WARMUP_ROWS
        subset = measured[start : start + WARMUP_ROWS]
        rounds.append({
            "round": round_index + 1,
            "schedule": schedule,
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
            phase, round_index, schedule, case_ordinal = "WARMUP", 0, "FORWARD", ordinal
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


def terminal_decision(
    subject: Mapping[str, Any],
    wrapper: Mapping[str, Any],
    trace_rows: Sequence[Mapping[str, Any]],
    trace_errors: Sequence[str],
    trace_summary: Mapping[str, Any] | None,
    retained_rows: Mapping[str, Any],
    retained_summary: Mapping[str, Any] | None,
) -> tuple[str, dict[str, list[str]]]:
    failures = {key: [] for key in ("provenance", "semantic", "capability")}
    if wrapper.get("schema") != "lay.m3-v9-trace-wrapper.v1" or wrapper.get("task_id") != TASK_ID or wrapper.get("transaction_id") != TRANSACTION_ID:
        failures["provenance"].append("producer wrapper namespace or schema drift")
    if wrapper.get("command") != expected_command():
        failures["provenance"].append("direct scientific command drift")
    if wrapper.get("environment") != {
        token.split("=", 1)[0]: token.split("=", 1)[1]
        for token in expected_command()
        if "=" in token and not token.startswith("/")
    }:
        failures["provenance"].append("scientific environment drift")
    direct = wrapper.get("direct_exec_identity", {})
    processes = direct.get("processes") if isinstance(direct.get("processes"), list) else []
    if direct.get("observed") is not True or not processes:
        failures["capability"].append("direct executable launch was not observed")
    if direct.get("target") != str(REMOTE_EXECUTABLE):
        failures["provenance"].append("direct executable target drift")
    for row in processes:
        if row.get("executable") != str(REMOTE_EXECUTABLE) or not row.get("argv") or row["argv"][0] != str(REMOTE_EXECUTABLE) or SCIENTIFIC_TEST not in row["argv"]:
            failures["provenance"].append("direct executable process identity drift")
    if trace_errors:
        failures["provenance"].extend(str(error) for error in trace_errors)
    if trace_summary is None:
        failures["provenance"].append("independent trace summary absent")
    if (
        retained_rows.get("schema") != "lay.m3-v9-materialization-trace-rows.v1"
        or retained_rows.get("task_id") != TASK_ID
        or retained_rows.get("transaction_id") != TRANSACTION_ID
        or retained_rows.get("parse_errors") != list(trace_errors)
        or retained_rows.get("rows") != list(trace_rows)
    ):
        failures["provenance"].append("retained trace rows disagree with independent parser")
    if retained_summary != trace_summary or wrapper.get("trace_summary") != trace_summary:
        failures["provenance"].append("retained trace summary disagrees with independent parser")
    if wrapper.get("trace_rows") != len(trace_rows) or wrapper.get("trace_parse_errors") != list(trace_errors):
        failures["provenance"].append("producer trace cardinality projection drift")
    if wrapper.get("controller_error") is not None:
        failures["capability"].append("subject launch capability failure")
    if wrapper.get("outputs_complete") is not True:
        failures["provenance"].append("producer observation is incomplete")
    if subject.get("schema") != "lay.m3-end-to-end-test-owner.v1":
        failures["capability"].append("scientific receipt schema unavailable")
    if wrapper.get("verdict") not in {"M3_V9_TRACE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"}:
        failures["provenance"].append("producer wrapper verdict drift")
    if wrapper.get("verdict") == "M3_V9_TRACE_CREATED_UNAUDITED" and (
        wrapper.get("exit_pair_consistent") is not True or wrapper.get("outputs_complete") is not True
    ):
        failures["provenance"].append("positive producer projection is incomplete")
    source = subject.get("source", {})
    if source.get("v13_sha256") != EXPECTED_V13_SHA or source.get("v7_sha256") != EXPECTED_V7_SHA:
        failures["provenance"].append("fixed input identity drift")
    if source.get("productive_v90_sha256") != EXPECTED_PRODUCTIVE_SHA or source.get("l11_sha256") != EXPECTED_L11_SHA:
        failures["provenance"].append("actual-owner package tuple drift")
    if source.get("test_elf_sha256") != EXPECTED_ELF_SHA256:
        failures["provenance"].append("test ELF identity drift")
    fixed = subject.get("fixed_proof", {})
    semantic = fixed.get("semantic", {})
    for field in SEMANTIC_FIELDS:
        if integer(semantic.get(field)) != 0:
            failures["semantic"].append(f"{field} is nonzero or unknown")
    if integer(fixed.get("empty_lane_mismatches")) != 0:
        failures["semantic"].append("empty-lane parity mismatch")
    if integer(semantic.get("capacity_failures")) != 0 or integer(semantic.get("unresolved")) != 0:
        failures["semantic"].append("capacity or unresolved count nonzero")
    if integer(fixed.get("maximum_query_scratch_bytes"), 2**63) > 512 * 1024:
        failures["semantic"].append("maximum query scratch exceeds 512 KiB")
    reload = subject.get("reload", {})
    expected_reload = {
        "reader_identity_mismatches": 0,
        "mixed_generation_observations": 0,
        "stale_a_commits": 0,
        "stale_a_cancellations": 1,
        "current_b_commits": 1,
        "failed_build_publications": 0,
        "rollback_identity_mismatches": 0,
        "typed_materializations": 2,
        "per_request_typed_materializations": 0,
    }
    for field, expected in expected_reload.items():
        if integer(reload.get(field)) != expected:
            failures["semantic"].append(f"reload field drift: {field}")
    if reload.get("held_a_survived_publication") is not True:
        failures["semantic"].append("held generation A did not survive")
    pss = subject.get("pss", {})
    if integer(pss.get("aggregate_delta_pss_kib"), 2**63) > 40 * 1024:
        failures["semantic"].append("aggregate two-process PSS delta exceeds 40 MiB")
    if integer(pss.get("typed_owned_bytes_per_process")) != 3_689_628:
        failures["semantic"].append("typed payload byte count drift")
    if integer(pss.get("sidecar_bytes"), 2**63) > 32 * 1024 * 1024 or integer(pss.get("helper_failures")) != 0:
        failures["semantic"].append("sidecar or PSS helper gate failed")
    if integer(fixed.get("cases")) != 382 or integer(fixed.get("measured_rounds")) != 4 or integer(fixed.get("measured_samples")) != 1_528:
        failures["provenance"].append("request denominator drift")
    if fixed.get("schedule") != ["FORWARD", "REVERSED", "FORWARD", "REVERSED"]:
        failures["provenance"].append("schedule drift")
    if integer(fixed.get("cpu")) != 0 or integer(fixed.get("cpu_mismatches")) != 0 or integer(fixed.get("warmup_cpu_mismatches")) != 0:
        failures["semantic"].append("CPU0 execution closure failed")
    if wrapper.get("thermal_throttle_drift") not in ({}, None):
        failures["provenance"].append("thermal throttle counters changed")
    if wrapper.get("subject_executions") != 1 or wrapper.get("cargo_invocations") != 0 or wrapper.get("rustc_compilations") != 0 or wrapper.get("perf_record_invocations") != 0 or wrapper.get("perf_stat_invocations") != 0:
        failures["provenance"].append("TRACE execution ledger drift")
    gates = subject.get("gates", {})
    for field in ("semantic", "capacity", "reload_identity", "rss", "environment"):
        if gates.get(field) is not True:
            failures["semantic"].append(f"non-latency gate is not true: {field}")
    subject_verdict = subject.get("verdict")
    exit_code = wrapper.get("exit_code")
    accepted_pair = (subject_verdict == "M3_END_TO_END_TEST_OWNER_PASS" and exit_code == 0 and gates.get("latency") is True) or (
        subject_verdict == "BLOCKED_LATENCY" and exit_code == 101 and gates.get("latency") is False
    )
    if not accepted_pair and not failures["semantic"]:
        failures["capability"].append("subject verdict/exit/latency pair is unsupported")
    claim = subject.get("claim_boundary", {})
    if (
        claim.get("test_only_generation_owner") is not True
        or claim.get("production_authority_admitted") is not False
        or claim.get("runtime_reload_edit_admitted") is not False
        or subject.get("runtime_authority_changed") is not False
        or subject.get("production_activation_admitted") is not False
    ):
        failures["provenance"].append("scientific claim boundary drift")
    if subject.get("perf_or_pmu_used") is not False or subject.get("network_used_by_subject") is not False or subject.get("installed_package_changed") is not False:
        failures["provenance"].append("subject side-effect boundary drift")
    priority = (("BLOCKED_PROVENANCE", "provenance"), ("BLOCKED_SEMANTIC", "semantic"), ("BLOCKED_CAPABILITY", "capability"))
    for verdict, key in priority:
        if failures[key]:
            return verdict, failures
    return "FINAL_MATERIALIZATION_DECOMPOSED", failures


def terminal_audit() -> dict[str, Any]:
    fixed_local()
    admission = load_json(ADMISSION_RECEIPT)
    quiet = load_json(QUIET_RECEIPT)
    need(quiet.get("verdict") == "M3_V9_QUIET_HOST_TRACE_ADMITTED", "quiet admission absent")
    snapshot = remote_snapshot()
    validate_host(snapshot)
    validate_v8r3(snapshot)
    need(snapshot.get("markers") == {"available": [], "consumed": ["trace.consumed-before-exec"]}, "terminal marker state drift")
    need(snapshot.get("latest_state", {}).get("state") in {"TRACE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"}, "terminal state drift")
    need(not snapshot.get("conflicting_processes"), "owned or conflicting process active at terminal audit")
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v9-terminal-"))
    try:
        remote_evidence = temporary / "REMOTE_TRACE"
        copy_remote(REMOTE_PARENT / "trace-v1", remote_evidence)
        manifest_entries = verify_manifest(remote_evidence)
        wrapper_path = remote_evidence / "TRACE_WRAPPER.json"
        rows_path = remote_evidence / "TRACE_ROWS.json"
        summary_path = remote_evidence / "TRACE_SUMMARY.json"
        stderr_path = remote_evidence / "stderr.log"
        wrapper = load_json(wrapper_path) if wrapper_path.is_file() else {}
        retained_rows = load_json(rows_path) if rows_path.is_file() else {}
        retained_summary = load_json(summary_path) if summary_path.is_file() else None
        stderr = stderr_path.read_bytes() if stderr_path.is_file() else b""
        rows, errors, summary = parse_trace(stderr)
        subject = wrapper.get("subject_receipt") if isinstance(wrapper.get("subject_receipt"), dict) else {}
        verdict, failures = terminal_decision(subject, wrapper, rows, errors, summary, retained_rows, retained_summary)
        subject_path = remote_evidence / "subject/SUBJECT_RECEIPT.json"
        if subject_path.is_file():
            if load_json(subject_path) != subject:
                failures["provenance"].append("retained subject receipt disagrees with wrapper")
        elif subject:
            failures["provenance"].append("wrapper embeds a subject receipt that is absent from retained evidence")
        expected_marker = marker_payload(quiet)
        marker = wrapper.get("marker", {})
        before_marker = marker.get("before", {}) if isinstance(marker, dict) else {}
        after_marker = marker.get("after", {}) if isinstance(marker, dict) else {}
        if (
            marker.get("consumed_before_execution") is not True
            or before_marker.get("path") != str(REMOTE_STATE / "markers/trace.available")
            or after_marker.get("path") != str(REMOTE_STATE / "markers/trace.consumed-before-exec")
            or before_marker.get("mode") != "0400"
            or after_marker.get("mode") != "0400"
            or before_marker.get("size_bytes") != len(expected_marker)
            or after_marker.get("size_bytes") != len(expected_marker)
            or before_marker.get("sha256") != sha256_bytes(expected_marker)
            or after_marker.get("sha256") != sha256_bytes(expected_marker)
        ):
            failures["provenance"].append("one-shot marker consumption evidence drift")
        if local_runtime_snapshot() != admission.get("local_runtime_before"):
            failures["provenance"].append("local runtime authority changed")
        if snapshot.get("runtime_projection") != admission.get("remote_runtime_before"):
            failures["provenance"].append("remote runtime authority changed")
        if failures["provenance"]:
            verdict = "BLOCKED_PROVENANCE"
        elif failures["semantic"]:
            verdict = "BLOCKED_SEMANTIC"
        elif failures["capability"]:
            verdict = "BLOCKED_CAPABILITY"
        else:
            verdict = "FINAL_MATERIALIZATION_DECOMPOSED"
        receipt = {
            "schema": "lay.m3-v9-terminal-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": verdict,
            "positive_verdict": "FINAL_MATERIALIZATION_DECOMPOSED",
            "failure_priority": ["provenance", "semantic", "capability", "complete_decomposition"],
            "failures": failures,
            "local_controller_sha256": quiet["local_controller_sha256"],
            "remote_controller_sha256": quiet["remote_controller_sha256"],
            "auditor_sha256": quiet["auditor_sha256"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
            "quiet_admission_sha256": sha256_file(QUIET_RECEIPT),
            "v8r3_terminal_audit_sha256": sha256_file(V8R3_TERMINAL),
            "v8r3_subject_receipt_sha256": sha256_file(V8R3_SUBJECT),
            "v8r3_journal_sha256sums_sha256": sha256_file(V8R3_JOURNAL),
            "trace_wrapper_sha256": sha256_file(wrapper_path) if wrapper_path.is_file() else None,
            "trace_rows_sha256": sha256_file(rows_path) if rows_path.is_file() else None,
            "trace_summary_sha256": sha256_file(summary_path) if summary_path.is_file() else None,
            "stderr_sha256": sha256_file(stderr_path) if stderr_path.is_file() else None,
            "remote_manifest_entries": manifest_entries,
            "scientific_receipt": subject,
            "trace": summary,
            "dominant_stage": summary.get("tail", {}).get("dominant_stage") if isinstance(summary, dict) else None,
            "trace_rows": len(rows),
            "warmup_rows": WARMUP_ROWS,
            "measured_rows": MEASURED_ROWS,
            "markers_created": 1,
            "markers_consumed": 1,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "subject_executions": 1,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "installed_package_changed": False,
            "runtime_authority_changed": False,
            "production_authority_admitted": False,
            "v8r3_latency_reinterpreted": False,
            "outer_per_request_join_claimed": False,
            "next_if_pass": "separate paper decision about the measured dominant stage or its absence only",
            "live_projection": snapshot,
        }
        return publish_tree(TERMINAL_ROOT, "TERMINAL_AUDIT.json", receipt, {"REMOTE_TRACE": remote_evidence})
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def synthetic_positive() -> tuple[
    dict[str, Any],
    dict[str, Any],
    list[dict[str, Any]],
    dict[str, Any],
    dict[str, Any],
]:
    subject = {
        "schema": "lay.m3-end-to-end-test-owner.v1",
        "verdict": "M3_END_TO_END_TEST_OWNER_PASS",
        "source": {
            "v13_sha256": EXPECTED_V13_SHA,
            "v7_sha256": EXPECTED_V7_SHA,
            "productive_v90_sha256": EXPECTED_PRODUCTIVE_SHA,
            "l11_sha256": EXPECTED_L11_SHA,
            "test_elf_sha256": EXPECTED_ELF_SHA256,
        },
        "fixed_proof": {
            "cases": 382,
            "measured_rounds": 4,
            "measured_samples": 1_528,
            "schedule": ["FORWARD", "REVERSED", "FORWARD", "REVERSED"],
            "semantic": {**{key: 0 for key in SEMANTIC_FIELDS}, "capacity_failures": 0, "unresolved": 0},
            "empty_lane_mismatches": 0,
            "maximum_query_scratch_bytes": 1,
            "maximum_round_search_p99_us": 1,
            "maximum_round_total_material_p99_us": 1,
            "cpu": 0,
            "cpu_mismatches": 0,
            "warmup_cpu_mismatches": 0,
        },
        "reload": {
            "reader_identity_mismatches": 0,
            "mixed_generation_observations": 0,
            "stale_a_commits": 0,
            "stale_a_cancellations": 1,
            "current_b_commits": 1,
            "failed_build_publications": 0,
            "rollback_identity_mismatches": 0,
            "typed_materializations": 2,
            "per_request_typed_materializations": 0,
            "held_a_survived_publication": True,
        },
        "pss": {
            "aggregate_delta_pss_kib": 1,
            "typed_owned_bytes_per_process": 3_689_628,
            "sidecar_bytes": 1,
            "helper_failures": 0,
        },
        "gates": {"semantic": True, "capacity": True, "reload_identity": True, "rss": True, "latency": True, "environment": True},
        "claim_boundary": {"test_only_generation_owner": True, "production_authority_admitted": False, "runtime_reload_edit_admitted": False},
        "runtime_authority_changed": False,
        "production_activation_admitted": False,
        "perf_or_pmu_used": False,
        "network_used_by_subject": False,
        "installed_package_changed": False,
    }
    sample = (
        "productive_v90_materialization_trace surfaces=4 emitted=4 setup_us=1 "
        "projection_us=2 classify_us=3 gate_us=100 evidence_us=4\n"
    ).encode() * EXPECTED_TRACE_ROWS
    rows, errors, summary = parse_trace(sample)
    need(not errors and summary is not None, "synthetic trace fixture failed")
    retained_rows = {
        "schema": "lay.m3-v9-materialization-trace-rows.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "rows": rows,
        "parse_errors": [],
    }
    wrapper = {
        "schema": "lay.m3-v9-trace-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_TRACE_CREATED_UNAUDITED",
        "controller_error": None,
        "command": expected_command(),
        "environment": {
            token.split("=", 1)[0]: token.split("=", 1)[1]
            for token in expected_command()
            if "=" in token and not token.startswith("/")
        },
        "direct_exec_identity": {
            "observed": True,
            "target": str(REMOTE_EXECUTABLE),
            "processes": [{"pid": 1, "executable": str(REMOTE_EXECUTABLE), "argv": [str(REMOTE_EXECUTABLE), "--ignored", "--exact", SCIENTIFIC_TEST]}],
        },
        "exit_code": 0,
        "outputs_complete": True,
        "exit_pair_consistent": True,
        "trace_rows": EXPECTED_TRACE_ROWS,
        "trace_parse_errors": [],
        "trace_summary": summary,
        "thermal_throttle_drift": {},
        "subject_executions": 1,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
    }
    return subject, wrapper, rows, retained_rows, summary


def self_check() -> dict[str, Any]:
    need(ACTIONS == ("self-check", "live-admission", "bootstrap", "quiet", "terminal", "status"), "auditor registry drift")
    need(SEMANTIC_FIELDS[-1] == "semantic_total" and len(SEMANTIC_FIELDS) == 10, "semantic schema drift")
    need(len({ADMISSION_ROOT, BOOTSTRAP_AUDIT_ROOT, QUIET_ROOT, TERMINAL_ROOT}) == 4, "audit destination collision")
    command = expected_command()
    executable_index = command.index(str(REMOTE_EXECUTABLE))
    need(command[executable_index - 3 : executable_index] == ["/usr/bin/taskset", "-c", "0"], "direct command model drift")
    need(not any("ld-linux" in token for token in command), "loader route became reachable")
    terminal_projection = validate_local_predecessor_pair()
    subject, wrapper, rows, retained_rows, summary = synthetic_positive()
    verdict, failures = terminal_decision(subject, wrapper, rows, [], summary, retained_rows, summary)
    need(verdict == "FINAL_MATERIALIZATION_DECOMPOSED" and not any(failures.values()), "positive dispatch model failed")
    blocked_latency_subject = json.loads(json.dumps(subject))
    blocked_latency_subject["verdict"] = "BLOCKED_LATENCY"
    blocked_latency_subject["gates"]["latency"] = False
    blocked_latency_wrapper = dict(wrapper)
    blocked_latency_wrapper["exit_code"] = 101
    verdict, failures = terminal_decision(
        blocked_latency_subject,
        blocked_latency_wrapper,
        rows,
        [],
        summary,
        retained_rows,
        summary,
    )
    need(verdict == "FINAL_MATERIALIZATION_DECOMPOSED" and not any(failures.values()), "BLOCKED_LATENCY/101 dispatch model failed")
    semantic_subject = json.loads(json.dumps(subject))
    semantic_subject["fixed_proof"]["semantic"]["candidate_mismatches"] = 1
    verdict, _ = terminal_decision(semantic_subject, wrapper, rows, [], summary, retained_rows, summary)
    need(verdict == "BLOCKED_SEMANTIC", "failure priority model failed")
    incomplete_wrapper = dict(wrapper)
    incomplete_wrapper["outputs_complete"] = False
    verdict, _ = terminal_decision(subject, incomplete_wrapper, rows, [], summary, retained_rows, summary)
    need(verdict == "BLOCKED_PROVENANCE", "incomplete observation priority failed")
    capability_wrapper = dict(wrapper)
    capability_wrapper["controller_error"] = "synthetic launch failure"
    verdict, _ = terminal_decision(subject, capability_wrapper, rows, [], summary, retained_rows, summary)
    need(verdict == "BLOCKED_CAPABILITY", "capability dispatch model failed")
    return {
        "schema": "lay.m3-v9-independent-auditor-self-check.v1",
        "verdict": "M3_V9_INDEPENDENT_AUDITOR_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "auditor_sha256": sha256_file(AUDITOR),
        "actions": list(ACTIONS),
        "positive_dispatch": "PASS",
        "blocked_latency_pair_dispatch": "PASS",
        "failure_priority_dispatch": "PASS",
        "incomplete_observation_dispatch": "PASS",
        "capability_dispatch": "PASS",
        "trace_parser": "PASS",
        "trace_rows": len(rows),
        "synthetic_dominant_stage": summary["tail"]["dominant_stage"],
        "terminal_projection_parity": terminal_projection,
        "remote_writes": 0,
        "scientific_actions": 0,
    }


def status() -> dict[str, Any]:
    return {
        "schema": "lay.m3-v9-auditor-status.v1",
        "verdict": "M3_V9_AUDITOR_STATUS",
        "receipts": {
            "execution_admission": ADMISSION_RECEIPT.exists(),
            "bootstrap_audit": BOOTSTRAP_AUDIT_RECEIPT.exists(),
            "quiet_admission": QUIET_RECEIPT.exists(),
            "terminal_audit": TERMINAL_RECEIPT.exists(),
        },
        "remote": remote_snapshot(),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=ACTIONS)
    args = parser.parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "live-admission":
            value = live_admission()
        elif args.action == "bootstrap":
            value = bootstrap_audit()
        elif args.action == "quiet":
            value = quiet_admission()
        elif args.action == "terminal":
            value = terminal_audit()
        else:
            value = status()
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.m3-v9-auditor-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
