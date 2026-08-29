"""Discover Cargo test executables and their complete harness test sets."""

from __future__ import annotations

import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import tempfile
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[2]
SCHEMA = "lay.test-lanes.v1"
CARGO_ARGS = ["test", "--locked", "--offline", "--all-targets", "--no-run"]

PACKAGE_TARGETS = {
    "test:decoder_alternating_stress",
    "test:l3_context_feedback_overlay_contract",
    "test:lexical_phase_compiler",
    "test:protected_words_runtime",
    "test:restore_word_cli",
    "test:typing_assist_alternating_stress",
    "test:typing_assist_mixed_corpus",
    "test:typing_assist_short_alternating",
}

PACKAGE_FIXTURE_PATHS = (
    "data/lexicon/l3_context_phase_v1.nwpc",
    "tests/fixtures/decoder_alternating_stress.tsv",
    "tests/fixtures/typing_assist_alternating.tsv",
    "tests/fixtures/typing_assist_alternating_stress.tsv",
    "tests/fixtures/typing_assist_beta_alternating.tsv",
    "tests/fixtures/typing_assist_clean_mixed.txt",
    "tests/fixtures/typing_assist_cli_commands.txt",
    "tests/fixtures/typing_assist_confident_en.txt",
    "tests/fixtures/typing_assist_dynamic_tail.tsv",
    "tests/fixtures/typing_assist_dynamic_tail_none.txt",
    "tests/fixtures/typing_assist_forbidden_fragments.txt",
    "tests/fixtures/typing_assist_forum_mixed.tsv",
    "tests/fixtures/typing_assist_full_opposite_ru.txt",
    "tests/fixtures/typing_assist_function_boundaries.txt",
    "tests/fixtures/typing_assist_live_spacing.tsv",
    "tests/fixtures/typing_assist_mixed_matrix_layout_words.tsv",
    "tests/fixtures/typing_assist_mixed_matrix_prefixes.txt",
    "tests/fixtures/typing_assist_mixed_matrix_terms.txt",
    "tests/fixtures/typing_assist_normal_reject.txt",
    "tests/fixtures/typing_assist_normal_safe.tsv",
    "tests/fixtures/typing_assist_policy_cases.tsv",
    "tests/fixtures/typing_assist_ru_to_en_synthetic.txt",
    "tests/fixtures/typing_assist_shell_keep.txt",
    "tests/fixtures/typing_context_enabled.txt",
)

PROCESS_ISOLATED_TESTS = {
    (
        "lib:lay",
        "action_log::tests::action_log_is_disabled_by_default_and_enabled_by_config",
    ),
    (
        "lib:lay",
        "action_log::tests::action_log_writes_candidate_before_apply_mutation_route",
    ),
    (
        "lib:lay",
        "action_log::tests::action_log_writes_dirty_task_for_applied_gate",
    ),
    (
        "bin:lay-ibus-engine",
        "preedit::tests::ignored_preedit_candidate_does_not_create_learning_feedback",
    ),
    (
        "bin:lay-ibus-engine",
        "preedit::tests::manually_finished_visible_prediction_records_positive_usage",
    ),
    (
        "lib:lay",
        "correction_core::candidate_sources_tests::canonical_l2_field_route_uses_owned_surface_source_ids",
    ),
    (
        "lib:lay",
        "correction_core::candidate_sources_tests::canonical_l2_field_self_prepares_l11_candidate_without_peak_context",
    ),
    (
        "lib:lay",
        "correction_core::candidate_sources_tests::l2_field_births_generic_short_layout_candidate_for_l3_context",
    ),
    (
        "lib:lay",
        "nanda_wave::l2::tests::l2_exposes_l3_phrase_forecast_candidate_when_llmwave_is_enabled",
    ),
    (
        "lib:lay",
        "nanda_wave::lexical_grokking::service::tests::default_socket_uses_explicit_override_when_present",
    ),
    (
        "lib:lay",
        "nanda_wave::lexical_grokking::service::tests::default_socket_uses_runtime_dir_when_present",
    ),
    (
        "lib:lay",
        "nanda_wave::lexical_grokking::service::tests::direct_package_environment_override_is_rejected",
    ),
    (
        "lib:lay",
        "nanda_wave::lexical_grokking::service::tests::discover_installed_package_uses_only_the_integrity_bound_active_receipt",
    ),
    (
        "test:input_gate_space_contract",
        "daemon_space_and_enter_decoders_share_input_gate_replacement_contract",
    ),
    (
        "test:input_gate_space_contract",
        "space_autocorrect_keeps_existing_public_gate_contract",
    ),
    (
        "test:input_gate_space_contract",
        "weak_known_word_drift_is_suggest_only_without_latent_state_proof",
    ),
}

