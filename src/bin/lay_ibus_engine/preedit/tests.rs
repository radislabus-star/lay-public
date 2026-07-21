use super::*;
use lay::config::LayConfig;
use std::sync::{Arc, Mutex};

#[test]
fn candidate_target_survives_suffix_shrink_while_typing() {
    let candidates = vec!["ст".to_string(), "рошо".to_string()];

    assert_eq!(
        candidate_index_for_target("хвост", "хво", &candidates),
        Some(0)
    );
    assert_eq!(
        candidate_index_for_target("хорошо", "хор", &["ошо".to_string(), "ма".to_string()]),
        Some(0)
    );
}

#[test]
fn candidate_target_is_released_when_new_input_invalidates_it() {
    assert_eq!(
        candidate_index_for_target("хвалить", "хво", &["ст".to_string(), "ровать".to_string()]),
        None
    );
}

#[test]
fn candidate_target_preserves_nonzero_selection_by_surface() {
    assert_eq!(
        candidate_index_for_target(
            "проверка",
            "прове",
            &["рить".to_string(), "рка".to_string(), "дение".to_string()]
        ),
        Some(1)
    );
}

#[test]
fn typed_continuation_releases_auto_target_but_keeps_learning_observation() {
    let mut fast = PreeditFastState::default();
    fast.remember_target(Some("перезагрузка".to_string()));
    fast.observe_prediction_target(Some("перезагрузка".to_string()));

    fast.push('з');

    assert_eq!(fast.target_surface(), None);
    assert_eq!(fast.observed_prediction_target(), Some("перезагрузка"));
}

#[test]
fn invalidated_target_retargets_to_fresh_top_candidate_without_blank_frame() {
    let candidates = vec!["ст".to_string(), "ровать".to_string()];

    assert_eq!(
        stable_candidate_index(Some("хвалить"), "хво", &candidates),
        0
    );
}

#[test]
fn whitespace_cancels_pending_inactive_preedit_flush() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );

    engine.push_tail_char('п');
    engine.preedit_dirty = true;
    engine.preedit_suffix = "ривет".to_string();
    engine.preedit_candidates = vec!["ривет".to_string(), "роект".to_string()];
    engine.preedit_candidate_index = 1;
    engine.push_tail_char(' ');

    assert!(
        !engine.preedit_dirty,
        "word boundary must not resurrect previous word suffix on cursor flush"
    );
    assert_eq!(engine.preedit_fast.token(), "");
    assert!(engine.preedit_suffix.is_empty());
    assert!(engine.preedit_candidates.is_empty());
    assert_eq!(engine.preedit_candidate_index, 0);
}

