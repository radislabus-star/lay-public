#!/usr/bin/env python3
"""Independent admission and evidence auditor for the V10 substage trace."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import struct
import subprocess
import sys
import tempfile
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r1-20260827"
TRANSACTION_ID = "7d067078f66e4c64724e5f6568304c0d6f6ab1c1ada92e09a98e05a19f0e1f17"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v10r1-{TRANSACTION_ID}"

LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r1.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r1-remote.py"
AUDITOR = pathlib.Path(__file__).resolve()
V13_SOURCE = ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
LIVE_SOURCE = ROOT / "src/nanda_wave/l2_field/productive_v1/live.rs"
ADMISSION_SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"
DECISION_SOURCE = ROOT / "src/typing_transition/decision.rs"
V10_PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_2026-08-27.md"
V10R1_CORRECTION = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_ADMISSION_UID_CONTEXT_CORRECTION_V1_2026-08-27.md"
V10_V1_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_V1_ADMISSION_FAILURE_DIAGNOSIS_2026-08-27.json"
V10_V1_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_IMPLEMENTATION_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V10_V1_JOURNAL_MANIFEST = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
CONTROLLER_PREFLIGHT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_REMOTE_CONTROLLER_V1_PREFLIGHT_2026-08-27.json"
CONTROLLER_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_REMOTE_CONTROLLER_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"

RECEIPTS = ROOT / "docs/structural_gates/receipts"
ADMISSION_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_EXECUTION_ADMISSION_V1_2026-08-27"
BOOTSTRAP_AUDIT_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_BOOTSTRAP_AUDIT_V1_2026-08-27"
BUILD_AUDIT_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_BUILD_AUDIT_V1_2026-08-27"
QUIET_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_QUIET_ADMISSION_V1_2026-08-27"
TERMINAL_ROOT = RECEIPTS / "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R1_TERMINAL_AUDIT_V1_2026-08-27"

ADMISSION_RECEIPT = ADMISSION_ROOT / "EXECUTION_ADMISSION.json"
BOOTSTRAP_AUDIT_RECEIPT = BOOTSTRAP_AUDIT_ROOT / "BOOTSTRAP_AUDIT.json"
BUILD_AUDIT_RECEIPT = BUILD_AUDIT_ROOT / "BUILD_AUDIT.json"
QUIET_RECEIPT = QUIET_ROOT / "QUIET_ADMISSION.json"
TERMINAL_RECEIPT = TERMINAL_ROOT / "TERMINAL_AUDIT.json"

ACTIONS = ("self-check", "live-admission", "bootstrap", "build", "quiet", "terminal", "status")
HOSTNAME = "e-MEGA-MINI-M1-13th"
KERNEL = "6.8.0-124-generic"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
EXPECTED_SOURCE_SHA = "28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2"
EXPECTED_V13_SHA = "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"
EXPECTED_V7_SHA = "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"
EXPECTED_PRODUCTIVE_SHA = "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"
EXPECTED_L11_SHA = "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)
EXPECTED_STAGE_NAMES = (
    "unchanged", "explain_candidate", "replacement_glues_separate_words",
    "boundary_glues_short_function_tail", "boundary_eats_known_current_word",
    "boundary_changes_non_whitespace_surface", "multiword_last_vowel_completion",
    "adjacent_transposition_boundary_competition", "boundary_splits_known_word",
    "boundary_splits_weak_tail", "reflexive_suffix_requires_grammar",
    "known_current_surface_drift", "verify_action_operator", "surface_changes_left_context",
    "l2_surface_stem_truncation", "structural_over_compress",
    "structural_function_prefix_drop", "structural_phrase_part_growth",
    "structural_short_initial_growth", "structural_short_case_vowel",
    "structural_soft_sign_vowel", "structural_short_internal_consonant",
    "structural_short_same_length_multi_edit", "structural_same_tail_consonant",
    "structural_infinitive_overreach", "structural_protected_context_authority",
    "structural_known_word_different_known", "structural_short_layout_context",
    "structural_short_cyrillic_ascii", "structural_short_nanda_shrink",
    "structural_short_nanda_internal_vowel", "structural_nanda_unknown_word",
    "unproven_stable_surface_shape", "semantic_surface_authority", "completion_only",
    "final_class_dispatch",
)
EXPECTED_ACTION_NAMES = ("eligible", "suggest_only", "keep_original", "veto")
EXPECTED_REASON_NAMES = (
    "unchanged", "unexplained_signal_loss", "word_count_shrink_requires_boundary_class",
    "unsafe_boundary_glue_short_function_tail", "moved_prefix_eats_known_current_word",
    "boundary_operator_changes_surface", "unsafe_multi_word_vowel_completion",
    "single_letter_boundary_beats_transposition", "known_single_word_boundary_split",
    "weak_boundary_split_tail", "reflexive_suffix_requires_grammar_proof",
    "known_current_word_surface_drift", "edit_transition_not_verified",
    "surface_left_context_apply_blocked", "l2_surface_stem_truncation_low",
    "candidate_over_compresses_word", "function_prefix_letter_drop",
    "known_phrase_part_one_letter_growth", "short_initial_letter_growth",
    "short_case_vowel_drift", "soft_sign_vowel_drift", "short_internal_consonant_drift",
    "short_same_length_multi_edit_drift", "same_tail_single_consonant_drift",
    "known_form_to_infinitive_overreach",
    "protected_current_surface_rewrite_requires_context_authority",
    "known_word_to_different_known_word", "short_layout_without_phrase_context",
    "short_cyrillic_to_ascii_layout", "short_nanda_word_shrink",
    "short_nanda_internal_vowel_growth", "nanda_surface_unknown_word",
    "unproven_stable_surface_shape_drift", "semantic_wave_surface_authority_low",
    "completion_is_not_autocorrect", "protected_or_technical",
    "single_step_typo_still_unknown", "unknown_error_class", "class_allows_apply",
    "productive_v90_lattice_requires_common_l3", "productive_v90_lattice_abstained",
    "productive_v90_lattice_unavailable", "productive_v90_non_winner_requires_common_l3",
)
EXPECTED_TRACE_ROWS = 1_910
WARMUP_ROWS = 382
MEASURED_ROWS = 1_528
SCHEDULE = ("FORWARD", "REVERSED", "FORWARD", "REVERSED")
PREREGISTERED_TAIL_ORDINALS = (375, 371, 223, 366)
V9_TRACE_PREFIX = "productive_v90_materialization_trace "
V10_TRACE_PREFIX = "proposal_admission_substage_trace "
V9_TRACE_PATTERN = re.compile(
    r"^productive_v90_materialization_trace surfaces=(\d+) emitted=(\d+) setup_us=(\d+) "
    r"projection_us=(\d+) classify_us=(\d+) gate_us=(\d+) evidence_us=(\d+)$"
)
V10_TRACE_PATTERN = re.compile(
    r"^proposal_admission_substage_trace schema=([^ ]+) surfaces=(\d+) emitted=(\d+) "
    r"admission_calls=(\d+) admission_ns=(\d+) leaf_ns=(\d+) residual_ns=(\d+) "
    r"post_calls=(\d+) post_hits=(\d+) post_ns=(\d+) stages=([^ ]+) "
    r"actions=([^ ]+) reasons=([^ ]+) unknown_reasons=(\d+)$"
)
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


class V10AuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V10AuditError(message)


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
    rows = (root / "SHA256SUMS").read_text().splitlines()
    for row in rows:
        digest, relative = row.split("  ", 1)
        need(sha256_file(root / relative) == digest, f"manifest mismatch: {relative}")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "SHA256SUMS"
    }
    need(actual == {row.split("  ", 1)[1] for row in rows}, "manifest inventory drift")
    return len(rows)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def publish_tree(destination: pathlib.Path, receipt_name: str, receipt: Mapping[str, Any], copied: Mapping[str, pathlib.Path] | None = None) -> dict[str, Any]:
    need(not destination.exists(), f"audit evidence already exists: {destination}")
    stage = destination.with_name(f"{destination.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / receipt_name, canonical(receipt))
        for name, source in (copied or {}).items():
            target = stage / name
            if source.is_dir():
                shutil.copytree(source, target)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, destination)
        fsync_dir(destination.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return load_json(destination / receipt_name)


def load_sealed_receipt(root: pathlib.Path, receipt: pathlib.Path, allowed: Sequence[str]) -> dict[str, Any]:
    need(mode_string(root) == "0555" and mode_string(receipt) == "0444", f"receipt tree is not immutable: {root}")
    verify_manifest(root)
    value = load_json(receipt)
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, f"receipt namespace drift: {receipt}")
    need(value.get("verdict") in set(allowed), f"receipt verdict drift: {receipt}")
    return value


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
        stdout = result.stdout[-4000:].decode(errors="replace")
        stderr = result.stderr[-4000:].decode(errors="replace")
        raise V10AuditError(
            f"command failed ({result.returncode}): {list(argv)!r}\n"
            f"stdout:\n{stdout}\nstderr:\n{stderr}"
        )
    return result


def ssh_python(program: str, arguments: Sequence[str] = (), *, root: bool = False, timeout: float = 3_600) -> dict[str, Any]:
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
    need(lines, "remote snapshot returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote snapshot is not an object")
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
        V10_PAPER: "afb4709efd22a63119527d21d126629a08a811e2b4e73a68c211144df946bd27",
        V10R1_CORRECTION: "262f8ab7609218de41c364ba91688ec88b9a3cf4b3e32bd5f170d214ba7d805e",
        V10_V1_DIAGNOSIS: "4935b5501eacc4d67ea404b4edd3ca1455dfe336a2c20f9ab050fa4d7672ec25",
        V10_V1_IMPLEMENTATION: "3b750bd1ec66d67c6d7e1dbe306b0c04a02b7ab9a839ec07bad0d7397b8652a8",
        V10_V1_JOURNAL_MANIFEST: "099d3c8334b11c98c3e7f0893a11d1347648aed1746cfcc1e466a166d818f12b",
        CONTROLLER_PREFLIGHT: "557b670a20774340c6e8c52d7870f58b898750d2363ded139e098da8b8b671c6",
        V13_SOURCE: EXPECTED_SOURCE_SHA,
        LIVE_SOURCE: "36aeddd5e605e67377f99343f9937606ac774d9ba4bb5710152de060bc9d183b",
        ADMISSION_SOURCE: "b563f8a9400f9ca61d1d4bf4f31c06b146b30d6f2d78cd364e4d69636aafd3e3",
        DECISION_SOURCE: "ad3c6d450c01811844a49e9c714d0eb9ff80f7de7d2f03a2e8b3e290deda3691",
    }
    rows = {}
    for path, digest in expected.items():
        row = file_row(path)
        need(row["sha256"] == digest, f"fixed local identity drift: {path}")
        rows[path.name] = row
    need(CONTROLLER_IMPLEMENTATION.is_file(), "controller implementation receipt absent")
    implementation = load_sealed_receipt(
        CONTROLLER_IMPLEMENTATION.parent,
        CONTROLLER_IMPLEMENTATION,
        ("V10R1_REMOTE_CONTROLLERS_VERIFIED_UNRUN",),
    )
    for key, path in {
        "local_controller_sha256": LOCAL_CONTROLLER,
        "remote_controller_sha256": REMOTE_CONTROLLER,
        "auditor_sha256": AUDITOR,
    }.items():
        need(implementation.get(key) == sha256_file(path), f"controller implementation binding drift: {key}")
    rows["controller_implementation"] = file_row(CONTROLLER_IMPLEMENTATION)
    return {"files": rows, "implementation": implementation}


def local_runtime_snapshot() -> dict[str, Any]:
    names = ("lay", "lay-daemon", "lay-ibus-engine")
    rows = {}
    for name in names:
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
def sha(path):
 h=hashlib.sha256()
 with open(path,'rb') as f:
  for b in iter(lambda:f.read(1048576),b''): h.update(b)
 return h.hexdigest()
def row(path): return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}','size_bytes':path.stat().st_size,'sha256':sha(path)}
def tree(root): return [row(p) for p in sorted(root.rglob('*')) if p.is_file()]
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
states=sorted(state.glob('STATE-*.json')) if state.is_dir() else []
markers=state/'markers'
out={
 'hostname':os.uname().nodename,'kernel':os.uname().release,'machine_id_sha256':sha('/etc/machine-id'),
 'online':pathlib.Path('/sys/devices/system/cpu/online').read_text().strip(),
 'core':pathlib.Path('/sys/bus/event_source/devices/cpu_core/cpus').read_text().strip(),
 'atom':pathlib.Path('/sys/bus/event_source/devices/cpu_atom/cpus').read_text().strip(),
 'paths':{'parent':parent.exists(),'state':state.exists(),'cache':cache.exists()},
 'parent_mode':f'{stat.S_IMODE(parent.stat().st_mode):04o}' if parent.exists() else None,
 'state_mode':f'{stat.S_IMODE(state.stat().st_mode):04o}' if state.exists() else None,
 'parent_tree':tree(parent) if parent.exists() else [],
 'state_tree':tree(state) if state.exists() else [],
 'latest_state':json.loads(states[-1].read_text()) if states else None,
 'markers':{'available':sorted(p.name for p in markers.glob('*.available')) if markers.is_dir() else [],'consumed':sorted(p.name for p in markers.glob('*.consumed-before-exec')) if markers.is_dir() else []},
 'conflicting_processes':conflicts,'loadavg':pathlib.Path('/proc/loadavg').read_text().strip(),
 'thermal':{str(p):int(p.read_text().strip()) for p in pathlib.Path('/sys/devices/system/cpu').glob('cpu*/thermal_throttle/*') if p.read_text().strip().isdigit()},
 'runtime_projection':runtime(),'free_bytes':shutil.disk_usage('/home/e').free if False else os.statvfs('/home/e').f_bavail*os.statvfs('/home/e').f_frsize,
 'monotonic_ns':time.monotonic_ns(),
}
print(json.dumps(out,sort_keys=True))
'''


def remote_snapshot() -> dict[str, Any]:
    return ssh_python(
        REMOTE_SNAPSHOT,
        [TASK_ID, str(REMOTE_PARENT), str(REMOTE_STATE), str(REMOTE_CACHE)],
        root=True,
        timeout=3_600,
    )


REMOTE_TOOLCHAIN = r'''
import json,os,subprocess
environment={
 'HOME':'/home/e',
 'LANG':'C.UTF-8',
 'LC_ALL':'C.UTF-8',
 'PATH':'/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
 'RUST_BACKTRACE':'0',
}
def cmd(argv):
 p=subprocess.run(argv,env=environment,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,check=False)
 return {'argv':argv,'returncode':p.returncode,'stdout':p.stdout.strip(),'stderr':p.stderr.strip()}
print(json.dumps({
 'execution_uid':os.geteuid(),
 'environment':environment,
 'cargo':cmd(['/home/e/.cargo/bin/cargo','-V']),
 'rustc':cmd(['/home/e/.cargo/bin/rustc','-Vv']),
},sort_keys=True))
'''


def remote_toolchain_snapshot() -> dict[str, Any]:
    return ssh_python(REMOTE_TOOLCHAIN, root=True, timeout=60)


def validate_host(value: Mapping[str, Any]) -> None:
    need(value.get("hostname") == HOSTNAME, "remote hostname drift")
    need(value.get("kernel") == KERNEL, "remote kernel drift")
    need(value.get("machine_id_sha256") == MACHINE_ID_SHA256, "remote machine-id drift")
    need((value.get("online"), value.get("core"), value.get("atom")) == ("0-19", "0-11", "12-19"), "remote topology drift")


def validate_toolchain(value: Mapping[str, Any]) -> None:
    expected_environment = {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
    }
    need(value.get("execution_uid") == 0, "toolchain observer UID drift")
    need(value.get("environment") == expected_environment, "controlled toolchain environment drift")
    cargo = value.get("cargo", {})
    need(cargo.get("argv") == ["/home/e/.cargo/bin/cargo", "-V"], "remote Cargo argv drift")
    need(cargo.get("returncode") == 0, "remote Cargo query failed")
    need(cargo.get("stdout") == "cargo 1.97.1 (c980f4866 2026-06-30)", "remote Cargo drift")
    rustc_row = value.get("rustc", {})
    need(rustc_row.get("argv") == ["/home/e/.cargo/bin/rustc", "-Vv"], "remote rustc argv drift")
    need(rustc_row.get("returncode") == 0, "remote rustc query failed")
    rustc = str(rustc_row.get("stdout", ""))
    for token in ("release: 1.97.1", "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452", "LLVM version: 22.1.6"):
        need(token in rustc, "remote rustc drift")


UID_PROBE = r'''
import json,os,pathlib,sys,time
p=pathlib.Path(sys.argv[1]); p.mkdir(mode=0o700)
try:
 a=p/'a'; b=p/'b'; f=open(a,'xb'); f.write(b'v10-admission\n'); f.flush(); os.fsync(f.fileno()); f.close(); os.rename(a,b)
 assert b.read_bytes()==b'v10-admission\n'; b.unlink(); p.rmdir()
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
    need(before["paths"] == {"parent": False, "state": False, "cache": False}, "V10 remote namespace is not absent")
    need(not before.get("conflicting_processes"), "conflicting remote performance process is active")
    need(int(before.get("free_bytes", 0)) >= 40 * 1024**3, "remote free-space gate failed")
    toolchain = remote_toolchain_snapshot()
    validate_toolchain(toolchain)
    probe_path = f"/home/e/.cache/lay-m3-v10r1-admission-{TRANSACTION_ID}"
    probe = ssh_python(UID_PROBE, [probe_path], root=False)
    need(probe.get("verdict") == "PASS" and probe.get("probe_absent_after") is True, "UID e capability probe failed")
    time.sleep(2)
    after = remote_snapshot()
    validate_host(after)
    need(after["paths"] == before["paths"], "remote namespace changed during admission")
    need(not after.get("conflicting_processes"), "remote host stopped being quiet during admission")
    receipt = {
        "schema": "lay.v10r1-execution-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R1_EXECUTION_ADMITTED",
        "safe_to_execute": True,
        "local_controller_sha256": sha256_file(LOCAL_CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "implementation_receipt_sha256": sha256_file(CONTROLLER_IMPLEMENTATION),
        "host_before": before,
        "host_after": after,
        "build_toolchain": toolchain,
        "toolchain_version_queries": 2,
        "uid_capability": probe,
        "local_runtime_before": local_runtime_snapshot(),
        "remote_runtime_before": before["runtime_projection"],
        "namespace_absent": True,
        "conflicting_processes": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "fixed_local": local["files"],
    }
    return publish_tree(ADMISSION_ROOT, "EXECUTION_ADMISSION.json", receipt)


def tree_index(value: Mapping[str, Any], key: str) -> dict[str, Mapping[str, Any]]:
    return {str(row["path"]): row for row in value.get(key, [])}


def validate_remote_manifest(snapshot: Mapping[str, Any], root: pathlib.PurePosixPath) -> int:
    index = tree_index(snapshot, "parent_tree")
    manifest_path = str(root / "SHA256SUMS")
    need(manifest_path in index, f"remote manifest absent: {manifest_path}")
    raw = ssh_python(
        "import json,pathlib,sys; print(json.dumps({'text':pathlib.Path(sys.argv[1]).read_text()},sort_keys=True))",
        [manifest_path],
        root=True,
    )["text"]
    rows = str(raw).splitlines()
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
    local = fixed_local()
    admission = load_sealed_receipt(ADMISSION_ROOT, ADMISSION_RECEIPT, ("V10R1_EXECUTION_ADMITTED",))
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot["paths"]["parent"] and snapshot["paths"]["state"], "remote bootstrap namespace absent")
    need(snapshot.get("parent_mode") == "0555", "remote parent mode drift")
    need(snapshot.get("markers") == {"available": [], "consumed": []}, "markers exist before bootstrap audit")
    need(snapshot.get("latest_state", {}).get("state") == "BOOTSTRAP_CREATED_UNAUDITED", "bootstrap state drift")
    need(not snapshot.get("conflicting_processes"), "conflicting process during bootstrap audit")
    manifest_entries = validate_remote_manifest(snapshot, REMOTE_PARENT)
    parent = tree_index(snapshot, "parent_tree")
    receipt_path = str(REMOTE_PARENT / "BOOTSTRAP_RECEIPT.json")
    need(receipt_path in parent, "remote bootstrap receipt absent")
    receipt = ssh_python(
        "import json,pathlib,sys; print(pathlib.Path(sys.argv[1]).read_text())",
        [receipt_path],
        root=True,
    )
    # ssh_python parsed the receipt itself because the file contains one JSON object.
    need(receipt.get("verdict") == "V10R1_BOOTSTRAP_CREATED_UNAUDITED", "remote bootstrap producer verdict drift")
    need(receipt.get("markers_created") == 0 and receipt.get("cargo_invocations") == 0 and receipt.get("subject_executions") == 0, "bootstrap execution ledger drift")
    need(receipt.get("source_closure", {}).get("files", 0) >= 500, "source closure unexpectedly small")
    need(
        receipt.get("source_closure", {}).get("content_sha256")
        == local["implementation"].get("source_closure", {}).get("content_sha256"),
        "remote source closure does not match implementation seal",
    )
    l11_receipt_path = REMOTE_PARENT / "bootstrap-v1/inputs/l11-installed.json"
    l11 = ssh_python("import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_text())", [str(l11_receipt_path)], root=True)
    need(l11.get("artifact_sha256") == EXPECTED_L11_SHA and l11.get("runtime_authority") is False, "experiment L1.1 receipt drift")
    receipt_out = {
        "schema": "lay.v10r1-bootstrap-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R1_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED",
        "local_controller_sha256": admission["local_controller_sha256"],
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "auditor_sha256": admission["auditor_sha256"],
        "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
        "remote_bootstrap_receipt_sha256": parent[receipt_path]["sha256"],
        "manifest_entries": manifest_entries,
        "source_files": receipt["source_closure"]["files"],
        "source_bytes": receipt["source_closure"]["bytes"],
        "source_content_sha256": receipt["source_closure"]["content_sha256"],
        "markers_expected": 2,
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
    return publish_tree(BOOTSTRAP_AUDIT_ROOT, "BOOTSTRAP_AUDIT.json", receipt_out)


def inspect_elf(path: pathlib.Path) -> dict[str, Any]:
    data = path.read_bytes()
    need(data[:4] == b"\x7fELF" and data[4] == 2 and data[5] == 1, "candidate is not ELF64 little-endian")
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, 0)
    elf_type = header[1]
    section_offset, program_offset = header[6], header[5]
    section_entry_size, section_count, names_index = header[11], header[12], header[13]
    program_entry_size, program_count = header[9], header[10]
    sections = []
    raw_sections = []
    for index in range(section_count):
        row = struct.unpack_from("<IIQQQQIIQQ", data, section_offset + index * section_entry_size)
        raw_sections.append(row)
    names_row = raw_sections[names_index]
    names = data[names_row[4] : names_row[4] + names_row[5]]
    for row in raw_sections:
        start = row[0]
        end = names.find(b"\0", start)
        name = names[start:end].decode(errors="replace")
        sections.append({"name": name, "address": row[3], "offset": row[4], "size": row[5]})
    programs = []
    for index in range(program_count):
        row = struct.unpack_from("<IIQQQQQQ", data, program_offset + index * program_entry_size)
        programs.append({"type": row[0], "flags": row[1], "offset": row[2], "vaddr": row[3], "filesz": row[5], "memsz": row[6]})
    by_name = {row["name"]: row for row in sections}
    for required in (".text", ".symtab", ".strtab", ".debug_info", ".debug_line"):
        need(required in by_name and by_name[required]["size"] > 0, f"ELF section absent: {required}")
    text = by_name[".text"]
    executable_loads = [row for row in programs if row["type"] == 1 and row["flags"] & 1]
    need(any(row["vaddr"] <= text["address"] and text["address"] + text["size"] <= row["vaddr"] + row["memsz"] for row in executable_loads), ".text is outside executable PT_LOAD")
    readelf = run(["/usr/bin/readelf", "-n", str(path)]).stdout.decode(errors="replace")
    match = re.search(r"Build ID:\s*([0-9a-f]+)", readelf)
    need(match is not None, "ELF Build ID absent")
    symbols = run(["/usr/bin/nm", "-C", str(path)], timeout=600).stdout.decode(errors="replace")
    need("m3_end_to_end_physical_proof" in symbols and "m3_end_to_end_pss_helper" in symbols, "V10 test symbols absent")
    return {
        "elf_type": elf_type,
        "et_dyn": elf_type == 3,
        "build_id": match.group(1),
        "text": text,
        "text_sha256": sha256_bytes(data[text["offset"] : text["offset"] + text["size"]]),
        "symtab_present": True,
        "dwarf_info_present": True,
        "dwarf_line_present": True,
        "text_in_executable_load": True,
        "v8_symbols_present": True,
    }


def build_audit() -> dict[str, Any]:
    fixed_local()
    bootstrap = load_sealed_receipt(
        BOOTSTRAP_AUDIT_ROOT,
        BOOTSTRAP_AUDIT_RECEIPT,
        ("V10R1_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED",),
    )
    snapshot = remote_snapshot()
    validate_host(snapshot)
    build_state = snapshot.get("latest_state", {}).get("state")
    need(build_state in {"BUILD_CREATED_UNAUDITED", "BLOCKED_BUILD"}, "remote build state drift")
    need(snapshot.get("markers") == {"available": ["trace.available"], "consumed": ["build.consumed-before-exec"]}, "post-build marker state drift")
    need(not snapshot.get("conflicting_processes"), "owned or conflicting process active after build")
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10r1-build-audit-"))
    try:
        if build_state == "BLOCKED_BUILD":
            remote_failure = temporary / "REMOTE_BUILD_FAILURE"
            copy_remote(REMOTE_PARENT / "build-failure-v1", remote_failure)
            manifest_entries = verify_manifest(remote_failure)
            failure = load_json(remote_failure / "BUILD_FAILURE.json")
            need(failure.get("schema") == "lay.v10r1-build-failure.v1", "build failure schema drift")
            need(failure.get("task_id") == TASK_ID and failure.get("transaction_id") == TRANSACTION_ID, "build failure namespace drift")
            need(failure.get("verdict") == "BLOCKED_BUILD", "build failure verdict drift")
            need(failure.get("retry_permitted") is False and failure.get("runtime_authority_changed") is False, "build failure boundary drift")
            expected_marker = marker_payload("BUILD", bootstrap)
            marker = failure.get("marker", {}) if isinstance(failure.get("marker"), dict) else {}
            before_marker = marker.get("before", {}) if isinstance(marker.get("before"), dict) else {}
            after_marker = marker.get("after", {}) if isinstance(marker.get("after"), dict) else {}
            need(
                marker.get("consumed_before_execution") is True
                and before_marker.get("path") == str(REMOTE_STATE / "markers/build.available")
                and after_marker.get("path") == str(REMOTE_STATE / "markers/build.consumed-before-exec")
                and before_marker.get("mode") == "0400"
                and after_marker.get("mode") == "0400"
                and before_marker.get("size_bytes") == len(expected_marker)
                and after_marker.get("size_bytes") == len(expected_marker)
                and before_marker.get("sha256") == sha256_bytes(expected_marker)
                and after_marker.get("sha256") == sha256_bytes(expected_marker),
                "failed build marker evidence drift",
            )
            cargo_started = failure.get("cargo_started") is True
            need(integer(failure.get("cargo_invocations")) == int(cargo_started), "failed build Cargo ledger drift")
            receipt = {
                "schema": "lay.v10r1-build-audit.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "verdict": "BLOCKED_BUILD",
                "local_controller_sha256": bootstrap["local_controller_sha256"],
                "remote_controller_sha256": bootstrap["remote_controller_sha256"],
                "auditor_sha256": bootstrap["auditor_sha256"],
                "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
                "build_failure_sha256": sha256_file(remote_failure / "BUILD_FAILURE.json"),
                "manifest_entries": manifest_entries,
                "cargo_started": cargo_started,
                "cargo_invocations": int(cargo_started),
                "rustc_compilations": int(cargo_started),
                "elf_executed": False,
                "subject_executions": 0,
                "perf_record_invocations": 0,
                "perf_stat_invocations": 0,
                "runtime_authority_changed": False,
                "retry_permitted": False,
                "live_projection": snapshot,
            }
            return publish_tree(
                BUILD_AUDIT_ROOT,
                "BUILD_AUDIT.json",
                receipt,
                {"REMOTE_BUILD_FAILURE": remote_failure},
            )
        remote_build = temporary / "REMOTE_BUILD"
        copy_remote(REMOTE_PARENT / "build-v1", remote_build)
        manifest_entries = verify_manifest(remote_build)
        provenance = load_json(remote_build / "BUILD_PROVENANCE.json")
        need(provenance.get("verdict") == "V10R1_BUILD_CREATED_UNAUDITED", "build producer verdict drift")
        expected_tail = ["test", "--offline", "--locked", "--release", "--lib", "--no-run", "m3_v8"]
        command = provenance.get("build", {}).get("command", [])
        need(command[-len(expected_tail) :] == expected_tail, "Cargo argv drift")
        environment = provenance.get("build", {}).get("environment", {})
        expected_environment = {
            "CARGO_BUILD_JOBS": "20",
            "CARGO_INCREMENTAL": "0",
            "CARGO_NET_OFFLINE": "true",
            "CARGO_PROFILE_RELEASE_DEBUG": "2",
            "CARGO_PROFILE_RELEASE_STRIP": "none",
            "RUSTFLAGS": "",
        }
        for key, expected in expected_environment.items():
            need(environment.get(key) == expected, f"build environment drift: {key}")
        need(provenance.get("build", {}).get("cargo_invocations") == 1, "Cargo invocation count drift")
        need(provenance.get("build", {}).get("exit_code") == 0, "Cargo did not succeed")
        elf = remote_build / "v10-test-elf"
        elf_row = file_row(elf)
        need(elf_row["mode"] == "0555", "sealed ELF is not directly executable")
        need(elf_row["sha256"] == provenance.get("executable", {}).get("sha256"), "ELF SHA drift")
        audit = inspect_elf(elf)
        need(audit["et_dyn"], "V10 test ELF is not ET_DYN")
        need(audit["build_id"] == provenance.get("executable", {}).get("build_id"), "ELF Build ID drift")
        receipt = {
            "schema": "lay.v10r1-build-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "V10R1_BUILD_AUDIT_PASS_TRACE_ADMITTED",
            "local_controller_sha256": bootstrap["local_controller_sha256"],
            "remote_controller_sha256": bootstrap["remote_controller_sha256"],
            "auditor_sha256": bootstrap["auditor_sha256"],
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
            "build_provenance_sha256": sha256_file(remote_build / "BUILD_PROVENANCE.json"),
            "manifest_entries": manifest_entries,
            "elf_sha256": elf_row["sha256"],
            "elf_size_bytes": elf_row["size_bytes"],
            "elf": audit,
            "source_sha256": provenance["source"]["v13_typed_peak_sha256"],
            "cargo_invocations": 1,
            "rustc_compilations": 1,
            "elf_executed": False,
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
            "live_projection": snapshot,
        }
        return publish_tree(BUILD_AUDIT_ROOT, "BUILD_AUDIT.json", receipt, {"REMOTE_BUILD": remote_build})
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


CPU_IDLE = r'''
import json,pathlib,time
def row():
 out={}
 for line in pathlib.Path('/proc/stat').read_text().splitlines():
  fields=line.split()
  if not fields or not fields[0].startswith('cpu'): continue
  values=[int(x) for x in fields[1:]]; total=sum(values); idle=values[3]+(values[4] if len(values)>4 else 0); out[fields[0]]=(total,idle)
 return out
a=row(); time.sleep(5); b=row(); ratios={k:(b[k][1]-a[k][1])/(b[k][0]-a[k][0]) for k in a if k in b and b[k][0]>a[k][0]}
print(json.dumps({'idle_ratios':ratios,'cpu0_idle_ratio':ratios.get('cpu0'),'all_idle_ratio':ratios.get('cpu')},sort_keys=True))
'''


def quiet_admission() -> dict[str, Any]:
    fixed_local()
    build = load_sealed_receipt(
        BUILD_AUDIT_ROOT,
        BUILD_AUDIT_RECEIPT,
        ("V10R1_BUILD_AUDIT_PASS_TRACE_ADMITTED",),
    )
    before = remote_snapshot()
    validate_host(before)
    need(before.get("latest_state", {}).get("state") == "BUILD_CREATED_UNAUDITED", "quiet preflight state drift")
    need(before.get("markers") == {"available": ["trace.available"], "consumed": ["build.consumed-before-exec"]}, "quiet preflight markers drift")
    need(not before.get("conflicting_processes"), "conflicting process before TRACE")
    idle = ssh_python(CPU_IDLE, root=False, timeout=30)
    need(float(idle.get("cpu0_idle_ratio") or 0.0) >= 0.95, "CPU0 was not at least 95% idle")
    need(float(idle.get("all_idle_ratio") or 0.0) >= 0.90, "host was not at least 90% idle")
    after = remote_snapshot()
    validate_host(after)
    need(not after.get("conflicting_processes"), "conflicting process appeared during quiet preflight")
    need(before.get("thermal") == after.get("thermal"), "thermal throttle counter changed during quiet preflight")
    need(before.get("runtime_projection") == after.get("runtime_projection"), "remote runtime changed during quiet preflight")
    receipt = {
        "schema": "lay.v10r1-quiet-admission.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R1_QUIET_HOST_TRACE_ADMITTED",
        "local_controller_sha256": build["local_controller_sha256"],
        "remote_controller_sha256": build["remote_controller_sha256"],
        "auditor_sha256": build["auditor_sha256"],
        "build_audit_sha256": sha256_file(BUILD_AUDIT_RECEIPT),
        "elf_sha256": build["elf_sha256"],
        "host_before": before,
        "host_after": after,
        "idle_observation": idle,
        "quiet_seconds": 5,
        "thermal_throttle_drift": {},
        "conflicting_processes": 0,
        "cargo_invocations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
    }
    return publish_tree(QUIET_ROOT, "QUIET_ADMISSION.json", receipt)


def integer(value: Any, default: int = -1) -> int:
    return value if isinstance(value, int) and not isinstance(value, bool) else default


def nearest_rank(values: Sequence[int], percentile: int) -> int:
    need(values, "cannot summarize an empty distribution")
    ordered = sorted(int(value) for value in values)
    rank = max(1, (percentile * len(ordered) + 99) // 100)
    return ordered[rank - 1]


def distribution(values: Sequence[int]) -> dict[str, int]:
    need(values, "cannot summarize an empty distribution")
    normalized = [int(value) for value in values]
    return {
        "count": len(normalized),
        "p50": nearest_rank(normalized, 50),
        "p99": nearest_rank(normalized, 99),
        "max": max(normalized),
        "sum": sum(normalized),
    }


def parse_named_registry(raw: str, names: Sequence[str], fields: Sequence[str], label: str) -> dict[str, Any]:
    parts = raw.split(",") if raw else []
    need(len(parts) == len(names), f"{label} registry cardinality drift")
    result: dict[str, Any] = {}
    for expected_name, part in zip(names, parts, strict=True):
        tokens = part.split(":")
        need(len(tokens) == len(fields) + 1 and tokens[0] == expected_name, f"{label} registry order drift")
        values = []
        for token in tokens[1:]:
            need(token.isdigit(), f"{label} registry has a non-integer value")
            values.append(int(token))
        result[expected_name] = values[0] if len(fields) == 1 else dict(zip(fields, values, strict=True))
    return result


def trace_position(ordinal: int) -> dict[str, Any]:
    if ordinal < WARMUP_ROWS:
        return {"phase": "WARMUP", "round": 0, "schedule": "FORWARD", "case_ordinal": ordinal}
    measured_ordinal = ordinal - WARMUP_ROWS
    round_index = measured_ordinal // WARMUP_ROWS
    return {
        "phase": "MEASURED",
        "round": round_index + 1,
        "schedule": SCHEDULE[min(round_index, len(SCHEDULE) - 1)],
        "case_ordinal": measured_ordinal % WARMUP_ROWS,
    }


def consistency_signature(row: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "surfaces": row["surfaces"],
        "emitted": row["emitted"],
        "admission_calls": row["admission_calls"],
        "post_calls": row["post_calls"],
        "post_hits": row["post_hits"],
        "actions": row["actions"],
        "reasons": row["reasons"],
        "stage_calls_hits": {
            name: {"calls": row["stages"][name]["calls"], "hits": row["stages"][name]["hits"]}
            for name in EXPECTED_STAGE_NAMES
        },
    }


def four_round_mismatches(rows: Sequence[Mapping[str, Any]]) -> list[str]:
    measured = rows[WARMUP_ROWS:]
    if len(measured) != MEASURED_ROWS:
        return ["measured trace cardinality prevents four-round comparison"]
    failures = []
    for case_ordinal in range(WARMUP_ROWS):
        cohort = [row for row in measured if integer(row.get("case_ordinal")) == case_ordinal]
        if len(cohort) != 4:
            failures.append(f"case {case_ordinal}: expected four measured rows, found {len(cohort)}")
            continue
        baseline = consistency_signature(cohort[0])
        if any(consistency_signature(row) != baseline for row in cohort[1:]):
            failures.append(f"case {case_ordinal}: four-round action/reason/call/hit signature drift")
    return failures


def parse_trace(stderr: bytes) -> tuple[list[dict[str, Any]], dict[str, list[str]], dict[str, Any] | None]:
    failures = {"provenance": [], "semantic": []}
    v9_rows: list[dict[str, Any]] = []
    v10_rows: list[dict[str, Any]] = []
    for line_number, line in enumerate(stderr.decode("utf-8", errors="replace").splitlines(), start=1):
        if line.startswith(V9_TRACE_PREFIX):
            match = V9_TRACE_PATTERN.fullmatch(line)
            if match is None:
                failures["provenance"].append(f"line {line_number}: malformed V9 aggregate row")
                continue
            values = [int(value) for value in match.groups()]
            v9_rows.append({
                "stderr_line": line_number,
                "surfaces": values[0],
                "emitted": values[1],
                "setup_us": values[2],
                "projection_us": values[3],
                "classify_us": values[4],
                "gate_us": values[5],
                "evidence_us": values[6],
            })
        elif line.startswith(V10_TRACE_PREFIX):
            match = V10_TRACE_PATTERN.fullmatch(line)
            if match is None:
                failures["provenance"].append(f"line {line_number}: malformed V10 substage row")
                continue
            values = match.groups()
            try:
                need(values[0] == "v10-admission-substage-v1", "V10 trace schema drift")
                stages = parse_named_registry(
                    values[10], EXPECTED_STAGE_NAMES, ("calls", "hits", "elapsed_ns"), "stage"
                )
                actions = parse_named_registry(values[11], EXPECTED_ACTION_NAMES, ("count",), "action")
                reasons = parse_named_registry(values[12], EXPECTED_REASON_NAMES, ("count",), "reason")
            except BaseException as error:
                failures["provenance"].append(f"line {line_number}: {type(error).__name__}: {error}")
                continue
            numeric = [int(value) for value in values[1:10]]
            v10_rows.append({
                "stderr_line": line_number,
                "surfaces": numeric[0],
                "emitted": numeric[1],
                "admission_calls": numeric[2],
                "admission_ns": numeric[3],
                "leaf_ns": numeric[4],
                "residual_ns": numeric[5],
                "post_calls": numeric[6],
                "post_hits": numeric[7],
                "post_ns": numeric[8],
                "stages": stages,
                "actions": actions,
                "reasons": reasons,
                "unknown_reasons": int(values[13]),
            })
    if len(v9_rows) != EXPECTED_TRACE_ROWS:
        failures["provenance"].append(f"V9 trace row count {len(v9_rows)} != {EXPECTED_TRACE_ROWS}")
    if len(v10_rows) != EXPECTED_TRACE_ROWS:
        failures["provenance"].append(f"V10 trace row count {len(v10_rows)} != {EXPECTED_TRACE_ROWS}")
    rows = []
    if len(v9_rows) == len(v10_rows):
        for ordinal, (v9, v10) in enumerate(zip(v9_rows, v10_rows, strict=True)):
            if v9["stderr_line"] >= v10["stderr_line"] or (
                ordinal + 1 < len(v9_rows) and v10["stderr_line"] >= v9_rows[ordinal + 1]["stderr_line"]
            ):
                failures["provenance"].append(f"ordinal {ordinal}: V9/V10 row order drift")
            row = {
                "ordinal": ordinal,
                **trace_position(ordinal),
                "stderr_lines": {"v9": v9["stderr_line"], "v10": v10["stderr_line"]},
                "v9": {key: value for key, value in v9.items() if key != "stderr_line"},
                **{key: value for key, value in v10.items() if key != "stderr_line"},
            }
            if v9["surfaces"] != v10["surfaces"] or v9["emitted"] != v10["emitted"]:
                failures["semantic"].append(f"ordinal {ordinal}: V9/V10 surface or emitted count mismatch")
            if (
                row["admission_calls"] != row["emitted"]
                or row["post_calls"] != row["emitted"]
                or sum(row["actions"].values()) != row["emitted"]
                or sum(row["reasons"].values()) != row["emitted"]
                or row["unknown_reasons"] != 0
            ):
                failures["semantic"].append(f"ordinal {ordinal}: admission/action/reason cardinality mismatch")
            if row["post_hits"] > row["post_calls"]:
                failures["semantic"].append(f"ordinal {ordinal}: post override hits exceed calls")
            if any(stage["hits"] > stage["calls"] for stage in row["stages"].values()):
                failures["semantic"].append(f"ordinal {ordinal}: stage hits exceed calls")
            elapsed_sum = sum(stage["elapsed_ns"] for stage in row["stages"].values())
            if row["leaf_ns"] != elapsed_sum or row["residual_ns"] != max(row["admission_ns"] - elapsed_sum, 0):
                failures["provenance"].append(f"ordinal {ordinal}: leaf/residual accounting drift")
            rows.append(row)
    else:
        failures["provenance"].append("V9 and V10 traces cannot be positionally joined")
    if len(rows) == EXPECTED_TRACE_ROWS:
        failures["semantic"].extend(four_round_mismatches(rows))
    summary = None
    if len(rows) == EXPECTED_TRACE_ROWS:
        try:
            summary = summarize_trace(rows, failures["semantic"])
        except BaseException as error:
            failures["provenance"].append(f"trace summary failed: {type(error).__name__}: {error}")
    return rows, failures, summary


def summed_named(rows: Sequence[Mapping[str, Any]], field: str, names: Sequence[str]) -> dict[str, int]:
    return {name: sum(integer(row[field][name], 0) for row in rows) for name in names}


def cohort_summary(rows: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    need(rows, "empty trace cohort")
    admission_values = [integer(row.get("admission_ns"), 0) for row in rows]
    leaf_values = [integer(row.get("leaf_ns"), 0) for row in rows]
    residual_values = [integer(row.get("residual_ns"), 0) for row in rows]
    post_values = [integer(row.get("post_ns"), 0) for row in rows]
    admission_total = sum(admission_values)
    leaf_total = sum(leaf_values)
    stage_rows = {}
    for name in EXPECTED_STAGE_NAMES:
        elapsed = [integer(row["stages"][name].get("elapsed_ns"), 0) for row in rows]
        elapsed_total = sum(elapsed)
        stage_rows[name] = {
            "calls": sum(integer(row["stages"][name].get("calls"), 0) for row in rows),
            "hits": sum(integer(row["stages"][name].get("hits"), 0) for row in rows),
            "elapsed_ns": distribution(elapsed),
            "share_of_admission": elapsed_total / admission_total if admission_total else None,
            "share_of_leaf": elapsed_total / leaf_total if leaf_total else None,
        }
    return {
        "rows": len(rows),
        "ordinals": [integer(row.get("ordinal")) for row in rows],
        "case_ordinals": [integer(row.get("case_ordinal")) for row in rows],
        "admission_ns": distribution(admission_values),
        "leaf_ns": distribution(leaf_values),
        "residual_ns": distribution(residual_values),
        "post_ns": distribution(post_values),
        "accounting": {
            "leaf_share_of_admission": leaf_total / admission_total if admission_total else None,
            "residual_share_of_admission": sum(residual_values) / admission_total if admission_total else None,
            "post_share_of_admission": sum(post_values) / admission_total if admission_total else None,
        },
        "surfaces": sum(integer(row.get("surfaces"), 0) for row in rows),
        "emitted": sum(integer(row.get("emitted"), 0) for row in rows),
        "admission_calls": sum(integer(row.get("admission_calls"), 0) for row in rows),
        "post_hits": sum(integer(row.get("post_hits"), 0) for row in rows),
        "actions": summed_named(rows, "actions", EXPECTED_ACTION_NAMES),
        "reasons": summed_named(rows, "reasons", EXPECTED_REASON_NAMES),
        "stages": stage_rows,
    }


def summarize_trace(rows: Sequence[Mapping[str, Any]], semantic_failures: Sequence[str]) -> dict[str, Any]:
    need(len(rows) == EXPECTED_TRACE_ROWS, "trace row count drift")
    warmup = list(rows[:WARMUP_ROWS])
    measured = list(rows[WARMUP_ROWS:])
    need(len(warmup) == WARMUP_ROWS and len(measured) == MEASURED_ROWS, "trace phase cardinality drift")
    rounds = []
    for round_index, schedule in enumerate(SCHEDULE, start=1):
        subset = [row for row in measured if row["round"] == round_index]
        need(len(subset) == WARMUP_ROWS, f"round {round_index} cardinality drift")
        rounds.append({"round": round_index, "schedule": schedule, "summary": cohort_summary(subset)})
    tail = [row for row in measured if row["case_ordinal"] in PREREGISTERED_TAIL_ORDINALS]
    need(len(tail) == 16, "preregistered tail cohort cardinality drift")
    top16 = sorted(measured, key=lambda row: (-integer(row.get("admission_ns"), 0), integer(row.get("ordinal"))))[:16]
    return {
        "schema": "lay.v10r1-admission-substage-trace.v1",
        "trace_rows": len(rows),
        "warmup_rows": len(warmup),
        "measured_rows": len(measured),
        "stage_registry": list(EXPECTED_STAGE_NAMES),
        "action_registry": list(EXPECTED_ACTION_NAMES),
        "reason_registry": list(EXPECTED_REASON_NAMES),
        "measured": cohort_summary(measured),
        "rounds": rounds,
        "preregistered_tail": cohort_summary(tail),
        "top16_admission_elapsed": cohort_summary(top16),
        "four_round_consistency": {
            "cases": WARMUP_ROWS,
            "mismatches": [failure for failure in semantic_failures if "four-round" in failure],
        },
        "claim_boundary": {
            "winner_threshold": None,
            "v8r3_latency_reinterpreted": False,
            "production_authority_admitted": False,
        },
    }


def expected_trace_environment() -> dict[str, str]:
    inputs = REMOTE_PARENT / "bootstrap-v1/inputs"
    subject = REMOTE_PARENT / "trace-v1/subject"
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(inputs / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_M3_ACTUAL_OWNER_V7": str(inputs / "slice8b-v7-fixed-13x100.json"),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LAY_L2_FIELD_TRACE": "1",
        "LAY_L2_PACKAGE": str(inputs / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m"),
        "LAY_L11_RECEIPT": str(inputs / "l11-installed.json"),
        "LAY_PROPOSAL_ADMISSION_TRACE": "1",
    }


def expected_trace_command() -> list[str]:
    environment = expected_trace_environment()
    return [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0",
        str(REMOTE_PARENT / "build-v1/v10-test-elf"),
        "--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]


def marker_payload(route: str, authority: Mapping[str, Any]) -> bytes:
    need(route in {"BUILD", "TRACE"}, "unknown marker route")
    return canonical({
        "schema": "lay.v10r1-one-shot-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "local_controller_sha256": authority["local_controller_sha256"],
        "remote_controller_sha256": authority["remote_controller_sha256"],
        "auditor_sha256": authority["auditor_sha256"],
        "one_shot": True,
        "retry_permitted": False,
    })


def dispatch_failures(failures: Mapping[str, Sequence[str]]) -> str:
    for verdict, key in (
        ("BLOCKED_PROVENANCE", "provenance"),
        ("BLOCKED_BUILD", "build"),
        ("BLOCKED_SEMANTIC", "semantic"),
        ("BLOCKED_CAPABILITY", "capability"),
    ):
        if failures.get(key):
            return verdict
    return "ADMISSION_SUBSTAGES_DECOMPOSED"


def terminal_decision(
    subject: Mapping[str, Any],
    wrapper: Mapping[str, Any],
    build: Mapping[str, Any],
    trace_rows: Sequence[Mapping[str, Any]],
    trace_failures: Mapping[str, Sequence[str]],
    trace_summary: Mapping[str, Any] | None,
) -> tuple[str, dict[str, list[str]]]:
    failures = {key: [] for key in ("provenance", "build", "semantic", "capability")}
    failures["provenance"].extend(str(value) for value in trace_failures.get("provenance", ()))
    failures["semantic"].extend(str(value) for value in trace_failures.get("semantic", ()))
    if wrapper.get("schema") != "lay.v10r1-trace-wrapper.v1" or wrapper.get("task_id") != TASK_ID or wrapper.get("transaction_id") != TRANSACTION_ID:
        failures["provenance"].append("producer wrapper namespace or schema drift")
    if wrapper.get("command") != expected_trace_command() or wrapper.get("environment") != expected_trace_environment():
        failures["provenance"].append("scientific command or environment drift")
    if wrapper.get("verdict") not in {"V10R1_TRACE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"}:
        failures["provenance"].append("producer wrapper verdict drift")
    if wrapper.get("controller_error") is not None:
        failures["capability"].append("subject launch or observation failed")
    if wrapper.get("outputs_complete") is not True or subject.get("schema") != "lay.m3-end-to-end-test-owner.v1":
        failures["capability"].append("complete scientific receipt unavailable")
    if wrapper.get("trace_lines") != {"v9_aggregate": EXPECTED_TRACE_ROWS, "v10_substage": EXPECTED_TRACE_ROWS}:
        failures["provenance"].append("producer trace-line count drift")
    if len(trace_rows) != EXPECTED_TRACE_ROWS or trace_summary is None:
        failures["provenance"].append("independent trace decomposition is incomplete")
    if build.get("verdict") != "V10R1_BUILD_AUDIT_PASS_TRACE_ADMITTED":
        failures["build"].append("build audit did not admit TRACE")
    source = subject.get("source", {})
    if source.get("v13_sha256") != EXPECTED_V13_SHA or source.get("v7_sha256") != EXPECTED_V7_SHA:
        failures["provenance"].append("fixed input identity drift")
    if source.get("productive_v90_sha256") != EXPECTED_PRODUCTIVE_SHA or source.get("l11_sha256") != EXPECTED_L11_SHA:
        failures["provenance"].append("actual-owner package tuple drift")
    if source.get("test_elf_sha256") != build.get("elf_sha256"):
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
    if integer(fixed.get("cases")) != 382 or integer(fixed.get("measured_rounds")) != 4 or integer(fixed.get("measured_samples")) != MEASURED_ROWS:
        failures["provenance"].append("request denominator drift")
    if fixed.get("schedule") != list(SCHEDULE):
        failures["provenance"].append("schedule drift")
    if integer(fixed.get("cpu")) != 0 or integer(fixed.get("cpu_mismatches")) != 0 or integer(fixed.get("warmup_cpu_mismatches")) != 0:
        failures["semantic"].append("CPU0 execution closure failed")
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
            failures["semantic"].append(f"reload evidence drift: {field}")
    if reload.get("held_a_survived_publication") is not True:
        failures["semantic"].append("held generation A did not survive publication")
    pss = subject.get("pss", {})
    if integer(pss.get("aggregate_delta_pss_kib"), 2**63) > 40 * 1024:
        failures["semantic"].append("aggregate two-process PSS delta exceeds 40 MiB")
    if integer(pss.get("typed_owned_bytes_per_process")) != 3_689_628:
        failures["semantic"].append("typed payload byte count drift")
    if integer(pss.get("sidecar_bytes"), 2**63) > 32 * 1024 * 1024 or integer(pss.get("helper_failures")) != 0:
        failures["semantic"].append("sidecar or PSS helper gate failed")
    gates = subject.get("gates", {})
    for field in ("semantic", "capacity", "reload_identity", "rss", "environment"):
        if gates.get(field) is not True:
            failures["semantic"].append(f"existing non-latency gate is not true: {field}")
    subject_verdict = subject.get("verdict")
    accepted_pair = (
        subject_verdict == "M3_END_TO_END_TEST_OWNER_PASS"
        and wrapper.get("exit_code") == 0
        and gates.get("latency") is True
    ) or (
        subject_verdict == "BLOCKED_LATENCY"
        and wrapper.get("exit_code") == 101
        and gates.get("latency") is False
    )
    if not accepted_pair and subject.get("schema") == "lay.m3-end-to-end-test-owner.v1":
        failures["capability"].append("subject verdict/exit/latency pair is unsupported")
    if wrapper.get("thermal_throttle_drift") not in ({}, None):
        failures["capability"].append("thermal throttle counters changed")
    if (
        wrapper.get("subject_executions") != 1
        or wrapper.get("cargo_invocations") != 0
        or wrapper.get("perf_record_invocations") != 0
        or wrapper.get("perf_stat_invocations") != 0
    ):
        failures["provenance"].append("TRACE execution ledger drift")
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
    return dispatch_failures(failures), failures


def terminal_audit() -> dict[str, Any]:
    fixed_local()
    admission = load_sealed_receipt(ADMISSION_ROOT, ADMISSION_RECEIPT, ("V10R1_EXECUTION_ADMITTED",))
    build = load_sealed_receipt(
        BUILD_AUDIT_ROOT,
        BUILD_AUDIT_RECEIPT,
        ("V10R1_BUILD_AUDIT_PASS_TRACE_ADMITTED",),
    )
    quiet = load_sealed_receipt(QUIET_ROOT, QUIET_RECEIPT, ("V10R1_QUIET_HOST_TRACE_ADMITTED",))
    snapshot = remote_snapshot()
    validate_host(snapshot)
    need(snapshot.get("markers") == {"available": [], "consumed": ["build.consumed-before-exec", "trace.consumed-before-exec"]}, "terminal marker state drift")
    need(snapshot.get("latest_state", {}).get("state") in {"TRACE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"}, "terminal state drift")
    need(not snapshot.get("conflicting_processes"), "owned or conflicting process active at terminal audit")
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10r1-terminal-"))
    try:
        remote_evidence = temporary / "REMOTE_TRACE"
        copy_remote(REMOTE_PARENT / "trace-v1", remote_evidence)
        manifest_entries = verify_manifest(remote_evidence)
        wrapper_path = remote_evidence / "TRACE_WRAPPER.json"
        stderr_path = remote_evidence / "stderr.log"
        wrapper = load_json(wrapper_path) if wrapper_path.is_file() else {}
        stderr = stderr_path.read_bytes() if stderr_path.is_file() else b""
        rows, trace_failures, summary = parse_trace(stderr)
        subject = wrapper.get("subject_receipt") if isinstance(wrapper.get("subject_receipt"), dict) else {}
        verdict, failures = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
        subject_path = remote_evidence / "subject/SUBJECT_RECEIPT.json"
        if subject_path.is_file():
            if load_json(subject_path) != subject:
                failures["provenance"].append("retained subject receipt disagrees with wrapper")
        elif subject:
            failures["provenance"].append("wrapper embeds a subject receipt absent from retained evidence")
        expected_marker = marker_payload("TRACE", quiet)
        marker = wrapper.get("marker", {}) if isinstance(wrapper.get("marker"), dict) else {}
        before_marker = marker.get("before", {}) if isinstance(marker.get("before"), dict) else {}
        after_marker = marker.get("after", {}) if isinstance(marker.get("after"), dict) else {}
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
            failures["provenance"].append("one-shot TRACE marker consumption evidence drift")
        if local_runtime_snapshot() != admission.get("local_runtime_before"):
            failures["provenance"].append("local runtime authority changed")
        if snapshot.get("runtime_projection") != admission.get("remote_runtime_before"):
            failures["provenance"].append("remote runtime authority changed")
        verdict = dispatch_failures(failures)
        derived = temporary / "DERIVED_TRACE"
        derived.mkdir(mode=0o700)
        rows_bytes = b"".join(canonical(row) for row in rows)
        write_new(derived / "TRACE_ROWS.jsonl", rows_bytes)
        write_new(derived / "TRACE_SUMMARY.json", canonical({
            "schema": "lay.v10r1-admission-substage-derived.v1",
            "parse_failures": trace_failures,
            "summary": summary,
        }))
        receipt = {
            "schema": "lay.v10r1-terminal-audit.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": verdict,
            "positive_verdict": "ADMISSION_SUBSTAGES_DECOMPOSED",
            "failure_priority": ["provenance", "build", "semantic", "capability", "complete_decomposition"],
            "failures": failures,
            "local_controller_sha256": build["local_controller_sha256"],
            "remote_controller_sha256": build["remote_controller_sha256"],
            "auditor_sha256": build["auditor_sha256"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "bootstrap_audit_sha256": sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
            "build_audit_sha256": sha256_file(BUILD_AUDIT_RECEIPT),
            "quiet_admission_sha256": sha256_file(QUIET_RECEIPT),
            "trace_wrapper_sha256": sha256_file(wrapper_path) if wrapper_path.is_file() else None,
            "stderr_sha256": sha256_file(stderr_path) if stderr_path.is_file() else None,
            "remote_manifest_entries": manifest_entries,
            "scientific_receipt": subject,
            "trace": summary,
            "trace_rows": len(rows),
            "v9_trace_rows": wrapper.get("trace_lines", {}).get("v9_aggregate"),
            "v10_trace_rows": wrapper.get("trace_lines", {}).get("v10_substage"),
            "warmup_rows": WARMUP_ROWS,
            "measured_rows": MEASURED_ROWS,
            "markers_created": 2,
            "markers_consumed": 2,
            "cargo_invocations": 1,
            "rustc_compilations": 1,
            "subject_executions": 1,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "installed_package_changed": False,
            "runtime_authority_changed": False,
            "production_authority_admitted": False,
            "v8r3_latency_reinterpreted": False,
            "next_if_pass": "separate paper mechanism decision from the measured substage evidence only",
            "live_projection": snapshot,
        }
        return publish_tree(
            TERMINAL_ROOT,
            "TERMINAL_AUDIT.json",
            receipt,
            {"REMOTE_TRACE": remote_evidence, "DERIVED_TRACE": derived},
        )
    finally:
        shutil.rmtree(temporary, ignore_errors=True)


def synthetic_trace() -> bytes:
    stages = ",".join(f"{name}:2:0:1" for name in EXPECTED_STAGE_NAMES)
    actions = ",".join(f"{name}:{2 if name == 'eligible' else 0}" for name in EXPECTED_ACTION_NAMES)
    reasons = ",".join(f"{name}:{2 if name == 'class_allows_apply' else 0}" for name in EXPECTED_REASON_NAMES)
    v9 = (
        "productive_v90_materialization_trace surfaces=2 emitted=2 setup_us=1 "
        "projection_us=1 classify_us=1 gate_us=1 evidence_us=1"
    )
    v10 = (
        "proposal_admission_substage_trace schema=v10-admission-substage-v1 "
        "surfaces=2 emitted=2 admission_calls=2 admission_ns=100 leaf_ns=36 residual_ns=64 "
        f"post_calls=2 post_hits=0 post_ns=10 stages={stages} actions={actions} "
        f"reasons={reasons} unknown_reasons=0"
    )
    return ((v9 + "\n" + v10 + "\n") * EXPECTED_TRACE_ROWS).encode()


def synthetic_subject() -> dict[str, Any]:
    return {
        "schema": "lay.m3-end-to-end-test-owner.v1",
        "verdict": "BLOCKED_LATENCY",
        "source": {
            "v13_sha256": EXPECTED_V13_SHA,
            "v7_sha256": EXPECTED_V7_SHA,
            "productive_v90_sha256": EXPECTED_PRODUCTIVE_SHA,
            "l11_sha256": EXPECTED_L11_SHA,
            "test_elf_sha256": "a" * 64,
        },
        "fixed_proof": {
            "cases": 382,
            "measured_rounds": 4,
            "measured_samples": MEASURED_ROWS,
            "schedule": list(SCHEDULE),
            "semantic": {**{key: 0 for key in SEMANTIC_FIELDS}, "capacity_failures": 0, "unresolved": 0},
            "empty_lane_mismatches": 0,
            "maximum_query_scratch_bytes": 1,
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
        "gates": {
            "semantic": True,
            "capacity": True,
            "reload_identity": True,
            "rss": True,
            "latency": False,
            "environment": True,
        },
        "claim_boundary": {
            "test_only_generation_owner": True,
            "production_authority_admitted": False,
            "runtime_reload_edit_admitted": False,
        },
        "runtime_authority_changed": False,
        "production_activation_admitted": False,
        "perf_or_pmu_used": False,
        "network_used_by_subject": False,
        "installed_package_changed": False,
    }


def self_check() -> dict[str, Any]:
    need(ACTIONS == ("self-check", "live-admission", "bootstrap", "build", "quiet", "terminal", "status"), "auditor registry drift")
    need(len(EXPECTED_STAGE_NAMES) == 36 and len(EXPECTED_ACTION_NAMES) == 4 and len(EXPECTED_REASON_NAMES) == 43, "trace registry drift")
    need(SEMANTIC_FIELDS[-1] == "semantic_total" and len(SEMANTIC_FIELDS) == 10, "semantic schema drift")
    need(len({ADMISSION_ROOT, BOOTSTRAP_AUDIT_ROOT, BUILD_AUDIT_ROOT, QUIET_ROOT, TERMINAL_ROOT}) == 5, "audit destination collision")
    need("/home/e/.cargo/bin/cargo" not in REMOTE_SNAPSHOT and "/home/e/.cargo/bin/rustc" not in REMOTE_SNAPSHOT, "root host snapshot reaches toolchain")
    need("'execution_uid':os.geteuid()" in REMOTE_TOOLCHAIN and "env=environment" in REMOTE_TOOLCHAIN, "controlled toolchain observer contract absent")
    rows, trace_failures, summary = parse_trace(synthetic_trace())
    need(not any(trace_failures.values()) and summary is not None, "synthetic trace parser failed")
    subject = synthetic_subject()
    wrapper = {
        "schema": "lay.v10r1-trace-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R1_TRACE_CREATED_UNAUDITED",
        "controller_error": None,
        "command": expected_trace_command(),
        "environment": expected_trace_environment(),
        "exit_code": 101,
        "outputs_complete": True,
        "trace_lines": {"v9_aggregate": EXPECTED_TRACE_ROWS, "v10_substage": EXPECTED_TRACE_ROWS},
        "thermal_throttle_drift": {},
        "subject_executions": 1,
        "cargo_invocations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
    }
    build = {"verdict": "V10R1_BUILD_AUDIT_PASS_TRACE_ADMITTED", "elf_sha256": "a" * 64}
    verdict, failures = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
    need(verdict == "ADMISSION_SUBSTAGES_DECOMPOSED" and not any(failures.values()), "positive dispatch model failed")
    subject["fixed_proof"]["semantic"]["candidate_mismatches"] = 1
    wrapper["controller_error"] = "synthetic capability failure"
    verdict, _ = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
    need(verdict == "BLOCKED_SEMANTIC", "failure-priority model failed")
    wrapper["command"] = []
    verdict, _ = terminal_decision(subject, wrapper, build, rows, trace_failures, summary)
    need(verdict == "BLOCKED_PROVENANCE", "provenance priority model failed")
    malformed_rows, malformed_failures, malformed_summary = parse_trace(b"proposal_admission_substage_trace broken\n")
    need(not malformed_rows and malformed_summary is None and malformed_failures["provenance"], "malformed trace did not fail closed")
    return {
        "schema": "lay.v10r1-independent-auditor-self-check.v1",
        "verdict": "V10R1_INDEPENDENT_AUDITOR_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "auditor_sha256": sha256_file(AUDITOR),
        "actions": list(ACTIONS),
        "trace_rows": len(rows),
        "stage_registry": len(EXPECTED_STAGE_NAMES),
        "action_registry": len(EXPECTED_ACTION_NAMES),
        "reason_registry": len(EXPECTED_REASON_NAMES),
        "positive_dispatch": "PASS",
        "failure_priority_dispatch": "PASS",
        "provenance_priority_dispatch": "PASS",
        "malformed_trace_fail_closed": "PASS",
        "root_snapshot_toolchain_queries": 0,
        "controlled_toolchain_queries": 2,
        "remote_writes": 0,
        "scientific_actions": 0,
    }


def status() -> dict[str, Any]:
    return {
        "schema": "lay.v10r1-auditor-status.v1",
        "verdict": "V10R1_AUDITOR_STATUS",
        "receipts": {
            "execution_admission": ADMISSION_RECEIPT.exists(),
            "bootstrap_audit": BOOTSTRAP_AUDIT_RECEIPT.exists(),
            "build_audit": BUILD_AUDIT_RECEIPT.exists(),
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
        elif args.action == "build":
            value = build_audit()
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
            "schema": "lay.v10r1-auditor-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
