#!/usr/bin/env python3
"""Real GTK dialog/async-child checks; privilege commands use harmless fixtures."""
import importlib.util
import os
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("input_access_ui", ROOT / "scripts/lay-input-access.py")
ACCESS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ACCESS)


@unittest.skipUnless(os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"), "requires isolated graphical session")
class InputAccessUiTests(unittest.TestCase):
    def exercise(self, status, exit_code=None):
        import gi
        gi.require_version("Gtk", "3.0")
        from gi.repository import GLib, Gtk
        observed = []
        failures = []
        deadline = time.monotonic() + 6
        result = {"status": status}

        def tick():
            windows = [window for window in Gtk.Window.list_toplevels()
                       if window.get_visible() and window.get_title() == "Lay — доступ к клавиатуре"]
            if not windows:
                if time.monotonic() < deadline:
                    return True
                failures.append("dialog never became visible")
                Gtk.main_quit()
                return False
            dialog = windows[0]
            try:
                label = dialog.get_content_area().get_children()[0]
                text = label.get_text()
                button = dialog.get_widget_for_response(2)
                if not observed:
                    self.assertIn(ACCESS.explanation(result)[0], text)
                    observed.append("rendered")
                    if exit_code is None:
                        self.assertFalse(button.get_sensitive())
                        dialog.response(Gtk.ResponseType.CLOSE)
                        return False
                    self.assertTrue(button.get_sensitive())
                    dialog.response(2)
                    self.assertFalse(button.get_sensitive())
                    observed.append("started")
                elif button.get_sensitive():
                    self.assertIn(ACCESS.repair_message(exit_code, result), text)
                    self.assertTrue(dialog.get_widget_for_response(Gtk.ResponseType.CLOSE).get_sensitive())
                    observed.append("finished-and-rechecked")
                    if exit_code == 0:
                        self.assertIn("пока недоступны", text)
                        self.assertNotIn("Настройка завершена, устройства доступны", text)
                    result["status"] = "READY"
                    dialog.response(1)
                    self.assertIn("Устройства ввода доступны", label.get_text())
                    self.assertFalse(button.get_sensitive())
                    observed.append("retry-observed-recovery")
                    dialog.response(Gtk.ResponseType.CLOSE)
                    return False
                if time.monotonic() >= deadline:
                    self.fail("async child did not complete")
            except Exception as error:
                failures.append(str(error))
                dialog.destroy()
                return False
            return True

        def command(current):
            if current["status"] not in ("KEYBOARD_ACCESS", "UINPUT_ACCESS"):
                return None
            return ["/usr/bin/python3", "-c", f"raise SystemExit({exit_code})"]

        with tempfile.TemporaryDirectory(prefix="lay-access-ui-") as runtime, \
                patch.dict(os.environ, {"XDG_RUNTIME_DIR": runtime}), \
                patch.object(ACCESS, "probe", side_effect=lambda: dict(result)) as probe, \
                patch.object(ACCESS, "repair_command", side_effect=command):
            source = GLib.timeout_add(20, tick)
            try:
                self.assertEqual(ACCESS.show_window(dict(result)), 0)
            finally:
                if GLib.MainContext.default().find_source_by_id(source):
                    GLib.source_remove(source)
        self.assertEqual(failures, [])
        if exit_code is None:
            self.assertEqual(observed, ["rendered"])
        else:
            self.assertEqual(observed, ["rendered", "started", "finished-and-rechecked", "retry-observed-recovery"])
            self.assertGreaterEqual(probe.call_count, 3)

    def test_missing_keyboard_does_not_offer_privilege(self):
        self.exercise("NO_KEYBOARD")

    def test_unknown_device_failure_does_not_offer_privilege(self):
        self.exercise("DEVICE_DISCOVERY")

    def test_setup_exit_zero_does_not_turn_blocked_access_into_success(self):
        self.exercise("KEYBOARD_ACCESS", 0)

    def test_cancelled_authorization_is_visible_and_retryable(self):
        self.exercise("KEYBOARD_ACCESS", 126)

    def test_failed_setup_is_visible_and_retryable(self):
        self.exercise("UINPUT_ACCESS", 1)

    def test_close_during_setup_keeps_child_supervised_and_releases_lock(self):
        import gi
        gi.require_version("Gtk", "3.0")
        from gi.repository import Gdk, GLib, Gtk

        for close_method in ("button", "window-manager"):
            with self.subTest(close_method=close_method), tempfile.TemporaryDirectory(prefix="lay-access-close-") as runtime:
                marker = Path(runtime) / "child-finished"
                failures = []
                observed = []
                deadline = time.monotonic() + 6
                result = {"status": "KEYBOARD_ACCESS"}
                command = ["/usr/bin/python3", "-c",
                           "import sys,time; from pathlib import Path; time.sleep(.2); Path(sys.argv[1]).write_text('done')",
                           str(marker)]

                def tick():
                    windows = [window for window in Gtk.Window.list_toplevels()
                               if window.get_title() == "Lay — доступ к клавиатуре"]
                    try:
                        if windows and not observed:
                            dialog = windows[0]
                            dialog.response(2)
                            self.assertFalse(dialog.get_widget_for_response(2).get_sensitive())
                            if close_method == "button":
                                dialog.response(Gtk.ResponseType.CLOSE)
                            else:
                                self.assertTrue(dialog.emit("delete-event", Gdk.Event.new(Gdk.EventType.DELETE)))
                            self.assertFalse(dialog.get_visible())
                            self.assertIsNone(ACCESS.window_lock())
                            self.assertFalse(marker.exists())
                            observed.append("hidden-with-child-and-lock")
                        if time.monotonic() >= deadline:
                            self.fail("dialog did not finish after its child")
                    except Exception as error:
                        failures.append(str(error))
                        for window in windows:
                            window.destroy()
                        Gtk.main_quit()
                        return False
                    return True

                with patch.dict(os.environ, {"XDG_RUNTIME_DIR": runtime}), \
                        patch.object(ACCESS, "probe", return_value=result) as probe, \
                        patch.object(ACCESS, "repair_command", return_value=command):
                    source = GLib.timeout_add(20, tick)
                    try:
                        self.assertEqual(ACCESS.show_window(result), 0)
                    finally:
                        if GLib.MainContext.default().find_source_by_id(source):
                            GLib.source_remove(source)
                    self.assertEqual(failures, [])
                    self.assertEqual(observed, ["hidden-with-child-and-lock"])
                    self.assertEqual(marker.read_text(), "done")
                    self.assertGreaterEqual(probe.call_count, 2)
                    lock = ACCESS.window_lock()
                    self.assertIsNotNone(lock)
                    os.close(lock)


@unittest.skipUnless(os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"), "requires isolated graphical session")
class KdeInputAccessUiTests(unittest.TestCase):
    def test_menu_reports_previous_failure_even_when_reopened_before_status_timer(self):
        try:
            from PyQt6 import QtCore, QtWidgets
        except ImportError:
            try:
                from PyQt5 import QtCore, QtWidgets
            except ImportError:
                self.skipTest("PyQt6 or PyQt5 required for KDE menu proof")
        import sys
        spec = importlib.util.spec_from_file_location("access_tray", ROOT / "scripts/lay-kde-tray.py")
        tray = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(tray)
        app = QtWidgets.QApplication(["lay-isolated-menu-test"])
        warnings, notices, failures, observed, children = [], [], [], [], []
        deadline = time.monotonic() + 8
        original_spawn = tray.subprocess.Popen

        def spawn(argv, **kwargs):
            self.assertEqual(argv, [str(helper)])
            child = original_spawn(argv, **kwargs)
            children.append(child)
            return child

        def find_action(menu):
            for action in menu.actions():
                if "Доступ к клавиатуре" in action.text():
                    return action
                if action.menu():
                    found = find_action(action.menu())
                    if found is not None:
                        return found

        def tick():
            try:
                actions = [find_action(menu) for menu in app.topLevelWidgets() if isinstance(menu, QtWidgets.QMenu)]
                action = next((action for action in actions if action is not None), None)
                if not observed and action is not None:
                    action.trigger()
                    action.trigger()
                    self.assertEqual(len(children), 1)
                    self.assertTrue(notices)
                    observed.append("duplicate-rejected")
                elif len(observed) == 1 and children[0].poll() is not None:
                    # The existing two-second timer has not run yet.
                    self.assertLess(time.monotonic(), first_started + 1.5)
                    action.trigger()
                    self.assertEqual(len(warnings), 1)
                    self.assertEqual(len(children), 2)
                    observed.append("previous-failure-reported-before-relaunch")
                elif len(observed) == 2 and len(warnings) == 2:
                    self.assertTrue(all("Окно диагностики недоступно" in text for text in warnings))
                    observed.append("timer-reports-next-failure")
                    app.quit()
                if time.monotonic() >= deadline:
                    self.fail("KDE menu did not report child completion")
            except Exception as error:
                failures.append(str(error))
                app.quit()

        with tempfile.TemporaryDirectory(prefix="lay-qt-access-") as temporary:
            home = Path(temporary)
            helper = home / ".local/bin/lay-input-access"
            helper.parent.mkdir(parents=True)
            helper.write_text("#!/usr/bin/python3\nimport time\ntime.sleep(.2)\nraise SystemExit(7)\n")
            helper.chmod(0o755)
            timer = QtCore.QTimer()
            timer.timeout.connect(tick)
            timer.start(20)
            first_started = time.monotonic()
            with patch.object(QtWidgets, "QApplication", return_value=app), \
                    patch.object(QtWidgets.QMessageBox, "warning", side_effect=lambda *args: warnings.append(args[-1])), \
                    patch.object(QtWidgets.QMessageBox, "information", side_effect=lambda *args: notices.append(args[-1])), \
                    patch.object(tray, "acquire_single_instance_lock", return_value=object()), \
                    patch.object(tray, "daemon_active", return_value=False), \
                    patch.object(tray, "lay_version", return_value="lay isolated fixture"), \
                    patch.object(tray, "auto_update_control", return_value=(False, "")), \
                    patch.object(tray, "load_config", return_value=dict(tray.DEFAULTS)), \
                    patch.object(tray.Path, "home", return_value=home), \
                    patch.object(tray.subprocess, "Popen", side_effect=spawn), \
                    patch.object(sys, "argv", ["lay-kde-tray"]):
                try:
                    self.assertEqual(tray.main(), 0)
                finally:
                    timer.stop()
                    for child in children:
                        child.wait(timeout=2)
        self.assertEqual(failures, [])
        self.assertEqual(observed, ["duplicate-rejected", "previous-failure-reported-before-relaunch", "timer-reports-next-failure"])


if __name__ == "__main__":
    unittest.main()
