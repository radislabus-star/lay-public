#!/usr/bin/python3
"""Contracts for the repository-owned private IME client harness."""

import hashlib
import ast
import importlib.util
import io
import json
from pathlib import Path, PurePosixPath
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[1]
HARNESS_ROOT = ROOT / "scripts/proof/ime-client"
SPEC = importlib.util.spec_from_file_location("ime_client_harness", HARNESS_ROOT / "run.py")
assert SPEC is not None and SPEC.loader is not None
HARNESS = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = HARNESS
SPEC.loader.exec_module(HARNESS)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def function_digest(path: Path, name: str) -> str:
    source = path.read_text(encoding="utf-8")
    tree = ast.parse(source)
    function = next(
        node for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name == name
    )
    lines = source.splitlines(keepends=True)
    return hashlib.sha256(
        "".join(lines[function.lineno - 1:function.end_lineno]).encode("utf-8")
    ).hexdigest()


class Fixture:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.candidate = root / "candidate/lay-ibus-engine"
        self.deployed = root / "deployed"
        self.deps = root / "deps"
        self.manifest = self.deps / "dependency-manifest.json"
        self.config = root / "harness-config.json"
        self.embedded = Path("/home/ubu/.local/share/lay/nanda_wave/l1.1")
        self._write_executable(self.candidate, b"candidate-fixture\n")
        self._write_executable(
            self.deployed / "bundle/bin/ibus-daemon", b"private-daemon\n"
        )
        self._write_executable(
            self.deployed / "bundle/lib/ld-linux-x86-64.so.2", b"private-loader\n"
        )
        entries = []
        recorded_root = Path("/recorded/td121-authority-deps")
        for index, relative in enumerate(HARNESS.ROLE_PATHS.values(), start=1):
            path = self.deps.joinpath(*relative.parts)
            payload = f"dependency-{index}-{relative}\n".encode("utf-8")
            if relative == HARNESS.ROLE_PATHS["l11_service"]:
                self._write_executable(path, payload)
            else:
                self._write(path, payload)
            entries.append(
                {
                    "path": str(recorded_root.joinpath(*relative.parts)),
                    "size_bytes": len(payload),
                    "mode": "0755" if relative == HARNESS.ROLE_PATHS["l11_service"] else "0644",
                    "sha256": digest(path),
                    "source_path": f"/source/{relative}",
                    "admission_status": "TEST_EXACT_BYTES",
                }
            )
        self._write_json(
            self.manifest,
            {
                "schema": "lay.td121.authority-dependency-manifest.v1",
                "status": "PREPARED_NOT_RUN",
                "remote_root": str(recorded_root),
                "proof_executed": False,
                "service_provenance": {},
                "receipt_topology": {
                    "runtime_receipt": str(self.embedded / "active.installed.json"),
                    "runtime_model_dir": str(self.embedded),
                    "mapping": "read-only namespace bind from remote_root/l1.1",
                    "receipt_bytes_rewritten": False,
                    "direct_lay_l11_package_env_forbidden": True,
                },
                "files": entries,
            },
        )
        self.rewrite_config()

    @staticmethod
    def _write(path: Path, payload: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)

    @classmethod
    def _write_executable(cls, path: Path, payload: bytes) -> None:
        cls._write(path, payload)
        path.chmod(0o755)

    @staticmethod
    def _write_json(path: Path, value: object) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

    def rewrite_config(self, **overrides: object) -> None:
        value = {
            "schema": HARNESS.SCHEMA,
            "origin_machine_id_sha256": "a" * 64,
            "execution_lease_id": "td124-ime-client-unit",
            "candidate": {
                "path": str(self.candidate),
                "sha256": digest(self.candidate),
            },
            "deployed_ibus_root": str(self.deployed),
            "dependencies": {
                "root": str(self.deps),
                "manifest_path": str(self.manifest),
                "manifest_sha256": digest(self.manifest),
                "receipt_embedded_mount_path": str(self.embedded),
            },
        }
        value.update(overrides)
        self._write_json(self.config, value)


class ImeClientHarnessTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="lay-ime-client-test-")
        self.root = Path(self.temporary.name)
        self.fixture = Fixture(self.root)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_v2_driver_and_preserved_scenarios_have_exact_identities(self) -> None:
        self.assertEqual(
            "d80447f21db4d689ea49d39742feb36e2202b23979d820380c1fcef916b88c12",
            HARNESS.verify_driver_identity(HARNESS_ROOT),
        )
        self.assertEqual(
            "eea5c141f6a8a10f6f9d6823fe7a526607c9a0f0e5db34dee4da9ad2ae1f98ab",
            function_digest(HARNESS_ROOT / "driver.py", "run_cases"),
        )
        source = (HARNESS_ROOT / "driver.py").read_text(encoding="utf-8")
        cases = [
            "same_context_us_to_us_authority_restoration",
            "same_context_us_to_ru_mixed",
            "same_context_ru_to_us_mixed_retention",
            "different_field_unknown_negative_authority_on",
            "unknown_start_negative_authority_on",
        ]
        for case in cases:
            self.assertEqual(1, source.count(f"case_name = '{case}'"), case)
        self.assertIn("signal.alarm(52)", source)
        self.assertIn("time.sleep(1.25)", source)
        self.assertIn("missing candidate Barrier; no retry", source)
        self.assertIn("GLib.timeout_add(2500, timeout)", source)
        self.assertIn("daemon.wait(timeout=3)", source)
        self.assertIn("daemon.wait(timeout=2)", source)
        self.assertEqual(1, source.count("IBus.Capabilite.SURROUNDING_TEXT"))
        self.assertNotIn("'require-surrounding-text'", source)
        self.assertEqual(
            1, source.count("self.apply_native_input(character, 'ProcessKeyEvent')")
        )

    def test_v2_contract_has_immutable_v1_git_provenance(self) -> None:
        self.assertEqual("lay.ime-client-harness.v1", HARNESS.SCHEMA)
        self.assertEqual(
            "lay.ime-client.actual-input-context.v2", HARNESS.PROOF_CONTRACT
        )
        self.assertEqual(
            "lay.ime-client-harness.run-metadata.v2", HARNESS.RUN_METADATA_SCHEMA
        )
        self.assertEqual(
            {
                "version": "v1",
                "git_commit": "708245298a3f553ac3c52243728c02ba6344a140",
                "git_blob": "04f7dbac56c92dbe0238c0754bcb6db4e242256c",
                "sha256": "9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750",
                "normalized_sha256": "669e3ef88cc2639794fde79dcb9132b99ebcaf142cafe39065376f42c4a056dc",
            },
            HARNESS.V1_DRIVER_PROVENANCE,
        )

    def test_explicit_startup_schedule_is_bound_to_command_and_metadata(self) -> None:
        self.assertEqual("immediate", HARNESS.parser().parse_args(
            ["--config", "c", "--output", "o"]).startup_schedule)
        plan = HARNESS.replace(HARNESS.load_plan(self.fixture.config),
                               startup_schedule="post-exact-ready")
        output = self.root / "post-ready-output"
        HARNESS.prepare_output(plan, output)
        metadata = json.loads((output / "run-metadata.json").read_text())
        self.assertEqual("post-exact-ready", metadata["startup_schedule"])
        command = HARNESS.build_command(plan, output)
        index = command.index("IME_CLIENT_STARTUP_SCHEDULE")
        self.assertEqual("--setenv", command[index - 1])
        self.assertEqual("post-exact-ready", command[index + 1])

    def test_lifecycle_scenarios_are_opt_in_and_bound_to_command_and_metadata(self) -> None:
        self.assertEqual("restoration", HARNESS.parser().parse_args(
            ["--config", "c", "--output", "o"]).scenario_set)
        plan = HARNESS.replace(HARNESS.load_plan(self.fixture.config),
                               scenario_set="lifecycle")
        output = self.root / "lifecycle-output"
        HARNESS.prepare_output(plan, output)
        metadata = json.loads((output / "run-metadata.json").read_text())
        self.assertEqual("lifecycle", metadata["scenario_set"])
        self.assertEqual("immediate", metadata["startup_schedule"])
        command = HARNESS.build_command(plan, output)
        index = command.index("IME_CLIENT_SCENARIO_SET")
        self.assertEqual("--setenv", command[index - 1])
        self.assertEqual("lifecycle", command[index + 1])

    def test_manual_lane_binds_terminal_consumer_and_enabled_preedit(self) -> None:
        for scenario_set in ("manual-toggle", "terminal-delivery", "first-word", "first-word-us", "first-word-ru"):
            plan = HARNESS.replace(HARNESS.load_plan(self.fixture.config),
                                   scenario_set=scenario_set)
            output = self.root / (scenario_set + "-output")
            HARNESS.prepare_output(plan, output)
            metadata = json.loads((output / "run-metadata.json").read_text())
            self.assertEqual(scenario_set, metadata["scenario_set"])
            self.assertTrue(json.loads((output / "config.json").read_text())["nanda_precognition"])
            self.assertEqual(digest(output / "config.json"), metadata["private_config_sha256"])
            self.assertEqual(digest(HARNESS_ROOT / "readline_consumer.py"),
                             digest(output / "readline_consumer.py"))
            self.assertEqual(digest(output / "readline_consumer.py"),
                             metadata["readline_consumer_sha256"])
            command = HARNESS.build_command(plan, output)
            self.assertEqual(scenario_set, command[command.index("IME_CLIENT_SCENARIO_SET") + 1])

    def test_startup_proof_profiles_bind_config_and_reject_scope_drift(self) -> None:
        base = HARNESS.load_plan(self.fixture.config)
        accepted = (("on", "first-word"), ("on", "first-word-us"),
                    ("on", "first-word-ru"), ("on", "fresh-preedit"),
                    ("on", "startup-only"), ("off", "fresh-preedit"),
                    ("off", "startup-only"), ("absent", "packages-absent-literal"),
                    ("absent", "startup-only"))
        for profile, scenario in accepted:
            plan = HARNESS.replace(base, startup_proof_profile=profile,
                                   scenario_set=scenario)
            HARNESS.validate_startup_proof_combination(plan)
        rejected = (("on", "restoration", "immediate"),
                    ("off", "first-word", "immediate"),
                    ("off", "first-word-us", "immediate"),
                    ("off", "first-word-ru", "immediate"),
                    ("absent", "fresh-preedit", "immediate"),
                    ("absent", "first-word-us", "immediate"),
                    ("absent", "first-word-ru", "immediate"),
                    ("on", "first-word-us", "post-exact-ready"),
                    ("on", "first-word-ru", "post-exact-ready"),
                    ("on", "startup-only", "post-exact-ready"))
        for profile, scenario, schedule in rejected:
            plan = HARNESS.replace(base, startup_proof_profile=profile,
                                   scenario_set=scenario, startup_schedule=schedule)
            with self.assertRaises(HARNESS.HarnessError):
                HARNESS.validate_startup_proof_combination(plan)

        off = HARNESS.replace(base, startup_proof_profile="off",
                              scenario_set="fresh-preedit")
        output = self.root / "off-preedit-output"
        HARNESS.prepare_output(off, output)
        config = json.loads((output / "config.json").read_text())
        metadata = json.loads((output / "run-metadata.json").read_text())
        self.assertFalse(config["nanda_autocorrect"])
        self.assertFalse(config["auto_replace"])
        self.assertFalse(config["auto_switch_layout"])
        self.assertFalse(config["typing_assist"])
        self.assertTrue(config["nanda_precognition"])
        self.assertEqual("off", metadata["startup_proof_profile"])

        driver_source = (HARNESS_ROOT / "driver.py").read_text(encoding="utf-8")
        driver_tree = ast.parse(driver_source)
        validator_node = next(
            node for node in driver_tree.body
            if isinstance(node, ast.FunctionDef)
            and node.name == "validate_private_config"
        )
        validator_namespace: dict[str, object] = {}
        exec(
            compile(
                ast.Module(body=[validator_node], type_ignores=[]),
                str(HARNESS_ROOT / "driver.py"),
                "exec",
            ),
            validator_namespace,
        )
        validate = validator_namespace["validate_private_config"]
        validate(config, "off")
        on_config = dict(config)
        on_config.update({
            "nanda_autocorrect": True,
            "auto_replace": True,
            "auto_switch_layout": True,
        })
        validate(on_config, "on")
        with self.assertRaises(AssertionError):
            validate(config, "on")
        with self.assertRaises(AssertionError):
            validate(on_config, "off")

    def test_absent_profile_omits_all_model_mounts_and_environment(self) -> None:
        plan = HARNESS.replace(HARNESS.load_plan(self.fixture.config),
                               startup_proof_profile="absent",
                               scenario_set="packages-absent-literal")
        output = self.root / "absent-output"
        HARNESS.prepare_output(plan, output)
        command = HARNESS.build_command(plan, output)
        rendered = "\0".join(command)
        self.assertNotIn(str(plan.deps_root), rendered)
        for name in ("LAY_L11_SERVICE_BIN", "LAY_L11_RECEIPT", "LAY_L2_PACKAGE",
                     "LAY_L2_LEXICAL_PHASE_MEMORY", "LAY_L2_PRODUCTIVE_V1_PACKAGE",
                     "LAY_L2_V13_DAFSA"):
            positions = [index for index, value in enumerate(command) if value == name]
            self.assertEqual(1, len(positions), name)
            self.assertEqual("--unsetenv", command[positions[0] - 1])
        self.assertEqual("/tmp/proof/no-models/l1.1",
                         command[command.index("LAY_L11_MODEL_DIR") + 1])
        self.assertEqual("/tmp/proof/no-models/l2",
                         command[command.index("LAY_L2_MODEL_DIR") + 1])

    def test_startup_trace_contract_rejects_oversized_malformed_and_inconsistent_input(self) -> None:
        source = (HARNESS_ROOT / "driver.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        function = next(node for node in tree.body if isinstance(node, ast.FunctionDef)
                        and node.name == "startup_trace_measurement")
        namespace = {"json": json, "STARTUP_PROOF_PROFILE": "on"}
        exec(compile(ast.Module(body=[function], type_ignores=[]), "driver.py", "exec"), namespace)
        measure = namespace["startup_trace_measurement"]
        trace = self.root / "startup-trace.jsonl"
        trace.write_bytes(b" " * (1024 * 1024 + 1))
        with self.assertRaisesRegex(ValueError, "bounded proof input"):
            measure(trace)
        trace.write_text("{malformed}\n", encoding="utf-8")
        with self.assertRaises(json.JSONDecodeError):
            measure(trace)
        rows = (
            {"kind": "ibus_startup_warmup", "stage": "completed",
             "exact_available": True, "l2_complete": True, "l2_available": False,
             "l2_candidate_ready": False, "elapsed_us": 10},
            {"kind": "ibus_exact_authority_warmup", "stage": "completed",
             "available": False, "elapsed_us": 4},
            {"kind": "ibus_context_admission", "member": "CreateEngine",
             "stage": "factory_acquisition_state"},
        )
        trace.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")
        with self.assertRaises(AssertionError):
            measure(trace)

    def test_startup_trace_contract_accepts_completed_unavailable_lexical_memory(self) -> None:
        source = (HARNESS_ROOT / "driver.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        function = next(node for node in tree.body if isinstance(node, ast.FunctionDef)
                        and node.name == "startup_trace_measurement")
        namespace = {"json": json, "STARTUP_PROOF_PROFILE": "absent"}
        exec(compile(ast.Module(body=[function], type_ignores=[]), "driver.py", "exec"),
             namespace)
        trace = self.root / "absent-startup-trace.jsonl"
        rows = (
            {"kind": "ibus_exact_authority_warmup", "stage": "completed",
             "available": False, "elapsed_us": 4},
            {"kind": "ibus_startup_warmup", "stage": "completed",
             "exact_available": False, "l2_complete": True, "l2_available": False,
             "l2_candidate_ready": False, "elapsed_us": 10},
            {"kind": "ibus_context_admission", "member": "CreateEngine",
             "stage": "factory_acquisition_state"},
        )
        trace.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")

        measurement = namespace["startup_trace_measurement"](trace)

        self.assertTrue(measurement["l2_complete"])
        self.assertFalse(measurement["l2_available"])
        self.assertFalse(measurement["l2_candidate_ready"])

    def test_lifecycle_scenarios_use_real_context_without_retry_or_new_wait(self) -> None:
        tree = ast.parse((HARNESS_ROOT / "driver.py").read_text())
        scenario = next(node for node in tree.body if isinstance(node, ast.FunctionDef)
                        and node.name == "run_lifecycle_cases")
        def shape(expression):
            return ast.dump(ast.parse(expression, mode="eval").body)

        calls = [ast.dump(node.func) for node in ast.walk(scenario)
                 if isinstance(node, ast.Call)]
        self.assertIn(shape("client.context.set_content_type"), calls)
        self.assertIn(shape("client.context.reset"), calls)
        self.assertIn(shape("client.focus_out"), calls)
        self.assertIn(shape("client.focus_in"), calls)
        self.assertEqual(1, calls.count(shape("select_engine")))
        self.assertNotIn(shape("time.sleep"), calls)
        self.assertFalse(any(isinstance(node, ast.While) for node in ast.walk(scenario)))
        self.assertIn(shape("assert_no_edits"), calls)
        assertions = {ast.dump(node.test) for node in ast.walk(scenario)
                      if isinstance(node, ast.Assert)}
        for expression in (
            "client.key('j')",
            "after['state'] == 'passive:unknown-context'",
            "after['text'] == '' and not after['focus_receipt']",
            "empty['text'] == '' and not empty['focus_receipt']",
            "continued['text'] == ' j'",
        ):
            self.assertIn(shape(expression), assertions)
        self.assertFalse(any(keyword.arg == "expected_prefix"
                             for node in ast.walk(scenario) if isinstance(node, ast.Call)
                             for keyword in node.keywords))

        def key_sequence(statements):
            values = []
            for node in statements:
                call = (node.test if isinstance(node, ast.Assert) else
                        node.value if isinstance(node, ast.Expr) else None)
                if isinstance(call, ast.Call) and ast.dump(call.func) == shape("client.key"):
                    values.append(ast.literal_eval(call.args[0]))
            return values

        self.assertEqual([" ", "l", " ", "j"], key_sequence(scenario.body))
        transitions = [node for node in scenario.body if isinstance(node, ast.For)]
        self.assertEqual(1, len(transitions))
        self.assertEqual(("terminal_content_type", "reset"),
                         ast.literal_eval(transitions[0].iter))
        self.assertEqual([" ", "l"], key_sequence(transitions[0].body))

    def test_lifecycle_pass_requires_exact_ordered_three_case_denominator(self) -> None:
        tree = ast.parse((HARNESS_ROOT / "driver.py").read_text())
        branch = next(node for node in ast.walk(tree) if isinstance(node, ast.If)
                      and isinstance(node.body[0], ast.Expr)
                      and isinstance(node.body[0].value, ast.Call)
                      and isinstance(node.body[0].value.func, ast.Name)
                      and node.body[0].value.func.id == "run_lifecycle_cases")
        self.assertIsInstance(branch.body[1], ast.Assert)
        check = compile(ast.Expression(branch.body[1].test), "case denominator", "eval")
        expected = ["same_engine_refocus_via_dummy_context_discards_word",
                    "refocus_terminal_content_type_drops_word_not_context",
                    "refocus_reset_drops_word_not_context"]
        for names in (expected, expected[:2], expected + expected[:1],
                      expected[::-1], [expected[0]] * 3):
            actual = eval(check, {"receipt": {"cases": [{"case": name} for name in names]}})
            self.assertEqual(names == expected, actual, names)

    def test_driver_and_template_have_no_historical_cache_path(self) -> None:
        for path in (HARNESS_ROOT / "driver.py", HARNESS_ROOT / "component/lay.xml.template"):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("/home/e/projects/lay-td119-gate-v1", source)
            self.assertNotIn("td121-private-optimized", source)

    def test_candidate_process_observer_does_not_count_wrapper_arguments(self) -> None:
        tree = ast.parse((HARNESS_ROOT / "driver.py").read_text())
        function = next(node for node in tree.body
                        if isinstance(node, ast.FunctionDef) and node.name == "candidate_pids")
        condition = next(node.test for node in ast.walk(function)
                         if isinstance(node, ast.If) and "CANDIDATE" in ast.unparse(node.test))
        expression = compile(ast.Expression(condition), "candidate_pids predicate", "eval")
        candidate = str(HARNESS.SANDBOX_CANDIDATE)
        for command, expected in ((candidate + " --ibus --managed ", True),
                                  ("/usr/bin/bwrap --ro-bind " + candidate + " /tmp/bin", False),
                                  ("/usr/bin/python3 driver.py " + candidate, False)):
            with self.subTest(command=command):
                self.assertEqual(expected, eval(expression, {"CANDIDATE": Path(candidate), "command": command}))

    def test_missing_candidate_refuses_before_output_or_launch(self) -> None:
        output = self.root / "missing-candidate-output"
        self.fixture.candidate.unlink()
        with mock.patch.object(HARNESS, "execute") as execute:
            status = HARNESS.main(
                [
                    "--remote-worker",
                    "--config",
                    str(self.fixture.config),
                    "--output",
                    str(output),
                ]
            )
        self.assertEqual(65, status)
        self.assertFalse(output.exists())
        execute.assert_not_called()

    def test_dependency_hash_mismatch_refuses_before_output_or_launch(self) -> None:
        for role in ("l2_v13", "l2_lexical_phase"):
            with self.subTest(role=role):
                output = self.root / f"bad-{role}-output"
                relative = HARNESS.ROLE_PATHS[role]
                dependency = self.fixture.deps.joinpath(*relative.parts)
                admitted = dependency.read_bytes()
                changed = bytes([admitted[0] ^ 1]) + admitted[1:]
                self.assertEqual(len(admitted), len(changed))
                dependency.write_bytes(changed)
                stderr = io.StringIO()
                try:
                    with mock.patch.object(
                        HARNESS, "execute"
                    ) as execute, mock.patch.object(HARNESS.sys, "stderr", stderr):
                        status = HARNESS.main(
                            [
                                "--remote-worker",
                                "--config",
                                str(self.fixture.config),
                                "--output",
                                str(output),
                            ]
                        )
                finally:
                    dependency.write_bytes(admitted)
                self.assertEqual(65, status)
                self.assertIn(
                    f"dependency {relative} hash mismatch",
                    stderr.getvalue(),
                )
                self.assertFalse(output.exists())
                execute.assert_not_called()

    def test_missing_l2_lexical_phase_refuses_before_output_or_launch(self) -> None:
        output = self.root / "missing-l2-lexical-phase-output"
        dependency = self.fixture.deps.joinpath(
            *HARNESS.ROLE_PATHS["l2_lexical_phase"].parts
        )
        dependency.unlink()
        with mock.patch.object(HARNESS, "execute") as execute:
            status = HARNESS.main(
                [
                    "--remote-worker",
                    "--config",
                    str(self.fixture.config),
                    "--output",
                    str(output),
                ]
            )
        self.assertEqual(65, status)
        self.assertFalse(output.exists())
        execute.assert_not_called()

    def test_manifest_without_l2_lexical_phase_refuses_before_output_or_launch(self) -> None:
        output = self.root / "old-eight-role-manifest-output"
        manifest = json.loads(self.fixture.manifest.read_text(encoding="utf-8"))
        lexical_path = str(
            Path(manifest["remote_root"]).joinpath(
                *HARNESS.ROLE_PATHS["l2_lexical_phase"].parts
            )
        )
        manifest["files"] = [
            entry for entry in manifest["files"] if entry["path"] != lexical_path
        ]
        self.assertEqual(8, len(manifest["files"]))
        self.fixture._write_json(self.fixture.manifest, manifest)
        self.fixture.rewrite_config()
        config = json.loads(self.fixture.config.read_text(encoding="utf-8"))
        self.assertEqual(
            digest(self.fixture.manifest),
            config["dependencies"]["manifest_sha256"],
        )
        stderr = io.StringIO()
        with mock.patch.object(HARNESS, "execute") as execute, mock.patch.object(
            HARNESS.sys, "stderr", stderr
        ):
            status = HARNESS.main(
                [
                    "--remote-worker",
                    "--config",
                    str(self.fixture.config),
                    "--output",
                    str(output),
                ]
            )
        self.assertEqual(64, status)
        self.assertIn(
            "missing=['l2/l2_lexical_phase_v2.bin']",
            stderr.getvalue(),
        )
        self.assertFalse(output.exists())
        execute.assert_not_called()

    def test_candidate_is_consistent_across_bind_driver_and_component(self) -> None:
        plan = HARNESS.load_plan(self.fixture.config)
        output = self.root / "prepared-output"
        HARNESS.prepare_output(plan, output, HARNESS_ROOT)
        command = HARNESS.build_command(plan, output)
        generated_exec = ET.parse(output / "component/lay.xml").getroot().findtext("exec")
        self.assertEqual("/tmp/candidate/lay-ibus-engine --ibus --managed", generated_exec)
        candidate_bind = command.index(str(plan.candidate))
        self.assertEqual("--ro-bind", command[candidate_bind - 1])
        self.assertEqual(str(HARNESS.SANDBOX_CANDIDATE), command[candidate_bind + 1])
        injected = command.index("IME_CLIENT_CANDIDATE")
        self.assertEqual(str(HARNESS.SANDBOX_CANDIDATE), command[injected + 1])
        metadata = json.loads((output / "run-metadata.json").read_text(encoding="utf-8"))
        self.assertEqual(HARNESS.RUN_METADATA_SCHEMA, metadata["schema"])
        self.assertEqual(HARNESS.PROOF_CONTRACT, metadata["proof_contract"])
        self.assertEqual(HARNESS.SCHEMA, metadata["configuration_schema"])
        self.assertEqual(str(plan.candidate), metadata["candidate"]["configured_path"])
        self.assertEqual(str(HARNESS.SANDBOX_CANDIDATE), metadata["candidate"]["sandbox_path"])
        self.assertEqual("v2", metadata["driver"]["version"])
        self.assertEqual(HARNESS.V2_DRIVER_SHA256, metadata["driver"]["sha256"])
        self.assertEqual(HARNESS.V1_DRIVER_PROVENANCE, metadata["driver"]["predecessor"])
        self.assertEqual(
            "SUCCESSOR_CONTRACT_NOT_BASELINE_PARITY",
            metadata["driver"]["comparison"],
        )
        self.assertEqual(HARNESS.V2_DRIVER_SHA256, digest(output / "driver.py"))

    def test_existing_output_is_immutable_and_never_launches(self) -> None:
        output = self.root / "existing-output"
        output.mkdir()
        sentinel = output / "sentinel"
        sentinel.write_text("preserve\n", encoding="utf-8")
        with mock.patch.object(HARNESS, "load_plan") as load_plan, mock.patch.object(
            HARNESS, "execute"
        ) as execute:
            status = HARNESS.main(
                [
                    "--remote-worker",
                    "--config",
                    str(self.fixture.config),
                    "--output",
                    str(output),
                ]
            )
        self.assertEqual(65, status)
        self.assertEqual("preserve\n", sentinel.read_text(encoding="utf-8"))
        load_plan.assert_not_called()
        execute.assert_not_called()

    def test_remote_guard_requires_flag_and_distinct_machine_id(self) -> None:
        machine_id = self.root / "machine-id"
        machine_id.write_bytes(b"origin-machine\n")
        origin_hash = digest(machine_id)
        with self.assertRaisesRegex(HARNESS.HarnessError, "--remote-worker"):
            HARNESS.validate_remote_guard(False, origin_hash, machine_id)
        with self.assertRaisesRegex(HARNESS.HarnessError, "origin machine"):
            HARNESS.validate_remote_guard(True, origin_hash, machine_id)
        with mock.patch.dict(HARNESS.os.environ, HARNESS.REQUIRED_GUARD, clear=True):
            HARNESS.validate_remote_guard(True, "b" * 64, machine_id)

    def test_remote_guard_requires_every_existing_lease_marker(self) -> None:
        machine_id = self.root / "machine-id"
        machine_id.write_bytes(b"worker-machine\n")
        for missing in HARNESS.REQUIRED_GUARD:
            environment = {key: value for key, value in HARNESS.REQUIRED_GUARD.items()
                           if key != missing}
            with self.subTest(missing=missing), mock.patch.dict(HARNESS.os.environ, environment, clear=True):
                with self.assertRaisesRegex(HARNESS.HarnessError, "heavy lease required"):
                    HARNESS.validate_remote_guard(True, "b" * 64, machine_id)

    def test_main_without_guard_refuses_before_output_or_process(self) -> None:
        with mock.patch.dict(HARNESS.os.environ, {}, clear=True), mock.patch.object(HARNESS, "prepare_output") as prepare, mock.patch.object(HARNESS, "execute") as execute:
            status = HARNESS.main(["--remote-worker", "--config", str(self.fixture.config),
                                   "--output", str(self.root / "not-created")])
        self.assertEqual(64, status)
        prepare.assert_not_called()
        execute.assert_not_called()
        self.assertFalse((self.root / "not-created").exists())

    def test_manifest_rebases_all_exact_dependencies_and_receipt_mount(self) -> None:
        plan = HARNESS.load_plan(self.fixture.config)
        self.assertEqual(
            PurePosixPath("l2/l2_lexical_phase_v2.bin"),
            HARNESS.ROLE_PATHS["l2_lexical_phase"],
        )
        self.assertEqual(set(HARNESS.ROLE_PATHS), set(plan.dependencies))
        self.assertEqual(9, len(plan.dependencies))
        for dependency in plan.dependencies.values():
            self.assertTrue(dependency.host_path.is_relative_to(self.fixture.deps))
            self.assertEqual(dependency.sha256, digest(dependency.host_path))
        self.assertEqual(self.fixture.embedded, plan.receipt_embedded_mount_path)

    def test_command_keeps_private_isolation_and_frozen_resource_budget(self) -> None:
        plan = HARNESS.load_plan(self.fixture.config)
        command = HARNESS.build_command(plan, self.root / "future-output")
        for value in (
            "--property=CPUQuota=200%",
            "--property=MemoryMax=1536M",
            "--property=MemorySwapMax=0",
            "--property=TasksMax=128",
            "--property=RuntimeMaxSec=90s",
            "--unshare-all",
            "--die-with-parent",
            "--new-session",
            "--tmpfs",
            "/usr/bin/dbus-run-session",
            "--config-file=/tmp/proof/dbus.conf",
        ):
            self.assertIn(value, command)
        self.assertNotIn("ibus restart", " ".join(command))
        self.assertNotIn("systemctl restart", " ".join(command))
        lexical_env = command.index("LAY_L2_LEXICAL_PHASE_MEMORY")
        self.assertEqual("--setenv", command[lexical_env - 1])
        self.assertEqual(
            "/tmp/deps/l2/l2_lexical_phase_v2.bin",
            command[lexical_env + 1],
        )
        self.assertEqual(1, command.count("LAY_L2_LEXICAL_PHASE_MEMORY"))

    def test_productive_v90_command_binds_live_runtime_environment(self) -> None:
        plan = HARNESS.load_plan(self.fixture.config)
        command = HARNESS.build_command(plan, self.root / "future-output")
        key = "LAY_L2_PRODUCTIVE_V1_PACKAGE"
        index = command.index(key)
        self.assertEqual("--setenv", command[index - 1])
        self.assertEqual(
            "/tmp/deps/l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m",
            command[index + 1],
        )
        self.assertEqual(1, command.count(key))
        self.assertNotIn("LAY_L2_PRODUCTIVE_PACKAGE", command)
        hash_index = command.index("EXPECTED_" + key + "_SHA256")
        self.assertEqual(
            plan.dependencies["productive_v90_package"].sha256,
            command[hash_index + 1],
        )
        driver = (HARNESS_ROOT / "driver.py").read_text(encoding="utf-8")
        self.assertIn("'" + key + "'", driver)
        self.assertIn("'EXPECTED_" + key + "_SHA256'", driver)
        self.assertNotIn("'LAY_L2_PRODUCTIVE_PACKAGE'", driver)


