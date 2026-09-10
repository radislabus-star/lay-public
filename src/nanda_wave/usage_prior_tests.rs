use super::*;

#[test]
fn first_hot_readout_initializes_persisted_usage_memory_once() {
    let mut cache = UsageCache::default();
    let mut loads = 0;
    ensure_usage_cache_initialized(&mut cache, || {
        loads += 1;
        let mut counts = UsageCounts::default();
        counts.rejected_words.insert("ошибка".to_string(), 8);
        counts
    });
    ensure_usage_cache_initialized(&mut cache, || {
        panic!("an initialized hot cache must not reload on every readout")
    });

    assert_eq!(loads, 1);
    assert_eq!(cache.hot.rejected_word_count_for_tests("ошибка"), 8);
    assert!(cache.loaded_at.is_some());
}

#[test]
fn live_cache_owns_only_numeric_hot_state() {
    let source = include_str!("usage_prior.rs");
    let cache_body = source
        .split_once("struct UsageCache {")
        .and_then(|(_, tail)| tail.split_once('}'))
        .map(|(body, _)| body)
        .expect("UsageCache definition");
    let cold_type = ["Usage", "Counts"].concat();

    assert!(!cache_body.contains(&cold_type));
    assert!(!cache_body.contains("String"));
    assert_eq!(
        mem::size_of::<UsageCache>(),
        mem::size_of::<Option<Instant>>() + mem::size_of::<Arc<UsageHotState>>()
    );
}

#[test]
fn legacy_usage_prior_counts_target_words() {
    let counts = legacy_usage_counts_from_json(
        r#"{
            "а\u001fпроверка": {"to":"проверка","count":2,"promoted":false},
            "б\u001fпроверка": {"to":"проверка","count":1,"promoted":true},
            "в\u001f": {"to":"","count":99,"promoted":true}
        }"#,
    );

    assert_eq!(counts.get("проверка"), Some(&6));
    assert!(!counts.contains_key(""));
}

#[test]
fn usage_events_count_typed_fix_and_ime_words() {
    let text = r#"{"ts":1,"kind":"typed","word":"дождь","context":["на","улице","идёт"]}
{"ts":2,"kind":"accepted_fix","word":"дождь","from":"дожть","to":"дождь"}
{"ts":3,"kind":"accepted_ime","word":"дождь","context":["на","улице","идёт"],"to":"дождь"}
"#;
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);

    assert_eq!(counts.words.get("дождь"), Some(&12));
    assert_eq!(
        counts
            .context_words
            .get("на улице идёт\u{1f}дождь")
            .copied(),
        Some(6)
    );
    assert_eq!(
        counts.context_words.get("идёт\u{1f}дождь").copied(),
        Some(6)
    );
    assert_eq!(
        counts.context_words.get("улице идёт\u{1f}дождь").copied(),
        Some(6)
    );
    assert_eq!(counts.rejected_words.get("дожть"), Some(&6));
}

#[test]
fn hot_usage_prior_compiles_string_counts_into_packed_payload() {
    let text = r#"{"ts":1,"kind":"accepted_ime","word":"сверхдлиннаялокальнаякоманда","context":["предыдущийсверхдлинныйтокен","операторскийконтекст","детальныймаршрут"],"to":"сверхдлиннаялокальнаякоманда","source":"L2LiveCandidateGate32","operation":"completion","surface":"сверхдлиннаяповерхностькандидата"}
{"ts":2,"kind":"rejected_candidate","word":"сверхдлиннаяошибкакандидата","context":["предыдущийсверхдлинныйтокен","операторскийконтекст","детальныймаршрут"],"to":"сверхдлиннаяошибкакандидата","source":"L2LiveCandidateGate32","operation":"completion","surface":"сверхдлиннаяповерхностькандидата"}
"#;
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);
    let cold_dictionary_logical_bytes = usage_counts_cold_dictionary_logical_bytes(&counts);
    let usage = usage_snapshot_from_counts(counts);

    assert!(usage.hot_logical_payload_bytes() > 0);
    assert!(
        usage.hot_logical_payload_bytes() < cold_dictionary_logical_bytes,
        "hot logical payload must stay smaller than reversible cold strings: hot={} cold={}",
        usage.hot_logical_payload_bytes(),
        cold_dictionary_logical_bytes
    );
    assert!(usage.word_prior("сверхдлиннаялокальнаякоманда") > 0.0);
    assert_eq!(usage.accepted_word_count("сверхдлиннаялокальнаякоманда"), 5);
    assert!(
        usage
            .hot_readout(
                &[
                    "предыдущийсверхдлинныйтокен".to_string(),
                    "операторскийконтекст".to_string(),
                    "детальныймаршрут".to_string()
                ],
                "L2LiveCandidateGate32",
                "completion",
                "*",
                "сверхдлиннаялокальнаякоманда",
            )
            .transition
            .attraction
            > 0.0
    );
}

