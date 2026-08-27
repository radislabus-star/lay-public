#!/usr/bin/env python3
"""Independent read-only audit of the one-shot primary-only D2 ELF."""

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
import subprocess
import sys
import time
from typing import Any, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

AUDITOR = pathlib.Path(__file__).resolve()
PROJECT_ROOT = AUDITOR.parents[1]
BUILD_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUILD_V1_2026-08-25"
)
D1_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25"
)
AUDIT_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUILD_AUDIT_V1_2026-08-26"
)
D2_ELF = BUILD_RESULT / "REMOTE_EVIDENCE/d2-test-elf"
D2_SOURCE = BUILD_RESULT / "REMOTE_EVIDENCE/assembled_d2_source.rs"
D2_WORKSPACE_SOURCE = BUILD_RESULT / (
    "REMOTE_EVIDENCE/source/src/nanda_wave/l2_field/v13_typed_peak.rs"
)
PREBUILD = BUILD_RESULT / "REMOTE_EVIDENCE/PREBUILD.json"
BUILD_RECEIPT = BUILD_RESULT / "D2_BUILD_RECEIPT.json"
LOCAL_BUILD_RECEIPT = BUILD_RESULT / "LOCAL_BUILD_RECEIPT.json"
D1_DECISION = D1_RESULT / "D1_DECISION.json"

REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_D2_ELF = REMOTE_PARENT / "build-v1/d2-test-elf"
REMOTE_D1_ELF = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-remaining-cost-d1-20260825/build-v1/diagnostic-test-elf"
)
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

READELF = pathlib.Path("/usr/bin/readelf")
NM = pathlib.Path("/usr/bin/nm")
ADDR2LINE = pathlib.Path("/usr/bin/addr2line")

EXPECTED = {
    "build_receipt": "a49a05bdf95ddbcc5cad78ef2376e861498a4311828d06abec83404f66b50953",
    "build_receipt_size": 1_667,
    "local_build_receipt": "5bed316600b1e04b7973d78697f5133f22fc3264f7027ddeafba4ad3360df240",
    "local_build_receipt_size": 2_558,
    "build_manifest": "aead466e48392f22db9394fc601724b7d8e90930f515c993390c37ab4702f28b",
    "build_manifest_size": 65_533,
    "d1_decision": "80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf",
    "d1_decision_size": 5_361_257,
    "d2_elf": "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178",
    "d2_elf_size": 317_706_232,
    "d2_build_id": "eb951f1a7526a9f1cb365040c10989aa5d3fc50f",
    "d2_text": "f57eba60bc4b1cadbeb2dfc524af59a7ab011a2e64afb0e1a0fe610755129d94",
    "d2_text_size": 15_980_919,
    "d2_text_address": 0x3F72C0,
    "source": "6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181",
    "source_size": 204_722,
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "production_prefix_size": 39_047,
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "d1_elf": "550f0d80ee49b114ac621b2f5099323480fd45847956a6807393511a8027d8fd",
    "d1_elf_size": 20_681_800,
    "d1_build_id": "9bdc0fb00420fd6358341d1a198927859cff89b8",
    "d1_text": "7336c3897a87172bf5175574d329196b84d43d499bc1f9e9274ecbd40889993b",
    "d1_text_size": 15_938_743,
}

MARKERS = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.available": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.available": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "u-single.available": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.available": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.available": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.available": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.available": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
    "t-single.available": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}

