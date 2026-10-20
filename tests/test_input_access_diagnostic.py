#!/usr/bin/env python3
"""Device readiness diagnostics use isolated sysfs/dev fixtures, never input events."""
import contextlib
import errno
import importlib.util
import io
import os
from pathlib import Path
import re
import tempfile
import unittest
import subprocess
from types import SimpleNamespace
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("input_access", ROOT / "scripts/lay-input-access.py")
ACCESS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ACCESS)


class InputAccessDiagnosticTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.dev = self.root / "dev"
        self.sys = self.root / "sys"
        (self.dev / "input").mkdir(parents=True)
        self.sys.mkdir()
        (self.dev / "uinput").touch()

    def device(self, name="USB keyboard", keys=(1 << 30) | (1 << 42)):
        event = f"event{len(list((self.dev / 'input').iterdir()))}"
        node = self.dev / "input" / event
        node.touch()
        source = self.sys / event / "device"
        (source / "capabilities").mkdir(parents=True)
        (source / "name").write_text(name)
        (source / "capabilities/key").write_text(f"{keys:x}")
        return node

    def probe(self):
        return ACCESS.probe(self.dev, self.sys)

    def failed_open(self, node, error_number):
        original = ACCESS.os.open

        def open_device(path, flags, *args, **kwargs):
            if Path(path) == node:
                raise OSError(error_number, "fixture: device open failed")
            return original(path, flags, *args, **kwargs)

        return patch.object(ACCESS.os, "open", side_effect=open_device)

    def denied(self, node):
        return self.failed_open(node, errno.EACCES)

    def test_ready_opens_and_closes_but_never_reads_events(self):
        self.device()
        with patch.object(ACCESS.os, "read", side_effect=AssertionError("must not consume events")), \
                patch.object(ACCESS.os, "close", wraps=os.close) as close:
            result = self.probe()
        self.assertEqual(result["status"], "READY")
        self.assertEqual(result["readable_keyboards"], 1)
        self.assertEqual(close.call_count, 2)

    def test_existing_keyboard_with_denied_access_is_not_absent(self):
        with self.denied(self.device()):
            result = self.probe()
        self.assertEqual(result["status"], "KEYBOARD_ACCESS")
        self.assertEqual(result["keyboards"], 1)

    def test_disconnected_or_failed_keyboard_does_not_offer_permissions_repair(self):
        node = self.device()
        for number, category in ((errno.ENOENT, "missing"), (errno.ENODEV, "missing"), (errno.EIO, "unavailable")):
            with self.subTest(error=number), self.failed_open(node, number):
                result = self.probe()
            self.assertEqual(result["status"], "DEVICE_DISCOVERY")
            self.assertEqual(result["keyboard_errors"][category], 1)
            self.assertIsNone(ACCESS.repair_command(result))

    def test_uinput_io_error_does_not_offer_permissions_repair(self):
        self.device()
        with self.failed_open(self.dev / "uinput", errno.EIO):
            result = self.probe()
        self.assertEqual(result["status"], "DEVICE_DISCOVERY")
        self.assertEqual(result["uinput_error"], "unavailable")
        self.assertIsNone(ACCESS.repair_command(result))

    def test_only_service_keyboards_are_not_real_input(self):
        for name in ACCESS.IGNORED_NAMES:
            self.device(name)
        self.assertEqual(self.probe()["status"], "NO_KEYBOARD")

    def test_ignored_service_names_match_the_runtime_exclusions(self):
        # A changed daemon exclusion must not make diagnostics promise a feedback device.
        source = (ROOT / "src/bin/lay_daemon/keyboard_io.rs").read_text()
        exclusion = source.split("fn should_ignore_keyboard_device_name", 1)[1].split("}", 1)[0]
        self.assertEqual(set(re.findall(r'"([^\"]+)"', exclusion)), ACCESS.IGNORED_NAMES)

    def test_non_keyboard_permission_does_not_hide_accessible_keyboard(self):
        self.device()
        with self.denied(self.device("Power button", keys=1 << 116)):
            self.assertEqual(self.probe()["status"], "READY")

    def test_missing_input_directory_is_explained_as_no_keyboard(self):
        (self.dev / "input").rmdir()
        self.assertEqual(self.probe()["status"], "NO_KEYBOARD")

    def test_unreadable_sysfs_is_unknown_instead_of_no_keyboard(self):
        event = self.device()
        (self.sys / event.name / "device/capabilities/key").unlink()
        self.assertEqual(self.probe()["status"], "DEVICE_DISCOVERY")

    def test_uinput_denied_and_missing_are_reported(self):
        self.device()
        with self.denied(self.dev / "uinput"):
            result = self.probe()
            self.assertEqual(result["status"], "UINPUT_ACCESS")
            self.assertEqual(result["uinput_error"], "permission")
        (self.dev / "uinput").unlink()
        result = self.probe()
        self.assertEqual(result["status"], "UINPUT_ACCESS")
        self.assertEqual(result["uinput_error"], "missing")

    def test_kernel_capability_words_and_high_unrelated_keys(self):
        self.assertTrue(ACCESS.has_typing_keys("1 400 40000000", word_bits=32))
        self.assertTrue(ACCESS.has_typing_keys("100 0 40040000000", word_bits=64))
        self.assertFalse(ACCESS.has_typing_keys("40000000"))

    def test_provisioning_is_offered_only_for_a_repairable_problem(self):
        with patch.object(ACCESS.shutil, "which", side_effect=lambda name: "/usr/bin/" + name):
            for status in ("READY", "NO_KEYBOARD", "DEVICE_DISCOVERY"):
                self.assertIsNone(ACCESS.repair_command({"status": status}))
            for status in ("KEYBOARD_ACCESS", "UINPUT_ACCESS"):
                command = ACCESS.repair_command({"status": status})
                self.assertEqual(command[:6], ["/usr/bin/pkexec", "/usr/bin/timeout", "--signal=TERM",
                                               "--kill-after=5s", "30s", "/usr/bin/bash"])
                self.assertEqual(Path(command[6]), ROOT / "scripts/setup-input-access.sh")
        with patch.object(ACCESS.shutil, "which", return_value=None):
            self.assertIsNone(ACCESS.repair_command({"status": "KEYBOARD_ACCESS"}))

    def test_zero_setup_exit_without_device_access_is_not_success(self):
        message = ACCESS.repair_message(0, {"status": "KEYBOARD_ACCESS"})
        self.assertIn("пока недоступны", message)
        self.assertNotIn("устройства доступны", message)
        self.assertIn("отменена", ACCESS.repair_message(126, {"status": "KEYBOARD_ACCESS"}))
        self.assertIn("не завершена", ACCESS.repair_message(1, {"status": "READY"}))
        self.assertIn("частично", ACCESS.repair_message(124, {"status": "KEYBOARD_ACCESS"}))

    def test_json_mode_returns_failure_without_opening_a_window(self):
        with patch.object(ACCESS, "probe", return_value={"status": "NO_KEYBOARD"}), \
                patch.object(ACCESS.sys, "argv", ["lay-input-access", "--json"]), \
                patch.object(ACCESS, "show_window", side_effect=AssertionError("JSON must not show GUI")), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(ACCESS.main(), 1)
        self.assertIn("NO_KEYBOARD", output.getvalue())

    def test_automatic_check_is_silent_when_devices_are_ready(self):
        with patch.object(ACCESS, "probe", return_value={"status": "READY"}), \
                patch.object(ACCESS.sys, "argv", ["lay-input-access", "--show-if-needed"]), \
                patch.object(ACCESS, "show_window", side_effect=AssertionError("ready must not show GUI")):
            self.assertEqual(ACCESS.main(), 0)

    def test_native_remote_ready_does_not_offer_root_repair_or_show_failure(self):
        result = {"status": "NATIVE_READY"}
        self.assertIsNone(ACCESS.repair_command(result))
        self.assertIn("удалённой", ACCESS.explanation(result)[0])
        with patch.object(ACCESS, "probe", return_value=result), \
                patch.object(ACCESS.sys, "argv", ["lay-input-access", "--show-if-needed"]), \
                patch.object(ACCESS, "show_window", side_effect=AssertionError("ready must stay silent")):
            self.assertEqual(ACCESS.main(), 0)

    def test_native_probe_requires_successful_typed_capability_and_is_bounded(self):
        with patch.dict(os.environ, {"DISPLAY": ":fixture", "XDG_SESSION_TYPE": "x11"}), \
                patch.object(ACCESS.shutil, "which", return_value="/fixture/lay-daemon"), \
                patch.object(ACCESS.subprocess, "run") as run:
            for code, payload, expected in (
                    (0, '{"ready":true,"transport":"x11-xinput2"}', True),
                    (1, '{"ready":true,"transport":"x11-xinput2"}', False),
                    (0, '{"ready":true}', False), (0, 'invalid', False)):
                run.return_value = SimpleNamespace(returncode=code, stdout=payload)
                self.assertEqual(ACCESS.native_session_ready(), expected)
            self.assertEqual(run.call_args.kwargs["timeout"], 2)
            self.assertEqual(run.call_args.args[0], ["/fixture/lay-daemon", "--input-status"])
            run.side_effect = subprocess.TimeoutExpired("fixture", 2)
            self.assertFalse(ACCESS.native_session_ready())

    def test_headless_failure_is_actionable(self):
        with patch.dict(os.environ, {"DISPLAY": "", "WAYLAND_DISPLAY": ""}), \
                contextlib.redirect_stderr(io.StringIO()) as error:
            self.assertEqual(ACCESS.show_window({"status": "NO_KEYBOARD"}), 1)
        self.assertIn("lay-input-access --json", error.getvalue())

    def test_one_window_per_graphical_session_and_lock_releases(self):
        with patch.dict(os.environ, {"XDG_RUNTIME_DIR": str(self.root), "DISPLAY": ":fixture1"}):
            first = ACCESS.window_lock()
            self.assertIsNotNone(first)
            try:
                self.assertIsNone(ACCESS.window_lock())
                with patch.dict(os.environ, {"DISPLAY": ":fixture2"}):
                    separate = ACCESS.window_lock()
                    self.assertIsNotNone(separate)
                    os.close(separate)
            finally:
                os.close(first)
            reopened = ACCESS.window_lock()
            self.assertIsNotNone(reopened)
            os.close(reopened)


if __name__ == "__main__":
    unittest.main()
