use super::*;
use lay::config::LayConfig;
use std::sync::{Arc, Mutex};

#[test]
fn candidate_index_recovers_the_same_surface_from_a_shorter_suffix() {
    let candidates = vec!["ст".to_string(), "рошо".to_string()];

    assert_eq!(
        candidate_index_for_target("хвост", "хво", &candidates, &[None, None]),
        Some(0)
    );
    assert_eq!(
        candidate_index_for_target(
            "хорошо",
            "хор",
            &["ошо".to_string(), "ма".to_string()],
            &[None, None],
        ),
        Some(0)
    );
}

#[test]
fn candidate_target_is_released_when_new_input_invalidates_it() {
    assert_eq!(
        candidate_index_for_target(
            "хвалить",
            "хво",
            &["ст".to_string(), "ровать".to_string()],
            &[None, None],
        ),
        None
    );
}

#[test]
fn candidate_target_preserves_nonzero_selection_by_surface() {
    assert_eq!(
        candidate_index_for_target(
            "проверка",
            "прове",
            &["рить".to_string(), "рка".to_string(), "дение".to_string()],
            &[None, None, None],
        ),
        Some(1)
    );
}

#[test]
fn matching_typed_continuation_keeps_target_and_learning_observation() {
    let mut fast = PreeditFastState::default();
    for ch in "пере".chars() {
        fast.push(ch);
    }
    fast.remember_target(Some("перезагрузка".to_string()));
    fast.observe_prediction_target("пере", Some("перезагрузка".to_string()));

    fast.push('з');

    assert_eq!(fast.target_surface(), Some("перезагрузка"));
    assert!(fast.declined_target_surfaces.is_empty());
    assert_eq!(fast.observed_prediction_target(), Some("перезагрузка"));
}

#[test]
fn divergent_typed_continuation_suppresses_only_the_declined_full_target() {
    let mut fast = PreeditFastState::default();
    for ch in "про".chars() {
        fast.push(ch);
    }
    fast.remember_target(Some("проверка".to_string()));
    fast.observe_prediction_target("про", Some("проверка".to_string()));

    fast.push('д');

    let declined = ImeCandidateProposal::replacement(
        "проверка",
        0.9,
        lay::typing_cpu::ImeCandidateSource::L2Replacement,
    );
    let alternative = ImeCandidateProposal::replacement(
        "продолжить",
        0.8,
        lay::typing_cpu::ImeCandidateSource::L2Replacement,
    );
    assert!(proposal_repeats_declined_target(
        &fast.declined_target_surfaces,
        "прод",
        &declined
    ));
    assert!(!proposal_repeats_declined_target(
        &fast.declined_target_surfaces,
        "прод",
        &alternative
    ));
    assert_eq!(fast.target_surface(), None);
    assert_eq!(fast.observed_prediction_target(), Some("проверка"));
}

#[test]
fn prediction_feedback_distinguishes_confirmation_ending_change_and_censoring() {
    assert_eq!(
        observed_prediction_outcome("прек", "прекрасный", "прекрасный", true),
        ObservedPredictionOutcome::ConfirmedAttested
    );
    assert_eq!(
        observed_prediction_outcome("прек", "прекрасный", "прекрасно", true),
        ObservedPredictionOutcome::EndingChanged
    );
    assert_eq!(
        observed_prediction_outcome("прек", "прекрасный", "прекратить", true),
        ObservedPredictionOutcome::DivergedAfterPrefix
    );
    assert_eq!(
        observed_prediction_outcome("прек", "прекрасный", "другое", true),
        ObservedPredictionOutcome::Censored
    );
    assert_eq!(
        observed_prediction_outcome("прек", "прекрасный", "прекрасный", false),
        ObservedPredictionOutcome::MatchedUnattested
    );
}

#[test]
fn invalidated_target_retargets_to_fresh_top_candidate_without_blank_frame() {
    let candidates = vec!["ст".to_string(), "ровать".to_string()];

    assert_eq!(
        stable_candidate_index(Some("хвалить"), "хво", &candidates, &[None, None]),
        0
    );
}

#[test]
fn replacement_target_is_not_exposed_by_live_preedit() {
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
    for ch in "ytn".chars() {
        engine.push_tail_char(ch);
    }

    engine.refresh_precognition_candidates();

    assert!(
        engine
            .composition
            .preedit_candidates
            .iter()
            .all(|candidate| !candidate.starts_with('→')),
        "candidates={:?} replacements={:?}",
        engine.composition.preedit_candidates,
        engine.composition.preedit_replacement_targets
    );
    assert!(engine
        .composition
        .preedit_replacement_targets
        .iter()
        .all(Option::is_none));
}

#[test]
fn wrong_layout_letter_symbols_stay_inside_the_fast_token() {
    let mut fast = PreeditFastState::default();
    for ch in "ye;ty".chars() {
        fast.push(ch);
    }

    assert_eq!(fast.token(), "ye;ty");
    assert!(fast.is_ascii_live_candidate_token());

    fast.push('!');
    assert_eq!(fast.token(), "");
}

#[test]
fn leading_layout_symbol_waits_for_a_letter_before_becoming_a_candidate() {
    let mut fast = PreeditFastState::default();

    fast.push(',');
    assert_eq!(fast.token(), ",");
    assert!(!fast.is_ascii_live_candidate_token());

    fast.push('k');
    assert_eq!(fast.token(), ",k");
    assert!(fast.is_ascii_live_candidate_token());
}

