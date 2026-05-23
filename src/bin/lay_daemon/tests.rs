use super::*;
use evdev::KeyCode;
use lay::config::{
    default_typing_assist_pipeline, normalize_typing_assist_pipeline,
    typing_assist_pipeline_for_auto_replace, DEFAULT_TYPING_ASSIST_RULES,
};
use lay::correction::Correction;
use lay::decoder::{decode_manual_tail, CorrectionSource, DecoderAction, ManualDecodeRequest};
use lay::desktop::{
    is_ru_layout_id, normalize_layout_id, parse_setxkbmap_layout, resolve_layout_backend,
};
use lay::keyboard::{
    is_cyrillic_letter, is_layout_decision_key, keycode_to_ru_char, keycode_to_us_char,
    map_events_to_layout, map_opposite_events, map_original_events, preferred_layout_for_text,
    replay_layout_decision, split_event_words, text_to_uinput_runs, KeyEvent, ReplayLayoutDecision,
};
use lay::text_edit::{plan_committed_tail_replacement, plan_text_replacement, TextReplacement};
use lay::typing_assist::{
    apply_typing_assist_with_pipeline, are_ru_keyboard_neighbors,
    correct_duplicate_layout_prefix_on_ascii_token, correct_extra_letters, correct_missing_letter,
    correct_wrong_layout_ascii_technical_token, decide_completed_scope_word, decide_correction,
    decide_scoped_tail_correction, decide_scoped_tail_correction_with_lem, effective_replace_words,
    is_ascii_technical_token, is_known_russian_word_or_form, promoted_replacement_for_token,
    remember_promoted_replacement, repair_cyrillic_prefix_before_ascii_tail,
    russian_generated_form_dictionary, scoped_tail_lem_candidates,
    should_force_replay_for_short_fragment, should_keep_plain_cyrillic_before_ascii_technical,
    split_edge_whitespace, split_ws_segments, ScopedTailOptions,
};
use lay::word_buffer::{UserLearningCorrection, WordBuffer};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

fn seed_test_replacements() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for row in fixture_rows("daemon_seed_replacements.tsv") {
            assert_eq!(row.len(), 2, "seed replacement fixture must be TSV");
            remember_promoted_replacement(&row[0], &row[1]);
        }
    });
}

fn fixture_rows(name: &str) -> Vec<Vec<String>> {
    let path = fixture_path(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| line.split('\t').map(decode_fixture_field).collect())
        .collect()
}

