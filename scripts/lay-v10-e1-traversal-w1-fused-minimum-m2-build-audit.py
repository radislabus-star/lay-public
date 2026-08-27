#!/usr/bin/env python3
"""Independent read-only audit of the single M2 diagnostic ELF."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import shlex
import shutil
import stat
import struct
import subprocess
import sys
import time
from collections.abc import Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2-v1-20260826"
TRANSACTION_ID = "c760eea52b6416b3529f9d684c315147b5a1140522114642c417d7db4065102c"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
AUDITOR = pathlib.Path(__file__).resolve()
LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-remote.py"
IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "IMPLEMENTATION_SELF_CHECK_V2_2026-08-26.json"
)
BOOTSTRAP_AUDIT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BOOTSTRAP_AUDIT_V1_2026-08-26/M2_BOOTSTRAP_AUDIT_RECEIPT.json"
)
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BUILD_AUDIT_V1_2026-08-26"
)
EXPECTED_ASSEMBLED = "8654217a1509ef4ca9ef3c3dda5080a7c784fb767359c52531a772c0feae68dc"
EXPECTED_PREFIX = "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26"
OWNER_TOKENS = ("m2_enumerate_lane_b", "m2_enumerate_lane_g", "m2_enumerate_lane_i")
EXPECTED_BUILD_ENVIRONMENT = {
    "CARGO_BUILD_JOBS": "20",
    "CARGO_INCREMENTAL": "0",
    "CARGO_NET_OFFLINE": "true",
    "CARGO_PROFILE_RELEASE_DEBUG": "2",
    "CARGO_PROFILE_RELEASE_STRIP": "none",
    "RUSTFLAGS": "",
}
EXPECTED_CARGO_TAIL = (
    "test",
    "--offline",
    "--locked",
    "--release",
    "--lib",
    "--no-run",
    "nanda_wave::l2_field::v13_typed_peak::tests::v10_m2_fused_minimum_physical",
)


class BuildAuditError(RuntimeError):
    pass


class BuildAuditIssue(BuildAuditError):
    def __init__(self, verdict: str, detail: str) -> None:
        super().__init__(detail)
        self.verdict = verdict
        self.detail = detail


def need(condition: bool, message: str) -> None:
    if not condition:
        raise BuildAuditError(message)


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


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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


def write_json(path: pathlib.Path, value: Any) -> None:
    write_new(path, canonical(value))


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def run(command: Sequence[str], *, timeout: float = 3_600) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if result.returncode != 0:
        raise BuildAuditError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            + result.stderr.decode(errors="replace")[-5000:]
        )
    return result


def ssh(command: Sequence[str]) -> bytes:
    return run(["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", REMOTE, shlex.join(list(command))]).stdout


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), "build manifest absent")
    expected = set()
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        need(relative not in expected and path.is_file() and sha256_file(path) == digest, f"build manifest drift: {relative}")
        expected.add(relative)
    actual = {path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file() and path != manifest}
    need(actual == expected, "build manifest inventory drift")
    return len(expected)


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def sums(root: pathlib.Path) -> None:
    rows = [f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n" for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file() and candidate.name != "SHA256SUMS")]
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def fixed_inputs() -> dict[str, Any]:
    need(IMPLEMENTATION_RECEIPT.is_file(), "implementation receipt absent")
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(implementation.get("verdict") == "M2_CONTROLLER_VERIFIED_UNRUN", "implementation verdict drift")
    for path in (AUDITOR, LOCAL_CONTROLLER, REMOTE_CONTROLLER):
        need(path.is_file(), f"source absent: {path}")
        compile(path.read_text(), str(path), "exec")
    return {
        "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
    }


def self_check() -> dict[str, Any]:
    values = fixed_inputs()
    need(not RESULT.exists(), "build audit result already exists")
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-build-audit-self-check.v2",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2_BUILD_AUDITOR_VERIFIED_UNRUN",
        **values,
        "elf_executions": 0,
        "remote_writes": 0,
        "perf_stat_invocations": 0,
    }


def live_projection() -> dict[str, Any]:
    code = r'''
import hashlib,json,os,pathlib,stat,sys
build=pathlib.Path(sys.argv[1]); state=pathlib.Path(sys.argv[2]); elf=build/'diagnostic-test-elf'; provenance=build/'BUILD_PROVENANCE.json'
def sha(p):
 d=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''): d.update(b)
 return d.hexdigest()
def row(p): return {'path':str(p),'mode':f'{stat.S_IMODE(p.stat().st_mode):04o}','size_bytes':p.stat().st_size,'sha256':sha(p)}
states=sorted(state.glob('STATE-*.json'))
print(json.dumps({'hostname':os.uname().nodename,'uid':os.geteuid(),'build_mode':f'{stat.S_IMODE(build.stat().st_mode):04o}','elf':row(elf),'provenance':row(provenance),'provenance_value':json.loads(provenance.read_text()),'manifest':row(build/'SHA256SUMS'),'latest_state':json.loads(states[-1].read_text()),'marker_names':sorted(p.name for p in (state/'markers').iterdir()),'remote_writes':0},sort_keys=True))
'''
    return json.loads(ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", code, str(REMOTE_BUILD), str(REMOTE_STATE)]).decode().strip().splitlines()[-1])


def copy_build(destination: pathlib.Path) -> None:
    result = run(["/usr/bin/scp", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-q", "-p", "-r", f"{REMOTE}:{REMOTE_BUILD}", str(destination)], timeout=3_600)
    need(result.returncode == 0, "build evidence copy failed")


def elf_sections(data: bytes) -> tuple[list[dict[str, Any]], list[dict[str, Any]], int]:
    need(data[:4] == b"\x7fELF" and data[4] == 2 and data[5] == 1, "ELF64 little-endian identity drift")
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, 0)
    elf_type = header[1]
    phoff, shoff = header[5], header[6]
    phentsize, phnum = header[9], header[10]
    shentsize, shnum, shstrndx = header[11], header[12], header[13]
    sections_raw = [struct.unpack_from("<IIQQQQIIQQ", data, shoff + index * shentsize) for index in range(shnum)]
    string_row = sections_raw[shstrndx]
    names = data[string_row[4] : string_row[4] + string_row[5]]
    def name(offset: int) -> str:
        end = names.find(b"\0", offset)
        return names[offset : end if end >= 0 else len(names)].decode(errors="replace")
    sections = [{"name": name(row[0]), "type": row[1], "flags": row[2], "address": row[3], "offset": row[4], "size": row[5]} for row in sections_raw]
    programs = []
    for index in range(phnum):
        row = struct.unpack_from("<IIQQQQQQ", data, phoff + index * phentsize)
        programs.append({"type": row[0], "flags": row[1], "offset": row[2], "vaddr": row[3], "file_size": row[5], "memory_size": row[6], "align": row[7]})
    return sections, programs, elf_type


def inspect_elf(path: pathlib.Path) -> dict[str, Any]:
    data = path.read_bytes()
    sections, programs, elf_type = elf_sections(data)
    need(elf_type == 3, "M2 ELF is not ET_DYN")
    by_name = {row["name"]: row for row in sections}
    text = by_name.get(".text")
    need(text is not None and text["size"] > 0, ".text absent")
    text_bytes = data[text["offset"] : text["offset"] + text["size"]]
    required_sections = {".symtab", ".strtab", ".debug_info", ".debug_line", ".debug_abbrev"}
    need(required_sections <= set(by_name), "symbols or DWARF sections absent")
    executable_loads = [row for row in programs if row["type"] == 1 and row["flags"] & 1]
    need(len(executable_loads) == 1, "executable PT_LOAD cardinality drift")
    load = executable_loads[0]
    need(load["vaddr"] <= text["address"] and text["address"] + text["size"] <= load["vaddr"] + load["memory_size"], ".text outside executable PT_LOAD")
    notes = run(["/usr/bin/readelf", "-n", str(path)]).stdout.decode(errors="replace")
    match = re.search(r"Build ID:\s*([0-9a-f]+)", notes)
    need(match is not None, "Build ID absent")
    symbols = run(["/usr/bin/nm", "-C", "--defined-only", "--print-size", str(path)]).stdout.decode(errors="replace")
    owners = {}
    for token in OWNER_TOKENS:
        rows = [line for line in symbols.splitlines() if token in line]
        need(rows, f"M2 machine owner absent: {token}")
        parsed = []
        for row in rows:
            fields = row.split(maxsplit=3)
            if len(fields) >= 4 and re.fullmatch(r"[0-9a-fA-F]+", fields[0]) and re.fullmatch(r"[0-9a-fA-F]+", fields[1]):
                parsed.append({"address": int(fields[0], 16), "size": int(fields[1], 16), "symbol": fields[3]})
        need(parsed and all(item["size"] > 0 for item in parsed), f"M2 owner range invalid: {token}")
        owners[token] = parsed
    starts = [item["address"] for rows in owners.values() for item in rows]
    need(len(starts) == len(set(starts)), "B/G/I code folding or owner overlap detected")
    owner_ranges = sorted(
        (item["address"], item["address"] + item["size"], token)
        for token, rows in owners.items()
        for item in rows
    )
    need(
        all(
            text["address"] <= start < end <= text["address"] + text["size"]
            for start, end, _ in owner_ranges
        ),
        "B/G/I machine owner outside .text",
    )
    need(
        all(left[1] <= right[0] for left, right in zip(owner_ranges, owner_ranges[1:])),
        "B/G/I machine owner ranges overlap",
    )
    return {
        "elf_sha256": sha256_bytes(data),
        "elf_size_bytes": len(data),
        "elf_mode": mode_string(path),
        "elf_type": "ET_DYN",
        "build_id": match.group(1),
        "text": {"address": text["address"], "size_bytes": text["size"], "sha256": sha256_bytes(text_bytes)},
        "executable_pt_load": load,
        "required_sections": sorted(required_sections),
        "machine_owners": owners,
        "machine_owner_ranges": [
            {"start": start, "end_exclusive": end, "owner": owner}
            for start, end, owner in owner_ranges
        ],
        "code_folding": False,
    }


def validate(live: dict[str, Any], build: pathlib.Path) -> dict[str, Any]:
    need(live.get("hostname") == "e-MEGA-MINI-M1-13th" and live.get("uid") == 0, "live host drift")
    need(live.get("latest_state", {}).get("state") == "BUILD_CREATED_UNAUDITED", "build state drift")
    provenance = live.get("provenance_value", {})
    need(provenance.get("verdict") == "M2_BUILD_CREATED_UNAUDITED", "build producer verdict drift")
    need(provenance.get("source", {}).get("assembled_source_sha256") == EXPECTED_ASSEMBLED, "assembled source drift")
    need(provenance.get("source", {}).get("production_prefix_sha256") == EXPECTED_PREFIX, "production prefix drift")
    need(provenance.get("executable", {}).get("executed") is False, "ELF executed before audit")
    manifest_files = verify_manifest(build)
    try:
        need(provenance.get("build", {}).get("cargo_invocations") == 1 and provenance.get("build", {}).get("exit_code") == 0, "Cargo ledger drift")
        prebuild = json.loads((build / "PREBUILD.json").read_text())
        build_environment = provenance.get("build", {}).get("environment", {})
        need(prebuild.get("environment") == build_environment, "PREBUILD/build environment drift")
        need(
            set(build_environment) == {*EXPECTED_BUILD_ENVIRONMENT, "CARGO_TARGET_DIR"}
            and all(build_environment.get(key) == value for key, value in EXPECTED_BUILD_ENVIRONMENT.items()),
            "frozen build environment drift",
        )
        target = pathlib.PurePosixPath(str(build_environment.get("CARGO_TARGET_DIR", "")))
        need(target.name == "target" and target.parent.name.startswith("workspace-"), "isolated Cargo target drift")
        command = provenance.get("build", {}).get("command", [])
        need(
            prebuild.get("command") == command
            and isinstance(command, list)
            and command
            and pathlib.PurePosixPath(str(command[0])).name == "cargo-guard.sh"
            and tuple(command[1:]) == EXPECTED_CARGO_TAIL,
            "frozen Cargo argv drift",
        )
        need(prebuild.get("cargo_started") is False and prebuild.get("retry_permitted") is False, "PREBUILD lifecycle drift")
        elf = inspect_elf(build / "diagnostic-test-elf")
    except BuildAuditError as error:
        raise BuildAuditIssue("BLOCKED_BUILD", str(error)) from error
    except Exception as error:
        raise BuildAuditIssue("BLOCKED_BUILD", f"{type(error).__name__}: {error}") from error
    need(elf["elf_sha256"] == live["elf"]["sha256"] == provenance["executable"]["sha256"], "ELF identity drift")
    return {"manifest_files": manifest_files, **elf}


def audit() -> dict[str, Any]:
    check = self_check()
    need(BOOTSTRAP_AUDIT.is_file(), "bootstrap audit absent")
    bootstrap = json.loads(BOOTSTRAP_AUDIT.read_text())
    need(bootstrap.get("verdict") == "M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED", "bootstrap audit drift")
    before = live_projection()
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        issue: BuildAuditIssue | None = None
        science: dict[str, Any] = {}
        try:
            copy_build(stage / "REMOTE_BUILD")
            science = validate(before, stage / "REMOTE_BUILD")
        except BuildAuditIssue as error:
            issue = error
        except Exception as error:
            issue = BuildAuditIssue("BLOCKED_PROVENANCE", f"{type(error).__name__}: {error}")
        try:
            after = live_projection()
            if after != before:
                issue = BuildAuditIssue("BLOCKED_PROVENANCE", "remote build changed during read-only audit")
        except Exception as error:
            after = {"projection_error": f"{type(error).__name__}: {error}"}
            issue = BuildAuditIssue("BLOCKED_PROVENANCE", "post-audit remote projection unavailable")
        verdict = "M2_BUILD_AUDITED_PARITY_ADMITTED" if issue is None else issue.verdict
        receipt = {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-build-audit.v2",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": verdict,
            "local_controller_sha256": check["local_controller_sha256"],
            "remote_controller_sha256": check["remote_controller_sha256"],
            "build_auditor_sha256": check["auditor_sha256"],
            "implementation_receipt_sha256": check["implementation_receipt_sha256"],
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT),
            **science,
            "failure_causes": [] if issue is None else [issue.detail],
            "elf_executions": 0,
            "perf_stat_invocations": 0,
            "subject_executions": 0,
            "remote_writes": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "one M2 parity route only" if issue is None else "terminal audit only; no rebuild",
        }
        write_json(stage / "M2_BUILD_AUDIT_RECEIPT.json", receipt)
        write_json(stage / "SELF_CHECK.json", check)
        write_json(stage / "REMOTE_BEFORE.json", before)
        write_json(stage / "REMOTE_AFTER.json", after)
        write_new(stage / "auditor.py", AUDITOR.read_bytes())
        sums(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
        return {**receipt, "receipt_sha256": sha256_file(RESULT / "M2_BUILD_AUDIT_RECEIPT.json")}
    except BaseException:
        if stage.exists():
            seal(stage)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2 BUILD AUDIT ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