#[test]
fn background_result_requires_the_current_focus_tail_and_token_identity() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/engine/a".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            correction_safety: "normal".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
    }
    let current = engine
        .capture_input_frame_identity()
        .expect("live input frame");
    assert!(engine.precognition_identity_matches(&current));

    let mut stale = current.clone();
    stale.tail_epoch = stale.tail_epoch.wrapping_add(1);
    assert!(!engine.precognition_identity_matches(&stale));

    shared.lock().expect("shared state").active_path = Some("/engine/b".to_string());
    assert!(!engine.precognition_identity_matches(&current));
}

#[test]
fn display_cancellation_does_not_change_space_authority_identity() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/engine/a".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    for ch in "слово".chars() {
        engine.push_tail_char(ch);
    }
    let frame = engine
        .capture_input_frame_identity()
        .expect("current input frame");

    engine.cancel_precognition_display_generation();

    assert!(engine.input_frame_authority_matches(&frame));
    assert!(engine.precognition_identity_matches(&frame));
}

#[test]
fn space_authority_rejects_complete_frame_identity_faults() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/engine/a".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    for ch in "слово".chars() {
        engine.push_tail_char(ch);
    }
    let frame = engine
        .capture_input_frame_identity()
        .expect("current input frame");
    assert!(engine.input_frame_authority_matches(&frame));
    assert!(engine.input_frame_identity_matches(&frame));

    engine.client_context.focus_receipt = Some("different-focus".to_string());
    assert!(!engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine
        .client_context
        .focus_receipt
        .clone_from(&frame.focus_receipt);

    engine.committed_tail.buffer.push('x');
    assert!(!engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine
        .committed_tail
        .buffer
        .clone_from(&frame.committed_tail);

    engine.layout_gesture.layout_is_ru = !frame.active_layout_is_ru;
    assert!(!engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine.layout_gesture.layout_is_ru = frame.active_layout_is_ru;

    engine.config.auto_replace = !engine.config.auto_replace;
    assert!(!engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine.config.auto_replace = !engine.config.auto_replace;

    engine.client_context.factory_engine_profile =
        lay::exact_layout_authority::FactoryEngineProfile::Unknown;
    assert!(!engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine.client_context.factory_engine_profile = frame.factory_engine_profile;

    engine.client_context.cursor_cell_width =
        engine.client_context.cursor_cell_width.saturating_add(1);
    assert!(!engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine.client_context.cursor_cell_width = 0;

    let active_input_at = engine.committed_tail.last_input_at;
    engine.committed_tail.last_input_at = None;
    assert!(engine.input_frame_authority_matches(&frame));
    assert!(!engine.input_frame_identity_matches(&frame));
    engine.committed_tail.last_input_at = active_input_at;
    assert!(engine.input_frame_identity_matches(&frame));

    let mut stale_fingerprint = frame.clone();
    stale_fingerprint.frame_fingerprint ^= 1;
    assert!(!engine.input_frame_identity_matches(&stale_fingerprint));
}

#[test]
fn layout_switch_away_and_back_keeps_the_old_frame_stale() {
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/engine/layout-generation".to_string(),
        shared,
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    assert!(engine.bind_focus_path());
    for ch in "слово".chars() {
        engine.push_tail_char(ch);
    }
    let old_frame = engine
        .capture_input_frame_identity()
        .expect("initial exact frame");
    let old_generation = old_frame
        .lexical_coordinates
        .as_ref()
        .expect("initial lexical coordinates")
        .layout_generation();
    let original_layout = engine.layout_gesture.layout_is_ru;

    engine.set_layout_is_ru(!original_layout);
    engine.set_layout_is_ru(original_layout);

    assert_eq!(engine.layout_gesture.layout_is_ru, original_layout);
    assert_ne!(engine.layout_gesture.layout_generation, old_generation);
    assert!(!engine.input_frame_authority_matches(&old_frame));
    let current_frame = engine
        .capture_input_frame_identity()
        .expect("refreshed exact frame");
    assert!(engine.input_frame_identity_matches(&current_frame));
}

#[test]
fn matching_continuation_keeps_the_same_full_target() {
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

    let initial_partial = "пров";
    for ch in initial_partial.chars() {
        engine.push_tail_char(ch);
    }
    engine.refresh_precognition_candidates();
    let previous_target = engine
        .composition
        .preedit_fast
        .target_surface()
        .map(str::to_owned);
    let previous_target = previous_target.expect("initial completion target");
    let continuation = previous_target
        .strip_prefix(initial_partial)
        .and_then(|suffix| suffix.chars().next())
        .expect("completion must extend the typed prefix");

    engine.push_tail_char(continuation);
    assert_eq!(
        engine.composition.preedit_fast.target_surface(),
        Some(previous_target.as_str())
    );
    engine.refresh_precognition_candidates();

    let partial = format!("{initial_partial}{continuation}");
    let refreshed_targets = engine
        .composition
        .preedit_candidates
        .iter()
        .zip(&engine.composition.preedit_replacement_targets)
        .map(|(suffix, replacement)| {
            replacement
                .clone()
                .unwrap_or_else(|| format!("{partial}{suffix}"))
        })
        .collect::<Vec<_>>();
    let suffix = engine.selected_visible_completion_suffix();
    assert!(
        !suffix.is_empty(),
        "fresh candidates={:?}",
        engine.composition.preedit_candidates
    );
    assert!(format!("{partial}{suffix}").starts_with(&partial));
    assert!(
        refreshed_targets.contains(&previous_target),
        "stable target={previous_target} refreshed targets={refreshed_targets:?}"
    );
    assert_eq!(
        engine.composition.preedit_fast.target_surface(),
        Some(previous_target.as_str())
    );
}

#[test]
fn preedit_publisher_installs_payload_before_showing_it() {
    let source = include_str!("../preedit.rs");
    let publisher = source
        .split("async fn publish_preedit_payload")
        .nth(1)
        .expect("single preedit publisher");
    let update = publisher
        .find(".update_preedit_text(")
        .expect("payload update");
    let show = publisher
        .find(".show_preedit_text(")
        .expect("visibility signal");

    assert!(
        update < show,
        "preedit payload must exist before it is shown"
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
    engine.composition.preedit_dirty = true;
    engine.composition.pending_display_frame = engine.capture_input_frame_identity();
    engine.composition.preedit_suffix = "ривет".to_string();
    engine.composition.preedit_candidates = vec!["ривет".to_string(), "роект".to_string()];
    engine.composition.preedit_candidate_index = 1;
    engine.push_tail_char(' ');

    assert!(
        !engine.composition.preedit_dirty,
        "word boundary must not resurrect previous word suffix on cursor flush"
    );
    assert_eq!(engine.composition.preedit_fast.token(), "");
    assert!(engine.composition.preedit_suffix.is_empty());
    assert!(engine.composition.preedit_candidates.is_empty());
    assert_eq!(engine.composition.preedit_candidate_index, 0);
    assert!(engine.composition.pending_display_frame.is_none());
}

#[test]
fn ignored_preedit_candidate_does_not_create_learning_feedback() {
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
    engine.composition.preedit_suffix = "ша".to_string();
    engine.composition.preedit_candidates = vec!["ша".to_string()];
    engine.push_tail_char(' ');

    std::thread::sleep(std::time::Duration::from_millis(50));
    let text = std::fs::read_to_string(&events_path).unwrap_or_default();
    let ignored_target_recorded = text.lines().any(|line| {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        matches!(
            event.get("kind").and_then(serde_json::Value::as_str),
            Some("rejected_ime" | "accepted_ime" | "confirmed_ime_prediction")
        ) && ["word", "to", "proposal"]
            .into_iter()
            .filter_map(|field| event.get(field).and_then(serde_json::Value::as_str))
            .any(|surface| surface == "даша")
    });
    assert!(!ignored_target_recorded, "{text}");

    std::env::remove_var("LAY_NANDA_WORD_USAGE_EVENTS");
    std::env::remove_var("LAY_NANDA_WORD_USAGE_COUNTS");
    let _ = std::fs::remove_file(events_path);
    let _ = std::fs::remove_file(counts_path);
}

#[test]
fn daemon_is_the_only_raw_typed_usage_owner() {
    let source = include_str!("../preedit.rs");
    assert!(!source.contains("TypingCpu::record_typed_tail"));
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
    for ch in "н".chars() {
        engine.push_tail_char(ch);
    }
    engine.composition.preedit_suffix = "у".to_string();
    engine.composition.preedit_candidates = vec!["у".to_string()];
    engine
        .composition
        .preedit_fast
        .observe_prediction_target("н", Some("ну".to_string()));
    engine.push_tail_char('у');
    assert_eq!(
        engine.composition.preedit_fast.observed_prediction_target(),
        Some("ну"),
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
    assert!(text.contains(r#""word":"ну""#), "{text}");
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
            .composition
            .preedit_candidates
            .iter()
            .all(|suffix| format!("нев{suffix}").starts_with("нев")),
        "shared gate returned a non-prefix completion: {:?}",
        engine.composition.preedit_candidates
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
        engine.composition.preedit_candidates.iter().any(|suffix| {
            let word = format!("пров{suffix}");
            word.starts_with("провер") || word.starts_with("прове")
        }),
        "expected contextual Russian wave candidates for 'я хочу пров', got {:?}",
        engine.composition.preedit_candidates
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
        engine.composition.preedit_candidates.iter().all(|suffix| {
            let projected = format!("звгрузи{suffix}");
            projected != "загрузи" && suffix != "агрузи"
        }),
        "replacement candidates must not leak into IME as suffix fragments: {:?}",
        engine.composition.preedit_candidates
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
            .composition
            .preedit_candidates
            .iter()
            .all(|suffix| !format!("за{suffix}").contains("запят")),
        "ambiguous prefix should not suggest project/chat noise: {:?}",
        engine.composition.preedit_candidates
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
        engine.composition.preedit_candidates.is_empty(),
        "command-like uppercase sentence tail must not get noisy IME suffixes: {:?}",
        engine.composition.preedit_candidates
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
            engine.composition.preedit_candidates.iter().all(|suffix| {
                !matches!(
                    suffix.as_str(),
                    "авило" | "ахать" | "алина" | "ббизм" | "арифм"
                )
            }),
            "short prefix {input:?} must not emit wide dictionary noise: {:?}",
            engine.composition.preedit_candidates
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
            .composition
            .preedit_candidates
            .iter()
            .all(|suffix| suffix != "алия"),
        "short prefix must not leak long dictionary-only tails: {:?}",
        engine.composition.preedit_candidates
    );
}

#[test]
fn bracketed_mode_is_display_only_and_does_not_create_a_second_candidate_gate() {
    fn candidates(bracketed: bool) -> Vec<String> {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                ime_bracket_candidates: bracketed,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "интересно инт".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();
        engine.composition.preedit_candidates
    }

    lay::nanda_wave::warm_up_l2_for_ime();
    assert_eq!(candidates(false), candidates(true));
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
fn settled_known_russian_word_does_not_get_extended_by_precognition() {
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
    // Managed IME input normally remains active until a boundary. Model the
    // distinct settled-token contract explicitly; active exact forms may still
    // expose morphology continuations while the user is editing the word.
    engine.committed_tail.last_input_at = None;
    engine.refresh_precognition_candidates();

    assert!(
        engine.composition.preedit_candidates.is_empty(),
        "known word must not be extended by weak suffixes: {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.preedit_suffix = "ка".to_string();
    engine.composition.preedit_candidates = vec!["ка".to_string()];
    engine.push_tail_char('?');
    engine.refresh_precognition_candidates();

    assert!(
        engine.composition.preedit_candidates.is_empty(),
        "punctuation must not revive completion for a closed word: {:?}",
        engine.composition.preedit_candidates
    );
    assert_eq!(engine.composition.preedit_fast.token(), "");
    assert_eq!(engine.composition.preedit_suffix, "");
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
            .composition
            .preedit_candidates
            .iter()
            .all(|suffix| suffix.chars().count() != 1
                || is_allowed_visible_completion_suffix(suffix)),
        "short prefix candidates must keep the single-letter guard: {:?}",
        engine.composition.preedit_candidates
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

    for suffix in &engine.composition.preedit_candidates {
        if suffix.chars().count() == 1 && !is_allowed_visible_completion_suffix(suffix) {
            let completed = format!("слов{suffix}");
            assert!(
                lay::lexicon::is_common_ru_word(&completed)
                    || lay::russian_lexicon::is_known_russian_word_or_form(&completed),
                "a weak one-letter suffix needs an attested completed center: {completed:?} from {:?}",
                engine.composition.preedit_candidates
            );
        }
    }
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
    for ch in "exi".chars() {
        engine.push_tail_char(ch);
    }

    let candidates = engine.word_candidate_proposals();

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
        engine.composition.preedit_candidates.iter().all(|suffix| {
            let word = format!("следую{suffix}");
            word.starts_with("следую")
        }),
        "long prefix suffixes must be prefix-preserving: {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();
    let (text, cursor_pos) = engine.composition_preedit_payload();

    assert!(
        text.starts_with("пров") && text.chars().count() > "пров".chars().count(),
        "normal IME should show an aggressive completion for raw Russian prefix: text={text:?}, candidates={:?}, replacements={:?}",
        engine.composition.preedit_candidates,
        engine.composition.preedit_replacement_targets,
    );
    assert_eq!(cursor_pos, 4);
    assert!(!engine.composition.preedit_suffix.is_empty());
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
        engine.composition.preedit_candidates.is_empty(),
        "word boundary must close visible IME suffixes, got {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.buffer = "ghbdtn".to_string();
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.composition.preedit_candidates = vec!["ий".to_string()];
    engine.composition.preedit_replacement_targets = vec![None];
    let (text, cursor_pos) = engine.composition_preedit_payload();

    assert_eq!(text, "ghbdtnий");
    assert_eq!(cursor_pos, 6);
    assert_eq!(engine.composition.preedit_suffix, "ий");
}

#[test]
fn candidate_installation_defensively_discards_typed_replacements() {
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
    engine.install_precognition_candidates(vec![ImeCandidateProposal::replacement(
        "работает".to_string(),
        1.0,
        lay::typing_cpu::ImeCandidateSource::L2Replacement,
    )]);

    assert!(engine.composition.preedit_candidates.is_empty());
    assert!(engine.composition.preedit_replacement_targets.is_empty());
    assert_eq!(engine.selected_precognition_replacement(), None);
}

#[test]
fn visible_active_composition_requires_one_preedit_clear_even_without_suffix() {
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
    engine.composition.buffer = "ghbdtn".to_string();
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix.clear();
    engine.composition.preedit_candidates.clear();

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
        !engine.composition.preedit_candidates.is_empty(),
        "experimental L2 should not stay silent for contextual prefix 'при'"
    );
}

#[test]
fn managed_committed_token_remains_active_until_its_boundary() {
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

    for ch in "аб".chars() {
        engine.push_tail_char(ch);
    }
    assert!(engine.live_completion_input_is_active());

    engine.push_tail_char(' ');
    assert!(!engine.live_completion_input_is_active());
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
            .composition
            .preedit_candidates
            .iter()
            .any(|suffix| suffix == "кий" || suffix == "ких"),
        "first Russian prefix should produce a useful word suffix: {:?}",
        engine.composition.preedit_candidates
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
        engine.composition.preedit_candidates.iter().any(|suffix| {
            let word = format!("писа{suffix}");
            word == "писать" || word.starts_with("писа")
        }),
        "punctuation before Russian prefix must not silence IME: {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine.composition.preedit_candidates.iter().any(|suffix| {
            let word = format!("пров{suffix}");
            word.starts_with("провер")
        }),
        "first active Russian word should produce a useful suffix after four chars: {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .composition
            .preedit_candidates
            .iter()
            .any(|suffix| suffix == "ь"),
        "authorized final-letter completion must stay visible: {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .composition
            .preedit_candidates
            .iter()
            .any(|suffix| matches!(suffix.as_str(), "ой" | "ие")),
        "Keep must compete with longer centers instead of stopping readout: {:?}",
        engine.composition.preedit_candidates
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
        engine.composition.preedit_candidates.iter().all(|suffix| {
            let word = format!("кандидат{suffix}");
            word.starts_with("кандидат") && word != "кандидоз"
        }),
        "live IME must not turn a prefix into unrelated semantic replacement: {:?}",
        engine.composition.preedit_candidates
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
            .composition
            .preedit_candidates
            .iter()
            .all(|suffix| suffix != "агрузи"),
        "word replacement belongs to boundary autocorrect, not IME suffix: {:?}",
        engine.composition.preedit_candidates
    );
}

#[test]
fn active_ime_does_not_render_a_longer_replacement_after_a_single_prefix_typo() {
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
    for ch in "переспектив".chars() {
        engine.insert_composition_char(ch);
    }
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        engine
            .composition.preedit_replacement_targets
            .iter()
            .flatten()
            .next()
            .is_none(),
        "the shared gate retains this family, but live IBus must not replace the visible token: candidates={:?}, replacements={:?}",
        engine.composition.preedit_candidates,
        engine.composition.preedit_replacement_targets
    );
}

#[test]
fn repeated_current_token_does_not_leak_a_full_replacement_into_completion_preedit() {
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
        engine.composition.preedit_replacement_targets.iter().flatten().all(|target| target != "то есть"),
        "the completion route must not render a boundary replacement: candidates={:?}, replacements={:?}",
        engine.composition.preedit_candidates,
        engine.composition.preedit_replacement_targets
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
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        !engine.composition.preedit_candidates.is_empty(),
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
    engine.composition.cursor = engine.composition.buffer.chars().count();
    engine.refresh_precognition_candidates();

    assert!(
        !engine.composition.preedit_candidates.is_empty(),
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
        .composition
        .preedit_candidates
        .first()
        .map(String::as_str)
        .unwrap_or("");
    assert!(
        first.chars().count() > 2,
        "short Russian prefix should not rank tiny suffix first: {:?}",
        engine.composition.preedit_candidates
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
        engine
            .composition
            .preedit_candidates
            .first()
            .map(String::as_str),
        Some("нь"),
        "IBus must render the common completion selected by the shared gate: {:?}",
        engine.composition.preedit_candidates
    );
}

#[test]
fn ime_can_render_a_bound_morphology_surface_from_the_full_l2_field() {
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
        engine
            .composition
            .preedit_candidates
            .iter()
            .any(|suffix| suffix == "ю"),
        "the attested жуть -> жутью morphology center must remain visible: {:?}",
        engine.composition.preedit_candidates
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
    engine.composition.preedit_candidates = vec!["ождь".to_string(), "ождик".to_string()];
    assert!(
        engine.composition.preedit_candidates.len() >= 2,
        "expected NANDA phrase candidates, got {:?}",
        engine.composition.preedit_candidates
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
    engine.committed_tail.buffer = "ab".to_string();
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
    engine.committed_tail.buffer = "пров".to_string();
    for ch in "пров".chars() {
        engine.composition.preedit_fast.push(ch);
    }

    assert!(!engine.precognition_preedit_enabled());
    engine.refresh_precognition_candidates();
    assert!(engine.composition.preedit_candidates.is_empty());
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
    engine.committed_tail.buffer = "ab".to_string();
    engine.composition.preedit_suffix = PREEDIT_PROBE_SYMBOL.to_string();

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
    engine.committed_tail.buffer = "при".to_string();
    engine.composition.preedit_suffix = "вет".to_string();
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
    engine.composition.buffer = "хоро".to_string();
    engine.composition.cursor = 4;
    engine.composition.preedit_candidates = vec!["шо".to_string()];

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
    engine.committed_tail.buffer = "проверк".to_string();
    engine.composition.preedit_suffix = "а".to_string();

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
    let mut cold_timings = Vec::new();
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
        let cold_stages = measured_precognition_stages(&engine);
        let cold_started = Instant::now();
        engine.refresh_precognition_candidates();
        let cold_elapsed = cold_started.elapsed().as_micros() as u64;
        eprintln!(
            "precognition cold sample {:?}: first_refresh_after_stages={}us stages={:?}",
            sample, cold_elapsed, cold_stages
        );
        cold_timings.push((sample, cold_elapsed));
        for _ in 0..2 {
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
                engine.composition.preedit_candidates.len(),
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
    eprintln!("precognition cold readouts: {cold_timings:?}");
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

    let word_started = Instant::now();
    let word = engine.word_candidate_proposals();
    let word_us = word_started.elapsed().as_micros();

    vec![
        ("semantic", semantic_us, semantic.len()),
        ("word", word_us, word.len()),
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
    engine.client_context.surrounding_text_supported = true;
    engine.committed_tail.buffer = "ab".to_string();
    engine.composition.preedit_suffix = PREEDIT_PROBE_SYMBOL.to_string();

    assert_eq!(engine.preedit_text_for_client(), ("".to_string(), 0));
}

#[test]
fn tail_buffer_stays_bounded() {
    let mut text = "x".repeat(PREEDIT_TAIL_LIMIT + 10);
    trim_tail_buffer(&mut text);
    assert_eq!(text.chars().count(), PREEDIT_TAIL_LIMIT);
}

#[test]
fn pending_refresh_shortens_the_retained_surface_without_accepting_it() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
    }
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "ерка".to_string();
    engine.composition.preedit_candidates = vec!["ерка".to_string()];
    engine.composition.preedit_replacement_targets = vec![None];
    engine
        .composition
        .preedit_fast
        .remember_target(Some("проверка".to_string()));

    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_FRAME_READY};

    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);
    zbus::block_on(engine.begin_pending_precognition_refresh(&mut output, true))
        .expect("pending retained surface");

    assert!(
        engine.composition.preedit_visible,
        "visible surface must not blink"
    );
    assert_eq!(engine.composition.preedit_suffix, "ерка");
    assert!(engine.composition.preedit_candidates.is_empty());
    assert!(engine.composition.preedit_replacement_targets.is_empty());
    assert_eq!(
        engine.composition.preedit_fast.target_surface(),
        Some("проверка")
    );
    assert_eq!(
        engine.selected_precognition_suffix(),
        None,
        "the retained suffix is display-only until the worker installs it"
    );
    assert!(
        !engine.cycle_precognition_candidate(1),
        "candidate cycling cannot repopulate pending acceptance authority"
    );
    assert_eq!(builder.pending_preedit_update(), Some(("ерка", 0, 0)));
    assert_eq!(builder.preedit_calls(), ["update-visible"]);
    let proposal = builder.finish(false);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(
        proposal.1.iter().map(|effect| effect.0).collect::<Vec<_>>(),
        [3],
        "pending refresh must emit one UpdatePreeditText and no hide/show frame"
    );
}

#[test]
fn pending_refresh_hides_a_target_that_no_longer_matches_the_partial() {
    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_FRAME_READY};

    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    for ch in "прод".chars() {
        engine.push_tail_char(ch);
    }
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "ерка".to_string();
    engine.composition.preedit_candidates = vec!["ерка".to_string()];
    engine
        .composition
        .preedit_fast
        .remember_target(Some("проверка".to_string()));

    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);
    zbus::block_on(engine.begin_pending_precognition_refresh(&mut output, true))
        .expect("mismatched pending surface");

    assert!(!engine.composition.preedit_visible);
    assert!(engine.composition.preedit_suffix.is_empty());
    assert!(engine.composition.preedit_candidates.is_empty());
    assert_eq!(engine.composition.preedit_fast.target_surface(), None);
    assert_eq!(builder.preedit_calls(), ["update-hidden", "hide"]);
    let proposal = builder.finish(false);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert_eq!(
        proposal.1.iter().map(|effect| effect.0).collect::<Vec<_>>(),
        [4],
        "a mismatched retained target must emit exactly one HidePreeditText"
    );
}

#[test]
fn terminal_cursor_ack_wait_publishes_the_shortened_surface_first() {
    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_FRAME_READY};

    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    shared.lock().expect("shared state").active_path = Some("/test".to_string());
    engine.client_context.surrounding_text_supported = false;
    engine.client_context.cursor_cell_width = 1;
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
    }
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "верка".to_string();
    engine.composition.preedit_candidates = vec!["верка".to_string()];
    engine
        .composition
        .preedit_fast
        .remember_target(Some("проверка".to_string()));
    let frame = engine.capture_input_frame_identity();

    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);
    zbus::block_on(
        engine.refresh_precognition_after_visible_input_with_background(
            &mut output,
            frame.clone(),
            true,
        ),
    )
    .expect("terminal pending refresh");

    assert_eq!(builder.pending_preedit_update(), Some(("ерка", 0, 0)));
    assert_eq!(builder.preedit_calls(), ["update-visible"]);
    assert_eq!(builder.finish(false).0, PROPOSAL_FRAME_READY);
    assert!(engine.composition.preedit_dirty);
    assert_eq!(engine.composition.pending_display_frame, frame);
    assert!(engine.composition.preedit_candidates.is_empty());
    assert_eq!(engine.selected_precognition_suffix(), None);
}

