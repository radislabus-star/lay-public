from __future__ import annotations

import importlib.util
import dataclasses
import io
import json
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


SMOKE = load_module("runtime_smoke_main", SCRIPTS / "run_runtime_smoke.py")
EXECUTION = sys.modules["runtime_smoke.execution"]
ISOLATION = sys.modules["runtime_smoke.isolation"]
CASE_MODULE = sys.modules["runtime_smoke.cases"]
RECEIPT = sys.modules["runtime_smoke.receipt"]
VERIFY = load_module(
    "runtime_smoke_isolation_verifier",
    SCRIPTS / "verify_runtime_smoke_isolation.py",
)
IME = load_module("runtime_smoke_ime_isolation", SCRIPTS / "runtime_smoke" / "ime.py")
DESKTOP = load_module(
    "runtime_smoke_desktop", SCRIPTS / "runtime_smoke" / "desktop.py"
)


class FakeProcess:
    def __init__(
        self,
        name: str,
        events: list[str],
        timeout_once: bool = False,
        stdout_line: str = "",
        output: tuple[str, str] = ("", ""),
    ):
        self.name = name
        self.events = events
        self.timeout_once = timeout_once
        self.returncode = None
        self.stdout = io.StringIO(stdout_line)
        self.stderr = io.StringIO()
        self.output = output
        self.terminated = False

    def poll(self):
        return self.returncode

    def terminate(self):
        self.events.append(f"terminate:{self.name}")
        self.terminated = True

    def kill(self):
        self.events.append(f"kill:{self.name}")
        self.returncode = -9

    def communicate(self, timeout=None):
        self.events.append(f"communicate:{self.name}:{timeout}")
        if self.timeout_once:
            self.timeout_once = False
            raise subprocess.TimeoutExpired(self.name, timeout)
        if self.returncode is None:
            self.returncode = -15 if self.terminated else 0
        return self.output


class RuntimeSmokeIsolationTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="lay-runtime-smoke-test-")
        self.root = Path(self.temp.name)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_live_execution_requires_explicit_managed_desktop_admission(self) -> None:
        with self.assertRaisesRegex(ValueError, "managed-desktop"):
            SMOKE.validate_live_admission(
                SimpleNamespace(
                    managed_desktop=False,
                    ime_engine=False,
                    ime_managed=False,
                    verify_ime_trace=False,
                    use_system_daemon=False,
                )
            )

    def test_shared_system_daemon_route_is_never_admitted(self) -> None:
        with self.assertRaisesRegex(ValueError, "isolated daemon"):
            SMOKE.validate_live_admission(
                SimpleNamespace(
                    managed_desktop=True,
                    ime_engine=False,
                    ime_managed=True,
                    verify_ime_trace=True,
                    use_system_daemon=True,
                    timeout=1.0,
                    focus_delay=0.0,
                )
            )

    def test_live_execution_requires_per_case_engine_isolation(self) -> None:
        with self.assertRaisesRegex(ValueError, "ime-managed"):
            SMOKE.validate_live_admission(
                SimpleNamespace(
                    managed_desktop=True,
                    ime_engine=False,
                    ime_managed=False,
                    verify_ime_trace=True,
                    use_system_daemon=False,
                    timeout=1.0,
                    focus_delay=0.0,
                )
            )

    def test_live_execution_requires_trace_validation(self) -> None:
        with self.assertRaisesRegex(ValueError, "verify-ime-trace"):
            SMOKE.validate_live_admission(
                SimpleNamespace(
                    managed_desktop=True,
                    ime_engine=False,
                    ime_managed=True,
                    verify_ime_trace=False,
                    use_system_daemon=False,
                    timeout=1.0,
                    focus_delay=0.0,
                )
            )

    def test_case_identity_and_trace_are_stable_and_isolated(self) -> None:
        first = ISOLATION.prepare_case_context(self.root, "run-fixed", "case-a")
        second = ISOLATION.prepare_case_context(self.root, "run-fixed", "case-b")

        self.assertNotEqual(first.case_id, second.case_id)
        self.assertNotEqual(first.trace_path, second.trace_path)
        self.assertEqual(
            first.case_id,
            ISOLATION.case_id_for("run-fixed", "case-a"),
        )
        self.assertTrue(first.trace_path.parent.is_dir())

    def test_case_directory_is_never_reused(self) -> None:
        ISOLATION.prepare_case_context(self.root, "run-fixed", "case-a")

        with self.assertRaises(FileExistsError):
            ISOLATION.prepare_case_context(self.root, "run-fixed", "case-a")

    def test_binary_build_uses_repository_cargo_guard(self) -> None:
        binary = self.root / "target" / "release" / "lay-daemon"
        calls: list[list[str]] = []

        def fake_run(argv, **kwargs):
            calls.append(list(argv))
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"candidate")
            return subprocess.CompletedProcess(argv, 0)

        with mock.patch.object(SMOKE.subprocess, "run", side_effect=fake_run):
            self.assertEqual(binary, SMOKE.ensure_binary(binary, "lay-daemon", False))

        self.assertEqual(
            [
                str(ROOT / "scripts" / "cargo-guard.sh"),
                "build",
                "--release",
                "--bin",
                "lay-daemon",
            ],
            calls[0],
        )

    def test_process_supervisor_cleans_in_reverse_order_and_kills_timeout(self) -> None:
        events: list[str] = []
        supervisor = ISOLATION.ProcessSupervisor()
        supervisor.track("dialog", FakeProcess("dialog", events))
        supervisor.track("sender", FakeProcess("sender", events, timeout_once=True))
        supervisor.close()

        self.assertEqual(
            [
                "terminate:sender",
                "communicate:sender:3.0",
                "kill:sender",
                "communicate:sender:3.0",
                "terminate:dialog",
                "communicate:dialog:3.0",
            ],
            events,
        )

    def test_process_supervisor_attempts_every_cleanup_after_one_failure(self) -> None:
        events: list[str] = []
        supervisor = ISOLATION.ProcessSupervisor()
        first = supervisor.track("first", FakeProcess("first", events))
        second = supervisor.track("second", FakeProcess("second", events))

        def stop(process, timeout=3.0):
            events.append(f"stop:{process.name}")
            if process is second:
                raise RuntimeError("second cleanup failed")
            process.returncode = -15
            return "", ""

        with (
            mock.patch.object(supervisor, "stop", side_effect=stop),
            self.assertRaisesRegex(RuntimeError, "second cleanup failed"),
        ):
            supervisor.close()

        self.assertEqual(["stop:second", "stop:first"], events)

    def test_process_supervisor_never_waits_unbounded_after_kill(self) -> None:
        events: list[str] = []
        process = FakeProcess("stuck", events)
        process.communicate = mock.Mock(
            side_effect=[
                subprocess.TimeoutExpired("stuck", 3.0),
                subprocess.TimeoutExpired("stuck", 3.0),
            ]
        )
        supervisor = ISOLATION.ProcessSupervisor()

        with self.assertRaisesRegex(RuntimeError, "after SIGKILL"):
            supervisor.stop(process)

        self.assertEqual(
            [mock.call(timeout=3.0), mock.call(timeout=3.0)],
            process.communicate.call_args_list,
        )

    def test_spawn_is_owned_before_pending_signal_is_unblocked(self) -> None:
        events: list[str] = []
        process = FakeProcess("spawned", events)
        handlers = {}

        def original(signum, _frame):
            events.append(f"replayed:{signum}")
            raise ISOLATION.SmokeInterrupted("pending signal")

        def install(signum, handler):
            handlers[signum] = handler

        def popen(*_args, **_kwargs):
            events.append("popen")
            handlers[signal.SIGTERM](signal.SIGTERM, None)
            return process

        with (
            mock.patch.object(ISOLATION.signal, "getsignal", return_value=original),
            mock.patch.object(ISOLATION.signal, "signal", side_effect=install),
            mock.patch.object(ISOLATION.subprocess, "Popen", side_effect=popen),
            self.assertRaisesRegex(ISOLATION.SmokeInterrupted, "pending signal"),
        ):
            with ISOLATION.ProcessSupervisor() as supervisor:
                supervisor.spawn("candidate", ["/candidate"])

        self.assertIn("terminate:spawned", events)
        self.assertLess(events.index("popen"), events.index("terminate:spawned"))

    def test_trace_summary_reads_only_the_case_owned_file(self) -> None:
        trace = self.root / "case" / "trace.jsonl"
        other = self.root / "other.jsonl"
        trace.parent.mkdir()
        trace.write_text(
            json.dumps({"kind": "ibus_manual_toggle_plan"}) + "\n",
            encoding="utf-8",
        )
        other.write_text(
            json.dumps({"kind": "ibus_manual_toggle_plan"}) * 5,
            encoding="utf-8",
        )

        summary = IME.trace_summary(trace)

        self.assertEqual(1, summary["records"])
        self.assertEqual(1, summary["manual_toggles"])
        self.assertEqual(1, summary["semantic_records"])

    def test_trace_summary_projects_visible_preedit_updates_and_clears(self) -> None:
        trace = self.root / "preedit.jsonl"
        trace.write_text(
            "\n".join(
                [
                    json.dumps(
                        {
                            "kind": "ibus_preedit",
                            "stage": "update",
                            "visible": True,
                            "text": "верка",
                        }
                    ),
                    json.dumps(
                        {
                            "kind": "ibus_preedit",
                            "stage": "update",
                            "visible": True,
                            "text": "ерка",
                        }
                    ),
                    json.dumps(
                        {
                            "kind": "ibus_preedit",
                            "stage": "update",
                            "visible": False,
                            "text": "ignored",
                        }
                    ),
                    json.dumps({"kind": "ibus_preedit", "stage": "clear"}),
                    json.dumps(
                        {
                            "kind": "ibus_key",
                            "stage": "printable_managed_commit",
                            "decoded": "в",
                        }
                    ),
                    json.dumps(
                        {
                            "kind": "ibus_precognition_display",
                            "stage": "retained_shortened",
                        }
                    ),
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        summary = IME.trace_summary(trace)

        self.assertEqual(["верка", "ерка"], summary["preedit_updates"])
        self.assertEqual(1, summary["preedit_clears"])
        self.assertEqual(["в"], summary["managed_commits"])
        self.assertEqual(1, summary["pending_shortens"])

    def test_case_trace_contract_rejects_text_bypass_of_candidate_ime(self) -> None:
        case = CASE_MODULE.CASES["ime_prefix_prov_pending_alt_enter"]
        trace = IME.trace_error("unused")
        trace.update(
            {
                "read_error": None,
                "kind_counts": {"ibus_focus": 8},
                "preedit_updates": [],
                "preedit_clears": 0,
                "managed_commits": [],
                "pending_shortens": 0,
            }
        )

        ok, detail = SMOKE.validate_case_trace_contract(case, trace)

        self.assertFalse(ok)
        self.assertIn("ibus_keys=0>=8", detail)
        self.assertIn(
            "preedit_updates=()/('овод', 'ивет', 'верка', 'ерка')", detail
        )

    def test_case_trace_contract_accepts_exact_pending_refresh_route(self) -> None:
        case = CASE_MODULE.CASES["ime_prefix_prov_pending_alt_enter"]
        trace = IME.trace_error("unused")
        trace.update(
            {
                "read_error": None,
                "kind_counts": {"ibus_key": 8, "ibus_preedit": 4},
                "preedit_updates": ["овод", "ивет", "верка", "ерка"],
                "preedit_clears": 1,
                "managed_commits": ["п", "р", "о", "в"],
                "pending_shortens": 1,
            }
        )

        ok, detail = SMOKE.validate_case_trace_contract(case, trace)

        self.assertTrue(ok, detail)

    def test_managed_ime_cleanup_stops_candidate_when_fallback_fails(self) -> None:
        events: list[str] = []
        engine = FakeProcess("ime", events)
        case = SimpleNamespace(
            trace_path=self.root / "case-trace.jsonl",
            environment=lambda: {},
        )
        with (
            mock.patch.object(IME, "discover_lay_ibus_engines", return_value=()),
            mock.patch.object(IME.subprocess, "Popen", return_value=engine),
            mock.patch.object(IME.time, "sleep"),
            mock.patch.object(
                IME,
                "set_ibus_engine",
                side_effect=RuntimeError("fallback failed"),
            ),
            self.assertRaisesRegex(RuntimeError, "fallback failed"),
        ):
            with IME.managed_ime_case(
                ROOT,
                Path("/candidate/lay-ibus-engine"),
                case,
                "xkb:us::eng",
            ):
                pass

        self.assertIn("terminate:ime", events)

    def test_process_identity_rejects_pid_reuse(self) -> None:
        expected = DESKTOP.ProcessIdentity(
            pid=42,
            start_time=100,
            executable="/usr/bin/lay-ibus-engine",
            argv=("lay-ibus-engine", "--ibus", "--managed"),
        )
        reused = DESKTOP.ProcessIdentity(
            pid=42,
            start_time=101,
            executable="/usr/bin/lay-ibus-engine",
            argv=("lay-ibus-engine", "--ibus", "--managed"),
        )

        self.assertFalse(DESKTOP.same_process(expected, reused))
        self.assertTrue(DESKTOP.same_process(expected, expected))

    def case_context(self):
        context = ISOLATION.prepare_case_context(self.root, "run-case", "fake")
        SMOKE.write_case_config(context.config_path, None, managed_ime=False)
        return context

    def test_dialog_start_failure_has_no_live_child_leak(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(
                EXECUTION.subprocess, "Popen", side_effect=OSError("dialog")
            ),
        ):
            ok, _, detail = SMOKE.run_case(
                case,
                context,
                Path("/input"),
                None,
                "gtk-entry-capture",
                0,
                1,
                False,
                False,
            )

        self.assertFalse(ok)
        self.assertIn("OSError: dialog", detail)

    def test_managed_ime_dialog_forces_synchronous_ibus_key_routing(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        events: list[str] = []
        dialog = FakeProcess("dialog", events, output=("expected\n", ""))
        sender = FakeProcess("sender", events)
        environments: list[dict[str, str]] = []

        def popen(*_args, **kwargs):
            environments.append(kwargs["env"])
            return (dialog, sender)[len(environments) - 1]

        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(EXECUTION.time, "sleep"),
            mock.patch.object(EXECUTION.subprocess, "Popen", side_effect=popen),
        ):
            ok, _, detail = SMOKE.run_case(
                case,
                context,
                Path("/input"),
                None,
                "gtk-entry-capture",
                0,
                1,
                False,
                True,
            )

        self.assertTrue(ok, detail)
        self.assertEqual("ibus", environments[0]["GTK_IM_MODULE"])
        self.assertEqual("1", environments[0]["IBUS_ENABLE_SYNC_MODE"])

    def test_invalid_device_path_cleans_sender_then_dialog(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        events: list[str] = []
        dialog = FakeProcess("dialog", events)
        sender = FakeProcess("sender", events, stdout_line="not-a-device\n")
        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(EXECUTION.time, "sleep"),
            mock.patch.object(
                EXECUTION,
                "readline_with_timeout",
                return_value="not-a-device\n",
            ),
            mock.patch.object(
                EXECUTION.subprocess, "Popen", side_effect=[dialog, sender]
            ),
        ):
            ok, _, detail = SMOKE.run_case(
                case,
                context,
                Path("/input"),
                Path("/daemon"),
                "gtk-entry-capture",
                0,
                1,
                False,
                False,
            )

        self.assertFalse(ok)
        self.assertIn("invalid test device path", detail)
        self.assertEqual("terminate:sender", events[0])
        self.assertIn("terminate:dialog", events)

    def test_daemon_start_failure_cleans_existing_children(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        events: list[str] = []
        dialog = FakeProcess("dialog", events)
        sender = FakeProcess(
            "sender", events, stdout_line="/dev/input/event999\n"
        )
        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(EXECUTION.time, "sleep"),
            mock.patch.object(
                EXECUTION,
                "readline_with_timeout",
                return_value="/dev/input/event999\n",
            ),
            mock.patch.object(EXECUTION, "wait_for_device_access", return_value=True),
            mock.patch.object(
                EXECUTION.subprocess,
                "Popen",
                side_effect=[dialog, sender, OSError("daemon")],
            ),
        ):
            ok, _, detail = SMOKE.run_case(
                case,
                context,
                Path("/input"),
                Path("/daemon"),
                "gtk-entry-capture",
                0,
                1,
                False,
                False,
            )

        self.assertFalse(ok)
        self.assertIn("OSError: daemon", detail)
        self.assertIn("terminate:sender", events)
        self.assertIn("terminate:dialog", events)

    def test_sender_timeout_is_a_case_failure_and_cleanup_completes(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        events: list[str] = []
        dialog = FakeProcess("dialog", events, output=("expected\n", ""))
        sender = FakeProcess("sender", events, timeout_once=True)
        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(EXECUTION.time, "sleep"),
            mock.patch.object(
                EXECUTION.subprocess, "Popen", side_effect=[dialog, sender]
            ),
        ):
            ok, got, detail = SMOKE.run_case(
                case,
                context,
                Path("/input"),
                None,
                "gtk-entry-capture",
                0,
                1,
                False,
                False,
            )

        self.assertFalse(ok)
        self.assertEqual("expected", got)
        self.assertIn("sender timeout", detail)

    def test_daemon_sigkill_during_cleanup_is_a_case_failure(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        events: list[str] = []
        dialog = FakeProcess("dialog", events, output=("expected\n", ""))
        sender = FakeProcess(
            "sender", events, stdout_line="/dev/input/event999\n"
        )
        daemon = FakeProcess("daemon", events, timeout_once=True)
        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(EXECUTION.time, "sleep"),
            mock.patch.object(
                EXECUTION,
                "readline_with_timeout",
                return_value="/dev/input/event999\n",
            ),
            mock.patch.object(EXECUTION, "wait_for_device_access", return_value=True),
            mock.patch.object(
                EXECUTION.subprocess,
                "Popen",
                side_effect=[dialog, sender, daemon],
            ),
        ):
            ok, got, detail = SMOKE.run_case(
                case,
                context,
                Path("/input"),
                Path("/daemon"),
                "gtk-entry-capture",
                0,
                1,
                False,
                False,
            )

        self.assertFalse(ok)
        self.assertEqual("expected", got)
        self.assertIn("daemon exited -9", detail)

    def test_cleanup_signal_aborts_batch_after_cleaning_started_children(self) -> None:
        context = self.case_context()
        case = CASE_MODULE.Case("fake", "expected")
        events: list[str] = []
        dialog = FakeProcess("dialog", events)
        sender = FakeProcess("sender", events)
        sender.communicate = mock.Mock(
            side_effect=[ISOLATION.SmokeInterrupted("signal 15"), ("", "")]
        )
        with (
            mock.patch.object(EXECUTION, "activate_layout"),
            mock.patch.object(EXECUTION.time, "sleep"),
            mock.patch.object(
                EXECUTION.subprocess, "Popen", side_effect=[dialog, sender]
            ),
            self.assertRaisesRegex(ISOLATION.SmokeInterrupted, "signal 15"),
        ):
            SMOKE.run_case(
                case,
                context,
                Path("/input"),
                None,
                "gtk-entry-capture",
                0,
                1,
                False,
                False,
            )

        self.assertIn("terminate:sender", events)
        self.assertIn("terminate:dialog", events)

    def test_malformed_case_trace_is_reported_not_merged(self) -> None:
        trace = self.root / "trace.jsonl"
        trace.write_text(
            '{"kind":"ibus_manual_toggle_plan"}\nnot-json\n',
            encoding="utf-8",
        )

        summary = IME.trace_summary(trace)

        self.assertEqual(1, summary["manual_toggles"])
        self.assertEqual(1, summary["malformed"])

    def test_trace_read_failure_is_explicit(self) -> None:
        trace = self.root / "trace.jsonl"
        trace.write_text("{}\n", encoding="utf-8")
        with mock.patch.object(
            Path, "read_bytes", side_effect=PermissionError("denied")
        ):
            summary = IME.trace_summary(trace)

        self.assertIn("PermissionError", str(summary["read_error"]))
        self.assertIsNone(summary["sha256"])

    def test_missing_empty_and_invalid_utf8_trace_fail_closed(self) -> None:
        missing = IME.trace_summary(self.root / "missing.jsonl")
        self.assertIn("missing", str(missing["read_error"]))

        empty_path = self.root / "empty.jsonl"
        empty_path.touch()
        empty = IME.trace_summary(empty_path)
        self.assertIn("empty", str(empty["read_error"]))

        invalid_path = self.root / "invalid.jsonl"
        invalid_path.write_bytes(b'{"kind":"ibus_key","value":"\xff"}\n')
        invalid = IME.trace_summary(invalid_path)
        self.assertIn("UnicodeDecodeError", str(invalid["read_error"]))

    def test_non_object_and_truncated_trace_rows_are_malformed(self) -> None:
        trace = self.root / "malformed-shapes.jsonl"
        trace.write_text('["not", "an", "object"]\n{"kind":', encoding="utf-8")

        summary = IME.trace_summary(trace)

        self.assertEqual(0, summary["records"])
        self.assertEqual(2, summary["malformed"])

    def test_case_projection_is_independent_of_execution_order(self) -> None:
        first = {
            "case_id": "a-id",
            "name": "a",
            "ok": True,
            "got": "a",
            "expected": "a",
            "trace": {
                "records": 3,
                "semantic_records": 2,
                "volatile_records": 1,
                "kind_counts": {"ibus_cursor": 1, "ibus_key": 2},
                "semantic_kind_counts": {"ibus_key": 2},
                "malformed": 0,
                "manual_toggles": 1,
                "read_error": None,
                "sha256": "run-specific-a",
            },
        }
        second = {
            "case_id": "b-id",
            "name": "b",
            "ok": True,
            "got": "b",
            "expected": "b",
            "trace": {
                "records": 2,
                "semantic_records": 2,
                "volatile_records": 0,
                "kind_counts": {"ibus_key": 2},
                "semantic_kind_counts": {"ibus_key": 2},
                "malformed": 0,
                "manual_toggles": 0,
                "read_error": None,
                "sha256": "run-specific-b",
            },
        }

        self.assertEqual(
            RECEIPT.case_results_sha256([first, second]),
            RECEIPT.case_results_sha256([second, first]),
        )

    def test_fatal_preflight_writes_persistent_failure_receipt(self) -> None:
        evidence = self.root / "fatal-evidence"
        previous_sync_mode = SMOKE.os.environ.get("IBUS_ENABLE_SYNC_MODE")
        argv = [
            "run_runtime_smoke.py",
            "--managed-desktop",
            "--ime-managed",
            "--verify-ime-trace",
            "--case",
            "ghbdtn_enter",
            "--evidence-dir",
            str(evidence),
            "--no-build",
        ]
        with (
            mock.patch.object(SMOKE.sys, "argv", argv),
            mock.patch.object(
                SMOKE,
                "choose_dialog_command",
                side_effect=RuntimeError("preflight failed"),
            ),
            self.assertRaisesRegex(RuntimeError, "preflight failed"),
        ):
            SMOKE.main()

        receipt_path = evidence / "RECEIPT.json"
        self.assertTrue(receipt_path.is_file())
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        self.assertEqual("RuntimeError: preflight failed", receipt["fatal_error"])
        self.assertFalse(receipt["desktop_restoration_verified"])
        self.assertFalse(receipt["all_passed"])
        self.assertEqual("1", receipt["ibus_sync_mode"])
        self.assertEqual(
            previous_sync_mode, SMOKE.os.environ.get("IBUS_ENABLE_SYNC_MODE")
        )

    def valid_receipt(
        self,
        *,
        run_id: str = "receipt-test",
        order: tuple[str, ...] = ("case-a",),
    ) -> dict[str, object]:
        rows = []
        for name in sorted(order):
            trace = {
                "records": 2,
                "semantic_records": 1,
                "volatile_records": 1,
                "kind_counts": {"ibus_cursor": 1, "ibus_key": 1},
                "semantic_kind_counts": {"ibus_key": 1},
                "malformed": 0,
                "manual_toggles": 0,
                "sha256": "1" * 64,
                "read_error": None,
            }
            rows.append(
                {
                    "case_id": ISOLATION.case_id_for(run_id, name),
                    "name": name,
                    "ok": True,
                    "got": name,
                    "expected": name,
                    "detail": "",
                    "trace": trace,
                }
            )
        digest = "2" * 64
        receipt = {
            "schema": RECEIPT.CURRENT_SCHEMA,
            "run_id": run_id,
            "selected_cases": list(order),
            "execution_order": list(order),
            "active_case_at_failure": None,
            "all_passed": True,
            "cases": rows,
            "case_results_sha256": RECEIPT.case_results_sha256(rows),
            "desktop_before": dataclasses.asdict(
                self.desktop_snapshot(active_state="active")
            ),
            "desktop_restoration_verified": True,
            "harness_process_group": 123,
            "evidence_root": str(self.root),
            "fatal_error": None,
            "binaries": {
                role: {"path": f"/candidate/{role}", "size": 1, "sha256": digest}
                for role in ("input", "daemon", "ibus_engine")
            },
            "invocation": [
                "/repo/scripts/run_runtime_smoke.py",
                "--managed-desktop",
                "--ime-managed",
                "--verify-ime-trace",
            ],
        }
        RECEIPT.validate_runtime_smoke_receipt(receipt)
        return receipt

    def test_v3_receipt_rejects_semantic_contradictions(self) -> None:
        mutations = {
            "selected case": lambda value: value.update(
                selected_cases=["different"], execution_order=["different"]
            ),
            "case_id": lambda value: value["cases"][0].update(case_id="wrong"),
            "output": lambda value: value["cases"][0].update(got="wrong"),
            "malformed": lambda value: value["cases"][0]["trace"].update(
                malformed=1
            ),
            "desktop": lambda value: value.update(desktop_before={}),
        }
        for label, mutate in mutations.items():
            with self.subTest(label=label):
                receipt = self.valid_receipt()
                mutate(receipt)
                receipt["case_results_sha256"] = RECEIPT.case_results_sha256(
                    receipt["cases"]
                )
                with self.assertRaises(ValueError):
                    RECEIPT.validate_runtime_smoke_receipt(receipt)

    def test_receipt_bound_single_forward_reverse_equivalence(self) -> None:
        run_id = "isolation-verifier"
        orders = {
            "single_a": ("case-a",),
            "single_b": ("case-b",),
            "forward": ("case-a", "case-b"),
            "reversed": ("case-b", "case-a"),
        }
        paths = {}
        for role, order in orders.items():
            path = self.root / f"{role}.json"
            path.write_text(
                json.dumps(self.valid_receipt(run_id=run_id, order=order)),
                encoding="utf-8",
            )
            paths[role] = path

        verdict = VERIFY.verify_isolation(
            [paths["single_a"], paths["single_b"]],
            paths["forward"],
            paths["reversed"],
        )

        self.assertEqual("RUNTIME_SMOKE_ORDER_ISOLATION_PASS", verdict["verdict"])

    def test_identity_change_blocks_signal(self) -> None:
        expected = DESKTOP.ProcessIdentity(
            42, 100, "/usr/bin/lay-ibus-engine", ("lay-ibus-engine", "--ibus")
        )
        reused = dataclasses.replace(expected, start_time=101)
        with (
            mock.patch.object(DESKTOP, "read_process_identity", return_value=reused),
            mock.patch.object(DESKTOP.os, "kill") as kill,
            self.assertRaisesRegex(RuntimeError, "refusing to signal"),
        ):
            DESKTOP.terminate_captured_process(expected)
        kill.assert_not_called()

    def test_gnome_layout_activation_fails_closed(self) -> None:
        refused = subprocess.CompletedProcess(
            ["gdbus"],
            0,
            stdout="(false,)\n",
            stderr="",
        )
        with (
            mock.patch.object(EXECUTION, "activate_layout_kde", return_value=False),
            mock.patch.object(EXECUTION, "gnome_layout_call", return_value=refused),
            self.assertRaisesRegex(RuntimeError, "GNOME refused layout"),
        ):
            EXECUTION.activate_layout("ru", ime_engine=False)

    def test_desktop_layout_parser_rejects_ambiguous_reply(self) -> None:
        reply = subprocess.CompletedProcess(
            ["gdbus"],
            0,
            stdout="('ru', 'us')\n",
            stderr="",
        )
        with (
            mock.patch.object(DESKTOP, "gnome_layout_call", return_value=reply),
            self.assertRaisesRegex(RuntimeError, "ambiguous GNOME current layout"),
        ):
            DESKTOP.current_desktop_layout()

    def test_engine_trace_path_comes_from_captured_process_environment(self) -> None:
        proc_root = self.root / "proc"
        process_root = proc_root / "42"
        process_root.mkdir(parents=True)
        (process_root / "environ").write_bytes(
            b"HOME=/home/test\0LAY_IBUS_TRACE_PATH=/evidence/original.jsonl\0"
        )
        identity = DESKTOP.ProcessIdentity(
            42,
            100,
            "/usr/bin/lay-ibus-engine",
            ("lay-ibus-engine", "--ibus"),
        )

        self.assertEqual(
            "/evidence/original.jsonl",
            DESKTOP.effective_engine_trace_path(identity, proc_root),
        )

    def test_process_command_identity_ignores_only_argv_zero_path_spelling(self) -> None:
        captured = DESKTOP.ProcessIdentity(
            42,
            100,
            "/home/test/.local/lib/lay/bin/lay-daemon",
            ("/home/test/.local/bin/lay-daemon", "--example"),
        )
        restored = dataclasses.replace(
            captured,
            pid=43,
            start_time=200,
            argv=("/home/test/.local/lib/lay/bin/lay-daemon", "--example"),
        )

        self.assertTrue(DESKTOP.same_process_command(captured, restored))

    def test_process_command_identity_rejects_executable_drift(self) -> None:
        captured = DESKTOP.ProcessIdentity(
            42,
            100,
            "/home/test/.local/lib/lay/bin/lay-daemon",
            ("/home/test/.local/bin/lay-daemon",),
        )
        restored = dataclasses.replace(
            captured,
            pid=43,
            start_time=200,
            executable="/tmp/lay-daemon",
            argv=("/tmp/lay-daemon",),
        )

        self.assertFalse(DESKTOP.same_process_command(captured, restored))

    def test_process_command_identity_rejects_argument_drift(self) -> None:
        captured = DESKTOP.ProcessIdentity(
            42,
            100,
            "/home/test/.local/lib/lay/bin/lay-daemon",
            ("/home/test/.local/bin/lay-daemon", "--example"),
        )
        restored = dataclasses.replace(
            captured,
            pid=43,
            start_time=200,
            argv=("/home/test/.local/lib/lay/bin/lay-daemon", "--different"),
        )

        self.assertFalse(DESKTOP.same_process_command(captured, restored))

    def desktop_snapshot(self, *, active_state: str = "inactive"):
        main = None
        main_pid = 0
        sub_state = "dead"
        if active_state == "active":
            main = DESKTOP.ProcessIdentity(
                70,
                700,
                "/usr/bin/lay-daemon",
                ("/usr/bin/lay-daemon",),
            )
            main_pid = 70
            sub_state = "running"
        return DESKTOP.DesktopSnapshot(
            service=DESKTOP.ServiceSnapshot(
                active_state,
                sub_state,
                "enabled",
                main_pid,
                main,
            ),
            active_layout="xkb:us::eng",
            active_engine="xkb:us::eng",
            ibus_daemons=(
                DESKTOP.ProcessIdentity(
                    80,
                    800,
                    "/usr/bin/ibus-daemon",
                    ("/usr/bin/ibus-daemon", "--daemonize"),
                ),
            ),
            lay_engines=(),
            lay_engine_trace_paths=(),
            harness_trace_path="/tmp/original-trace.jsonl",
        )

    def test_desktop_cleanup_attempts_every_restoration_dimension(self) -> None:
        snapshot = self.desktop_snapshot()
        calls: list[str] = []

        def fail_layout(_layout):
            calls.append("layout")
            raise RuntimeError("layout restore failed")

        with (
            mock.patch.object(DESKTOP, "capture_desktop_snapshot", return_value=snapshot),
            mock.patch.object(
                DESKTOP,
                "restore_service",
                side_effect=lambda _snapshot: calls.append("service"),
            ),
            mock.patch.object(DESKTOP, "set_desktop_layout", side_effect=fail_layout),
            mock.patch.object(
                DESKTOP,
                "set_ibus_engine",
                side_effect=lambda _engine: calls.append("engine"),
            ),
            mock.patch.object(
                DESKTOP,
                "verify_lay_engines_restored",
                side_effect=lambda *_args, **_kwargs: calls.append("lay-engines"),
            ),
            mock.patch.object(
                DESKTOP,
                "verify_ibus_daemons_unchanged",
                side_effect=lambda _expected: calls.append("ibus-daemon"),
            ),
            mock.patch.object(
                DESKTOP,
                "verify_harness_trace_path_unchanged",
                side_effect=lambda _expected: calls.append("trace-path"),
            ),
            self.assertRaisesRegex(RuntimeError, "layout restore failed"),
        ):
            with DESKTOP.managed_desktop_session(
                admitted=True,
                replace_service=True,
                replace_lay_engines=False,
            ):
                pass

        self.assertEqual(
            [
                "service",
                "layout",
                "engine",
                "lay-engines",
                "ibus-daemon",
                "trace-path",
            ],
            calls,
        )

    def test_ambiguous_service_state_blocks_before_mutation(self) -> None:
        snapshot = self.desktop_snapshot(active_state="failed")
        with (
            mock.patch.object(DESKTOP, "capture_desktop_snapshot", return_value=snapshot),
            mock.patch.object(DESKTOP, "systemctl") as systemctl,
            self.assertRaisesRegex(RuntimeError, "replaceable active/inactive"),
        ):
            with DESKTOP.managed_desktop_session(
                admitted=True,
                replace_service=True,
                replace_lay_engines=False,
            ):
                pass
        systemctl.assert_not_called()

    def test_active_service_identity_is_revalidated_before_stop(self) -> None:
        snapshot = self.desktop_snapshot(active_state="active")
        calls: list[str] = []
        with (
            mock.patch.object(DESKTOP, "capture_desktop_snapshot", return_value=snapshot),
            mock.patch.object(
                DESKTOP,
                "verify_service_unchanged",
                side_effect=lambda _snapshot: calls.append("verify-service"),
            ),
            mock.patch.object(
                DESKTOP,
                "systemctl",
                side_effect=lambda *_args, **_kwargs: calls.append("stop-service"),
            ),
            mock.patch.object(DESKTOP, "restore_service"),
            mock.patch.object(DESKTOP, "set_desktop_layout"),
            mock.patch.object(DESKTOP, "set_ibus_engine"),
            mock.patch.object(DESKTOP, "verify_lay_engines_restored"),
            mock.patch.object(DESKTOP, "verify_ibus_daemons_unchanged"),
            mock.patch.object(DESKTOP, "verify_harness_trace_path_unchanged"),
        ):
            with DESKTOP.managed_desktop_session(
                admitted=True,
                replace_service=True,
                replace_lay_engines=False,
            ):
                pass

        self.assertEqual(["verify-service", "stop-service"], calls)

    def test_desktop_session_without_admission_captures_nothing(self) -> None:
        with (
            mock.patch.object(DESKTOP, "capture_desktop_snapshot") as capture,
            self.assertRaisesRegex(RuntimeError, "not admitted"),
        ):
            with DESKTOP.managed_desktop_session(
                admitted=False,
                replace_service=True,
                replace_lay_engines=True,
            ):
                pass
        capture.assert_not_called()


if __name__ == "__main__":
    unittest.main()
