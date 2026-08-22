#!/usr/bin/env python3
"""Classify immutable IME cases by their first observed loss stage."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import re
import tarfile
from pathlib import Path


CANDIDATE_LINE = re.compile(r"candidate-lattice source=nanda count=(\d+) replacements=(\[.*\])")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def target_is_present(log_text: str, target: str) -> tuple[bool, int]:
    normalized_target = target.strip()
    candidate_count = 0
    for line in log_text.splitlines():
        match = CANDIDATE_LINE.search(line)
        if match is None:
            continue
        candidates = json.loads(match.group(2))
        candidate_count += len(candidates)
        for candidate in candidates:
            normalized = candidate.strip()
            if normalized == normalized_target or normalized.endswith(" " + normalized_target):
                return True, candidate_count
    return False, candidate_count


def classify(status: str, target_present: bool, observation: dict | None, log_text: str) -> str:
    if status == "PASS":
        return "AlreadySatisfied"
    if observation is None:
        raise ValueError("failed case has no frozen target observation")
    if observation["assertion_scope"] == "legacy_producer_identity":
        if not target_present:
            raise ValueError("producer-identity assertion failed before the semantic target was born")
        if 'left: Some("CanonicalL2FieldBoundary")' not in log_text or 'right: Some("glued_phrase")' not in log_text:
            raise ValueError("legacy producer assertion is not evidenced by the immutable panic")
        return "NonSemanticLegacyProducerAssertion"
    if not target_present:
        return "BirthOrRetention"
    if observation["requires_independent_context"]:
        return "ContextSettlement"
    return "LexicalAuthority"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    spec_bytes = args.input.read_bytes()
    spec = json.loads(spec_bytes)
    root = Path(spec["immutable_evidence_root"])
    summary_path = root / spec["run_summary"]
    summary_bytes = summary_path.read_bytes()
    summary = json.loads(summary_bytes)

    with tarfile.open(root / spec["source_archive"], "r:*") as archive:
        source_member = archive.extractfile(spec["source_member"])
        if source_member is None:
            raise ValueError("frozen source member is absent")
        source_bytes = source_member.read()
    if sha256(source_bytes) != spec["source_sha256"]:
        raise ValueError("frozen source SHA-256 mismatch")

    receipts = [item for item in summary["receipts"] if item["planned"]["series"] == "ime36"]
    observations = {item["id"]: item for item in spec["failed_case_observations"]}
    if len(receipts) != spec["expected_case_count"] or len(observations) != spec["expected_fail_count"]:
        raise ValueError("frozen denominator count mismatch")
    if len(observations) != len(spec["failed_case_observations"]):
        raise ValueError("duplicate failed-case observation ID")

    cases = []
    counts: dict[str, int] = {}
    statuses: dict[str, int] = {}
    for receipt in sorted(receipts, key=lambda item: item["planned"]["id"]):
        case_id = receipt["planned"]["id"]
        status = receipt["observed_status"]
        statuses[status] = statuses.get(status, 0) + 1
        log_path = root / "output" / "logs" / "ime36" / f"{case_id}.log"
        log_bytes = log_path.read_bytes()
        if sha256(log_bytes) != receipt["log"]["sha256"]:
            raise ValueError(f"immutable log SHA-256 mismatch: {case_id}")
        log_text = log_bytes.decode("utf-8", errors="strict")
        observation = observations.get(case_id)
        if status == "PASS" and observation is not None:
            raise ValueError(f"passing case must not have failed observation: {case_id}")
        target = observation["first_failed_target"] if observation is not None else None
        present, enumerated = target_is_present(log_text, target) if target is not None else (False, 0)
        first_loss = classify(status, present, observation, log_text)
        counts[first_loss] = counts.get(first_loss, 0) + 1
        cases.append(
            {
                "id": case_id,
                "baseline_status": status,
                "first_failed_target": target,
                "target_observed_in_candidate_lattice": present if target is not None else None,
                "enumerated_candidate_occurrences_before_classification": enumerated,
                "first_loss": first_loss,
                "log_sha256": receipt["log"]["sha256"],
            }
        )

    if statuses != {"FAIL": spec["expected_fail_count"], "PASS": spec["expected_pass_count"]}:
        raise ValueError(f"immutable status count mismatch: {statuses}")
    if sum(counts.values()) != spec["expected_case_count"]:
        raise ValueError("classified denominator is incomplete")

    result = {
        "schema": "lay.ime-target-authority-first-loss-manifest.v1",
        "verdict": "PASS_FROZEN_FIRST_LOSS_DENOMINATOR",
        "classifier_sha256": sha256(Path(__file__).read_bytes()),
        "input_sha256": sha256(spec_bytes),
        "run_summary_sha256": sha256(summary_bytes),
        "source_sha256": sha256(source_bytes),
        "case_count": len(cases),
        "counts": dict(sorted(counts.items())),
        "cases": cases,
    }
    serialized = json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True).encode() + b"\n"
    args.out.write_bytes(serialized)
    print(json.dumps({"verdict": result["verdict"], "counts": result["counts"]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
