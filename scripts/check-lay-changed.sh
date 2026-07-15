#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

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

if has_file_matching '^extension/.*\.js$'; then
  echo "== node --check changed GNOME JS =="
  mapfile -t js_files < <(changed_list_for_ext ".js")
  for file in "${js_files[@]}"; do
    node --check "$file"
  done
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
    patch = int(version.rsplit(".", 1)[1])
except (IndexError, ValueError):
    print(f"version has no numeric patch: {version}", file=sys.stderr)
    sys.exit(1)
if metadata.get("version") != patch:
    print(f"metadata numeric version drift: {metadata.get('version')} != {patch}", file=sys.stderr)
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

if has_file_matching '^src/bin/lay_ibus_engine'; then
  if [[ "${LAY_CHANGED_FULL_IME:-0}" == "1" ]]; then
    echo "== cargo test --bin lay-ibus-engine =="
    cargo test --bin lay-ibus-engine
  else
    echo "== cargo test --bin lay-ibus-engine targeted =="
    cargo test --bin lay-ibus-engine live_ime_
    cargo test --bin lay-ibus-engine known_russian_word_does_not_get_extended_by_precognition
    cargo test --bin lay-ibus-engine short_russian_prefix_stays_fast_without_dropping_valid_candidates
    cargo test --bin lay-ibus-engine manual_toggle_
    cargo test --bin lay-ibus-engine committed_tail
    cargo test --bin lay-ibus-engine daemon_bridge
    cargo test --bin lay-ibus-engine delete_profile
    cargo test --bin lay-ibus-engine handoff
    cargo test --bin lay-ibus-engine reset
  fi
  echo "== isolated IME latency budget =="
  LAY_ENFORCE_IME_LATENCY_BUDGET=1 \
    cargo test --bin lay-ibus-engine cold_english_wave_memory_does_not_block_precognition \
      -- --nocapture --test-threads=1
  LAY_ENFORCE_IME_LATENCY_BUDGET=1 \
    cargo test --bin lay-ibus-engine precognition_candidate_generation_stays_under_budget \
      -- --nocapture --test-threads=1
fi

if has_file_matching '^src/bin/lay_daemon'; then
  if [[ "${LAY_CHANGED_FULL_DAEMON:-0}" == "1" ]]; then
    echo "== cargo test --bin lay-daemon =="
    cargo test --bin lay-daemon
  else
    echo "== cargo test --bin lay-daemon targeted =="
    cargo test --bin lay-daemon text_output_contract
    cargo test --bin lay-daemon enter_autocorrect
    cargo test --bin lay-daemon runtime_state::typing_assist
    cargo test --bin lay-daemon layout_switch_policy
  fi
fi

if has_file_matching '^src/bin/lay_debug_actions\.rs$'; then
  echo "== cargo test --bin lay-debug-actions =="
  cargo test --bin lay-debug-actions
fi

if has_file_matching '^src/correction_core(\.rs|/)'; then
  if [[ "${LAY_CHANGED_FULL_CORE:-0}" == "1" ]]; then
    echo "== cargo test correction_core:: --lib =="
    cargo test correction_core:: --lib
  else
    echo "== correction-core route contracts =="
    cargo test --test typing_transition_authority_contract
    cargo test --test text_mutation_monopoly_contract
  fi

  echo "== cargo test input_gate:: --lib =="
  cargo test input_gate:: --lib
elif has_file_matching '^src/input_gate\.rs$'; then
  echo "== cargo test input_gate:: --lib =="
  cargo test input_gate:: --lib
fi

if has_file_matching '^src/phrase_reader'; then
  echo "== cargo test phrase_reader:: --lib =="
  cargo test phrase_reader:: --lib
fi

if has_file_matching '^src/ru_typo'; then
  echo "== cargo test ru_typo:: --lib =="
  cargo test ru_typo:: --lib
fi

if has_file_matching '^src/nanda_wave/(l3|l3_phrase_gate)\.rs$'; then
  echo "== cargo test nanda_wave::l3:: --lib =="
  cargo test nanda_wave::l3:: --lib

  echo "== cargo test nanda_wave::l3_phrase_gate --lib =="
  cargo test nanda_wave::l3_phrase_gate --lib
elif has_file_matching '^src/nanda_wave'; then
  if [[ "${LAY_CHANGED_FULL_L2:-0}" == "1" ]]; then
    echo "== cargo test nanda_wave:: --lib =="
    cargo test nanda_wave:: --lib
  else
    echo "== nanda-wave route contracts =="
    cargo test --test typing_transition_authority_contract
    cargo test --test text_mutation_monopoly_contract
  fi
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
  echo "== cargo clippy --all-targets -- -D warnings =="
  cargo clippy --all-targets -- -D warnings
fi

if [[ "${LAY_CHANGED_RELEASE:-0}" == "1" ]]; then
  echo "== cargo build --release --bins =="
  cargo build --release --bins
fi

echo "== lay changed check OK =="
