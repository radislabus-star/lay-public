#!/usr/bin/env python3
"""Small KDE/Plasma tray frontend for lay.

The GNOME UI is a Shell extension. KDE cannot load that extension, so this
process provides the same basic controls through Qt's StatusNotifier tray icon.
It intentionally shares the daemon config file instead of duplicating behavior.
"""

from __future__ import annotations

import argparse
import fcntl
import html
import json
import math
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


CONFIG_PATH = Path.home() / ".config" / "lay" / "config.json"
RECENT_ACTIONS_PATH = Path.home() / ".local" / "share" / "lay" / "recent_actions.jsonl"
PROJECT_DIR = Path(__file__).resolve().parents[1]
UPDATE_LOG_PATH = Path.home() / ".local" / "state" / "lay" / "update.log"
TRAY_LOCK_PATH = Path.home() / ".local" / "state" / "lay" / "kde-tray.lock"
CONFIG_DEFAULTS: dict[str, Any] = {
    "mode": "simple",
    "correction_engine": "smart",
    "layout_backend": "auto",
    "text_backend": "uinput",
    "trigger": "double-lshift",
    "force_layout_hotkeys": False,
    "force_ru_key": "single-rctrl",
    "force_en_key": "single-ralt",
    "multi_tap_scope": False,
    "multi_tap_max_taps": 4,
    "tap_max_ms": 200,
    "shift_window_ms": 250,
    "debounce_ms": 50,
    "replace_words": 1,
    "typing_assist_words": 2,
    "auto_replace": False,
    "typing_assist": False,
    "correction_safety": "normal",
    "enter_autocorrect": False,
    "auto_switch_layout": True,
    "lem_2_words": True,
    "lem_3_words": True,
    "learning_log": False,
}


def run_cmd(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, text=True, capture_output=True, check=False)


