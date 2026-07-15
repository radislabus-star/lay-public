#!/usr/bin/env python3
"""Build and verify Lay's architecture receipt from the Graphify AST graph."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "graphify-out" / "graph.json"
MANIFEST_PATH = ROOT / "graphify-out" / "manifest.json"
RECEIPT_PATH = (
    ROOT / "src" / "generated" / "architecture_graph_receipt.json"
)
SCHEMA = "lay.architecture-graph-receipt.v1"

RECEIPT_INPUTS = (
    ROOT / "Cargo.toml",
    ROOT / "docs" / "phase-word-recovery-canonical-cutover.md",
    Path(__file__).resolve(),
)

PROTECTED_SINGLE_OWNER_SYMBOLS = {
    "mix64_avalanche()": "src/stable_hash.rs",
    "mix64_golden()": "src/stable_hash.rs",
    "phase_center_from_sum()": "src/nanda_wave/l2_candidate_phase.rs",
    "phase_vector_from_atoms()": "src/nanda_wave/l2_candidate_phase.rs",
    "split_last_alphabetic_token()": "src/word_reader.rs",
    "split_last_trimmed_ws_token()": "src/word_reader.rs",
    "split_last_ws_token()": "src/word_reader.rs",
}

REPORT_DUPLICATE_SYMBOLS = {
    "is_cyrillic()",
    "is_cyrillic_char()",
    "is_cyrillic_word()",
    "normalize_l2_surface_word()",
    "normalize_surface()",
    "normalize_word()",
    "normalized_edit_distance()",
    "phase_coherence()",
}


def relevant_files() -> list[Path]:
    files = list(RECEIPT_INPUTS)
    for directory in (ROOT / "src", ROOT / "tests"):
        files.extend(path for path in directory.rglob("*.rs") if path.is_file())
    return sorted(set(files), key=lambda path: path.relative_to(ROOT).as_posix())


def source_fingerprint() -> str:
    digest = hashlib.sha256()
    for path in relevant_files():
        relative = path.relative_to(ROOT).as_posix().encode()
        digest.update(relative)
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"cannot read {path.relative_to(ROOT)}: {error}") from error


def git_head() -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()


def graph_freshness_violations(manifest: dict[str, Any]) -> list[str]:
    violations: list[str] = []
    for path in sorted((ROOT / "src").rglob("*.rs")):
        relative = path.relative_to(ROOT).as_posix()
        entry = manifest.get(relative)
        if entry is None:
            violations.append(f"graph_missing_source:{relative}")
            continue
        recorded_mtime = entry.get("mtime")
        if not isinstance(recorded_mtime, (int, float)):
            violations.append(f"graph_missing_mtime:{relative}")
            continue
        if abs(path.stat().st_mtime - float(recorded_mtime)) > 0.001:
            violations.append(f"graph_stale_source:{relative}")
    return violations


class ArchitectureGraph:
    def __init__(self, payload: dict[str, Any]) -> None:
        self.payload = payload
        self.nodes = payload.get("nodes", [])
        self.links = payload.get("links", [])
        self.nodes_by_id = {node.get("id"): node for node in self.nodes}
        self.nodes_by_label: dict[str, list[dict[str, Any]]] = defaultdict(list)
        for node in self.nodes:
            self.nodes_by_label[str(node.get("label", ""))].append(node)

    def production_nodes(self, label: str) -> list[dict[str, Any]]:
        return [
            node
            for node in self.nodes_by_label.get(label, [])
            if is_production_source(str(node.get("source_file", "")))
        ]

    def node(self, label: str, owner: str) -> tuple[list[str], list[str]]:
        nodes = self.production_nodes(label)
        evidence = [node_ref(node) for node in nodes]
        violations: list[str] = []
        if len(nodes) != 1:
            violations.append(f"owner_count:{label}:{len(nodes)}")
        elif nodes[0].get("source_file") != owner:
            violations.append(
                f"wrong_owner:{label}:{nodes[0].get('source_file')}!=${owner}".replace("$", "")
            )
        return evidence, violations

    def source_imports(self, prefix: str, forbidden_target_fragments: tuple[str, ...]) -> list[str]:
        violations: list[str] = []
        for edge in self.links:
            if edge.get("relation") not in {"imports", "imports_from"}:
                continue
            source_file = str(edge.get("source_file", ""))
            target = str(edge.get("target", ""))
            if source_file.startswith(prefix) and any(
                fragment in target for fragment in forbidden_target_fragments
            ):
                violations.append(
                    f"forbidden_import:{source_file}:{edge.get('source_location')}:{target}"
                )
        return sorted(set(violations))

    def references_type(self, function_label: str, owner: str, type_id: str) -> bool:
        function_nodes = [
            node
            for node in self.production_nodes(function_label)
            if node.get("source_file") == owner
        ]
        if len(function_nodes) != 1:
            return False
        function_id = function_nodes[0].get("id")
        return any(
            edge.get("source") == function_id
            and edge.get("target") == type_id
            and edge.get("relation") == "references"
            and edge.get("context") == "parameter_type"
            for edge in self.links
        )

    def callers(self, target_id: str) -> list[dict[str, Any]]:
        return [
            edge
            for edge in self.links
            if edge.get("target") == target_id and edge.get("relation") == "calls"
        ]


def is_production_source(path: str) -> bool:
    return path.startswith("src/") and not (
        "/tests/" in path
        or path.endswith("_tests.rs")
        or path.endswith("/tests.rs")
        or path.startswith("src/bin/lay_test_input")
        or path.startswith("src/bin/lay_lem_research")
        or path.startswith("src/bin/lay_nanda_wave_eval")
    )


def node_ref(node: dict[str, Any]) -> str:
    return f"{node.get('source_file')}:{node.get('source_location')}:{node.get('label')}"


def check(check_id: str, evidence: list[str], violations: list[str]) -> dict[str, Any]:
    return {
        "id": check_id,
        "status": "PASS" if not violations else "WATCH",
        "evidence": sorted(set(evidence)),
        "violations": sorted(set(violations)),
    }


def build_receipt() -> dict[str, Any]:
    graph_payload = load_json(GRAPH_PATH)
    manifest = load_json(MANIFEST_PATH)
    graph = ArchitectureGraph(graph_payload)
    checks: list[dict[str, Any]] = []

    evidence: list[str] = []
    violations: list[str] = []
    for label, owner in (
        ("TransitionDecisionCore", "src/typing_transition/decision.rs"),
        (".select_apply_candidate()", "src/typing_transition/decision.rs"),
        (".decide_visible_text_transition()", "src/typing_transition/decision.rs"),
        ("L2CandidateLattice", "src/typing_transition/candidate.rs"),
    ):
        item_evidence, item_violations = graph.node(label, owner)
        evidence.extend(item_evidence)
        violations.extend(item_violations)
    violations.extend(
        graph.source_imports("src/nanda_wave", ("typing_transition_decision", "src_text_edit"))
    )
    checks.append(check("decision-authority", evidence, violations))

    ime_forbidden = graph.source_imports(
        "src/bin/lay_ibus_engine",
        (
            "src_correction_bayes",
            "src_nanda_wave_l2",
            "src_nanda_wave_l3",
            "src_nanda_wave_usage_prior",
            "src_typing_transition_decision",
        ),
    )
    checks.append(
        check(
            "ime-backend-only",
            [
                "src/bin/lay_ibus_engine/preedit.rs -> src/ime_candidate_readout.rs",
                "src/bin/lay_ibus_engine/composition_commit.rs -> src/ime_correction.rs",
            ],
            ime_forbidden,
        )
    )

    evidence = []
    violations = []
    for label, owner in (
        ("AuthorizedEdit", "src/text_edit/executor.rs"),
        ("authorize_backend_edit()", "src/text_edit/executor.rs"),
    ):
        item_evidence, item_violations = graph.node(label, owner)
        evidence.extend(item_evidence)
        violations.extend(item_violations)
    authorized_type_id = "src_text_edit_executor_authorizededit"
    for label, owner in (
        ("apply_text_replacement_pipeline()", "src/bin/lay_daemon/text_output/replacement.rs"),
        ("call_replace_text()", "src/bin/lay_daemon/layout_controller.rs"),
        ("try_ime_replace_tail()", "src/bin/lay_daemon/layout_controller.rs"),
    ):
        matching = [
            node
            for node in graph.production_nodes(label)
            if node.get("source_file") == owner
        ]
        evidence.extend(node_ref(node) for node in matching)
        if len(matching) != 1:
            violations.append(f"mutation_sink_count:{label}:{owner}:{len(matching)}")
        elif not graph.references_type(label, owner, authorized_type_id):
            violations.append(f"mutation_sink_without_capability:{label}:{owner}")
    checks.append(check("edit-plan-verifier", evidence, violations))

    executor_source = (ROOT / "src/text_edit/executor.rs").read_text(encoding="utf-8")
    constructor_sites: list[str] = []
    for path in (ROOT / "src").rglob("*.rs"):
        relative = path.relative_to(ROOT).as_posix()
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if "AuthorizedEdit {" in line and "struct AuthorizedEdit" not in line and "impl AuthorizedEdit" not in line:
                constructor_sites.append(f"{relative}:L{line_number}")
    capability_violations: list[str] = []
    if len(constructor_sites) != 1 or not constructor_sites[0].startswith(
        "src/text_edit/executor.rs:"
    ):
        capability_violations.append(
            "authorized_edit_constructor_sites:" + ",".join(constructor_sites)
        )
    if "authorized: Option<AuthorizedEdit>" not in executor_source:
        capability_violations.append("sealed_capability_storage_missing")
    checks.append(
        check(
            "typed-transition-capability",
            constructor_sites,
            capability_violations,
        )
    )

    hot_evidence: list[str] = []
    hot_violations: list[str] = []
    for label, owner in (
        ("LexicalPhaseMemory", "src/nanda_wave/lexical_phase/runtime.rs"),
        ("L2CandidateLattice", "src/typing_transition/candidate.rs"),
    ):
        item_evidence, item_violations = graph.node(label, owner)
        hot_evidence.extend(item_evidence)
        hot_violations.extend(item_violations)
    for relative in (
        "src/nanda_wave/lexical_phase/runtime.rs",
        "src/nanda_wave/l2/hot_memory.rs",
    ):
        source = (ROOT / relative).read_text(encoding="utf-8")
        for forbidden in (r"HashSet\s*<\s*String\s*>", r"Vec\s*<\s*String\s*>"):
            if re.search(forbidden, source):
                hot_violations.append(f"hot_full_word_authority:{relative}:{forbidden}")
    checks.append(check("hot-field-memory", hot_evidence, hot_violations))

    l2_violations = graph.source_imports(
        "src/nanda_wave", ("typing_transition_decision", "src_text_edit_executor")
    )
    checks.append(
        check(
            "l2-candidate-field",
            ["src/nanda_wave/l2.rs", "src/typing_transition/candidate.rs"],
            l2_violations,
        )
    )

    learning_evidence: list[str] = []
    learning_violations: list[str] = []
    for label, owner in (
        ("L4StateEstimator", "src/typing_transition/l4_state_estimator.rs"),
        ("L4SignedMemorySignal", "src/nanda_wave/l4_signed_memory.rs"),
    ):
        item_evidence, item_violations = graph.node(label, owner)
        learning_evidence.extend(item_evidence)
        learning_violations.extend(item_violations)
    checks.append(check("l3-l4-learning", learning_evidence, learning_violations))

    duplicate_symbols: dict[str, list[str]] = {}
    for label in sorted(REPORT_DUPLICATE_SYMBOLS | set(PROTECTED_SINGLE_OWNER_SYMBOLS)):
        refs = [node_ref(node) for node in graph.production_nodes(label)]
        if len(refs) > 1:
            duplicate_symbols[label] = refs
    graph_violations = graph_freshness_violations(manifest)
    for label, owner in PROTECTED_SINGLE_OWNER_SYMBOLS.items():
        nodes = graph.production_nodes(label)
        if len(nodes) != 1 or nodes[0].get("source_file") != owner:
            graph_violations.append(f"duplicate_or_wrong_owner:{label}:{len(nodes)}")
    checks.append(
        check(
            "fast-verifiable",
            [
                f"graph_nodes:{len(graph.nodes)}",
                f"graph_links:{len(graph.links)}",
                f"graph_built_at_commit:{graph_payload.get('built_at_commit', '')}",
            ],
            graph_violations,
        )
    )

    verdict = "PASS" if all(item["status"] == "PASS" for item in checks) else "WATCH"
    return {
        "schema": SCHEMA,
        "verdict": verdict,
        "source_fingerprint": source_fingerprint(),
        "git_head": git_head(),
        "graph_built_at_commit": graph_payload.get("built_at_commit", ""),
        "graph_nodes": len(graph.nodes),
        "graph_links": len(graph.links),
        "checks": checks,
        "duplicate_symbols": duplicate_symbols,
    }


def canonical_json(payload: dict[str, Any]) -> str:
    return json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-receipt", action="store_true")
    parser.add_argument("--check-receipt", action="store_true")
    parser.add_argument("--format", choices=("text", "json"), default="text")
    args = parser.parse_args()

    try:
        receipt = build_receipt()
    except RuntimeError as error:
        print(f"architecture graph error: {error}", file=sys.stderr)
        return 2

    rendered = canonical_json(receipt)
    if args.write_receipt:
        RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
        RECEIPT_PATH.write_text(rendered, encoding="utf-8")

    if args.check_receipt:
        try:
            existing = RECEIPT_PATH.read_text(encoding="utf-8")
        except OSError as error:
            print(f"architecture receipt missing: {error}", file=sys.stderr)
            return 2
        if existing != rendered:
            print(
                "architecture receipt is stale; run graphify update . and "
                "scripts/architecture_graph_gate.py --write-receipt",
                file=sys.stderr,
            )
            return 1

    if args.format == "json":
        sys.stdout.write(rendered)
    else:
        print(f"architecture_graph_verdict={receipt['verdict']}")
        for item in receipt["checks"]:
            print(
                f"{item['id']}={item['status']} "
                f"violations={len(item['violations'])}"
            )
            for violation in item["violations"]:
                print(f"  {violation}")
        for label, refs in receipt["duplicate_symbols"].items():
            print(f"duplicate_symbol={label} count={len(refs)}")

    return 0 if receipt["verdict"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