FROZEN_CARGO_SUFFIX = (
    "test",
    "--offline",
    "--locked",
    "--release",
    "--lib",
    "--no-run",
    "nanda_wave::l2_field::v13_typed_peak::tests",
)
FROZEN_ENV = {
    "CARGO_BUILD_JOBS": "20",
    "CARGO_INCREMENTAL": "0",
    "CARGO_NET_OFFLINE": "true",
    "CARGO_PROFILE_RELEASE_DEBUG": "2",
    "CARGO_PROFILE_RELEASE_STRIP": "none",
    "RUSTFLAGS": "",
}
REQUIRED_DEBUG_SECTIONS = {
    ".debug_info",
    ".debug_line",
    ".debug_abbrev",
    ".debug_ranges",
    ".debug_loc",
}


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


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
    value = file_identity(path)
    if digest is not None:
        require(value["sha256"] == digest, f"SHA mismatch: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode mismatch: {path}")
    return value


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


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


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new_bytes(path, json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n", mode)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    rows = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            rows.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return rows


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts, f"unsafe manifest path: {relative}")
        require(relative not in seen and relative != "SHA256SUMS", f"duplicate manifest row: {relative}")
        seen.add(relative)
        require(sha256_file(root / path) == digest, f"manifest mismatch: {root / path}")
    actual = {row["path"] for row in inventory(root) if row["path"] != "SHA256SUMS"}
    require(seen == actual, f"manifest membership mismatch: {root}")
    return len(seen)


def write_sha256sums(root: pathlib.Path) -> None:
    rows = [row for row in inventory(root) if row["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{row['sha256']}  {row['path']}\n" for row in rows).encode(),
        0o444,
    )


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


