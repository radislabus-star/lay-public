"""Exact manifest and known-failure contracts for test lanes."""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
from typing import Any

from .discovery import SCHEMA, canonical


KNOWN_SCHEMA = "lay.test-known-failures.v1"
KNOWN_CLUSTERS = {
    "architecture_integration",
    "candidate_birth",
    "correction_ranking_admission",
    "edit_safety_contract",
    "ime_authority",
    "nanda_l2_field",
    "nanda_l3_context",
    "phrase_boundary",
    "remaining_semantic",
    "typing_assist_surface",
}
PANIC_LOCATION = re.compile(
    r"^thread '.+'(?: \(\d+\))? panicked at .+?:\d+:\d+:(?:\s*(.*))?$"
)
VOLATILE_JSON_TIMESTAMP = re.compile(r'"ts":\d+')
VOLATILE_EPISODE_ID = re.compile(r'"episode_id":"[^"]+"')


class ContractError(RuntimeError):
    pass


def canonical_json(payload: Any) -> str:
    return json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n"


def load_json(path: pathlib.Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot read {path}: {error}") from error
    if not isinstance(payload, dict):
        raise ContractError(f"{path}: root must be an object")
    return payload


def file_sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalized_failure_block(block: str) -> str:
    """Remove harness/source-location noise while retaining the exact failure."""
    lines = []
    for line in block.splitlines():
        match = PANIC_LOCATION.match(line)
        if match:
            if match.group(1):
                lines.append(match.group(1).rstrip())
            continue
        if line.startswith("note: run with `RUST_BACKTRACE="):
            continue
        line = VOLATILE_JSON_TIMESTAMP.sub('"ts":<volatile>', line)
        line = VOLATILE_EPISODE_ID.sub('"episode_id":"<volatile>"', line)
        lines.append(line.rstrip())
    while lines and not lines[0]:
        lines.pop(0)
    while lines and not lines[-1]:
        lines.pop()
    return "\n".join(lines)


def failure_signature(block: str) -> str:
    normalized = normalized_failure_block(block)
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def failure_excerpt(block: str) -> str:
    normalized = normalized_failure_block(block)
    excerpt = " | ".join(line.strip() for line in normalized.splitlines()[:3])[:320]
    return excerpt or "<empty normalized failure block>"


def write_json(path: pathlib.Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(canonical_json(payload), encoding="utf-8")


def validate_manifest_shape(payload: dict[str, Any]) -> None:
    if payload.get("schema") != SCHEMA:
        raise ContractError(f"unexpected test manifest schema: {payload.get('schema')!r}")
    tests = payload.get("tests")
    targets = payload.get("targets")
    if not isinstance(tests, list) or not isinstance(targets, list):
        raise ContractError("test manifest must contain tests and targets lists")
    keys = [canonical(row) for row in tests]
    if keys != sorted(set(keys)):
        raise ContractError("test manifest rows must be unique and canonically sorted")
    identities = [(row.get("target"), row.get("name")) for row in tests]
    if len(identities) != len(set(identities)):
        raise ContractError("test manifest contains duplicate target/test identities")
    accepted = {"correctness", "package", "performance", "ignored"}
    if any(row.get("lane") not in accepted for row in tests):
        raise ContractError("test manifest contains an unknown lane")
    if any(row.get("isolation") not in {"process", "target"} for row in tests):
        raise ContractError("test manifest contains an unknown isolation mode")
    if any(
        row["lane"] == "performance" and row["isolation"] != "process"
        for row in tests
    ):
        raise ContractError("performance tests must be process-isolated")
    counts = {lane: 0 for lane in accepted}
    isolation_counts = {kind: 0 for kind in ("process", "target")}
    for row in tests:
        counts[row["lane"]] += 1
        isolation_counts[row["isolation"]] += 1
    if payload.get("counts") != counts:
        raise ContractError("test manifest counts do not match rows")
    if payload.get("isolation_counts") != isolation_counts:
        raise ContractError("test manifest isolation counts do not match rows")
    cargo_configuration = payload.get("cargo_configuration")
    if not isinstance(cargo_configuration, dict):
        raise ContractError("test manifest lacks Cargo configuration closure")
    if cargo_configuration.get("external") != "ABSENT":
        raise ContractError("external Cargo configuration is not closed")
    project_configs = cargo_configuration.get("project")
    if not isinstance(project_configs, list):
        raise ContractError("project Cargo configuration rows are invalid")
    project_paths = [row.get("path") for row in project_configs]
    if project_paths != sorted(set(project_paths)):
        raise ContractError("project Cargo configuration rows are not path-sorted")
    for row in project_configs:
        if not isinstance(row.get("size"), int) or row["size"] < 0:
            raise ContractError(f"invalid project Cargo config size: {row!r}")
        if not re.fullmatch(r"[0-9a-f]{64}", str(row.get("sha256", ""))):
            raise ContractError(f"invalid project Cargo config SHA-256: {row!r}")
    fixtures = payload.get("package_fixtures")
    if not isinstance(fixtures, list) or not fixtures:
        raise ContractError("test manifest must pin package fixtures")
    fixture_paths = [row.get("path") for row in fixtures]
    if fixture_paths != sorted(set(fixture_paths)):
        raise ContractError("package fixture rows must be unique and path-sorted")
    for row in fixtures:
        if not isinstance(row.get("size"), int) or row["size"] < 0:
            raise ContractError(f"invalid package fixture size: {row!r}")
        if not re.fullmatch(r"[0-9a-f]{64}", str(row.get("sha256", ""))):
            raise ContractError(f"invalid package fixture SHA-256: {row!r}")


def compare_manifest(current: dict[str, Any], expected: dict[str, Any]) -> None:
    validate_manifest_shape(current)
    validate_manifest_shape(expected)
    if canonical_json(current) == canonical_json(expected):
        return
    current_rows = {
        (row["target"], row["name"]): row for row in current["tests"]
    }
    expected_rows = {
        (row["target"], row["name"]): row for row in expected["tests"]
    }
    added = sorted(current_rows.keys() - expected_rows)
    removed = sorted(expected_rows.keys() - current_rows)
    changed = sorted(
        key
        for key in current_rows.keys() & expected_rows
        if current_rows[key] != expected_rows[key]
    )
    raise ContractError(
        "test manifest drift: "
        f"added={added[:20]} removed={removed[:20]} changed={changed[:20]}"
    )


def load_known_failures(path: pathlib.Path, manifest_path: pathlib.Path) -> dict[str, Any]:
    payload = load_json(path)
    if payload.get("schema") != KNOWN_SCHEMA:
        raise ContractError(f"unexpected known-failure schema: {payload.get('schema')!r}")
    if payload.get("manifest_sha256") != file_sha256(manifest_path):
        raise ContractError("known-failure manifest is bound to a different test manifest")
    rows = payload.get("failures")
    if not isinstance(rows, list):
        raise ContractError("known-failure rows must be a list")
    if payload.get("failure_count") != len(rows):
        raise ContractError("known-failure count does not match rows")
    if not re.fullmatch(r"[0-9a-f]{64}", str(payload.get("observation_sha256", ""))):
        raise ContractError("known-failure observation SHA-256 is missing or invalid")
    observation_relative = payload.get("observation_path")
    if not isinstance(observation_relative, str) or not observation_relative:
        raise ContractError("known-failure observation path is missing")
    observation_fragment = pathlib.PurePosixPath(observation_relative)
    if observation_fragment.is_absolute() or ".." in observation_fragment.parts:
        raise ContractError("known-failure observation path must stay inside the repository")
    repository = manifest_path.resolve().parents[2]
    observation_path = repository / observation_fragment
    try:
        observation_sha256 = file_sha256(observation_path)
    except OSError as error:
        raise ContractError(
            f"cannot read known-failure observation {observation_path}: {error}"
        ) from error
    if observation_sha256 != payload["observation_sha256"]:
        raise ContractError("known-failure observation bytes do not match SHA-256")
    identities = [(row.get("target"), row.get("test")) for row in rows]
    if identities != sorted(set(identities)):
        raise ContractError("known-failure rows must be unique and identity-sorted")
    manifest = load_json(manifest_path)
    validate_manifest_shape(manifest)
    manifest_lanes = {
        (row["target"], row["name"]): row["lane"] for row in manifest["tests"]
    }
    for row in rows:
        for field in (
            "target",
            "test",
            "lane",
            "cluster",
            "owner",
            "signature_sha256",
            "signature_excerpt",
        ):
            if not isinstance(row.get(field), str) or not row[field]:
                raise ContractError(f"known-failure row lacks {field}: {row!r}")
        identity = (row["target"], row["test"])
        if manifest_lanes.get(identity) != row["lane"]:
            raise ContractError(f"known-failure lane/test is absent or drifted: {identity}")
        if not re.fullmatch(r"[0-9a-f]{64}", row["signature_sha256"]):
            raise ContractError(f"invalid known-failure signature: {identity}")
        if row["owner"] != "TD-007" or row["cluster"] not in KNOWN_CLUSTERS:
            raise ContractError(f"invalid known-failure ownership: {identity}")
    observation = load_json(observation_path)
    if observation.get("schema") != "lay.test-failure-observation.v1":
        raise ContractError("known-failure observation schema is invalid")
    observation_rows = observation.get("failures")
    if not isinstance(observation_rows, list):
        raise ContractError("known-failure observation rows are missing")
    observed = {
        (row.get("target"), row.get("test")): row for row in observation_rows
    }
    if set(observed) != set(identities):
        raise ContractError("known-failure observation identities do not match ledger")
    known_by_id = {(row["target"], row["test"]): row for row in rows}
    for identity, row in observed.items():
        block = row.get("failure_block")
        if not isinstance(block, str):
            raise ContractError(f"known-failure observation block is invalid: {identity}")
        known_row = known_by_id[identity]
        if failure_signature(block) != known_row["signature_sha256"]:
            raise ContractError(f"known-failure observation signature drift: {identity}")
        if failure_excerpt(block) != known_row["signature_excerpt"]:
            raise ContractError(f"known-failure observation excerpt drift: {identity}")
    return payload


def compare_known_failures(
    observed: list[dict[str, str]], known: dict[str, Any]
) -> None:
    observed_by_id = {(row["target"], row["test"]): row for row in observed}
    known_by_id = {(row["target"], row["test"]): row for row in known["failures"]}
    unexpected = sorted(observed_by_id.keys() - known_by_id)
    fixed = sorted(known_by_id.keys() - observed_by_id)
    signature_mismatches = []
    for identity in sorted(observed_by_id.keys() & known_by_id):
        expected = known_by_id[identity]["signature_sha256"]
        actual = failure_signature(observed_by_id[identity].get("failure_block", ""))
        if expected != actual:
            signature_mismatches.append(identity)
    if unexpected or fixed or signature_mismatches:
        raise ContractError(
            "known-failure set drift: "
            f"unexpected={unexpected[:20]} fixed={fixed[:20]} "
            f"signature_mismatch={signature_mismatches[:20]}"
        )