#[test]
fn ignored_preedit_candidate_records_negative_usage_without_promoting_it() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let events_path = std::env::temp_dir().join(format!(
        "lay-ime-usage-events-{}-{test_id}.jsonl",
        std::process::id()
    ));
    let counts_path = std::env::temp_dir().join(format!(
        "lay-ime-usage-counts-{}-{test_id}.json",
        std::process::id()
    ));
    std::env::set_var("LAY_NANDA_WORD_USAGE_EVENTS", &events_path);
    std::env::set_var("LAY_NANDA_WORD_USAGE_COUNTS", &counts_path);

    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "ну да".chars() {
        engine.push_tail_char(ch);
    }
    engine.preedit_suffix = "ша".to_string();
    engine.preedit_candidates = vec!["ша".to_string()];
    engine.push_tail_char(' ');

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1300);
    let text = loop {
        if let Ok(text) = std::fs::read_to_string(&events_path) {
            break text;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "usage persistence did not flush within its active interval"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert!(text.contains(r#""kind":"rejected_ime""#), "{text}");
    assert!(text.contains(r#""word":"даша""#), "{text}");
    assert!(!text.contains(r#""kind":"accepted_ime""#), "{text}");

    std::env::remove_var("LAY_NANDA_WORD_USAGE_EVENTS");
    std::env::remove_var("LAY_NANDA_WORD_USAGE_COUNTS");
    let _ = std::fs::remove_file(events_path);
    let _ = std::fs::remove_file(counts_path);
}

#[test]
fn manually_finished_visible_prediction_records_positive_usage() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let events_path = std::env::temp_dir().join(format!(
        "lay-ime-usage-events-{}-{test_id}.jsonl",
        std::process::id()
    ));
    let counts_path = std::env::temp_dir().join(format!(
        "lay-ime-usage-counts-{}-{test_id}.json",
        std::process::id()
    ));
    std::env::set_var("LAY_NANDA_WORD_USAGE_EVENTS", &events_path);
    std::env::set_var("LAY_NANDA_WORD_USAGE_COUNTS", &counts_path);

    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "ну д".chars() {
        engine.push_tail_char(ch);
    }
    engine.preedit_suffix = "а".to_string();
    engine.preedit_candidates = vec!["а".to_string()];
    engine
        .preedit_fast
        .observe_prediction_target(Some("да".to_string()));
    engine.push_tail_char('а');
    assert_eq!(
        engine.preedit_fast.observed_prediction_target(),
        Some("да"),
        "typing through a prediction must preserve its target until Space"
    );
    engine.push_tail_char(' ');

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1300);
    let text = loop {
        if let Ok(text) = std::fs::read_to_string(&events_path) {
            break text;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "usage persistence did not flush within its active interval"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert!(
        text.contains(r#""kind":"confirmed_ime_prediction""#),
        "{text}"
    );
    assert!(text.contains(r#""word":"да""#), "{text}");
    assert!(!text.contains(r#""kind":"rejected_ime""#), "{text}");

    std::env::remove_var("LAY_NANDA_WORD_USAGE_EVENTS");
    std::env::remove_var("LAY_NANDA_WORD_USAGE_COUNTS");
    let _ = std::fs::remove_file(events_path);
    let _ = std::fs::remove_file(counts_path);
}

#[test]
fn russian_prefixes_delegate_to_shared_candidate_authority() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "нев".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .all(|suffix| format!("нев{suffix}").starts_with("нев")),
        "shared gate returned a non-prefix completion: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn russian_fast_lexical_prior_generates_contextual_suffix() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "я хочу пров".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.iter().any(|suffix| {
            let word = format!("пров{suffix}");
            word.starts_with("провер") || word.starts_with("прове")
        }),
        "expected contextual Russian wave candidates for 'я хочу пров', got {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn ime_precognition_projects_only_l2_completion_words_to_suffixes() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "звгрузи".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.iter().all(|suffix| {
            let projected = format!("звгрузи{suffix}");
            projected != "загрузи" && suffix != "агрузи"
        }),
        "replacement candidates must not leak into IME as suffix fragments: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn ambiguous_short_russian_prefix_does_not_emit_dictionary_noise() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "я без за".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .all(|suffix| !format!("за{suffix}").contains("запят")),
        "ambiguous prefix should not suggest project/chat noise: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn long_command_tail_does_not_emit_sentence_precognition() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "ЧИТАЙ ЛОГИ ПРОСТО ЦЕЛЫЕ ПРЕДЛОЖЕНИЯ АВТОКОРРЕКЦИ".chars()
    {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.is_empty(),
        "command-like uppercase sentence tail must not get noisy IME suffixes: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn short_prefixes_do_not_emit_wide_dictionary_noise() {
    lay::nanda_wave::warm_up_l2_for_ime();
    for input in ["про", "прочти л", "прочти ло"] {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in input.chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine.preedit_candidates.iter().all(|suffix| {
                !matches!(
                    suffix.as_str(),
                    "авило" | "ахать" | "алина" | "ббизм" | "арифм"
                )
            }),
            "short prefix {input:?} must not emit wide dictionary noise: {:?}",
            engine.preedit_candidates
        );
    }
}

#[test]
fn three_letter_russian_prefix_does_not_emit_long_lexical_tail_without_l3() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "интересно инт".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .all(|suffix| suffix != "алия"),
        "short prefix must not leak long dictionary-only tails: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn bracketed_mode_suppresses_three_letter_russian_lexical_noise() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ime_bracket_candidates: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "интересно инт".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.is_empty(),
        "bracket mode must not show weak three-letter Russian guesses: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn weak_single_russian_suffixes_are_not_visible_by_default() {
    for suffix in ["а", "в", "к", "о", "с", "у"] {
        assert!(
            !is_allowed_visible_completion_suffix(suffix),
            "weak single suffix {suffix:?} must need stronger candidate authority"
        );
    }
    assert!(is_allowed_visible_completion_suffix("и"));
    assert!(is_allowed_visible_completion_suffix("я"));
    assert!(is_allowed_visible_completion_suffix("ть"));
}

#[test]
fn known_russian_word_does_not_get_extended_by_precognition() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ime_bracket_candidates: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "просто просто".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.is_empty(),
        "known word must not be extended by weak suffixes: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn punctuation_closes_inactive_completion_for_previous_word() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ime_bracket_candidates: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "Читал логи".chars() {
        engine.push_tail_char(ch);
    }
    engine.preedit_suffix = "ка".to_string();
    engine.preedit_candidates = vec!["ка".to_string()];
    engine.push_tail_char('?');
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.is_empty(),
        "punctuation must not revive completion for a closed word: {:?}",
        engine.preedit_candidates
    );
    assert_eq!(engine.preedit_fast.token(), "");
    assert_eq!(engine.preedit_suffix, "");
}

#[test]
fn short_russian_prefix_stays_fast_without_dropping_valid_candidates() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "ка сло".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .all(|suffix| suffix.chars().count() != 1
                || is_allowed_visible_completion_suffix(suffix)),
        "short prefix candidates must keep the single-letter guard: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn four_letter_russian_prefix_can_use_wave_lookup() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "ка слов".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .all(|suffix| suffix.chars().count() != 1
                || is_allowed_visible_completion_suffix(suffix)),
        "single-letter suffix guard must still apply: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn cold_english_wave_memory_does_not_block_precognition() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "this exam".chars() {
        engine.push_tail_char(ch);
    }
    let started = std::time::Instant::now();
    engine.refresh_precognition_candidates();
    let elapsed_us = started.elapsed().as_micros();

    if std::env::var_os("LAY_ENFORCE_IME_LATENCY_BUDGET").is_some() {
        assert!(
            elapsed_us < 5_000,
            "cold English wave memory must not block IME, took {elapsed_us}us"
        );
    }
}

