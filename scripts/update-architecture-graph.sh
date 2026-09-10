#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

graphify update .
python3 scripts/prune-graphify-self-sources.py
python3 scripts/prune-graphify-self-sources.py --check
python3 scripts/architecture_graph_gate.py --write-graph-binding
python3 scripts/architecture_graph_gate.py --write-receipt
scripts/check-architecture.sh