fn fixture_lines(name: &str) -> Vec<String> {
    let path = fixture_path(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(decode_fixture_field)
        .collect()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn decode_fixture_field(value: &str) -> String {
    value.replace("\\s", " ")
}

fn apply_typing_assist_exact(text: &str) -> Option<String> {
    seed_test_replacements();
    lay::typing_assist::apply_typing_assist_exact(text)
}

fn apply_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    seed_test_replacements();
    lay::typing_assist::apply_typing_assist(text, allow_layout_auto)
}

fn apply_auto_replace(original: &str, target: &str) -> Option<String> {
    seed_test_replacements();
    lay::typing_assist::apply_auto_replace(original, target)
}

fn key_event(key: KeyCode, layout_is_ru: bool) -> KeyEvent {
    KeyEvent {
        keycode: key.code(),
        shift: false,
        layout_is_ru,
    }
}

fn push_keys(buffer: &mut WordBuffer, keys: &[KeyCode], layout_is_ru: bool) {
    for key in keys {
        buffer.push(key_event(*key, layout_is_ru));
    }
}

fn key_events(keys: &[KeyCode], layout_is_ru: bool) -> Vec<KeyEvent> {
    keys.iter()
        .map(|key| key_event(*key, layout_is_ru))
        .collect()
}

#[path = "tests/learning.rs"]
mod learning;
#[path = "tests/scoped_tail.rs"]
mod scoped_tail;
#[path = "tests/typing_assist_rules.rs"]
mod typing_assist_rules;

#[test]
fn idle_wait_uses_long_sleep_when_no_internal_deadlines() {
    let now = Instant::now();

    assert_eq!(
        idle_wait_timeout_at(now, None, now, Duration::from_millis(120)),
        Duration::from_millis(IDLE_EVENT_WAIT_MAX_MS)
    );
}

#[test]
fn idle_wait_keeps_multi_tap_deadline_precise() {
    let now = Instant::now();
    let pending = MultiTapPending {
        tap_count: 2,
        last_release: now - Duration::from_millis(80),
    };

    assert_eq!(
        idle_wait_timeout_at(now, Some(&pending), now, Duration::from_millis(120)),
        Duration::from_millis(40)
    );
}

#[test]
fn idle_wait_returns_zero_when_a_deadline_is_due() {
    let now = Instant::now();
    let pending = MultiTapPending {
        tap_count: 2,
        last_release: now - Duration::from_millis(120),
    };

    assert_eq!(
        idle_wait_timeout_at(now, Some(&pending), now, Duration::from_millis(120),),
        Duration::ZERO
    );
}

#[test]
fn shift_state_cleanup_after_trigger_keeps_shortcuts_but_drops_caps() {
    let mut state = ShiftState::default();
    state.update(KeyCode::KEY_LEFTSHIFT, 1);
    state.update(KeyCode::KEY_RIGHTSHIFT, 1);
    state.update(KeyCode::KEY_LEFTCTRL, 1);

    assert!(state.any());
    assert!(state.shortcut_active());

    state.clear_shifts();

    assert!(!state.any());
    assert!(state.shortcut_active());
}

fn ascii_hyphen_token_keycodes() -> [KeyCode; 5] {
    [
        KeyCode::KEY_W,
        KeyCode::KEY_I,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_F,
        KeyCode::KEY_I,
    ]
}

fn typing_pipeline_with_disabled(disabled: &[&str]) -> Vec<TypingAssistRuleConfig> {
    default_typing_assist_pipeline()
        .into_iter()
        .map(|mut rule| {
            if disabled.iter().any(|id| *id == rule.id) {
                rule.enabled = false;
            }
            rule
        })
        .collect()
}

fn typing_pipeline_with_only(enabled: &str) -> Vec<TypingAssistRuleConfig> {
    default_typing_assist_pipeline()
        .into_iter()
        .map(|mut rule| {
            rule.enabled = rule.id == enabled;
            rule
        })
        .collect()
}

fn typing_pipeline_with_first(first: &str) -> Vec<TypingAssistRuleConfig> {
    let mut rules = default_typing_assist_pipeline();
    for rule in &mut rules {
        rule.priority += 10;
        if rule.id == first {
            rule.priority = 1;
        }
    }
    rules
}

#[test]
fn text_insert_runs_use_uinput_layout_channels() {
    for row in fixture_rows("daemon_text_insert_runs.tsv") {
        assert_eq!(row.len(), 4, "text insert fixture must be TSV");
        let default_layout_is_ru = row[1] == "ru";
        if row[2] == "none" {
            assert!(text_to_uinput_runs(&row[0], default_layout_is_ru).is_none());
            continue;
        }

        let expected_targets: Vec<bool> = row[2].split(',').map(|part| part == "ru").collect();
        let expected_outputs: Vec<&str> = row[3].split('|').collect();
        let runs = text_to_uinput_runs(&row[0], default_layout_is_ru).expect("typable text");
        assert_eq!(runs.len(), expected_targets.len());
        assert_eq!(runs.len(), expected_outputs.len());
        for (idx, run) in runs.iter().enumerate() {
            assert_eq!(run.target_is_ru, expected_targets[idx], "row={row:?}");
            assert_eq!(
                map_events_to_layout(&run.events, run.target_is_ru),
                expected_outputs[idx],
                "row={row:?}"
            );
        }
    }
}

#[test]
fn typing_assist_minimal_plan_keeps_inter_word_space() {
    let row = fixture_rows("daemon_typing_assist_minimal_plan.tsv")
        .into_iter()
        .next()
        .expect("minimal plan fixture");
    let plan = plan_text_replacement(&row[0], &row[1]).expect("replacement");

    assert_eq!(plan.move_left, 1);
    assert_eq!(plan.backspaces, 1);
    assert_eq!(plan.insert, "о");
    assert_eq!(plan.move_right, 1);
}

#[test]
fn replacement_memory_keeps_space_boundary_after_i_autofix() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "double b ", false);
    let events = buffer
        .last_completed_words_events(2)
        .expect("completed two-word tail");
    let original = map_original_events(&events);
    let replacement = "double и ";
    let plan = plan_committed_tail_replacement(&original, replacement).expect("replacement");

    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, replacement));
    assert!(buffer.current_is_empty());
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(buffer.prev_words_len(), 1);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("prev word")),
        "и"
    );

    push_text_as_layout(&mut buffer, "слово", true);
    let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&tail), "и слово");
}