#[test]
fn live_cache_applies_typed_events_incrementally_with_cold_parity() {
    let text = r#"{"ts":1,"kind":"typed","word":"дождь","context":["на","улице"]}
{"ts":2,"kind":"accepted_fix","word":"дождь","context":["на","улице"],"from":"на улисе дожть","to":"на улице дождь","source":"autocorrect","operation":"replacement","surface":"дождь"}
{"ts":3,"kind":"accepted_ime","word":"комитет","context":["новый"],"to":"комитет","source":"ime","operation":"completion","surface":"комитет"}
{"ts":4,"kind":"rejected_ime","word":"камитет","context":["новый"],"to":"камитет","source":"ime","operation":"completion","surface":"камитет"}
{"ts":5,"kind":"rejected_candidate","word":"даша","context":["ну"],"from":"ну исходник","to":"ну даша","source":"L2LiveCandidateGate32","operation":"completion","surface":"даша"}
"#;
    let events = usage_events_from_jsonl(text).collect::<Vec<_>>();
    let mut cold = UsageCounts::default();
    let mut cache = UsageCache::default();
    ensure_usage_cache_initialized(&mut cache, UsageCounts::default);
    let hot_owner = Arc::as_ptr(&cache.hot);

    for event in &events {
        add_usage_event_count(&mut cold, event);
        apply_usage_event_to_cache(&mut cache, event, || {
            panic!("initialized live cache must not reload cold counts")
        });
        assert_eq!(Arc::as_ptr(&cache.hot), hot_owner);
    }

    let rebuilt = UsageHotState::from_counts(&cold);
    for surface in ["дождь", "комитет", "камитет", "даша"] {
        let live = cache.hot.phase_witness(surface);
        let cold = rebuilt.phase_witness(surface);
        assert_eq!(live.supported, cold.supported, "surface={surface}");
        assert_eq!(
            live.margin.total_cmp(&0.0),
            cold.margin.total_cmp(&0.0),
            "surface={surface} live={} cold={}",
            live.margin,
            cold.margin
        );
    }
    assert_eq!(cache.hot.word_prior("дождь"), rebuilt.word_prior("дождь"));
    assert_eq!(
        cache.hot.rejected_word_prior("даша"),
        rebuilt.rejected_word_prior("даша")
    );
    assert!(cache.hot.logical_payload_bytes() > 0);
}

#[test]
fn live_cache_make_mut_clones_when_snapshot_holds_hot_state() {
    let mut cache = UsageCache::default();
    ensure_usage_cache_initialized(&mut cache, UsageCounts::default);
    let snapshot = UsagePriorSnapshot {
        hot: Arc::clone(&cache.hot),
    };
    let snapshot_owner = Arc::as_ptr(&snapshot.hot);
    let cache_owner = Arc::as_ptr(&cache.hot);

    apply_usage_event_to_cache(
        &mut cache,
        &UsageEvent {
            ts: 1,
            schema: None,
            episode_id: None,
            kind: UsageEventKind::Typed,
            word: Some("дождь".to_string()),
            context: Vec::new(),
            from: None,
            to: None,
            source: None,
            operation: None,
            surface: None,
            operator: None,
            layout_direction: None,
            layout_scope: None,
            source_language: None,
            target_language: None,
            source_layout: None,
            target_layout: None,
            source_script: None,
            target_script: None,
            keyboard_geometry: None,
            identity_evidence: None,
            sentence_language: None,
            outcome: None,
            evidence_source_code: None,
            operation_code: None,
            operator_code: None,
            layout_direction_code: None,
            layout_scope_code: None,
            source_language_id: None,
            target_language_id: None,
            source_layout_id: None,
            target_layout_id: None,
            source_script_code: None,
            target_script_code: None,
            keyboard_geometry_id: None,
            identity_evidence_code: None,
            sentence_language_id: None,
            sentence_language_support_milli: None,
            sentence_language_alternative_milli: None,
            sentence_language_observed_tokens: None,
            outcome_code: None,
            completion_edit: None,
            proposal: None,
        },
        || panic!("initialized live cache must not reload cold counts"),
    );

    assert_eq!(Arc::as_ptr(&snapshot.hot), snapshot_owner);
    assert_ne!(Arc::as_ptr(&cache.hot), cache_owner);
    assert_eq!(snapshot.word_prior("дождь"), 0.0);
    assert!(cache.hot.word_prior("дождь") > 0.0);
}

#[test]
fn accepted_fix_creates_negative_trace_for_corrected_away_word_only() {
    let text = r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим"}
"#;
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);

    assert_eq!(counts.accepted_words.get("отравим"), Some(&6));
    assert_eq!(counts.rejected_words.get("отвравим"), Some(&6));
    assert!(!counts.rejected_words.contains_key("мы"));
    assert_eq!(
        counts
            .rejected_context_words
            .get("мы\u{1f}отвравим")
            .copied(),
        Some(6)
    );
}

#[test]
fn edited_ime_attracts_final_word_without_globally_rejecting_valid_suggestion() {
    let event = TypingMemoryEvent::edited_ime("это было", "прек", "прекрасный", "прекрасно")
        .expect("partial IME edit");
    let persisted = UsageEvent::from_typing_memory_event(&event);
    let text = format!("{}\n", serde_json::to_string(&persisted).unwrap());
    let usage = snapshot_from_usage_events_for_tests(&text);

    assert_eq!(persisted.kind, UsageEventKind::EditedIme);
    assert_eq!(
        persisted
            .completion_edit
            .as_ref()
            .map(|trace| trace.preserved_suffix_chars),
        Some(4)
    );
    assert!(usage.accepted_word_count("прекрасно") > 0);
    assert_eq!(usage.rejected_word_prior("прекрасный"), 0.0);
    let context = ["это", "было"].map(String::from);
    let state = crate::transition_relation::signed_memory_state_id("прекрасный");
    let transition = usage
        .hot_readout(
            &context,
            "ime",
            persisted.operator.as_deref().unwrap(),
            &state,
            "прекрасно",
        )
        .transition;
    assert!(transition.attraction > transition.repulsion);
}

#[test]
fn state_map_summary_counts_signed_word_states() {
    let text = r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим"}
{"ts":2,"kind":"accepted_ime","word":"дождь","context":["идёт"],"to":"дождь"}
"#;
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);

    let signed_word_states = counts
        .accepted_words
        .keys()
        .chain(counts.rejected_words.keys())
        .collect::<HashSet<_>>()
        .len();

    assert_eq!(counts.accepted_words.len(), 2);
    assert_eq!(counts.rejected_words.len(), 1);
    assert_eq!(signed_word_states, 3);
    assert!(!counts.transition_attract.is_empty());
    assert!(counts.transition_repel.is_empty());
}

#[test]
fn transition_signal_attracts_accepted_completion() {
    let usage = snapshot_from_usage_events_for_tests(
        r#"{"ts":1,"kind":"accepted_ime","word":"дождь","context":["на","улице","идёт"],"to":"дождь","source":"ime","operation":"completion"}
"#,
    );
    let context = ["на", "улице", "идёт"].map(String::from);
    let signal = usage
        .hot_readout(
            &context,
            "L2LiveCandidateGate32",
            "completion",
            "д",
            "дождь",
        )
        .transition;

    assert!(signal.attraction > signal.repulsion);
    assert!(signal.signed_weight > 0.0);
    assert_eq!(signal.reason, "transition_attracts");
}

#[test]
fn transition_signal_attracts_accepted_state_change() {
    let usage = snapshot_from_usage_events_for_tests(
        r#"{"ts":1,"kind":"accepted_fix","word":"отравим","context":["мы"],"from":"мы отвравим","to":"мы отравим","source":"user_correction","operation":"replacement"}
"#,
    );
    let context = ["мы"].map(String::from);
    let state = crate::transition_relation::signed_memory_state_id("мы отвравим");
    let signal = usage
        .hot_readout(
            &context,
            "SemanticWordCell32",
            "replacement",
            &state,
            "мы отравим",
        )
        .transition;

    assert!(signal.attraction > signal.repulsion);
    assert!(signal.signed_weight > 0.0);
    assert_eq!(signal.reason, "transition_attracts");
}

#[test]
fn automatic_apply_is_not_positive_feedback() {
    let usage = snapshot_from_usage_events_for_tests(
        r#"{"ts":1,"kind":"accepted_fix","word":"lfdfq","from":"давай","to":"lfdfq","source":"autocorrect","operation":"replacement"}
{"ts":2,"kind":"accepted_fix","word":"vpn","from":"МЗТ","to":"VPN","source":"layout","operation":"replacement"}
"#,
    );
    let signal = usage.hot_readout(&[], "autocorrect", "replacement", "давай", "lfdfq");
    let layout_state = crate::transition_relation::signed_memory_state_id("МЗТ");
    let layout = usage.hot_readout(&[], "layout", "replacement", &layout_state, "VPN");

    assert_eq!(signal.accepted_count, 0);
    assert_eq!(signal.transition.attraction, 0.0);
    assert_eq!(layout.accepted_count, 0);
    assert_eq!(layout.transition.attraction, 0.0);
}

