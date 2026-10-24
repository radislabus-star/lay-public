#!/usr/bin/env python3
"""Configure application IME clients and diagnose the current desktop session."""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tempfile

MARKER = "# Managed by Lay: GLFW IBus client\n"
DESKTOP_MARKER = "X-Lay-IBus-Launcher=true\n"
EXEC_MARKER = "# Managed by Lay: GLFW IBus Exec\n"
EXEC_PREFIX = "/usr/bin/env GLFW_IM_MODULE=ibus "
COMMAND_MARKER = "# Managed by Lay: Kitty IBus command"
MODULES = {"GTK_IM_MODULE": "ibus", "QT_IM_MODULE": "ibus",
           "XMODIFIERS": "@im=ibus", "GLFW_IM_MODULE": "ibus"}


def session_files(config):
    return {
        config / "environment.d/90-lay-glfw-ime.conf": MARKER + "".join(
            f"{key}=${{{key}:-{value}}}\n" for key, value in MODULES.items()),
        config / "plasma-workspace/env/lay-glfw-ime.sh": MARKER + "".join(
            f'if [ -z "${{{key}:-}}" ]; then\n'
            f'    export {key}={value}\nfi\n' for key, value in MODULES.items()),
    }


def previous_session_files(config):
    return {
        config / "environment.d/90-lay-glfw-ime.conf":
            MARKER + "GLFW_IM_MODULE=${GLFW_IM_MODULE:-ibus}\n",
        config / "plasma-workspace/env/lay-glfw-ime.sh":
            MARKER + 'if [ -z "${GLFW_IM_MODULE:-}" ]; then\n'
            '    export GLFW_IM_MODULE=ibus\nfi\n',
    }


def write(path, text, mode=0o644):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as f:
        try:
            f.write(text)
            f.close()
            os.chmod(f.name, mode)
            os.replace(f.name, path)
        finally:
            Path(f.name).unlink(missing_ok=True)


def kitty_command_text(record):
    return ("#!/bin/sh\n" + COMMAND_MARKER + "\n# " + json.dumps(record) + "\n"
            'if [ -z "${GLFW_IM_MODULE:-}" ]; then\n'
            '    export GLFW_IM_MODULE=ibus\nfi\n'
            "exec " + shlex.quote(record["target"]) + ' "$@"\n')


def owned_kitty_command(path):
    if path.is_symlink() or not path.is_file():
        return None
    try:
        text = path.read_text()
        lines = text.splitlines()
        if len(lines) < 3 or lines[1] != COMMAND_MARKER or not lines[2].startswith("# "):
            return None
        record = json.loads(lines[2][2:])
        if (not isinstance(record, dict) or set(record) != {"target", "previous_link"}
                or not isinstance(record["target"], str)
                or not Path(record["target"]).is_absolute()
                or record["previous_link"] is not None and not isinstance(record["previous_link"], str)):
            return None
        return record if text == kitty_command_text(record) else None
    except (ValueError, UnicodeError):
        return None


def kitty_command_plan(home):
    path = home / ".local/bin/kitty"
    if path.is_symlink():
        record = dict(target=str(path.resolve(strict=True)), previous_link=os.readlink(path))
    elif path.exists():
        record = owned_kitty_command(path)
        if record is None:
            raise RuntimeError(f"preserving existing custom Kitty command: {path}")
    else:
        found = shutil.which("kitty")
        if not found:
            return None
        record = dict(target=str(Path(found).resolve(strict=True)), previous_link=None)
    target = Path(record["target"])
    if (target.resolve(strict=True) == path.parent.resolve() / path.name
            or not target.is_file() or not os.access(target, os.X_OK)):
        raise RuntimeError("Kitty command target is unavailable or recursive")
    return path, kitty_command_text(record)


def remove_kitty_command(home):
    path = home / ".local/bin/kitty"
    record = owned_kitty_command(path)
    if record is None:
        return
    if record["previous_link"] is None:
        path.unlink()
    else:
        with tempfile.TemporaryDirectory(dir=path.parent) as directory:
            original = Path(directory) / "kitty"
            original.symlink_to(record["previous_link"])
            os.replace(original, path)


