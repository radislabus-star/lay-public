from __future__ import annotations

import hashlib
import json
from pathlib import Path

from runtime_smoke.isolation import case_id_for


SCHEMA_V2 = "lay.runtime-smoke-receipt.v2"
SCHEMA_V3 = "lay.runtime-smoke-receipt.v3"
CURRENT_SCHEMA = SCHEMA_V3


def case_results_projection(
    results: list[dict[str, object]], *, schema: str = CURRENT_SCHEMA
) -> list[dict[str, object]]:
    projection = []
    for result in sorted(results, key=lambda item: str(item["name"])):
        trace = result.get("trace")
        if not isinstance(trace, dict):
            raise ValueError("case trace projection is missing")
        row = {
            "case_id": result["case_id"],
            "name": result["name"],
            "ok": result["ok"],
            "got": result["got"],
            "expected": result["expected"],
            "trace_malformed": trace.get("malformed"),
            "trace_manual_toggles": trace.get("manual_toggles"),
            "trace_read_error": trace.get("read_error"),
        }
        if schema == SCHEMA_V2:
            row["trace_records"] = trace.get("records")
        elif schema == SCHEMA_V3:
            row["trace_semantic_records"] = trace.get("semantic_records")
            row["trace_semantic_kind_counts"] = trace.get(
                "semantic_kind_counts"
            )
        else:
            raise ValueError("unsupported runtime smoke receipt schema")
        projection.append(row)
    return projection


