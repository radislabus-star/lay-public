#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail=0
pass=0
EXPECTED_PASSES=50
AUDIT_TMP_DIR="${XDG_RUNTIME_DIR:-$ROOT/target/tmp}"
mkdir -p "$AUDIT_TMP_DIR"
OUT_FILE="$(mktemp "$AUDIT_TMP_DIR/lay-audit-out.XXXXXX")"
ERR_FILE="$(mktemp "$AUDIT_TMP_DIR/lay-audit-err.XXXXXX")"
HIT_FILE="$(mktemp "$AUDIT_TMP_DIR/lay-audit-hit.XXXXXX")"
trap 'rm -f "$OUT_FILE" "$ERR_FILE" "$HIT_FILE"' EXIT

check() {
  local name="$1"
  shift
  pass=$((pass + 1))
  if "$@" >"$OUT_FILE" 2>"$ERR_FILE"; then
    printf '%02d OK  %s\n' "$pass" "$name"
  else
    printf '%02d BAD %s\n' "$pass" "$name"
    cat "$OUT_FILE" >&2 || true
    cat "$ERR_FILE" >&2 || true
    fail=1
  fi
}

no_grep() {
  local pattern="$1"
  shift
  ! grep -RInE -- "$pattern" "$@" >"$HIT_FILE" 2>/dev/null
}

one_owner() {
  local pattern="$1"
  local owner="$2"
  local hits count
  hits="$(grep -RInF -- "$pattern" src || true)"
  count="$(printf '%s\n' "$hits" | sed '/^$/d' | wc -l)"
  [[ "$count" == "1" ]] && printf '%s\n' "$hits" | grep -F -- "$owner" >/dev/null
}

max_lines() {
  local file="$1"
  local max="$2"
  (( $(wc -l < "$file") <= max ))
}

no_runtime_sleep_outside_output() {
  ! grep -RInF -- "thread::sleep" src \
    | grep -Ev '(^src/bin/lay_daemon/text_output(\.rs|/)|^src/bin/lay_daemon/layout_controller\.rs:|^src/bin/lay_test_input(\.rs|/))'
}

gdbus_fallback_only() {
  ! grep -RInE -- 'Command::new\("gdbus"|gdbus call' src \
    | grep -Ev '(^src/bin/lay_daemon/layout_controller\.rs:|^src/bin/lay_test_input(\.rs|/))'
}

private_file_open_centralized() {
  ! grep -RInF -- "OpenOptionsExt" src | grep -v '^src/private_file.rs:'
}

chmod_centralized() {
  ! grep -RInE "PermissionsExt|set_permissions" src \
    | grep -Ev '(^src/private_file\.rs:|_tests\.rs:|^src/.*/tests/)'
}

