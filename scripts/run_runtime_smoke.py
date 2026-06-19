#!/usr/bin/env python3
"""Run live lay-daemon smoke tests against a real GTK text field.

This is intentionally a runtime harness, not a unit test. It opens a Zenity
entry dialog, sends physical key events through `lay-test-input`, then compares
the text returned by the dialog after Enter.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from contextlib import nullcontext
from pathlib import Path

from runtime_smoke.cases import CASES, Case
from runtime_smoke.ime import managed_ime_session


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "target/release/lay-test-input"
DEFAULT_DAEMON = ROOT / "target/release/lay-daemon"
DEFAULT_IBUS_ENGINE = ROOT / "target/release/lay-ibus-engine"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--case",
        action="append",
        choices=sorted(CASES),
        help="Case to run. Repeatable. Defaults to all smoke cases.",
    )
    parser.add_argument("--focus-delay", type=float, default=1.0)
    parser.add_argument("--timeout", type=float, default=20.0)
    parser.add_argument(
        "--dialog",
        choices=("auto", "gtk-entry-capture", "zenity", "kdialog"),
        default="auto",
        help="Dialog backend used as the focused text field.",
    )
    parser.add_argument("--input-bin", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--daemon-bin", type=Path, default=DEFAULT_DAEMON)
    parser.add_argument("--ibus-engine-bin", type=Path, default=DEFAULT_IBUS_ENGINE)
    parser.add_argument("--use-system-daemon", action="store_true")
    parser.add_argument("--daemon-debug", action="store_true")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument(
        "--ime-engine",
        action="store_true",
        help="use lay-ime-ru/lay-ime-us IBus engines as the start layout",
    )
    parser.add_argument(
        "--ime-managed",
        action="store_true",
        help="run Rust lay IBus engine in managed mode for this test and restore daemon after",
    )
    args = parser.parse_args()

    dialog = choose_dialog_command(args.dialog)
    require_command("gdbus")
    if args.ime_managed:
        require_command("ibus")
    input_bin = ensure_binary(args.input_bin, "lay-test-input", args.no_build)
    daemon_bin = None if args.use_system_daemon else ensure_binary(args.daemon_bin, "lay-daemon", args.no_build)
    ibus_engine_bin = None
    if args.ime_managed:
        ibus_engine_bin = ensure_binary(args.ibus_engine_bin, "lay-ibus-engine", args.no_build)
        daemon_bin = None

    selected = [CASES[name] for name in (args.case or sorted(CASES))]
    failures = 0
    ime_context = (
        managed_ime_session(ROOT, ibus_engine_bin) if args.ime_managed else nullcontext()
    )
    with ime_context:
        for case in selected:
            ok, got, detail = run_case(
                case,
                input_bin,
                daemon_bin,
                dialog,
                args.focus_delay,
                args.timeout,
                args.daemon_debug,
                args.ime_engine or args.ime_managed,
            )
            status = "OK" if ok else "BAD"
            print(f"{status} {case.name}: got={got!r} expected={case.expected!r}")
            if detail:
                print(indent(detail.rstrip()))
            failures += 0 if ok else 1

    return 1 if failures else 0


def require_command(name: str) -> None:
    if shutil.which(name) is None:
        raise SystemExit(f"required command not found: {name}")


def choose_dialog_command(preferred: str) -> str:
    if preferred != "auto":
        if preferred == "gtk-entry-capture" and (ROOT / "scripts" / "gtk_entry_capture.py").exists():
            return preferred
        if preferred in {"zenity", "kdialog"} and shutil.which(preferred) is not None:
            return preferred
        raise SystemExit(f"required dialog backend not found: {preferred}")

    custom = ROOT / "scripts" / "gtk_entry_capture.py"
    if custom.exists():
        return "gtk-entry-capture"
    for name in ("zenity", "kdialog"):
        if shutil.which(name) is not None:
            return name
    raise SystemExit("required command not found: zenity or kdialog")


def ensure_binary(path: Path, bin_name: str, no_build: bool) -> Path:
    if path.exists():
        return path
    if no_build:
        raise SystemExit(f"{bin_name} binary not found: {path}")
    subprocess.run(
        ["cargo", "build", "--release", "--bin", bin_name],
        cwd=ROOT,
        check=True,
    )
    if not path.exists():
        raise SystemExit(f"{bin_name} binary was not built: {path}")
    return path


def run_case(
    case: Case,
    input_bin: Path,
    daemon_bin: Path | None,
    dialog: str,
    focus_delay: float,
    timeout: float,
    daemon_debug: bool,
    ime_engine: bool,
) -> tuple[bool, str, str]:
    activate_layout(case.start_layout, ime_engine)
    runtime_env = dict_env()
    temp_home: tempfile.TemporaryDirectory[str] | None = None
    if case.config_overrides:
        if daemon_bin is None:
            return (
                False,
                "",
                "case needs isolated config; run without --use-system-daemon",
            )
        temp_home = tempfile.TemporaryDirectory(prefix="lay-smoke-home-")
        config_dir = Path(temp_home.name) / ".config" / "lay"
        config_dir.mkdir(parents=True, exist_ok=True)
        config_path = config_dir / "config.json"
        runtime_env["LAY_CONFIG_PATH"] = str(config_path)
        config = {
            "mode": "simple",
            "correction_engine": "replay",
            "replace_words": 1,
            "auto_replace": False,
            "typing_assist": False,
            "auto_switch_layout": True,
            **case.config_overrides,
        }
        config_path.write_text(
            json.dumps(config, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        if config.get("enter_autocorrect"):
            runtime_env["LAY_EXPERIMENTAL_ENTER_AUTOCORRECT"] = "1"
    dialog_proc = subprocess.Popen(
        dialog_args(dialog, case),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    time.sleep(focus_delay)

    sender_env = {
        **dict_env(),
        "LAY_TEST_START_DELAY_MS": "3500",
        "LAY_TEST_INITIAL_LAYOUT": case.start_layout,
    }
    if ime_engine:
        sender_env["LAY_TEST_IME_ENGINE"] = "1"
    sender = subprocess.Popen(
        [str(input_bin), case.name],
        cwd=ROOT,
        env=sender_env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    device_path = ""
    daemon = None
    daemon_stderr = ""
    if daemon_bin is not None:
        assert sender.stdout is not None
        device_path = sender.stdout.readline().strip()
        if not device_path.startswith("/dev/input/event"):
            sender.kill()
            stdout, stderr = dialog_proc.communicate(timeout=1)
            return False, stdout.strip(), f"invalid test device path: {device_path!r}\nsender stderr:\n{stderr}"
        if not wait_for_device_access(Path(device_path), timeout=3.0):
            sender.kill()
            stdout, stderr = dialog_proc.communicate(timeout=1)
            return False, stdout.strip(), f"test device is not readable: {device_path}"
        daemon_args = [str(daemon_bin), "--device", device_path]
        if daemon_debug:
            daemon_args.extend(["--debug-log", "--verbose"])
        daemon = subprocess.Popen(
            daemon_args,
            cwd=ROOT,
            text=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            env=runtime_env,
        )
        time.sleep(0.8)

    try:
        sender_stdout, sender_stderr = sender.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        sender.kill()
        sender_stdout, sender_stderr = sender.communicate()
        sender_stderr += "\nsender timeout"

    try:
        stdout, stderr = dialog_proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        dialog_proc.kill()
        stdout, stderr = dialog_proc.communicate()
        stderr += f"\n{dialog} timeout"

    if daemon is not None:
        daemon.terminate()
        try:
            _, daemon_stderr = daemon.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            daemon.kill()
            _, daemon_stderr = daemon.communicate()

    got = stdout.strip()
    details = []
    if sender.returncode != 0:
        details.append(f"sender exited {sender.returncode}")
    if dialog_proc.returncode != 0:
        details.append(f"{dialog} exited {dialog_proc.returncode}")
    if device_path:
        details.append(f"device: {device_path}")
    if sender_stdout:
        details.append(f"sender stdout:\n{sender_stdout}")
    if sender_stderr:
        details.append(f"sender stderr:\n{sender_stderr}")
    if daemon_stderr:
        details.append(f"daemon stderr:\n{daemon_stderr}")
    if daemon is not None and daemon.returncode not in {None, 0, -15}:
        details.append(f"daemon exited {daemon.returncode}")
    if stderr:
        details.append(f"{dialog} stderr:\n{stderr}")
    if temp_home is not None:
        temp_home.cleanup()

    return got == case.expected and sender.returncode == 0 and dialog_proc.returncode == 0, got, "\n".join(details)


def dialog_args(dialog: str, case: Case) -> list[str]:
    if dialog == "gtk-entry-capture":
        return [
            sys.executable,
            str(ROOT / "scripts" / "gtk_entry_capture.py"),
            "--title",
            f"Lay runtime smoke: {case.name}",
            "--text",
            f"Runtime smoke: {case.name}",
        ]
    if dialog == "zenity":
        return [
            "zenity",
            "--entry",
            "--title",
            f"Lay runtime smoke: {case.name}",
            "--text",
            f"Runtime smoke: {case.name}",
            "--width",
            "520",
        ]
    if dialog == "kdialog":
        return [
            "kdialog",
            "--title",
            f"Lay runtime smoke: {case.name}",
            "--inputbox",
            f"Runtime smoke: {case.name}",
            "",
        ]
    raise ValueError(f"unsupported dialog: {dialog}")


def dict_env() -> dict[str, str]:
    return dict(os.environ)


def wait_for_device_access(path: Path, timeout: float) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if os.access(path, os.R_OK):
            return True
        time.sleep(0.05)
    return os.access(path, os.R_OK)


def activate_layout(layout: str, ime_engine: bool = False) -> None:
    if ime_engine:
        engine = "lay-ime-ru" if layout == "ru" else "lay-ime-us"
        subprocess.run(
            ["ibus", "engine", engine],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return

    if activate_layout_kde(layout):
        return

    subprocess.run(
        [
            "gdbus",
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/io/github/radislabus_star/LayDaemon",
            "--method",
            "io.github.radislabus_star.LayDaemon.ActivateLayout",
            f'"{layout}"',
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    engine = "xkb:ru::rus" if layout == "ru" else "xkb:us::eng"
    subprocess.run(
        ["ibus", "engine", engine],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def activate_layout_kde(layout: str) -> bool:
    qdbus = shutil.which("qdbus6") or shutil.which("qdbus-qt6") or shutil.which("qdbus")
    if qdbus is None:
        return False

    index = kde_layout_index(qdbus, layout)
    if index is None:
        return False

    return (
        subprocess.run(
            [qdbus, "org.kde.keyboard", "/Layouts", "setLayout", str(index)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode
        == 0
    )


def kde_layout_index(qdbus: str, layout: str) -> int | None:
    out = subprocess.run(
        [qdbus, "--literal", "org.kde.keyboard", "/Layouts", "getLayoutsList"],
        text=True,
        capture_output=True,
        check=False,
    )
    if out.returncode != 0:
        return None

    layouts: list[str] = []
    for chunk in out.stdout.split("[Argument: (sss)")[1:]:
        first = chunk.find('"')
        if first < 0:
            continue
        second = chunk.find('"', first + 1)
        if second < 0:
            continue
        layouts.append(chunk[first + 1 : second])

    try:
        return layouts.index(layout)
    except ValueError:
        return None


def indent(text: str) -> str:
    return "\n".join(f"  {line}" for line in text.splitlines())


if __name__ == "__main__":
    sys.exit(main())
