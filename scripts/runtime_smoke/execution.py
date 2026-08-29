from __future__ import annotations

import json
import os
import selectors
import shutil
import subprocess
import sys
import time
from pathlib import Path

from runtime_smoke.cases import Case
from runtime_smoke.desktop import (
    current_desktop_layout,
    current_ibus_engine,
    gnome_layout_call,
    set_ibus_engine,
)
from runtime_smoke.isolation import CaseContext, ProcessSupervisor, SmokeInterrupted


ROOT = Path(__file__).resolve().parents[2]


def run_case(
    case: Case,
    context: CaseContext,
    input_bin: Path,
    daemon_bin: Path | None,
    dialog: str,
    focus_delay: float,
    timeout: float,
    daemon_debug: bool,
    ime_engine: bool,
) -> tuple[bool, str, str]:
    if case.config_overrides and daemon_bin is None:
        return False, "", "case needs isolated config; do not use --use-system-daemon"
    runtime_env = context.environment()
    config = json.loads(context.config_path.read_text(encoding="utf-8"))
    if config.get("enter_autocorrect"):
        runtime_env["LAY_EXPERIMENTAL_ENTER_AUTOCORRECT"] = "1"
    dialog_env = context.environment()
    if ime_engine:
        dialog_env["GTK_IM_MODULE"] = "ibus"
        dialog_env["IBUS_ENABLE_SYNC_MODE"] = "1"
    sender_env = context.environment()
    sender_env.update(
        {
            "LAY_TEST_INPUT_ARMED": "1",
            "LAY_TEST_START_DELAY_MS": "3500",
            "LAY_TEST_INITIAL_LAYOUT": case.start_layout,
        }
    )
    if ime_engine:
        sender_env["LAY_TEST_IME_ENGINE"] = "1"

    device_path = ""
    daemon: subprocess.Popen[str] | None = None
    dialog_proc: subprocess.Popen[str] | None = None
    sender: subprocess.Popen[str] | None = None
    sender_stdout = ""
    sender_stderr = ""
    stdout = ""
    stderr = ""
    daemon_stderr = ""
    execution_error = ""
    with ProcessSupervisor() as processes:
        try:
            activate_layout(case.start_layout, ime_engine)
            dialog_proc = processes.spawn(
                "dialog",
                dialog_args(dialog, case),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env=dialog_env,
            )
            time.sleep(focus_delay)
            sender = processes.spawn(
                "sender",
                [str(input_bin), case.name],
                cwd=ROOT,
                env=sender_env,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            if daemon_bin is not None:
                assert sender.stdout is not None
                device_path = readline_with_timeout(
                    sender.stdout, min(timeout, 5.0)
                ).strip()
                if not device_path.startswith("/dev/input/event"):
                    raise RuntimeError(f"invalid test device path: {device_path!r}")
                if not wait_for_device_access(Path(device_path), timeout=3.0):
                    raise RuntimeError(f"test device is not readable: {device_path}")
                daemon_args = [str(daemon_bin), "--device", device_path]
                if daemon_debug:
                    daemon_args.extend(["--debug-log", "--verbose"])
                daemon = processes.spawn(
                    "daemon",
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
                sender_stdout, sender_stderr = processes.stop(
                    sender, timeout=min(timeout, 3.0)
                )
                sender_stderr += "\nsender timeout"

            try:
                stdout, stderr = dialog_proc.communicate(timeout=timeout)
            except subprocess.TimeoutExpired:
                stdout, stderr = processes.stop(
                    dialog_proc, timeout=min(timeout, 3.0)
                )
                stderr += f"\n{dialog} timeout"

            if daemon is not None:
                _, daemon_stderr = processes.stop(daemon)
        except SmokeInterrupted:
            raise
        except Exception as error:
            execution_error = f"{type(error).__name__}: {error}"

    got = stdout.strip()
    details = []
    if execution_error:
        details.append(f"execution error: {execution_error}")
    if sender is not None and sender.returncode != 0:
        details.append(f"sender exited {sender.returncode}")
    if dialog_proc is not None and dialog_proc.returncode != 0:
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
    return (
        got == case.expected
        and sender is not None
        and sender.returncode == 0
        and dialog_proc is not None
        and dialog_proc.returncode == 0
        and (daemon is None or daemon.returncode in {0, -15})
        and not execution_error,
        got,
        "\n".join(details),
    )


def readline_with_timeout(stream, timeout: float) -> str:
    selector = selectors.DefaultSelector()
    try:
        selector.register(stream, selectors.EVENT_READ)
        if not selector.select(timeout):
            raise TimeoutError("timed out waiting for test device path")
        return stream.readline()
    finally:
        selector.close()


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
    source = f"lay-ime-{layout}" if ime_engine else layout
    engine = (
        source
        if ime_engine
        else ("xkb:ru::rus" if layout == "ru" else "xkb:us::eng")
    )
    if (
        ime_engine
        and current_desktop_layout() == source
        and current_ibus_engine() != engine
    ):
        alternate = "lay-ime-us" if layout == "ru" else "lay-ime-ru"
        activate_gnome_source(alternate)
        wait_for_layout_postcondition(alternate, alternate, exact_source=True)
    activate_gnome_source(source)
    if not ime_engine:
        set_ibus_engine(engine)
    wait_for_layout_postcondition(source, engine, exact_source=ime_engine)


def activate_gnome_source(source: str) -> None:
    result = gnome_layout_call("ActivateLayout", source)
    if result.returncode == 0 and result.stdout.strip() == "(true,)":
        return
    raise RuntimeError(
        f"GNOME refused layout {source!r}: "
        f"stdout={result.stdout.strip()!r} stderr={result.stderr.strip()!r}"
    )


def wait_for_layout_postcondition(
    source: str,
    engine: str,
    *,
    exact_source: bool,
) -> None:
    deadline = time.monotonic() + 2.0
    while time.monotonic() < deadline:
        current_source = current_desktop_layout()
        source_matches = (
            current_source == source
            if exact_source
            else layout_kind(current_source) == source
        )
        if source_matches and current_ibus_engine() == engine:
            return
        time.sleep(0.05)
    raise RuntimeError(
        f"layout postcondition was not observed for source={source!r} "
        f"engine={engine!r}"
    )


def layout_kind(source: str) -> str:
    value = source.strip().lower()
    if value == "ru" or value == "lay-ime-ru" or ":ru" in value or "rus" in value:
        return "ru"
    if value in {"us", "en", "lay-ime-us"} or ":us" in value or "eng" in value:
        return "us"
    return value


def activate_layout_kde(layout: str) -> bool:
    qdbus = (
        shutil.which("qdbus6")
        or shutil.which("qdbus-qt6")
        or shutil.which("qdbus")
    )
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
