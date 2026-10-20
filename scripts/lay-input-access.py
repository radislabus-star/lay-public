#!/usr/bin/python3
"""Explain device readiness; provision only through the existing installer helper.

Opening and closing devices here never reads events or creates an input route.
READY means the device preflight passed, not that a client accepted an edit.
"""
import argparse
import errno
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile


IGNORED_NAMES = {"lay-virtual-keyboard", "lay-ime-virtual-keyboard", "ydotoold virtual device"}


def native_session_ready():
    """Read-only capability, never create replay devices or start a listener."""
    if not os.environ.get("DISPLAY") or os.environ.get("XDG_SESSION_TYPE") == "wayland":
        return False
    daemon = shutil.which("lay-daemon")
    if not daemon:
        return False
    try:
        result = subprocess.run([daemon, "--input-status"], text=True,
                                capture_output=True, timeout=2)
        state = json.loads(result.stdout) if result.returncode == 0 else {}
        return state.get("ready") is True and state.get("transport") == "x11-xinput2"
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return False


def has_typing_keys(bitmap, word_bits=None):
    word_bits = word_bits or struct.calcsize("L") * 8
    bits = sum(int(word, 16) << (index * word_bits)
               for index, word in enumerate(reversed(bitmap.split())))
    return bool(bits & (1 << 30) and bits & (1 << 42))


def access_error(path, flags):
    try:
        fd = os.open(path, flags | os.O_NONBLOCK | os.O_CLOEXEC)
    except OSError as error:
        if error.errno in (errno.EACCES, errno.EPERM):
            return "permission"
        if error.errno in (errno.ENOENT, errno.ENODEV, errno.ENXIO):
            return "missing"
        return "unavailable"
    os.close(fd)
    return None


def probe(dev_root=Path("/dev"), sys_root=Path("/sys/class/input")):
    keyboards = readable = unknown = 0
    keyboard_errors = {"permission": 0, "missing": 0, "unavailable": 0}
    try:
        paths = list((dev_root / "input").iterdir())
    except FileNotFoundError:
        paths = []
    except OSError:
        paths = []
        unknown = 1
    for path in paths:
        if not path.name.startswith("event"):
            continue
        device = sys_root / path.name / "device"
        try:
            name = (device / "name").read_text().strip()
            if name in IGNORED_NAMES:
                continue
            if not has_typing_keys((device / "capabilities/key").read_text()):
                continue
        except (OSError, ValueError):
            unknown += 1
            continue
        keyboards += 1
        error = access_error(path, os.O_RDONLY)
        if error is None:
            readable += 1
        else:
            keyboard_errors[error] += 1
    uinput = access_error(dev_root / "uinput", os.O_WRONLY)
    if not readable:
        status = "KEYBOARD_ACCESS" if keyboard_errors["permission"] else (
            "DEVICE_DISCOVERY" if keyboards or unknown else "NO_KEYBOARD")
    else:
        status = "DEVICE_DISCOVERY" if uinput == "unavailable" else ("UINPUT_ACCESS" if uinput else "READY")
    if (not readable and dev_root == Path("/dev")
            and sys_root == Path("/sys/class/input") and native_session_ready()):
        status = "NATIVE_READY"
    return {"status": status, "keyboards": keyboards, "readable_keyboards": readable,
            "keyboard_errors": keyboard_errors, "unknown_devices": unknown, "uinput_error": uinput}


