#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail=0
ARCH_TMP_DIR="${XDG_RUNTIME_DIR:-$ROOT/target/tmp}"
mkdir -p "$ARCH_TMP_DIR"
HIT_FILE="$(mktemp "$ARCH_TMP_DIR/lay-architecture-hit.XXXXXX")"
trap 'rm -f "$HIT_FILE"' EXIT

error() {
  printf 'architecture error: %s\n' "$*" >&2
  fail=1
}

search_fixed() {
  local pattern="$1"
  shift
  if command -v rg >/dev/null 2>&1; then
    rg -n --fixed-strings "$pattern" "$@"
  else
    grep -RInF -- "$pattern" "$@"
  fi
}

assert_no_import() {
  local file="$1"
  local pattern="$2"
  local reason="$3"
  if search_fixed "$pattern" "$file" >"$HIT_FILE"; then
    cat "$HIT_FILE" >&2
    error "$reason"
  fi
}

assert_single_owner() {
  local pattern="$1"
  local owner="$2"
  local hits
  hits="$(search_fixed "$pattern" src || true)"
  local count
  count="$(printf '%s\n' "$hits" | sed '/^$/d' | wc -l)"
  if [[ "$count" != "1" ]]; then
    printf '%s\n' "$hits" >&2
    error "'$pattern' must have exactly one owner: $owner"
    return
  fi
  if ! printf '%s\n' "$hits" | grep -F -- "$owner" >/dev/null; then
    printf '%s\n' "$hits" >&2
    error "'$pattern' owner must be $owner"
  fi
}

assert_max_lines() {
  local file="$1"
  local max="$2"
  local count
  count="$(wc -l < "$file")"
  if (( count > max )); then
    error "$file has $count lines; max is $max"
  fi
}

assert_no_runtime_example() {
  local pattern="$1"
  local hits
  if command -v rg >/dev/null 2>&1; then
    hits="$(rg -n --fixed-strings "$pattern" src \
        --glob '!src/**/*tests.rs' \
        --glob '!src/**/tests/**' \
        --glob '!src/*_tests.rs' \
        --glob '!src/bin/lay_test_input.rs' \
        --glob '!src/bin/lay_lem_research.rs' || true)"
  else
    hits="$(grep -RInF -- "$pattern" src || true)"
    hits="$(printf '%s\n' "$hits" \
      | grep -Ev '(^src/.*/tests/|_tests\.rs:|^src/bin/lay_daemon/tests\.rs:|^src/bin/lay_test_input\.rs:|^src/bin/lay_lem_research\.rs:)' || true)"
  fi
  if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits" >&2
    error "chat/log regression example '$pattern' must stay in tests or data, not runtime code"
  fi
}

assert_no_rust_fixture_phrase() {
  local pattern="$1"
  local hits
  if command -v rg >/dev/null 2>&1; then
    hits="$(rg -n --fixed-strings "$pattern" src tests --glob '*.rs' || true)"
  else
    hits="$(grep -RInF --include='*.rs' "$pattern" src tests || true)"
  fi
  if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits" >&2
    error "live regression phrase '$pattern' must stay in fixture data, not Rust code"
  fi
}

assert_no_rust_phrase_literal_in_file() {
  local file="$1"
  local hits
  if grep -nP '"[^"\n]*[А-Яа-яЁё][^"\n]*\S\s+\S[^"\n]*"' "$file" >"$HIT_FILE"; then
    hits="$(cat "$HIT_FILE")"
  else
    hits=""
  fi
  if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits" >&2
    error "$file must keep phrase fixtures in tests/fixtures, not inline Rust strings"
  fi
}

assert_max_lines src/bin/lay_daemon.rs 500
assert_max_lines src/typing_assist.rs 80
assert_max_lines src/llm.rs 750
assert_max_lines src/ru_typo.rs 900
assert_max_lines src/phrase_reader.rs 800
assert_max_lines src/scoped_tail.rs 600
assert_max_lines src/lem.rs 650
assert_max_lines src/bin/lay_daemon/daemon_runtime.rs 850
assert_max_lines src/bin/lay_daemon/trigger_dispatch.rs 200
assert_max_lines src/bin/lay_daemon/boundary_runtime.rs 260
assert_max_lines src/bin/lay_daemon/typing_key_runtime.rs 120
assert_max_lines src/bin/lay_daemon/layout_controller.rs 650
assert_max_lines src/bin/lay_daemon/layout_kde.rs 180
assert_max_lines src/bin/lay_daemon/layout_x11.rs 100
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen.rs 550
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope.rs 550
assert_max_lines extension/lay@radislabus-star.github.io/lay-impl.js 1800
assert_max_lines src/typing_candidate.rs 280
assert_max_lines src/text_edit.rs 320

