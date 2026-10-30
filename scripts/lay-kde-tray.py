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


CONFIG_PATH = Path(os.environ.get("LAY_CONFIG_PATH") or Path.home() / ".config" / "lay" / "config.json")
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
    "fragment_layout_autoflip": False,
    "auto_replace": False,
    "nanda_precognition": False,
}


class InlineHintProjection:
    """Versioned display material; no keys, document text or acceptance API."""

    def __init__(self) -> None:
        self.owner = ""
        self.revision = -1

    def reset(self, owner: str) -> None:
        self.owner = owner
        self.revision = -1

    def apply(self, raw: str) -> tuple[bool, dict[str, Any] | None]:
        if not isinstance(raw, str) or len(raw) > 16_384:
            return False, None
        try:
            value = json.loads(raw)
            revision = value.get("revision")
            if (value.get("version") != 1 or value.get("stream") != self.owner
                    or not self.owner or not isinstance(revision, str)
                    or not revision.isascii() or not revision.isdecimal()
                    or len(revision) > 20):
                return False, None
            number = int(revision)
            if number <= self.revision:
                return False, None
            text, rect = value.get("text"), value.get("rect")
            if (not isinstance(text, str) or len(text) > 512
                    or any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in text)):
                return False, None
            if rect is not None and (not isinstance(rect, list) or len(rect) != 4
                    or any(type(n) is not int for n in rect)
                    or abs(rect[0]) > 100_000 or abs(rect[1]) > 100_000
                    or not 0 <= rect[2] <= 4096 or not 0 < rect[3] <= 512):
                return False, None
            if type(value.get("monospace")) is not bool:
                return False, None
            if type(value.get("outlined", False)) is not bool:
                return False, None
            style = value.get("suffix_style")
            if style is not None:
                if not isinstance(style, dict):
                    return False, None
                family = style.get("font_family")
                if family is not None and (not isinstance(family, str)
                        or not family.strip() or len(family) > 128
                        or any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in family)):
                    return False, None
        except (ValueError, TypeError, AttributeError):
            return False, None
        self.revision = number
        return True, value if text and rect is not None else None


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


def auto_update_control(action: str) -> tuple[bool | None, str]:
    try:
        result = subprocess.run(
            [str(Path.home() / ".local/bin/lay-update"), action],
            text=True, capture_output=True, timeout=5, check=False,
        )
        state = result.stdout.strip()
        if result.returncode == 0 and state in ("enabled", "disabled"):
            return state == "enabled", ""
        return None, result.stderr.strip() or "Не удалось прочитать состояние автообновления."
    except (OSError, subprocess.TimeoutExpired):
        return None, "Не удалось запустить lay-update."


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
    result = run_cmd([str(Path.home() / ".local" / "bin" / "lay-ibus-engine"), "--version"])
    prefix = "lay-ibus-engine "
    if result.returncode == 0 and result.stdout.startswith(prefix):
        return "lay " + result.stdout.removeprefix(prefix).strip()
    return "lay (IME недоступен)"


def daemon_active() -> bool:
    return runtime_control("daemon-is-active")