#[test]
fn ascii_known_word_completion_allows_single_letter_suffix() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut fast = PreeditFastState::default();
    for ch in "exi".chars() {
        fast.push(ch);
    }

    let candidates = fast.ascii_candidates(16, 8);

    assert_eq!(
        candidates
            .first()
            .map(|candidate| candidate.suffix.as_str()),
        Some("t"),
        "candidates={candidates:?}"
    );
    assert!(
        !candidates.iter().any(|candidate| candidate.suffix == "il"),
        "known technical completion must outrank noisy wave suffixes: {candidates:?}"
    );
}

#[test]
fn long_russian_prefix_only_holds_prefix_preserving_suffix() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "normal".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "следую".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();
    assert!(
        engine.preedit_candidates.iter().all(|suffix| {
            let word = format!("следую{suffix}");
            word.starts_with("следую")
        }),
        "long prefix suffixes must be prefix-preserving: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn normal_composition_preedit_completes_raw_russian_prefix() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "normal".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "пров".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition_cursor = engine.buffer.chars().count();
    engine.refresh_precognition_candidates();
    let (text, cursor_pos) = engine.composition_preedit_payload();

    assert!(
        text.starts_with("пров") && text.chars().count() > "пров".chars().count(),
        "normal IME should show an aggressive completion for raw Russian prefix: text={text:?}, candidates={:?}, replacements={:?}",
        engine.preedit_candidates,
        engine.preedit_replacement_targets,
    );
    assert_eq!(cursor_pos, 4);
    assert!(!engine.preedit_suffix.is_empty());
}