def run(command: Sequence[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if check and result.returncode != 0:
        raise AuditError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            + result.stderr.decode(errors="replace")[-4000:]
        )
    return result


def parse_sections(output: str) -> dict[str, dict[str, int]]:
    sections = {}
    pattern = re.compile(
        r"^\s*\[\s*\d+\]\s+(\S+)\s+\S+\s+([0-9a-fA-F]+)\s+"
        r"([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+"
    )
    for line in output.splitlines():
        match = pattern.match(line)
        if match:
            sections[match.group(1)] = {
                "address": int(match.group(2), 16),
                "offset": int(match.group(3), 16),
                "size": int(match.group(4), 16),
            }
    return sections


def parse_loads(output: str) -> list[dict[str, Any]]:
    loads = []
    for line in output.splitlines():
        parts = line.split()
        if not parts or parts[0] != "LOAD":
            continue
        require(len(parts) >= 8, f"bad PT_LOAD row: {line}")
        loads.append(
            {
                "offset": int(parts[1], 16),
                "vaddr": int(parts[2], 16),
                "paddr": int(parts[3], 16),
                "filesz": int(parts[4], 16),
                "memsz": int(parts[5], 16),
                "flags": "".join(parts[6:-1]),
                "align": int(parts[-1], 16),
            }
        )
    return loads


def hash_region(path: pathlib.Path, offset: int, size: int) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        source.seek(offset)
        remaining = size
        while remaining:
            block = source.read(min(remaining, 1024 * 1024))
            require(bool(block), f"short ELF region: {path}")
            digest.update(block)
            remaining -= len(block)
    return digest.hexdigest()


def elf_audit() -> tuple[dict[str, Any], dict[str, bytes]]:
    header = run([str(READELF), "-hW", str(D2_ELF)]).stdout
    notes = run([str(READELF), "-nW", str(D2_ELF)]).stdout
    section_output = run([str(READELF), "-SW", str(D2_ELF)]).stdout
    program_output = run([str(READELF), "-lW", str(D2_ELF)]).stdout
    symbols_output = run([str(NM), "-nS", "-C", str(D2_ELF)]).stdout.decode(errors="replace")
    header_text = header.decode(errors="replace")
    notes_text = notes.decode(errors="replace")
    sections_text = section_output.decode(errors="replace")
    programs_text = program_output.decode(errors="replace")

    type_match = re.search(r"^\s*Type:\s+(\S+)", header_text, re.MULTILINE)
    build_match = re.search(r"Build ID:\s*([0-9a-f]+)", notes_text)
    require(type_match is not None and type_match.group(1) == "DYN", "D2 ELF is not ET_DYN")
    require(build_match is not None and build_match.group(1) == EXPECTED["d2_build_id"], "D2 Build ID mismatch")
    sections = parse_sections(sections_text)
    require(".text" in sections, "D2 .text is absent")
    text = sections[".text"]
    require(text["address"] == EXPECTED["d2_text_address"], "D2 .text address mismatch")
    require(text["size"] == EXPECTED["d2_text_size"], "D2 .text size mismatch")
    text_hash = hash_region(D2_ELF, text["offset"], text["size"])
    require(text_hash == EXPECTED["d2_text"], "D2 .text SHA mismatch")
    require({".symtab", ".strtab"} <= set(sections), "D2 symbol tables are absent")
    require(REQUIRED_DEBUG_SECTIONS <= set(sections), "required D2 DWARF sections are absent")

    loads = parse_loads(programs_text)
    executable_loads = [row for row in loads if "E" in row["flags"]]
    require(len(executable_loads) == 1, "D2 executable PT_LOAD is not unique")
    executable_load = executable_loads[0]
    text_end = text["address"] + text["size"]
    load_end = executable_load["vaddr"] + executable_load["filesz"]
    require(executable_load["vaddr"] <= text["address"] and text_end <= load_end, ".text is outside executable PT_LOAD")

    hot_lines = [line for line in symbols_output.splitlines() if line.endswith("d1_enumerate_lane_prepared::<false>")]
    require(len(hot_lines) == 1, f"hot traversal symbol count mismatch: {len(hot_lines)}")
    symbol_match = re.match(r"^([0-9a-f]+)\s+([0-9a-f]+)\s+\w\s+(.+)$", hot_lines[0])
    require(symbol_match is not None, "hot traversal symbol row is malformed")
    symbol_start = int(symbol_match.group(1), 16)
    symbol_size = int(symbol_match.group(2), 16)
    require(symbol_start == 0x778320 and symbol_size > 0, "hot traversal range mismatch")
    addresses = (symbol_start, symbol_start + 1, symbol_start + 0xE0)
    addr2line = run(
        [str(ADDR2LINE), "-a", "-f", "-i", "-C", "-e", str(D2_ELF), *[hex(value) for value in addresses]]
    ).stdout
    addr2line_text = addr2line.decode(errors="replace")
    require("d1_enumerate_lane_prepared::<false>" in addr2line_text, "hot traversal DWARF function is unresolved")
    require("v13_typed_peak.rs:3105" in addr2line_text, "hot traversal source line is unresolved")
    require("v13_typed_peak.rs:3139" in addr2line_text, "hot traversal inline return frame is unresolved")

    audit = {
        "elf": require_file(D2_ELF, digest=EXPECTED["d2_elf"], size=EXPECTED["d2_elf_size"], mode="0555"),
        "elf_type": "ET_DYN",
        "pie": True,
        "build_id": build_match.group(1),
        "text": {
            "address": text["address"],
            "offset": text["offset"],
            "size_bytes": text["size"],
            "sha256": text_hash,
            "end_address_exclusive": text_end,
        },
        "pt_load": loads,
        "executable_pt_load": executable_load,
        "text_inside_executable_pt_load": True,
        "executable_pt_load_end_exclusive": load_end,
        "remaining_executable_slack_bytes": load_end - text_end,
        "symtab_present": True,
        "strtab_present": True,
        "dwarf_sections_present": sorted(REQUIRED_DEBUG_SECTIONS),
        "hot_symbol": {
            "name": symbol_match.group(3),
            "start": symbol_start,
            "size_bytes": symbol_size,
            "end_exclusive": symbol_start + symbol_size,
            "source_line_resolved": True,
            "inline_frames_resolved": True,
            "source_anchor": "src/nanda_wave/l2_field/v13_typed_peak.rs:3105",
        },
    }
    outputs = {
        "D2_ELF_HEADER.txt": header,
        "D2_ELF_NOTES.txt": notes,
        "D2_ELF_SECTIONS.txt": section_output,
        "D2_ELF_PROGRAM_HEADERS.txt": program_output,
        "D2_HOT_SYMBOL.txt": (hot_lines[0] + "\n").encode(),
        "D2_HOT_ADDR2LINE.txt": addr2line,
    }
    return audit, outputs


def verify_local_admission() -> dict[str, Any]:
    require(not AUDIT_RESULT.exists(), "D2 build audit already exists")
    require_file(SSH_IDENTITY, mode="0600")
    rows = {
        "build_receipt": require_file(BUILD_RECEIPT, digest=EXPECTED["build_receipt"], size=EXPECTED["build_receipt_size"], mode="0444"),
        "local_build_receipt": require_file(LOCAL_BUILD_RECEIPT, digest=EXPECTED["local_build_receipt"], size=EXPECTED["local_build_receipt_size"], mode="0444"),
        "build_manifest": require_file(BUILD_RESULT / "SHA256SUMS", digest=EXPECTED["build_manifest"], size=EXPECTED["build_manifest_size"], mode="0444"),
        "d1_decision": require_file(D1_DECISION, digest=EXPECTED["d1_decision"], size=EXPECTED["d1_decision_size"], mode="0444"),
        "d2_source": require_file(D2_SOURCE, digest=EXPECTED["source"], size=EXPECTED["source_size"], mode="0444"),
        "d2_workspace_source": require_file(D2_WORKSPACE_SOURCE, digest=EXPECTED["source"], size=EXPECTED["source_size"], mode="0444"),
    }
    require(verify_sha256sums(BUILD_RESULT) == 532, "build evidence manifest count mismatch")
    build = load_json(BUILD_RECEIPT)
    require(build.get("verdict") == "D2_BUILD_CREATED_UNAUDITED", "build receipt state mismatch")
    require(build.get("candidate_elf", {}).get("sha256") == EXPECTED["d2_elf"], "build receipt ELF mismatch")
    require(build.get("cargo_invocations") == 1 and build.get("cargo_exit") == 0, "build Cargo ledger mismatch")
    require(build.get("elf_executed") is False and build.get("elf_scientific_audit") is False, "build receipt audit boundary mismatch")
    require(build.get("other_markers_consumed") == 0, "build consumed another marker")
    prebuild = load_json(PREBUILD)
    require(tuple(prebuild.get("cargo_argv", [])[1:]) == FROZEN_CARGO_SUFFIX, "frozen Cargo argv mismatch")
    for key, expected in FROZEN_ENV.items():
        require(prebuild.get("build_environment", {}).get(key) == expected, f"frozen build environment mismatch: {key}")
    require(prebuild.get("assembled_source", {}).get("sha256") == EXPECTED["source"], "PREBUILD source mismatch")
    require(prebuild.get("production_prefix", {}).get("sha256") == EXPECTED["production_prefix"], "PREBUILD prefix mismatch")
    require(prebuild.get("cargo_toml", {}).get("sha256") == EXPECTED["cargo_toml"], "Cargo.toml mismatch")
    require(prebuild.get("cargo_lock", {}).get("sha256") == EXPECTED["cargo_lock"], "Cargo.lock mismatch")
    require(prebuild.get("cargo_guard", {}).get("sha256") == EXPECTED["cargo_guard"], "cargo-guard mismatch")
    d1 = load_json(D1_DECISION)
    encoded = canonical_json_bytes(d1)
    require(EXPECTED["d1_elf"].encode() in encoded and EXPECTED["d1_build_id"].encode() in encoded, "D1 decision ELF identity mismatch")
    return rows


def remote_probe_source() -> str:
    return f'''import hashlib,json,os,pathlib,re,stat,subprocess
TASK_ID={TASK_ID!r}
TRANSACTION_ID={TRANSACTION_ID!r}
PARENT=pathlib.Path({str(REMOTE_PARENT)!r})
STATE=pathlib.Path({str(REMOTE_STATE)!r})
D2=pathlib.Path({str(REMOTE_D2_ELF)!r})
D1=pathlib.Path({str(REMOTE_D1_ELF)!r})
def need(value,message):
    if not value: raise RuntimeError(message)
def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda:source.read(1024*1024),b''): h.update(block)
    return h.hexdigest()
def row(path):
    return {{'path':str(path),'mode':f'{{stat.S_IMODE(path.stat().st_mode):04o}}','size_bytes':path.stat().st_size,'sha256':sha(path)}}
def command(argv):
    result=subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
    need(result.returncode==0,f'command failed: {{argv}}: {{result.stderr[-2000:]!r}}')
    return result.stdout.decode(errors='replace')
header=command(['/usr/bin/readelf','-hW',str(D1)])
notes=command(['/usr/bin/readelf','-nW',str(D1)])
sections_output=command(['/usr/bin/readelf','-SW',str(D1)])
type_match=re.search(r'^\\s*Type:\\s+(\\S+)',header,re.MULTILINE)
build_match=re.search(r'Build ID:\\s*([0-9a-f]+)',notes)
section_match=re.search(r'^\\s*\\[\\s*\\d+\\]\\s+\\.text\\s+\\S+\\s+([0-9a-fA-F]+)\\s+([0-9a-fA-F]+)\\s+([0-9a-fA-F]+)\\s+',sections_output,re.MULTILINE)
need(type_match is not None and build_match is not None and section_match is not None,'D1 ELF parse failed')
address,offset,size=(int(section_match.group(index),16) for index in (1,2,3))
h=hashlib.sha256()
with D1.open('rb') as source:
    source.seek(offset); remaining=size
    while remaining:
        block=source.read(min(remaining,1024*1024)); need(bool(block),'short D1 .text')
        h.update(block); remaining-=len(block)
markers=[]
for path in sorted((STATE/'markers').iterdir()):
    markers.append({{**row(path),'name':path.name,'value':json.loads(path.read_text())}})
result={{
 'hostname':os.uname().nodename,
 'machine_id_sha256':sha(pathlib.Path('/etc/machine-id')),
 'parent_entries':sorted(path.name for path in PARENT.iterdir()),
 'state_entries':sorted(path.name for path in STATE.iterdir()),
 'build_state':row(STATE/'BUILD_STATE.json'),
 'build_state_value':json.loads((STATE/'BUILD_STATE.json').read_text()),
 'markers':markers,
 'bucket_map_artifacts_present':(PARENT/'bucket-map-v1').exists(),
 'd2_elf':row(D2),
 'd1_elf':row(D1),
 'd1_elf_type':'ET_DYN' if type_match.group(1)=='DYN' else type_match.group(1),
 'd1_build_id':build_match.group(1),
 'd1_text':{{'address':address,'offset':offset,'size_bytes':size,'sha256':h.hexdigest()}},
 'cargo_version':command(['cargo','-V']).strip(),
 'rustc_vv':command(['rustc','-Vv']).strip(),
 'readelf_version':command(['/usr/bin/readelf','--version']).splitlines()[0],
 'remote_writes':0,
}}
print(json.dumps(result,sort_keys=True,separators=(',',':')))
'''


def remote_projection() -> dict[str, Any]:
    source = remote_probe_source()
    compile(source, "<d2-build-audit-remote-probe>", "exec")
    command = [
        "/usr/bin/ssh",
        "-i",
        str(SSH_IDENTITY),
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        REMOTE,
        shlex.join(["/usr/bin/python3", "-c", source]),
    ]
    return json.loads(run(command).stdout)


def verify_remote_projection(value: Mapping[str, Any]) -> None:
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote hostname drift")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote machine identity drift")
    require(value.get("parent_entries") == ["build-v1", "d2a-v1"], "remote D2 parent drift")
    require(value.get("state_entries") == ["BUILD_STATE.json", "STATE.json", "markers", "route.lock"], "remote state entries drift")
    require(value.get("bucket_map_artifacts_present") is False, "bucket-map evidence already exists")
    require(value.get("remote_writes") == 0, "remote probe write ledger mismatch")
    build_state = value.get("build_state_value") or {}
    require(build_state.get("state") == "D2_BUILD_CREATED_UNAUDITED", "remote build state drift")
    require(build_state.get("transaction_id") == TRANSACTION_ID, "remote build transaction drift")
    d2 = value.get("d2_elf") or {}
    require(d2.get("sha256") == EXPECTED["d2_elf"] and d2.get("size_bytes") == EXPECTED["d2_elf_size"], "remote D2 ELF drift")
    require(d2.get("mode") == "0555", "remote D2 ELF mode drift")
    d1 = value.get("d1_elf") or {}
    require(d1.get("sha256") == EXPECTED["d1_elf"] and d1.get("size_bytes") == EXPECTED["d1_elf_size"], "remote D1 ELF drift")
    require(value.get("d1_elf_type") == "ET_DYN" and value.get("d1_build_id") == EXPECTED["d1_build_id"], "remote D1 ELF metadata drift")
    d1_text = value.get("d1_text") or {}
    require(d1_text.get("sha256") == EXPECTED["d1_text"] and d1_text.get("size_bytes") == EXPECTED["d1_text_size"], "remote D1 .text drift")
    require(value.get("cargo_version") == "cargo 1.97.1 (c980f4866 2026-06-30)", "remote Cargo version drift")
    rustc = value.get("rustc_vv", "")
    for expected in (
        "release: 1.97.1",
        "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452",
        "host: x86_64-unknown-linux-gnu",
        "LLVM version: 22.1.6",
    ):
        require(expected in rustc, f"remote rustc drift: {expected}")
    observed = {row["name"]: row for row in value.get("markers", [])}
    require(set(observed) == set(MARKERS), f"remote marker set drift: {sorted(observed)}")
    for name, (digest, size) in MARKERS.items():
        row = observed[name]
        require(row.get("sha256") == digest and row.get("size_bytes") == size, f"marker identity drift: {name}")
        require(row.get("mode") == "0400", f"marker mode drift: {name}")


