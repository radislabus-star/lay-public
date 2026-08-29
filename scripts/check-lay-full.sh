#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo() {
  "$ROOT/scripts/cargo-guard.sh" "$@"
}

echo "== cargo fmt --all --check =="
cargo fmt --all --check

echo "== scripts/check-architecture.sh =="
scripts/check-architecture.sh
if [[ "${LAY_AUDIT_50:-0}" == "1" ]]; then
  echo "== scripts/check-lay-audit-50.sh =="
  scripts/check-lay-audit-50.sh
fi

echo "== hermetic Rust test-lane contracts =="
scripts/check-lay-tests.sh self-test
scripts/check-lay-tests.sh fetch
scripts/check-lay-tests.sh all
if [[ "${LAY_CHECK_PERFORMANCE:-0}" == "1" ]]; then
  echo "== serialized Rust performance lane =="
  scripts/check-lay-tests.sh performance
else
  echo "== skip performance lane (set LAY_CHECK_PERFORMANCE=1) =="
fi

echo "== scripts/check-lay-lints.sh =="
scripts/check-lay-lints.sh

echo "== node --check GNOME extension =="
python3 -m json.tool extension/lay@radislabus-star.github.io/metadata.json >/dev/null
node --check extension/lay@radislabus-star.github.io/lay-impl.js
node --check extension/lay@radislabus-star.github.io/extension.js

echo "== python compile desktop helpers =="
python3 -m py_compile scripts/*.py
bash -n install.sh update.sh dev-reload.sh scripts/*.sh

echo "== CLI explain smoke =="
cargo run --quiet --bin lay -- --explain-correct 'кторое ' | grep -F 'confidence:' >/dev/null

echo "== cargo build --release --bins =="
cargo build --release --bins

if [[ "${LAY_CHECK_NGRAM:-0}" == "1" ]]; then
  NGRAM_CHECK_CACHE="${LAY_NGRAM_CHECK_CACHE:-${HOME:-}/.cache/lay/ngram_ru_v1.json}"
  if [[ -n "$NGRAM_CHECK_CACHE" && -f "$NGRAM_CHECK_CACHE" ]]; then
    echo "== cargo run --quiet --bin lay-ngram-corpus -- check-cache =="
    cargo run --quiet --bin lay-ngram-corpus -- check-cache --cache "$NGRAM_CHECK_CACHE"
  else
    echo "== cargo run --quiet --bin lay-ngram-corpus -- cache/check-cache target =="
    NGRAM_CHECK_CACHE="target/lay-full-ngram-ru.json"
    cargo run --quiet --bin lay-ngram-corpus -- cache --out "$NGRAM_CHECK_CACHE"
    cargo run --quiet --bin lay-ngram-corpus -- check-cache --cache "$NGRAM_CHECK_CACHE"
  fi
else
  echo "== skip ngram cache check (set LAY_CHECK_NGRAM=1) =="
fi

echo "== git diff --check =="
git diff --check

if [[ "${LAY_RUNTIME_SMOKE:-0}" == "1" ]]; then
  if [[ "${LAY_RUNTIME_SMOKE_MANAGED_DESKTOP:-0}" != "1" ]]; then
    echo "LAY_RUNTIME_SMOKE_MANAGED_DESKTOP=1 is required for live desktop mutation" >&2
    exit 1
  fi
  echo "== scripts/run_runtime_smoke.py =="
  scripts/run_runtime_smoke.py \
    --managed-desktop \
    --ime-managed \
    --verify-ime-trace
fi

echo "== lay full check OK =="
