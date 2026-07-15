use super::*;
use crate::nanda_wave::l1::run_l1;

#[test]
fn accepted_transition_can_admit_unknown_layout_surface() {
    assert!(!layout_candidate_allowed("полняй", "gjkyzq", false, false));
    assert!(layout_candidate_allowed("полняй", "gjkyzq", false, true));
}

#[test]
fn layout_candidate_for_last_token() {
    let original = "html djn ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.text == "html вот"));
}

#[test]
fn l2_weight_scales_candidate_energy() {
    let original = "html djn ";
    let l1 = run_l1(original);
    let normal = run_l2_with_options(original, &l1, &WaveOptions::default());
    let muted = run_l2_with_options(
        original,
        &l1,
        &WaveOptions::default().with_layer_weights(0.5, 1.0),
    );
    let normal_layout = normal
        .iter()
        .find(|candidate| candidate.text == "html вот")
        .expect("normal layout candidate");
    let muted_layout = muted
        .iter()
        .find(|candidate| candidate.text == "html вот")
        .expect("muted layout candidate");

    assert!(muted_layout.energy < normal_layout.energy);
    assert!(muted_layout
        .support
        .iter()
        .any(|item| item == "l2-weight:0.50"));
}

#[test]
fn keeps_known_technical_ascii_token() {
    let original = "git status ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.source != "LayoutWordCell32"));
}

#[test]
fn technical_context_does_not_flip_argument_like_ascii() {
    let original = "vpn port ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.source != "LayoutWordCell32"));
}

#[test]
fn scans_previous_layout_token_before_technical_tail() {
    let original = "html djn api ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.text == "html вот api"));
}

#[test]
fn exposes_current_and_previous_layout_candidates_to_mesh() {
    let original = "html djn api ашду ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.text == "html djn api file"));
    assert!(candidates
        .iter()
        .any(|candidate| candidate.text == "html вот api ашду"));
}

#[test]
fn mixed_ru_en_context_does_not_emit_raw_malformed_layout_candidate() {
    let original = "тест Ghjljkbv file ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);

    assert!(candidates
        .iter()
        .all(|candidate| candidate.text != "тест Продолим file"));
}

#[test]
fn guard_prefix_blocks_short_layout_argument() {
    let original = "api djn ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.text != "api вот"));
}

#[test]
fn does_not_flip_normal_cyrillic_word_to_ascii_noise() {
    let original = "у нас есть ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates.iter().all(|candidate| {
        candidate.origin != CandidateOrigin::Layout
            && candidate.source != "LayoutWordCell32"
            && !candidate.text.chars().any(|ch| ch.is_ascii_alphabetic())
    }));
}

#[test]
fn layout_word_cell_respects_known_short_russian_words() {
    let original = "ой ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.text != "jq" && candidate.source != "LayoutWordCell32"));
}

#[test]
fn grammar_cell_keeps_known_plural_forms_after_verbs() {
    let original = "имеет волнистые ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.text != "имеет волнистый"));
}

#[test]
fn boundary_cell_gets_structural_candidate() {
    let original = "у насесть ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.source == "BoundaryCell32"));
}

#[test]
fn boundary_cell_splits_dictionary_glue() {
    let original = "она есть ";
    let glued = original.replace(' ', "");
    let l1 = run_l1(&glued);
    let candidates = run_l2(&glued, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.text == "она есть"));
}

#[test]
fn boundary_cell_does_not_split_known_russian_word_forms() {
    for original in [
        "упоминай ",
        "поехал ",
        "поплыл ",
        "указать ",
        "сторона ",
        "улетели ",
        "кодировании ",
    ] {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source != "BoundaryCell32"),
            "known word must not become boundary split: {original:?} -> {candidates:?}"
        );
    }
}

#[test]
fn boundary_cell_recovers_one_letter_function_boundary() {
    let original = "влогах ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    let split = candidates
        .iter()
        .find(|candidate| candidate.text == "в логах")
        .expect("hidden short function boundary candidate");
    assert_eq!(split.source, "BoundaryCell32");
    assert!(
        split.energy - split.risk > 0.90,
        "split candidate must outrank single-word typo: {split:?}"
    );
}

#[test]
fn boundary_cell_does_not_split_multi_letter_preposition_guesses() {
    for original in ["заполни поспорта ", "в задани "] {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "BoundaryCell32"),
                "multi-letter preposition guesses must not split automatically: {original:?} -> {candidates:?}"
            );
    }
}

#[test]
fn boundary_cell_scans_glued_word_inside_tail() {
    let original = "я пишу мои слова мои предложения чтобыточно проверить дальше ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == "BoundaryCell32"
                && candidate.text == "я пишу мои слова мои предложения чтобы точно проверить дальше"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn boundary_cell_scans_split_pair_inside_tail() {
    let original = "сейчас думаю тако й пример работает ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == "BoundaryCell32"
                && candidate.text == "сейчас думаю такой пример работает"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn boundary_cell_scans_moved_prefix_pair_inside_tail() {
    let original = "сервер работает н апостоянку ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == "BoundaryShiftCell32"
                && candidate.text == "сервер работает на постоянку"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn boundary_cell_uses_context_to_split_known_glued_form() {
    let original = "мы должны помнить что у насесть право на информацию ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == "BoundaryCell32"
                && candidate.text == "мы должны помнить что у нас есть право на информацию"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn phrase_cell_does_not_rewrite_single_all_caps_russian_terms() {
    for original in ["БЕЙСОВ ", "БЕЙСОВК ", "БЕЙСОВКИ ", "БЕЙСОВСКИ "]
    {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "PhraseCell32"),
                "all-caps term should not get PhraseCell typo candidate: {original:?} -> {candidates:?}"
            );
    }
}

#[test]
fn phrase_cell_does_not_delete_n_from_pattern_terms() {
    for (original, rejected) in [
        ("патерн ", "патер"),
        ("патерна ", "патера"),
        ("патернов ", "патеров"),
    ] {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.text.trim() != rejected),
            "pattern-like term should not get n-deletion candidate: {original:?} -> {candidates:?}"
        );
        assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "PhraseCell32"),
                "pattern-like term should not get PhraseCell typo candidate: {original:?} -> {candidates:?}"
            );
    }
}

#[test]
fn phrase_cell_gets_typo_candidate() {
    let original = "рабоатет ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.source == L2_SURFACE_MOTIF_CELL));
}

#[test]
fn l2_surface_motif_cell_generates_word_candidate() {
    let original = "делай проверк ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "делай проверка"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn l2_surface_motif_cell_recovers_known_word_from_fuzzy_dictionary() {
    let original = "звгрузи ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "загрузи"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn l2_surface_layer_recovers_adjacent_transposition() {
    let original = "пукнт ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            matches!(
                candidate.source,
                L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
            ) && candidate.text == "пункт"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn l2_form_attractor_prefers_clean_corpus_center() {
    let original = "пукнт ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    let first = candidates
        .first()
        .expect("dirty transposition should produce L2 attractor candidates");
    assert!(matches!(
        first.source,
        L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
    ));
    assert_eq!(first.text, "пункт");
    assert!(
        candidates.iter().any(|candidate| {
            candidate.text == "пуант" && candidate.source == LEXICAL_ATTRACTOR_CELL
        }),
        "near clean centers should remain visible but lower-ranked: {candidates:?}"
    );
}

#[test]
fn l2_form_attractor_does_not_rewrite_stable_word() {
    let original = "писать ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.source != LEXICAL_ATTRACTOR_CELL),
        "stable word should not become an attractor rewrite: {candidates:?}"
    );
}

#[test]
fn l2_form_attractor_does_not_rewrite_known_verb_form() {
    assert!(surface_motif_stable_existing_word("можем"));
    assert!(!surface_motif_stable_existing_word("пукнт"));
    assert!(!surface_motif_stable_existing_word("звгрузи"));

    for original in ["можем ", "проверка можем "] {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates.iter().all(|candidate| {
                !matches!(
                    candidate.source,
                    L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
                ) || candidate.text == original.trim_end()
            }),
            "known verb form should not drift to a neighboring word: {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| !matches!(candidate.text.as_str(), "модем" | "может")),
            "known verb form leaked standalone drift candidates: {candidates:?}"
        );
        assert!(
            candidates.iter().all(|candidate| {
                !matches!(candidate.text.as_str(), "проверка модем" | "проверка может")
            }),
            "known verb form leaked phrase drift candidates: {candidates:?}"
        );
    }
}