def command_graph() -> dict[str, Any]:
    return {
        "external_actions": ["self-check", "audit"],
        "local_readers": [
            [str(READELF), "-hW|-nW|-SW|-lW", "<sealed-D2-ELF>"],
            [str(NM), "-nS", "-C", "<sealed-D2-ELF>"],
            [str(ADDR2LINE), "-a", "-f", "-i", "-C", "-e", "<sealed-D2-ELF>", "<hot-addresses>"],
        ],
        "remote_readers": ["readelf sealed D1", "cargo -V", "rustc -Vv", "state projection"],
        "remote_writes": [],
        "marker_mutations": [],
        "elf_execution": [],
        "perf_routes": [],
        "bucket_map_generation": [],
    }


def self_check() -> dict[str, Any]:
    compile(AUDITOR.read_text(encoding="utf-8"), str(AUDITOR), "exec")
    admission = verify_local_admission()
    graph = command_graph()
    require(graph["external_actions"] == ["self-check", "audit"], "auditor action graph drift")
    require(not graph["remote_writes"] and not graph["marker_mutations"], "auditor has a mutation route")
    require(not graph["elf_execution"] and not graph["perf_routes"], "auditor has a forbidden execution route")
    probe = remote_probe_source()
    for forbidden in ("write_text", "write_bytes", "os.rename", "unlink(", "mkdir(", "perf record", "perf stat"):
        require(forbidden not in probe, f"remote probe contains forbidden effect: {forbidden}")
    local_elf, outputs = elf_audit()
    remote = remote_projection()
    verify_remote_projection(remote)
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-build-auditor-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D2_BUILD_AUDITOR_VERIFIED_UNRUN",
        "auditor": file_identity(AUDITOR),
        "admission": admission,
        "command_graph": graph,
        "local_elf": local_elf,
        "local_reader_outputs_sha256": {name: sha256_bytes(value) for name, value in outputs.items()},
        "remote_projection_sha256": sha256_bytes(canonical_json_bytes(remote)),
        "remote_writes": 0,
        "marker_mutations": 0,
        "elf_executed": False,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "bucket_map_created": False,
    }


