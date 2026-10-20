"""Daemon-only KDE controls: real shell dispatch and real Qt action wiring."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]


def load_tray():
    spec = importlib.util.spec_from_file_location("daemon_control_tray", ROOT / "scripts/lay-kde-tray.py")
    tray = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tray)
    return tray


class DaemonControllerTests(unittest.TestCase):
    def test_daemon_commands_have_a_bounded_wait(self):
        tray = load_tray()
        for operation in ("start", "stop", "restart"):
            with self.subTest(operation=operation), patch.object(tray.subprocess, "run") as run:
                run.return_value = subprocess.CompletedProcess([], 0, "", "")
                self.assertTrue(tray.runtime_control(f"daemon-{operation}"))
                self.assertEqual(run.call_args.kwargs.get("timeout"), 5)

    def test_daemon_failure_keeps_controller_diagnostic(self):
        tray = load_tray()
        with patch.object(tray.subprocess, "run", return_value=subprocess.CompletedProcess([], 7, "", "isolated manager failure")):
            with self.assertRaisesRegex(RuntimeError, "isolated manager failure"):
                tray.runtime_control("daemon-stop")

    def test_full_runtime_commands_keep_existing_dispatch(self):
        tray = load_tray()
        for operation in ("start", "stop", "restart"):
            with self.subTest(operation=operation), patch.object(tray, "run_cmd") as run:
                run.return_value = subprocess.CompletedProcess([], 0, "", "")
                self.assertTrue(tray.runtime_control(operation))
                run.assert_called_once_with([str(Path.home() / ".local/bin/lay-runtime-control"), operation])


class DaemonDispatchTests(unittest.TestCase):
    def dispatch(self, action, exit_code=0):
        with tempfile.TemporaryDirectory(prefix="lay-daemon-control-") as temporary:
            home = Path(temporary)
            commands = home / "commands.jsonl"
            binary_dir = home / "bin"
            binary_dir.mkdir()
            stub = """#!/usr/bin/python3
import json, os, sys
from pathlib import Path
with Path(os.environ['COMMAND_LOG']).open('a') as stream:
    stream.write(json.dumps([Path(sys.argv[0]).name, *sys.argv[1:]]) + '\\n')
if sys.argv[1:] == ['--user', 'show-environment']:
    raise SystemExit(0)