#[test]
fn stale_layout_frame_cannot_publish_a_retained_surface() {
    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_FRAME_READY};

    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::clone(&shared),
        true,
        true,
        LayConfig::default(),
    );
    shared.lock().expect("shared state").active_path = Some("/test".to_string());
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
    }
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "верка".to_string();
    engine.composition.preedit_candidates = vec!["верка".to_string()];
    engine
        .composition
        .preedit_fast
        .remember_target(Some("проверка".to_string()));
    let frame = engine
        .capture_input_frame_identity()
        .expect("current frame");
    engine.layout_gesture.layout_generation =
        engine.layout_gesture.layout_generation.wrapping_add(1);

    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);
    zbus::block_on(
        engine.refresh_precognition_after_visible_input_with_background(
            &mut output,
            Some(frame),
            true,
        ),
    )
    .expect("stale frame refusal");

    assert!(!engine.composition.preedit_visible);
    assert!(!engine.composition.preedit_display_only_pending);
    assert_eq!(
        builder.preedit_calls(),
        ["update-hidden", "hide"],
        "a stale layout frame must clear without a preceding visible update"
    );
    assert_eq!(builder.finish(false).0, PROPOSAL_FRAME_READY);
}

#[test]
fn atomic_output_refresh_materializes_candidates_before_publication() {
    lay::nanda_wave::warm_up_l2_for_ime();
    let shared = Arc::new(Mutex::new(Default::default()));
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::clone(&shared),
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
        engine.push_tail_char(ch);
    }
    shared.lock().expect("shared state").active_path = Some("/test".to_string());
    let frame = engine.capture_input_frame_identity();

    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_FRAME_READY};
    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);
    zbus::block_on(
        engine.refresh_precognition_after_visible_input_with_background(&mut output, frame, false),
    )
    .expect("atomic synchronous refresh");

    assert_eq!(
        engine.selected_precognition_suffix().as_deref(),
        Some("ерка")
    );
    assert_eq!(builder.pending_preedit_update(), Some(("ерка", 0, 0)));
    assert_eq!(builder.finish(false).0, PROPOSAL_FRAME_READY);
}

