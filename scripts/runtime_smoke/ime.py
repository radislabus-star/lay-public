from __future__ import annotations

import contextlib
import json
import os
import subprocess
import tempfile
import time
from pathlib import Path


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
    temp_dir = tempfile.TemporaryDirectory(prefix="lay-ime-managed-")
    engine = None
    try:
        subprocess.run(
            ["systemctl", "--user", "stop", "lay-daemon"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        subprocess.run(
            ["pkill", "-x", "lay-ibus-engine"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

        config_path = Path(temp_dir.name) / "config.json"
        write_managed_ime_config(config_path)
        env = {**os.environ, "LAY_CONFIG_PATH": str(config_path)}
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
        subprocess.run(
            ["ibus", "engine", "xkb:ru::rus"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if daemon_was_active:
            subprocess.run(
                ["systemctl", "--user", "restart", "lay-daemon"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        temp_dir.cleanup()


def stop_managed_ime(engine: subprocess.Popen[str] | None) -> None:
    if engine is not None and engine.poll() is None:
        engine.terminate()
        try:
            engine.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            engine.kill()
            engine.communicate()
    subprocess.run(
        ["pkill", "-x", "lay-ibus-engine"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


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
        "text_backend": "ime",
    }
    path.write_text(json.dumps(config, ensure_ascii=False, indent=2), encoding="utf-8")
