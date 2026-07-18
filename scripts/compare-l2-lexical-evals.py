#!/usr/bin/env python3
"""Compare two shadow lexical-memory evaluations without granting promotion."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    return parser.parse_args()


def load(path: Path) -> dict:
    report = json.loads(path.read_text(encoding="utf-8"))
    if report.get("schema") != "lay.l2-lexical-corpus-eval.v1":
        raise SystemExit(f"unsupported report: {path}")
    return report


def delta(candidate: dict, baseline: dict, key: str) -> float:
    return float(candidate[key]) - float(baseline[key])


def main() -> None:
    args = parse_args()
    baseline = load(args.baseline)
    candidate = load(args.candidate)
    blockers: list[str] = []
    watches: list[str] = []

    if baseline["total"]["rows"] != candidate["total"]["rows"]:
        blockers.append("row_count_mismatch")
    if candidate.get("memory", {}).get("raw_word_table") is not False:
        blockers.append("candidate_raw_word_table_not_false")

    coverage_delta = delta(candidate["total"], baseline["total"], "coverage_pct")
    top1_delta = delta(candidate["total"], baseline["total"], "top1_pct")
    top3_delta = delta(candidate["total"], baseline["total"], "top3_pct")
    if coverage_delta < 0.0:
        blockers.append("coverage_regressed")
    if top1_delta < 0.0:
        blockers.append("top1_regressed")
    if top3_delta < 0.0:
        blockers.append("top3_regressed")

    for operation, baseline_metrics in baseline["by_operation"].items():
        candidate_metrics = candidate["by_operation"].get(operation)
        if candidate_metrics is None:
            blockers.append(f"operation_missing:{operation}")
            continue
        if delta(candidate_metrics, baseline_metrics, "top1_pct") < -0.5:
            blockers.append(f"operation_top1_regressed:{operation}")

    baseline_p99 = int(baseline["hot_latency_us"]["p99"])
    candidate_p99 = int(candidate["hot_latency_us"]["p99"])
    latency_ceiling = max(500, int(baseline_p99 * 1.5))
    if candidate_p99 > latency_ceiling:
        blockers.append("p99_latency_regressed")
    if top1_delta < 1.0:
        watches.append("top1_gain_below_one_point")

    verdict = "VETO" if blockers else "PASS" if not watches else "WATCH"
    print(
        json.dumps(
            {
                "schema": "lay.l2-lexical-eval-comparison.v1",
                "verdict": verdict,
                "baseline": str(args.baseline),
                "candidate": str(args.candidate),
                "rows": candidate["total"]["rows"],
                "coverage_delta_pct": coverage_delta,
                "top1_delta_pct": top1_delta,
                "top3_delta_pct": top3_delta,
                "baseline_p99_us": baseline_p99,
                "candidate_p99_us": candidate_p99,
                "latency_ceiling_us": latency_ceiling,
                "blockers": blockers,
                "watches": watches,
                "live_authority": False,
            },
            ensure_ascii=False,
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