def case_results_sha256(
    results: list[dict[str, object]], *, schema: str = CURRENT_SCHEMA
) -> str:
    encoded = json.dumps(
        case_results_projection(results, schema=schema),
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def validate_runtime_smoke_receipt(receipt: dict[str, object]) -> None:
    schema = receipt.get("schema")
    if schema not in {SCHEMA_V2, SCHEMA_V3}:
        raise ValueError("unsupported runtime smoke receipt schema")
    run_id = receipt.get("run_id")
    if not isinstance(run_id, str) or not run_id:
        raise ValueError("runtime smoke run_id is missing")
    all_passed = receipt.get("all_passed")
    if not isinstance(all_passed, bool):
        raise ValueError("runtime smoke all_passed is not boolean")
    selected = _string_list(receipt.get("selected_cases"), "selected cases")
    if not selected or len(set(selected)) != len(selected):
        raise ValueError("runtime smoke selected cases are empty or duplicated")
    if schema == SCHEMA_V3:
        execution_order = _string_list(
            receipt.get("execution_order"), "execution order"
        )
        if execution_order != selected:
            raise ValueError("runtime smoke execution order contradicts selected cases")

    cases = receipt.get("cases")
    if not isinstance(cases, list):
        raise ValueError("runtime smoke cases are missing")
    names: set[str] = set()
    case_ids: set[str] = set()
    clean_results = True
    for row in cases:
        if not isinstance(row, dict):
            raise ValueError("runtime smoke case is not an object")
        for key in ("case_id", "name", "got", "expected", "detail"):
            if not isinstance(row.get(key), str):
                raise ValueError(f"runtime smoke case {key} is missing")
        if not isinstance(row.get("ok"), bool):
            raise ValueError("runtime smoke case ok is not boolean")
        name = str(row["name"])
        case_id = str(row["case_id"])
        if name in names or case_id in case_ids:
            raise ValueError("runtime smoke case identity is duplicated")
        if name not in selected:
            raise ValueError("runtime smoke case was not selected")
        if case_id != case_id_for(run_id, name):
            raise ValueError("runtime smoke case_id is not deterministic")
        names.add(name)
        case_ids.add(case_id)
        trace_clean = _validate_trace(row.get("trace"), schema=schema)
        output_exact = row["got"] == row["expected"]
        if row["ok"] is True and (not output_exact or not trace_clean):
            raise ValueError("passing runtime smoke case contradicts output or trace")
        clean_results = clean_results and bool(row["ok"]) and output_exact and trace_clean

    expected_hash = case_results_sha256(cases, schema=schema)
    if receipt.get("case_results_sha256") != expected_hash:
        raise ValueError("runtime smoke case projection hash mismatch")
    restoration = receipt.get("desktop_restoration_verified")
    if not isinstance(restoration, bool):
        raise ValueError("runtime smoke restoration verdict is missing")
    process_group = receipt.get("harness_process_group")
    if (
        not isinstance(process_group, int)
        or isinstance(process_group, bool)
        or process_group <= 0
    ):
        raise ValueError("runtime smoke process group is invalid")
    evidence_root = receipt.get("evidence_root")
    if not isinstance(evidence_root, str) or not Path(evidence_root).is_absolute():
        raise ValueError("runtime smoke evidence root is invalid")
    fatal_error = receipt.get("fatal_error")
    if fatal_error is not None and not isinstance(fatal_error, str):
        raise ValueError("runtime smoke fatal_error is invalid")
    if "ibus_sync_mode" in receipt and receipt["ibus_sync_mode"] != "1":
        raise ValueError("runtime smoke IBus sync mode is invalid")
    active_case = receipt.get("active_case_at_failure")
    if active_case is not None and (
        not isinstance(active_case, str) or active_case not in selected
    ):
        raise ValueError("runtime smoke active failure case is invalid")
    desktop_before = receipt.get("desktop_before")
    if desktop_before is not None:
        _validate_desktop_snapshot(desktop_before)
    if schema == SCHEMA_V3:
        _validate_invocation(receipt.get("invocation"))
        binaries_complete = _validate_binaries(receipt.get("binaries"))
    else:
        binaries_complete = True

    computed_pass = (
        bool(cases)
        and names == set(selected)
        and clean_results
        and fatal_error is None
        and restoration
        and isinstance(desktop_before, dict)
        and binaries_complete
    )
    if all_passed != computed_pass:
        raise ValueError("runtime smoke all_passed contradicts evidence")


def _validate_trace(value: object, *, schema: str) -> bool:
    if not isinstance(value, dict):
        raise ValueError("runtime smoke case trace is missing")
    integer_fields = ["records", "malformed", "manual_toggles"]
    if schema == SCHEMA_V3:
        integer_fields.extend(["semantic_records", "volatile_records"])
        if "preedit_clears" in value or "preedit_updates" in value:
            integer_fields.append("preedit_clears")
        if "pending_shortens" in value:
            integer_fields.append("pending_shortens")
    for key in integer_fields:
        item = value.get(key)
        if not isinstance(item, int) or isinstance(item, bool) or item < 0:
            raise ValueError(f"runtime smoke trace {key} is invalid")
    digest = value.get("sha256")
    if digest is not None and not _is_sha256(digest):
        raise ValueError("runtime smoke trace sha256 is invalid")
    read_error = value.get("read_error")
    if read_error is not None and not isinstance(read_error, str):
        raise ValueError("runtime smoke trace read_error is invalid")
    if schema == SCHEMA_V3:
        if "preedit_clears" in value or "preedit_updates" in value:
            preedit_updates = value.get("preedit_updates")
            if not isinstance(preedit_updates, list) or not all(
                isinstance(item, str) for item in preedit_updates
            ):
                raise ValueError("runtime smoke preedit updates are invalid")
        if "managed_commits" in value:
            managed_commits = value.get("managed_commits")
            if not isinstance(managed_commits, list) or not all(
                isinstance(item, str) and len(item) == 1 for item in managed_commits
            ):
                raise ValueError("runtime smoke managed commits are invalid")
        kinds = _count_map(value.get("kind_counts"), "trace kind counts")
        semantic_kinds = _count_map(
            value.get("semantic_kind_counts"), "semantic trace kind counts"
        )
        if sum(kinds.values()) != value["records"]:
            raise ValueError("runtime smoke trace kind count total mismatch")
        if sum(semantic_kinds.values()) != value["semantic_records"]:
            raise ValueError("runtime smoke semantic trace count total mismatch")
        if value["semantic_records"] + value["volatile_records"] != value["records"]:
            raise ValueError("runtime smoke trace partition mismatch")
        expected_semantic = {
            key: count for key, count in kinds.items() if key != "ibus_cursor"
        }
        if semantic_kinds != expected_semantic:
            raise ValueError("runtime smoke semantic trace projection is invalid")
        if value["volatile_records"] != kinds.get("ibus_cursor", 0):
            raise ValueError("runtime smoke volatile trace count is invalid")
    return (
        read_error is None
        and value["malformed"] == 0
        and value["records"] > 0
        and _is_sha256(digest)
    )


def _validate_desktop_snapshot(value: object) -> None:
    if not isinstance(value, dict):
        raise ValueError("runtime smoke desktop snapshot is invalid")
    for key in ("active_layout", "active_engine"):
        if not isinstance(value.get(key), str) or not value[key]:
            raise ValueError(f"runtime smoke desktop {key} is invalid")
    service = value.get("service")
    if not isinstance(service, dict):
        raise ValueError("runtime smoke service snapshot is missing")
    for key in ("active_state", "sub_state", "unit_file_state"):
        if not isinstance(service.get(key), str) or not service[key]:
            raise ValueError(f"runtime smoke service {key} is invalid")
    main_pid = service.get("main_pid")
    if not isinstance(main_pid, int) or isinstance(main_pid, bool) or main_pid < 0:
        raise ValueError("runtime smoke service MainPID is invalid")
    main_process = service.get("main_process")
    if main_process is not None:
        _validate_process_identity(main_process)
        if main_process["pid"] != main_pid:
            raise ValueError("runtime smoke service identity contradicts MainPID")
    if service["active_state"] == "active" and (main_pid <= 0 or main_process is None):
        raise ValueError("active runtime smoke service identity is incomplete")
    if service["active_state"] == "inactive" and (main_pid != 0 or main_process is not None):
        raise ValueError("inactive runtime smoke service identity is contradictory")
    for key in ("ibus_daemons", "lay_engines"):
        processes = value.get(key)
        if not isinstance(processes, (list, tuple)):
            raise ValueError(f"runtime smoke desktop {key} is invalid")
        for process in processes:
            _validate_process_identity(process)
    trace_paths = value.get("lay_engine_trace_paths")
    if not isinstance(trace_paths, (list, tuple)) or not all(
        isinstance(path, str) and path for path in trace_paths
    ):
        raise ValueError("runtime smoke Lay engine trace paths are invalid")
    harness_path = value.get("harness_trace_path")
    if harness_path is not None and not isinstance(harness_path, str):
        raise ValueError("runtime smoke harness trace path is invalid")


def _validate_process_identity(value: object) -> None:
    if not isinstance(value, dict):
        raise ValueError("runtime smoke process identity is invalid")
    for key in ("pid", "start_time"):
        item = value.get(key)
        if not isinstance(item, int) or isinstance(item, bool) or item <= 0:
            raise ValueError(f"runtime smoke process {key} is invalid")
    if not isinstance(value.get("executable"), str) or not value["executable"]:
        raise ValueError("runtime smoke process executable is invalid")
    argv = value.get("argv")
    if not isinstance(argv, (list, tuple)) or not argv or not all(
        isinstance(item, str) for item in argv
    ):
        raise ValueError("runtime smoke process argv is invalid")


def _validate_invocation(value: object) -> None:
    if not isinstance(value, list) or not value or not all(
        isinstance(item, str) for item in value
    ):
        raise ValueError("runtime smoke invocation is invalid")
    required = {"--managed-desktop", "--ime-managed", "--verify-ime-trace"}
    if not required.issubset(value) or "--use-system-daemon" in value:
        raise ValueError("runtime smoke invocation does not prove isolated admission")


def _validate_binaries(value: object) -> bool:
    if value is None:
        return False
    if not isinstance(value, dict) or set(value) != {"input", "daemon", "ibus_engine"}:
        raise ValueError("runtime smoke binary identities are incomplete")
    for identity in value.values():
        if not isinstance(identity, dict):
            raise ValueError("runtime smoke binary identity is invalid")
        path = identity.get("path")
        size = identity.get("size")
        if not isinstance(path, str) or not Path(path).is_absolute():
            raise ValueError("runtime smoke binary path is invalid")
        if not isinstance(size, int) or isinstance(size, bool) or size <= 0:
            raise ValueError("runtime smoke binary size is invalid")
        if not _is_sha256(identity.get("sha256")):
            raise ValueError("runtime smoke binary sha256 is invalid")
    return True


def _string_list(value: object, label: str) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise ValueError(f"runtime smoke {label} is invalid")
    return value


def _count_map(value: object, label: str) -> dict[str, int]:
    if not isinstance(value, dict):
        raise ValueError(f"runtime smoke {label} is invalid")
    if not all(
        isinstance(key, str)
        and isinstance(item, int)
        and not isinstance(item, bool)
        and item >= 0
        for key, item in value.items()
    ):
        raise ValueError(f"runtime smoke {label} is invalid")
    return value


def _is_sha256(value: object) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )
