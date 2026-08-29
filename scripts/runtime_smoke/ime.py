from __future__ import annotations

import contextlib
import hashlib
import json
import subprocess
import time
from pathlib import Path
from typing import Protocol

from runtime_smoke.desktop import (
    discover_lay_ibus_engines,
    set_ibus_engine,
)
from runtime_smoke.isolation import ProcessSupervisor


VOLATILE_TRACE_KINDS = frozenset({"ibus_cursor"})


class CaseEnvironment(Protocol):
    trace_path: Path

    def environment(self) -> dict[str, str]: ...


@contextlib.contextmanager
def managed_ime_case(
    root: Path,
    ibus_engine_bin: Path | None,
    case: CaseEnvironment,
    fallback_source: str,
):
    if ibus_engine_bin is None:
        raise RuntimeError("managed IME binary is not configured")
    existing = discover_lay_ibus_engines()
    if existing:
        pids = ", ".join(str(identity.pid) for identity in existing)
        raise RuntimeError(f"Lay IBus engine still active before case: {pids}")

    processes = ProcessSupervisor()
    try:
        case.trace_path.touch(exist_ok=False)
        engine = processes.spawn(
            "candidate-engine",
            [str(ibus_engine_bin), "--ibus", "--managed"],
            cwd=root,
            env=case.environment(),
            text=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        time.sleep(0.8)
        if engine.poll() is not None:
            stderr = engine.stderr.read() if engine.stderr is not None else ""
            raise RuntimeError(f"lay-ibus-engine exited early:\n{stderr}")
        yield
    finally:
        errors: list[str] = []
        for label, action in (
            ("fallback-engine", lambda: set_ibus_engine(fallback_source)),
            ("candidate-engine", processes.close),
        ):
            try:
                action()
            except Exception as error:
                errors.append(f"{label}: {type(error).__name__}: {error}")
        remaining = discover_lay_ibus_engines()
        if remaining:
            pids = ", ".join(str(identity.pid) for identity in remaining)
            errors.append(f"Lay IBus engine survived case cleanup: {pids}")
        if errors:
            raise RuntimeError("managed IME cleanup failed: " + "; ".join(errors))

def trace_summary(path: Path) -> dict[str, object]:
    records = 0
    semantic_records = 0
    volatile_records = 0
    malformed = 0
    manual_toggles = 0
    kind_counts: dict[str, int] = {}
    semantic_kind_counts: dict[str, int] = {}
    if not path.is_file():
        return trace_error("FileNotFoundError: trace file is missing")
    try:
        raw = path.read_bytes()
    except OSError as error:
        return trace_error(f"{type(error).__name__}: {error}")
    digest = hashlib.sha256(raw).hexdigest()
    if not raw:
        return trace_error("ValueError: trace file is empty", digest=digest)
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        return trace_error(f"UnicodeDecodeError: {error}", digest=digest)
    for line in text.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            malformed += 1
            continue
        if not isinstance(record, dict) or not isinstance(record.get("kind"), str):
            malformed += 1
            continue
        kind = record["kind"]
        records += 1
        kind_counts[kind] = kind_counts.get(kind, 0) + 1
        if kind in VOLATILE_TRACE_KINDS:
            volatile_records += 1
        else:
            semantic_records += 1
            semantic_kind_counts[kind] = semantic_kind_counts.get(kind, 0) + 1
        if kind in {
            "ibus_manual_toggle_plan",
            "ibus_manual_toggle_delegation",
        }:
            manual_toggles += 1
    return {
        "records": records,
        "semantic_records": semantic_records,
        "volatile_records": volatile_records,
        "kind_counts": dict(sorted(kind_counts.items())),
        "semantic_kind_counts": dict(sorted(semantic_kind_counts.items())),
        "malformed": malformed,
        "manual_toggles": manual_toggles,
        "sha256": digest,
        "read_error": None,
    }


def trace_error(message: str, *, digest: str | None = None) -> dict[str, object]:
    return {
        "records": 0,
        "semantic_records": 0,
        "volatile_records": 0,
        "kind_counts": {},
        "semantic_kind_counts": {},
        "malformed": 0,
        "manual_toggles": 0,
        "sha256": digest,
        "read_error": message,
    }


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