def explanation(result):
    return {
        "NATIVE_READY": ("Ввод удалённой X11-сессии доступен",
                         "Lay может получать нажатия удалённого рабочего стола без физической "
                         "клавиатуры и /dev/uinput. Права администратора не нужны. "
                         "Это проверка доступности ввода; подсказку, Tab и Double Shift "
                         "нужно проверить в приложении."),
        "READY": ("Устройства ввода доступны",
                  "Lay может открыть клавиатуру и виртуальное устройство ввода. "
                  "При восстановлении доступа демон подключится автоматически. "
                  "Эта проверка не подтверждает работу подсказки или Tab в приложении."),
        "KEYBOARD_ACCESS": ("Lay не может открыть клавиатуру",
                            "Клавиатура найдена, но недоступна этому пользователю. "
                            "Можно применить штатные правила доступа через активную системную сессию. "
                            "Настройка запросит права администратора. В удалённой или неактивной сессии "
                            "система может не предоставить доступ даже после настройки."),
        "NO_KEYBOARD": ("Клавиатура для Double Shift не найдена",
                        "В системе нет подходящего устройства клавиатуры для Lay. "
                        "В удалённом рабочем столе нажатия могут поступать только в приложения, "
                        "минуя устройства, которые читает Lay. Изменение прав это не исправит. "
                        "Проверьте, что Lay запущен внутри нужной X11-сессии; "
                        "поддерживаемая удалённая сессия не требует физической клавиатуры."),
        "DEVICE_DISCOVERY": ("Не удалось проверить устройства ввода",
                             "Не удалось прочитать параметры или открыть устройство. "
                             "Оно могло отключиться или стать недоступным по другой причине. "
                             "Отказ прав доступа не подтверждён. Проверьте подключение "
                             "и повторите проверку."),
        "UINPUT_ACCESS": ("Виртуальное устройство ввода недоступно",
                          "Lay видит клавиатуру, но не может открыть /dev/uinput. "
                          "Штатная настройка загрузит модуль и применит правила доступа, "
                          "запросив права администратора. Доступ зависит от активной системной сессии."),
    }[result["status"]]


def setup_helper():
    script = Path(__file__).resolve()
    for root in (script.parent / "input-access", script.parent.parent):
        helper = root / "scripts/setup-input-access.sh"
        if helper.is_file():
            return helper
    return None


def repair_command(result):
    helper = setup_helper()
    pkexec = shutil.which("pkexec")
    timeout = shutil.which("timeout")
    if result["status"] not in ("KEYBOARD_ACCESS", "UINPUT_ACCESS") or not helper or not pkexec or not timeout:
        return None
    # Run the deadline with the helper's privilege, so it can also stop a stalled
    # root process. Authentication itself remains the normal Polkit interaction.
    return [pkexec, timeout, "--signal=TERM", "--kill-after=5s", "30s", "/usr/bin/bash", str(helper)]


def repair_message(exit_code, result):
    if exit_code == 126:
        return "Настройка отменена. Права доступа не подтверждены."
    if exit_code in (124, 137):
        return "Истекло время настройки. Правила могли примениться частично; доступ проверен заново."
    if exit_code:
        return "Настройка не завершена. Проверьте сообщение об ошибке ниже."
    if result["status"] in ("READY", "NATIVE_READY"):
        return "Настройка завершена, устройства доступны. Перезапуск Lay не требуется."
    return ("Правила применены, но устройства пока недоступны. "
            "Проверьте активную локальную сессию и подключение клавиатуры.")


