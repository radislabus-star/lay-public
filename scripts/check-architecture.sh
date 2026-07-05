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
        --glob '!src/bin/lay_test_input/**' \
        --glob '!src/bin/lay_lem_research.rs' \
        --glob '!src/bin/lay_lem_research/**' || true)"
  else
    hits="$(grep -RInF -- "$pattern" src || true)"
    hits="$(printf '%s\n' "$hits" \
      | grep -Ev '(^src/.*/tests/|_tests\.rs:|^src/bin/lay_daemon/tests\.rs:|^src/bin/lay_test_input(\.rs|/)|^src/bin/lay_lem_research(\.rs|/))' || true)"
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

assert_no_runtime_rule_id_literals() {
  local rule_id_regex
  rule_id_regex='moved_prefix_pair|split_word_pair|visual_b|personal_phrase|personal_token|duplicate_layout_prefix|mixed_script_layout|layout_technical|layout_ru_to_en|layout_en_to_ru|contextual_layout_en_to_ru|cyrillic_case|hard_sign|adjacent_transposition|repeated_letter|single_letter_substitution|verb_ending|vowel_confusion|extra_letters|missing_letter|glued_phrase'
  local hits
  hits="$(grep -RInE "\"(${rule_id_regex})\"" src --include='*.rs' || true)"
  hits="$(printf '%s\n' "$hits" \
    | grep -Ev '(^src/typing_rule_graph/ids\.rs:|_tests\.rs:|^src/.*/tests/|^src/bin/lay_daemon/tests\.rs:|^src/bin/lay_lem_research(\.rs|/))' || true)"
  if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits" >&2
    error "runtime rule id strings must go through src/typing_rule_graph/ids.rs"
  fi
}

assert_live_correction_entrypoint_owned_by_input_gate() {
  local pattern="$1"
  local hits
  if command -v rg >/dev/null 2>&1; then
    hits="$(rg -n --fixed-strings "$pattern" src \
        --glob '!src/correction_core.rs' \
        --glob '!src/correction_pipeline.rs' \
        --glob '!src/input_gate.rs' \
        --glob '!src/main.rs' \
        --glob '!src/**/*tests.rs' \
        --glob '!src/**/tests/**' \
        --glob '!src/*_tests.rs' || true)"
  else
    hits="$(grep -RInF -- "$pattern" src || true)"
    hits="$(printf '%s\n' "$hits" \
      | grep -Ev '(^src/correction_core\.rs:|^src/correction_pipeline\.rs:|^src/input_gate\.rs:|^src/main\.rs:|_tests\.rs:|^src/.*/tests/)' || true)"
  fi
  if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits" >&2
    error "live correction decisions must enter through input_gate, not direct '$pattern' calls"
  fi
}

assert_text_mutation_call_owners() {
  local pattern="$1"
  local reason="$2"
  shift 2
  local allowed=("$@")
  local hits
  if command -v rg >/dev/null 2>&1; then
    hits="$(rg -n --fixed-strings "$pattern" src \
        --glob '!src/**/*tests.rs' \
        --glob '!src/**/tests/**' \
        --glob '!src/*_tests.rs' || true)"
  else
    hits="$(grep -RInF -- "$pattern" src || true)"
    hits="$(printf '%s\n' "$hits" \
      | grep -Ev '(_tests\.rs:|^src/.*/tests/)' || true)"
  fi

  local filtered=""
  local line file ok allowed_file
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    file="${line%%:*}"
    ok=0
    for allowed_file in "${allowed[@]}"; do
      if [[ "$file" == "$allowed_file" ]]; then
        ok=1
        break
      fi
    done
    if [[ "$ok" == "0" ]]; then
      filtered+="${line}"$'\n'
    fi
  done <<< "$hits"

  if [[ -n "$filtered" ]]; then
    printf '%s' "$filtered" >&2
    error "$reason"
  fi
}

assert_single_owner "fn unix_timestamp(" "src/time.rs"
assert_single_owner "fn is_cyrillic_letter(" "src/keyboard/text_input/script.rs"
assert_single_owner "fn mix64(" "src/nanda_wave/mode.rs"
assert_single_owner "fn mix64_golden(" "src/nanda_wave/mode.rs"
assert_single_owner "fn split_last_ws_token(" "src/word_reader.rs"
assert_single_owner "fn split_last_trimmed_ws_token(" "src/word_reader.rs"
assert_single_owner "fn split_last_alphabetic_token(" "src/word_reader.rs"
assert_single_owner "pub enum LanguageActionOperator" "src/language_action.rs"
assert_single_owner "pub fn operator_for_candidate(" "src/language_action.rs"
if search_fixed "fn split_last_token(" src >"$HIT_FILE"; then
  cat "$HIT_FILE" >&2
  error "ambiguous split_last_token helper is forbidden; use explicit word_reader split helpers"
fi
assert_live_correction_entrypoint_owned_by_input_gate "resolve_text_correction("
assert_live_correction_entrypoint_owned_by_input_gate "decide_text_correction("

assert_text_mutation_call_owners \
  "apply_text_replacement_pipeline(" \
  "direct text replacement must stay inside approved text-edit output owners; route through TextEditAuthority before adding new callers" \
  src/bin/lay_daemon/text_output/replacement.rs \
  src/bin/lay_daemon/text_output.rs \
  src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs \
  src/bin/lay_daemon/correction_runtime/output/text_replace.rs \
  src/bin/lay_daemon/enter_autocorrect_runtime.rs \
  src/bin/lay_daemon/auto_undo_runtime.rs