assert_no_import src/scoped_tail.rs \
  "use crate::typing_assist" \
  "scoped_tail must use typing_pipeline/core helpers, not the public typing_assist facade"

assert_no_import src/ru_typo.rs \
  "use crate::phrase_reader" \
  "ru_typo must not depend on phrase_reader; shared phrase predicates belong in phrase_lexicon"

assert_no_import src/layout_autoswitch.rs \
  "use crate::phrase_reader" \
  "layout_autoswitch must not depend on phrase_reader; shared phrase predicates belong in phrase_lexicon"

assert_no_import src/phrase_lexicon.rs \
  "use crate::ru_typo" \
  "phrase_lexicon must stay below correction generators"

assert_no_import src/phrase_lexicon.rs \
  "use crate::layout_autoswitch" \
  "phrase_lexicon must stay below layout correction generators"

assert_no_import src/typing_pipeline.rs \
  "use crate::scoped_tail" \
  "typing_pipeline must not call smart scoped-tail; both are separate decision layers"

assert_no_import src/llm.rs \
  "ureq::" \
  "llm.rs must stay a deterministic arbiter facade; model transport belongs in llm_backend"

assert_no_import src/llm.rs \
  "llama_cpp" \
  "llm.rs must stay a deterministic arbiter facade; direct GGUF runtime belongs in llm_backend"

assert_no_import src/llm.rs \
  "serde::" \
  "llm.rs must stay a deterministic arbiter facade; model response structs belong in llm_backend"

assert_no_import src/lem.rs \
  "RU_HUNSPELL" \
  "lem.rs must use token_language for dictionaries, not direct Hunspell loading"

assert_no_import src/lem.rs \
  "EN_HUNSPELL" \
  "lem.rs must use token_language for dictionaries, not direct Hunspell loading"

assert_no_import src/lem.rs \
  "OnceLock" \
  "lem.rs must stay a scorer, not own hot dictionary caches"

assert_no_import src/lem.rs \
  "russian_lexicon" \
  "lem.rs must use token_language as its lexical boundary"

assert_no_import src/lem.rs \
  "read_to_string" \
  "lem.rs must not read dictionary files directly"

assert_single_owner "fn split_ws_segments" "src/word_reader.rs"
assert_single_owner "fn is_russian_vowel" "src/russian_chars.rs"
assert_single_owner "fn same_letter_ignore_case" "src/russian_chars.rs"
assert_single_owner "fn is_cli_option_token" "src/word_recognizer.rs"
assert_single_owner "fn is_protected_ascii_token" "src/word_recognizer.rs"
assert_single_owner "fn is_ascii_technical_token" "src/word_recognizer.rs"
assert_single_owner "fn is_ascii_technical_or_brand_token" "src/word_recognizer.rs"
assert_single_owner "fn is_upper_ascii_acronym" "src/word_recognizer.rs"
assert_single_owner "fn is_mixed_case_ascii_brand" "src/word_recognizer.rs"
assert_single_owner "fn is_mixed_cyrillic_ascii_alpha_token" "src/word_recognizer.rs"
assert_single_owner "fn is_ascii_layout_letter_symbol" "src/layout_autoswitch.rs"
assert_single_owner "fn is_ascii_shift_letter_symbol" "src/layout_autoswitch.rs"
assert_single_owner "fn has_cyrillic(text" "src/text_metrics.rs"
assert_single_owner "fn without_whitespace" "src/text_metrics.rs"
assert_single_owner "fn normalized_edit_distance" "src/text_metrics.rs"
assert_single_owner "fn common_replacement_span" "src/text_metrics.rs"
assert_single_owner "fn damerau_levenshtein" "src/text_metrics.rs"
assert_single_owner "fn apply_replacement_plan_to_text" "src/text_edit.rs"
assert_single_owner "fn committed_separator_is_preserved" "src/text_edit.rs"
assert_single_owner "fn rank_typing_candidates" "src/typing_candidate.rs"
assert_single_owner "fn classify_typing_confidence" "src/typing_candidate.rs"
assert_single_owner "fn plan_matches_replacement" "src/decoder.rs"

for example in \
  "тожесамое" \
  "тоесамое" \
  "янебуду" \
  "котовые" \
  "онаубыточная" \
  "какпроверка" \
  "прболематут" \
  "робило" \
  "банный" \
  "поения" \
  "перпаратов"
