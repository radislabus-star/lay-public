#!/usr/bin/env python3
"""Run live lay-daemon smoke tests against a real GTK text field.

This is intentionally a runtime harness, not a unit test. It opens a Zenity
entry dialog, sends physical key events through `lay-test-input`, then compares
the text returned by the dialog after Enter.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "target/release/lay-test-input"
DEFAULT_DAEMON = ROOT / "target/release/lay-daemon"


@dataclasses.dataclass(frozen=True)
class Case:
    name: str
    expected: str
    start_layout: str = "us"
    config_overrides: dict[str, object] | None = None


CASES = {
    "ghbdtn_enter": Case("ghbdtn_enter", "привет"),
    "ghbdtn_enter_autocorrect": Case(
        "ghbdtn_enter_autocorrect",
        "ghbdtn",
        config_overrides={"enter_autocorrect": True},
    ),
    "ghbdtn_fast_lshift_enter": Case("ghbdtn_fast_lshift_enter", "привет"),
    "ghbdtn_extra_lshift_enter": Case("ghbdtn_extra_lshift_enter", "привет"),
    "ctrl_plus_ghbdtn_enter": Case("ctrl_plus_ghbdtn_enter", "привет"),
    "dhtvz_toggle_enter": Case("dhtvz_toggle_enter", "dhtvz"),
    "dhtvz_toggle3_enter": Case("dhtvz_toggle3_enter", "время"),
    "g_to_ru_enter": Case("g_to_ru_enter", "п"),
    "eng_ru_to_us_enter": Case("eng_ru_to_us_enter", "eng", start_layout="ru"),
    "plain_layout_ashdu_space_enter": Case(
        "plain_layout_ashdu_space_enter",
        "file",
        start_layout="ru",
        config_overrides={
            "auto_replace": True,
            "typing_assist": True,
            "correction_safety": "normal",
        },
    ),
    "plain_layout_cargo_space_enter": Case(
        "plain_layout_cargo_space_enter",
        "cargo",
        start_layout="ru",
        config_overrides={
            "auto_replace": True,
            "typing_assist": True,
            "correction_safety": "normal",
        },
    ),
    "plain_layout_abkt_space_enter": Case(
        "plain_layout_abkt_space_enter",
        "abkt",
        start_layout="us",
        config_overrides={
            "auto_replace": True,
            "typing_assist": True,
            "correction_safety": "normal",
        },
    ),
    "good_toggle4_enter": Case("good_toggle4_enter", "good"),
    "good_ntrcn_enter": Case("good_ntrcn_enter", "good текст"),
    "good_text_enter": Case("good_text_enter", "good текст", start_layout="ru"),
    "good_vshgidu_enter": Case("good_vshgidu_enter", "good Double"),
    "mixed_word": Case("mixed_word", "при"),
    "mixed_coke_enter": Case("mixed_coke_enter", "слово кока-колу", start_layout="ru"),
    "mixed_coke_toggle3_enter": Case(
        "mixed_coke_toggle3_enter", "слово кока-колу", start_layout="ru"
    ),
    "n_teper_mixed_enter": Case("n_teper_mixed_enter", "Теперь"),
    "auto_switch_words_enter": Case("auto_switch_words_enter", "njkmrj yt hf,jnftn"),
    "no_ne_ty_enter": Case("no_ne_ty_enter", "но не ты", start_layout="ru"),
    "preparatov_typo_enter": Case(
        "preparatov_typo_enter", "препаратов", start_layout="ru"
    ),
    "proverka_ntrcn_enter": Case(
        "proverka_ntrcn_enter", "проверка текст", start_layout="ru"
    ),
    "glued_toesamoe_next_enter": Case(
        "glued_toesamoe_next_enter", "тоже самое склено", start_layout="ru"
    ),
    "glued_tozhesamoe_next_enter": Case(
        "glued_tozhesamoe_next_enter", "тоже самое склено", start_layout="ru"
    ),
    "glued_yanebudu_next_enter": Case(
        "glued_yanebudu_next_enter", "я не буду склено", start_layout="ru"
    ),
    "glued_context_yanebudu_next_enter": Case(
        "glued_context_yanebudu_next_enter",
        "тоже самое я не буду склено",
        start_layout="ru",
    ),
    "glued_long_phrase_next_enter": Case(
        "glued_long_phrase_next_enter",
        "я не буду за вас тоже самое склено",
        start_layout="ru",
    ),
    "ru_p_enter": Case("ru_p_enter", "п", start_layout="ru"),
    "ru_p_to_g_enter": Case("ru_p_to_g_enter", "g", start_layout="ru"),
    "ru_p_toggle2_enter": Case("ru_p_toggle2_enter", "п", start_layout="ru"),
    "slovo_ru_to_us_fast_lshift_enter": Case(
        "slovo_ru_to_us_fast_lshift_enter", "ckjdj", start_layout="ru"
    ),
    "slovo_ru_to_us_extra_lshift_enter": Case(
        "slovo_ru_to_us_extra_lshift_enter", "ckjdj", start_layout="ru"
    ),
    "vyvodim_dva_enter": Case("vyvodim_dva_enter", "выводим два"),
    "wifi_ye_enter": Case("wifi_ye_enter", "wi-fi ну"),
}


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
    parser.add_argument("--use-system-daemon", action="store_true")
    parser.add_argument("--daemon-debug", action="store_true")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument(
        "--ime-engine",
        action="store_true",
        help="use lay-ime-ru/lay-ime-us IBus engines as the start layout",
    )
    args = parser.parse_args()

    dialog = choose_dialog_command(args.dialog)
    require_command("gdbus")
    input_bin = ensure_binary(args.input_bin, "lay-test-input", args.no_build)
    daemon_bin = None if args.use_system_daemon else ensure_binary(args.daemon_bin, "lay-daemon", args.no_build)

    selected = [CASES[name] for name in (args.case or sorted(CASES))]
    failures = 0
    for case in selected:
        ok, got, detail = run_case(
            case,
            input_bin,
            daemon_bin,
            dialog,
            args.focus_delay,
            args.timeout,
            args.daemon_debug,
            args.ime_engine,
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
        runtime_env["HOME"] = temp_home.name
        config_dir = Path(temp_home.name) / ".config" / "lay"
        config_dir.mkdir(parents=True, exist_ok=True)
        config = {
            "mode": "simple",
            "correction_engine": "replay",
            "replace_words": 1,
            "auto_replace": False,
            "typing_assist": False,
            "auto_switch_layout": True,
            **case.config_overrides,
        }
        (config_dir / "config.json").write_text(
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

    sender = subprocess.Popen(
        [str(input_bin), case.name],
        cwd=ROOT,
        env={
            **dict_env(),
            "LAY_TEST_START_DELAY_MS": "3500",
            "LAY_TEST_INITIAL_LAYOUT": case.start_layout,
        },
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
    if ime_engine:
        engine = "lay-ime-ru" if layout == "ru" else "lay-ime-us"
    else:
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