assert_text_mutation_call_owners \
  "try_ime_replace_tail(" \
  "direct IME tail replacement must stay inside approved text-edit output owners; route through TextEditAuthority before adding new callers" \
  src/bin/lay_daemon/layout_controller.rs \
  src/bin/lay_daemon/typing_assist_runtime/output/ime.rs \
  src/bin/lay_daemon/correction_runtime/output/native.rs \
  src/bin/lay_daemon/enter_autocorrect_runtime.rs \
  src/bin/lay_daemon/auto_undo_runtime.rs

assert_text_mutation_call_owners \
  "replace_tail_plan(" \
  "raw IME replace-tail plans must stay inside manual-toggle bridge owners" \
  src/bin/lay_daemon/layout_controller/ime_bridge.rs \
  src/bin/lay_daemon/layout_controller/ime_manual_toggle.rs

assert_text_mutation_call_owners \
  "try_replace_tail(" \
  "raw IME replace-tail bridge calls must stay inside layout_controller bridge owners" \
  src/bin/lay_daemon/layout_controller.rs \
  src/bin/lay_daemon/layout_controller/ime_bridge.rs

assert_text_mutation_call_owners \
  "replace_committed_tail(" \
  "IBus committed-tail replacement must stay inside approved IME backend owners" \
  src/bin/lay_ibus_engine/state.rs \
  src/bin/lay_ibus_engine/committed_tail.rs \
  src/bin/lay_ibus_engine/bridge_actions.rs \
  src/bin/lay_ibus_engine/composition_commit.rs

assert_text_mutation_call_owners \
  "commit_active_composition(" \
  "IBus active composition commits must stay inside approved IME backend owners" \
  src/bin/lay_ibus_engine/composition_commit.rs \
  src/bin/lay_ibus_engine/shift.rs \
  src/bin/lay_ibus_engine/managed.rs

assert_text_mutation_call_owners \
  "autocorrect_active_composition_text(" \
  "active-composition autocorrect must stay inside IME composition owner until TextEditAuthority migration" \
  src/bin/lay_ibus_engine/composition_commit.rs \
  src/bin/lay_ibus_engine/preedit.rs

assert_text_mutation_call_owners \
  "autocorrect_committed_tail_text(" \
  "committed-tail autocorrect must stay inside IME composition owner until TextEditAuthority migration" \
  src/bin/lay_ibus_engine/composition_commit.rs

assert_text_mutation_call_owners \
  "apply_pending_committed_tail_space_autocorrect(" \
  "pending committed-tail autocorrect must stay inside approved IME boundary owners" \
  src/bin/lay_ibus_engine/committed_tail.rs \
  src/bin/lay_ibus_engine/ibus_interface.rs

