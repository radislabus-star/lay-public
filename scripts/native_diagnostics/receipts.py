"""Validate and summarize native actor/controller/verification receipts."""
from __future__ import annotations

from collections import Counter, defaultdict
import hashlib
import json
import math
from pathlib import Path
import re
import statistics

STATUSES = ("PASS", "FAIL", "BLOCKED", "NOT_TESTED")
CASE_KEYS = ("window", "field", "name", "word")
MAX_BYTES = 32 * 1024 * 1024


class EvidenceError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def read_json(path):
    path = Path(path)
    require(path.stat().st_size <= MAX_BYTES, "receipt exceeds 32 MiB")
    try:
        return json.loads(path.read_text(), parse_constant=lambda _: (_ for _ in ()).throw(
            EvidenceError("non-finite JSON number")))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise EvidenceError("invalid JSON receipt") from error


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def safe_label(value):
    return value if isinstance(value, str) and re.fullmatch(r"[a-zA-Z0-9_-]{1,64}", value) else "[redacted]"


def sha(value):
    return value if isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) else None


def load_bundle(path):
    path = Path(path)
    if not path.is_dir():
        data = read_json(path)
        require(isinstance(data, dict), "receipt root must be an object")
        if "actor" in data:
            return data
        require("rows" in data and "expected_rows" in data, "unsupported receipt format")
        return {"actor": data}
    actors = [path / browser / "result.json" for browser in ("firefox", "chrome")
              if (path / browser / "result.json").is_file()]
    require(len(actors) == 1, "directory must contain exactly one browser result")
    verification = path.parent / (path.name + "-verification.json")
    return {"actor": read_json(actors[0]),
            "controller": read_json(path / "result.json") if (path / "result.json").is_file() else {},
            "verification": read_json(verification) if verification.is_file() else {}}


def assertion_class(row):
    error = str(row.get("error", ""))
    if row["status"] == "BLOCKED":
        if "TD137:" in error:
            return "SHARED_LEARNING_BLOCK"
        if "sanitizes trailing ASCII Space" in error:
            return "FIELD_SPACE_SANITIZATION"
        if "native/DOM text disagreement" in error:
            return "OBSERVERS_DISAGREE"
        return "UNCLASSIFIED_BLOCK"
    if error.startswith("{'late_targets':") or error.startswith('{"late_targets":'):
        return "TARGET_AFTER_NEXT_GESTURE"
    if error.startswith("{'expected':") or error.startswith('{"expected":'):
        return "EXPECTED_TARGET_ASSERTION"
    return "UNCLASSIFIED_FAILURE"


