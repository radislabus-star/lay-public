#!/usr/bin/env python3
"""Small KDE/Plasma tray frontend for lay.

The GNOME UI is a Shell extension. KDE cannot load that extension, so this
process provides the same basic controls through Qt's StatusNotifier tray icon.
It intentionally shares the daemon config file instead of duplicating behavior.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


CONFIG_PATH = Path.home() / ".config" / "lay" / "config.json"
CONFIG_DEFAULTS: dict[str, Any] = {
    "mode": "simple",
    "correction_engine": "smart",
    "layout_backend": "auto",
    "trigger": "double-lshift",
    "tap_max_ms": 200,
    "shift_window_ms": 250,
    "debounce_ms": 50,
    "replace_words": 1,
    "auto_replace": False,
    "typing_assist": False,
    "auto_switch_layout": True,
    "lem_2_words": True,
    "lem_3_words": True,
    "learning_log": False,
}


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, text=True, capture_output=True, check=False)


def load_config() -> dict[str, Any]:
    cfg = dict(CONFIG_DEFAULTS)
    try:
        loaded = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return cfg
    except Exception:
        return cfg
    if isinstance(loaded, dict):
        cfg.update(loaded)
    return cfg


def save_config(cfg: dict[str, Any]) -> None:
    CONFIG_PATH.parent.mkdir(parents=True, exist_ok=True)
    CONFIG_PATH.write_text(
        json.dumps(cfg, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def lay_version() -> str:
    out = run_cmd([str(Path.home() / ".local" / "bin" / "lay"), "--version"])
    text = (out.stdout or out.stderr).strip()
    return text or "lay"


def daemon_active() -> bool:
    return run_cmd(["systemctl", "--user", "is-active", "--quiet", "lay-daemon.service"]).returncode == 0


def service_action(action: str) -> bool:
    return run_cmd(["systemctl", "--user", action, "lay-daemon.service"]).returncode == 0


def config_status_text() -> str:
    cfg = load_config()
    return (
        f"{lay_version()}\n"
        f"daemon={'active' if daemon_active() else 'stopped'}\n"
        f"engine={cfg.get('correction_engine') or cfg.get('mode')}\n"
        f"scope={cfg.get('replace_words')}\n"
        f"typing_assist={bool(cfg.get('typing_assist'))}\n"
        f"auto_replace={bool(cfg.get('auto_replace'))}\n"
        f"config={CONFIG_PATH}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="KDE tray frontend for lay")
    parser.add_argument("--status", action="store_true", help="print daemon/config status and exit")
    args = parser.parse_args()
    if args.status:
        print(config_status_text())
        return 0

    try:
        from PyQt6.QtCore import QTimer, Qt
        from PyQt6.QtGui import QAction, QActionGroup, QColor, QIcon, QPainter, QPixmap
        from PyQt6.QtWidgets import QApplication, QMenu, QMessageBox, QSystemTrayIcon
    except Exception as exc:
        print(f"lay-kde-tray: PyQt6 is not available: {exc}", file=sys.stderr)
        return 1

    class LayTray:
        def __init__(self) -> None:
            self.app = QApplication(sys.argv)
            self.app.setApplicationName("lay")
            self.app.setQuitOnLastWindowClosed(False)

            if not QSystemTrayIcon.isSystemTrayAvailable():
                print("lay-kde-tray: system tray is not available", file=sys.stderr)

            self.tray = QSystemTrayIcon(self.make_icon(daemon_active()), self.app)
            self.menu = QMenu()
            self.tray.setContextMenu(self.menu)
            self.tray.activated.connect(self.on_activated)
            self.menu.aboutToShow.connect(self.rebuild_menu)

            self.timer = QTimer()
            self.timer.timeout.connect(self.refresh_status)
            self.timer.start(2000)

            self.rebuild_menu()
            self.tray.show()

        def run(self) -> int:
            return self.app.exec()

        def make_icon(self, active: bool) -> QIcon:
            theme_icon = QIcon.fromTheme("input-keyboard")
            if not theme_icon.isNull():
                return theme_icon

            pixmap = QPixmap(48, 48)
            pixmap.fill(Qt.GlobalColor.transparent)
            painter = QPainter(pixmap)
            painter.setRenderHint(QPainter.RenderHint.Antialiasing)
            painter.setBrush(QColor("#202124"))
            painter.setPen(Qt.PenStyle.NoPen)
            painter.drawRoundedRect(4, 4, 40, 40, 8, 8)
            painter.setBrush(QColor("#2ecc71" if active else "#e74c3c"))
            painter.drawEllipse(30, 30, 10, 10)
            painter.setPen(QColor("#ffffff"))
            font = painter.font()
            font.setBold(True)
            font.setPointSize(15)
            painter.setFont(font)
            painter.drawText(pixmap.rect(), Qt.AlignmentFlag.AlignCenter, "L")
            painter.end()
            return QIcon(pixmap)

        def refresh_status(self) -> None:
            active = daemon_active()
            cfg = load_config()
            self.tray.setIcon(self.make_icon(active))
            self.tray.setToolTip(
                "lay\n"
                f"Daemon: {'active' if active else 'stopped'}\n"
                f"Mode: {cfg.get('correction_engine') or cfg.get('mode')}\n"
                f"Scope: {cfg.get('replace_words')}"
            )

        def rebuild_menu(self) -> None:
            cfg = load_config()
            self.menu.clear()

            title = QAction(f"Lay KDE Tray  {lay_version()}", self.menu)
            title.setEnabled(False)
            self.menu.addAction(title)

            status = QAction(f"Daemon: {'active' if daemon_active() else 'stopped'}", self.menu)
            status.setEnabled(False)
            self.menu.addAction(status)
            self.menu.addSeparator()

            daemon_toggle = QAction("Daemon active", self.menu)
            daemon_toggle.setCheckable(True)
            daemon_toggle.setChecked(daemon_active())
            daemon_toggle.triggered.connect(lambda checked: self.set_daemon(checked))
            self.menu.addAction(daemon_toggle)

            restart = QAction("Restart daemon", self.menu)
            restart.triggered.connect(lambda: self.run_service_action("restart"))
            self.menu.addAction(restart)
            self.menu.addSeparator()

            smart = QAction("Smart correction", self.menu)
            smart.setCheckable(True)
            smart.setChecked((cfg.get("correction_engine") or cfg.get("mode")) == "smart")
            smart.triggered.connect(lambda checked: self.update_config("correction_engine", "smart" if checked else "replay"))
            self.menu.addAction(smart)

            scope_menu = self.menu.addMenu("Scope")
            scope_group = QActionGroup(scope_menu)
            scope_group.setExclusive(True)
            for value in (1, 2, 3):
                action = QAction(f"{value} word{'s' if value > 1 else ''}", scope_menu)
                action.setCheckable(True)
                action.setChecked(int(cfg.get("replace_words", 1)) == value)
                action.triggered.connect(lambda _checked, chosen=value: self.update_config("replace_words", chosen))
                scope_group.addAction(action)
                scope_menu.addAction(action)

            trigger_menu = self.menu.addMenu("Trigger")
            trigger_group = QActionGroup(trigger_menu)
            trigger_group.setExclusive(True)
            for key, label in (
                ("double-lshift", "Double left Shift"),
                ("double-rshift", "Double right Shift"),
                ("caps-lock", "Caps Lock"),
            ):
                action = QAction(label, trigger_menu)
                action.setCheckable(True)
                action.setChecked(cfg.get("trigger") == key)
                action.triggered.connect(lambda _checked, chosen=key: self.update_config("trigger", chosen))
                trigger_group.addAction(action)
                trigger_menu.addAction(action)

            self.menu.addSeparator()
            self.add_bool_action("Typing assist", "typing_assist", cfg)
            self.add_bool_action("Auto-replace", "auto_replace", cfg)
            self.add_bool_action("Auto-switch layout", "auto_switch_layout", cfg)
            self.add_bool_action("Remember corrections", "learning_log", cfg)

            advanced = self.menu.addMenu("Arbiter")
            self.add_bool_action("LEM for 2 words", "lem_2_words", cfg, advanced)
            self.add_bool_action("LEM for 3 words", "lem_3_words", cfg, advanced)

            self.menu.addSeparator()
            about = QAction("About lay", self.menu)
            about.triggered.connect(self.show_about)
            self.menu.addAction(about)

            quit_action = QAction("Quit tray", self.menu)
            quit_action.triggered.connect(self.app.quit)
            self.menu.addAction(quit_action)

            self.refresh_status()

        def add_bool_action(
            self,
            label: str,
            key: str,
            cfg: dict[str, Any],
            menu: QMenu | None = None,
        ) -> None:
            target_menu = menu or self.menu
            action = QAction(label, target_menu)
            action.setCheckable(True)
            action.setChecked(bool(cfg.get(key)))
            action.triggered.connect(lambda checked, config_key=key: self.update_config(config_key, bool(checked)))
            target_menu.addAction(action)

        def update_config(self, key: str, value: Any) -> None:
            cfg = load_config()
            cfg[key] = value
            if key == "correction_engine":
                cfg["mode"] = "simple"
            save_config(cfg)
            self.run_service_action("restart", notify=False)
            self.rebuild_menu()

        def set_daemon(self, checked: bool) -> None:
            self.run_service_action("start" if checked else "stop")
            self.rebuild_menu()

        def run_service_action(self, action: str, notify: bool = True) -> None:
            ok = service_action(action)
            if notify and not ok:
                QMessageBox.warning(None, "lay", f"systemctl --user {action} lay-daemon.service failed")
            self.refresh_status()

        def show_about(self) -> None:
            QMessageBox.about(
                None,
                "About lay",
                "<b>lay</b><br>"
                "Double Shift RU/EN layout rescue for Linux desktops.<br><br>"
                f"{lay_version()}<br>"
                "KDE tray frontend uses the same config and lay-daemon service.<br><br>"
                'GitHub: <a href="https://github.com/radislabus-star/lay-public">'
                "https://github.com/radislabus-star/lay-public</a>",
            )

        def on_activated(self, reason: QSystemTrayIcon.ActivationReason) -> None:
            if reason == QSystemTrayIcon.ActivationReason.Trigger:
                self.rebuild_menu()
                self.menu.popup(self.tray.geometry().center())

    return LayTray().run()


if __name__ == "__main__":
    raise SystemExit(main())