#[test]
fn space_boundary_suppresses_inactive_phrase_precognition() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "я хочу ".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.is_empty(),
        "word boundary must close visible IME suffixes, got {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn composition_preedit_keeps_visible_suffix_when_autocorrect_is_pending() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            nanda_precognition: true,
            correction_safety: "normal".to_string(),
            ..LayConfig::default()
        },
    );
    engine.buffer = "ghbdtn".to_string();
    engine.composition_cursor = engine.buffer.chars().count();
    engine.preedit_candidates = vec!["ий".to_string()];
    engine.preedit_replacement_targets = vec![None];
    let (text, cursor_pos) = engine.composition_preedit_payload();

    assert_eq!(text, "ghbdtnий");
    assert_eq!(cursor_pos, 6);
    assert_eq!(engine.preedit_suffix, "ий");
}

#[test]
fn composition_preedit_renders_typed_replacement_as_the_full_token() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    engine.buffer = "рабоает".to_string();
    engine.composition_cursor = engine.buffer.chars().count();
    engine.preedit_candidates = vec!["работает".to_string()];
    engine.preedit_replacement_targets = vec![Some("работает".to_string())];

    let (text, cursor_pos) = engine.composition_preedit_payload();

    assert_eq!(text, "работает");
    assert_eq!(cursor_pos, "работает".chars().count() as u32);
    assert!(engine.preedit_suffix.is_empty());
}

#[test]
fn active_composition_requires_preedit_clear_even_without_suffix() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        },
    );
    engine.buffer = "ghbdtn".to_string();
    engine.preedit_suffix.clear();
    engine.preedit_candidates.clear();

    assert!(engine.preedit_clear_needed());
}

#[test]
fn mid_sentence_short_prefix_delegates_to_shared_candidate_gate() {
    let readout = include_str!("../preedit_readout.rs");
    assert!(
        !readout.contains("has_left_context\n            && partial_len <= 3"),
        "IME must not suppress a shared L2/L3/L4 candidate only because it is inside a phrase"
    );
}

#[test]
fn experimental_short_russian_prefix_gets_lexical_candidates() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "смотрим что будет происходить когда при".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        !engine.preedit_candidates.is_empty(),
        "experimental L2 should not stay silent for contextual prefix 'при'"
    );
}

#[test]
fn russian_single_letter_case_suffix_can_complete_unknown_prefix() {
    let mut candidates = Vec::new();
    push_unique_ru_known_suffix(&mut candidates, "буде", "будет", Some("т".to_string()));

    assert!(
        candidates.iter().any(|suffix| suffix == "т"),
        "unknown Russian prefix should allow strong one-letter completion: {candidates:?}"
    );
}

#[test]
fn first_russian_word_prefix_gets_precognition_candidate() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "русс".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .any(|suffix| suffix == "кий" || suffix == "ких"),
        "first Russian prefix should produce a useful word suffix: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn quoted_russian_prefix_gets_precognition_candidate() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "\"писа".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.iter().any(|suffix| {
            let word = format!("писа{suffix}");
            word == "писать" || word.starts_with("писа")
        }),
        "punctuation before Russian prefix must not silence IME: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn first_active_russian_word_prefix_gets_precognition_candidate_after_four_chars() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "пров".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition_cursor = engine.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.iter().any(|suffix| {
            let word = format!("пров{suffix}");
            word.starts_with("провер")
        }),
        "first active Russian word should produce a useful suffix after four chars: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn first_active_word_keeps_authorized_single_letter_completion_visible() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "писат".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition_cursor = engine.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.iter().any(|suffix| suffix == "ь"),
        "authorized final-letter completion must stay visible: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn complete_word_state_can_still_offer_a_stronger_longer_center() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "как".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition_cursor = engine.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .any(|suffix| matches!(suffix.as_str(), "ой" | "ие")),
        "Keep must compete with longer centers instead of stopping readout: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn live_ime_prefers_prefix_completion_over_semantic_replacement_noise() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "как будто нет кандидат".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.iter().all(|suffix| {
            let word = format!("кандидат{suffix}");
            word.starts_with("кандидат") && word != "кандидоз"
        }),
        "live IME must not turn a prefix into unrelated semantic replacement: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn live_ime_does_not_project_typo_replacement_as_suffix() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "звгрузи".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_candidates
            .iter()
            .all(|suffix| suffix != "агрузи"),
        "word replacement belongs to boundary autocorrect, not IME suffix: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn committed_tail_ime_renders_boundary_replacement_for_explicit_tab() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "тоесть".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_replacement_targets
            .iter()
            .flatten()
            .any(|target| target == "то есть"),
        "a proven BoundaryCell32 proposal must be visible for explicit Tab: candidates={:?}, replacements={:?}",
        engine.preedit_candidates,
        engine.preedit_replacement_targets
    );
}

