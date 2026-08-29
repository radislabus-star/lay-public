#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE="$ROOT/scripts/lint-baseline/dead_code.json"
MODE="${1:-check}"

case "$MODE" in
  check|--write-baseline) ;;
  --self-test)
    exec python3 "$ROOT/scripts/lint_inventory.py" self-test
    ;;
  *)
    echo "usage: scripts/check-lay-lints.sh [--self-test|--write-baseline]" >&2
    exit 2
    ;;
esac

cd "$ROOT"
scripts/verify-rust-toolchain.sh lint

stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

check_json="$stage/check.jsonl"
check_stderr="$stage/check.stderr"
if ! scripts/cargo-guard.sh check --locked --all-targets --message-format=json \
    >"$check_json" 2>"$check_stderr"; then
  cat "$check_stderr" >&2
  python3 scripts/lint_inventory.py clean --input "$check_json" || true
  exit 1
fi

if [[ "$MODE" == "--write-baseline" ]]; then
  candidate_baseline="$stage/dead_code.json"
  python3 scripts/lint_inventory.py inventory \
    --input "$check_json" --baseline "$BASELINE" --write-to "$candidate_baseline"
else
  python3 scripts/lint_inventory.py inventory \
    --input "$check_json" --baseline "$BASELINE"
fi

clippy_json="$stage/clippy.jsonl"
clippy_stderr="$stage/clippy.stderr"
if ! scripts/cargo-guard.sh clippy --locked --all-targets --message-format=json \
    -- -D warnings -A dead-code >"$clippy_json" 2>"$clippy_stderr"; then
  python3 scripts/lint_inventory.py clean --input "$clippy_json" || true
  cat "$clippy_stderr" >&2
  exit 1
fi
python3 scripts/lint_inventory.py clean --input "$clippy_json"

if [[ "$MODE" == "--write-baseline" ]]; then
  mkdir -p "$(dirname "$BASELINE")"
  mv "$candidate_baseline" "$BASELINE"
  echo "dead_code_baseline=COMMITTED"
fi

echo "lay_lint_contract=PASS"