def configure(remove=False):
    home = Path.home()
    config = Path(os.environ.get("XDG_CONFIG_HOME", home / ".config"))
    data = Path(os.environ.get("XDG_DATA_HOME", home / ".local/share"))
    roots = [Path(p) for p in os.environ.get(
        "XDG_DATA_DIRS", "/usr/local/share:/usr/share"
    ).split(":") if p]
    source = next((p / "applications/kitty.desktop" for p in roots
                   if (p / "applications/kitty.desktop").is_file()), None)
    desktop = data / "applications/kitty.desktop"
    envs = session_files(config)
    previous = previous_session_files(config)
    if remove:
        remove_kitty_command(home)
        for path, content in envs.items():
            if path.is_file() and path.read_text() in (content, previous[path]):
                path.unlink()
        if desktop.is_file() and DESKTOP_MARKER in desktop.read_text():
            original = desktop.read_text().splitlines(keepends=True)
            lines = []
            for index, line in enumerate(original):
                if (line == EXEC_MARKER and index + 1 < len(original)
                        and original[index + 1].startswith("Exec=" + EXEC_PREFIX)):
                    continue
                if (index and original[index - 1] == EXEC_MARKER
                        and line.startswith("Exec=" + EXEC_PREFIX)):
                    line = "Exec=" + line[len("Exec=" + EXEC_PREFIX):]
                lines.append(line)
            # A legacy global marker cannot prove ownership of each Exec prefix.
            text = "".join(lines).replace(DESKTOP_MARKER, "")
            if source and text == source.read_text():
                desktop.unlink()
            else:
                write(desktop, text)
        return "removed owned GLFW setup; preserved other settings"
    conflicts = [key for key, value in MODULES.items()
                 if os.environ.get(key) not in (None, "", value)]
    if conflicts:
        raise RuntimeError("another input method is selected: " + ", ".join(conflicts))
    text = desktop.read_text() if desktop.is_file() else source.read_text() if source else ""
    for line in text.splitlines():
        if line.startswith("Exec=") and "GLFW_IM_MODULE=" in line:
            if not re.search(r"(?:^|\s)GLFW_IM_MODULE=ibus(?:\s|$)", line[5:]):
                raise RuntimeError("Kitty launcher explicitly selects another input method")
    for path, content in envs.items():
        if path.exists() and path.read_text() not in (content, previous[path]):
            raise RuntimeError(f"preserving existing custom file: {path}")
    command = kitty_command_plan(home)
    lines = []
    for line in text.splitlines(keepends=True):
        if line.startswith("Exec=") and "GLFW_IM_MODULE=" not in line:
            lines.append(EXEC_MARKER)
            line = "Exec=" + EXEC_PREFIX + line[5:]
        lines.append(line)
    text = "".join(lines)
    if DESKTOP_MARKER not in text:
        text = text.replace("[Desktop Entry]\n", "[Desktop Entry]\n" + DESKTOP_MARKER, 1)
    if text:
        write(desktop, text)
    for path, content in envs.items():
        write(path, content)
    if command:
        write(*command, mode=0o755)
        selected = shutil.which("kitty")
        if (not os.environ.get("GLFW_IM_MODULE")
                and (not selected or Path(selected).resolve() != command[0].resolve())):
            raise RuntimeError(
                f"Kitty menu setup saved, but PATH bypasses {command[0]}. "
                f"Put {command[0].parent} before other Kitty directories in PATH, "
                "then run setup again.")
    return "IME clients configured. New Kitty commands and menu launches use IBus; existing windows stay open. Other session defaults apply at next login."


def status():
    """Read configuration and bus state, never claim client acceptance."""
    env = {key: os.environ.get(key) for key in MODULES}
    conflicts = [key for key, value in MODULES.items()
                 if env[key] not in (None, "", value)]
    try:
        result = subprocess.run(["ibus", "engine"], text=True,
                                capture_output=True, timeout=2)
        engine = result.stdout.strip() if result.returncode == 0 else None
    except (OSError, subprocess.TimeoutExpired):
        engine = None
    data = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share"))
    desktop = data / "applications/kitty.desktop"
    configured = desktop.is_file() and DESKTOP_MARKER in desktop.read_text()
    problems = []
    if conflicts:
        problems.append("Выбран другой метод ввода. Lay пока показывает подсказки через IBus.")
    if not engine:
        problems.append("IBus недоступен в этом сеансе.")
    elif engine not in ("lay-ime-ru", "lay-ime-us"):
        problems.append("В IBus сейчас не выбран Lay. В меню Lay выбери режим «IME-подсказки».")
    if any(not env[key] for key in ("GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS")):
        problems.append("В окружении сеанса заданы не все параметры IBus. На X11 это может мешать подсказкам. Новые параметры сеанса применяются после выхода и повторного входа.")
    kitty = []
    for process in Path("/proc").glob("[0-9]*"):
        try:
            if (process / "exe").resolve(strict=True).name != "kitty":
                continue
            values = dict(item.split("=", 1) for item in
                          (process / "environ").read_text().split("\0") if "=" in item)
            if values.get("HOME") != str(Path.home()) or values.get("DISPLAY") != os.environ.get("DISPLAY"):
                continue
            if values.get("GLFW_IM_MODULE") != "ibus":
                kitty.append(int(process.name))
        except (OSError, ValueError):
            continue
    if kitty:
        problems.append("Уже открытый Kitty не подключён к IBus. После настройки открой новое окно Kitty; рабочие окна Lay не закрывает.")
    command = Path.home() / ".local/bin/kitty"
    selected = shutil.which("kitty")
    command_configured = False
    try:
        if owned_kitty_command(command) and kitty_command_plan(Path.home()):
            command_configured = bool(selected and os.access(command, os.X_OK)
                                      and Path(selected).resolve() == command.resolve())
    except (OSError, RuntimeError):
        pass
    command_needed = bool(selected) and not env["GLFW_IM_MODULE"]
    if command_needed and not command_configured:
        problems.append("Обычная команда kitty ещё не настроена для IBus. Нажми «Настроить приложения для подсказок».")
    return dict(engine=engine, modules=env, conflicts=conflicts,
                kitty_launcher_configured=configured, kitty_without_ibus=kitty,
                kitty_command_configured=command_configured,
                kitty_command_needed=command_needed,
                problems=problems, client_acceptance="NOT_TESTED")


