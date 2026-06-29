#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

changed_files() {
  {
    git diff --name-only --diff-filter=ACMRTUXB --cached
    git diff --name-only --diff-filter=ACMRTUXB
  } | sed '/^$/d' | sort -u
}

mapfile -t files < <(changed_files)

if [[ "${#files[@]}" == "0" ]]; then
  echo "== no modified files =="
  exit 0
fi

has_file_matching() {
  local pattern="$1"
  local file
  for file in "${files[@]}"; do
    if [[ "$file" =~ $pattern ]]; then
      return 0
    fi
  done
  return 1
}

changed_list_for_ext() {
  local ext="$1"
  local file
  for file in "${files[@]}"; do
    [[ "$file" == *"$ext" ]] && printf '%s\n' "$file"
  done
}

echo "== changed files =="
printf '  %s\n' "${files[@]}"

echo "== git diff --check =="
git diff --check

mapfile -t shell_files < <(changed_list_for_ext ".sh")
if [[ "${#shell_files[@]}" -gt 0 ]]; then
  echo "== bash -n changed shell files =="
  bash -n "${shell_files[@]}"
fi

mapfile -t python_files < <(changed_list_for_ext ".py")
if [[ "${#python_files[@]}" -gt 0 ]]; then
  echo "== py_compile changed python files =="
  python3 -m py_compile "${python_files[@]}"
fi

if has_file_matching '^extension/.*\.json$'; then
  echo "== json check changed GNOME metadata =="
  python3 -m json.tool extension/lay@radislabus-star.github.io/metadata.json >/dev/null
fi

if has_file_matching '^extension/.*\.js$'; then
  echo "== node --check changed GNOME JS =="
  mapfile -t js_files < <(changed_list_for_ext ".js")
  for file in "${js_files[@]}"; do
    node --check "$file"
  done
fi

if ! has_file_matching '(^src/|^tests/|^data/|^Cargo\.toml$|^Cargo\.lock$)'; then
  echo "== no Rust/data/test changes; changed check OK =="
  exit 0
fi

echo "== cargo fmt --check =="
cargo fmt --check

if has_file_matching '^src/bin/lay_ibus_engine'; then
  echo "== cargo test --bin lay-ibus-engine =="
  cargo test --bin lay-ibus-engine
fi

if has_file_matching '^src/bin/lay_daemon'; then
  echo "== cargo test --bin lay-daemon =="
  cargo test --bin lay-daemon
fi

if has_file_matching '^src/correction_core\.rs$'; then
  echo "== cargo test correction_core:: --lib =="
  cargo test correction_core:: --lib
fi

if has_file_matching '^src/phrase_reader'; then
  echo "== cargo test phrase_reader:: --lib =="
  cargo test phrase_reader:: --lib
fi

if has_file_matching '^src/ru_typo'; then
  echo "== cargo test ru_typo:: --lib =="
  cargo test ru_typo:: --lib
fi

if has_file_matching '^tests/.*\.rs$'; then
  echo "== cargo test changed integration tests =="
  for file in "${files[@]}"; do
    if [[ "$file" =~ ^tests/([^/]+)\.rs$ ]]; then
      cargo test --test "${BASH_REMATCH[1]}"
    fi
  done
fi

if has_file_matching '^tests/fixtures/'; then
  echo "== cargo test typing assist fixture suites =="
  cargo test typing_assist_rules:: --bin lay-daemon
fi

echo "== cargo check --all-targets =="
cargo check --all-targets

if [[ "${LAY_CHANGED_CLIPPY:-0}" == "1" ]]; then
  echo "== cargo clippy --all-targets -- -D warnings =="
  cargo clippy --all-targets -- -D warnings
fi

if [[ "${LAY_CHANGED_RELEASE:-0}" == "1" ]]; then
  echo "== cargo build --release --bins =="
  cargo build --release --bins
fi

echo "== lay changed check OK =="