no_clipboard_in_correction_hot_path() {
  ! grep -RInE -- "clipboard|xclip|xsel|wl-copy|wl-paste" src/bin src/*.rs \
    | grep -Ev '(^src/main\.rs:|^src/bin/lay_daemon/tests\.rs:|^src/bin/lay_daemon/focus_guard\.rs:)'
}

no_runtime_user_phrase_rules() {
  no_grep "тожесамое|тоесамое|янебуду|какпроверка|онаубыточная|прболематут|робило|банный|поения|перпаратов" \
    src/typing_replacements.rs \
    src/typing_rule_graph.rs \
    src/typing_pipeline.rs \
    src/ru_typo.rs \
    src/phrase_reader.rs
}

check "architecture guard" bash scripts/check-architecture.sh
check "format check" cargo fmt --all --check
check "no profanity/live abusive snippets" no_grep "БЛЯТЬ|ДОЛБА|СУКА|ДИКТУ|ебан|хуе|пизд|YTN YJ|LJK<FT" src tests docs README.md HOW_IT_WORKS.md
check "no runtime /tmp lay paths" no_grep "/tmp/lay" src extension install.sh update.sh README.md HOW_IT_WORKS.md
check "daemon orchestrator <=500 lines" max_lines src/bin/lay_daemon.rs 500
check "typing facade <=80 lines" max_lines src/typing_assist.rs 80
check "phrase reader <=800 lines" max_lines src/phrase_reader.rs 800
check "ru typo <=900 lines" max_lines src/ru_typo.rs 900
check "no sleeps outside output/layout/test" no_runtime_sleep_outside_output
check "gdbus fallback only" gdbus_fallback_only

check "split ws single owner" one_owner "fn split_ws_segments" "src/word_reader.rs"
check "candidate ranking single owner" one_owner "fn rank_typing_candidates" "src/typing_candidate.rs"
check "layout letter symbol single owner" one_owner "fn is_ascii_layout_letter_symbol" "src/layout_autoswitch.rs"
check "shift layout symbol single owner" one_owner "fn is_ascii_shift_letter_symbol" "src/layout_autoswitch.rs"
check "private file open centralized" private_file_open_centralized
check "chmod centralized" chmod_centralized
check "scoped tail independent from public facade" bash -c '! grep -nF "use crate::typing_assist" src/scoped_tail.rs'
check "typing pipeline independent from scoped tail" bash -c '! grep -nF "use crate::scoped_tail" src/typing_pipeline.rs'
check "lem no direct hunspell" bash -c '! grep -nE "RU_HUNSPELL|EN_HUNSPELL|read_to_string|OnceLock" src/lem.rs'
check "llm facade no transport crates" bash -c '! grep -nE "ureq::|llama_cpp|serde::" src/llm.rs'

check "typing context tests" cargo test -q typing_context --lib
check "dict tests" cargo test -q dict --lib
check "typing candidate tests" cargo test -q typing_candidate --lib
check "text edit tests" cargo test -q text_edit --lib
check "decoder tests" cargo test -q decoder --lib
check "word reader tests" cargo test -q word_reader --lib
check "word recognizer tests" cargo test -q word_recognizer --lib
check "phrase reader tests" cargo test -q phrase_reader --lib
check "ru typo tests" cargo test -q ru_typo --lib
check "mixed corpus integration" cargo test -q --test typing_assist_mixed_corpus

check "no previous-last format allocation" no_grep 'format!\("\{previous\} \{last\}"' src
check "space preservation helper exists" grep -RInF -- "committed_separator_is_preserved" src
check "replacement plan helper exists" grep -RInF -- "apply_replacement_plan_to_text" src
check "no clipboard in correction hot path" no_clipboard_in_correction_hot_path
check "no user phrase rules in runtime rule sources" no_runtime_user_phrase_rules
check "typing rules centralized" grep -RInF -- "static RULES" src/typing_rule_graph
check "pipeline default centralized" grep -RInF -- "default_typing_assist_rules" src/config
check "layout-only policy exists" grep -RInF -- "enabled_without_auto_replace" src/typing_rule_graph
check "strict policy exists" grep -RInF -- "TypingRuleRequiredSafety::Normal" src/typing_rule_graph
check "experimental policy exists" grep -RInF -- "TypingRuleRequiredSafety::Experimental" src/typing_rule_graph

check "core tests" cargo test -q core --lib
check "engine tests" cargo test -q engine --lib
check "keyboard tests" cargo test -q keyboard --lib
check "scoped tail tests" cargo test -q scoped_tail --lib
check "token language tests" cargo test -q token_language --lib
check "typing pipeline tests" cargo test -q typing_pipeline --lib
check "typing rule graph compiles via tests" cargo test -q typing_rule --lib
check "daemon scoped tail tests" cargo test -q scoped_tail --bin lay-daemon
check "daemon typing assist rules tests" cargo test -q typing_assist --bin lay-daemon
check "git diff whitespace" git diff --check

if [[ "$pass" != "$EXPECTED_PASSES" ]]; then
  printf 'audit error: expected %d checks, ran %d\n' "$EXPECTED_PASSES" "$pass" >&2
  fail=1
fi

if [[ "$fail" != "0" ]]; then
  exit 1
fi

echo "lay 50-pass audit OK"
