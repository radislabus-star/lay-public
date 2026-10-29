#!/usr/bin/env python3
"""Offline native receipt analysis. Never types, installs, connects or retries."""
import argparse
import json
from pathlib import Path
import time

from native_diagnostics import receipts, wire


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    report = sub.add_parser("report", help="validate/group a saved matrix; optionally compare a prior receipt")
    report.add_argument("input", type=Path, help="combined JSON, actor JSON or one-browser run directory")
    report.add_argument("--compare", type=Path, help="previous receipt, in the same supported formats")
    report.add_argument("--output", type=Path, required=True, help="new output directory (never overwritten)")
    timeline = sub.add_parser("wire", help="pair sanitized event JSONL and export a Perfetto timeline")
    timeline.add_argument("input", type=Path)
    timeline.add_argument("--compare", type=Path, help="previous sanitized wire capture")
    timeline.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    started = time.perf_counter()
    try:
        if args.command == "report":
            data = receipts.summarize(receipts.load_bundle(args.input))
            if args.compare:
                previous = receipts.summarize(receipts.load_bundle(args.compare))
                data["comparison"] = receipts.compare(previous, data)
            artifacts = {"report.json": data}
        else:
            data, trace = wire.analyze(wire.load_events(args.input))
            if args.compare:
                previous, _ = wire.analyze(wire.load_events(args.compare))
                data["comparison"] = wire.compare(previous, data)
            artifacts = {"report.json": data, "trace.perfetto.json": trace}
        data["analysis_elapsed_ms"] = (time.perf_counter() - started) * 1000
        args.output.mkdir(parents=True, exist_ok=False, mode=0o700)
        for name, value in artifacts.items():
            path = args.output / name
            path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n")
            path.chmod(0o600)
        if args.command == "report":
            compact = {k: data[k] for k in ("schema", "coverage", "counts", "warnings",
                       "causal_evidence", "rapid_timing", "analysis_elapsed_ms")}
            compact["failure_groups"] = [g for g in data["groups"] if g["status"] == "FAIL"]
            compact["blocked_by_assertion"] = {}
            for group in data["groups"]:
                if group["status"] == "BLOCKED":
                    key = group["assertion"]
                    compact["blocked_by_assertion"][key] = compact["blocked_by_assertion"].get(key, 0) + group["count"]
        else:
            compact = {k: v for k, v in data.items() if k != "comparison"}
        if "comparison" in data:
            compact["comparison"] = {k: v for k, v in data["comparison"].items() if k != "changed_cases"}
        compact["report"] = str(args.output / "report.json")
        print(json.dumps(compact, ensure_ascii=False, indent=2))
        return 3 if data.get("pairing") == "INCOMPLETE" or data.get("comparison", {}).get("pairing_complete") is False else 0
    except (receipts.EvidenceError, OSError, ValueError) as error:
        print(json.dumps({"status": "INVALID_EVIDENCE_OR_OUTPUT", "error": str(error)}))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