#[test]
fn context_ngram_keys_cover_recent_suffixes() {
    let context = ["на", "улице", "опять", "идёт"].map(String::from);

    assert_eq!(
        context_ngram_keys(&context),
        [
            "идёт",
            "опять идёт",
            "улице опять идёт",
            "на улице опять идёт"
        ]
    );
    assert_eq!(
        hot::context_ngram_ids(&context).as_slice(),
        context_ngram_keys(&context)
            .iter()
            .map(|key| hot::usage_text_id(key))
            .collect::<Vec<_>>()
    );
}

#[test]
fn context_ngram_prior_scores_partial_context_match() {
    let mut counts = UsageCounts::default();
    add_usage_event_counts(
        &mut counts,
        r#"{"ts":1,"kind":"accepted_ime","word":"дождь","context":["на","улице","опять","идёт"],"to":"дождь"}
"#,
    );

    let close_context = ["вечером", "опять", "идёт"].map(String::from);
    let far_context = ["в", "другом", "месте"].map(String::from);

    let close_score = context_ngram_prior_from_counts(&counts, &close_context, "дождь");
    let far_score = context_ngram_prior_from_counts(&counts, &far_context, "дождь");

    assert!(close_score > 0.0);
    assert_eq!(far_score, 0.0);
    assert!(close_score > far_score);
}

#[test]
fn usage_surface_words_promote_repeated_local_words() {
    let mut counts = UsageCounts::default();
    add_usage_event_counts(
        &mut counts,
        r#"{"ts":1,"kind":"typed","word":"комитет"}
{"ts":2,"kind":"accepted_fix","word":"комитет","from":"коммит","to":"комитет"}
{"ts":3,"kind":"typed","word":"x"}
"#,
    );

    let words = usage_surface_words_from_counts(counts);

    assert_eq!(words.first().map(String::as_str), Some("комитет"));
    assert!(!words.iter().any(|word| word == "x"));
}

#[test]
fn unknown_raw_cyrillic_typo_does_not_promote_usage_prior() {
    let mut counts = UsageCounts::default();
    add_usage_event_counts(
        &mut counts,
        r#"{"ts":1,"kind":"typed","word":"звгрузи","context":["пожалуйста"]}
{"ts":2,"kind":"typed","word":"загрузи","context":["пожалуйста"]}"#,
    );

    assert!(!counts.words.contains_key("звгрузи"));
    assert!(counts.words.contains_key("загрузи"));
}

#[test]
fn usage_surface_words_promote_accepted_ime_word_into_hot_set() {
    let mut counts = UsageCounts::default();
    add_usage_event_counts(
        &mut counts,
        r#"{"ts":1,"kind":"accepted_ime","word":"архитектура","context":["новая"],"to":"архитектура"}
"#,
    );

    let words = usage_surface_words_from_counts(counts);

    assert_eq!(words.first().map(String::as_str), Some("архитектура"));
}

#[test]
fn unbound_rejected_ime_cannot_create_negative_learning() {
    let text = r#"{"ts":1,"kind":"rejected_ime","word":"даша","context":["ну"],"to":"даша","source":"ime","operation":"completion"}
"#;
    let mut counts = UsageCounts::default();
    add_usage_event_counts(&mut counts, text);

    assert!(!counts.words.contains_key("даша"));
    assert!(!counts.accepted_words.contains_key("даша"));
    assert!(!counts.rejected_words.contains_key("даша"));
    assert!(!counts.rejected_context_words.contains_key("ну\u{1f}даша"));

    let usage = snapshot_from_usage_events_for_tests(text);
    let context = ["ну"].map(String::from);
    let signal = usage
        .hot_readout(&context, "L2LiveCandidateGate32", "completion", "*", "даша")
        .transition;
    assert_eq!(signal.repulsion, signal.attraction);
}

#[test]
fn full_rebuild_deduplicates_same_second_same_payload_events() {
    let text = r#"{"ts":10,"kind":"typed","word":"дождь","context":["на","улице"]}
{"ts":10,"kind":"typed","word":"дождь","context":["на","улице"]}
{"ts":11,"kind":"typed","word":"дождь","context":["на","улице"]}
"#;
    let mut counts = UsageCounts::default();

    add_usage_event_counts(&mut counts, text);

    assert_eq!(counts.words.get("дождь"), Some(&2));
}

#[test]
fn historical_unattested_prediction_positive_is_ignored() {
    let text = r#"{"ts":1,"kind":"confirmed_ime_prediction","word":"режимем","context":["в","норм"],"source":"ime","operation":"prediction_match","outcome":"confirmed_positive"}
"#;
    let mut counts = UsageCounts::default();

    add_usage_event_counts(&mut counts, text);

    assert!(!counts.words.contains_key("режимем"));
    assert!(!counts.accepted_words.contains_key("режимем"));
}

#[test]
fn typing_memory_event_routes_accepted_fix_into_usage_counts() {
    let events = TypingMemoryEvent::accepted_fix("мы отвравим", "мы отравим");
    let mut counts = UsageCounts::default();
    for event in events {
        add_usage_event_count(&mut counts, &UsageEvent::from_typing_memory_event(&event));
    }

    assert_eq!(counts.accepted_words.get("отравим"), Some(&6));
    assert_eq!(counts.rejected_words.get("отвравим"), Some(&6));
    assert!(!counts.transition_attract.is_empty());
    assert!(counts.transition_repel.is_empty());
    assert_eq!(counts.surface_observed.len(), 1);
    assert_eq!(counts.surface_attract.len(), 1);
    assert!(counts.surface_repel.is_empty());
}

#[test]
fn typing_memory_event_routes_rejected_candidate_into_l4_repulsion() {
    let events = TypingMemoryEvent::rejected_candidate(
        "ну исходник",
        "ну даша",
        "L2LiveCandidateGate32",
        "completion",
    );
    let mut counts = UsageCounts::default();
    for event in events {
        add_usage_event_count(&mut counts, &UsageEvent::from_typing_memory_event(&event));
    }

    assert!(!counts.words.contains_key("даша"));
    assert_eq!(counts.rejected_words.get("даша"), Some(&8));
    assert_eq!(counts.surface_observed.len(), 1);
    assert!(counts.surface_attract.is_empty());
    assert_eq!(counts.surface_repel.len(), 1);

    let usage = usage_snapshot_from_counts(counts);
    let context = ["ну"].map(String::from);
    let signal = usage
        .hot_readout(
            &context,
            "L2LiveCandidateGate32",
            "completion",
            &crate::transition_relation::signed_memory_state_id("ну исходник"),
            "даша",
        )
        .transition;
    assert!(signal.repulsion > signal.attraction);
    assert_eq!(signal.reason, "transition_repels");
}

#[test]
fn typed_v3_event_replays_the_legacy_signed_state_exactly() {
    let event = TypingMemoryEvent::accepted_layout_projection("ltkfq", "делай")
        .into_iter()
        .next()
        .expect("layout event");
    let typed = UsageEvent::from_typing_memory_event(&event);
    assert_eq!(typed.schema, Some(TYPED_EVENT_SCHEMA_V3));
    assert!(typed.typed_v3_is_consistent());

    let mut legacy = typed.clone();
    legacy.schema = None;
    legacy.evidence_source_code = None;
    legacy.operation_code = None;
    legacy.operator_code = None;
    legacy.layout_direction_code = None;
    legacy.layout_scope_code = None;
    legacy.outcome_code = None;

    let mut typed_counts = UsageCounts::default();
    let mut legacy_counts = UsageCounts::default();
    add_usage_event_count(&mut typed_counts, &typed);
    add_usage_event_count(&mut legacy_counts, &legacy);
    assert_eq!(typed_counts, legacy_counts);

    let encoded = serde_json::to_string(&typed).expect("encode typed event");
    let decoded: UsageEvent = serde_json::from_str(&encoded).expect("decode typed event");
    assert!(decoded.typed_v3_is_consistent());
}

#[test]
fn legacy_v1_event_remains_readable_without_typed_codes() {
    let legacy = r#"{
        "ts":1,
        "kind":"accepted_fix",
        "word":"делай",
        "context":[],
        "from":"ltkfq",
        "to":"делай",
        "source":"user_correction",
        "operation":"replacement",
        "operator":"layout_projection:en_to_ru:current_token",
        "layout_direction":"en_to_ru",
        "layout_scope":"current_token",
        "outcome":"confirmed_positive"
    }"#;
    let event: UsageEvent = serde_json::from_str(legacy).expect("decode V1 usage event");

    assert_eq!(event.schema, None);
    assert_eq!(event.evidence_source_code, None);
    assert!(UsageEventProjection::from_event(&event).is_some());

    let mut migrated = event.clone();
    migrated.enrich_typed_v2();
    assert!(migrated.typed_v2_is_consistent());

    let mut before = UsageCounts::default();
    let mut after = UsageCounts::default();
    add_usage_event_count(&mut before, &event);
    add_usage_event_count(&mut after, &migrated);
    assert_eq!(before, after);
}

#[test]
fn malformed_typed_v3_identity_fails_closed() {
    let event = TypingMemoryEvent::accepted_layout_projection("ltkfq", "делай")
        .into_iter()
        .next()
        .expect("layout event");
    let mut persisted = UsageEvent::from_typing_memory_event(&event);
    persisted.layout_direction_code = Some(LayoutProjectionDirection::RuToEn.code());

    assert!(!persisted.typed_v3_is_consistent());
    assert!(UsageEventProjection::from_event(&persisted).is_none());
}

#[test]
fn usage_count_merge_preserves_signed_surface_memory() {
    let mut target = UsageCounts::default();
    let mut source = UsageCounts::default();
    source.surface_observed.insert("surface".to_string(), 11);
    source.surface_attract.insert("surface".to_string(), 6);
    source.surface_repel.insert("surface".to_string(), 5);

    merge_usage_counts(&mut target, source);

    assert_eq!(target.surface_observed.get("surface"), Some(&11));
    assert_eq!(target.surface_attract.get("surface"), Some(&6));
    assert_eq!(target.surface_repel.get("surface"), Some(&5));
}

#[test]
fn hot_readout_collects_usage_and_rejection_in_one_pass() {
    let usage = snapshot_from_usage_events_for_tests(
        r#"{"ts":1,"kind":"accepted_fix","word":"проверить","context":["можно"],"from":"можно проврить","to":"можно проверить","source":"user_correction","operation":"replacement"}
{"ts":2,"kind":"rejected_candidate","word":"проврить","context":["можно"],"to":"проврить","source":"autocorrect","operation":"auto_undo"}
"#,
    );
    let context = ["можно".to_string()];
    let state = crate::transition_relation::signed_memory_state_id("можно проврить");

    let good = usage.hot_readout(
        &context,
        "autocorrect",
        "replacement",
        &state,
        "можно проверить",
    );
    let prepared = usage.prepare_hot_context(&context);
    let prepared_good = usage.hot_readout_prepared(
        &prepared,
        "autocorrect",
        "replacement",
        &state,
        "можно проверить",
    );
    let bad = usage.hot_readout(&context, "autocorrect", "auto_undo", "*", "проврить");

    assert_eq!(prepared_good, good);
    assert!(good.accepted_count > 0);
    assert!(good.transition.attraction > good.transition.repulsion);
    assert!(bad.rejected_count > 0);
    assert!(bad.transition.repulsion > bad.transition.attraction);
}

#[test]
fn exact_state_transition_overrides_global_target_frequency() {
    let usage = snapshot_from_usage_events_for_tests(
        r#"{"ts":1,"kind":"accepted_fix","word":"так","from":"nfr","to":"так","source":"user_correction","operation":"replacement"}
{"ts":2,"kind":"accepted_fix","word":"так","from":"другой","to":"так","source":"user_correction","operation":"replacement"}
{"ts":3,"kind":"rejected_candidate","word":"так","from":"nfr","to":"так","source":"user_correction","operation":"replacement"}
"#,
    );

    let rejected_state = crate::transition_relation::signed_memory_state_id("nfr");
    let fallback_state = crate::transition_relation::signed_memory_state_id("новый");
    let rejected = usage.hot_readout(&[], "layout", "replacement", &rejected_state, "так");
    let fallback = usage.hot_readout(&[], "layout", "replacement", &fallback_state, "так");

    assert!(rejected.transition.repulsion > rejected.transition.attraction);
    assert!(rejected.transition.state_specific);
    assert!(fallback.transition.attraction > 0.0);
    assert!(!fallback.transition.state_specific);
}

#[test]
fn context_rejection_does_not_leak_into_empty_context_transition() {
    let usage = snapshot_from_usage_events_for_tests(
        r#"{"ts":1,"kind":"accepted_fix","word":"проверь","from":"ghjdthm","to":"проверь","source":"manual_layout_replay","operation":"layout"}
{"ts":2,"kind":"rejected_candidate","word":"проверь","context":["gfzvnm"],"from":"gfzvnm ghjdthm","to":"gfzvnm проверь","source":"typing-assist","operation":"mixed_layout"}
"#,
    );
    let state = crate::transition_relation::signed_memory_state_id("ghjdthm");
    let contextual_state = crate::transition_relation::signed_memory_state_id("gfzvnm ghjdthm");

    let empty = usage.hot_readout(&[], "layout", "layout", &state, "проверь");
    let contextual = usage.hot_readout(
        &["gfzvnm".to_string()],
        "layout",
        "mixed_layout",
        &contextual_state,
        "gfzvnm проверь",
    );

    assert!(empty.transition.attraction > empty.transition.repulsion);
    assert!(contextual.transition.repulsion > contextual.transition.attraction);
}