assert_max_lines src/bin/lay_daemon.rs 240
assert_max_lines src/bin/lay_daemon/action_log_runtime.rs 40
assert_max_lines src/bin/lay_daemon/config_runtime.rs 180
assert_max_lines src/bin/lay_daemon/daemon_state.rs 200
assert_max_lines src/bin/lay_daemon/log_runtime.rs 40
assert_max_lines src/bin/lay_daemon/startup_runtime.rs 190
assert_max_lines src/typing_assist.rs 80
assert_max_lines src/config.rs 40
assert_max_lines src/config/active.rs 70
assert_max_lines src/config/defaults.rs 90
assert_max_lines src/config/load.rs 25
assert_max_lines src/config/pipeline.rs 100
assert_max_lines src/config/types.rs 90
assert_max_lines src/llm.rs 60
assert_max_lines src/llm/consensus.rs 90
assert_max_lines src/llm/hybrid.rs 50
assert_max_lines src/llm/token_choice.rs 110
assert_max_lines src/llm/tokenwise.rs 120
assert_max_lines src/llm_backend.rs 160
assert_max_lines src/llm_backend/direct.rs 280
assert_max_lines src/llm_backend/http.rs 220
assert_max_lines src/layout_autoswitch/ascii.rs 40
assert_max_lines src/layout_autoswitch/ascii/candidate.rs 90
assert_max_lines src/layout_autoswitch/ascii/phrase.rs 140
assert_max_lines src/layout_autoswitch/ascii/symbols.rs 70
assert_max_lines src/layout_autoswitch/ascii/word.rs 180
assert_max_lines src/decoder.rs 60
assert_max_lines src/decoder/edit_plan.rs 90
assert_max_lines src/decoder/manual.rs 160
assert_max_lines src/decoder/ranked.rs 120
assert_max_lines src/decoder/types.rs 70
assert_max_lines src/decoder/typing_tail.rs 120
assert_max_lines src/decoder/punctuation.rs 90
assert_max_lines src/typing_pipeline/candidates.rs 120
assert_max_lines src/typing_rule_graph.rs 50
assert_max_lines src/typing_rule_graph/builders.rs 160
assert_max_lines src/typing_rule_graph/definitions.rs 180
assert_max_lines src/typing_rule_graph/ids.rs 40
assert_max_lines src/typing_rule_graph/registry.rs 60
assert_max_lines src/typing_rule_graph/rules.rs 180
assert_max_lines src/typing_rule_graph/types.rs 40
assert_max_lines src/typing_rule_graph/weights.rs 50
assert_max_lines src/ru_typo.rs 80
assert_max_lines src/ru_typo/case_rule.rs 60
assert_max_lines src/ru_typo/coverage.rs 80
assert_max_lines src/ru_typo/extra.rs 80
assert_max_lines src/ru_typo/guards.rs 130
assert_max_lines src/ru_typo/hard_sign.rs 40
assert_max_lines src/ru_typo/keyboard.rs 40
assert_max_lines src/ru_typo/missing.rs 120
assert_max_lines src/ru_typo/repeated.rs 110
assert_max_lines src/ru_typo/substitution.rs 90
assert_max_lines src/ru_typo/thresholds.rs 30
assert_max_lines src/ru_typo/transposition.rs 80
assert_max_lines src/ru_typo/verb.rs 80
assert_max_lines src/ru_typo/vowel.rs 50
assert_max_lines src/phrase_reader.rs 60
assert_max_lines src/phrase_reader/contextual_tail.rs 100
assert_max_lines src/phrase_reader/glued_phrase.rs 180
assert_max_lines src/phrase_reader/guards.rs 180
assert_max_lines src/phrase_reader/moved_prefix.rs 130
assert_max_lines src/phrase_reader/split_pair.rs 90
assert_max_lines src/russian_lexicon.rs 140
assert_max_lines src/russian_lexicon/forms.rs 260
assert_max_lines src/russian_lexicon/hunspell.rs 220
assert_max_lines src/scoped_tail.rs 220
assert_max_lines src/scoped_tail/completed_word.rs 140
assert_max_lines src/scoped_tail/lem_candidates.rs 160
assert_max_lines src/scoped_tail/scope_policy.rs 60
assert_max_lines src/scoped_tail/word_flip.rs 140
assert_max_lines src/lem.rs 30
assert_max_lines src/lem/language.rs 110
assert_max_lines src/lem/noise.rs 110
assert_max_lines src/lem/rank.rs 40
assert_max_lines src/lem/score.rs 120
assert_max_lines src/lem/token.rs 100
assert_max_lines src/lem/types.rs 20
assert_max_lines src/lem/warmup.rs 20
assert_max_lines src/ngram.rs 30
assert_max_lines src/ngram/cache.rs 60
assert_max_lines src/ngram/model.rs 120
assert_max_lines src/ngram/sources.rs 70
assert_max_lines src/ngram/static_models.rs 60
assert_max_lines src/ngram/tokenize.rs 50
assert_max_lines src/keyboard.rs 80
assert_max_lines src/keyboard/keymap.rs 30
assert_max_lines src/keyboard/keymap/ru_char.rs 130
assert_max_lines src/keyboard/keymap/typing_key.rs 70
assert_max_lines src/keyboard/keymap/us_char.rs 130
assert_max_lines src/keyboard/text_input.rs 30
assert_max_lines src/keyboard/text_input/ru_emit.rs 100
assert_max_lines src/keyboard/text_input/runs.rs 70
assert_max_lines src/keyboard/text_input/script.rs 40
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/decoder.rs 90
assert_max_lines src/bin/lay_daemon/tests/field_context.rs 120
assert_max_lines src/keyboard/text_input/us_emit.rs 100
assert_max_lines src/keyboard/event_words.rs 30
assert_max_lines src/keyboard/event_words/decision.rs 60
assert_max_lines src/keyboard/event_words/mapping.rs 70
assert_max_lines src/keyboard/event_words/visual_latin.rs 70
assert_max_lines src/keyboard/event_words/word_split.rs 70
assert_max_lines src/word_buffer.rs 180
assert_max_lines src/word_buffer/learning.rs 180
assert_max_lines src/word_buffer/replay_memory.rs 280
assert_max_lines src/word_buffer/replay_scope.rs 160
assert_max_lines src/word_recognizer.rs 60
assert_max_lines src/word_recognizer/identity.rs 160
assert_max_lines src/word_recognizer/lexicon.rs 90
assert_max_lines src/word_recognizer/risk.rs 60
assert_max_lines src/word_recognizer/script.rs 80
assert_max_lines src/word_recognizer/technical.rs 180
assert_max_lines src/typing_candidate.rs 50
assert_max_lines src/typing_candidate/confidence.rs 50
assert_max_lines src/typing_candidate/ranking.rs 90
assert_max_lines src/typing_candidate/scoring.rs 180
assert_max_lines src/typing_candidate/types.rs 70
assert_max_lines src/typing_pipeline.rs 40
assert_max_lines src/typing_pipeline/engine.rs 140
assert_max_lines src/typing_pipeline/rule_order.rs 40
assert_max_lines src/typing_pipeline/types.rs 120
assert_max_lines src/typing_pipeline/warmup.rs 20
assert_max_lines src/typing_context.rs 50
assert_max_lines src/typing_context/context_window.rs 50
assert_max_lines src/typing_context/layout_signal.rs 160
assert_max_lines src/typing_context/pipeline.rs 90
assert_max_lines src/typing_context/tokens.rs 110
assert_max_lines src/bin/lay_daemon/daemon_runtime.rs 500
assert_max_lines src/bin/lay_daemon/trigger_dispatch.rs 160
assert_max_lines src/bin/lay_daemon/manual_trigger_runtime.rs 60
assert_max_lines src/bin/lay_daemon/manual_trigger_runtime/context.rs 120
assert_max_lines src/bin/lay_daemon/manual_trigger_runtime/event.rs 280
assert_max_lines src/bin/lay_daemon/manual_trigger_runtime/fire.rs 70
assert_max_lines src/bin/lay_daemon/manual_trigger_runtime/timeout.rs 60
assert_max_lines src/bin/lay_daemon/boundary_runtime.rs 40
assert_max_lines src/bin/lay_daemon/boundary_runtime/deferred.rs 70
assert_max_lines src/bin/lay_daemon/boundary_runtime/enter.rs 80
assert_max_lines src/bin/lay_daemon/boundary_runtime/hard.rs 90
assert_max_lines src/bin/lay_daemon/boundary_runtime/space.rs 120
assert_max_lines src/bin/lay_daemon/typing_key_runtime.rs 120
assert_max_lines src/bin/lay_daemon/text_output.rs 40
assert_max_lines src/bin/lay_daemon/text_output/device.rs 90
assert_max_lines src/bin/lay_daemon/text_output/key_emit.rs 200
assert_max_lines src/bin/lay_daemon/text_output/modifiers.rs 70
assert_max_lines src/bin/lay_daemon/text_output/replacement.rs 240
assert_max_lines src/bin/lay_daemon/typing_assist_runtime.rs 80
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/candidate.rs 60
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/output.rs 140
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/output/defer.rs 30
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/output/ime.rs 100
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/output/memory.rs 80
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs 130
assert_max_lines src/bin/lay_daemon/typing_assist_runtime/output/nanda_trace.rs 40
assert_max_lines src/bin/lay_daemon/buffer_filter_runtime.rs 90
assert_max_lines src/bin/lay_daemon/manual_trigger_diagnostics.rs 50
assert_max_lines src/bin/lay_daemon/command_runtime.rs 80
assert_max_lines src/bin/lay_daemon/force_layout_hotkeys.rs 170
assert_max_lines src/bin/lay_daemon/correction_runtime.rs 180
assert_max_lines src/bin/lay_daemon/correction_runtime/memory.rs 60
assert_max_lines src/bin/lay_daemon/correction_runtime/output.rs 90
assert_max_lines src/bin/lay_daemon/correction_runtime/output/context.rs 70
assert_max_lines src/bin/lay_daemon/correction_runtime/output/native.rs 180
assert_max_lines src/bin/lay_daemon/correction_runtime/output/replay.rs 100
assert_max_lines src/bin/lay_daemon/correction_runtime/output/text_replace.rs 180
assert_max_lines src/bin/lay_daemon/auto_undo_runtime.rs 140
assert_max_lines src/bin/lay_daemon/layout_controller.rs 280
assert_max_lines src/bin/lay_daemon/layout_controller/gnome_dbus.rs 360
assert_max_lines src/bin/lay_daemon/layout_controller/ibus_bridge.rs 100
assert_max_lines src/bin/lay_daemon/layout_controller/ime_bridge.rs 140
assert_max_lines src/bin/lay_daemon/layout_kde.rs 180
assert_max_lines src/bin/lay_daemon/layout_x11.rs 100
assert_max_lines src/bin/lay_daemon/learning_runtime.rs 80
assert_max_lines src/bin/lay_daemon/learning_runtime/log_file.rs 140
assert_max_lines src/bin/lay_daemon/learning_runtime/promotion.rs 190
assert_max_lines src/bin/lay_daemon/tests.rs 350
assert_max_lines src/bin/lay_daemon/tests/config_contract.rs 110
assert_max_lines src/bin/lay_daemon/tests/enter_autocorrect.rs 70
assert_max_lines src/bin/lay_daemon/tests/layout_backend.rs 140
assert_max_lines src/bin/lay_daemon/tests/learning_log.rs 20
assert_max_lines src/bin/lay_daemon/tests/runtime_state.rs 300
assert_max_lines src/bin/lay_daemon/tests/text_output_contract.rs 160
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules.rs 30
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/exact_layout.rs 200
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/pipeline.rs 110
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/safety_regression.rs 130
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/typo_families.rs 30
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/typo_families/missing_case.rs 130
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/typo_families/phrase_spacing.rs 80
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/typo_families/repeated_extra.rs 70
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/typo_families/technical_prefix.rs 80
assert_max_lines src/bin/lay_daemon/tests/typing_assist_rules/typo_families/transposition.rs 30
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen.rs 30
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen/matrix_visual.rs 150
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen/mixed_prefix.rs 150
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen/replacement_memory.rs 40
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen/short_tail.rs 90
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen/token_layout.rs 100
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/technical_hyphen/typing_assist.rs 50
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope.rs 30
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope/mixed_current.rs 150
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope/previous_context.rs 120
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope/three_word.rs 150
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/lem_scope/two_word.rs 190
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/mixed_context.rs 30
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/mixed_context/current_tail.rs 170
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/mixed_context/stale_layout.rs 70
assert_max_lines src/bin/lay_daemon/tests/scoped_tail/mixed_context/trailing_space.rs 90
assert_max_lines src/bin/lay_lem_research.rs 40
assert_max_lines src/bin/lay_lem_research/candidates.rs 100
assert_max_lines src/bin/lay_lem_research/cases.rs 300
assert_max_lines src/bin/lay_lem_research/report.rs 140
assert_max_lines src/bin/lay_test_input.rs 120
assert_max_lines src/bin/lay_test_input/desktop_probe.rs 320
assert_max_lines src/bin/lay_test_input/input_device.rs 180
assert_max_lines src/bin/lay_test_input/scenarios.rs 480
assert_max_lines scripts/run_runtime_smoke.py 450
assert_max_lines scripts/runtime_smoke/cases.py 180
assert_max_lines scripts/runtime_smoke/ime.py 130
assert_max_lines src/bin/lay_ibus_engine.rs 60
assert_max_lines src/bin/lay_ibus_engine/args.rs 30
assert_max_lines src/bin/lay_ibus_engine/bridge.rs 90
assert_max_lines src/bin/lay_ibus_engine/engine.rs 220
assert_max_lines src/bin/lay_ibus_engine/factory.rs 90
assert_max_lines src/bin/lay_ibus_engine/managed.rs 160
assert_max_lines src/bin/lay_ibus_engine/protocol.rs 40
assert_max_lines src/bin/lay_ibus_engine/server.rs 80
assert_max_lines src/bin/lay_ibus_engine/text.rs 30
assert_max_lines src/bin/lay_ibus_engine/xml.rs 60
assert_max_lines extension/lay@radislabus-star.github.io/lay-impl.js 1800
assert_max_lines src/typing_candidate.rs 280
assert_max_lines src/text_edit.rs 50
assert_max_lines src/text_edit/committed_tail.rs 180
assert_max_lines src/text_edit/cursor.rs 40
assert_max_lines src/text_edit/diff_plan.rs 90
assert_max_lines src/text_edit/types.rs 20

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