#[test]
fn repeated_current_token_does_not_disable_shared_l2_readout() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "тоесть тоесть".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .preedit_replacement_targets
            .iter()
            .flatten()
            .any(|target| target == "то есть"),
        "a repeated token must still reach the shared L2 lattice: candidates={:?}, replacements={:?}",
        engine.preedit_candidates,
        engine.preedit_replacement_targets
    );
}

#[test]
fn first_active_russian_word_prefix_gets_precognition_candidate_after_three_chars() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "при".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition_cursor = engine.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        !engine.preedit_candidates.is_empty(),
        "first active Russian word should produce suffixes after three chars"
    );
}

#[test]
fn experimental_first_active_russian_word_prefix_gets_bayes_candidates_after_two_chars() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "пр".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition_cursor = engine.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        !engine.preedit_candidates.is_empty(),
        "experimental Bayes-backed IME should not stay silent after two Russian chars"
    );
}

#[test]
fn short_russian_prefix_prefers_informative_suffix_over_tiny_tail() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "мало рус".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    let first = engine
        .preedit_candidates
        .first()
        .map(String::as_str)
        .unwrap_or("");
    assert!(
        first.chars().count() > 2,
        "short Russian prefix should not rank tiny suffix first: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn ime_preserves_shared_gate_rank_after_admission_score_saturates() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "подсказка не оче".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert_eq!(
        engine.preedit_candidates.first().map(String::as_str),
        Some("нь"),
        "IBus must render the common completion selected by the shared gate: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn ime_does_not_render_unbound_generated_word_forms() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "в телеграме жуть".chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();

    assert!(
        engine.preedit_candidates.is_empty(),
        "an unbound generated surface must not become visible IME text: {:?}",
        engine.preedit_candidates
    );
}

#[test]
fn strict_precognition_keeps_short_suffix_limit() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "strict".to_string(),
            ..LayConfig::default()
        },
    );
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
    }
    assert_eq!(engine.precognition_suffix().as_deref(), None);
}

#[test]
fn experimental_phrase_readout_can_complete_partial_word() {
    let engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text(
        "идёт дом\nна улице опять идёт дождь\nна улице опять идёт снег",
    );
    let candidates = engine.llmwave_phrase_candidates_from_memory("На улице опять идёт д", &memory);

    assert_eq!(
        candidates
            .first()
            .map(|candidate| candidate.suffix.as_str()),
        Some("ождь")
    );
}

#[test]
fn experimental_precognition_can_suggest_next_word_from_l3_memory_after_space() {
    let engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text(
        "я хочу проверить подсказки\nя хочу проверить ввод",
    );

    let candidates = engine.llmwave_phrase_candidates_from_memory("я хочу ", &memory);

    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.suffix == "проверить"),
        "expected next-word L2 suffix from L3 memory, got {:?}",
        candidates
    );
}

#[test]
fn experimental_precognition_keeps_l3_word_after_user_started_it() {
    let engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text("у нас мало слов");

    let candidates = engine.llmwave_phrase_candidates_from_memory("у нас мало с", &memory);

    assert!(
        candidates.iter().any(|candidate| candidate.suffix == "лов"),
        "expected L3 suffix to survive started next word, got {:?}",
        candidates
    );
}

#[test]
fn experimental_precognition_uses_sentence_context_for_word_ending() {
    let engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text(
        "я хочу проверить подсказки\nя хочу проверить ввод",
    );

    let candidates = engine.llmwave_phrase_candidates_from_memory("я хочу пров", &memory);

    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.suffix == "ерить"),
        "expected sentence-aware word ending, got {:?}",
        candidates
    );
}

