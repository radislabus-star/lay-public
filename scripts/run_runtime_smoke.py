#!/usr/bin/env python3
"""Run live lay-daemon smoke tests against a real GTK text field.

This is intentionally a runtime harness, not a unit test. It opens a Zenity
entry dialog, sends physical key events through `lay-test-input`, then compares
the text returned by the dialog after Enter.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path

if __name__ == "__main__" and os.environ.get("IBUS_ENABLE_SYNC_MODE") != "1":
    environment = {**os.environ, "IBUS_ENABLE_SYNC_MODE": "1"}
    os.execvpe(sys.executable, [sys.executable, *sys.argv], environment)

from runtime_smoke.cases import CASES
from runtime_smoke.desktop import (
    capture_desktop_snapshot,
    managed_desktop_session,
    xkb_fallback_for,
)
from runtime_smoke.execution import run_case
from runtime_smoke.ime import managed_ime_case, trace_summary, write_managed_ime_config
from runtime_smoke.isolation import (
    CleanupSignalHandlers,
    prepare_case_context,
)
from runtime_smoke.receipt import (
    CURRENT_SCHEMA,
    case_results_sha256,
    validate_runtime_smoke_receipt,
)


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INPUT = ROOT / "target/release/lay-test-input"
DEFAULT_DAEMON = ROOT / "target/release/lay-daemon"
DEFAULT_IBUS_ENGINE = ROOT / "target/release/lay-ibus-engine"
RUN_ID = re.compile(r"[A-Za-z0-9._-]+")


def validate_live_admission(args) -> None:
    if not args.managed_desktop:
        raise ValueError(
            "live runtime smoke requires explicit --managed-desktop admission"
        )
    if not args.ime_managed:
        raise ValueError(
            "live runtime smoke requires --ime-managed per-case engine isolation"
        )
    if not args.verify_ime_trace:
        raise ValueError(
            "live runtime smoke requires --verify-ime-trace evidence validation"
        )
    if args.ime_engine and not args.ime_managed:
        raise ValueError("--ime-engine requires --ime-managed isolation")
    if args.use_system_daemon:
        raise ValueError(
            "--use-system-daemon is not admitted; every case requires an isolated daemon"
        )
    if getattr(args, "timeout", 1.0) <= 0:
        raise ValueError("--timeout must be positive")
    if getattr(args, "focus_delay", 0.0) < 0:
        raise ValueError("--focus-delay cannot be negative")


def validate_case_trace_contract(case, trace: dict[str, object]) -> tuple[bool, str]:
    checks: list[bool] = []
    details: list[str] = []

    if case.expected_manual_toggles is not None:
        got = trace["manual_toggles"]
        expected = case.expected_manual_toggles
        checks.append(got == expected)
        details.append(f"manual_toggles={got}/{expected}")

    if case.expected_preedit_updates is not None:
        got = tuple(trace["preedit_updates"])
        expected = case.expected_preedit_updates
        checks.append(got == expected)
        details.append(f"preedit_updates={got!r}/{expected!r}")

    if case.expected_managed_commits is not None:
        got = tuple(trace["managed_commits"])
        expected = case.expected_managed_commits
        checks.append(got == expected)
        details.append(f"managed_commits={got!r}/{expected!r}")

    if case.expected_pending_shortens is not None:
        got = trace["pending_shortens"]
        expected = case.expected_pending_shortens
        checks.append(got == expected)
        details.append(f"pending_shortens={got}/{expected}")

    if case.expected_completion_accepts is not None:
        got = trace["kind_counts"].get("ibus_completion_accept", 0)
        expected = case.expected_completion_accepts
        checks.append(got == expected)
        details.append(f"completion_accepts={got}/{expected}")

    ibus_keys = int(trace["kind_counts"].get("ibus_key", 0))
    if case.minimum_ibus_keys:
        checks.append(ibus_keys >= case.minimum_ibus_keys)
        details.append(f"ibus_keys={ibus_keys}>={case.minimum_ibus_keys}")

    preedit_clears = int(trace["preedit_clears"])
    if case.minimum_preedit_clears:
        checks.append(preedit_clears >= case.minimum_preedit_clears)
        details.append(
            f"preedit_clears={preedit_clears}>={case.minimum_preedit_clears}"
        )

    trace_clean = trace["read_error"] is None and trace["malformed"] == 0
    return trace_clean and all(checks), " ".join(details)


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
    parser.add_argument(
        "--use-system-daemon",
        action="store_true",
        help="rejected legacy option; every live case requires an isolated daemon",
    )
    parser.add_argument(
        "--managed-desktop",
        action="store_true",
        help="admit temporary, verified mutation and restoration of the live desktop",
    )
    parser.add_argument(
        "--run-id",
        help="stable run identity for order-equivalence checks; defaults to a UUID",
    )
    parser.add_argument(
        "--evidence-dir",
        type=Path,
        help="new persistent evidence directory; defaults under target/runtime-smoke-runs",
    )
    parser.add_argument("--daemon-debug", action="store_true")
    parser.add_argument(
        "--verify-ime-trace",
        action="store_true",
        help="require the preregistered manual-toggle count from the IBus trace",
    )
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument(
        "--json-out",
        type=Path,
        help="also write the complete v2 receipt to this path",
    )
    parser.add_argument(
        "--ime-engine",
        action="store_true",
        help="use lay-ime-ru/lay-ime-us IBus engines as the start layout",
    )
    parser.add_argument(
        "--ime-managed",
        action="store_true",
        help="run a fresh repository-local IBus engine for every case",
    )
    args = parser.parse_args()
    try:
        validate_live_admission(args)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    run_id = args.run_id or uuid.uuid4().hex
    if RUN_ID.fullmatch(run_id) is None:
        raise SystemExit("--run-id accepts only letters, digits, '.', '_', and '-'")

    evidence_root = prepare_evidence_root(args.evidence_dir, run_id)
    evidence_receipt = evidence_root / "RECEIPT.json"
    selected = [CASES[name] for name in (args.case or sorted(CASES))]
    results: list[dict[str, object]] = []
    desktop_receipt: dict[str, object] | None = None
    desktop_restoration_verified = False
    harness_process_group = os.getpgrp()
    active_case: str | None = None
    fatal_exception: BaseException | None = None
    binary_receipt: dict[str, object] | None = None
    previous_ibus_sync_mode = os.environ.get("IBUS_ENABLE_SYNC_MODE")
    os.environ["IBUS_ENABLE_SYNC_MODE"] = "1"

    try:
        dialog = choose_dialog_command(args.dialog)
        require_command("gdbus")
        require_command("ibus")
        require_command("systemctl")
        input_bin = ensure_binary(args.input_bin, "lay-test-input", args.no_build)
        daemon_bin = ensure_binary(args.daemon_bin, "lay-daemon", args.no_build)
        ibus_engine_bin = ensure_binary(
            args.ibus_engine_bin, "lay-ibus-engine", args.no_build
        )
        binary_receipt = {
            "input": binary_identity(input_bin),
            "daemon": binary_identity(daemon_bin),
            "ibus_engine": binary_identity(ibus_engine_bin),
        }

        with CleanupSignalHandlers():
            desktop_before = capture_desktop_snapshot()
            desktop_receipt = dataclasses.asdict(desktop_before)
            with managed_desktop_session(
                admitted=args.managed_desktop,
                replace_service=True,
                replace_lay_engines=args.ime_managed,
                snapshot=desktop_before,
            ) as desktop_before:
                fallback_source = xkb_fallback_for(desktop_before.active_engine)
                for case in selected:
                    active_case = case.name
                    context = prepare_case_context(
                        root=evidence_root,
                        run_id=run_id,
                        case_name=case.name,
                    )
                    write_case_config(
                        context.config_path,
                        case.config_overrides,
                        managed_ime=True,
                    )
                    with managed_ime_case(
                        ROOT,
                        ibus_engine_bin,
                        context,
                        fallback_source,
                    ):
                        ok, got, detail = run_case(
                            case,
                            context,
                            input_bin,
                            daemon_bin,
                            dialog,
                            args.focus_delay,
                            args.timeout,
                            args.daemon_debug,
                            True,
                        )
                    trace = trace_summary(context.trace_path)
                    if trace["read_error"] is not None or trace["malformed"] != 0:
                        ok = False
                        detail = append_detail(
                            detail,
                            "IME trace integrity: "
                            f"read_error={trace['read_error']!r} "
                            f"malformed={trace['malformed']}",
                        )
                    if args.verify_ime_trace:
                        trace_ok, trace_detail = validate_case_trace_contract(
                            case, trace
                        )
                        ok = ok and trace_ok
                        if trace_detail:
                            detail = append_detail(
                                detail,
                                f"IME case trace: {trace_detail} "
                                f"malformed={trace['malformed']}",
                            )
                    status = "OK" if ok else "BAD"
                    print(
                        f"{status} {case.name}: "
                        f"got={got!r} expected={case.expected!r}"
                    )
                    if detail:
                        print(indent(detail.rstrip()))
                    results.append(
                        {
                            "case_id": context.case_id,
                            "name": case.name,
                            "ok": ok,
                            "got": got,
                            "expected": case.expected,
                            "detail": detail,
                            "trace": trace,
                        }
                    )
                    active_case = None
            desktop_restoration_verified = True
        if os.getpgrp() != harness_process_group:
            raise RuntimeError("runtime smoke process group changed during execution")
    except BaseException as error:
        fatal_exception = error
    finally:
        if previous_ibus_sync_mode is None:
            os.environ.pop("IBUS_ENABLE_SYNC_MODE", None)
        else:
            os.environ["IBUS_ENABLE_SYNC_MODE"] = previous_ibus_sync_mode

    sorted_results = sorted(results, key=lambda item: str(item["name"]))
    fatal_error = (
        None
        if fatal_exception is None
        else f"{type(fatal_exception).__name__}: {fatal_exception}"
    )
    all_passed = (
        fatal_exception is None
        and desktop_restoration_verified
        and bool(sorted_results)
        and all(result["ok"] is True for result in sorted_results)
    )
    receipt: dict[str, object] = {
        "schema": CURRENT_SCHEMA,
        "run_id": run_id,
        "selected_cases": [case.name for case in selected],
        "execution_order": [case.name for case in selected],
        "active_case_at_failure": active_case,
        "all_passed": all_passed,
        "cases": sorted_results,
        "case_results_sha256": case_results_sha256(sorted_results),
        "desktop_before": desktop_receipt,
        "desktop_restoration_verified": desktop_restoration_verified,
        "harness_process_group": harness_process_group,
        "evidence_root": str(evidence_root),
        "fatal_error": fatal_error,
        "binaries": binary_receipt,
        "ibus_sync_mode": "1",
        "invocation": [str(Path(sys.argv[0]).resolve()), *sys.argv[1:]],
    }
    validate_runtime_smoke_receipt(receipt)
    write_json_atomic(evidence_receipt, receipt)
    if args.json_out is not None and args.json_out.resolve() != evidence_receipt:
        write_json_atomic(args.json_out, receipt)
    print(f"runtime smoke evidence: {evidence_receipt}")

    if fatal_exception is not None:
        raise fatal_exception
    return 0 if all_passed else 1


def require_command(name: str) -> None:
    if shutil.which(name) is None:
        raise SystemExit(f"required command not found: {name}")


def binary_identity(path: Path) -> dict[str, object]:
    resolved = path.resolve()
    with resolved.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    return {
        "path": str(resolved),
        "size": resolved.stat().st_size,
        "sha256": digest,
    }


def prepare_evidence_root(requested: Path | None, run_id: str) -> Path:
    path = requested
    if path is None:
        path = (
            ROOT
            / "target"
            / "runtime-smoke-runs"
            / f"{run_id}-{uuid.uuid4().hex[:12]}"
        )
    path = path.expanduser().resolve()
    path.mkdir(parents=True, exist_ok=False)
    return path


def choose_dialog_command(preferred: str) -> str:
    if preferred != "auto":
        if preferred == "gtk-entry-capture" and (
            ROOT / "scripts" / "gtk_entry_capture.py"
        ).exists():
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
        [
            str(ROOT / "scripts" / "cargo-guard.sh"),
            "build",
            "--release",
            "--bin",
            bin_name,
        ],
        cwd=ROOT,
        check=True,
    )
    if not path.exists():
        raise SystemExit(f"{bin_name} binary was not built: {path}")
    return path


def write_case_config(
    path: Path,
    overrides: dict[str, object] | None,
    *,
    managed_ime: bool,
) -> None:
    if managed_ime:
        write_managed_ime_config(path)
        config = json.loads(path.read_text(encoding="utf-8"))
    else:
        config = {
            "mode": "simple",
            "correction_engine": "replay",
            "replace_words": 1,
            "auto_replace": False,
            "typing_assist": False,
            "auto_switch_layout": True,
        }
    config.update(overrides or {})
    path.write_text(
        json.dumps(config, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def append_detail(current: str, extra: str) -> str:
    return "\n".join(part for part in (current, extra) if part)


def write_json_atomic(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as output:
            json.dump(value, output, ensure_ascii=False, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temp_name, path)
    except BaseException:
        Path(temp_name).unlink(missing_ok=True)
        raise


def indent(text: str) -> str:
    return "\n".join(f"  {line}" for line in text.splitlines())


if __name__ == "__main__":
    sys.exit(main())