class ImeClientConsumerTest(unittest.TestCase):
    """Exercise the real driver class without starting its private processes."""

    def setUp(self) -> None:
        source = (HARNESS_ROOT / "driver.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        node = next(node for node in tree.body
                    if isinstance(node, ast.ClassDef) and node.name == "Client")
        oracle_nodes = [node for node in tree.body
                        if isinstance(node, ast.FunctionDef)
                        and node.name in {"outputs_since", "deliver_exact_literal"}]
        self.context = mock.Mock()
        self.context.get_object_path.return_value = "/private/context"
        self.context.needs_surrounding_text.return_value = True
        self.context.process_key_event.return_value = False
        self.events = []
        self.namespace = {
            "ibus_bus": mock.Mock(), "contexts": [], "drain": lambda: None,
            "emit": lambda _handle, **row: self.events.append(row) or row,
            "client_log": None, "input_log": None, "output_log": None,
            "KEYCODES": {"l": 38}, "RELEASE_MASK": 1 << 30,
            "IBus": SimpleNamespace(
                unicode_to_keyval=lambda character: ord(character),
                Capabilite=SimpleNamespace(PREEDIT_TEXT=1, FOCUS=8, SURROUNDING_TEXT=32),
                InputPurpose=SimpleNamespace(FREE_FORM=0),
                Text=SimpleNamespace(new_from_string=lambda text: text)),
        }
        self.namespace["ibus_bus"].create_input_context.return_value = self.context
        exec(compile(ast.Module(body=[node, *oracle_nodes], type_ignores=[]),
                     str(HARNESS_ROOT / "driver.py"), "exec"), self.namespace)
        self.client = self.namespace["Client"]("unit-consumer")
        self.deliver_exact_literal = self.namespace["deliver_exact_literal"]

    def test_unhandled_press_inserts_once_and_release_never_duplicates(self) -> None:
        self.assertFalse(self.client.key("l"))
        self.assertEqual(("l", 1), (self.client.visible, self.client.cursor))
        self.assertEqual(1, sum(row["kind"] == "NativeUnhandledInput"
                               for row in self.events))
        self.assertEqual([mock.call(ord("l"), 38, 0),
                          mock.call(ord("l"), 38, 1 << 30)],
                         self.context.process_key_event.call_args_list)
        self.context.set_surrounding_text.assert_called_with("l", 1, 1)

    def test_handled_commit_is_not_reinserted_on_unhandled_release(self) -> None:
        def process(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "л"))
                return True
            return False
        self.context.process_key_event.side_effect = process
        self.assertTrue(self.client.key("l"))
        self.assertEqual(("л", 1), (self.client.visible, self.client.cursor))
        self.assertFalse(any(row["kind"] == "NativeUnhandledInput" for row in self.events))

    def test_literal_oracle_accepts_exact_native_and_managed_delivery(self) -> None:
        native = self.deliver_exact_literal(self.client, "l")
        self.assertFalse(native["handled"])
        self.assertEqual(("l", 1), (self.client.visible, self.client.cursor))

        self.client.visible, self.client.cursor, self.client.output = "", 0, []
        def managed(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "l"))
                return True
            return False
        self.context.process_key_event.side_effect = managed
        committed = self.deliver_exact_literal(self.client, "l")
        self.assertTrue(committed["handled"])
        self.assertEqual(("l", 1), (self.client.visible, self.client.cursor))

    def test_literal_oracle_rejects_lost_duplicate_mismatched_and_destructive_delivery(self) -> None:
        def lost(_keyval, _keycode, state):
            return not state

        def duplicate(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "l"))
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "l"))
                return True
            return False

        def mismatched(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "x"))
                return True
            return False

        def deleted(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "l"))
                self.client.on_delete(None, -1, 1)
                return True
            return False

        def forwarded(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "l"))
                self.client.on_forward(None, ord("l"), 38, 0)
                return True
            return False

        def wrong_cursor(_keyval, _keycode, state):
            if not state:
                self.client.on_commit(None, SimpleNamespace(get_text=lambda: "l"))
                self.client.cursor = 0
                return True
            return False

        for name, behavior in (("lost", lost), ("duplicate", duplicate),
                               ("mismatched", mismatched), ("delete", deleted),
                               ("forward", forwarded), ("cursor", wrong_cursor)):
            with self.subTest(name=name):
                self.client.visible, self.client.cursor, self.client.output = "", 0, []
                self.context.process_key_event.side_effect = behavior
                with self.assertRaises(AssertionError):
                    self.deliver_exact_literal(self.client, "l")

    def test_requested_snapshot_is_published_with_current_cursor(self) -> None:
        self.client.visible, self.client.cursor = "метка слово", 11
        self.client.publish_surrounding("initial")
        self.context.set_surrounding_text.reset_mock()
        self.client.publish_surrounding("clean-not-requested")
        self.context.set_surrounding_text.assert_not_called()
        self.client.publish_surrounding("requested", force=True)
        self.context.set_surrounding_text.assert_called_once_with("метка слово", 11, 11)
        self.client.on_delete(None, -5, 5)
        self.client.on_commit(None, SimpleNamespace(get_text=lambda: "слова"))
        self.client.publish_surrounding("replacement")
        self.assertEqual("метка слова", self.client.visible)
        self.context.set_surrounding_text.assert_called_with("метка слова", 11, 11)

    def test_pre_request_publication_cannot_cache_an_unsent_snapshot(self) -> None:
        self.context.needs_surrounding_text.return_value = False
        self.client.publish_surrounding("not-requested", force=True)
        self.context.set_surrounding_text.assert_not_called()
        self.assertTrue(self.client.surrounding_dirty)
        self.context.needs_surrounding_text.return_value = True
        self.client.publish_surrounding("requested")
        self.context.set_surrounding_text.assert_called_once_with("", 0, 0)
        self.assertFalse(self.client.surrounding_dirty)

    def test_focus_publication_dispatches_the_request_first(self) -> None:
        self.context.needs_surrounding_text.return_value = False
        def dispatch():
            self.context.needs_surrounding_text.return_value = True
        self.namespace["drain"] = dispatch
        self.client.focus_in()
        self.context.set_surrounding_text.assert_called_once_with("", 0, 0)

    def test_actual_installed_gi_supports_all_registered_signals_and_methods(self) -> None:
        try:
            import gi
            gi.require_version("IBus", "1.0")
            from gi.repository import IBus
        except (ImportError, ValueError):
            self.skipTest("installed IBus GI metadata unavailable; actual-client gate still required")
        signals = {signal.get_name() for signal in IBus.InputContext.__info__.get_signals()}
        registered = [call.args[0] for call in self.context.connect.call_args_list]
        self.assertTrue(registered)
        self.assertTrue(set(registered) <= signals, (registered, signals))
        self.assertTrue(callable(IBus.InputContext.needs_surrounding_text))
        self.assertTrue(callable(IBus.InputContext.set_surrounding_text))
        self.namespace["IBus"].unicode_to_keyval = IBus.unicode_to_keyval
        self.namespace["KEYCODES"]["п"] = 34
        for character in ("l", "п"):
            self.assertFalse(self.client.key(character))
        self.assertEqual("lп", self.client.visible)
        self.assertEqual(
            mock.call(IBus.unicode_to_keyval("п"), 34, 1 << 30),
            self.context.process_key_event.call_args,
        )


class ImeClientStartupScheduleTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="ime-startup-unit-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.loop, self.monitor = mock.Mock(), mock.Mock()
        self.gio, self.glib = mock.Mock(), mock.Mock()
        self.gio.File.new_for_path.return_value.monitor_file.return_value = self.monitor
        self.glib.MainLoop.return_value = self.loop
        self.glib.timeout_add.return_value = 1
        self.namespace = {
            "startup_observed": False, "STARTUP_SCHEDULE": "post-exact-ready",
            "ROOT": self.root, "GLib": self.glib, "Gio": self.gio,
            "time": time, "json": json, "receipt": {}, "internal_log": None,
            "emit": lambda _handle, **row: row,
        }
        source = (HARNESS_ROOT / "driver.py").read_text()
        node = next(node for node in ast.parse(source).body
                    if isinstance(node, ast.FunctionDef)
                    and node.name == "observe_startup_schedule")
        exec(compile(ast.Module(body=[node], type_ignores=[]), "driver-startup", "exec"),
             self.namespace)
        self.observe = self.namespace["observe_startup_schedule"]

    def complete(self, available=True):
        (self.root / "ibus-engine-trace.jsonl").write_text(json.dumps({
            "kind": "ibus_exact_authority_warmup", "stage": "completed",
            "available": available, "elapsed_us": 116257}) + "\n")

    def test_immediate_schedule_does_not_monitor_or_wait(self) -> None:
        self.namespace["STARTUP_SCHEDULE"] = "immediate"
        self.observe()
        self.gio.File.new_for_path.assert_not_called()
        self.loop.run.assert_not_called()

    def test_already_completed_trace_is_observed_once_without_wait(self) -> None:
        self.complete()
        self.observe()
        self.observe()
        self.gio.File.new_for_path.assert_called_once()
        self.loop.run.assert_not_called()
        self.assertTrue(self.namespace["startup_observed"])
        row = self.namespace["receipt"]["startup_observation"]
        self.assertFalse(row["production_latency_proof"])
        self.assertEqual(116257, row["candidate_warmup_us"])
        self.monitor.cancel.assert_called_once()

    def test_notification_after_subscription_closes_the_same_wait(self) -> None:
        def notify():
            self.complete()
            self.monitor.connect.call_args.args[1]()
        self.loop.run.side_effect = notify
        self.observe()
        self.loop.run.assert_called_once()
        self.assertTrue(self.namespace["startup_observed"])

    def test_unavailable_or_missing_completion_never_marks_ready(self) -> None:
        self.complete(available=False)
        with self.assertRaisesRegex(ValueError, "unavailable"):
            self.observe()
        self.assertFalse(self.namespace["startup_observed"])
        (self.root / "ibus-engine-trace.jsonl").unlink()
        with self.assertRaisesRegex(AssertionError, "no input/retry"):
            self.observe()
        self.assertFalse(self.namespace["startup_observed"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
