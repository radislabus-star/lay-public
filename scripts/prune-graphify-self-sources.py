#!/usr/bin/env python3
"""Remove Graphify's generated output corpus from the tracked source graph.

Graphify query memory is useful local state, but it is a consumer of the source
graph rather than project source. Incremental Graphify releases can retain old
self-sourced nodes after the path becomes ignored. This helper removes only
communities made entirely from ``graphify-out/`` sources and fails closed if a
self-sourced node is mixed with project nodes.
"""

from __future__ import annotations

import argparse
import json
import re
import tempfile
from pathlib import Path


SELF_PREFIX = "graphify-out/"
COMMUNITY_HEADING = re.compile(r"^### Community (\d+) -")


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value, *, ensure_ascii: bool = True, newline: bool = False):
    text = json.dumps(value, indent=2, ensure_ascii=ensure_ascii)
    path.write_text(text + ("\n" if newline else ""), encoding="utf-8")


def prune_report(text: str, community_ids: set[int], labels: set[str]) -> str:
    output: list[str] = []
    skipping = False
    for line in text.splitlines(keepends=True):
        heading = COMMUNITY_HEADING.match(line)
        if heading:
            skipping = int(heading.group(1)) in community_ids
        elif skipping and line.startswith("## "):
            skipping = False
        if skipping:
            continue
        if line.rstrip("\n") in {f"- {label}" for label in labels}:
            continue
        output.append(line)
    result = "".join(output)
    for community_id in community_ids:
        if f"### Community {community_id} -" in result:
            raise ValueError(f"community {community_id} survived report pruning")
    for label in labels:
        if label.startswith("Q:") and label in result:
            raise ValueError(f"self-sourced query label survived report pruning: {label}")
    return result


def prune(root: Path, *, check_only: bool = False) -> dict[str, int]:
    out = root / "graphify-out"
    graph_path = out / "graph.json"
    manifest_path = out / "manifest.json"
    labels_path = out / ".graphify_labels.json"
    report_path = out / "GRAPH_REPORT.md"

    graph = read_json(graph_path)
    nodes = graph.get("nodes", [])
    bad_nodes = [
        node
        for node in nodes
        if str(node.get("source_file", "")).startswith(SELF_PREFIX)
    ]
    bad_ids = {node["id"] for node in bad_nodes}
    bad_communities = {
        int(node["community"])
        for node in bad_nodes
        if node.get("community") is not None
    }
    project_communities = {
        int(node["community"])
        for node in nodes
        if node.get("id") not in bad_ids and node.get("community") is not None
    }
    mixed = bad_communities & project_communities
    if mixed:
        raise ValueError(
            "refusing to prune mixed Graphify communities: "
            + ",".join(str(value) for value in sorted(mixed))
        )

    manifest = read_json(manifest_path)
    if not isinstance(manifest, dict):
        raise ValueError("unexpected Graphify manifest shape")
    bad_manifest = [key for key in manifest if key.startswith(SELF_PREFIX)]

    if check_only:
        if bad_nodes or bad_manifest:
            raise ValueError(
                f"Graphify corpus boundary failed: nodes={len(bad_nodes)} "
                f"manifest_paths={len(bad_manifest)}"
            )
        return {
            "nodes": 0,
            "links": 0,
            "manifest_paths": 0,
            "communities": 0,
        }

    labels = read_json(labels_path)
    removed_labels = {
        labels[str(community_id)]
        for community_id in bad_communities
        if str(community_id) in labels
    }

    graph["nodes"] = [node for node in nodes if node.get("id") not in bad_ids]
    links = graph.get("links", [])
    removed_links = [
        link
        for link in links
        if link.get("source") in bad_ids or link.get("target") in bad_ids
    ]
    graph["links"] = [
        link
        for link in links
        if link.get("source") not in bad_ids and link.get("target") not in bad_ids
    ]
    graph["hyperedges"] = [
        edge
        for edge in graph.get("hyperedges", [])
        if not any(node_id in json.dumps(edge, ensure_ascii=False) for node_id in bad_ids)
    ]
    for key in bad_manifest:
        manifest.pop(key)
    for community_id in bad_communities:
        labels.pop(str(community_id), None)

    if bad_nodes or bad_manifest:
        write_json(graph_path, graph)
        write_json(manifest_path, manifest, newline=True)
        write_json(labels_path, labels, ensure_ascii=False, newline=True)
        report_path.write_text(
            prune_report(
                report_path.read_text(encoding="utf-8"),
                bad_communities,
                removed_labels,
            ),
            encoding="utf-8",
        )

    result = {
        "nodes": len(bad_nodes),
        "links": len(removed_links),
        "manifest_paths": len(bad_manifest),
        "communities": len(bad_communities),
    }
    prune(root, check_only=True)
    return result


def self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="lay-graphify-prune-") as directory:
        root = Path(directory)
        out = root / "graphify-out"
        out.mkdir()
        write_json(
            out / "graph.json",
            {
                "nodes": [
                    {"id": "project", "source_file": "src/lib.rs", "community": 1},
                    {"id": "query", "source_file": "graphify-out/memory/q.md", "community": 2},
                ],
                "links": [{"source": "query", "target": "query"}],
                "hyperedges": [],
            },
        )
        write_json(
            out / "manifest.json",
            {"src/lib.rs": {}, "graphify-out/memory/q.md": {}},
            newline=True,
        )
        write_json(out / ".graphify_labels.json", {"1": "Project", "2": "Q: local"}, newline=True)
        (out / "GRAPH_REPORT.md").write_text(
            "## God Nodes\n\n- Project\n- Q: local\n\n"
            "### Community 1 - Project\nCohesion: 1\n\n"
            "### Community 2 - Q: local\nCohesion: 1\nNodes (1): Q: local\n\n",
            encoding="utf-8",
        )
        result = prune(root)
        assert result == {"nodes": 1, "links": 1, "manifest_paths": 1, "communities": 1}
        prune(root, check_only=True)
        report = (out / "GRAPH_REPORT.md").read_text(encoding="utf-8")
        assert "Q: local" not in report
    print("graphify self-source prune self-test: PASS")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    result = prune(args.root.resolve(), check_only=args.check)
    print(
        "graphify_self_sources="
        f"nodes:{result['nodes']} links:{result['links']} "
        f"manifest_paths:{result['manifest_paths']} communities:{result['communities']}"
    )


if __name__ == "__main__":
    main()