raise SystemExit(int(os.environ['ACTION_EXIT']))
"""
            for name in ("systemctl", "ibus", "gdbus", "pgrep"):
                executable = binary_dir / name
                executable.write_text(stub)
                executable.chmod(0o755)
            environment = dict(os.environ, HOME=str(home), COMMAND_LOG=str(commands),
                               ACTION_EXIT=str(exit_code), DISPLAY=":999",
                               XAUTHORITY=str(home / "authority"), XDG_RUNTIME_DIR=str(home),
                               DBUS_SESSION_BUS_ADDRESS="unix:path=/nonexistent-isolated-bus",
                               PATH=f"{binary_dir}:/usr/bin:/bin")
            result = subprocess.run(["bash", str(ROOT / "scripts/lay-runtime-control.sh"), action],
                                    env=environment, capture_output=True, text=True, timeout=5)
            return result, [json.loads(line) for line in commands.read_text().splitlines()]

    def test_actions_control_only_the_daemon_unit(self):
        for operation in ("start", "stop", "restart"):
            with self.subTest(operation=operation):
                result, commands = self.dispatch(f"daemon-{operation}")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(commands, [["systemctl", "--user", "show-environment"],
                                            ["systemctl", "--user", operation, "lay-daemon.service"]])

    def test_systemctl_failure_is_not_hidden(self):
        result, commands = self.dispatch("daemon-stop", exit_code=7)
        self.assertEqual(result.returncode, 7)
        self.assertEqual(commands[-1], ["systemctl", "--user", "stop", "lay-daemon.service"])


@unittest.skipUnless(os.environ.get("DISPLAY") or os.environ.get("QT_QPA_PLATFORM") in ("offscreen", "minimal"),
                     "requires isolated graphical session")
class KdeDaemonMenuTests(unittest.TestCase):
    def test_actual_menu_dispatch_state_and_errors(self):
        try:
            from PyQt6 import QtCore, QtWidgets
        except ImportError:
            from PyQt5 import QtCore, QtWidgets
        tray = load_tray()
        app = QtWidgets.QApplication(["lay-isolated-daemon-menu"])
        active = [False]
        calls, warnings, failures = [], [], []
        timer_type, timers = QtCore.QTimer, []

        def timer():
            value = timer_type()
            timers.append(value)
            return value

        def control(action):
            calls.append(action)
            if action in ("start", "daemon-start"):
                active[0] = True
            elif action in ("stop", "daemon-stop"):
                active[0] = False
            return True

        def exercise():
            try:
                menu = next(widget for widget in app.topLevelWidgets()
                            if isinstance(widget, QtWidgets.QMenu)
                            and widget.title() == "Демон Lay")
                start, stop, restart = menu.actions()
                master = next(action for widget in app.topLevelWidgets() if isinstance(widget, QtWidgets.QMenu)
                              for action in widget.actions() if action.text() == "Lay включён")
                self.assertEqual([action.text() for action in menu.actions()],
                                 ["Запустить демон", "Остановить демон", "Перезапустить демон"])
                self.assertEqual([action.isEnabled() for action in menu.actions()], [True, False, False])
                self.assertFalse(master.isChecked())
                start.trigger()
                self.assertEqual([action.isEnabled() for action in menu.actions()], [False, True, True])
                self.assertTrue(master.isChecked())
                restart.trigger()
                with patch.object(tray, "runtime_control", return_value=False):
                    stop.trigger()
                self.assertEqual(len(warnings), 1)
                self.assertTrue(stop.isEnabled())
                with patch.object(tray, "runtime_control", side_effect=FileNotFoundError("isolated controller missing")):
                    stop.trigger()
                self.assertEqual(len(warnings), 2)
                self.assertIn("isolated controller missing", warnings[-1])
                with patch.object(tray, "runtime_control", side_effect=subprocess.TimeoutExpired("daemon-stop", 5)):
                    stop.trigger()
                self.assertEqual(len(warnings), 3)
                self.assertIn("5", warnings[-1])
                self.assertTrue(master.isChecked())
                self.assertTrue(stop.isEnabled())
                stop.trigger()
                self.assertEqual([action.isEnabled() for action in menu.actions()], [True, False, False])
                self.assertFalse(master.isChecked())
                self.assertEqual(calls, ["daemon-start", "daemon-restart", "daemon-stop"])
                active[0] = True
                timers[0].timeout.emit()
                self.assertTrue(master.isChecked())
                self.assertFalse(start.isEnabled())
                active[0] = False
                timers[0].timeout.emit()
                self.assertFalse(master.isChecked())
                self.assertTrue(start.isEnabled())
                master.trigger()
                self.assertTrue(active[0])
                # The full-runtime toggle rebuilds its menu; use the new QAction.
                master = next(action for widget in app.topLevelWidgets() if isinstance(widget, QtWidgets.QMenu)
                              for action in widget.actions() if action.text() == "Lay включён")
                master.trigger()
                self.assertFalse(active[0])
                self.assertEqual(calls[-2:], ["start", "stop"])
                setup = next(action for widget in app.topLevelWidgets() if isinstance(widget, QtWidgets.QMenu)
                             for action in widget.actions() if action.text() == "Настроить подсказки…")
                with patch.object(tray.subprocess, "Popen") as launch:
                    setup.trigger()
                    launch.assert_called_once_with(
                        [str(Path.home() / ".local/bin/lay-ime-setup"), "--gui"],
                        start_new_session=True)
                with patch.object(tray.subprocess, "Popen", side_effect=FileNotFoundError("isolated helper missing")):
                    setup.trigger()
                self.assertEqual(len(warnings), 4)
                self.assertIn("настройку подсказок", warnings[-1])
            except Exception as error:
                failures.append(str(error))
            finally:
                app.quit()

        with patch.object(QtWidgets, "QApplication", return_value=app), \
                patch.object(QtWidgets.QMessageBox, "warning", side_effect=lambda *args: warnings.append(args[-1])), \
                patch.object(tray, "acquire_single_instance_lock", return_value=object()), \
                patch.object(tray, "daemon_active", side_effect=lambda: active[0]), \
                patch.object(tray, "runtime_control", side_effect=control), \
                patch.object(tray, "lay_version", return_value="lay isolated fixture"), \
                patch.object(tray, "auto_update_control", return_value=(False, "")), \
                patch.object(tray, "load_config", return_value=dict(tray.DEFAULTS)), \
                patch.object(tray, "load_recent_actions", return_value=[]), \
                patch.object(QtCore, "QTimer", side_effect=timer), \
                patch.object(sys, "argv", ["lay-kde-tray"]):
            timer_type.singleShot(0, exercise)
            self.assertEqual(tray.main(), 0)
        self.assertEqual(failures, [])


if __name__ == "__main__":
    unittest.main()
