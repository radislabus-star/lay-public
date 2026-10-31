"""Exercise the real Qt tray menu with private config and controller fixtures."""

from contextlib import ExitStack
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


@unittest.skipUnless(os.environ.get("DISPLAY") or os.environ.get("QT_QPA_PLATFORM") in ("offscreen", "minimal"),
                     "requires an isolated graphical session")
class KdeCheckboxMenuTests(unittest.TestCase):
    def test_real_pointer_keyboard_state_and_dismissal(self):
        if not any(importlib.util.find_spec(name) for name in ("PyQt6", "PyQt5")):
            self.skipTest("requires PyQt5 or PyQt6")
        # QApplication owns process-wide native state; do not share it with
        # the existing daemon-menu fixture or later DBus/GJS tests.
        run = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--qt-case"],
                             text=True, capture_output=True, timeout=30)
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)

    def _exercise_real_menu(self):
        try:
            from PyQt6 import QtCore, QtGui, QtTest, QtWidgets
        except ImportError:
            try:
                from PyQt5 import QtCore, QtGui, QtTest, QtWidgets
            except ImportError:
                self.skipTest("requires PyQt5 or PyQt6")
        spec = importlib.util.spec_from_file_location("checkbox_menu_tray", ROOT / "scripts/lay-kde-tray.py")
        tray = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(tray)
        app = QtWidgets.QApplication.instance() or QtWidgets.QApplication(["lay-isolated-checkbox-menu"])
        key = getattr(QtCore.Qt, "Key", QtCore.Qt)
        button = getattr(QtCore.Qt, "MouseButton", QtCore.Qt)
        commands, failures, opened = [], [], []
        active, auto_update = [True], [False]

        def control(operation):
            commands.append(operation)
            if operation in ("start", "stop"):
                active[0] = operation == "start"
            return True

        def updates(operation):
            if operation in ("--auto-enable", "--auto-disable"):
                commands.append(operation)
                auto_update[0] = operation == "--auto-enable"
            return auto_update[0], ""

        def exercise():
            try:
                menu = next(widget for widget in app.topLevelWidgets()
                            if isinstance(widget, QtWidgets.QMenu)
                            and any(action.text() == "Помощь при наборе" for action in widget.actions()))
                icons = app.findChildren(QtWidgets.QSystemTrayIcon)
                self.assertEqual(len(icons), 1)
                self.assertIs(icons[0].contextMenu(), menu)
                self.assertEqual(os.environ.get("KSNI_NO_DBUSMENU"), "1")
                activation = getattr(QtWidgets.QSystemTrayIcon, "ActivationReason", QtWidgets.QSystemTrayIcon)
                icons[0].activated.emit(activation.Trigger)
                app.processEvents()
                self.assertTrue(menu.isVisible())
                QtTest.QTest.keyClick(menu, key.Key_Escape)

                def action(label):
                    return next(item for item in menu.actions() if item.text() == label)

                def click(label):
                    QtTest.QTest.mouseClick(menu, button.LeftButton,
                                           pos=menu.actionGeometry(action(label)).center())
                    app.processEvents()

                def popup():
                    icons[0].activated.emit(activation.Context)
                    app.processEvents()
                    self.assertTrue(menu.isVisible())

                popup()
                batch_start = len(commands)
                for label, config_key in (("Помощь при наборе", "typing_assist"),
                                          ("Автозамена", "auto_replace"),
                                          ("Автопереворот по фрагментам", "fragment_layout_autoflip")):
                    before = len(commands)
                    click(label)
                    self.assertTrue(menu.isVisible(), "checkbox selection must keep the same menu visible")
                    self.assertEqual(commands[before:], [], "settings apply once when the menu closes")
                    self.assertTrue(json.loads(tray.CONFIG_PATH.read_text())[config_key])
                    self.assertTrue(action(label).isChecked())
                click("Автопереворот по фрагментам")
                self.assertFalse(tray.load_config()["fragment_layout_autoflip"])
                self.assertTrue(menu.isVisible())

                menu.setActiveAction(action("Помощь при наборе"))
                before = len(commands)
                QtTest.QTest.keyClick(menu, key.Key_Space)
                app.processEvents()
                self.assertFalse(tray.load_config()["typing_assist"])
                self.assertEqual(commands[before:], [])
                self.assertTrue(menu.isVisible())
                menu.setActiveAction(action("Помощь при наборе"))
                QtTest.QTest.keyClick(menu, key.Key_Return)
                app.processEvents()
                self.assertTrue(tray.load_config()["typing_assist"])
                self.assertTrue(menu.isVisible())
                before = list(commands)
                event_type = getattr(QtCore.QEvent, "Type", QtCore.QEvent)
                modifier = getattr(QtCore.Qt, "KeyboardModifier", QtCore.Qt)
                repeat = QtGui.QKeyEvent(event_type.KeyPress, key.Key_Space,
                                         modifier.NoModifier, " ", True, 1)
                app.sendEvent(menu, repeat)
                self.assertEqual(commands, before, "held-key autorepeat must not toggle again")
                self.assertTrue(menu.isVisible())
                QtTest.QTest.keyClick(menu, key.Key_Escape)
                app.processEvents()
                self.assertFalse(menu.isVisible())
                self.assertEqual(commands[batch_start:], ["restart"], "six toggles require one apply")
                popup()

                for label, expected in (("Lay включён", "stop"), ("Lay включён", "start"),
                                         ("Автообновление", "--auto-enable"),
                                         ("Автообновление", "--auto-disable")):
                    before = len(commands)
                    click(label)
                    self.assertEqual(commands[before:], [expected])
                    self.assertTrue(menu.isVisible())
                self.assertTrue(action("Lay включён").isChecked())
                self.assertFalse(action("Автообновление").isChecked())
                with patch.object(tray, "runtime_control", return_value=False) as failed_stop:
                    click("Lay включён")
                    failed_stop.assert_called_once_with("stop")
                    self.assertTrue(action("Lay включён").isChecked(), "failed stop must show actual state")
                with patch.object(tray, "auto_update_control", side_effect=[(None, "isolated failure"), (False, "")]), \
                        patch.object(QtWidgets.QMessageBox, "warning") as warning:
                    click("Автообновление")
                    warning.assert_called_once()
                    self.assertFalse(action("Автообновление").isChecked())
                    self.assertTrue(action("Автообновление").isEnabled())

                before = list(commands)
                title = menu.actions()[0]
                self.assertFalse(title.isEnabled())
                QtTest.QTest.mouseClick(menu, button.LeftButton, pos=menu.actionGeometry(title).center())
                self.assertEqual(commands, before)
                self.assertTrue(menu.isVisible())
                QtTest.QTest.keyClick(menu, key.Key_Escape)
                app.processEvents()
                self.assertFalse(menu.isVisible())

                popup()
                QtTest.QTest.mouseClick(menu, button.LeftButton, pos=QtCore.QPoint(-5, -5))
                app.processEvents()
                self.assertFalse(menu.isVisible(), "outside click must retain normal Qt dismissal")

                popup()
                mode = next(item.menu() for item in menu.actions() if item.text().startswith("Режим ввода:"))
                group = mode.actions()[0].actionGroup()
                radio = type(action("Автозамена"))("isolated radio control", menu)
                radio.setCheckable(True)
                radio.setActionGroup(group)
                radios = []
                radio.triggered.connect(lambda checked: radios.append(checked))
                menu.addAction(radio)
                app.processEvents()
                click("isolated radio control")
                self.assertEqual(radios, [True])
                self.assertFalse(menu.isVisible(), "exclusive radio actions keep normal Qt behavior")

                popup()
                self.assertTrue(action("Автозамена").isChecked())
                click("Настройки")
                self.assertEqual(opened, ["settings"])
                self.assertFalse(menu.isVisible(), "ordinary actions must still close the menu")

                # KDE's platform theme presents another QMenu with bridged
                # actions. Exercise that widget type as well as our menu.
                projection = QtWidgets.QMenu()
                source = action("Автозамена")
                proxy = type(source)(source.text(), projection)
                proxy.setCheckable(True)
                proxy.setChecked(source.isChecked())
                proxy.triggered.connect(lambda checked: source.trigger())
                source.toggled.connect(proxy.setChecked)
                projection.addAction(proxy)
                projection.popup(QtCore.QPoint(60, 60))
                app.processEvents()
                before = len(commands)
                QtTest.QTest.mouseClick(projection, button.LeftButton,
                                       pos=projection.actionGeometry(proxy).center())
                app.processEvents()
                self.assertTrue(projection.isVisible(), "platform-projected checkboxes must remain open")
                self.assertFalse(tray.load_config()["auto_replace"])
                self.assertEqual(commands[before:], [])
                projection.setActiveAction(proxy)
                before = len(commands)
                QtTest.QTest.keyClick(projection, key.Key_Space)
                app.processEvents()
                self.assertTrue(projection.isVisible())
                self.assertTrue(tray.load_config()["auto_replace"])
                self.assertEqual(commands[before:], [])
                QtTest.QTest.keyClick(projection, key.Key_Escape)
                app.processEvents()
                self.assertFalse(projection.isVisible())
                self.assertEqual(commands[before:], ["restart"])

                popup()
                before = len(commands)
                click("Автозамена")
                click("Lay включён")
                QtTest.QTest.keyClick(menu, key.Key_Escape)
                app.processEvents()
                self.assertEqual(commands[before:], ["stop"], "explicit stop cancels pending reload")

                popup()
                click("Автозамена")
                with patch.object(tray, "runtime_control", return_value=False) as failed_apply, \
                        patch.object(QtWidgets.QMessageBox, "warning") as warning:
                    QtTest.QTest.keyClick(menu, key.Key_Escape)
                    app.processEvents()
                    failed_apply.assert_called_once_with("restart")
                    warning.assert_called_once()
                    self.assertFalse(menu.isVisible())
            except BaseException as error:
                failures.append(error)
            finally:
                for widget in app.topLevelWidgets():
                    widget.close()
                app.quit()

        with tempfile.TemporaryDirectory(prefix="lay-checkbox-menu-") as temporary, ExitStack() as stack:
            stack.enter_context(patch.object(tray, "CONFIG_PATH", Path(temporary) / "config.json"))
            tray.save_config(dict(tray.DEFAULTS))
            for owner, name, kwargs in (
                    (QtWidgets, "QApplication", {"return_value": app}),
                    (QtWidgets.QSystemTrayIcon, "show", {}),
                    (tray, "acquire_single_instance_lock", {"return_value": object()}),
                    (tray, "daemon_active", {"side_effect": lambda: active[0]}),
                    (tray, "runtime_control", {"side_effect": control}),
                    (tray, "auto_update_control", {"side_effect": updates}),
                    (tray, "lay_version", {"return_value": "lay isolated checkbox fixture"}),
                    (tray, "load_recent_actions", {"return_value": []}),
                    (tray, "open_settings", {"side_effect": lambda: opened.append("settings") or True}),
                    (sys, "argv", {"new": ["lay-kde-tray"]})):
                stack.enter_context(patch.object(owner, name, **kwargs))
            QtCore.QTimer.singleShot(0, exercise)
            self.assertEqual(tray.main(), 0)
        if failures:
            raise failures[0]


if __name__ == "__main__":
    if sys.argv[1:] == ["--qt-case"]:
        KdeCheckboxMenuTests("_exercise_real_menu")._exercise_real_menu()
    else:
        unittest.main()