#[test]
fn pending_tab_and_cursor_arrow_retire_the_display_without_accepting_it() {
    use crate::output::{AtomicEffectBuilder, EngineOutput};
    use crate::protocol::{KEY_LEFT, KEY_RIGHT, KEY_TAB};

    for keyval in [KEY_TAB, KEY_LEFT, KEY_RIGHT] {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );
        engine.committed_tail.buffer = "пров".to_string();
        engine.composition.preedit_visible = true;
        engine.composition.preedit_suffix = "ерка".to_string();
        engine.composition.preedit_display_only_pending = true;
        engine.composition.preedit_dirty = true;
        engine.composition.pending_display_frame = engine.capture_input_frame_identity();

        let mut builder = AtomicEffectBuilder::default();
        let handled = {
            let mut output = EngineOutput::atomic(&mut builder);
            zbus::block_on(engine.process_pressed_key(&mut output, keyval, 0, 0))
                .expect("pending candidate key")
        };

        assert!(!handled);
        assert!(!engine.composition.preedit_visible);
        assert!(!engine.composition.preedit_display_only_pending);
        assert!(!engine.composition.preedit_dirty);
        assert!(engine.composition.pending_display_frame.is_none());
        assert_eq!(builder.preedit_calls(), ["update-hidden", "hide"]);

        let mut cursor_builder = AtomicEffectBuilder::default();
        let mut cursor_output = EngineOutput::atomic(&mut cursor_builder);
        zbus::block_on(engine.flush_dirty_preedit(&mut cursor_output))
            .expect("retired cursor acknowledgement");
        assert!(cursor_builder.preedit_calls().is_empty());
    }
}

#[test]
fn pending_candidate_arrows_refresh_the_current_list_before_cycling() {
    use crate::output::{AtomicEffectBuilder, EngineOutput};
    use crate::protocol::{KEY_DOWN, KEY_UP};

    lay::nanda_wave::warm_up_l2_for_ime();
    for keyval in [KEY_DOWN, KEY_UP] {
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
        for ch in "вариан".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();
        assert!(
            engine.composition.preedit_candidates.len() >= 2,
            "test prefix must expose multiple candidates: {:?}",
            engine.composition.preedit_candidates
        );
        engine.composition.preedit_visible = true;
        engine.composition.preedit_suffix = engine
            .selected_precognition_suffix()
            .expect("initial candidate");

        let mut pending_builder = AtomicEffectBuilder::default();
        let mut pending_output = EngineOutput::atomic(&mut pending_builder);
        zbus::block_on(engine.begin_pending_precognition_refresh(&mut pending_output, true))
            .expect("pending display");
        assert!(engine.composition.preedit_display_only_pending);
        assert!(engine.composition.preedit_candidates.is_empty());

        let mut builder = AtomicEffectBuilder::default();
        let handled = {
            let mut output = EngineOutput::atomic(&mut builder);
            zbus::block_on(engine.process_pressed_key(&mut output, keyval, 0, 0))
                .expect("pending candidate arrow")
        };

        assert!(handled, "candidate arrow must not escape to the client");
        assert!(!engine.composition.preedit_display_only_pending);
        assert!(engine.composition.preedit_candidates.len() >= 2);
        assert_ne!(engine.composition.preedit_candidate_index, 0);
        assert_eq!(builder.preedit_calls(), ["update-visible"]);
    }
}

#[test]
fn pending_active_composition_keeps_its_buffer_visible_and_editable() {
    use crate::output::{AtomicEffectBuilder, EngineOutput};
    use crate::protocol::{KEY_LEFT, KEY_TAB};

    let mut tab_engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    tab_engine.composition.buffer = "пров".to_string();
    tab_engine.composition.cursor = 4;
    tab_engine.composition.preedit_visible = true;
    tab_engine.composition.preedit_suffix = "ерка".to_string();
    tab_engine.composition.preedit_display_only_pending = true;

    let mut tab_builder = AtomicEffectBuilder::default();
    let tab_handled = {
        let mut output = EngineOutput::atomic(&mut tab_builder);
        zbus::block_on(tab_engine.process_pressed_key(&mut output, KEY_TAB, 0, 0))
            .expect("pending composition Tab")
    };

    assert!(!tab_handled);
    assert_eq!(tab_engine.composition.buffer, "пров");
    assert_eq!(tab_engine.composition.cursor, 4);
    assert!(tab_engine.composition.preedit_visible);
    assert!(!tab_engine.composition.preedit_display_only_pending);
    assert_eq!(tab_engine.selected_precognition_suffix(), None);
    assert_eq!(tab_builder.pending_preedit_update(), Some(("пров", 4, 0)));
    assert!(!tab_builder.preedit_calls().contains(&"hide"));

    let mut cursor_engine = tab_engine;
    cursor_engine.composition.preedit_display_only_pending = true;
    cursor_engine.composition.preedit_suffix = "ерка".to_string();
    let mut cursor_builder = AtomicEffectBuilder::default();
    let cursor_handled = {
        let mut output = EngineOutput::atomic(&mut cursor_builder);
        zbus::block_on(cursor_engine.process_pressed_key(&mut output, KEY_LEFT, 0, 0))
            .expect("pending composition cursor move")
    };

    assert!(cursor_handled);
    assert_eq!(cursor_engine.composition.buffer, "пров");
    assert_eq!(cursor_engine.composition.cursor, 3);
    assert!(cursor_engine.composition.preedit_visible);
    assert!(!cursor_engine.composition.preedit_display_only_pending);
    assert!(!cursor_builder.preedit_calls().contains(&"hide"));
}

