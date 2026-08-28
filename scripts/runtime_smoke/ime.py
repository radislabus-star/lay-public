from __future__ import annotations

import contextlib
import json
import os
import signal
import subprocess
import tempfile
import time
from pathlib import Path


def stop_all_lay_ibus_engines() -> None:
    result = subprocess.run(
        ["pgrep", "-f", r"(^|/)lay-ibus-engine --ibus( --managed)?$"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    for raw_pid in result.stdout.splitlines():
        try:
            pid = int(raw_pid)
            executable = os.readlink(f"/proc/{pid}/exe")
        except (OSError, ValueError):
            continue
        executable = executable.removesuffix(" (deleted)")
        if Path(executable).name == "lay-ibus-engine":
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass


def current_ibus_engine() -> str:
    result = subprocess.run(
        ["ibus", "engine"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    return result.stdout.strip()


def restore_ibus_engine(engine: str) -> None:
    if not engine:
        engine = "xkb:ru::rus"
    if engine.startswith("lay-ime-"):
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
                engine,
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    for _ in range(8):
        subprocess.run(
            ["ibus", "engine", engine],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if current_ibus_engine() == engine:
            return
        time.sleep(0.15)


@contextlib.contextmanager
def managed_ime_session(root: Path, ibus_engine_bin: Path | None):
    if ibus_engine_bin is None:
        raise SystemExit("managed IME requested but lay-ibus-engine binary is not configured")

    daemon_was_active = (
        subprocess.run(
            ["systemctl", "--user", "is-active", "--quiet", "lay-daemon"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode
        == 0
    )
    original_engine = current_ibus_engine()
    temp_dir = tempfile.TemporaryDirectory(prefix="lay-ime-managed-")
    previous_trace_path = os.environ.get("LAY_IBUS_TRACE_PATH")
    trace_path = (
        Path(previous_trace_path)
        if previous_trace_path
        else Path(temp_dir.name) / "ibus_engine_debug.jsonl"
    )
    engine = None
    try:
        subprocess.run(
            ["systemctl", "--user", "stop", "lay-daemon"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        stop_all_lay_ibus_engines()

        config_path = Path(temp_dir.name) / "config.json"
        write_managed_ime_config(config_path)
        env = {
            **os.environ,
            "LAY_CONFIG_PATH": str(config_path),
            "LAY_NANDA_WORD_USAGE_EVENTS": str(Path(temp_dir.name) / "events.jsonl"),
            "LAY_NANDA_WORD_USAGE_COUNTS": str(Path(temp_dir.name) / "counts.json"),
            "LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS": str(
                Path(temp_dir.name) / "feedback-counts.json"
            ),
            "LAY_IBUS_TRACE_PATH": str(trace_path),
        }
        os.environ["LAY_IBUS_TRACE_PATH"] = str(trace_path)
        engine = subprocess.Popen(
            [str(ibus_engine_bin), "--ibus", "--managed"],
            cwd=root,
            env=env,
            text=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        time.sleep(0.8)
        if engine.poll() is not None:
            stderr = engine.stderr.read() if engine.stderr is not None else ""
            raise SystemExit(f"lay-ibus-engine exited early:\n{stderr}")
        yield
    finally:
        stop_managed_ime(engine)
        restore_ibus_engine(original_engine)
        if daemon_was_active:
            subprocess.run(
                ["systemctl", "--user", "restart", "lay-daemon"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        if previous_trace_path is None:
            os.environ.pop("LAY_IBUS_TRACE_PATH", None)
        else:
            os.environ["LAY_IBUS_TRACE_PATH"] = previous_trace_path
        temp_dir.cleanup()


def stop_managed_ime(engine: subprocess.Popen[str] | None) -> None:
    if engine is not None and engine.poll() is None:
        engine.terminate()
        try:
            engine.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            engine.kill()
            engine.communicate()
    stop_all_lay_ibus_engines()


def write_managed_ime_config(path: Path) -> None:
    config = {
        "mode": "simple",
        "correction_engine": "replay",
        "replace_words": 1,
        "typing_assist_words": 2,
        "auto_replace": True,
        "typing_assist": True,
        "correction_safety": "experimental",
        "auto_switch_layout": True,
        "nanda_autocorrect": True,
        "text_backend": "ime",
        "nanda_precognition": True,
        "debug_action_log": True,
    }
    path.write_text(json.dumps(config, ensure_ascii=False, indent=2), encoding="utf-8")