#[test]
fn l2_surface_motif_does_not_treat_usage_typo_as_stable_word() {
    assert!(!surface_motif_stable_existing_word("пукнт"));
    assert!(!fuzzy_surface_candidate_blocked("пукнт", "пукнт", "пункт"));
    let fuzzy = crate::ru_typo::fuzzy_known_word_candidates("пукнт");
    assert!(fuzzy.iter().any(|candidate| candidate == "пункт"));
    assert!(surface_motif_typo_has_authority(
        "пукнт",
        "пункт",
        900,
        &[],
        &fuzzy
    ));
    assert!(surface_motif_typo_allowed("пукнт", "пункт", 5, 1, 900));
    let l1 = run_l1("пукнт");
    let context = TailContext::from_text("пукнт");
    let cell_candidates =
        surface_motif_word_candidates("", "пукнт", &context, &l1, &WaveOptions::default());
    assert!(
        cell_candidates
            .iter()
            .any(|candidate| candidate.text == "пункт"),
        "cell_candidates={cell_candidates:?}"
    );
}

#[test]
fn l2_surface_motif_memory_recovers_missing_letter_without_fuzzy_route() {
    let candidates = surface_motif_memory().surface_candidates("звгрузи", 8);
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.word == "загрузи"),
        "candidates={candidates:?}"
    );
}

#[test]
fn lexical_phase_field_recovers_inflected_forms_from_compiled_transition_mass() {
    let cases = [
        ("рабоатет", "работает"),
        ("кнокопками", "кнопками"),
        ("фактческим", "фактическим"),
        ("подлючись", "подключись"),
        ("исправленно", "исправлено"),
    ];
    for (input, expected) in cases {
        let candidates = surface_motif_memory().surface_candidates(input, 32);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == expected),
            "{input} -> {expected}, candidates={candidates:?}"
        );
    }
}

#[test]
fn ime_l2_word_candidates_return_whole_words_not_suffixes() {
    let candidates = ime_l2_word_candidates("я хочу ", "пров", 8);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.kind == L2ImeWordCandidateKind::Completion
                && candidate.surface.starts_with("провер")
        }),
        "L2 IME candidates must expose complete word surfaces, got {candidates:?}"
    );
    assert!(
        candidates
            .iter()
            .all(|candidate| !candidate.surface.starts_with("ер")),
        "L2 must not return display suffixes as word candidates: {candidates:?}"
    );
}

#[test]
fn lexical_phase_field_feeds_ime_completion_candidates() {
    let candidates = ime_l2_word_candidates("я хочу ", "пров", 8);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.kind == L2ImeWordCandidateKind::Completion
                && candidate.surface.starts_with("пров")
                && candidate.surface.chars().count() > 4
        }),
        "lexical phase field must feed complete generated surfaces, got {candidates:?}"
    );
}