#[test]
fn pending_alt_gesture_retires_before_release_and_cannot_accept() {
    use crate::output::{AtomicEffectBuilder, EngineOutput};
    use crate::protocol::{KEY_ISO_LEVEL3_SHIFT, KEY_LEFT_ALT, KEY_RIGHT_ALT, RELEASE_MASK};

    for keyval in [KEY_LEFT_ALT, KEY_RIGHT_ALT, KEY_ISO_LEVEL3_SHIFT] {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }
        engine.composition.preedit_visible = true;
        engine.composition.preedit_suffix = "ерка".to_string();
        engine.composition.preedit_display_only_pending = true;
        engine
            .composition
            .preedit_fast
            .remember_target(Some("проверка".to_string()));

        let mut builder = AtomicEffectBuilder::default();
        let press = {
            let mut output = EngineOutput::atomic(&mut builder);
            zbus::block_on(engine.process_key_event_with_output(&mut output, keyval, 64, 0))
                .expect("pending Alt press")
        };
        let release = {
            let mut output = EngineOutput::atomic(&mut builder);
            zbus::block_on(engine.process_key_event_with_output(
                &mut output,
                keyval,
                64,
                RELEASE_MASK,
            ))
            .expect("pending Alt release")
        };

        assert!(!press);
        assert!(!release);
        assert!(engine.composition.preedit_candidates.is_empty());
        assert_eq!(engine.selected_precognition_suffix(), None);
        assert_eq!(builder.preedit_calls(), ["update-hidden", "hide"]);
    }
}

