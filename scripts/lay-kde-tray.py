#!/usr/bin/env python3
"""Compact KDE/Plasma tray frontend for Lay."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


CONFIG_PATH = Path.home() / ".config" / "lay" / "config.json"
RECENT_ACTIONS_PATH = Path.home() / ".local" / "share" / "lay" / "recent_actions.jsonl"
STATE_DIR = Path.home() / ".local" / "state" / "lay"
TRAY_LOCK_PATH = STATE_DIR / "kde-tray.lock"
PROJECT_DIR = Path(__file__).resolve().parents[1]
INSTALLED_SETTINGS_JS = (
    Path.home()
    / ".local"
    / "share"
    / "gnome-shell"
    / "extensions"
    / "lay@radislabus-star.github.io"
    / "settings.js"
)
DEFAULTS: dict[str, Any] = {
    "text_backend": "uinput",
    "typing_assist": False,
    "auto_replace": False,
    "nanda_precognition": False,
}


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, text=True, capture_output=True, check=False)


def acquire_single_instance_lock() -> Any | None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    lock_file = TRAY_LOCK_PATH.open("w", encoding="utf-8")
    try:
        fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        return None
    lock_file.write(str(os.getpid()))
    lock_file.flush()
    return lock_file


def load_config() -> dict[str, Any]:
    cfg = dict(DEFAULTS)
    try:
        loaded = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    except Exception:
        return cfg
    if isinstance(loaded, dict):
        cfg.update(loaded)
    if cfg.get("text_backend") == "auto":
        cfg["text_backend"] = "ime"
    return cfg


def save_config(cfg: dict[str, Any]) -> None:
    CONFIG_PATH.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = CONFIG_PATH.with_suffix(".json.tmp")
    tmp_path.write_text(json.dumps(cfg, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    tmp_path.replace(CONFIG_PATH)


def lay_version() -> str:
    result = run_cmd([str(Path.home() / ".local" / "bin" / "lay"), "--version"])
    return (result.stdout or result.stderr).strip() or "lay"


def daemon_active() -> bool:
    return run_cmd(["systemctl", "--user", "is-active", "--quiet", "lay-daemon.service"]).returncode == 0


def runtime_control(action: str, value: str | None = None) -> bool:
    command = [str(Path.home() / ".local" / "bin" / "lay-runtime-control"), action]
    if value:
        command.append(value)
    return run_cmd(command).returncode == 0


def input_mode_label(value: Any) -> str:
    return "IME-подсказки" if value == "ime" else "Быстрый ввод"


def switch_kde_layout() -> bool:
    qdbus = next(
        (binary for binary in ("qdbus6", "qdbus-qt6", "qdbus") if shutil.which(binary)),
        None,
    )
    if not qdbus:
        return False
    current = run_cmd([qdbus, "org.kde.keyboard", "/Layouts", "getLayout"])
    try:
        current_index = int(current.stdout.strip())
    except (TypeError, ValueError):
        return False
    target_index = 1 if current_index == 0 else 0
    return run_cmd(
        [qdbus, "org.kde.keyboard", "/Layouts", "setLayout", str(target_index)]
    ).returncode == 0


def action_kind_label(kind: Any) -> str:
    return {
        "layout-replay": "Двойной Shift",
        "smart-text": "Умная замена",
        "auto-replace": "Автозамена",
        "typing-assist": "Помощь",
        "enter-autocorrect": "Enter",
        "auto-undo": "Откат",
    }.get(str(kind), str(kind or "действие"))


def load_recent_actions(limit: int = 5) -> list[dict[str, Any]]:
    try:
        lines = [line for line in RECENT_ACTIONS_PATH.read_text(encoding="utf-8").splitlines() if line]
    except Exception:
        return []
    actions: list[dict[str, Any]] = []
    for line in reversed(lines[-limit:]):
        try:
            action = json.loads(line)
        except Exception:
            continue
        if isinstance(action, dict):
            actions.append(action)
    return actions


def recent_action_label(action: dict[str, Any]) -> str:
    before = " ".join(str(action.get("from", "")).split())
    after = " ".join(str(action.get("to", "")).split())
    if len(before) > 22:
        before = before[:19] + "..."
    if len(after) > 22:
        after = after[:19] + "..."
    return f"{action_kind_label(action.get('kind'))}: {before} → {after} · {int(action.get('elapsed_ms', 0))}мс"


def settings_script() -> Path:
    return INSTALLED_SETTINGS_JS if INSTALLED_SETTINGS_JS.exists() else (
        PROJECT_DIR / "extension" / "lay@radislabus-star.github.io" / "settings.js"
    )


def open_settings() -> bool:
    try:
        subprocess.Popen(["gjs", "-m", str(settings_script())], start_new_session=True)
        return True
    except Exception:
        return False


def open_logs() -> bool:
    command = (
        "journalctl --user -u lay-daemon.service -u lay-l3-online.service "
        "-u lay-l2-online.service -n 250 --no-pager; "
        "printf '\\nНажми Enter, чтобы закрыть окно...'; read -r"
    )
    variants = [
        ("konsole", ["-e", "bash", "-lc", command]),
        ("gnome-terminal", ["--", "bash", "-lc", command]),
        ("kgx", ["--", "bash", "-lc", command]),
        ("xterm", ["-e", "bash", "-lc", command]),
    ]
    for binary, args in variants:
        if not shutil.which(binary):
            continue
        try:
            subprocess.Popen([binary, *args], start_new_session=True)
            return True
        except Exception:
            continue
    return False


def status_text() -> str:
    cfg = load_config()
    return (
        f"{lay_version()}\n"
        f"службы={'работают' if daemon_active() else 'остановлены'}\n"
        f"режим_ввода={cfg.get('text_backend')}\n"
        f"помощь_при_наборе={bool(cfg.get('typing_assist'))}\n"
        f"автозамена={bool(cfg.get('auto_replace'))}\n"
        f"конфиг={CONFIG_PATH}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="KDE tray frontend for Lay")
    parser.add_argument("--status", action="store_true", help="print runtime status and exit")
    args = parser.parse_args()
    if args.status:
        print(status_text())
        return 0

    try:
        from PyQt6.QtCore import QTimer, Qt
        from PyQt6.QtGui import QAction, QActionGroup, QColor, QCursor, QIcon, QPainter, QPixmap
        from PyQt6.QtWidgets import QApplication, QMenu, QMessageBox, QSystemTrayIcon
    except Exception as exc:
        print(f"lay-kde-tray: PyQt6 is not available: {exc}", file=sys.stderr)
        return 1

    class LayTray:
        def __init__(self) -> None:
            self.app = QApplication(sys.argv)
            self.app.setApplicationName("lay")
            self.app.setQuitOnLastWindowClosed(False)
            self.menu = QMenu()
            self.tray = QSystemTrayIcon(self._icon(daemon_active()), self.app)
            self.tray.activated.connect(self._activated)
            self.timer = QTimer()
            self.timer.timeout.connect(self._refresh_status)
            self.timer.start(2000)
            self._build_menu()
            self.tray.show()

        @staticmethod
        def _icon(active: bool) -> QIcon:
            pixmap = QPixmap(48, 48)
            pixmap.fill(Qt.GlobalColor.transparent)
            painter = QPainter(pixmap)
            painter.setRenderHint(QPainter.RenderHint.Antialiasing)
            painter.setBrush(QColor("#20242a"))
            painter.setPen(Qt.PenStyle.NoPen)
            painter.drawRoundedRect(4, 4, 40, 40, 7, 7)
            painter.setBrush(QColor("#2ec27e" if active else "#e01b24"))
            painter.drawEllipse(31, 31, 9, 9)
            painter.setPen(QColor("#ffffff"))
            font = painter.font()
            font.setBold(True)
            font.setPointSize(15)
            painter.setFont(font)
            painter.drawText(pixmap.rect(), Qt.AlignmentFlag.AlignCenter, "L")
            painter.end()
            return QIcon(pixmap)

        def _build_menu(self) -> None:
            cfg = load_config()
            self.menu.clear()

            title = QAction(f"Lay {lay_version().removeprefix('lay ')}", self.menu)
            title.setEnabled(False)
            self.menu.addAction(title)
            self.status_action = QAction("", self.menu)
            self.status_action.setEnabled(False)
            self.menu.addAction(self.status_action)
            self.menu.addSeparator()

            layout = QAction("Переключить раскладку RU / EN", self.menu)
            layout.triggered.connect(self._switch_layout)
            self.menu.addAction(layout)

            mode_menu = self.menu.addMenu(f"Режим ввода: {input_mode_label(cfg.get('text_backend'))}")
            mode_group = QActionGroup(mode_menu)
            mode_group.setExclusive(True)
            for value, label in (("uinput", "Быстрый ввод"), ("ime", "IME-подсказки")):
                action = QAction(label, mode_menu)
                action.setCheckable(True)
                action.setChecked(cfg.get("text_backend") == value)
                action.triggered.connect(lambda _checked, selected=value: self._set_input_mode(selected))
                mode_group.addAction(action)
                mode_menu.addAction(action)

            enabled = QAction("Lay включён", self.menu)
            enabled.setCheckable(True)
            enabled.setChecked(daemon_active())
            enabled.triggered.connect(self._set_enabled)
            self.menu.insertAction(mode_menu.menuAction(), enabled)

            self._add_switch("Помощь при наборе", "typing_assist", cfg)
            self._add_switch("Автозамена", "auto_replace", cfg)
            self.menu.addSeparator()

            settings = QAction("Настройки", self.menu)
            settings.triggered.connect(self._open_settings)
            self.menu.addAction(settings)

            diagnostics = self.menu.addMenu("Диагностика")
            diagnostic_status = QAction("Службы: работают" if daemon_active() else "Службы: остановлены", diagnostics)
            diagnostic_status.setEnabled(False)
            diagnostics.addAction(diagnostic_status)
            logs = QAction("Открыть журнал", diagnostics)
            logs.triggered.connect(self._open_logs)
            diagnostics.addAction(logs)
            recent = diagnostics.addMenu("Последние действия")
            actions = load_recent_actions()
            if not actions:
                empty = QAction("пока нет действий", recent)
                empty.setEnabled(False)
                recent.addAction(empty)
            for item in actions:
                row = QAction(recent_action_label(item), recent)
                row.setEnabled(False)
                recent.addAction(row)

            close_tray = QAction("Закрыть значок", diagnostics)
            close_tray.triggered.connect(self.app.quit)
            diagnostics.addAction(close_tray)
            self._refresh_status()

        def _add_switch(self, label: str, key: str, cfg: dict[str, Any]) -> None:
            action = QAction(label, self.menu)
            action.setCheckable(True)
            action.setChecked(bool(cfg.get(key)))
            action.triggered.connect(lambda checked, config_key=key: self._set_bool(config_key, checked))
            self.menu.addAction(action)

        def _set_bool(self, key: str, value: bool) -> None:
            cfg = load_config()
            cfg[key] = value
            save_config(cfg)
            runtime_control("restart")
            self._build_menu()

        def _set_enabled(self, enabled: bool) -> None:
            runtime_control("start" if enabled else "stop")
            self._build_menu()

        def _switch_layout(self) -> None:
            if not switch_kde_layout():
                QMessageBox.warning(None, "Lay", "Не удалось переключить раскладку KDE")

        def _set_input_mode(self, value: str) -> None:
            cfg = load_config()
            cfg["text_backend"] = value
            cfg["nanda_precognition"] = value == "ime"
            save_config(cfg)
            runtime_control("channel", value)
            self._build_menu()

        def _open_settings(self) -> None:
            if not open_settings():
                QMessageBox.warning(None, "Lay", "Не удалось открыть настройки")

        def _open_logs(self) -> None:
            if not open_logs():
                QMessageBox.warning(None, "Lay", "Не найден терминал для журнала")

        def _refresh_status(self) -> None:
            active = daemon_active()
            self.tray.setIcon(self._icon(active))
            self.tray.setToolTip(f"Lay\nСлужбы: {'работают' if active else 'остановлены'}")
            self.status_action.setText(f"Службы: {'работают' if active else 'остановлены'}")

        def _activated(self, reason: QSystemTrayIcon.ActivationReason) -> None:
            if reason in (QSystemTrayIcon.ActivationReason.Trigger, QSystemTrayIcon.ActivationReason.Context):
                self._build_menu()
                self.menu.popup(QCursor.pos())

        def run(self) -> int:
            return self.app.exec()

    lock = acquire_single_instance_lock()
    if lock is None:
        print("lay-kde-tray: already running", file=sys.stderr)
        return 0
    return LayTray().run()


if __name__ == "__main__":
    raise SystemExit(main())