#[test]
fn replacement_memory_synthesizes_last_word_after_glued_phrase_split() {
    let mut buffer = WordBuffer::new();
    let row = fixture_rows("daemon_replacement_memory_glued.tsv")
        .into_iter()
        .next()
        .expect("replacement memory fixture");
    push_text_as_layout(&mut buffer, &row[0], true);
    let events = buffer
        .last_completed_words_events(1)
        .expect("completed one-word tail");
    let original = map_original_events(&events);
    let replacement = &row[1];
    let plan = plan_committed_tail_replacement(&original, replacement).expect("replacement");

    assert_eq!(original, row[0]);
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 6,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 6,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, replacement));
    assert!(buffer.current_is_empty());
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(buffer.prev_words_len(), 2);
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("first prev word")),
        row[2]
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("second prev word")),
        row[3]
    );

    push_text_as_layout(&mut buffer, &row[4], true);
    let (tail, _) = buffer.what_to_replay(2).expect("two-word tail");
    assert_eq!(map_original_events(&tail), row[5]);
}

#[test]
fn replacement_memory_can_update_completed_words_without_dropping_current_word() {
    let mut buffer = WordBuffer::new();
    let row = fixture_rows("daemon_replacement_memory_completed.tsv")
        .into_iter()
        .next()
        .expect("replacement completed fixture");
    push_text_as_layout(&mut buffer, &row[0], true);
    push_text_as_layout(&mut buffer, &row[4], true);

    assert!(buffer.remember_completed_replacement_words_for_replay(&row[1]));
    assert_eq!(buffer.prev_words_len(), 2);
    assert!(buffer.prev_had_trailing_space());
    assert_eq!(
        map_original_events(buffer.prev_word_events(0).expect("first prev")),
        row[2]
    );
    assert_eq!(
        map_original_events(buffer.prev_word_events(1).expect("second prev")),
        row[3]
    );
    assert_eq!(buffer.current_len(), 6);

    let (tail, _) = buffer.what_to_replay(1).expect("current word tail");
    assert_eq!(map_original_events(&tail), row[4]);
}

#[test]
fn enter_autocorrect_candidate_is_off_contract_until_enabled_by_config() {
    let cfg = LayConfig::default();
    assert!(!cfg.enter_autocorrect);
    assert!(!active_enter_autocorrect_from_env(false, None));
    assert!(active_enter_autocorrect_from_env(true, None));
    assert!(!active_enter_autocorrect_from_env(true, Some("0")));
    assert!(active_enter_autocorrect_from_env(true, Some("1")));
    assert!(active_enter_autocorrect_from_env(true, Some("true")));
}

#[test]
fn enter_autocorrect_candidate_rejects_plain_layout_word_guess() {
    let pipeline = typing_pipeline_with_only("layout_en_to_ru");

    for input in ["ghbdtn", "lfkmit"] {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, input, false);

        assert!(
            enter_autocorrect_candidate(&buffer, 1, true, &pipeline).is_none(),
            "plain layout words are not safe enough for Enter autocorrect: {input}"
        );
    }
}

#[test]
fn enter_autocorrect_candidate_keeps_normal_english_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);
    let pipeline = typing_pipeline_with_only("layout_en_to_ru");

    assert!(enter_autocorrect_candidate(&buffer, 1, true, &pipeline).is_none());
}