assert_no_import src/bin/lay_test_input.rs \
  "use std::process" \
  "lay-test-input desktop command probing belongs in lay_test_input/desktop_probe.rs"

assert_no_import src/bin/lay_test_input.rs \
  "use lay::config::LayConfig" \
  "lay-test-input config diagnostics belong in lay_test_input/desktop_probe.rs"

assert_no_import src/bin/lay_daemon.rs \
  "SystemTime" \
  "lay-daemon root must not own logging internals; use lay_daemon/log_runtime.rs"

assert_no_import src/bin/lay_daemon.rs \
  "UNIX_EPOCH" \
  "lay-daemon root must not own logging internals; use lay_daemon/log_runtime.rs"

assert_no_import src/bin/lay_daemon.rs \
  "LayConfig" \
  "lay-daemon root must not own config access or startup loading; use config_runtime/startup_runtime"

assert_no_import extension/lay@radislabus-star.github.io/dbus_service.js \
  "Gio.Subprocess.new(['ibus'" \
  "GNOME DBus service must not spawn ibus; use IBus.Bus or daemon layout controller"

assert_no_import src/llm.rs \
  "ureq::" \
  "llm.rs must stay a deterministic arbiter facade; model transport belongs in llm_backend"

assert_no_import src/llm.rs \
  "llama_cpp" \
  "llm.rs must stay a deterministic arbiter facade; direct GGUF runtime belongs in llm_backend"