#[test]
fn lexical_phase_field_recovers_english_typo_without_context_wave() {
    let original = "dowenload ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);

    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "download"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn lexical_phase_field_completes_english_prefix() {
    let candidates = ime_l2_word_candidates("please ", "down", 12);

    assert!(
        candidates.iter().any(|candidate| {
            candidate.kind == L2ImeWordCandidateKind::Completion && candidate.surface == "download"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn common_english_center_leads_short_prefix_frontier() {
    let candidates = ime_l2_word_candidates("", "exi", 32);

    assert_eq!(
        candidates
            .first()
            .map(|candidate| candidate.surface.as_str()),
        Some("exit"),
        "candidates={candidates:?}"
    );
}

#[test]
fn layout_projection_settles_in_english_phase_center() {
    let original = "вщцутдщфв ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);

    assert!(
        candidates.iter().any(|candidate| {
            candidate.origin == CandidateOrigin::LayoutThenTypo
                && candidate.source == LAYOUT_THEN_L2_WORD_CENTER
                && candidate.text == "download"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn ime_l2_word_candidates_keep_replacements_distinct_from_completions() {
    let candidates = ime_l2_word_candidates("", "звгрузи", 8);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.kind == L2ImeWordCandidateKind::Replacement && candidate.surface == "загрузи"
        }),
        "noisy input should produce a whole-word replacement candidate, got {candidates:?}"
    );
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.surface != "агрузи"),
        "replacement candidates must not be converted into suffix fragments: {candidates:?}"
    );
}

#[test]
fn l2_surface_motif_memory_recovers_common_shadow_words() {
    for (input, expected) in [("эсперемнт", "эксперимент"), ("ффективная", "эффективная")]
    {
        let candidates = surface_motif_memory().surface_candidates(input, 32);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == expected),
            "input={input} expected={expected} candidates={candidates:?}"
        );
    }
}

#[test]
fn l2_surface_motif_cell_promotes_common_shadow_words() {
    for (input, expected) in [
        ("эсперемнт ", "эксперимент"),
        ("ффективная ", "эффективная"),
    ] {
        let l1 = run_l1(input);
        let candidates = run_l2(input, &l1);
        let surface_candidates = surface_motif_memory().surface_candidates(input.trim(), 24);
        assert!(
                candidates.iter().any(|candidate| {
                    candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == expected
                }),
                "input={input} expected={expected} candidates={candidates:?} surface_candidates={surface_candidates:?}"
            );
    }
}

#[test]
fn l2_surface_motif_cell_repairs_repeated_letter_all_caps_word() {
    let original = "ТРУССС ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates.iter().any(|candidate| {
            candidate.source == L2_SURFACE_MOTIF_CELL && candidate.text == "ТРУС"
        }),
        "candidates={candidates:?}"
    );
}

#[test]
fn l2_surface_motif_cell_does_not_rewrite_known_word_without_context() {
    let original = "пукнут ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.source != L2_SURFACE_MOTIF_CELL),
        "candidates={candidates:?}"
    );
}

#[test]
fn completion_proposals_remain_typed_before_decision_authority() {
    let original = "делай пров ";
    let l1 = run_l1(original);
    let autocorrect_candidates = run_l2(original, &l1);
    let completion_proposals = autocorrect_candidates
        .iter()
        .filter(|candidate| candidate.source == L2_SURFACE_COMPLETION_CELL)
        .collect::<Vec<_>>();
    assert!(
        !completion_proposals.is_empty()
            && completion_proposals
                .iter()
                .all(|candidate| candidate.origin == CandidateOrigin::Completion),
        "L2 completion material must stay typed until DecisionCore: {autocorrect_candidates:?}"
    );

    let ime_candidates = ime_l2_word_candidates("делай ", "пров", 8);
    assert!(
        ime_candidates
            .iter()
            .any(|candidate| candidate.kind == L2ImeWordCandidateKind::Completion),
        "live IME must retain completion authority: {ime_candidates:?}"
    );
}

#[test]
fn autocorrect_completion_does_not_extend_a_complete_inflected_form() {
    let original = "модель генерит ";
    assert!(surface_motif_stable_existing_word("генерит"));

    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);

    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.source != L2_SURFACE_COMPLETION_CELL),
        "a completed form must not enter the post-space completion route: {candidates:?}"
    );
}

#[test]
fn grammar_cell_does_not_fake_unknown_phrase_candidate() {
    let original = "фразы связанности ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.source != "GrammarCell32"));
}

#[test]
fn grammar_cell_generates_agreement_candidate() {
    let original = "расчёт приблизительные ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates.iter().any(|candidate| {
        candidate.source == "GrammarCell32" && candidate.text == "расчёт приблизительный"
    }));
}

#[test]
fn grammar_cell_completes_preposition_case_tail() {
    let original = "в задани ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates.iter().any(|candidate| {
        candidate.source == "GrammarCell32" && candidate.text == "в задании"
    }));
}

#[test]
fn phrase_cell_does_not_hardcode_customs_actor_candidate() {
    let original = "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates.iter().all(|candidate| {
        candidate.source != "PhraseCell32"
            || candidate.text
                != "Поставщик говорит что цена до склада нашего покупателя но таможим мы!"
    }));
}