PERFORMANCE_TESTS = {
    (
        "bin:lay-daemon",
        "typing_assist_runtime::candidate::decoder::tests::startup_warmup_removes_first_boundary_decision_stall",
    ),
    (
        "bin:lay-daemon",
        "typing_assist_worker::tests::submit_never_runs_boundary_decision_on_key_thread",
    ),
    (
        "lib:lay",
        "correction_core::tests::live_canonical_l2_field_stays_under_latency_budget",
    ),
    (
        "lib:lay",
        "exact_layout_authority::proof::v27_exact_en_guard_controlled_resource_budget",
    ),
    (
        "bin:lay-ibus-engine",
        "preedit::tests::cold_english_wave_memory_does_not_block_precognition",
    ),
    (
        "bin:lay-ibus-engine",
        "preedit::tests::precognition_candidate_generation_stays_under_budget",
    ),
    (
        "bin:lay-ibus-engine",
        "space_autocorrect_prefetch::proof::v27_component_latency_denominators",
    ),
    (
        "lib:lay",
        "nanda_wave::candidate_gate::tests::live_gate_short_prefixes_stay_under_hot_readout_budget",
    ),
    (
        "lib:lay",
        "nanda_wave::candidate_gate::tests::unique_prefix_cache_misses_stay_under_hot_readout_budget",
    ),
    (
        "lib:lay",
        "nanda_wave::context_phase::tests::compiled_hot_context_readout_stays_inside_microsecond_budget",
    ),
    (
        "lib:lay",
        "nanda_wave::context_phase::tests::compiled_sentence_context_readout_stays_inside_preedit_budget",
    ),
}


class DiscoveryError(RuntimeError):
    pass


def canonical(value: Any) -> str:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def target_id(target: dict[str, Any]) -> str:
    kinds = target.get("kind") or []
    if len(kinds) != 1 or not isinstance(target.get("name"), str):
        raise DiscoveryError(f"unsupported Cargo target identity: {target!r}")
    return f"{kinds[0]}:{target['name']}"


def parse_cargo_artifacts(path: pathlib.Path) -> list[dict[str, str]]:
    artifacts: dict[str, dict[str, str]] = {}
    build_finished = False
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if build_finished:
            raise DiscoveryError(f"{path}:{line_number}: content after build-finished")
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            raise DiscoveryError(f"{path}:{line_number}: invalid Cargo JSON: {error}") from error
        if message.get("reason") == "build-finished":
            if message.get("success") is not True:
                raise DiscoveryError("Cargo discovery reported build failure")
            build_finished = True
            continue
        if message.get("reason") != "compiler-artifact":
            continue
        executable = message.get("executable")
        if not executable or not (message.get("profile") or {}).get("test"):
            continue
        identity = target_id(message.get("target") or {})
        row = {"target": identity, "executable": str(executable)}
        previous = artifacts.get(identity)
        if previous is not None and previous != row:
            raise DiscoveryError(f"multiple test executables for {identity}")
        artifacts[identity] = row
    if not build_finished:
        raise DiscoveryError("Cargo discovery lacks terminal build-finished")
    if not artifacts:
        raise DiscoveryError("Cargo discovery produced no test executables")
    return [artifacts[key] for key in sorted(artifacts)]