def window_lock():
    runtime = Path(os.environ.get("XDG_RUNTIME_DIR", tempfile.gettempdir()))
    directory = runtime / f"lay-input-access-{os.getuid()}"
    directory.mkdir(mode=0o700, exist_ok=True)
    if directory.is_symlink() or directory.stat().st_uid != os.getuid():
        raise OSError("небезопасный каталог блокировки окна")
    scope = "\n".join(os.environ.get(key, "") for key in
                      ("DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS"))
    name = hashlib.sha256(scope.encode()).hexdigest()[:16]
    fd = os.open(directory / name, os.O_CREAT | os.O_RDWR | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        os.close(fd)
        return None
    return fd


def show_window(initial):
    if not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_DISPLAY"):
        print("Lay: нет графической сессии; запустите lay-input-access --json для диагностики.", file=sys.stderr)
        return 1
    try:
        import gi
        gi.require_version("Gtk", "3.0")
        from gi.repository import Gio, Gtk
        if not Gtk.init_check()[0]:
            raise RuntimeError("не удалось подключиться к графической сессии")
        lock = window_lock()
        if lock is None:
            notice = Gtk.MessageDialog(message_type=Gtk.MessageType.INFO, buttons=Gtk.ButtonsType.CLOSE,
                                       text="Диагностика Lay уже открыта или настройка ещё выполняется.")
            notice.format_secondary_text("Завершите или отмените запрос прав администратора. Повторная настройка не запущена.")
            notice.run()
            notice.destroy()
            return 0
    except (ImportError, ValueError, OSError, RuntimeError) as error:
        print(f"Lay: окно диагностики недоступно: {error}; используйте lay-input-access --json", file=sys.stderr)
        return 1

    dialog = Gtk.Dialog(title="Lay — доступ к клавиатуре")
    dialog.set_default_size(570, -1)
    dialog.set_border_width(16)
    dialog.add_button("Закрыть", Gtk.ResponseType.CLOSE)
    retry = dialog.add_button("Проверить снова", 1)
    repair = dialog.add_button("Настроить доступ…", 2)
    label = Gtk.Label(xalign=0)
    label.set_line_wrap(True)
    label.set_max_width_chars(70)
    label.set_selectable(True)
    dialog.get_content_area().add(label)
    state = {"result": initial, "busy": False, "closing": False}

    def refresh(result, message=""):
        state["result"] = result
        title, body = explanation(result)
        if result["status"] not in ("READY", "NATIVE_READY"):
            body += "\n\nDouble Shift может быть недоступен. Подсказка и Tab проверяются отдельно через IME."
        if result["status"] in ("KEYBOARD_ACCESS", "UINPUT_ACCESS") and not repair_command(result):
            body += "\n\nСредство настройки, pkexec или timeout не установлены. Повторите установку Lay."
        label.set_text(title + "\n\n" + body + ("\n\n" + message if message else ""))
        repair.set_sensitive(not state["busy"] and repair_command(result) is not None)
        retry.set_sensitive(not state["busy"])

    def finished(process, operation):
        state["busy"] = False
        try:
            _, stdout, stderr = process.communicate_utf8_finish(operation)
            code = process.get_exit_status() if process.get_if_exited() else 1
            result = probe()
            message = repair_message(code, result)
            if code:
                message += "\n" + (stderr or stdout or "Ошибка выполнения настройки.")[-2000:].strip()
            refresh(result, message)
        except Exception as error:
            refresh(probe(), f"Ошибка настройки: {error}")
        if state["closing"]:
            dialog.destroy()

    def response(_dialog, code):
        if code == 1:
            if not state["busy"]:
                refresh(probe())
        elif code == 2:
            if state["busy"]:
                return
            # Recheck before requesting privilege. Nothing is provisioned at startup.
            current = probe()
            command = repair_command(current)
            refresh(current)
            if command is None:
                return
            try:
                child = Gio.Subprocess.new(command, Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_PIPE)
                state["busy"] = True
                refresh(current, "Ожидается подтверждение администратора. Отменить его можно в окне авторизации. "
                        "Применение правил ограничено 30 секундами. Можно закрыть это окно: "
                        "подтверждённая настройка продолжится в фоне.")
                child.communicate_utf8_async(None, None, finished)
            except Exception as error:
                state["busy"] = False
                refresh(probe(), f"Не удалось запустить настройку: {error}")
        else:
            if state["busy"]:
                state["closing"] = True
                dialog.hide()  # Keep supervising/reaping the child and holding the lock.
            else:
                dialog.destroy()

    def delete_event(_dialog, _event):
        response(dialog, Gtk.ResponseType.CLOSE)
        return True

    dialog.connect("response", response)
    dialog.connect("delete-event", delete_event)
    dialog.connect("destroy", lambda _window: Gtk.main_quit())
    refresh(initial)
    dialog.show_all()
    try:
        Gtk.main()
    finally:
        os.close(lock)
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="проверка без окна и без изменения системы")
    parser.add_argument("--show-if-needed", action="store_true", help="открыть окно только при недоступных устройствах")
    args = parser.parse_args()
    result = probe()
    if args.json:
        print(json.dumps(result, ensure_ascii=False))
        return int(result["status"] not in ("READY", "NATIVE_READY"))
    if args.show_if_needed and result["status"] in ("READY", "NATIVE_READY"):
        return 0
    return show_window(result)


if __name__ == "__main__":
    raise SystemExit(main())