#[test]
fn phrase_cell_does_not_rewrite_customs_actor_without_right_anchor() {
    let original = "Поставщик говорит что цена до склада нашего покупателя но таможен ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| !candidate.text.contains("таможим")));
}

#[test]
fn phrase_cell_does_not_rewrite_customs_actor_without_domain_context() {
    let original = "я сказал что странно но таможен мы! ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .all(|candidate| !candidate.text.contains("таможим")));
}

#[test]
fn l2_exposes_l3_phrase_forecast_candidate_when_llmwave_is_enabled() {
    let memory = super::super::llmwave::LlmWaveMemory::from_text("на улице опять идёт дождь");
    let path = std::env::temp_dir().join(format!("lay-l2-llmwave-{}.llmw.bin", std::process::id()));
    super::super::llmwave::write_memory_packet(&path, &memory).unwrap();
    std::env::set_var("LAY_LLMWAVE_MEMORY", &path);

    let original = "на улице опять идёт д";
    let l1 = run_l1(original);
    let options = crate::nanda_wave::WaveOptions::default().with_llmwave_shadow(true);
    let candidates = run_l2_with_options(original, &l1, &options);
    std::env::remove_var("LAY_LLMWAVE_MEMORY");
    let _ = std::fs::remove_file(path);

    assert!(candidates.iter().any(|candidate| {
        candidate.source == crate::nanda_wave::PHRASE_FORECAST_CELL
            && candidate.text == "на улице опять идёт дождь"
    }));
}

#[test]
fn grammar_cell_keeps_plural_anchor_phrases() {
    for original in ["первые которые ", "такие условие ", "другие перемнные "]
    {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.source != "GrammarCell32"),
            "plural anchor phrase should not get grammar candidate: {original:?} -> {candidates:?}"
        );
    }
}

#[test]
fn grammar_cell_keeps_neuter_nouns_ending_with_ie() {
    for original in ["обратил внимание ", "срабатывает переварачивание "]
    {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "neuter noun should not get adjective agreement candidate: {original:?} -> {candidates:?}"
            );
    }
}

#[test]
fn grammar_cell_keeps_neutral_clause_context() {
    for original in ["там недоказно ", "что там недоказно "] {
        let l1 = run_l1(original);
        let candidates = run_l2(original, &l1);
        assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.source != "GrammarCell32"),
                "neutral clause should not get grammar agreement candidate: {original:?} -> {candidates:?}"
            );
    }
}

#[test]
fn technical_cell_protects_shell_phrase() {
    let original = "git checkout -b new ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.source == "TechTokenCell32"));
}

#[test]
fn layout_cell_does_not_overrule_teacher_for_plain_ascii() {
    let original = "ola ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates.iter().all(|candidate| {
        !matches!(
            candidate.origin,
            CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
        ) && !matches!(
            candidate.source,
            "LayoutWordCell32" | LAYOUT_THEN_L2_WORD_CENTER
        )
    }));
}

#[test]
fn layout_cell_exposes_known_english_target_even_with_russian_typo_shadow() {
    let original = "вудуеу ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.source == "LayoutWordCell32" && candidate.text == "delete"),
        "known English layout target must survive Russian typo shadow: {candidates:?}"
    );
}

#[test]
fn short_token_cell_exposes_keyboard_and_visual_hypotheses() {
    let original = "пер b ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    assert!(candidates.iter().any(|candidate| {
        candidate.source == "ShortTokenCell32" && candidate.text == "пер и"
    }));
    assert!(candidates.iter().any(|candidate| {
        candidate.source == "ShortTokenCell32" && candidate.text == "пер в"
    }));
}

#[test]
fn short_token_cell_marks_ascii_context_as_risky() {
    let original = "vitamin B ";
    let l1 = run_l1(original);
    let candidates = run_l2(original, &l1);
    let short = candidates
        .iter()
        .find(|candidate| candidate.source == "ShortTokenCell32" && candidate.text == "vitamin И")
        .expect("short token candidate");
    assert!(short.risk >= 0.40);
}