#[test]
fn enter_autocorrect_candidate_can_use_completed_tail_scope() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "double", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "b", false);
    let pipeline = typing_pipeline_with_only("visual_b");

    let (_events, edit) =
        enter_autocorrect_candidate(&buffer, 2, true, &pipeline).expect("correction");

    assert_eq!(edit.original, "double b");
    assert_eq!(edit.replacement, "double и");
}

fn push_key_events(buffer: &mut WordBuffer, keys: &[(KeyCode, bool)], layout_is_ru: bool) {
    for (key, shift) in keys {
        buffer.push(KeyEvent {
            keycode: key.code(),
            shift: *shift,
            layout_is_ru,
        });
    }
}

fn text_key_event(ch: char, layout_is_ru: bool) -> KeyEvent {
    const KEYS: &[KeyCode] = &[
        KeyCode::KEY_A,
        KeyCode::KEY_B,
        KeyCode::KEY_C,
        KeyCode::KEY_D,
        KeyCode::KEY_E,
        KeyCode::KEY_F,
        KeyCode::KEY_G,
        KeyCode::KEY_H,
        KeyCode::KEY_I,
        KeyCode::KEY_J,
        KeyCode::KEY_K,
        KeyCode::KEY_L,
        KeyCode::KEY_M,
        KeyCode::KEY_N,
        KeyCode::KEY_O,
        KeyCode::KEY_P,
        KeyCode::KEY_Q,
        KeyCode::KEY_R,
        KeyCode::KEY_S,
        KeyCode::KEY_T,
        KeyCode::KEY_U,
        KeyCode::KEY_V,
        KeyCode::KEY_W,
        KeyCode::KEY_X,
        KeyCode::KEY_Y,
        KeyCode::KEY_Z,
        KeyCode::KEY_1,
        KeyCode::KEY_2,
        KeyCode::KEY_3,
        KeyCode::KEY_4,
        KeyCode::KEY_5,
        KeyCode::KEY_6,
        KeyCode::KEY_7,
        KeyCode::KEY_8,
        KeyCode::KEY_9,
        KeyCode::KEY_0,
        KeyCode::KEY_SEMICOLON,
        KeyCode::KEY_APOSTROPHE,
        KeyCode::KEY_COMMA,
        KeyCode::KEY_DOT,
        KeyCode::KEY_LEFTBRACE,
        KeyCode::KEY_RIGHTBRACE,
        KeyCode::KEY_GRAVE,
        KeyCode::KEY_SLASH,
        KeyCode::KEY_BACKSLASH,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_EQUAL,
    ];

    for key in KEYS {
        for shift in [false, true] {
            let mapped = if layout_is_ru {
                keycode_to_ru_char(key.code(), shift)
            } else {
                keycode_to_us_char(key.code(), shift)
            };
            if mapped == Some(ch) {
                return KeyEvent {
                    keycode: key.code(),
                    shift,
                    layout_is_ru,
                };
            }
        }
    }

    panic!("no key event for {ch:?} in layout_is_ru={layout_is_ru}");
}

fn push_text_as_layout(buffer: &mut WordBuffer, text: &str, layout_is_ru: bool) {
    for ch in text.chars() {
        if ch == ' ' {
            buffer.handle_space();
        } else {
            buffer.push(text_key_event(ch, layout_is_ru));
        }
    }
}

fn assert_smart_pair(
    left: &str,
    left_layout_is_ru: bool,
    current_typed: &str,
    current_layout_is_ru: bool,
    expected: &str,
) {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, left, left_layout_is_ru);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, current_typed, current_layout_is_ru);
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let got = decide_scoped_tail_correction(&events).unwrap_or(original.clone());

    assert_eq!(got, expected, "original tail: {original:?}");
}

