#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== cargo fmt --all --check =="
cargo fmt --all --check

echo "== cargo test --all-targets =="
cargo test --all-targets

echo "== cargo clippy --all-targets -- -D warnings =="
cargo clippy --all-targets -- -D warnings

echo "== node --check GNOME extension =="
node --check extension/lay@radislabus-star.github.io/lay-impl.js
node --check extension/lay@radislabus-star.github.io/extension.js

echo "== python compile KDE tray =="
python3 -m py_compile scripts/lay-kde-tray.py
bash -n install.sh update.sh scripts/install-remote.sh

echo "== cargo build --release --bins =="
cargo build --release --bins

echo "== cargo run --quiet --bin lay-ngram-corpus -- check-cache =="
cargo run --quiet --bin lay-ngram-corpus -- check-cache

echo "== cargo run --quiet --bin lay-lem-research =="
cargo run --quiet --bin lay-lem-research

echo "== git diff --check =="
git diff --check

if [[ "${LAY_RUNTIME_SMOKE:-0}" == "1" ]]; then
  echo "== scripts/run_runtime_smoke.py =="
  scripts/run_runtime_smoke.py
fi

echo "== lay full check OK =="
