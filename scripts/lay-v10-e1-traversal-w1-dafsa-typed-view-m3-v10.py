#!/usr/bin/env python3
"""Local fail-closed orchestrator for the V10 admission-substage diagnostic."""

from __future__ import annotations

import argparse
import ast
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
import tempfile
import time
from collections.abc import Callable, Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10-20260827"
TRANSACTION_ID = "8058be994305226e9af3fbdee2e6b29bd9111ffbf8203ef20db9feeb1ca56a22"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v10-{TRANSACTION_ID}"

CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10-remote.py"
AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10-audit.py"
V13_SOURCE = ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
LIVE_SOURCE = ROOT / "src/nanda_wave/l2_field/productive_v1/live.rs"
ADMISSION_SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"
DECISION_SOURCE = ROOT / "src/typing_transition/decision.rs"
V10_PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_2026-08-27.md"
CONTROLLER_PREFLIGHT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_IMPLEMENTATION_V1_PREFLIGHT_2026-08-27.json"
CONTROLLER_EVIDENCE = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_IMPLEMENTATION_V1_2026-08-27"
CONTROLLER_IMPLEMENTATION = CONTROLLER_EVIDENCE / "IMPLEMENTATION_RECEIPT.json"
EXECUTION_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_EXECUTION_JOURNAL_V1_2026-08-27"

ADMISSION_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_EXECUTION_ADMISSION_V1_2026-08-27/EXECUTION_ADMISSION.json"
BOOTSTRAP_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_BOOTSTRAP_AUDIT_V1_2026-08-27/BOOTSTRAP_AUDIT.json"
BUILD_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_BUILD_AUDIT_V1_2026-08-27/BUILD_AUDIT.json"
QUIET_ADMISSION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_QUIET_ADMISSION_V1_2026-08-27/QUIET_ADMISSION.json"
TERMINAL_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"

V13_PACKAGE = pathlib.Path("/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin")
V7_PROOF = pathlib.Path("/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json")
PRODUCTIVE = pathlib.Path("/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m")
RECOVERY = pathlib.Path("/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r")
L11 = pathlib.Path("/home/ubu/.local/share/lay/nanda_wave/l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin")
L11_PROOF = pathlib.Path("/home/ubu/.local/share/lay/nanda_wave/l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d.proof.json")

ACTIONS = ("self-check", "seal-self-check", "execute", "status")
EXTERNAL_ACTIONS = (
    "live-admission",
    "remote-cache-create",
    "bootstrap-upload",
    "remote-bootstrap",
    "bootstrap-audit",
    "bootstrap-audit-upload",
    "create-markers",
    "build-once",
    "build-audit",
    "build-audit-upload",
    "quiet-audit",
    "quiet-audit-upload",
    "trace-once",
    "terminal-audit",
)
OUTSIDE_SOURCE_FILES = (
    "data/lexicon/builtin_replacements.tsv",
    "data/lexicon/common_en_guard_prefixes.txt",
    "data/lexicon/common_en_technical.txt",
    "data/lexicon/common_ru.txt",
    "data/lexicon/l2_surface_hot_ru.txt",
    "data/lexicon/ru_greeting_words.txt",
    "data/lexicon/ru_hyphen_particles.txt",
    "data/lexicon/ru_keyboard_rows.txt",
    "data/lexicon/ru_live_protected_words.txt",
    "data/lexicon/ru_one_letter_function.txt",
    "data/lexicon/ru_short_function.txt",
    "data/lexicon/ru_short_prepositions.txt",
    "data/lexicon/ru_short_pronouns.txt",
    "data/lexicon/ru_single_letter_pronouns.txt",
    "data/lexicon/ru_technical_loanword_stems.txt",
    "data/lexicon/ru_technical_loanword_suffixes.txt",
    "data/lexicon/ru_technical_loanwords.txt",
    "data/lexicon/russian_adjective_form_suffixes.txt",
    "data/lexicon/russian_adjective_lemma_endings.txt",
    "data/lexicon/russian_contextual_fuzzy_pairs.tsv",
    "data/lexicon/russian_derivational_prefixes.txt",
    "data/lexicon/russian_glued_phrase_part_fixes.tsv",
    "data/lexicon/russian_incomplete_reflexive_parts.txt",
    "data/lexicon/russian_ka_oblique_suffixes.txt",
    "data/lexicon/russian_past_tense_endings.txt",
    "data/lexicon/russian_possessive_suffixes.txt",
    "data/lexicon/russian_present_or_reflexive_endings.txt",
    "data/lexicon/russian_reflexive_confusion.tsv",
    "data/lexicon/russian_suffix_forms.txt",
    "data/lexicon/russian_verb_ending_confusion.tsv",
    "data/lexicon/russian_verb_form_endings.tsv",
    "data/lexicon/russian_zero_noun_suffixes.txt",
    "data/lexicon/visual_b_after_ascii.txt",
    "data/lexicon/visual_b_default.txt",
    "data/morphology/russian_noun_cases_small.tsv",
    "data/nanda_llmwave_seed_phrases.txt",
    "scripts/install-l11-shadow-package.sh",
    "tests/fixtures/daemon_typing_assist_context_window.tsv",
    "tests/fixtures/decoder_context_visual_b.tsv",
    "tests/fixtures/decoder_transition_manual_replace.tsv",
    "tests/fixtures/decoder_transition_manual_replay.tsv",
    "tests/fixtures/decoder_transition_visual_b.tsv",
    "tests/fixtures/lexicon_protected_ascii_expected.txt",
    "tests/fixtures/lexicon_protected_ascii_rejected.txt",
    "tests/fixtures/lexicon_protected_ascii_source.txt",
    "tests/fixtures/ngram_ru_train_words.txt",
    "tests/fixtures/replacement_rules.tsv",
    "tests/fixtures/russian_forms.txt",
    "tests/fixtures/typing_context_disabled.txt",
    "tests/fixtures/typing_context_enabled.txt",
    "tests/fixtures/word_reader_split_reject.txt",
)