def _harness_names(executable: str, *arguments: str) -> dict[str, str]:
    completed = subprocess.run(
        [executable, "--list", *arguments, "--format", "terse"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise DiscoveryError(
            f"test listing failed for {executable}: {completed.stderr.strip()}"
        )
    rows: dict[str, str] = {}
    for line in completed.stdout.splitlines():
        if line.endswith(": test"):
            rows[line.removesuffix(": test")] = "test"
        elif line.endswith(": benchmark"):
            rows[line.removesuffix(": benchmark")] = "benchmark"
        elif line.strip():
            raise DiscoveryError(f"unparsed test-list row from {executable}: {line!r}")
    return rows


def classify(target: str, name: str, item_kind: str, ignored: bool) -> str:
    if ignored:
        return "ignored"
    if item_kind == "benchmark" or (target, name) in PERFORMANCE_TESTS:
        return "performance"
    if target in PACKAGE_TARGETS:
        return "package"
    return "correctness"


def isolation(target: str, name: str, lane: str) -> str:
    if lane == "performance" or (target, name) in PROCESS_ISOLATED_TESTS:
        return "process"
    return "target"


def discover_from_artifacts(artifacts: list[dict[str, str]]) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    tests: list[dict[str, str]] = []
    targets: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for artifact in artifacts:
        target = artifact["target"]
        executable = artifact["executable"]
        listed = _harness_names(executable)
        ignored = set(_harness_names(executable, "--ignored"))
        if not ignored <= set(listed):
            raise DiscoveryError(f"ignored listing is not a subset for {target}")
        targets.append({"target": target})
        for name, item_kind in sorted(listed.items()):
            identity = (target, name)
            if identity in seen:
                raise DiscoveryError(f"duplicate Cargo test identity: {identity}")
            seen.add(identity)
            lane = classify(target, name, item_kind, name in ignored)
            tests.append(
                {
                    "target": target,
                    "name": name,
                    "kind": item_kind,
                    "lane": lane,
                    "isolation": isolation(target, name, lane),
                }
            )
    missing_performance = sorted(PERFORMANCE_TESTS - seen)
    missing_isolated = sorted(PROCESS_ISOLATED_TESTS - seen)
    discovered_targets = {row["target"] for row in targets}
    missing_package_targets = sorted(PACKAGE_TARGETS - discovered_targets)
    if missing_performance or missing_isolated or missing_package_targets:
        raise DiscoveryError(
            "test registry drift: "
            f"performance={missing_performance} isolated={missing_isolated} "
            f"package_targets={missing_package_targets}"
        )
    return sorted(tests, key=canonical), sorted(targets, key=canonical)


def toolchain_identity() -> dict[str, str]:
    output = subprocess.run(
        ["rustc", "-Vv"], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout
    fields = dict(
        line.split(": ", 1) for line in output.splitlines() if ": " in line
    )
    return {
        "release": fields.get("release", ""),
        "commit": fields.get("commit-hash", ""),
        "host": fields.get("host", ""),
    }


def package_fixture_rows() -> list[dict[str, Any]]:
    rows = []
    for relative in PACKAGE_FIXTURE_PATHS:
        path = ROOT / relative
        if not path.is_file():
            raise DiscoveryError(f"package fixture is missing: {relative}")
        data = path.read_bytes()
        rows.append(
            {
                "path": relative,
                "size": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    return rows


def cargo_configuration_closure(
    root: pathlib.Path = ROOT, cargo_home: pathlib.Path | None = None
) -> dict[str, Any]:
    """Pin project Cargo config and reject config owned by the calling user."""
    root = root.resolve()
    if cargo_home is None:
        real_home = pathlib.Path(os.environ.get("HOME", str(pathlib.Path.home())))
        cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", real_home / ".cargo"))
    cargo_home = cargo_home.resolve()
    candidates = {cargo_home / "config", cargo_home / "config.toml"}
    current = root
    while True:
        candidates.add(current / ".cargo" / "config")
        candidates.add(current / ".cargo" / "config.toml")
        if current.parent == current:
            break
        current = current.parent
    project = []
    external = []
    for path in sorted(path for path in candidates if path.is_file()):
        try:
            relative = path.relative_to(root)
        except ValueError:
            external.append(str(path))
            continue
        data = path.read_bytes()
        project.append(
            {
                "path": relative.as_posix(),
                "size": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    if external:
        raise DiscoveryError(
            "external Cargo configuration is forbidden: " + ", ".join(external)
        )
    return {"external": "ABSENT", "project": project}


def manifest_payload(tests: list[dict[str, str]], targets: list[dict[str, str]]) -> dict[str, Any]:
    counts = {lane: 0 for lane in ("correctness", "package", "performance", "ignored")}
    isolation_counts = {kind: 0 for kind in ("process", "target")}
    for row in tests:
        counts[row["lane"]] += 1
        isolation_counts[row["isolation"]] += 1
    return {
        "schema": SCHEMA,
        "cargo_args": CARGO_ARGS,
        "cargo_configuration": cargo_configuration_closure(),
        "toolchain": toolchain_identity(),
        "counts": counts,
        "isolation_counts": isolation_counts,
        "package_fixtures": package_fixture_rows(),
        "targets": targets,
        "tests": tests,
    }


def cargo_build_environment(target_dir: pathlib.Path) -> dict[str, str]:
    cargo = shutil.which("cargo")
    rustc = shutil.which("rustc")
    if cargo is None or rustc is None:
        raise DiscoveryError("cargo and rustc must be available before test discovery")
    sysroot = subprocess.run(
        [rustc, "--print", "sysroot"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    home = target_dir / "discovery-home"
    for path in (home, home / "config", home / "cache", home / "data"):
        path.mkdir(parents=True, exist_ok=True)
    real_home = pathlib.Path(os.environ.get("HOME", str(pathlib.Path.home())))
    cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", real_home / ".cargo"))
    cargo_configuration_closure(cargo_home=cargo_home)
    return {
        "PATH": f"{sysroot}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "HOME": str(home),
        "XDG_CONFIG_HOME": str(home / "config"),
        "XDG_CACHE_HOME": str(home / "cache"),
        "XDG_DATA_HOME": str(home / "data"),
        "CARGO_HOME": str(cargo_home),
        "RUSTUP_HOME": os.environ.get("RUSTUP_HOME", str(real_home / ".rustup")),
        "CARGO_TARGET_DIR": str(target_dir),
        "CARGO_NET_OFFLINE": "true",
        "CARGO_INCREMENTAL": "0",
        "RUSTC": f"{sysroot}/bin/rustc",
        "RUSTDOC": f"{sysroot}/bin/rustdoc",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "UTC",
        "TMPDIR": "/tmp",
    }


def cargo_sandbox_command(command: list[str], target_dir: pathlib.Path) -> list[str]:
    bubblewrap = shutil.which("bwrap")
    if bubblewrap is None:
        raise DiscoveryError("bubblewrap is required for hermetic Cargo discovery")
    return [
        bubblewrap,
        "--die-with-parent",
        "--new-session",
        "--unshare-net",
        "--unshare-ipc",
        "--unshare-pid",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--tmpfs",
        "/run",
        "--tmpfs",
        "/tmp",
        "--bind",
        str(target_dir),
        str(target_dir),
        "--chdir",
        str(ROOT),
        *command,
    ]


def cargo_discovery(target_dir: pathlib.Path) -> tuple[dict[str, Any], list[dict[str, str]]]:
    target_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix="lay-test-discovery-", suffix=".jsonl") as output:
        environment = cargo_build_environment(target_dir)
        command = [
            str(ROOT / "scripts" / "cargo-guard.sh"),
            *CARGO_ARGS,
            "--message-format=json",
        ]
        completed = subprocess.run(
            cargo_sandbox_command(command, target_dir),
            cwd=ROOT,
            env=environment,
            stdout=output,
            check=False,
        )
        output.flush()
        if completed.returncode != 0:
            raise DiscoveryError(f"Cargo test discovery exited {completed.returncode}")
        artifacts = parse_cargo_artifacts(pathlib.Path(output.name))
    tests, targets = discover_from_artifacts(artifacts)
    return manifest_payload(tests, targets), artifacts