#[test]
fn background_candidates_remain_unauthorized_when_publication_fails() {
    use crate::output::{AtomicEffectBuilder, EngineOutput};

    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    for ch in "пров".chars() {
        engine.push_tail_char(ch);
    }
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "ерка".to_string();
    engine.composition.preedit_display_only_pending = true;

    let proposals = vec![ImeCandidateProposal::new(
        "ерка",
        1.0,
        lay::typing_cpu::ImeCandidateSource::L2Completion,
    )];
    let mut builder = AtomicEffectBuilder::default();
    builder.fail_preedit_publication();
    let mut output = EngineOutput::atomic(&mut builder);
    let result = zbus::block_on(engine.apply_background_precognition(&mut output, proposals));

    assert!(result.is_err());
    assert!(engine.composition.preedit_display_only_pending);
    assert!(engine.composition.preedit_candidates.is_empty());
    assert_eq!(engine.selected_precognition_suffix(), None);
}

#[test]
fn late_worker_retires_the_display_exactly_once() {
    use crate::output::{AtomicEffectBuilder, EngineOutput};

    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    engine.composition.preedit_visible = true;
    engine.composition.preedit_suffix = "ерка".to_string();
    engine.composition.preedit_display_only_pending = true;

    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);
    assert!(zbus::block_on(engine.retire_late_precognition(&mut output)).expect("late retirement"));
    assert!(
        !zbus::block_on(engine.retire_late_precognition(&mut output))
            .expect("idempotent late retirement")
    );

    assert_eq!(builder.preedit_calls(), ["update-hidden", "hide"]);
    assert!(!engine.composition.preedit_visible);
    assert!(engine.composition.preedit_candidates.is_empty());
    assert_eq!(engine.selected_precognition_suffix(), None);
}

#[test]
fn visible_precognition_waits_for_three_letter_prefix() {
    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );

    for ch in "пр".chars() {
        engine.push_tail_char(ch);
    }
    assert!(!engine.precognition_display_ready());

    engine.push_tail_char('о');
    assert!(engine.precognition_display_ready());
}

#[test]
fn clearing_an_already_hidden_preedit_emits_no_empty_frame() {
    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_NATIVE_UNHANDLED};

    let mut engine = LayIbusEngine::new(
        "/test".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig::default(),
    );
    engine.committed_tail.buffer = "ghbdtn".to_string();
    engine
        .composition
        .preedit_fast
        .remember_target(Some("привет".to_string()));
    let mut builder = AtomicEffectBuilder::default();
    let mut output = EngineOutput::atomic(&mut builder);

    zbus::block_on(engine.clear_preedit(&mut output)).expect("hidden clear");

    assert_eq!(builder.finish(false).0, PROPOSAL_NATIVE_UNHANDLED);
    assert!(engine.composition.preedit_fast.target_surface().is_none());
}
