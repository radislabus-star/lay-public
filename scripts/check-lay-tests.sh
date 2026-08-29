#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
action="${1:-all}"
shift || true

cd "$ROOT"

case "$action" in
  self-test)
    exec python3 -u tests/test_test_lanes.py
    ;;
  live)
    if [[ "${LAY_RUNTIME_SMOKE_MANAGED_DESKTOP:-0}" != "1" ]]; then
      echo "LAY_RUNTIME_SMOKE_MANAGED_DESKTOP=1 is required for live desktop mutation" >&2
      exit 1
    fi
    exec scripts/run_runtime_smoke.py \
      --managed-desktop \
      --ime-managed \
      --verify-ime-trace \
      "$@"
    ;;
  fetch)
    scripts/verify-rust-toolchain.sh lint >/dev/null
    exec scripts/cargo-guard.sh fetch --locked "$@"
    ;;
  manifest|write-manifest|correctness|package|all|performance)
    scripts/verify-rust-toolchain.sh lint >/dev/null
    exec python3 -u scripts/test-lanes.py "$action" "$@"
    ;;
  *)
    echo "usage: scripts/check-lay-tests.sh self-test|fetch|manifest|write-manifest|correctness|package|all|performance|live" >&2
    exit 2
    ;;
esac