#[test]
fn phrase_candidate_suffix_preserves_word_boundary_before_space() {
    assert_eq!(
        phrase_candidate_suffix("я хочу", "я хочу проверить", 24).as_deref(),
        Some(" проверить")
    );
    assert_eq!(
        phrase_candidate_suffix("я хочу ", "я хочу проверить", 24).as_deref(),
        Some("проверить")
    );
}

#[test]
fn experimental_precognition_candidates_can_be_cycled() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "experimental".to_string(),
            ..LayConfig::default()
        },
    );
    engine.preedit_candidates = vec!["ождь".to_string(), "ождик".to_string()];
    assert!(
        engine.preedit_candidates.len() >= 2,
        "expected NANDA phrase candidates, got {:?}",
        engine.preedit_candidates
    );
    assert_eq!(
        engine.selected_precognition_suffix().as_deref(),
        Some("ождь")
    );
    assert!(engine.advance_precognition_candidate(1));
    assert_eq!(
        engine.selected_precognition_suffix().as_deref(),
        Some("ождик")
    );
    assert!(engine.advance_precognition_candidate(-1));
    assert_eq!(
        engine.selected_precognition_suffix().as_deref(),
        Some("ождь")
    );
}

#[test]
fn ime_backend_without_precognition_does_not_enable_probe_preedit() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: false,
            ..LayConfig::default()
        },
    );
    engine.tail_buffer = "ab".to_string();
    assert!(!engine.precognition_preedit_enabled());
    assert_eq!(engine.precognition_suffix(), None);
}

#[test]
fn ime_backend_with_zero_nanda_weights_does_not_show_precognition() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            nanda_l2_weight_percent: 0,
            nanda_l3_weight_percent: 0,
            ..LayConfig::default()
        },
    );
    engine.tail_buffer = "пров".to_string();
    for ch in "пров".chars() {
        engine.preedit_fast.push(ch);
    }

    assert!(!engine.precognition_preedit_enabled());
    engine.refresh_precognition_candidates();
    assert!(engine.preedit_candidates.is_empty());
    assert_eq!(engine.precognition_suffix(), None);
}

#[test]
fn preedit_for_plain_ime_client_hides_probe_marker() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    engine.tail_buffer = "ab".to_string();
    engine.preedit_suffix = PREEDIT_PROBE_SYMBOL.to_string();

    assert_eq!(engine.preedit_text_for_client(), ("".to_string(), 0));
}

#[test]
fn preedit_completion_has_no_visible_debug_marker() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    engine.tail_buffer = "при".to_string();
    engine.preedit_suffix = "вет".to_string();
    assert_eq!(engine.preedit_text_for_client(), ("вет".to_string(), 0));
}

#[test]
fn bracketed_precognition_is_display_only() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ime_bracket_candidates: true,
            nanda_l2_weight_percent: 200,
            ..LayConfig::default()
        },
    );
    engine.buffer = "хоро".to_string();
    engine.composition_cursor = 4;
    engine.preedit_candidates = vec!["шо".to_string()];

    assert_eq!(
        engine.composition_preedit_payload(),
        ("хоро[шо]".to_string(), 4)
    );
    assert_eq!(engine.selected_visible_completion_suffix().as_str(), "шо");
}

#[test]
fn preedit_candidates_suppress_noisy_single_letter_suffixes() {
    let mut candidates = Vec::new();

    push_unique_suffix(&mut candidates, Some("е".to_string()));
    push_unique_suffix(&mut candidates, Some("щ".to_string()));
    push_unique_suffix(&mut candidates, Some("и".to_string()));
    push_unique_suffix(&mut candidates, Some(" в".to_string()));
    push_unique_suffix(&mut candidates, Some("ет".to_string()));

    assert_eq!(candidates, vec!["и".to_string(), "ет".to_string()]);
}