def summarize(bundle):
    require(isinstance(bundle, dict), "bundle must be an object")
    actor = bundle.get("actor", {})
    verification = bundle.get("verification", {})
    controller = bundle.get("controller", {})
    require(all(isinstance(x, dict) for x in (actor, verification, controller)), "invalid receipt sections")
    require(actor.get("scope") == "ACTUAL_NATIVE_RFB_BROWSER", "receipt is not from the supported native actor")
    require(actor.get("browser") in ("firefox", "chrome"), "unsupported browser")
    rows = actor.get("rows")
    expected = actor.get("expected_rows")
    require(isinstance(rows, list) and rows, "empty or missing actor rows")
    require(type(expected) is int and expected >= len(rows), "invalid expected_rows")
    identities = []
    observed = Counter()
    for row in rows:
        require(isinstance(row, dict), "invalid row")
        require(all(isinstance(row.get(k), str) and row[k] for k in CASE_KEYS), "missing case identity")
        require(row.get("status") in STATUSES, "unknown case status")
        identities.append(tuple(row[k] for k in CASE_KEYS))
        observed[row["status"]] += 1
    require(len(set(identities)) == len(identities), "duplicate case identities")
    for label, declared in (("actor", actor.get("counts")), ("verification", verification.get("counts"))):
        if declared is None and label == "verification":
            continue
        require(isinstance(declared, dict) and not set(declared).difference(STATUSES), label + " invalid counts")
        require(all(type(n) is int and n >= 0 for n in declared.values()), label + " invalid count value")
        require(all(declared.get(s, 0) == observed[s] for s in STATUSES), label + " counts disagree with rows")
    require(actor.get("status") != "PASS" or (observed["PASS"] == expected), "actor PASS contradicts rows")
    require(not actor.get("held_after"), "held keys invalidate completed-run evidence")
    unrecorded = expected - len(rows)
    counts = {s: observed[s] + (unrecorded if s == "NOT_TESTED" else 0) for s in STATUSES}
    full = expected == 220 and not unrecorded
    if full:
        require({r["window"] for r in rows} == {"normal", "private", "fullscreen"}, "full matrix window modes differ")
        require(sum(r["name"] == "consecutive-two" for r in rows) == 60, "full matrix 160+60 denominator differs")
    before, after = verification.get("runtime_before"), verification.get("runtime_after")
    require(all(x is None or isinstance(x, dict) for x in (before, after)), "invalid runtime snapshots")
    runtime_consistent = bool(before and before == after and verification.get("runtime_preserved") is True)
    warnings = []
    if not runtime_consistent:
        warnings.append("RUNTIME_SNAPSHOTS_MISSING_OR_CHANGED")
    if verification.get("config_restored") is not True:
        warnings.append("CONFIG_RESTORATION_UNCONFIRMED")
    config_identity = sha(verification.get("config_sha256"))
    if not config_identity:
        warnings.append("CONFIG_IDENTITY_UNRECORDED")
    actor_hash = sha(verification.get("actor_sha256"))
    controller_hash = sha(controller.get("actor_sha256"))
    if not actor_hash or actor_hash != controller_hash:
        warnings.append("ACTOR_IDENTITY_MISSING_OR_MISMATCHED")
    if not verification.get("finished_utc"):
        warnings.append("SUPERVISOR_COMPLETION_UNCONFIRMED")
    if "held_after" not in actor:
        warnings.append("FINAL_HELD_KEYS_UNRECORDED")
    if verification.get("controller_exit_code", 0) != 0 and not (observed["FAIL"] or observed["BLOCKED"]):
        warnings.append("SUPERVISOR_ERROR_WITHOUT_RECORDED_PRODUCT_FAILURE")
    launcher = controller.get("owned_production_launcher_alignment")
    launcher = dict(library_sha256=sha(launcher.get("library_sha256")),
        gtk_ime=launcher.get("gtk_ime") if launcher.get("gtk_ime") in ("ibus", "fcitx", "xim") else "unknown",
        actor_unchanged=launcher.get("actor_unchanged") is True,
        host_environment_changed=launcher.get("host_environment_changed") is not False) if isinstance(launcher, dict) else None
    if launcher is None:
        warnings.append("LAUNCHER_IDENTITY_UNRECORDED")
    elif not launcher["library_sha256"] or launcher["gtk_ime"] == "unknown":
        warnings.append("LAUNCHER_IDENTITY_INCOMPLETE")
    if launcher and controller.get("owned_exact_refresh_observed") is not True:
        warnings.append("DECLARED_ADAPTER_NOT_OBSERVED")
    groups = defaultdict(list)
    cases = []
    latency_groups = defaultdict(list)
    for row, identity in zip(rows, identities):
        case = {k: safe_label(row[k]) for k in CASE_KEYS}
        case.update(id=digest(identity), status=row["status"])
        if row["status"] in ("FAIL", "BLOCKED"):
            case["assertion"] = assertion_class(row)
            groups[(row["status"], case["assertion"], case["name"], case["word"])].append(case)
        cases.append(case)
        if row["name"] == "rapid-five":
            latency_groups[row["word"]].append(row)
    failures = [r for r in rows if r["status"] == "FAIL"]
    anchor_count = sum(r.get("native_trace_complete") is True for r in failures)
    if anchor_count != len(failures) or controller.get("trace_complete") is not True:
        warnings.append("CAUSAL_CAPTURE_INCOMPLETE")
    timing = {}
    for word, group in latency_groups.items():
        require(all(isinstance(r.get("latencies_ms", []), list) for r in group), "invalid latency sample list")
        values = [v for r in group for v in r.get("latencies_ms", [])]
        valid = sorted(v for v in values if type(v) in (int, float) and math.isfinite(v) and v >= 0)
        target = sum(5 for r in group if r["status"] not in ("BLOCKED", "NOT_TESTED"))
        timing[safe_label(word)] = dict(expected_samples=target, observed_samples=len(valid),
            invalid_samples=len(values) - len(valid), median_ms=statistics.median(valid) if valid else None,
            p95_ms=valid[math.ceil(len(valid) * .95) - 1] if valid else None, max_ms=max(valid) if valid else None)
    source = verification.get("source", {})
    source = sha(source.get("sha256")) if isinstance(source, dict) else None
    if not source:
        warnings.append("SOURCE_IDENTITY_UNRECORDED")
    if any(t["observed_samples"] < t["expected_samples"] for t in timing.values()):
        warnings.append("TIMING_SAMPLES_MISSING")
    if any(t["invalid_samples"] or t["observed_samples"] > t["expected_samples"] for t in timing.values()):
        warnings.append("TIMING_COVERAGE_INCONSISTENT")
    binaries = {safe_label(k): sha(v.get("sha256")) for k, v in (before or {}).items() if isinstance(v, dict)}
    result = dict(schema="lay.native-diagnostic.v1", browser=safe_label(actor.get("browser")),
        coverage="COMPLETE_MATRIX" if full else ("PARTIAL" if unrecorded else "COMPLETE_SELECTED_SET"),
        counts=counts, expected_rows=expected, recorded_rows=len(rows), unrecorded_rows=unrecorded,
        provenance=dict(source_sha256=source, actor_sha256=actor_hash, config_sha256=config_identity, binaries=binaries, launcher=launcher,
                        runtime_snapshots_consistent=runtime_consistent), warnings=warnings,
        evidence_time="SAVED_RECEIPT_NOT_A_CURRENT_RUNTIME_PROBE",
        groups=[dict(status=k[0], assertion=k[1], action=k[2], word_state=k[3], count=len(v),
                     fields=sorted({c["field"] for c in v}), windows=sorted({c["window"] for c in v}))
                for k, v in sorted(groups.items())], cases=cases,
        causal_evidence=dict(failed_rows=len(failures), failed_rows_with_trace_anchor=anchor_count,
            global_trace_anchor=controller.get("trace_complete") is True,
            first_failed_transition="UNKNOWN_REQUIRES_CORRELATED_EVENTS",
            note="An intact file anchor is not proof of per-gesture causal completeness."),
        rapid_timing=timing, timing_scope="Actor RFB-send release mapped to DOM; not independent hardware timing.",
        failure_fingerprint=digest(sorted((c["id"], c.get("assertion")) for c in cases if c["status"] == "FAIL")),
        next_action="Use one representative failure and correlate the last proved transition before editing. Preserve capture gaps and test cadence.")
    return result


def compare(before, after):
    old = {r["id"]: r for r in before["cases"]}
    new = {r["id"]: r for r in after["cases"]}
    changes = []
    for key in ("source_sha256", "actor_sha256", "config_sha256", "binaries", "launcher"):
        if before["provenance"][key] != after["provenance"][key]:
            changes.append(key)
    missing = [k for k in ("source_sha256", "actor_sha256", "config_sha256", "launcher")
               if not before["provenance"][k] or not after["provenance"][k]]
    return dict(count_delta={s: after["counts"][s] - before["counts"][s] for s in STATUSES},
        changed_dimensions=changes, missing_identity_dimensions=missing,
        added_cases=len(new.keys() - old.keys()), removed_cases=len(old.keys() - new.keys()),
        changed_cases=[dict(case={k: new[i][k] for k in CASE_KEYS}, before=old[i]["status"], after=new[i]["status"])
                       for i in sorted(old.keys() & new.keys()) if old[i]["status"] != new[i]["status"]],
        same_failure_fingerprint=before["failure_fingerprint"] == after["failure_fingerprint"],
        causal_attribution="NOT_ESTABLISHED_BY_COUNT_DELTA",
        next_action="Repeated unchanged failures need a new distinguishing hypothesis or observation, not another identical run.")
