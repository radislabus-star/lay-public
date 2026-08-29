#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

graphify update .
python3 scripts/architecture_graph_gate.py --write-graph-binding