do
  assert_no_runtime_example "$example"
done

for phrase in \
  "djn cnhfpe" \
  "gthdthnsib" \
  "work ghjdthrf" \
  "<ELTV GBCFNM" \
  "RF:LJT CKJDJ"
do
  assert_no_rust_fixture_phrase "$phrase"
done

for cleaned_test_file in \
  src/bin/lay_daemon/tests.rs \
  src/bin/lay_daemon/tests/typing_assist_rules.rs \
  src/llm_tests.rs \
  src/phrase_reader_tests.rs \
  src/text_edit_tests.rs
do
  assert_no_rust_phrase_literal_in_file "$cleaned_test_file"
done

if command -v rg >/dev/null 2>&1; then
  second_best_hits="$(rg -n --fixed-strings "second_best" src --glob '!candidate_ranker.rs' || true)"
else
  second_best_hits="$(grep -RInF -- "second_best" src | grep -v '^src/candidate_ranker\.rs:' || true)"
fi
if [[ -n "$second_best_hits" ]]; then
  printf '%s\n' "$second_best_hits" >&2
  error "best/second-best arbitration must go through candidate_ranker"
fi

if command -v rg >/dev/null 2>&1; then
  private_file_hits="$(rg -n --fixed-strings "OpenOptionsExt" src --glob '!private_file.rs' || true)"
else
  private_file_hits="$(grep -RInF -- "OpenOptionsExt" src | grep -v '^src/private_file\.rs:' || true)"
fi
if [[ -n "$private_file_hits" ]]; then
  printf '%s\n' "$private_file_hits" >&2
  error "private file open mode must stay centralized in src/private_file.rs"
fi

permission_api_hits="$(search_fixed "PermissionsExt" src || true)"
permission_api_hits="$(printf '%s\n' "$permission_api_hits" \
  | grep -Ev '(^src/private_file\.rs:|_tests\.rs:|^src/.*/tests/)' || true)"
if [[ -n "$permission_api_hits" ]]; then
  printf '%s\n' "$permission_api_hits" >&2
  error "chmod-style permission changes must stay centralized in src/private_file.rs"
fi

set_permissions_hits="$(search_fixed "set_permissions" src || true)"
set_permissions_hits="$(printf '%s\n' "$set_permissions_hits" \
  | grep -Ev '(^src/private_file\.rs:|_tests\.rs:|^src/.*/tests/)' || true)"
if [[ -n "$set_permissions_hits" ]]; then
  printf '%s\n' "$set_permissions_hits" >&2
  error "private file chmod calls must go through src/private_file.rs"
fi

if command -v rg >/dev/null 2>&1; then
  sleep_hits="$(rg -n --fixed-strings "thread::sleep" src \
    --glob '!src/bin/lay_daemon/text_output.rs' \
    --glob '!src/bin/lay_daemon/layout_controller.rs' \
    --glob '!src/bin/lay_test_input.rs' || true)"
else
  sleep_hits="$(grep -RInF -- "thread::sleep" src \
    | grep -Ev '(^src/bin/lay_daemon/text_output\.rs:|^src/bin/lay_daemon/layout_controller\.rs:|^src/bin/lay_test_input\.rs:)' || true)"
fi
if [[ -n "$sleep_hits" ]]; then
  printf '%s\n' "$sleep_hits" >&2
  error "runtime delays must stay in text_output/layout_controller only; do not add sleep as correction logic"
fi

if command -v rg >/dev/null 2>&1; then
  tmp_lay_hits="$(rg -n --fixed-strings "/tmp/lay" . \
    --glob '!target/**' \
    --glob '!.git/**' \
    --glob '!scripts/check-architecture.sh' \
    --glob '!scripts/check-lay-audit-50.sh' \
    --glob '!scripts/lay-host-vm-guard.sh' || true)"
else
  tmp_lay_hits="$(grep -RInF \
    --exclude-dir=.git \
    --exclude-dir=target \
    --exclude='check-architecture.sh' \
    --exclude='check-lay-audit-50.sh' \
    --exclude='lay-host-vm-guard.sh' \
    -- "/tmp/lay" . || true)"
fi
if [[ -n "$tmp_lay_hits" ]]; then
  printf '%s\n' "$tmp_lay_hits" >&2
  error "lay runtime/install logs must use ~/.local/state/lay, not /tmp/lay-*"
fi

if [[ "$fail" != "0" ]]; then
  exit 1
fi

echo "lay architecture check OK"