#[test]
fn preedit_completion_does_not_duplicate_anchor() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    engine.tail_buffer = "проверк".to_string();
    engine.preedit_suffix = "а".to_string();

    assert_eq!(engine.preedit_text_for_client(), ("а".to_string(), 0));
}

#[test]
fn precognition_candidate_generation_stays_under_budget() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let samples = [
        "п",
        "пр",
        "пров",
        "file",
        "html d",
        "На улице опять идёт д",
        "смотрим что будет происходить когда при",
    ];
    let mut timings = Vec::new();
    let mut sample_max = Vec::new();
    for sample in samples {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in sample.chars() {
            engine.push_tail_char(ch);
        }
        for _ in 0..3 {
            engine.refresh_precognition_candidates();
        }
        let sample_start = timings.len();
        for _ in 0..20 {
            let started = Instant::now();
            engine.refresh_precognition_candidates();
            timings.push(started.elapsed().as_micros() as u64);
        }
        let mut local = timings[sample_start..].to_vec();
        local.sort_unstable();
        let local_p50 = percentile(&local, 50);
        let local_p90 = percentile(&local, 90);
        let local_p99 = percentile(&local, 99);
        let local_max = *local.last().unwrap_or(&0);
        eprintln!(
                "precognition sample {:?}: p50={}us p90={}us p99={}us max={}us candidates={} stages={:?}",
                sample,
                local_p50,
                local_p90,
                local_p99,
                local_max,
                engine.preedit_candidates.len(),
                measured_precognition_stages(&engine)
            );
        sample_max.push((sample, local_max));
    }
    timings.sort_unstable();
    let p50 = percentile(&timings, 50);
    let p90 = percentile(&timings, 90);
    let p99 = percentile(&timings, 99);
    let max = *timings.last().unwrap_or(&0);
    eprintln!(
        "precognition candidate generation: n={} p50={}us p90={}us p99={}us max={}us",
        timings.len(),
        p50,
        p90,
        p99,
        max
    );
    if let Some((sample, sample_max)) = sample_max.iter().max_by_key(|(_, max)| *max) {
        eprintln!(
            "precognition worst sample {:?}: max={}us",
            sample, sample_max
        );
    }
    let p90_budget_us = if cfg!(debug_assertions) {
        50_000
    } else {
        10_000
    };
    if std::env::var_os("LAY_ENFORCE_IME_LATENCY_BUDGET").is_some() {
        assert!(
            p90 <= p90_budget_us,
            "p90={p90}us exceeds budget {p90_budget_us}us"
        );
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let idx = ((values.len() - 1) * percentile) / 100;
    values[idx]
}

fn measured_precognition_stages(engine: &LayIbusEngine) -> Vec<(&'static str, u128, usize)> {
    let semantic_started = Instant::now();
    let semantic = engine.semantic_phrase_candidates();
    let semantic_us = semantic_started.elapsed().as_micros();

    let ru_started = Instant::now();
    let ru = engine.ru_l2_word_attractor_candidates();
    let ru_us = ru_started.elapsed().as_micros();

    let ascii_started = Instant::now();
    let ascii = engine.preedit_fast.ascii_candidates(
        engine.precognition_max_suffix_chars(),
        PREEDIT_ASCII_CANDIDATE_LIMIT,
    );
    let ascii_us = ascii_started.elapsed().as_micros();

    vec![
        ("semantic", semantic_us, semantic.len()),
        ("ru", ru_us, ru.len()),
        ("ascii", ascii_us, ascii.len()),
    ]
}

#[test]
fn preedit_for_surrounding_text_client_hides_probe_marker() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    engine.surrounding_text_supported = true;
    engine.tail_buffer = "ab".to_string();
    engine.preedit_suffix = PREEDIT_PROBE_SYMBOL.to_string();

    assert_eq!(engine.preedit_text_for_client(), ("".to_string(), 0));
}

#[test]
fn tail_buffer_stays_bounded() {
    let mut text = "x".repeat(PREEDIT_TAIL_LIMIT + 10);
    trim_tail_buffer(&mut text);
    assert_eq!(text.chars().count(), PREEDIT_TAIL_LIMIT);
}