#[test]
fn adjacent_duplicate_usage_events_ignore_timestamp() {
    let first = UsageEvent {
        ts: 1,
        schema: None,
        episode_id: None,
        kind: UsageEventKind::Typed,
        word: Some("лог".to_string()),
        context: vec!["смотри".to_string()],
        from: None,
        to: None,
        source: Some("user".to_string()),
        operation: Some("typed".to_string()),
        surface: None,
        operator: None,
        layout_direction: None,
        layout_scope: None,
        source_language: None,
        target_language: None,
        source_layout: None,
        target_layout: None,
        source_script: None,
        target_script: None,
        keyboard_geometry: None,
        identity_evidence: None,
        sentence_language: None,
        outcome: None,
        evidence_source_code: None,
        operation_code: None,
        operator_code: None,
        layout_direction_code: None,
        layout_scope_code: None,
        source_language_id: None,
        target_language_id: None,
        source_layout_id: None,
        target_layout_id: None,
        source_script_code: None,
        target_script_code: None,
        keyboard_geometry_id: None,
        identity_evidence_code: None,
        sentence_language_id: None,
        sentence_language_support_milli: None,
        sentence_language_alternative_milli: None,
        sentence_language_observed_tokens: None,
        outcome_code: None,
        completion_edit: None,
        proposal: None,
    };
    let second = UsageEvent {
        ts: 2,
        ..first.clone()
    };

    assert!(usage_event_payload_eq(&first, &second));
}

#[test]
fn distinct_causal_episodes_are_not_adjacent_duplicates() {
    let first = UsageEvent {
        ts: 1,
        schema: None,
        episode_id: Some("episode-1".to_string()),
        kind: UsageEventKind::AcceptedFix,
        word: Some("новости".to_string()),
        context: vec!["читай".to_string()],
        from: Some("новость".to_string()),
        to: Some("новости".to_string()),
        source: Some("user_correction".to_string()),
        operation: Some("ime_auto_undo".to_string()),
        surface: None,
        operator: None,
        layout_direction: None,
        layout_scope: None,
        source_language: None,
        target_language: None,
        source_layout: None,
        target_layout: None,
        source_script: None,
        target_script: None,
        keyboard_geometry: None,
        identity_evidence: None,
        sentence_language: None,
        outcome: Some("confirmed_positive".to_string()),
        evidence_source_code: None,
        operation_code: None,
        operator_code: None,
        layout_direction_code: None,
        layout_scope_code: None,
        source_language_id: None,
        target_language_id: None,
        source_layout_id: None,
        target_layout_id: None,
        source_script_code: None,
        target_script_code: None,
        keyboard_geometry_id: None,
        identity_evidence_code: None,
        sentence_language_id: None,
        sentence_language_support_milli: None,
        sentence_language_alternative_milli: None,
        sentence_language_observed_tokens: None,
        outcome_code: None,
        completion_edit: None,
        proposal: Some("новость".to_string()),
    };
    let second = UsageEvent {
        ts: 2,
        episode_id: Some("episode-2".to_string()),
        ..first.clone()
    };

    assert!(!usage_event_payload_eq(&first, &second));
}

