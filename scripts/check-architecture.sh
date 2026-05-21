#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail=0

error() {
  printf 'architecture error: %s\n' "$*" >&2
  fail=1
}

assert_no_import() {
  local file="$1"
  local pattern="$2"
  local reason="$3"
  if rg -n --fixed-strings "$pattern" "$file" >/tmp/lay-architecture-hit.txt; then
    cat /tmp/lay-architecture-hit.txt >&2
    error "$reason"
  fi
}

assert_single_owner() {
  local pattern="$1"
  local owner="$2"
  local hits
  hits="$(rg -n --fixed-strings "$pattern" src || true)"
  local count
  count="$(printf '%s\n' "$hits" | sed '/^$/d' | wc -l)"
  if [[ "$count" != "1" ]]; then
    printf '%s\n' "$hits" >&2
    error "'$pattern' must have exactly one owner: $owner"
    return
  fi
  if ! printf '%s\n' "$hits" | rg --fixed-strings "$owner" >/dev/null; then
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
  if rg -n --fixed-strings "$pattern" src \
      --glob '!src/**/*tests.rs' \
      --glob '!src/**/tests/**' \
      --glob '!src/*_tests.rs' \
      --glob '!src/bin/lay_test_input.rs' \
      --glob '!src/bin/lay_lem_research.rs' \
      >/tmp/lay-architecture-hit.txt; then
    cat /tmp/lay-architecture-hit.txt >&2
    error "chat/log regression example '$pattern' must stay in tests or data, not runtime code"
  fi
}

assert_max_lines src/bin/lay_daemon.rs 500
assert_max_lines src/typing_assist.rs 80
assert_max_lines src/llm.rs 750
assert_max_lines src/ru_typo.rs 900
assert_max_lines src/phrase_reader.rs 800
assert_max_lines src/scoped_tail.rs 600
assert_max_lines src/lem.rs 650
assert_max_lines src/bin/lay_daemon/daemon_runtime.rs 1000
assert_max_lines src/bin/lay_daemon/layout_controller.rs 850
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen.rs 550
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope.rs 550

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
assert_single_owner "fn has_cyrillic(text" "src/text_metrics.rs"
assert_single_owner "fn without_whitespace" "src/text_metrics.rs"
assert_single_owner "fn normalized_edit_distance" "src/text_metrics.rs"
assert_single_owner "fn common_replacement_span" "src/text_metrics.rs"
assert_single_owner "fn damerau_levenshtein" "src/text_metrics.rs"

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

if rg -n --fixed-strings "second_best" src --glob '!candidate_ranker.rs' >/tmp/lay-architecture-hit.txt; then
  cat /tmp/lay-architecture-hit.txt >&2
  error "best/second-best arbitration must go through candidate_ranker"
fi

rm -f /tmp/lay-architecture-hit.txt

if [[ "$fail" != "0" ]]; then
  exit 1
fi

echo "lay architecture check OK"