assert_no_import src/llm.rs \
  "serde::" \
  "llm.rs must stay a deterministic arbiter facade; model response structs belong in llm_backend"

assert_no_import src/llm_backend.rs \
  "ureq::" \
  "llm_backend facade must choose providers; HTTP transport belongs in llm_backend/http.rs"

assert_no_import src/llm_backend.rs \
  "llama_cpp" \
  "llm_backend facade must choose providers; direct GGUF runtime belongs in llm_backend/direct.rs"

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
assert_single_owner "pub fn correct_moved_prefix_letter_pair" "src/phrase_reader/moved_prefix.rs"
assert_single_owner "pub fn correct_split_word_pair" "src/phrase_reader/split_pair.rs"
assert_single_owner "pub fn correct_contextual_glued_tail" "src/phrase_reader/contextual_tail.rs"
assert_single_owner "pub fn correct_glued_russian_phrase" "src/phrase_reader/glued_phrase.rs"
assert_single_owner "fn is_confident_glued_phrase_split" "src/phrase_reader/guards.rs"
assert_single_owner "pub fn russian_dictionary" "src/russian_lexicon.rs"
assert_single_owner "pub(crate) fn is_known_russian_form" "src/russian_lexicon/forms.rs"
assert_single_owner "pub(super) fn load_hunspell_generated_forms_min_len" "src/russian_lexicon/hunspell.rs"
assert_single_owner "fn is_russian_vowel" "src/russian_chars.rs"
assert_single_owner "fn same_letter_ignore_case" "src/russian_chars.rs"
assert_single_owner "pub(crate) fn correct_cyrillic_word_case" "src/ru_typo/case_rule.rs"
assert_single_owner "pub(crate) fn has_plausible_russian_typo_candidate" "src/ru_typo/coverage.rs"
assert_single_owner "pub fn correct_extra_letters" "src/ru_typo/extra.rs"
assert_single_owner "pub(crate) fn correct_hard_sign_typo" "src/ru_typo/hard_sign.rs"
assert_single_owner "pub fn correct_missing_letter" "src/ru_typo/missing.rs"
assert_single_owner "pub(crate) fn safe_missing_letter_candidates" "src/ru_typo/missing.rs"
assert_single_owner "pub(crate) fn correct_repeated_letter" "src/ru_typo/repeated.rs"
assert_single_owner "pub(crate) fn correct_single_letter_substitution" "src/ru_typo/substitution.rs"
assert_single_owner "pub(crate) fn correct_adjacent_transposition" "src/ru_typo/transposition.rs"
assert_single_owner "pub(crate) fn correct_verb_ending_confusion" "src/ru_typo/verb.rs"
assert_single_owner "pub(crate) fn correct_vowel_confusion" "src/ru_typo/vowel.rs"
assert_single_owner "pub fn are_ru_keyboard_neighbors" "src/ru_typo/keyboard.rs"
assert_single_owner "fn ru_keyboard_position" "src/ru_typo/keyboard.rs"
assert_single_owner "fn looks_like_plausible_russian_past_tense" "src/ru_typo/guards.rs"
assert_single_owner "pub fn recognize_token" "src/word_recognizer/identity.rs"
assert_single_owner "pub(super) fn detect_script" "src/word_recognizer/script.rs"
assert_single_owner "pub(super) fn known_russian_word" "src/word_recognizer/lexicon.rs"
assert_single_owner "pub(super) fn known_english_word" "src/word_recognizer/lexicon.rs"
assert_single_owner "pub fn is_plain_layout_autocorrect_risky" "src/word_recognizer/risk.rs"
assert_single_owner "pub fn is_probably_completed_natural_word" "src/word_recognizer/risk.rs"
assert_single_owner "pub fn is_cli_option_token" "src/word_recognizer/technical.rs"
assert_single_owner "pub fn is_protected_ascii_token" "src/word_recognizer/technical.rs"
assert_single_owner "pub fn is_ascii_technical_token" "src/word_recognizer/technical.rs"
assert_single_owner "pub fn is_ascii_technical_or_brand_token" "src/word_recognizer/technical.rs"
assert_single_owner "pub fn is_upper_ascii_acronym" "src/word_recognizer/technical.rs"
assert_single_owner "pub fn is_mixed_case_ascii_brand" "src/word_recognizer/technical.rs"
assert_single_owner "pub fn is_mixed_cyrillic_ascii_alpha_token" "src/word_recognizer/technical.rs"
assert_single_owner "pub(crate) fn correct_wrong_layout_ascii_word(" "src/layout_autoswitch/ascii/word.rs"
assert_single_owner "pub(crate) fn correct_wrong_layout_ascii_phrase" "src/layout_autoswitch/ascii/phrase.rs"
assert_single_owner "pub(crate) fn is_confident_wrong_layout_ascii_pair" "src/layout_autoswitch/ascii/phrase.rs"
assert_single_owner "fn ascii_to_russian_layout_candidate" "src/layout_autoswitch/ascii/candidate.rs"
assert_single_owner "fn is_ascii_layout_letter_symbol" "src/layout_autoswitch/ascii/symbols.rs"
assert_single_owner "fn is_ascii_shift_letter_symbol" "src/layout_autoswitch/ascii/symbols.rs"
assert_single_owner "fn is_plain_ascii_layout_token" "src/layout_autoswitch/ascii/symbols.rs"
assert_single_owner "fn has_cyrillic(text" "src/text_metrics.rs"
assert_single_owner "fn without_whitespace" "src/text_metrics.rs"
assert_single_owner "fn normalized_edit_distance" "src/text_metrics.rs"
assert_single_owner "fn common_replacement_span" "src/text_metrics.rs"
assert_single_owner "fn damerau_levenshtein" "src/text_metrics.rs"
assert_single_owner "fn apply_replacement_plan_to_text" "src/text_edit/diff_plan.rs"
assert_single_owner "fn plan_text_replacement_with_options" "src/text_edit/diff_plan.rs"
assert_single_owner "fn committed_separator_is_preserved" "src/text_edit/committed_tail.rs"
assert_single_owner "fn plan_committed_tail_replacement" "src/text_edit/committed_tail.rs"
assert_single_owner "fn plan_committed_whitespace_insertions" "src/text_edit/committed_tail.rs"
assert_single_owner "fn offset_replacement_plan_for_cursor" "src/text_edit/cursor.rs"
assert_single_owner "pub(crate) fn choose_candidate" "src/llm_backend.rs"
assert_single_owner "pub(super) fn choose_candidate_ollama" "src/llm_backend/http.rs"
assert_single_owner "pub(super) fn choose_candidate_openai" "src/llm_backend/http.rs"
assert_single_owner "pub(super) fn choose_candidate_anthropic" "src/llm_backend/http.rs"
assert_single_owner "pub(super) fn choose_candidate_direct" "src/llm_backend/direct.rs"
assert_single_owner "pub(crate) fn make_virtual_keyboard" "src/bin/lay_daemon/text_output/device.rs"
assert_single_owner "pub(crate) fn replay_keycodes(" "src/bin/lay_daemon/text_output/key_emit.rs"
assert_single_owner "pub(crate) fn emit_backspaces(" "src/bin/lay_daemon/text_output/key_emit.rs"
assert_single_owner "pub(crate) fn release_possible_modifiers(" "src/bin/lay_daemon/text_output/modifiers.rs"
assert_single_owner "fn prepare_text_insert_for_replacement_plan(" "src/bin/lay_daemon/text_output/replacement.rs"
assert_single_owner "pub(crate) fn apply_text_replacement" "src/bin/lay_daemon/text_output/replacement.rs"
assert_single_owner "pub fn decide_completed_scope_word" "src/scoped_tail/completed_word.rs"
assert_single_owner "fn flip_word_events" "src/scoped_tail/word_flip.rs"
assert_single_owner "pub fn repair_cyrillic_prefix_before_ascii_tail" "src/scoped_tail/word_flip.rs"
assert_single_owner "pub fn scoped_tail_lem_candidates" "src/scoped_tail/lem_candidates.rs"
assert_single_owner "pub fn effective_replace_words" "src/scoped_tail/scope_policy.rs"
assert_single_owner "pub fn rank_typing_candidates" "src/typing_candidate/ranking.rs"
assert_single_owner "pub fn choose_typing_candidate" "src/typing_candidate/ranking.rs"
assert_single_owner "pub fn classify_typing_confidence" "src/typing_candidate/confidence.rs"
assert_single_owner "pub fn score_typing_candidate" "src/typing_candidate/scoring.rs"
assert_single_owner "pub fn classify_typing_rule" "src/typing_candidate/scoring.rs"
assert_single_owner "pub fn typing_assist_pipeline_for_context" "src/typing_context/pipeline.rs"
assert_single_owner "pub fn should_enable_ascii_to_ru_layout" "src/typing_context/layout_signal.rs"
assert_single_owner "pub fn completed_tail_context" "src/typing_context/context_window.rs"
assert_single_owner "fn strong_ascii_to_ru_layout_candidate" "src/typing_context/layout_signal/candidate.rs"
assert_single_owner "fn clean_ascii_to_ru_layout_candidate" "src/typing_context/layout_signal/candidate.rs"
assert_single_owner "fn is_russian_context_token" "src/typing_context/tokens.rs"
assert_single_owner "pub(crate) fn typing_rule_definitions" "src/typing_rule_graph/registry.rs"
assert_single_owner "pub(crate) fn find_typing_rule" "src/typing_rule_graph/registry.rs"
assert_single_owner "pub(crate) fn typing_rule_enabled_without_auto_replace" "src/typing_rule_graph/registry.rs"
assert_single_owner "pub(crate) fn typing_rule_required_safety" "src/typing_rule_graph/registry.rs"
assert_single_owner "pub(crate) fn typing_rule_family_weight" "src/typing_rule_graph/weights.rs"
assert_single_owner "pub(crate) fn typing_rule_candidate_is_safe" "src/typing_rule_graph/weights.rs"
assert_single_owner "fn apply_layout_en_to_ru(" "src/typing_rule_graph/rules.rs"
assert_single_owner "fn apply_word_rule(" "src/typing_rule_graph/rules.rs"
assert_single_owner "pub fn decode_manual_tail" "src/decoder/manual.rs"
assert_single_owner "pub fn rank_scoped_tail_candidates" "src/decoder/ranked.rs"
assert_single_owner "pub fn choose_ranked_scoped_tail" "src/decoder/ranked.rs"
assert_single_owner "pub fn decode_typing_assist_tail(" "src/decoder/typing_tail.rs"
assert_single_owner "pub fn decode_typing_assist_current_tail" "src/decoder/punctuation.rs"
assert_single_owner "pub fn decode_enter_autocorrect_tail" "src/decoder/punctuation.rs"
assert_single_owner "fn plan_matches_replacement" "src/decoder/edit_plan.rs"
assert_single_owner "fn is_typing_key" "src/keyboard/keymap/typing_key.rs"
assert_single_owner "fn keycode_to_ru_char" "src/keyboard/keymap/ru_char.rs"
assert_single_owner "fn keycode_to_us_char" "src/keyboard/keymap/us_char.rs"
assert_single_owner "fn text_to_uinput_runs" "src/keyboard/text_input/runs.rs"
assert_single_owner "fn text_to_key_events" "src/keyboard/text_input/runs.rs"
assert_single_owner "fn char_to_layout_key_event" "src/keyboard/text_input/runs.rs"
assert_single_owner "fn char_to_ru_key_event" "src/keyboard/text_input/ru_emit.rs"
assert_single_owner "fn char_to_us_key_event" "src/keyboard/text_input/us_emit.rs"
assert_single_owner "fn preferred_layout_for_text" "src/keyboard/text_input/script.rs"
assert_single_owner "fn replay_layout_decision" "src/keyboard/event_words/decision.rs"
assert_single_owner "fn is_layout_decision_key" "src/keyboard/event_words/decision.rs"
assert_single_owner "fn split_event_words" "src/keyboard/event_words/word_split.rs"
assert_single_owner "fn mark_word_layout" "src/keyboard/event_words/word_split.rs"
assert_single_owner "fn map_original_events" "src/keyboard/event_words/mapping.rs"
assert_single_owner "fn map_opposite_events" "src/keyboard/event_words/mapping.rs"
assert_single_owner "fn map_events_to_layout" "src/keyboard/event_words/mapping.rs"
assert_single_owner "fn original_event_char" "src/keyboard/event_words/mapping.rs"
assert_single_owner "fn mixed_visual_latin_word_target_layout" "src/keyboard/event_words/visual_latin.rs"
assert_single_owner "fn apply_typing_assist_with_pipeline" "src/typing_pipeline/engine.rs"
assert_single_owner "fn explain_typing_assist_with_pipeline" "src/typing_pipeline/engine.rs"
assert_single_owner "fn typing_rules_for_evaluation" "src/typing_pipeline/rule_order.rs"
assert_single_owner "struct TypingAssistExplanation" "src/typing_pipeline/types.rs"
assert_single_owner "pub(super) fn active_layout_backend" "src/bin/lay_daemon/config_runtime.rs"
assert_single_owner "pub(super) fn active_enter_autocorrect_from_env" "src/bin/lay_daemon/config_runtime.rs"
assert_single_owner "struct DaemonLoopState" "src/bin/lay_daemon/daemon_state.rs"
assert_single_owner "pub(super) fn set_log_enabled" "src/bin/lay_daemon/log_runtime.rs"
assert_single_owner "pub(super) fn record_recent_action" "src/bin/lay_daemon/action_log_runtime.rs"
assert_single_owner "pub fn what_to_replay" "src/word_buffer/replay_scope.rs"
assert_single_owner "pub fn remember_replacement_last_word_for_replay" "src/word_buffer/replay_memory.rs"
assert_single_owner "pub fn remember_pending_learning_correction" "src/word_buffer/learning.rs"
assert_single_owner "fn handle_pending_auto_undo" "src/bin/lay_daemon/auto_undo_runtime.rs"
assert_single_owner "struct ForceLayoutHotkeys" "src/bin/lay_daemon/force_layout_hotkeys.rs"
assert_single_owner "fn handle_manual_trigger_event" "src/bin/lay_daemon/manual_trigger_runtime/event.rs"
assert_single_owner "fn fire_expired_pending_multi_tap" "src/bin/lay_daemon/manual_trigger_runtime/timeout.rs"
assert_single_owner "fn apply_manual_correction_output" "src/bin/lay_daemon/correction_runtime/output.rs"
assert_single_owner "fn try_ime_replace_output" "src/bin/lay_daemon/correction_runtime/output/native.rs"
assert_single_owner "fn try_manual_text_replacement" "src/bin/lay_daemon/correction_runtime/output/text_replace.rs"
assert_single_owner "fn apply_layout_replay" "src/bin/lay_daemon/correction_runtime/output/replay.rs"
assert_single_owner "fn remember_layout_replay_success" "src/bin/lay_daemon/correction_runtime/memory.rs"
assert_single_owner "fn find_typing_assist_correction" "src/bin/lay_daemon/typing_assist_runtime/candidate.rs"
assert_single_owner "fn apply_typing_assist_correction" "src/bin/lay_daemon/typing_assist_runtime/output.rs"
assert_single_owner "fn try_apply_ime_replacement" "src/bin/lay_daemon/typing_assist_runtime/output/ime.rs"
assert_single_owner "fn apply_minimal_typing_replacement" "src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs"
assert_single_owner "fn remember_typing_assist_correction" "src/bin/lay_daemon/typing_assist_runtime/output/memory.rs"
assert_single_owner "fn try_handle_space_release" "src/bin/lay_daemon/boundary_runtime/space.rs"
assert_single_owner "fn handle_space_press" "src/bin/lay_daemon/boundary_runtime/space.rs"
assert_single_owner "fn try_handle_deferred_typing_assist" "src/bin/lay_daemon/boundary_runtime/deferred.rs"
assert_single_owner "fn try_handle_enter_autocorrect" "src/bin/lay_daemon/boundary_runtime/enter.rs"
assert_single_owner "fn handle_hard_boundary_if_needed" "src/bin/lay_daemon/boundary_runtime/hard.rs"
assert_single_owner "fn should_skip_buffer_input" "src/bin/lay_daemon/buffer_filter_runtime.rs"
assert_single_owner "fn log_manual_trigger_cross_check" "src/bin/lay_daemon/manual_trigger_diagnostics.rs"
assert_single_owner "fn append_learning_log_to_path" "src/bin/lay_daemon/learning_runtime/log_file.rs"
assert_single_owner "fn append_user_correction_learning_log_to_path" "src/bin/lay_daemon/learning_runtime/log_file.rs"
assert_single_owner "fn keep_last_jsonl_lines" "src/bin/lay_daemon/learning_runtime/log_file.rs"
assert_single_owner "fn promote_user_correction_if_repeated" "src/bin/lay_daemon/learning_runtime/promotion.rs"
assert_single_owner "fn normalizable_learning_rule" "src/bin/lay_daemon/learning_runtime/promotion.rs"
assert_single_owner "fn add_replacement_rule_to_path" "src/bin/lay_daemon/learning_runtime/promotion.rs"

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

assert_no_runtime_rule_id_literals

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
    --glob '!src/bin/lay_daemon/text_output/**' \
    --glob '!src/bin/lay_daemon/layout_controller.rs' \
    --glob '!src/bin/lay_test_input.rs' \
    --glob '!src/bin/lay_test_input/**' || true)"
else
  sleep_hits="$(grep -RInF -- "thread::sleep" src \
    | grep -Ev '(^src/bin/lay_daemon/text_output(\.rs|/)|^src/bin/lay_daemon/layout_controller\.rs:|^src/bin/lay_test_input(\.rs|/))' || true)"
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