fn map_target_events(events: &[KeyEvent], target_is_ru: bool) -> String {
    events
        .iter()
        .filter_map(|ev| {
            if target_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect()
}

fn apply_typing_assist_to_text_tail(text: &str) -> Option<String> {
    apply_typing_assist_exact(text).or_else(|| {
        let (leading, core, trailing) = split_edge_whitespace(text);
        let segments = split_ws_segments(core);
        if segments.len() < 3 {
            return None;
        }

        for word_count in [2, 1] {
            let mut suffix_start = core.len();
            let mut non_ws_seen = 0;
            for (segment, is_ws) in segments.iter().rev() {
                suffix_start -= segment.len();
                if !is_ws {
                    non_ws_seen += 1;
                    if non_ws_seen == word_count {
                        break;
                    }
                }
            }

            let prefix = &core[..suffix_start];
            let suffix = &core[suffix_start..];
            if let Some(replacement) = apply_typing_assist_exact(&format!("{suffix}{trailing}")) {
                return Some(format!("{leading}{prefix}{replacement}"));
            }
        }

        None
    })
}

#[test]
fn parses_gdbus_string_tuple() {
    assert_eq!(parse_gdbus_string("('us',)"), Some("us".to_string()));
}

#[test]
fn parses_current_layout_from_list_layouts_reply() {
    assert_eq!(
        parse_current_layout_from_list("('0:xkb:us,1:xkb:ru*',)"),
        Some("ru".to_string())
    );
}

#[test]
fn parses_kde6_layout_list_reply() {
    let reply = r#"[Argument: a(sss) {[Argument: (sss) "us", "", "English (US)"], [Argument: (sss) "ru", "", "Russian"]}]"#;
    assert_eq!(parse_kde_layouts_list(reply), vec!["us", "ru"]);
}

#[test]
fn parses_first_quoted_string_with_escapes() {
    assert_eq!(
        first_quoted_string(r#" "us\"intl", "", "English" "#),
        Some(r#"us"intl"#.to_string())
    );
}

#[test]
fn marks_current_word_after_replay_for_next_toggle() {
    let mut buffer = WordBuffer::new();
    for key in [
        KeyCode::KEY_D,
        KeyCode::KEY_H,
        KeyCode::KEY_T,
        KeyCode::KEY_V,
        KeyCode::KEY_Z,
    ] {
        buffer.push(KeyEvent {
            keycode: key.code(),
            shift: false,
            layout_is_ru: false,
        });
    }

    buffer.mark_replayed_layout(1, true);
    let (events, _) = buffer.what_to_replay(1).expect("word is buffered");

    assert!(events.iter().all(|event| event.layout_is_ru));
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn short_fragments_force_replay_without_llm() {
    assert!(should_force_replay_for_short_fragment("N"));
    assert!(should_force_replay_for_short_fragment("gh"));
    assert!(should_force_replay_for_short_fragment("т"));
    assert!(!should_force_replay_for_short_fragment("ghb"));
    assert!(!should_force_replay_for_short_fragment("a b"));
    assert!(!should_force_replay_for_short_fragment(""));
}

#[test]
fn typing_assist_after_space_is_suppressed_once_after_manual_replay() {
    let mut suppress_once = true;

    assert!(!should_schedule_typing_assist_after_space(
        true,
        &mut suppress_once
    ));
    assert!(!suppress_once);
    assert!(should_schedule_typing_assist_after_space(
        true,
        &mut suppress_once
    ));
    assert!(!should_schedule_typing_assist_after_space(
        false,
        &mut suppress_once
    ));
}

#[test]
fn typing_assist_runs_on_space_release_when_pending() {
    assert!(should_run_typing_assist_on_space_release(
        true, true, false, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        false, true, false, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        true, false, false, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        true, true, true, false
    ));
    assert!(!should_run_typing_assist_on_space_release(
        true, true, false, true
    ));
}

#[test]
fn typing_assist_drops_stale_previous_word_when_current_word_started() {
    assert!(!should_drop_stale_typing_assist_after_space(false, 3));
    assert!(!should_drop_stale_typing_assist_after_space(true, 0));
    assert!(should_drop_stale_typing_assist_after_space(true, 3));
}

#[test]
fn leading_cli_option_token_is_ignored_until_space() {
    for (leader, leader_shift, token_key, next_word) in [
        (KeyCode::KEY_MINUS, false, KeyCode::KEY_B, "feature"),
        (KeyCode::KEY_EQUAL, true, KeyCode::KEY_X, "script"),
    ] {
        let mut modifiers = ShiftState::default();
        modifiers.update(KeyCode::KEY_LEFTSHIFT, i32::from(leader_shift));
        let mut buffer = WordBuffer::new();
        let mut ignore_token =
            should_start_ignored_buffer_token(leader, &modifiers, buffer.current_is_empty());
        assert!(ignore_token);

        if !ignore_token {
            buffer.push(key_event(token_key, false));
        }
        assert!(buffer.current_is_empty());

        if ignore_token {
            ignore_token = false;
        } else {
            buffer.handle_space();
        }
        assert!(!ignore_token);
        assert!(!buffer.prev_had_trailing_space());

        push_text_as_layout(&mut buffer, next_word, false);
        let (events, _) = buffer.what_to_replay(1).expect("word");
        assert_eq!(map_original_events(&events), next_word);
    }
}

#[test]
fn config_replace_words_is_independent_from_engine_mode() {
    let simple = LayConfig {
        mode: "simple".to_string(),
        correction_engine: Some("replay".to_string()),
        replace_words: 2,
        ..LayConfig::default()
    };
    let smart = LayConfig {
        mode: "simple".to_string(),
        correction_engine: Some("smart".to_string()),
        replace_words: 2,
        ..LayConfig::default()
    };

    assert_eq!(simple.active_replace_words(), 2);
    assert_eq!(smart.active_replace_words(), 2);
    assert_eq!(simple.active_correction_engine(), CorrectionEngine::Replay);
    assert_eq!(smart.active_correction_engine(), CorrectionEngine::Smart);
}

#[test]
fn force_layout_hotkeys_use_single_key_ids_only() {
    assert_eq!(
        single_hotkey_keycode("single-rctrl"),
        Some(KeyCode::KEY_RIGHTCTRL)
    );
    assert_eq!(
        single_hotkey_keycode("single-ralt"),
        Some(KeyCode::KEY_RIGHTALT)
    );
    assert_eq!(
        single_hotkey_keycode("caps-lock"),
        Some(KeyCode::KEY_CAPSLOCK)
    );
    assert_eq!(single_hotkey_keycode("double-lshift"), None);
    assert_eq!(single_hotkey_keycode(""), None);
}

#[test]
fn multi_tap_scope_design_contract_maps_taps_to_scope() {
    assert_eq!(multi_tap_scope_for_taps(0), None);
    assert_eq!(multi_tap_scope_for_taps(1), None);
    assert_eq!(multi_tap_scope_for_taps(2), Some(1));
    assert_eq!(multi_tap_scope_for_taps(3), Some(2));
    assert_eq!(multi_tap_scope_for_taps(4), Some(3));
    assert_eq!(multi_tap_scope_for_taps(5), Some(3));
}

#[test]
fn layout_backend_can_be_explicit_or_auto_detected() {
    assert_eq!(
        resolve_layout_backend("gnome", Some("KDE"), None, Some("wayland")),
        LayoutBackend::Gnome
    );
    assert_eq!(
        resolve_layout_backend("kde", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::Kde
    );
    assert_eq!(
        resolve_layout_backend("x11", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::X11
    );
    assert_eq!(
        resolve_layout_backend("auto", Some("KDE"), Some("plasma"), Some("wayland")),
        LayoutBackend::Kde
    );
    assert_eq!(
        resolve_layout_backend("auto", Some("GNOME"), None, Some("wayland")),
        LayoutBackend::Gnome
    );
    assert_eq!(
        resolve_layout_backend("auto", None, None, Some("x11")),
        LayoutBackend::X11
    );
}

#[test]
fn parses_x11_layout_tool_output() {
    assert_eq!(
        parse_setxkbmap_layout("rules: evdev\nmodel: pc105\nlayout: us,ru\n"),
        Some("us".to_string())
    );
    assert_eq!(normalize_layout_id(" ru\n"), "ru");
    assert_eq!(normalize_layout_id("xkb:ru::rus"), "ru");
    assert!(is_ru_layout_id("xkb:ru"));
    assert!(!is_ru_layout_id("xkb:us"));
}

#[test]
fn host_focus_ignore_detects_vm_windows() {
    assert!(focused_window_json_is_ignored(
        r#"{"appId":"org.virt-manager.virt-manager","wmClass":"virt-manager","title":"KDE VM"}"#
    ));
    assert!(focused_window_json_is_ignored(
        r#"{"appId":"remote-viewer.desktop","wmClass":"remote-viewer","title":"SPICE display"}"#
    ));
    assert!(focused_window_json_is_ignored(
        r#"{"appId":"python3","wmClass":"python3","title":"lay-kde-test SPICE clipboard ON"}"#
    ));
    assert!(!focused_window_json_is_ignored(
        r#"{"appId":"org.gnome.Terminal.desktop","wmClass":"org.gnome.Terminal","title":"Terminal"}"#
    ));
}

#[test]
fn keyboard_discovery_ignores_service_virtual_devices() {
    assert!(should_ignore_keyboard_device_name("lay-virtual-keyboard"));
    assert!(should_ignore_keyboard_device_name(
        "ydotoold virtual device"
    ));
    assert!(!should_ignore_keyboard_device_name(
        "AT Translated Set 2 keyboard"
    ));
}

#[test]
fn config_allows_three_word_scope() {
    let cfg = LayConfig {
        replace_words: 3,
        ..LayConfig::default()
    };
    assert_eq!(cfg.active_replace_words(), 3);

    let too_large = LayConfig {
        replace_words: 8,
        ..LayConfig::default()
    };
    assert_eq!(too_large.active_replace_words(), 3);
}

#[test]
fn auto_switch_layout_is_enabled_by_default() {
    assert!(LayConfig::default().auto_switch_layout);
}

#[test]
fn lem_scope_flags_are_enabled_by_default() {
    let cfg = LayConfig::default();
    assert!(!cfg.lem_enabled_for_scope(1));
    assert!(cfg.lem_enabled_for_scope(2));
    assert!(cfg.lem_enabled_for_scope(3));
    assert!(cfg.lem_enabled_for_scope(8));
    assert_eq!(
        cfg.active_typing_assist_pipeline().len(),
        DEFAULT_TYPING_ASSIST_RULES.len()
    );
}

#[test]
fn legacy_llm_mode_maps_to_smart_only_without_explicit_engine() {
    let legacy = LayConfig {
        mode: "llm".to_string(),
        correction_engine: None,
        ..LayConfig::default()
    };
    let explicit_replay = LayConfig {
        mode: "llm".to_string(),
        correction_engine: Some("replay".to_string()),
        ..LayConfig::default()
    };

    assert_eq!(legacy.active_correction_engine(), CorrectionEngine::Smart);
    assert_eq!(
        explicit_replay.active_correction_engine(),
        CorrectionEngine::Replay
    );
}

#[test]
fn typing_after_replay_clears_toggle_shortcut() {
    let mut buffer = WordBuffer::new();
    buffer.push(KeyEvent {
        keycode: KeyCode::KEY_D.code(),
        shift: false,
        layout_is_ru: false,
    });
    buffer.mark_replayed_layout(1, true);

    buffer.push(KeyEvent {
        keycode: KeyCode::KEY_H.code(),
        shift: false,
        layout_is_ru: true,
    });

    assert!(!buffer.replay_toggle_ready());
}

#[test]
fn parses_gdbus_bool_tuple() {
    assert_eq!(parse_gdbus_bool("(true,)"), Some(true));
    assert_eq!(parse_gdbus_bool("(false,)"), Some(false));
    assert_eq!(parse_gdbus_bool("true"), None);
}

#[test]
fn keeps_only_last_jsonl_lines() {
    let compacted = keep_last_jsonl_lines("a\nb\nc\nd\n", 2);
    assert_eq!(compacted, "c\nd\n");
}