def acquire_single_instance_lock() -> Any | None:
    TRAY_LOCK_PATH.parent.mkdir(parents=True, exist_ok=True)
    lock_file = TRAY_LOCK_PATH.open("w", encoding="utf-8")
    try:
        fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        return None
    lock_file.write(str(os.getpid()))
    lock_file.flush()
    return lock_file


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
    tmp_path = CONFIG_PATH.with_suffix(CONFIG_PATH.suffix + ".tmp")
    tmp_path.write_text(
        json.dumps(cfg, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    tmp_path.replace(CONFIG_PATH)


def lay_version() -> str:
    out = run_cmd([str(Path.home() / ".local" / "bin" / "lay"), "--version"])
    text = (out.stdout or out.stderr).strip()
    return text or "lay"


def nanda_profile_text() -> str:
    out = run_cmd([str(Path.home() / ".local" / "bin" / "lay"), "--nanda-profile"])
    text = (out.stdout or out.stderr).strip()
    return text or "Клетка: 64 КБ · активно авто"


def nanda_status_text() -> str:
    out = run_cmd([str(Path.home() / ".local" / "bin" / "lay"), "--nanda-status"])
    text = (out.stdout or out.stderr).strip()
    return text or "NANDA: статус недоступен"


def daemon_active() -> bool:
    return run_cmd(["systemctl", "--user", "is-active", "--quiet", "lay-daemon.service"]).returncode == 0


def service_action(action: str) -> bool:
    return run_cmd(["systemctl", "--user", action, "lay-daemon.service"]).returncode == 0


def start_update() -> tuple[bool, str]:
    update_script = PROJECT_DIR / "update.sh"
    if not update_script.exists():
        return False, f"Не найден update.sh: {update_script}"
    UPDATE_LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    project_arg = shell_quote(str(PROJECT_DIR))
    log_arg = shell_quote(str(UPDATE_LOG_PATH))

    update_command = (
        f"cd {project_arg} && "
        f"bash update.sh 2>&1 | tee {log_arg}; "
        "code=${PIPESTATUS[0]}; "
        f"printf '\\nЛог: %s\\n\\n' {log_arg}; "
        "read -r -p 'Нажми Enter, чтобы закрыть окно...'; "
        "exit ${code}"
    )

    terminal = first_existing_command(["konsole", "kgx", "gnome-terminal", "xterm"])
    try:
        if terminal == "konsole":
            subprocess.Popen(
                ["konsole", "--workdir", str(PROJECT_DIR), "-e", "bash", "-lc", update_command],
                start_new_session=True,
            )
            return True, f"Проверка открыта в терминале. Лог: {UPDATE_LOG_PATH}"
        if terminal == "kgx":
            subprocess.Popen(
                ["kgx", "--working-directory", str(PROJECT_DIR), "--", "bash", "-lc", update_command],
                start_new_session=True,
            )
            return True, f"Проверка открыта в терминале. Лог: {UPDATE_LOG_PATH}"
        if terminal == "gnome-terminal":
            subprocess.Popen(
                ["gnome-terminal", "--working-directory", str(PROJECT_DIR), "--", "bash", "-lc", update_command],
                start_new_session=True,
            )
            return True, f"Проверка открыта в терминале. Лог: {UPDATE_LOG_PATH}"
        if terminal == "xterm":
            subprocess.Popen(
                ["xterm", "-e", "bash", "-lc", update_command],
                start_new_session=True,
            )
            return True, f"Проверка открыта в терминале. Лог: {UPDATE_LOG_PATH}"

        background_command = (
            f"cd {project_arg} && "
            f"bash update.sh > {log_arg} 2>&1"
        )
        subprocess.Popen(
            ["bash", "-lc", background_command],
            start_new_session=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return True, f"Терминал не найден, проверка запущена в фоне. Лог: {UPDATE_LOG_PATH}"
    except Exception as exc:
        return False, str(exc)


def first_existing_command(names: list[str]) -> str | None:
    for name in names:
        if shutil.which(name):
            return name
    return None


def shell_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


def config_status_text() -> str:
    cfg = load_config()
    return (
        f"{lay_version()}\n"
        f"демон={'работает' if daemon_active() else 'остановлен'}\n"
        f"режим={cfg.get('correction_engine') or cfg.get('mode')}\n"
        f"осторожность={cfg.get('correction_safety', 'normal')}\n"
        f"backend_раскладки={cfg.get('layout_backend', 'auto')}\n"
        f"вставка={cfg.get('text_backend', 'uinput')}\n"
        f"область={cfg.get('replace_words')}\n"
        f"помощь_при_наборе={bool(cfg.get('typing_assist'))}\n"
        f"автоподмена={bool(cfg.get('auto_replace'))}\n"
        f"nanda={nanda_profile_text()}\n"
        f"конфиг={CONFIG_PATH}"
    )


def load_recent_actions(limit: int = 5) -> list[dict[str, Any]]:
    try:
        lines = [
            line
            for line in RECENT_ACTIONS_PATH.read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
    except Exception:
        return []
    out: list[dict[str, Any]] = []
    for line in lines[-limit:]:
        try:
            value = json.loads(line)
        except Exception:
            continue
        if isinstance(value, dict):
            out.append(value)
    out.reverse()
    return out


def action_kind_label(kind: Any) -> str:
    return {
        "layout-replay": "Двойной Shift",
        "smart-text": "Умная замена",
        "auto-replace": "Автоподмена",
        "typing-assist": "Помощь",
        "enter-autocorrect": "Enter",
        "layout-text-fallback": "Резерв",
        "auto-undo": "Откат",
    }.get(str(kind), str(kind or "action"))


def recent_actions_summary(actions: list[dict[str, Any]]) -> str:
    if not actions:
        return "нет действий"
    counts: dict[str, int] = {}
    elapsed = 0
    undo = 0
    for item in actions:
        kind = str(item.get("kind", "action"))
        counts[kind] = counts.get(kind, 0) + 1
        elapsed += int(item.get("elapsed_ms", 0) or 0)
        if item.get("undo_available"):
            undo += 1
    top = " · ".join(
        f"{action_kind_label(kind)}:{count}"
        for kind, count in sorted(counts.items(), key=lambda pair: pair[1], reverse=True)[:3]
    )
    return f"{len(actions)} действий · среднее {round(elapsed / len(actions))}мс · undo {undo} · {top}"


def main() -> int:
    parser = argparse.ArgumentParser(description="KDE tray frontend for lay")
    parser.add_argument("--status", action="store_true", help="print daemon/config status and exit")
    args = parser.parse_args()
    if args.status:
        print(config_status_text())
        return 0

    try:
        from PyQt6.QtCore import QTimer, Qt
        from PyQt6.QtGui import QAction, QActionGroup, QColor, QCursor, QIcon, QPainter, QPixmap
        from PyQt6.QtWidgets import QApplication, QDialog, QLabel, QMenu, QMessageBox, QScrollArea, QSystemTrayIcon, QVBoxLayout, QWidget
    except Exception as exc:
        print(f"lay-kde-tray: PyQt6 is not available: {exc}", file=sys.stderr)
        return 1

    class NandaWaveWidget(QWidget):
        def __init__(self) -> None:
            super().__init__()
            self.setMinimumHeight(230)

        def paintEvent(self, _event: Any) -> None:
            painter = QPainter(self)
            painter.setRenderHint(QPainter.RenderHint.Antialiasing)
            width = max(1, self.width())
            height = max(1, self.height())
            waves = [
                ("генератор", 1.0, 0.00, QColor(46, 140, 242)),
                ("раскладка", 1.6, 0.72, QColor(51, 191, 107)),
                ("защита", 2.1, 1.35, QColor(237, 115, 64)),
                ("контекст", 1.25, 2.00, QColor(163, 115, 242)),
                ("память", 1.8, 2.65, QColor(242, 173, 64)),
                ("сетка", 0.7, 3.10, QColor(26, 178, 198)),
            ]

            painter.fillRect(0, 0, width, height, QColor(20, 24, 28, 12))
            painter.setPen(QColor(80, 80, 80, 45))
            for y in range(28, height - 24, 28):
                painter.drawLine(18, y, width - 18, y)

            left = 72
            right = width - 18
            span = max(1, right - left)
            mid = height / 2 + 8

            for index, (name, freq, phase, color) in enumerate(waves):
                base = 36 + index * 26
                color.setAlpha(190)
                painter.setPen(color)
                previous: tuple[int, int] | None = None
                for x in range(span + 1):
                    t = x / span
                    y = base + math.sin(t * math.pi * 2 * freq + phase) * 8
                    point = (left + x, int(y))
                    if previous is not None:
                        painter.drawLine(previous[0], previous[1], point[0], point[1])
                    previous = point
                painter.setPen(QColor(35, 35, 35, 220))
                painter.drawText(12, base + 4, name)

            painter.setPen(QColor(5, 5, 5, 230))
            previous = None
            for x in range(span + 1):
                t = x / span
                total = sum(math.sin(t * math.pi * 2 * freq + phase) for _, freq, phase, _ in waves)
                y = mid + total / len(waves) * 28
                point = (left + x, int(y))
                if previous is not None:
                    painter.drawLine(previous[0], previous[1], point[0], point[1])
                    painter.drawLine(previous[0], previous[1] + 1, point[0], point[1] + 1)
                previous = point
            painter.drawText(12, height - 12, "несущая мода ансамбля")
            painter.end()

    class LayTray:
        def __init__(self) -> None:
            self.app = QApplication(sys.argv)
            self.app.setApplicationName("lay")
            self.app.setQuitOnLastWindowClosed(False)

            if not QSystemTrayIcon.isSystemTrayAvailable():
                print("lay-kde-tray: system tray is not available", file=sys.stderr)

            self.tray = QSystemTrayIcon(self.make_icon(daemon_active()), self.app)
            self.menu = QMenu()
            self._menu_position = None
            self.tray.activated.connect(self.on_activated)

            self.timer = QTimer()
            self.timer.timeout.connect(self.refresh_status)
            self.timer.start(2000)

            self.rebuild_menu()
            self.tray.show()

        def run(self) -> int:
            return self.app.exec()

        def make_icon(self, active: bool) -> QIcon:
            pixmap = QPixmap(48, 48)
            pixmap.fill(Qt.GlobalColor.transparent)
            painter = QPainter(pixmap)
            painter.setRenderHint(QPainter.RenderHint.Antialiasing)
            painter.setBrush(QColor("#1f2937"))
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

        def show_menu_at_cursor(self) -> None:
            self._menu_position = QCursor.pos()
            self.rebuild_menu()
            self.menu.popup(self._menu_position)

        def reopen_menu(self) -> None:
            self.rebuild_menu()
            self.menu.popup(self._menu_position or QCursor.pos())

        def refresh_status(self) -> None:
            active = daemon_active()
            cfg = load_config()
            self.tray.setIcon(self.make_icon(active))
            self.tray.setToolTip(
                "lay\n"
                f"Демон: {'работает' if active else 'остановлен'}\n"
                f"Режим: {self.engine_label(cfg)}\n"
                f"Среда: {self.layout_backend_label(cfg.get('layout_backend', 'auto'))}\n"
                f"Область: {cfg.get('replace_words')} сл."
            )

        def rebuild_menu(self) -> None:
            cfg = load_config()
            self.menu.clear()

            title = QAction(f"Lay для KDE  {lay_version()}", self.menu)
            title.setEnabled(False)
            self.menu.addAction(title)

            status = QAction(
                f"Демон: {'работает' if daemon_active() else 'остановлен'}",
                self.menu,
            )
            status.setEnabled(False)
            self.menu.addAction(status)
            self.menu.addSeparator()

            daemon_toggle = QAction("Демон включён", self.menu)
            daemon_toggle.setCheckable(True)
            daemon_toggle.setChecked(daemon_active())
            daemon_toggle.triggered.connect(lambda checked: self.set_daemon(checked))
            self.menu.addAction(daemon_toggle)

            restart = QAction("Перезапустить демон", self.menu)
            restart.triggered.connect(lambda: self.run_service_action("restart"))
            self.menu.addAction(restart)

            self.add_layout_backend_menu(self.menu, cfg, keep_open=True)

            update = QAction("Проверить обновления", self.menu)
            update.triggered.connect(self.run_update)
            self.menu.addAction(update)
            self.menu.addSeparator()

            smart = QAction("Умная коррекция", self.menu)
            smart.setCheckable(True)
            smart.setChecked((cfg.get("correction_engine") or cfg.get("mode")) == "smart")
            smart.triggered.connect(lambda checked: self.update_config("correction_engine", "smart" if checked else "replay"))
            self.menu.addAction(smart)

            safety_menu = self.menu.addMenu("Осторожность")
            safety_group = QActionGroup(safety_menu)
            safety_group.setExclusive(True)
            for value, label in (
                ("strict", "Строго"),
                ("normal", "Норма"),
                ("experimental", "Экспериментально"),
            ):
                action = QAction(label, safety_menu)
                action.setCheckable(True)
                action.setChecked(str(cfg.get("correction_safety", "normal")) == value)
                action.triggered.connect(lambda _checked, chosen=value: self.update_config("correction_safety", chosen))
                safety_group.addAction(action)
                safety_menu.addAction(action)

            scope_menu = self.menu.addMenu("Область")
            scope_group = QActionGroup(scope_menu)
            scope_group.setExclusive(True)
            for value in (1, 2, 3):
                action = QAction(self.word_count_label(value), scope_menu)
                action.setCheckable(True)
                action.setChecked(int(cfg.get("replace_words", 1)) == value)
                action.triggered.connect(lambda _checked, chosen=value: self.update_config("replace_words", chosen))
                scope_group.addAction(action)
                scope_menu.addAction(action)

            trigger_menu = self.menu.addMenu("Триггер")
            trigger_group = QActionGroup(trigger_menu)
            trigger_group.setExclusive(True)
            for key, label in (
                ("double-lshift", "Двойной левый Shift"),
                ("double-rshift", "Двойной правый Shift"),
                ("caps-lock", "Caps Lock"),
            ):
                action = QAction(label, trigger_menu)
                action.setCheckable(True)
                action.setChecked(cfg.get("trigger") == key)
                action.triggered.connect(lambda _checked, chosen=key: self.update_config("trigger", chosen))
                trigger_group.addAction(action)
                trigger_menu.addAction(action)
            trigger_menu.addSeparator()
            self.add_bool_action("Несколько нажатий Shift", "multi_tap_scope", cfg, trigger_menu)

            force_menu = self.menu.addMenu("Прямой язык")
            self.add_bool_action("Хоткеи RU / EN", "force_layout_hotkeys", cfg, force_menu)
            self.add_force_key_menu(force_menu, "RU", "force_ru_key", cfg)
            self.add_force_key_menu(force_menu, "EN", "force_en_key", cfg)

            self.menu.addSeparator()
            self.add_bool_action("Помощь при наборе", "typing_assist", cfg)
            self.add_bool_action("Автоподмена", "auto_replace", cfg)
            self.add_bool_action("Автопереключение раскладки", "auto_switch_layout", cfg)
            self.add_bool_action("Запоминать правки", "learning_log", cfg)

            advanced = self.menu.addMenu("Арбитр")
            self.add_bool_action("LEM для 2 слов", "lem_2_words", cfg, advanced)
            self.add_bool_action("LEM для 3 слов", "lem_3_words", cfg, advanced)
            backend_menu = advanced.addMenu("Вставка текста")
            backend_group = QActionGroup(backend_menu)
            backend_group.setExclusive(True)
            for value, label in (("uinput", "Быстрый ввод"), ("ime", "IME / IBus")):
                action = QAction(label, backend_menu)
                action.setCheckable(True)
                action.setChecked(str(cfg.get("text_backend", "uinput")) == value)
                action.triggered.connect(lambda _checked, chosen=value: self.update_config("text_backend", chosen))
                backend_group.addAction(action)
                backend_menu.addAction(action)

            recent_menu = self.menu.addMenu("Последние действия")
            actions = load_recent_actions(5)
            if not actions:
                empty = QAction("пока нет действий", recent_menu)
                empty.setEnabled(False)
                recent_menu.addAction(empty)
            else:
                summary = QAction(recent_actions_summary(load_recent_actions(20)), recent_menu)
                summary.setEnabled(False)
                recent_menu.addAction(summary)
                recent_menu.addSeparator()
            for item in actions:
                action = QAction(self.recent_action_label(item), recent_menu)
                action.setEnabled(False)
                recent_menu.addAction(action)

            self.menu.addSeparator()
            nanda = QAction("NANDA ячейки", self.menu)
            nanda.triggered.connect(self.show_nanda)
            self.menu.addAction(nanda)

            about = QAction("О программе", self.menu)
            about.triggered.connect(self.show_about)
            self.menu.addAction(about)

            quit_action = QAction("Закрыть значок", self.menu)
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

        def add_layout_backend_menu(
            self,
            parent: QMenu,
            cfg: dict[str, Any],
            keep_open: bool = False,
        ) -> None:
            current = str(cfg.get("layout_backend", "auto"))
            menu = parent.addMenu(f"Среда раскладки: {self.layout_backend_label(current)}")
            group = QActionGroup(menu)
            group.setExclusive(True)
            for value, label in (
                ("auto", "Авто"),
                ("kde", "KDE/Plasma"),
                ("x11", "X11"),
                ("gnome", "GNOME"),
                ("niri", "Niri (эксп.)"),
            ):
                action = QAction(label, menu)
                action.setCheckable(True)
                action.setChecked(current == value)
                action.triggered.connect(
                    lambda _checked, chosen=value: self.update_config(
                        "layout_backend",
                        chosen,
                        keep_open=keep_open,
                    )
                )
                group.addAction(action)
                menu.addAction(action)

        def add_force_key_menu(
            self,
            parent: QMenu,
            label: str,
            key: str,
            cfg: dict[str, Any],
        ) -> None:
            menu = parent.addMenu(f"{label}: {self.force_key_label(cfg.get(key))}")
            group = QActionGroup(menu)
            group.setExclusive(True)
            for value, title in (
                ("single-rctrl", "Правый Ctrl"),
                ("single-ralt", "Правый Alt"),
                ("single-rshift", "Правый Shift"),
                ("single-pause", "Pause"),
                ("caps-lock", "Caps Lock"),
            ):
                action = QAction(title, menu)
                action.setCheckable(True)
                action.setChecked(cfg.get(key) == value)
                action.triggered.connect(
                    lambda _checked, config_key=key, chosen=value: self.update_config(config_key, chosen)
                )
                group.addAction(action)
                menu.addAction(action)

        def update_config(self, key: str, value: Any, keep_open: bool = False) -> None:
            cfg = load_config()
            cfg[key] = value
            if key == "correction_engine":
                cfg["mode"] = "simple"
            if cfg.get("force_ru_key") == cfg.get("force_en_key"):
                cfg["force_layout_hotkeys"] = False
            if cfg.get("text_backend") not in ("uinput", "ime", "auto"):
                cfg["text_backend"] = "uinput"
            if cfg.get("layout_backend") not in ("auto", "gnome", "kde", "x11", "niri"):
                cfg["layout_backend"] = "auto"
            if cfg.get("correction_safety") not in ("strict", "normal", "experimental"):
                cfg["correction_safety"] = "normal"
            cfg["multi_tap_max_taps"] = max(2, min(4, int(cfg.get("multi_tap_max_taps", 4))))
            save_config(cfg)
            self.run_service_action("restart", notify=False)
            self.rebuild_menu()
            if keep_open:
                QTimer.singleShot(80, self.reopen_menu)

        def set_daemon(self, checked: bool) -> None:
            self.run_service_action("start" if checked else "stop")
            self.rebuild_menu()

        def run_service_action(self, action: str, notify: bool = True) -> None:
            ok = service_action(action)
            if notify and not ok:
                QMessageBox.warning(
                    None,
                    "lay",
                    f"Не удалось выполнить: systemctl --user {action} lay-daemon.service",
                )
            self.refresh_status()

        def run_update(self) -> None:
            ok, message = start_update()
            if ok:
                self.tray.showMessage(
                    "lay",
                    f"Проверка обновлений запущена.\n{message}",
                    QSystemTrayIcon.MessageIcon.Information,
                    2500,
                )
            else:
                QMessageBox.warning(None, "lay", f"Не удалось запустить обновление:\n{message}")

        def show_about(self) -> None:
            QMessageBox.about(
                None,
                "О программе",
                "<b>lay</b><br>"
                "RU/EN-переключатель по двойному Shift и помощь при наборе.<br><br>"
                f"{lay_version()}<br>"
                "Платформы: GNOME, KDE, Wayland, X11.<br>"
                "KDE-меню использует тот же config и тот же lay-daemon.<br><br>"
                'GitHub: <a href="https://github.com/radislabus-star/lay-public">'
                "https://github.com/radislabus-star/lay-public</a>",
            )

        def show_nanda(self) -> None:
            dialog = QDialog()
            dialog.setWindowTitle("NANDA")
            dialog.resize(580, 640)
            layout = QVBoxLayout(dialog)

            title = QLabel("<b>NANDA — клеточный слой автокоррекции</b>")
            title.setWordWrap(True)
            layout.addWidget(title)

            intro = QLabel(
                "NANDA не является большой LLM. Это набор маленьких ячеек, которые рождают "
                "варианты, оценивают риск, защищают технические слова и согласуют решение перед вставкой."
            )
            intro.setWordWrap(True)
            layout.addWidget(intro)

            layout.addWidget(QLabel("<b>Волновая схема</b>"))
            layout.addWidget(NandaWaveWidget())

            details = QLabel(
                "<b>Профиль</b><br>"
                f"{html.escape(nanda_profile_text())}<br><br>"
                "<b>Ячейки и роли</b><br>"
                f"{html.escape(nanda_status_text()).replace(chr(10), '<br>')}<br><br>"
                "<b>Принцип</b><br>"
                "Генератор предлагает исправления, защитные ячейки гасят опасные варианты, "
                "сетка согласования пропускает только устойчивое решение. Если согласия нет, "
                "lay ничего не меняет."
            )
            details.setTextFormat(Qt.TextFormat.RichText)
            details.setWordWrap(True)
            scroll = QScrollArea()
            scroll.setWidgetResizable(True)
            holder = QWidget()
            holder_layout = QVBoxLayout(holder)
            holder_layout.addWidget(details)
            scroll.setWidget(holder)
            layout.addWidget(scroll)

            dialog.exec()

        def on_activated(self, reason: QSystemTrayIcon.ActivationReason) -> None:
            if reason in (
                QSystemTrayIcon.ActivationReason.Trigger,
                QSystemTrayIcon.ActivationReason.Context,
            ):
                self.show_menu_at_cursor()

        @staticmethod
        def word_count_label(value: int) -> str:
            if value == 1:
                return "1 слово"
            if value in (2, 3, 4):
                return f"{value} слова"
            return f"{value} слов"

        @staticmethod
        def engine_label(cfg: dict[str, Any]) -> str:
            engine = cfg.get("correction_engine") or cfg.get("mode")
            return "умный" if engine == "smart" else "обычный"

        @staticmethod
        def recent_action_label(item: dict[str, Any]) -> str:
            kind = action_kind_label(item.get("kind"))
            left = " ".join(str(item.get("from", "")).split())
            right = " ".join(str(item.get("to", "")).split())
            if len(left) > 24:
                left = left[:21] + "..."
            if len(right) > 24:
                right = right[:21] + "..."
            return f"{kind}: {left} → {right} · {int(item.get('elapsed_ms', 0))}мс"

        @staticmethod
        def force_key_label(key: Any) -> str:
            return {
                "single-rctrl": "Правый Ctrl",
                "single-ralt": "Правый Alt",
                "single-rshift": "Правый Shift",
                "single-pause": "Pause",
                "caps-lock": "Caps Lock",
            }.get(str(key), "Правый Ctrl")

        @staticmethod
        def layout_backend_label(value: Any) -> str:
            return {
                "auto": "Авто",
                "gnome": "GNOME",
                "kde": "KDE/Plasma",
                "x11": "X11",
                "niri": "Niri эксп.",
            }.get(str(value), "Авто")

    lock_file = acquire_single_instance_lock()
    if lock_file is None:
        print("lay-kde-tray: already running", file=sys.stderr)
        return 0
    return LayTray().run()


if __name__ == "__main__":
    raise SystemExit(main())
