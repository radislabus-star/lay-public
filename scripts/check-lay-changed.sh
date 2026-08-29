#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo() {
  "$ROOT/scripts/cargo-guard.sh" "$@"
}

changed_files() {
  {
    git diff --name-only --diff-filter=ACMRTUXB --cached
    git diff --name-only --diff-filter=ACMRTUXB
    git ls-files --others --exclude-standard
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

if has_file_matching '(^install\.sh$|^update\.sh$|^uninstall\.sh$|^scripts/install-remote\.sh$|^scripts/test-public-issues\.sh$)'; then
  echo "== public install/update/uninstall issue regressions =="
  scripts/test-public-issues.sh
fi

if has_file_matching '^scripts/(install-l2-transition-phase-package|install-release-binaries|test-install-l2-transition-phase-package)\.sh$'; then
  echo "== L2 transition phase package install regressions =="
  scripts/test-install-l2-transition-phase-package.sh
fi

if has_file_matching '^scripts/(install-l11-shadow-package|test-install-l11-shadow-package)\.sh$'; then
  echo "== L1.1 package installer integrity regressions =="
  scripts/test-install-l11-shadow-package.sh
fi

if has_file_matching '(^install\.sh$|^scripts/(install-release-binaries|resolve-l2-package|l2-package-contract|test-install-release-binaries)\.sh$)'; then
  echo "== canonical L2 release package install regressions =="
  scripts/test-install-release-binaries.sh
fi

if has_file_matching '^extension/.*\.js$'; then
  echo "== node --check changed GNOME JS =="
  mapfile -t js_files < <(changed_list_for_ext ".js")
  for file in "${js_files[@]}"; do
    node --check "$file"
  done
fi

if has_file_matching '(^scripts/test_lanes/|^scripts/test-lanes/|^scripts/test-lanes\.py$|^scripts/check-lay-tests\.sh$|^tests/test_test_lanes\.py$)'; then
  echo "== hermetic test-lane self-check =="
  scripts/check-lay-tests.sh self-test
  scripts/check-lay-tests.sh manifest
fi

if has_file_matching '(^Cargo\.toml$|^Cargo\.lock$|^VERSIONING\.md$|^extension/)'; then
  echo "== lay version consistency =="
  python3 - <<'PY'
import json
import pathlib
import re
import sys

root = pathlib.Path(".")
cargo = (root / "Cargo.toml").read_text(encoding="utf-8")
match = re.search(r'^version = "([^"]+)"', cargo, re.M)
if not match:
    print("Cargo.toml package version not found", file=sys.stderr)
    sys.exit(1)
version = match.group(1)

metadata_path = root / "extension/lay@radislabus-star.github.io/metadata.json"
metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
metadata_version = metadata.get("version-name")
if metadata_version != version:
    print(f"metadata version-name drift: {metadata_version} != {version}", file=sys.stderr)
    sys.exit(1)

try:
    major, minor, patch = map(int, version.split("."))
except ValueError:
    print(f"version is not numeric semver: {version}", file=sys.stderr)
    sys.exit(1)
extension_version = patch if major == 0 else major * 1_000_000 + minor * 1_000 + patch
if metadata.get("version") != extension_version:
    print(f"metadata numeric version drift: {metadata.get('version')} != {extension_version}", file=sys.stderr)
    sys.exit(1)

version_decl = re.compile(r"^(?:export\s+)?const\s+APP_VERSION\s*=\s*['\"]([^'\"]+)['\"]\s*;", re.M)
date_decl = re.compile(r"^(?:export\s+)?const\s+APP_RELEASE_DATE\s*=\s*['\"]([^'\"]+)['\"]\s*;", re.M)
release_dates = set()
for path in sorted((root / "extension/lay@radislabus-star.github.io").glob("*.js")):
    text = path.read_text(encoding="utf-8")
    for app_version in version_decl.findall(text):
        if app_version != version:
            print(f"{path}: APP_VERSION drift: {app_version} != {version}", file=sys.stderr)
            sys.exit(1)
    release_dates.update(date_decl.findall(text))
if len(release_dates) > 1:
    print(f"APP_RELEASE_DATE drift: {sorted(release_dates)}", file=sys.stderr)
    sys.exit(1)
PY
fi

if ! has_file_matching '(^src/|^tests/|^data/|^Cargo\.toml$|^Cargo\.lock$)'; then
  echo "== no Rust/data/test changes; changed check OK =="
  exit 0
fi

echo "== cargo fmt --check =="
cargo fmt --check

echo "== hermetic Rust correctness and package lanes =="
scripts/check-lay-tests.sh all
if [[ "${LAY_CHANGED_PERFORMANCE:-0}" == "1" ]]; then
  echo "== serialized Rust performance lane =="
  scripts/check-lay-tests.sh performance
fi

echo "== cargo check --lib --bins =="
cargo check --lib --bins

recent_actions="${XDG_DATA_HOME:-$HOME/.local/share}/lay/recent_actions.jsonl"
if [[ -s "$recent_actions" ]]; then
  echo "== transition replay release gate =="
  cargo run --quiet --bin lay-debug-actions -- --transition-replay

  echo "== unsafe edit release gate =="
  cargo run --quiet --bin lay-debug-actions -- --unsafe-gate
else
  echo "== transition replay release gate skipped: no recent_actions.jsonl =="
fi

if [[ "${LAY_CHANGED_CLIPPY:-0}" == "1" ]]; then
  echo "== Lay lint contract =="
  scripts/check-lay-lints.sh
fi

if [[ "${LAY_CHANGED_RELEASE:-0}" == "1" ]]; then
  echo "== cargo build --release --bins =="
  cargo build --release --bins
fi

echo "== lay changed check OK =="