def audit() -> dict[str, Any]:
    check = self_check()
    before = remote_projection()
    verify_remote_projection(before)
    local_elf, outputs = elf_audit()
    after = remote_projection()
    verify_remote_projection(after)
    require(before == after, "remote state/provenance changed during build audit")

    d1_text = before["d1_text"]
    size_delta = local_elf["text"]["size_bytes"] - d1_text["size_bytes"]
    delta_percent = size_delta * 100.0 / d1_text["size_bytes"]
    comparison = {
        "d1_elf": before["d1_elf"],
        "d1_build_id": before["d1_build_id"],
        "d1_text": d1_text,
        "d2_text": local_elf["text"],
        "text_byte_identical": local_elf["text"]["sha256"] == d1_text["sha256"],
        "text_size_delta_bytes": size_delta,
        "text_size_delta_percent": delta_percent,
        "causal_explanation": "NOT_ESTABLISHED",
        "difference_is_not_an_automatic_build_rejection": True,
        "physical_admissibility_owner": "future U/V perturbation gates",
    }
    require(comparison["text_byte_identical"] is False, "unexpected D1/D2 .text identity")
    require(size_delta == 42_176, "D1/D2 .text size delta mismatch")

    stage = pathlib.Path(f"{AUDIT_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        shutil.copy2(AUDITOR, stage / "auditor.py")
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "REMOTE_BEFORE.json", before)
        write_new_json(stage / "REMOTE_AFTER.json", after)
        for name, value in outputs.items():
            write_new_bytes(stage / name, value)
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-build-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_BUILD_AUDITED",
            "auditor": file_identity(stage / "auditor.py"),
            "build_receipt_sha256": EXPECTED["build_receipt"],
            "local_build_receipt_sha256": EXPECTED["local_build_receipt"],
            "d1_decision_sha256": EXPECTED["d1_decision"],
            "d2": local_elf,
            "d1_comparison": comparison,
            "source_provenance": {
                "assembled_source": require_file(D2_SOURCE, digest=EXPECTED["source"], size=EXPECTED["source_size"], mode="0444"),
                "workspace_source": require_file(D2_WORKSPACE_SOURCE, digest=EXPECTED["source"], size=EXPECTED["source_size"], mode="0444"),
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "production_prefix_size_bytes": EXPECTED["production_prefix_size"],
                "cargo_argv_exact": True,
                "build_environment_exact": True,
                "toolchain_exact": True,
            },
            "live_remote_projection_stable": True,
            "build_state_before_after": "D2_BUILD_CREATED_UNAUDITED",
            "build_marker_consumed": True,
            "bucket_map_marker_available": True,
            "other_available_markers": 9,
            "other_consumed_markers": 0,
            "remote_writes": 0,
            "marker_mutations": 0,
            "cargo_build_invocations": 0,
            "cargo_version_queries": 3,
            "rustc_compilations": 0,
            "rustc_version_queries": 3,
            "elf_executed": False,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "bucket_map_created": False,
            "parity_executed": False,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "separate bucket-map controller may verify this receipt and consume bucket-map.available",
            "stop_before_bucket_map": True,
        }
        write_new_json(stage / "D2_BUILD_AUDIT_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, AUDIT_RESULT)
        fsync_directory(AUDIT_RESULT.parent)
        return receipt
    except BaseException:
        if stage.exists():
            for path in [stage, *stage.rglob("*")]:
                path.chmod(0o700 if path.is_dir() else 0o600)
            shutil.rmtree(stage)
        raise


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "audit"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2 BUILD AUDIT ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