def show_dialog():
    import gi
    gi.require_version("Gtk", "3.0")
    from gi.repository import Gtk
    window = Gtk.Window(title="Подсказки Lay")
    window.set_default_size(580, 260)
    window.set_border_width(18)
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=14)
    window.add(box)
    label = Gtk.Label()
    label.set_line_wrap(True)
    label.set_xalign(0)
    box.pack_start(label, True, True, 0)

    def refresh(note=""):
        current = status()
        text = "\n\n".join(current["problems"]) or "IBus доступен, выбран Lay. Проверить подсказку в приложении можно набором «пров» и нажатием Tab."
        label.set_text((note + "\n\n" if note else "") + text)

    def setup(_button):
        try:
            configure()
            for command in (["update-desktop-database", str(Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share")) / "applications")],
                            ["kbuildsycoca5", "--noincremental"]):
                if shutil.which(command[0]):
                    subprocess.run(command, capture_output=True, timeout=15, check=False)
            refresh("Настройки сохранены. Открой новое окно Kitty обычным способом. Для остальных приложений может потребоваться повторный вход в сеанс.")
        except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
            refresh("Настройка не завершена: " + str(error))

    button = Gtk.Button(label="Настроить приложения для подсказок")
    button.connect("clicked", setup)
    box.pack_start(button, False, False, 0)
    if shutil.which("im-config"):
        choose = Gtk.Button(label="Выбрать метод ввода…")
        choose.connect("clicked", lambda _button: subprocess.Popen(["im-config"], start_new_session=True))
        box.pack_start(choose, False, False, 0)
    close = Gtk.Button(label="Закрыть")
    close.connect("clicked", lambda _button: window.destroy())
    box.pack_start(close, False, False, 0)
    window.connect("destroy", Gtk.main_quit)
    refresh()
    window.show_all()
    Gtk.main()


def needs_setup(report):
    if report["conflicts"] or report["engine"] not in ("lay-ime-ru", "lay-ime-us"):
        return True
    # GNOME and native Wayland clients can connect without these variables.
    # Their absence alone is not evidence of a broken application connection.
    kde = "KDE" in os.environ.get("XDG_CURRENT_DESKTOP", "").upper().split(":")
    x11 = os.environ.get("XDG_SESSION_TYPE") == "x11"
    return kde and x11 and (
        any(not report["modules"][key] for key in
            ("GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS"))
        or report.get("kitty_command_needed", False)
        and not report.get("kitty_command_configured", False))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--remove", action="store_true")
    parser.add_argument("--json", action="store_true", help="read-only session diagnostics")
    parser.add_argument("--gui", action="store_true", help="open setup window")
    parser.add_argument("--show-if-needed", action="store_true", help="explain incomplete session setup")
    args = parser.parse_args()
    try:
        if args.json:
            print(json.dumps(status(), ensure_ascii=False))
        elif args.gui or args.show_if_needed:
            report = status()
            if args.gui or needs_setup(report):
                show_dialog()
        else:
            print(configure(args.remove))
    except (OSError, RuntimeError) as error:
        parser.exit(1, f"Lay IME client setup: {error}\n")


if __name__ == "__main__":
    main()