class V10ControllerError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V10ControllerError(message)


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
        raise V10ControllerError(
            f"command failed ({result.returncode}): {list(argv)!r}\n{result.stderr[-4000:].decode(errors='replace')}"
        )
    return result


def ssh(argv: Sequence[str], *, timeout: float = 3_600) -> bytes:
    command = shlex.join(list(argv))
    return run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            command,
        ],
        timeout=timeout,
    ).stdout


def scp_file(local: pathlib.Path, remote: pathlib.PurePosixPath) -> None:
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
            str(local),
            f"{REMOTE}:{remote}",
        ],
        timeout=3_600,
    )


def parse_json_output(result: subprocess.CompletedProcess[bytes], label: str) -> dict[str, Any]:
    lines = result.stdout.decode().strip().splitlines()
    need(lines, f"{label} returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), f"{label} response is not an object")
    return value


def auditor_call(action: str) -> dict[str, Any]:
    return parse_json_output(run([str(AUDITOR), action], timeout=10_800), f"auditor {action}")


def remote_controller_path(cache: bool = False) -> pathlib.PurePosixPath:
    return (REMOTE_CACHE if cache else REMOTE_PARENT / "bootstrap-v1") / "remote-controller.py"


def remote_call(action: str, *arguments: str, cache: bool = False, timeout: float = 3_600) -> dict[str, Any]:
    raw = ssh(
        ["/usr/bin/sudo", "-n", "/usr/bin/python3", str(remote_controller_path(cache)), action, *arguments],
        timeout=timeout,
    )
    lines = raw.decode().strip().splitlines()
    need(lines, f"remote {action} returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), f"remote {action} response is not an object")
    return value


def verify_controller_preflight() -> dict[str, Any]:
    value = load_json(CONTROLLER_PREFLIGHT)
    need(value.get("verdict") == "READY_TO_IMPLEMENT" and value.get("safe_to_implement") is True, "controller implementation preflight not ready")
    need(value.get("manifest_sha256") == "ff57e6d0c65bb93fe5526c04eacb4c82606a9dd9d0eaef72f615538899ba54a0", "controller preflight manifest identity drift")
    return value


def verify_fixed_inputs() -> dict[str, Any]:
    expected = {
        V10_PAPER: (14_154, "afb4709efd22a63119527d21d126629a08a811e2b4e73a68c211144df946bd27"),
        V13_SOURCE: (253_080, "28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2"),
        LIVE_SOURCE: (71_012, "36aeddd5e605e67377f99343f9937606ac774d9ba4bb5710152de060bc9d183b"),
        ADMISSION_SOURCE: (87_732, "b563f8a9400f9ca61d1d4bf4f31c06b146b30d6f2d78cd364e4d69636aafd3e3"),
        DECISION_SOURCE: (45_197, "ad3c6d450c01811844a49e9c714d0eb9ff80f7de7d2f03a2e8b3e290deda3691"),
        V13_PACKAGE: (140_556_462, "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"),
        V7_PROOF: (1_606_189, "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"),
        PRODUCTIVE: (17_309_944, "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"),
        RECOVERY: (2_123_112, "de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e"),
        L11: (77_962_328, "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"),
        L11_PROOF: (539_536, "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d"),
    }
    rows = {}
    for path, (size, digest) in expected.items():
        row = file_row(path)
        need(row["size_bytes"] == size and row["sha256"] == digest, f"fixed input drift: {path}")
        rows[path.name] = row
    return rows


def parse_python(path: pathlib.Path) -> ast.Module:
    source = path.read_text()
    compile(source, str(path), "exec")
    return ast.parse(source, filename=str(path))


def python_literal(tree: ast.Module, name: str) -> Any:
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
            return ast.literal_eval(node.value)
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name) and node.target.id == name:
            return ast.literal_eval(node.value)
    raise V10ControllerError(f"Python registry absent: {name}")


def rust_string_array(source: str, name: str) -> tuple[str, ...]:
    match = re.search(rf"const {re.escape(name)}[^=]*= \[(.*?)\];", source, re.DOTALL)
    need(match is not None, f"Rust registry absent: {name}")
    return tuple(re.findall(r'"([^"]+)"', match.group(1)))


def static_graph() -> dict[str, Any]:
    trees = {path.name: parse_python(path) for path in (CONTROLLER, REMOTE_CONTROLLER, AUDITOR)}
    remote_source = REMOTE_CONTROLLER.read_text()
    need("ACTIONS = (" in remote_source and "ROUTES = (\"BUILD\", \"TRACE\")" in remote_source, "remote registry source drift")
    remote_tree = trees[REMOTE_CONTROLLER.name]
    route_functions = {
        node.name: node
        for node in remote_tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name in {"build_once", "trace_once"}
    }
    need(set(route_functions) == {"build_once", "trace_once"}, "remote route functions absent")
    route_literals = {
        name: {
            node.value
            for node in ast.walk(function)
            if isinstance(node, ast.Constant) and isinstance(node.value, str)
        }
        for name, function in route_functions.items()
    }
    forbidden_exec_literals = {"perf", "/usr/bin/perf", "perf record", "perf stat", "--pid", "SIGINT"}
    need(
        all(not (literals & forbidden_exec_literals) for literals in route_literals.values()),
        "perf attach or SIGINT route is reachable",
    )
    need("scripts/cargo-guard.sh" in route_literals["build_once"], "build route lacks cargo guard")
    need("scripts/cargo-guard.sh" not in route_literals["trace_once"] and "cargo" not in route_literals["trace_once"], "Cargo is reachable from TRACE")
    need("cargo-guard.sh" in remote_source and "m3_end_to_end_physical_proof" in remote_source, "build or TRACE route absent")
    need("ld-linux" not in remote_source, "TRACE is not a direct ELF execution")
    need("candidate.chmod(0o555)" in remote_source and 'str(BUILD / "v10-test-elf")' in remote_source, "direct executable lifecycle absent")
    local_source = CONTROLLER.read_text()
    need(all(f'"{action}"' in local_source for action in EXTERNAL_ACTIONS), "local external action registry incomplete")
    auditor_source = AUDITOR.read_text()
    need("terminal_decision" in auditor_source and "ADMISSION_SUBSTAGES_DECOMPOSED" in auditor_source, "terminal dispatch absent")
    admission_source = ADMISSION_SOURCE.read_text()
    live_source = LIVE_SOURCE.read_text()
    need(len(re.findall(r"\nfn candidate_admission\(", admission_source)) == 1, "candidate_admission implementation count drift")
    need(len(re.findall(r"admission_trace_(?:bool|value)!\(", admission_source)) == 36, "timed predicate count drift")
    need(
        "#[cfg(not(test))]\nmacro_rules! admission_trace_bool" in admission_source
        and "#[cfg(not(test))]\nmacro_rules! admission_trace_value" in admission_source,
        "non-test observer erasure macros absent",
    )
    need(
        all(
            token in live_source
            for token in (
                "#[cfg(test)]\n    let admission_trace_session",
                "#[cfg(test)]\n        let post_override_started",
                "#[cfg(test)]\n        if let Some(started) = post_override_started",
                "#[cfg(test)]\n    let admission_trace_line",
            )
        ),
        "live observer is not fully test-gated",
    )
    auditor_tree = trees[AUDITOR.name]
    rust_stages = rust_string_array(admission_source, "ADMISSION_TRACE_STAGE_NAMES")
    rust_reasons = rust_string_array(admission_source, "ADMISSION_TRACE_REASON_NAMES")
    need(rust_stages == tuple(python_literal(auditor_tree, "EXPECTED_STAGE_NAMES")), "stage registry parity drift")
    need(rust_reasons == tuple(python_literal(auditor_tree, "EXPECTED_REASON_NAMES")), "reason registry parity drift")
    need(tuple(python_literal(auditor_tree, "EXPECTED_ACTION_NAMES")) == ("eligible", "suggest_only", "keep_original", "veto"), "action registry parity drift")
    remote_source_contract = python_literal(remote_tree, "EXPECTED_SOURCE")
    for relative, (size, digest) in remote_source_contract.items():
        row = file_row(ROOT / relative)
        need(
            row["size_bytes"] == size and row["sha256"] == digest,
            f"remote source contract does not match local source: {relative}",
        )
    return {
        "compiled": sorted(trees),
        "remote_actions": ["self-check", "status", "bootstrap", "create-markers", "build-once", "trace-once"],
        "routes": ["BUILD", "TRACE"],
        "external_actions": list(EXTERNAL_ACTIONS),
        "perf_reachable": False,
        "production_runtime_edit_reachable": False,
        "direct_elf_execution": True,
        "candidate_admission_implementations": 1,
        "timed_predicates": len(rust_stages),
        "reason_registry": len(rust_reasons),
        "remote_source_contract_files": len(remote_source_contract),
        "test_only_erasure": True,
    }


def fault_model() -> dict[str, Any]:
    rows = {}
    for index, action in enumerate(EXTERNAL_ACTIONS, 1):
        intents = [{"sequence": index, "action": action, "status": "INTENT_DURABLE"}]
        completions: list[dict[str, Any]] = []
        pending = len(intents) != len(completions)
        rows[action] = {
            "intent_durable": True,
            "completion_absent": True,
            "pending_blocks_next_action": pending,
            "affected_facts": "UNKNOWN",
            "retry_permitted": False,
        }
        need(pending, f"fault model failed: {action}")
    return {"cases": rows, "cases_passed": len(rows), "cases_expected": len(EXTERNAL_ACTIONS)}


def self_check() -> dict[str, Any]:
    preflight = verify_controller_preflight()
    inputs = verify_fixed_inputs()
    graph = static_graph()
    remote = parse_json_output(run([str(REMOTE_CONTROLLER), "self-check"]), "remote static self-check")
    auditor = parse_json_output(run([str(AUDITOR), "self-check"]), "auditor static self-check")
    need(remote.get("verdict") == "V10_REMOTE_CONTROLLER_STATIC_PASS", "remote self-check failed")
    need(auditor.get("verdict") == "V10_INDEPENDENT_AUDITOR_STATIC_PASS", "auditor self-check failed")
    faults = fault_model()
    source_closure = source_closure_snapshot()
    return {
        "schema": "lay.v10-controller-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10_CONTROLLER_STATIC_SELF_CHECK_PASS",
        "local_controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "controller_preflight_sha256": sha256_file(CONTROLLER_PREFLIGHT),
        "controller_preflight_manifest_sha256": preflight["manifest_sha256"],
        "fixed_inputs": inputs,
        "command_graph": graph,
        "remote_self_check": remote,
        "auditor_self_check": auditor,
        "fault_injection": faults,
        "source_closure": source_closure,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
    }


def seal_self_check() -> dict[str, Any]:
    need(not CONTROLLER_EVIDENCE.exists(), "controller implementation evidence already exists")
    for path in (CONTROLLER, REMOTE_CONTROLLER, AUDITOR):
        need(mode_string(path) == "0555", f"controller source is not sealed executable: {path}")
    check = self_check()
    inputs = check["fixed_inputs"]
    receipt = {
        "schema": "lay.v10-controller-implementation.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10_REMOTE_CONTROLLERS_VERIFIED_UNRUN",
        "local_controller_sha256": check["local_controller_sha256"],
        "remote_controller_sha256": check["remote_controller_sha256"],
        "auditor_sha256": check["auditor_sha256"],
        "controller_preflight_sha256": check["controller_preflight_sha256"],
        "source_files": {
            "v13_typed_peak.rs": inputs[V13_SOURCE.name],
            "live.rs": inputs[LIVE_SOURCE.name],
            "proposal_admission.rs": inputs[ADMISSION_SOURCE.name],
            "decision.rs": inputs[DECISION_SOURCE.name],
        },
        "source_closure": check["source_closure"],
        "self_check": check,
        "execution_admission_present": False,
        "journal_present": False,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "production_authority_admitted": False,
        "next_action_admitted": "independent live execution admission only",
    }
    stage = CONTROLLER_EVIDENCE.with_name(f"{CONTROLLER_EVIDENCE.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / "SELF_CHECK.json", canonical(check))
        write_new(stage / "IMPLEMENTATION_RECEIPT.json", canonical(receipt))
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, CONTROLLER_EVIDENCE)
        fsync_dir(CONTROLLER_EVIDENCE.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return load_json(CONTROLLER_IMPLEMENTATION)


def copy_source_file(source: pathlib.Path, destination: pathlib.Path) -> None:
    need(source.is_file() and not source.is_symlink(), f"source closure file absent: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def source_closure_paths() -> list[pathlib.Path]:
    paths = {
        path
        for root in (ROOT / "src", ROOT / "data/test_input")
        for path in root.rglob("*")
        if path.is_file()
    }
    paths.update(ROOT / relative for relative in OUTSIDE_SOURCE_FILES)
    paths.update(ROOT / relative for relative in ("Cargo.toml", "Cargo.lock", "build.rs", "scripts/cargo-guard.sh"))
    for path in paths:
        need(path.is_file() and not path.is_symlink(), f"source closure file absent or symbolic: {path}")
    return sorted(paths)


def source_content_identity(rows: Mapping[str, Mapping[str, Any]]) -> str:
    content = [
        {"path": relative, "size_bytes": int(row["size_bytes"]), "sha256": str(row["sha256"])}
        for relative, row in sorted(rows.items())
    ]
    return sha256_bytes(canonical(content))


def source_closure_snapshot() -> dict[str, Any]:
    rows = {
        path.relative_to(ROOT).as_posix(): {
            "size_bytes": path.stat().st_size,
            "sha256": sha256_file(path),
        }
        for path in source_closure_paths()
    }
    need(len(rows) >= 700, "source closure unexpectedly small")
    return {
        "files_count": len(rows),
        "bytes": sum(int(row["size_bytes"]) for row in rows.values()),
        "content_sha256": source_content_identity(rows),
    }


def build_source_closure(destination: pathlib.Path) -> dict[str, Any]:
    destination.mkdir(mode=0o700)
    shutil.copytree(ROOT / "src", destination / "src")
    shutil.copytree(ROOT / "data/test_input", destination / "data/test_input")
    for relative in OUTSIDE_SOURCE_FILES:
        copy_source_file(ROOT / relative, destination / relative)
    for relative in ("Cargo.toml", "Cargo.lock", "build.rs", "scripts/cargo-guard.sh"):
        copy_source_file(ROOT / relative, destination / relative)
    (destination / "scripts/cargo-guard.sh").chmod(0o775)
    rows = {}
    for path in sorted(destination.rglob("*")):
        if path.is_file() and path.name != "SOURCE_MANIFEST.json":
            relative = path.relative_to(destination).as_posix()
            rows[relative] = {
                "size_bytes": path.stat().st_size,
                "sha256": sha256_file(path),
                "mode": mode_string(path),
            }
    need(len(rows) >= 700, "source closure unexpectedly small")
    manifest = {
        "schema": "lay.v10-source-closure.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "files": rows,
        "files_count": len(rows),
        "bytes": sum(row["size_bytes"] for row in rows.values()),
        "content_sha256": source_content_identity(rows),
    }
    write_new(destination / "SOURCE_MANIFEST.json", canonical(manifest), 0o444)
    return manifest


def prepare_bootstrap(admission: Mapping[str, Any]) -> pathlib.Path:
    staging = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10-bootstrap-"))
    try:
        source = build_source_closure(staging / "source-closure")
        implementation = load_json(CONTROLLER_IMPLEMENTATION)
        need(
            {
                "files_count": source["files_count"],
                "bytes": source["bytes"],
                "content_sha256": source["content_sha256"],
            }
            == implementation.get("source_closure"),
            "source closure changed after implementation seal",
        )
        inputs = staging / "inputs"
        inputs.mkdir(mode=0o700)
        files = {
            inputs / "LAY-L2-RU-FULL-v13.bin": V13_PACKAGE,
            inputs / "slice8b-v7-fixed-13x100.json": V7_PROOF,
            inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m": PRODUCTIVE,
            inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r": RECOVERY,
            inputs / "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin": L11,
            inputs / "l11-proof.json": L11_PROOF,
            staging / "local-controller.py": CONTROLLER,
            staging / "remote-controller.py": REMOTE_CONTROLLER,
            staging / "independent-auditor.py": AUDITOR,
            staging / "CONTROLLER_IMPLEMENTATION.json": CONTROLLER_IMPLEMENTATION,
            staging / "EXECUTION_ADMISSION.json": ADMISSION_RECEIPT,
        }
        for target, source_path in files.items():
            shutil.copyfile(source_path, target)
        (staging / "admissions").mkdir(mode=0o700)
        inventory = {}
        for path in sorted(staging.rglob("*")):
            if path.is_file() and path.name != "PAYLOAD.json":
                inventory[path.relative_to(staging).as_posix()] = {
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
        payload = {
            "schema": "lay.v10-bootstrap-payload.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "execution_admission_verdict": admission["verdict"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "source_files": source["files_count"],
            "source_bytes": source["bytes"],
            "source_content_sha256": source["content_sha256"],
            "files": inventory,
        }
        write_new(staging / "PAYLOAD.json", canonical(payload), 0o444)
        return staging
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def initialize_journal() -> pathlib.Path:
    need(not EXECUTION_JOURNAL.exists(), "V10 execution journal already exists; retry is forbidden")
    EXECUTION_JOURNAL.mkdir(parents=True, mode=0o700)
    metadata = {
        "schema": "lay.v10-execution-journal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "external_actions": list(EXTERNAL_ACTIONS),
        "retry_permitted": False,
    }
    write_new(EXECUTION_JOURNAL / "JOURNAL.json", canonical(metadata), 0o444)
    fsync_dir(EXECUTION_JOURNAL)
    return EXECUTION_JOURNAL


def journal_rows(root: pathlib.Path, suffix: str) -> list[pathlib.Path]:
    return sorted(root.glob(f"[0-9][0-9]-*.{suffix}.json"))


def pending_intent(root: pathlib.Path) -> bool:
    return len(journal_rows(root, "intent")) != len(journal_rows(root, "complete"))


def append_intent(root: pathlib.Path, sequence: int, action: str) -> pathlib.Path:
    need(not pending_intent(root), "journal has a pending external action; retry is forbidden")
    path = root / f"{sequence:02d}-{action}.intent.json"
    write_new(path, canonical({
        "schema": "lay.v10-external-intent.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action": action,
        "status": "INTENT_DURABLE",
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(root)
    return path


def append_completion(root: pathlib.Path, sequence: int, action: str, response: Mapping[str, Any]) -> pathlib.Path:
    path = root / f"{sequence:02d}-{action}.complete.json"
    write_new(path, canonical({
        "schema": "lay.v10-external-completion.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action": action,
        "status": "RESPONSE_VERIFIED",
        "response_verdict": response.get("verdict"),
        "response_sha256": sha256_bytes(canonical(response)),
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(root)
    return path


def journaled(
    root: pathlib.Path,
    sequence: int,
    action: str,
    callback: Callable[[], Mapping[str, Any]],
    allowed: Sequence[str],
) -> dict[str, Any]:
    append_intent(root, sequence, action)
    response = dict(callback())
    need(response.get("verdict") in set(allowed), f"external action verdict drift: {action}: {response.get('verdict')}")
    append_completion(root, sequence, action, response)
    return response


def create_remote_cache() -> dict[str, Any]:
    output = ssh(["/usr/bin/mkdir", "-m", "0700", str(REMOTE_CACHE)])
    return {"verdict": "V10_REMOTE_CACHE_CREATED", "stdout_sha256": sha256_bytes(output)}


def upload_bootstrap(staging: pathlib.Path) -> dict[str, Any]:
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
            f"{staging}/.",
            f"{REMOTE}:{REMOTE_CACHE}",
        ],
        timeout=3_600,
    )
    return {"verdict": "V10_BOOTSTRAP_UPLOADED", "payload_sha256": sha256_file(staging / "PAYLOAD.json")}


def upload_audit(local: pathlib.Path, name: str, verdict: str) -> dict[str, Any]:
    remote = REMOTE_CACHE / "admissions" / name
    scp_file(local, remote)
    return {"verdict": verdict, "local_sha256": sha256_file(local), "remote_path": str(remote)}


def verify_implementation() -> dict[str, Any]:
    need(CONTROLLER_IMPLEMENTATION.is_file(), "controller implementation receipt absent")
    need(mode_string(CONTROLLER_EVIDENCE) == "0555" and mode_string(CONTROLLER_IMPLEMENTATION) == "0444", "controller implementation is not immutable")
    verify_manifest(CONTROLLER_EVIDENCE)
    value = load_json(CONTROLLER_IMPLEMENTATION)
    need(value.get("verdict") == "V10_REMOTE_CONTROLLERS_VERIFIED_UNRUN", "controller implementation verdict drift")
    for key, path in {
        "local_controller_sha256": CONTROLLER,
        "remote_controller_sha256": REMOTE_CONTROLLER,
        "auditor_sha256": AUDITOR,
    }.items():
        need(value.get(key) == sha256_file(path), f"controller implementation identity drift: {key}")
    need(value.get("source_closure") == source_closure_snapshot(), "sealed source closure identity drift")
    return value


def finish_journal(journal: pathlib.Path, verdict: str, *, terminal_audit: pathlib.Path | None = None) -> None:
    terminal = {
        "schema": "lay.v10-controller-terminal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "terminal_audit_sha256": sha256_file(terminal_audit) if terminal_audit is not None else None,
        "external_actions_completed": len(journal_rows(journal, "complete")),
        "retry_permitted": False,
        "runtime_authority_changed": False,
    }
    write_new(journal / "TERMINAL.json", canonical(terminal), 0o444)
    write_manifest(journal)
    seal_tree(journal)


def execute() -> dict[str, Any]:
    verify_implementation()
    journal = initialize_journal()
    sequence = 1
    staging: pathlib.Path | None = None
    try:
        admission = journaled(journal, sequence, "live-admission", lambda: auditor_call("live-admission"), ["V10_EXECUTION_ADMITTED"])
        sequence += 1
        staging = prepare_bootstrap(admission)
        journaled(journal, sequence, "remote-cache-create", create_remote_cache, ["V10_REMOTE_CACHE_CREATED"])
        sequence += 1
        journaled(journal, sequence, "bootstrap-upload", lambda: upload_bootstrap(staging), ["V10_BOOTSTRAP_UPLOADED"])
        sequence += 1
        journaled(
            journal,
            sequence,
            "remote-bootstrap",
            lambda: remote_call("bootstrap", "--bootstrap", str(REMOTE_CACHE), cache=True, timeout=3_600),
            ["V10_BOOTSTRAP_CREATED_UNAUDITED"],
        )
        sequence += 1
        journaled(journal, sequence, "bootstrap-audit", lambda: auditor_call("bootstrap"), ["V10_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED"])
        sequence += 1
        journaled(
            journal,
            sequence,
            "bootstrap-audit-upload",
            lambda: upload_audit(BOOTSTRAP_AUDIT, "BOOTSTRAP_AUDIT.json", "V10_BOOTSTRAP_AUDIT_UPLOADED"),
            ["V10_BOOTSTRAP_AUDIT_UPLOADED"],
        )
        sequence += 1
        audit_remote = str(REMOTE_CACHE / "admissions/BOOTSTRAP_AUDIT.json")
        journaled(
            journal,
            sequence,
            "create-markers",
            lambda: remote_call("create-markers", "--audit", audit_remote),
            ["V10_ALL_MARKERS_AVAILABLE"],
        )
        sequence += 1
        build = journaled(
            journal,
            sequence,
            "build-once",
            lambda: remote_call("build-once", "--audit", audit_remote, timeout=10_800),
            ["V10_BUILD_CREATED_UNAUDITED", "BLOCKED_BUILD"],
        )
        sequence += 1
        build_audit = journaled(
            journal,
            sequence,
            "build-audit",
            lambda: auditor_call("build"),
            ["V10_BUILD_AUDIT_PASS_TRACE_ADMITTED", "BLOCKED_BUILD"],
        )
        sequence += 1
        if build.get("verdict") == "BLOCKED_BUILD" or build_audit.get("verdict") == "BLOCKED_BUILD":
            need(build.get("verdict") == build_audit.get("verdict"), "producer and auditor build verdicts disagree")
            finish_journal(journal, "BLOCKED_BUILD")
            return build_audit
        need(build.get("verdict") == "V10_BUILD_CREATED_UNAUDITED", "build producer verdict drift")
        journaled(
            journal,
            sequence,
            "build-audit-upload",
            lambda: upload_audit(BUILD_AUDIT, "BUILD_AUDIT.json", "V10_BUILD_AUDIT_UPLOADED"),
            ["V10_BUILD_AUDIT_UPLOADED"],
        )
        sequence += 1
        journaled(journal, sequence, "quiet-audit", lambda: auditor_call("quiet"), ["V10_QUIET_HOST_TRACE_ADMITTED"])
        sequence += 1
        journaled(
            journal,
            sequence,
            "quiet-audit-upload",
            lambda: upload_audit(QUIET_ADMISSION, "QUIET_ADMISSION.json", "V10_QUIET_ADMISSION_UPLOADED"),
            ["V10_QUIET_ADMISSION_UPLOADED"],
        )
        sequence += 1
        build_remote = str(REMOTE_CACHE / "admissions/BUILD_AUDIT.json")
        quiet_remote = str(REMOTE_CACHE / "admissions/QUIET_ADMISSION.json")
        journaled(
            journal,
            sequence,
            "trace-once",
            lambda: remote_call("trace-once", "--build-audit", build_remote, "--quiet", quiet_remote, timeout=10_800),
            ["V10_TRACE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"],
        )
        sequence += 1
        terminal = journaled(
            journal,
            sequence,
            "terminal-audit",
            lambda: auditor_call("terminal"),
            [
                "ADMISSION_SUBSTAGES_DECOMPOSED",
                "BLOCKED_PROVENANCE",
                "BLOCKED_BUILD",
                "BLOCKED_SEMANTIC",
                "BLOCKED_CAPABILITY",
            ],
        )
        finish_journal(journal, terminal["verdict"], terminal_audit=TERMINAL_AUDIT)
        return terminal
    except BaseException as error:
        failure = {
            "schema": "lay.v10-controller-failure.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "pending_intent": pending_intent(journal),
            "external_actions_completed": len(journal_rows(journal, "complete")),
            "affected_remote_facts": "UNKNOWN",
            "retry_permitted": False,
            "runtime_authority_changed": False,
        }
        write_new(journal / "CONTROLLER_FAILURE.json", canonical(failure), 0o444)
        write_manifest(journal)
        seal_tree(journal)
        raise V10ControllerError(json.dumps(failure, sort_keys=True)) from error
    finally:
        if staging is not None:
            shutil.rmtree(staging, ignore_errors=True)


def status() -> dict[str, Any]:
    remote = None
    try:
        path = remote_controller_path(cache=not (ADMISSION_RECEIPT.exists() and BOOTSTRAP_AUDIT.exists()))
        raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(path), "status"], timeout=60)
        lines = raw.decode().strip().splitlines()
        remote = json.loads(lines[-1]) if lines else None
    except BaseException as error:
        remote = {"verdict": "UNKNOWN", "error": f"{type(error).__name__}: {error}"}
    return {
        "schema": "lay.v10-controller-status.v1",
        "verdict": "V10_CONTROLLER_STATUS",
        "controller_implementation": CONTROLLER_IMPLEMENTATION.exists(),
        "execution_journal": EXECUTION_JOURNAL.exists(),
        "receipts": {
            "admission": ADMISSION_RECEIPT.exists(),
            "bootstrap": BOOTSTRAP_AUDIT.exists(),
            "build": BUILD_AUDIT.exists(),
            "quiet": QUIET_ADMISSION.exists(),
            "terminal": TERMINAL_AUDIT.exists(),
        },
        "remote": remote,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=ACTIONS)
    args = parser.parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "seal-self-check":
            value = seal_self_check()
        elif args.action == "execute":
            value = execute()
        else:
            value = status()
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v10-controller-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