def runtime_control(action: str, value: str | None = None) -> bool:
    command = [str(Path.home() / ".local" / "bin" / "lay-runtime-control"), action]
    if value:
        command.append(value)
    if action in ("daemon-start", "daemon-stop", "daemon-restart"):
        result = subprocess.run(command, text=True, capture_output=True, timeout=5, check=False)
        if result.returncode:
            raise RuntimeError(result.stderr.strip() or f"Контроллер завершился с кодом {result.returncode}.")
        return True
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
        from PyQt6.QtCore import QObject, QPoint, QTimer, Qt, pyqtSlot
        from PyQt6.QtDBus import QDBusConnection, QDBusInterface, QDBusPendingCallWatcher, QDBusPendingReply, QDBusServiceWatcher
        from PyQt6.QtGui import QAction, QActionGroup, QColor, QCursor, QFont, QFontDatabase, QFontMetrics, QIcon, QPainter, QPixmap
        from PyQt6.QtWidgets import QApplication, QLabel, QMenu, QMessageBox, QSystemTrayIcon
    except ImportError:
        try:
            from PyQt5.QtCore import QObject, QPoint, QTimer, Qt, pyqtSlot
            from PyQt5.QtDBus import QDBusConnection, QDBusInterface, QDBusPendingCallWatcher, QDBusPendingReply, QDBusServiceWatcher
            from PyQt5.QtGui import QColor, QCursor, QFont, QFontDatabase, QFontMetrics, QIcon, QPainter, QPixmap
            from PyQt5.QtWidgets import QAction, QActionGroup, QApplication, QLabel, QMenu, QMessageBox, QSystemTrayIcon
        except ImportError as exc:
            print(f"lay-kde-tray: install PyQt6 or PyQt5: {exc}", file=sys.stderr)
            return 1

    class InlineHintView(QObject):
        SERVICE = "io.github.radislabus_star.LayIme"
        PATH = "/io/github/radislabus_star/LayIme"

        def __init__(self, app: Any) -> None:
            super().__init__(app)
            self.app = app
            self.projection = InlineHintProjection()
            window = getattr(Qt, "WindowType", Qt)
            attribute = getattr(Qt, "WidgetAttribute", Qt)
            self.label = QLabel(None, window.ToolTip | window.FramelessWindowHint
                                | window.WindowStaysOnTopHint | window.WindowDoesNotAcceptFocus
                                | window.WindowTransparentForInput)
            for name in ("WA_ShowWithoutActivating", "WA_TransparentForMouseEvents"):
                self.label.setAttribute(getattr(attribute, name), True)
            self.label.setFocusPolicy(getattr(Qt, "FocusPolicy", Qt).NoFocus)
            self.label.setTextFormat(getattr(Qt, "TextFormat", Qt).PlainText)
            self.label.setStyleSheet("color: #000000; background: #ffffff; border: none; margin: 0; padding: 0;")
            self.bus = QDBusConnection.sessionBus()
            self.watcher = QDBusServiceWatcher(self.SERVICE, self.bus,
                getattr(QDBusServiceWatcher, "WatchModeFlag", QDBusServiceWatcher).WatchForOwnerChange, self)
            self.watcher.serviceOwnerChanged.connect(self._owner_changed)
            self.bus.connect(self.SERVICE, self.PATH, self.SERVICE, "InlineHintV1", self.receive)
            owner = self.bus.interface().serviceOwner(self.SERVICE)
            self._owner_changed(self.SERVICE, "", owner.value() if owner.isValid() else "")
            app.aboutToQuit.connect(self.label.close)

        def _owner_changed(self, _name: str, _old: str, owner: str) -> None:
            self.label.hide()
            self.projection.reset(owner)
            if not owner:
                return
            interface = QDBusInterface(owner, self.PATH, self.SERVICE, self.bus, self)
            pending = QDBusPendingCallWatcher(interface.asyncCall("InlineHintSnapshotV1"), self)

            def completed(call: Any) -> None:
                reply = QDBusPendingReply(call)
                if owner == self.projection.owner and not reply.isError():
                    self.receive(reply.value())
                call.deleteLater()
                interface.deleteLater()

            pending.finished.connect(completed)

        @pyqtSlot(str)
        def receive(self, raw: str) -> None:
            accepted, frame = self.projection.apply(raw)
            if not accepted:
                return
            if frame is None:
                self.label.hide()
                return
            x, y, _width, height = frame["rect"]
            screen = self.app.screenAt(QPoint(x, y))
            if screen is None:
                self.label.hide()
                return
            font = (QFontDatabase.systemFont(getattr(QFontDatabase, "SystemFont", QFontDatabase).FixedFont)
                    if frame["monospace"] else QFont(self.app.font()))
            # A tall Entry may report its full area as the cursor rectangle.
            # QLabel centers the ordinary theme font within that area.
            if height <= QFontMetrics(font).height() * 2:
                font.setPixelSize(max(8, int(height * 0.78)))
            self.label.setFont(font)
            border = "1px solid #777777" if frame.get("outlined", False) else "none"
            self.label.setStyleSheet(
                f"color: #000000; background: #ffffff; border: {border}; margin: 0; padding: 0;"
            )
            self.label.setText(frame["text"])
            metrics = QFontMetrics(font)
            available = screen.geometry().right() - x + 1
            self.label.resize(max(1, min(available, metrics.horizontalAdvance(frame["text"]) + 2)), height)
            self.label.move(x, y)
            self.label.clearMask()
            if frame.get("suffix_style") is not None:
                from lay_kde_firefox_hint import render_suffix
                render_suffix(self.label, frame["suffix_style"], available, height)
            # This view owns only the unaccepted suffix. Keep its complete
            # white rectangle opaque, including on noncomposited desktops.
            self.label.show()

    class LayTray:
        def __init__(self) -> None:
            self.app = QApplication(sys.argv)
            self.app.setApplicationName("lay")
            self.app.setQuitOnLastWindowClosed(False)
            self.hint_view = InlineHintView(self.app)
            # The updater restarts this tray after installation. Its terminal
            # must survive tray destruction, including with kitty and xterm.
            self.update_process: subprocess.Popen[bytes] | None = None
            self.input_access_process: subprocess.Popen[bytes] | None = None
            self.menu = QMenu()
            self.tray = QSystemTrayIcon(self._icon(daemon_active()), self.app)
            self.tray.activated.connect(self._activated)
            self.timer = QTimer()
            self.timer.timeout.connect(self._refresh_status)
            self.timer.start(2000)
            self._build_menu()
            self.tray.setContextMenu(self.menu)
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
            self.menu.addSeparator()

            enabled = QAction("Lay включён", self.menu)
            self.enabled_action = enabled
            enabled.setCheckable(True)
            enabled.setChecked(daemon_active())
            enabled.triggered.connect(self._set_enabled)
            self.menu.addAction(enabled)
            daemon_menu = self.menu.addMenu("Демон Lay")
            self.daemon_actions = {}
            for operation, label in (("start", "Запустить демон"),
                                     ("stop", "Остановить демон"),
                                     ("restart", "Перезапустить демон")):
                action = QAction(label, daemon_menu)
                action.triggered.connect(lambda _checked, selected=operation: self._control_daemon(selected))
                daemon_menu.addAction(action)
                self.daemon_actions[operation] = action
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

            self.menu.addSeparator()
            self._add_switch("Помощь при наборе", "typing_assist", cfg)
            self._add_switch("Автозамена", "auto_replace", cfg)
            self._add_switch("Автопереворот по фрагментам", "fragment_layout_autoflip", cfg)
            self.menu.addSeparator()

            settings = QAction("Настройки", self.menu)
            settings.triggered.connect(self._open_settings)
            self.menu.addAction(settings)

            updates = QAction(QIcon.fromTheme("system-software-update"), "Проверить обновления", self.menu)
            updates.triggered.connect(self._open_updates)
            self.menu.addAction(updates)

            auto_update = QAction("Автообновление", self.menu)
            auto_update.setCheckable(True)
            state, error = auto_update_control("--auto-status")
            auto_update.setChecked(state is True)
            auto_update.setEnabled(state is not None)
            auto_update.setToolTip(error or "Стабильные релизы; отключение не прерывает уже начатую установку")
            auto_update.triggered.connect(self._set_auto_update)
            self.menu.addAction(auto_update)

            diagnostics = self.menu.addMenu("Диагностика")
            diagnostic_status = QAction("Службы: работают" if daemon_active() else "Службы: остановлены", diagnostics)
            diagnostic_status.setEnabled(False)
            diagnostics.addAction(diagnostic_status)
            self.status_action = diagnostic_status
            access = QAction("Доступ к клавиатуре…", diagnostics)
            access.triggered.connect(self._open_input_access)
            diagnostics.addAction(access)
            ime_setup = QAction("Настроить подсказки…", diagnostics)
            ime_setup.triggered.connect(self._open_ime_setup)
            diagnostics.addAction(ime_setup)
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
            close_tray.triggered.connect(self._close_tray)
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

        def _control_daemon(self, operation: str) -> None:
            detail = "Контроллер вернул ошибку."
            try:
                succeeded = runtime_control(f"daemon-{operation}")
            except subprocess.TimeoutExpired:
                succeeded = False
                detail = "Команда не ответила за 5 с. Состояние демона будет проверено повторно."
            except (OSError, RuntimeError) as error:
                succeeded = False
                detail = str(error) or detail
            if not succeeded:
                QMessageBox.warning(None, "Lay", f"Не удалось выполнить команду демона.\n{detail}")
            self._refresh_status()

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

        def _set_auto_update(self, enabled: bool) -> None:
            state, error = auto_update_control("--auto-enable" if enabled else "--auto-disable")
            if state is None:
                QMessageBox.warning(None, "Автообновление Lay", error)
            self._build_menu()

        def _open_updates(self) -> None:
            if self.update_process is not None and self.update_process.poll() is None:
                return
            try:
                self.update_process = subprocess.Popen(
                    [str(Path.home() / ".local/bin/lay-update"), "--menu"],
                    start_new_session=True,
                )
            except OSError:
                QMessageBox.warning(None, "Обновление Lay", "Не удалось запустить lay-update. Переустанови Lay.")

        def _close_tray(self) -> None:
            if self.update_process is not None and self.update_process.poll() is None:
                QMessageBox.information(None, "Обновление Lay", "Дождись завершения обновления и закрой его терминал.")
                return
            self.app.quit()

        def _input_access_running(self) -> bool:
            if self.input_access_process is None:
                return False
            code = self.input_access_process.poll()
            if code is None:
                return True
            self.input_access_process = None
            if code:
                QMessageBox.warning(None, "Lay", "Окно диагностики недоступно. Повтори установку Lay или запусти lay-input-access --json.")
            return False

        def _open_input_access(self) -> None:
            if self._input_access_running():
                QMessageBox.information(None, "Lay", "Диагностика уже открыта или настройка ещё выполняется. Заверши или отмени запрос прав администратора.")
                return
            try:
                self.input_access_process = subprocess.Popen(
                    [str(Path.home() / ".local/bin/lay-input-access")], start_new_session=True)
            except OSError:
                QMessageBox.warning(None, "Lay", "Не удалось открыть диагностику доступа. Повтори установку Lay.")

        def _open_logs(self) -> None:
            if not open_logs():
                QMessageBox.warning(None, "Lay", "Не найден терминал для журнала")

        def _open_ime_setup(self) -> None:
            try:
                subprocess.Popen([str(Path.home() / ".local/bin/lay-ime-setup"), "--gui"],
                                 start_new_session=True)
            except OSError:
                QMessageBox.warning(None, "Lay", "Не удалось открыть настройку подсказок. Повтори установку Lay.")

        def _refresh_status(self) -> None:
            self._input_access_running()
            if self.update_process is not None:
                code = self.update_process.poll()
                if code is not None:
                    self.update_process = None
                    if code:
                        QMessageBox.warning(None, "Обновление Lay", "Не удалось выполнить обновление или открыть терминал. Запусти lay-update в терминале.")
            active = daemon_active()
            self.enabled_action.setChecked(active)
            self.tray.setIcon(self._icon(active))
            self.tray.setToolTip(f"Lay\nСлужбы: {'работают' if active else 'остановлены'}")
            self.status_action.setText(f"Службы: {'работают' if active else 'остановлены'}")
            for operation, action in self.daemon_actions.items():
                action.setEnabled(not active if operation == "start" else active)

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