#[test]
fn compiled_feedback_snapshot_restores_exact_negative_transition() {
    let dir = std::env::temp_dir().join(format!("lay-l4-feedback-snapshot-{}", std::process::id()));
    let input = dir.join("events.jsonl");
    let output = dir.join("feedback-counts.json");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        &input,
        r#"{"ts":1,"kind":"rejected_candidate","word":"так","from":"nfr","to":"так","source":"user_correction","operation":"typing-assist","surface":"op=layout_projection|shape=replace|words=1:1"}
"#,
    )
    .unwrap();

    let report = compile_usage_feedback_snapshot(&input, &output).unwrap();
    let counts = load_persisted_usage_counts(&output, None).unwrap();
    let usage = usage_snapshot_from_counts(counts);
    let state = crate::transition_relation::signed_memory_state_id("nfr");
    let readout = usage.hot_readout(&[], "layout", "replacement", &state, "так");

    assert_eq!(report["parsed_events"], 1);
    assert_eq!(report["surface_anti_states"], 1);
    assert!(readout.transition.state_specific);
    assert!(readout.transition.repulsion > readout.transition.attraction);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn compiled_feedback_snapshot_recovers_reverted_auto_undo_receipt() {
    let dir = std::env::temp_dir().join(format!(
        "lay-l4-correction-feedback-snapshot-{}",
        std::process::id()
    ));
    let input = dir.join("corrections.jsonl");
    let output = dir.join("feedback-counts.json");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        &input,
        r#"{"ts":1,"kind":"user-correction","lay_kind":"typing-assist","lay_from":"проверрка ","lay_to":"проверка ","from":"проверка ","to":"проверрка ","user_target":"проверрка "}
"#,
    )
    .unwrap();

    let report = compile_usage_feedback_snapshot(&input, &output).unwrap();
    let counts = load_persisted_usage_counts(&output, None).unwrap();
    let usage = usage_snapshot_from_counts(counts);
    let state = crate::transition_relation::signed_memory_state_id("проверрка");
    let readout = usage.hot_readout(&[], "autocorrect", "replacement", &state, "проверка");
    let unseen = usage.hot_readout(
        &[],
        "new_runtime_source",
        "new_operator",
        &state,
        "проверит",
    );

    assert_eq!(report["usage_events"], 0);
    assert_eq!(report["correction_receipts"], 1);
    assert_eq!(report["correction_events"], 1);
    assert!(readout.transition.state_specific);
    assert!(readout.transition.repulsion > readout.transition.attraction);
    assert!(unseen.transition.state_specific);
    assert!(unseen.transition.repulsion > unseen.transition.attraction);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn corrected_state_repels_unseen_future_candidate_but_accepts_user_target() {
    let dir = std::env::temp_dir().join(format!(
        "lay-l4-correction-state-barrier-{}",
        std::process::id()
    ));
    let input = dir.join("corrections.jsonl");
    let output = dir.join("feedback-counts.json");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        &input,
        r#"{"ts":1,"kind":"user-correction","lay_kind":"typing-assist","lay_from":"cltkftv ","lay_to":"сделкаем ","from":"ем ","to":"м ","user_target":"сделкам "}
"#,
    )
    .unwrap();

    compile_usage_feedback_snapshot(&input, &output).unwrap();
    let counts = load_persisted_usage_counts(&output, None).unwrap();
    let usage = usage_snapshot_from_counts(counts);
    let state = crate::transition_relation::signed_memory_state_id("cltkftv");
    let unseen = usage.hot_readout(
        &[],
        "future_deterministic_source",
        "future_operator",
        &state,
        "седлаем",
    );
    let accepted = usage.hot_readout(
        &[],
        "future_deterministic_source",
        "future_operator",
        &state,
        "сделкам",
    );

    assert!(unseen.transition.state_specific);
    assert!(unseen.transition.repulsion > unseen.transition.attraction);
    assert!(accepted.transition.state_specific);
    assert!(accepted.transition.attraction > accepted.transition.repulsion);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn corrected_multiword_state_repels_candidate_with_a_different_context_split() {
    let dir = std::env::temp_dir().join(format!(
        "lay-l4-correction-multiword-state-{}",
        std::process::id()
    ));
    let input = dir.join("corrections.jsonl");
    let output = dir.join("feedback-counts.json");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        &input,
        r#"{"ts":1,"kind":"user-correction","lay_kind":"typing-assist","lay_from":"Ye cltkf","lay_to":"Ye сделай","from":"сделай","to":"сделНу","user_target":"Ye сделНу"}
"#,
    )
    .unwrap();

    compile_usage_feedback_snapshot(&input, &output).unwrap();
    let counts = load_persisted_usage_counts(&output, None).unwrap();
    let usage = usage_snapshot_from_counts(counts);
    let state = crate::transition_relation::signed_memory_state_id("Ye cltkf");
    let unseen = usage.hot_readout(
        &[],
        "future_layout_source",
        "future_layout_operator",
        &state,
        "Ну сделай",
    );

    assert!(unseen.transition.state_specific);
    assert!(unseen.transition.repulsion > unseen.transition.attraction);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn context_and_last_word_uses_recent_context() {
    let (context, word) =
        typing_memory::context_and_last_word("на улице опять идёт дождь ").unwrap();

    assert_eq!(word, "дождь");
    assert_eq!(context, ["на", "улице", "опять", "идёт"]);
}

#[test]
fn tail_compaction_keeps_recent_complete_lines() {
    let text = "one\ntwo\nthree\nfour\n";
    let compacted = keep_jsonl_tail_bytes(text, 11);

    assert_eq!(compacted, "three\nfour\n");
}

#[test]
fn tail_compaction_never_keeps_half_of_an_episode() {
    let b1 = "{\"episode_id\":\"b\",\"row\":1}\n";
    let b2 = "{\"episode_id\":\"b\",\"row\":2}\n";
    let c = "{\"episode_id\":\"c\",\"row\":1}\n";
    let text = format!(
        "{{\"episode_id\":\"a\",\"row\":1}}\n{{\"episode_id\":\"a\",\"row\":2}}\n{b1}{b2}{c}"
    );
    let budget_that_would_split_b = b2.len() + c.len();

    let compacted = keep_jsonl_tail_bytes(&text, budget_that_would_split_b);

    assert_eq!(compacted, c);
    assert!(!compacted.contains("\"episode_id\":\"b\""));
}
