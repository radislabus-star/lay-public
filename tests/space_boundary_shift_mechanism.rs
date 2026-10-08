//! The final pair is semantic evidence; a current-word lease cannot edit it.

use lay::config::{CorrectionSafety, LayConfig};
use lay::correction_core::{live_correction_mode, CandidateReadoutRoute, TypingErrorClass};
use lay::ime_correction::{
    decide_active_composition_autocorrect_observed, decide_space_autocorrect_observed_if_current,
    decide_space_autocorrect_observed_with_pair, ActiveCompositionAutocorrectRequest,
    AutocorrectNoApplyStage,
};
use lay::input_gate::{decide_input_gate, InputGateAction, InputGateRequest, InputGateTrigger};
use lay::text_edit::{
    plan_input_gate_edit, plan_space_boundary_edit, TextReplacement, TransitionOperator,
    TransitionProof,
};

const SHIFTS: &[(&str, &str)] = &[
    ("должн ыбыть ", "должны быть "),
    ("план ыработают ", "планы работают "),
    ("расчет ыготовы ", "расчеты готовы "),
    ("допусти мнабираю ", "допустим набираю "),
];

#[path = "common/space_boundary_lexical.rs"]
mod lexical_fixture;

fn config() -> LayConfig {
    lay::hot_field::set_process_policy(lay::hot_field::HotFieldPolicy::ime());
    lexical_fixture::warm_lexical_fixture();
    LayConfig {
        text_backend: "ime".to_string(),
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        nanda_autocorrect: true,
        correction_safety: "normal".to_string(),
        ..LayConfig::default()
    }
}

fn decide(text: &str, cfg: &LayConfig) -> lay::input_gate::InputGateDecision {
    decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Space,
        text_tail: text,
        lexical_authority_frame: None,
        auto_replace: cfg.auto_replace,
        typing_assist: cfg.typing_assist,
        auto_switch_layout: cfg.auto_switch_layout,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: &cfg.typing_assist_pipeline,
        nanda_autocorrect: cfg.nanda_autocorrect,
        nanda_candidate_route: CandidateReadoutRoute::live_default(),
        nanda_wave_options: cfg.active_nanda_wave_options(),
        correction_mode: live_correction_mode(cfg.nanda_autocorrect),
    })
}

#[test]
fn selected_boundary_pair_projects_pending_space_through_the_same_receipt() {
    let cfg = config();
    let mut failures = Vec::new();
    // Ordinary selection runs first. Exact boundary evidence settles abstention
    // or a competing letter mutation within the same DecisionCore.
    for &(input, expected) in SHIFTS {
        let decision = decide(input, &cfg);
        let selected = decision
            .correction
            .as_ref()
            .and_then(|resolution| resolution.selected.as_ref());
        let matched = matches!(
            &decision.action,
            InputGateAction::ApplyReplacement { replacement, .. } if replacement == expected
        ) && selected.is_some_and(|candidate| {
            candidate.replacement == expected
                && candidate.error_class == TypingErrorClass::BoundaryShift
        });
        if !matched {
            #[cfg(feature = "research-tools")]
            {
                let mut surfaces = input
                    .split_whitespace()
                    .chain(expected.split_whitespace())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                surfaces.extend(["расчёт".to_string(), "расчёты".to_string()]);
                eprintln!(
                    "exact_form_readings={}",
                    lay::nanda_wave::probe_cached_morphology_slots(&surfaces).unwrap()
                );
                eprintln!(
                    "boundary_lattice={:?}",
                    decision.correction.as_ref().map(|resolution| {
                        resolution
                            .candidates
                            .iter()
                            .filter(|candidate| {
                                candidate.error_class == TypingErrorClass::BoundaryShift
                            })
                            .map(|candidate| (&candidate.replacement, &candidate.gate))
                            .collect::<Vec<_>>()
                    })
                );
            }
            failures.push(format!(
                "{input:?}: selected={:?}, gate={:?}",
                selected.map(|candidate| (&candidate.replacement, candidate.error_class)),
                decision.action
            ));
            continue;
        }
        let original = input.trim_end_matches(char::is_whitespace);
        let bare_action = plan_input_gate_edit(
            "boundary-pair-contract",
            original,
            expected,
            TextReplacement {
                move_left: 0,
                backspaces: original.chars().count() as u32,
                insert: expected.to_string(),
                move_right: 0,
            },
            &decision,
        );
        assert!(
            !bare_action.allow_apply(),
            "bare whitespace change gained authority"
        );
        let Some(action) =
            plan_space_boundary_edit("boundary-pair-contract", original, expected, &decision)
        else {
            failures.push(format!(
                "{input:?}: missing sealed pending-Space projection"
            ));
            continue;
        };
        if !action.allow_apply()
            || action.transition().operator() != Some(TransitionOperator::BoundaryShift)
            || action.transition().proof() != Some(TransitionProof::Boundary)
            || action.transition().changed_tokens() != Some(2)
        {
            failures.push(format!("{input:?}: unexecutable pair action={action:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn competing_or_suggested_shift_stays_in_the_shared_lattice() {
    let cfg = config();
    for &(input, expected) in [SHIFTS[1], SHIFTS[3]].iter() {
        let decision = decide(input, &cfg);
        let resolution = decision.correction.as_ref().expect("shared field");
        assert!(
            resolution.candidates.iter().any(|candidate| {
                candidate.replacement == expected
                    && candidate.error_class == TypingErrorClass::BoundaryShift
            }),
            "grounded shift was lost: {input:?}"
        );
    }
}

#[test]
fn exact_pair_request_preserves_winner_and_one_space() {
    let cfg = config();
    let input = SHIFTS[0].0;
    let tail = input.strip_suffix(' ').unwrap();
    let request = || ActiveCompositionAutocorrectRequest {
        text: "ыбыть ",
        committed_tail: tail,
        config: &cfg,
        lexical_authority_frame: None,
        active_layout_is_ru: Some(true),
    };
    assert!(
        decide_space_autocorrect_observed_if_current(request(), None, Some(tail), &|| false,)
            .is_none(),
        "cancelled computation produced a terminal result"
    );
    // The next request arrives just after the first liveness observation.
    // The older computation must stop at a subsequent stage, without turning
    // cancellation into a ranked refusal or issuing an edit receipt.
    let newer_request_arrived = std::cell::Cell::new(false);
    assert!(
        decide_space_autocorrect_observed_if_current(request(), None, Some(tail), &|| {
            !newer_request_arrived.replace(true)
        },)
        .is_none(),
        "superseded computation continued through selection"
    );
    let observed =
        decide_space_autocorrect_observed_if_current(request(), None, Some(tail), &|| true)
            .expect("current request must complete");
    let ordinary = decide_space_autocorrect_observed_with_pair(request(), None, tail);
    let decision = observed
        .decision
        .expect("shared winner with exact pair projection");
    let ordinary = ordinary.decision.expect("ordinary uncancelled decision");
    assert_eq!(decision.replacement, ordinary.replacement);
    assert_eq!(decision.action.plan(), ordinary.action.plan());
    assert_eq!(decision.action.allow_apply(), ordinary.action.allow_apply());
    assert_eq!(decision.replacement, SHIFTS[0].1);
    assert_eq!(decision.action.from_text(), tail);
    assert!(decision.action.allow_apply());
    assert_eq!(decision.action.plan().unwrap().backspaces, 11);
    let gate = decide(input, &cfg);
    for (from, to) in [
        (tail, "должны быть  "),
        (tail, "должны быть\t"),
        ("должн ыбыть ", SHIFTS[0].1),
        ("чужое ыбыть", SHIFTS[0].1),
        (tail, "должны делать "),
    ] {
        assert!(plan_space_boundary_edit("invalid-projection", from, to, &gate).is_none());
    }
}

#[test]
fn recorded_pair_inverse_is_bound_to_the_forward_receipt() {
    let cfg = config();
    let gate = decide(SHIFTS[0].0, &cfg);
    let forward = plan_space_boundary_edit("pair", "должн ыбыть", SHIFTS[0].1, &gate)
        .expect("forward BoundaryShift");
    let inverse = lay::text_edit::plan_recorded_boundary_inverse(&forward).expect("exact inverse");
    assert!(inverse.allow_apply());
    assert_eq!(inverse.from_text(), "должны быть ");
    assert_eq!(inverse.to_text(), "должн ыбыть ");
    let committed = forward
        .clone()
        .with_committed_boundary_space()
        .expect("same sealed semantic action after actual Space");
    assert_eq!(committed.from_text(), "должн ыбыть ");
    assert_eq!(committed.to_text(), forward.to_text());
    assert_eq!(committed.plan().unwrap().backspaces, 12);
    assert!(committed.allow_apply());
    assert!(
        committed.with_committed_boundary_space().is_none(),
        "cannot consume Space twice"
    );
    assert!(lay::text_edit::EditAction::keep("bare", "должн ыбыть")
        .with_committed_boundary_space()
        .is_none());

    assert_eq!(
        inverse.transition().operator(),
        Some(TransitionOperator::Undo)
    );
    let bare = lay::text_edit::plan_recorded_undo_edit(
        inverse.from_text(),
        inverse.to_text(),
        inverse.plan().unwrap().clone(),
        2,
    );
    assert!(
        !bare.allow_apply(),
        "generic two-word undo acquired authority"
    );
    assert!(lay::text_edit::plan_recorded_boundary_inverse(&bare).is_none());
    assert!(lay::text_edit::plan_recorded_boundary_inverse(&inverse).is_none());
}

#[test]
fn clean_right_words_and_unsupported_transfers_never_become_boundary_edits() {
    let cfg = config();
    let mut failures = Vec::new();
    for input in [
        "должны быть ",
        "планы работают ",
        "расчеты готовы ",
        "допустим набираю ",
        "план работает ",
        "должен ыбыть ",
        "план ыжжж ",
    ] {
        let decision = decide(input, &cfg);
        if let InputGateAction::ApplyReplacement { replacement, .. } = &decision.action {
            if plan_space_boundary_edit(
                "negative-pair-permission",
                input.strip_suffix(' ').unwrap(),
                replacement,
                &decision,
            )
            .is_some()
            {
                failures.push(format!("{input:?}: acquired physical pair permission"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let invalid = decide("должен ыбыть ", &cfg);
    let InputGateAction::ApplyReplacement { replacement, .. } = &invalid.action else {
        panic!(
            "invalid-target semantic control missing: {:?}",
            invalid.action
        );
    };
    assert_eq!(replacement, "должены быть ");
    assert!(invalid
        .correction
        .as_ref()
        .unwrap()
        .selected
        .as_ref()
        .is_some_and(|candidate| candidate.error_class == TypingErrorClass::BoundaryShift));
    let semantic = plan_input_gate_edit(
        "invalid-target-semantic-control",
        "должен ыбыть ",
        replacement,
        TextReplacement {
            move_left: 0,
            backspaces: "должен ыбыть ".chars().count() as u32,
            insert: replacement.clone(),
            move_right: 0,
        },
        &invalid,
    );
    assert!(
        semantic.allow_apply(),
        "invalid-target control has no semantic authority"
    );
    assert!(plan_space_boundary_edit(
        "invalid-target-pair-permission",
        "должен ыбыть",
        replacement,
        &invalid,
    )
    .is_none());
    let refused = decide_space_autocorrect_observed_with_pair(
        ActiveCompositionAutocorrectRequest {
            text: "ыбыть ",
            committed_tail: "должен ыбыть",
            config: &cfg,
            lexical_authority_frame: None,
            active_layout_is_ru: Some(true),
        },
        None,
        "должен ыбыть",
    );
    assert!(refused.decision.is_none());
    assert_eq!(
        refused.no_apply_stage,
        Some(AutocorrectNoApplyStage::Verifier)
    );
}

#[test]
fn current_word_scope_cannot_apply_last_pair_from_read_only_context() {
    let cfg = config();
    let mut failures = Vec::new();
    for &(input, _) in SHIFTS {
        let tail = input.trim_end_matches(char::is_whitespace);
        let token = tail.split_whitespace().last().expect("right token");
        let active = format!("{token} ");
        let observed =
            decide_active_composition_autocorrect_observed(ActiveCompositionAutocorrectRequest {
                text: &active,
                committed_tail: tail,
                config: &cfg,
                lexical_authority_frame: None,
                active_layout_is_ru: Some(true),
            });
        let expected_stage = if matches!(
            decide(input, &cfg).action,
            InputGateAction::ApplyReplacement { .. }
        ) {
            AutocorrectNoApplyStage::Verifier
        } else {
            AutocorrectNoApplyStage::Rank
        };
        if observed.decision.is_some() || observed.no_apply_stage != Some(expected_stage) {
            failures.push(format!(
                "{input:?}: prepared={:?}, stage={:?}",
                observed
                    .decision
                    .as_ref()
                    .map(|decision| &decision.replacement),
                observed.no_apply_stage
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn canonical_known_right_form_cannot_supply_a_moved_prefix() {
    const CHILD: &str = "LAY_KNOWN_RIGHT_CONTROL_CHILD";
    if std::env::var_os(CHILD).is_none() {
        // The alternate synthetic provider positively attests the right word;
        // all reconstruction targets remain independently present.
        let bytes = include_bytes!("fixtures/space_boundary_known_right_v2.bin");
        let path = std::env::temp_dir().join(format!(
            "lay-known-right-control-{}.bin",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "canonical_known_right_form_cannot_supply_a_moved_prefix",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("LAY_L2_PACKAGE", &path)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let cfg = config();
    #[cfg(feature = "research-tools")]
    {
        let evidence =
            lay::nanda_wave::probe_cached_morphology_slots(&["ыбыть".to_string()]).unwrap();
        assert!(
            !evidence[0]["readings"].as_array().unwrap().is_empty(),
            "provider-positive control"
        );
    }
    // The general semantic candidate and its original verifier remain valid.
    // Independent positive evidence for the observed right form refuses only
    // the newly introduced physical pair permission.
    let decision = decide("должн ыбыть ", &cfg);
    let InputGateAction::ApplyReplacement { replacement, .. } = &decision.action else {
        panic!(
            "controlled lossless core winner missing: {:?}",
            decision.action
        );
    };
    assert_eq!(replacement, "должны быть ");
    assert!(decision
        .correction
        .as_ref()
        .unwrap()
        .selected
        .as_ref()
        .is_some_and(|candidate| candidate.error_class == TypingErrorClass::BoundaryShift));
    let semantic = plan_input_gate_edit(
        "known-right-semantic-control",
        "должн ыбыть ",
        replacement,
        TextReplacement {
            move_left: 0,
            backspaces: "должн ыбыть ".chars().count() as u32,
            insert: replacement.clone(),
            move_right: 0,
        },
        &decision,
    );
    assert!(
        semantic.allow_apply(),
        "control receipt has no semantic authority"
    );
    assert!(plan_space_boundary_edit(
        "known-right-pair-permission",
        "должн ыбыть",
        replacement,
        &decision,
    )
    .is_none());
    let refused = decide_space_autocorrect_observed_with_pair(
        ActiveCompositionAutocorrectRequest {
            text: "ыбыть ",
            committed_tail: "должн ыбыть",
            config: &cfg,
            lexical_authority_frame: None,
            active_layout_is_ru: Some(true),
        },
        None,
        "должн ыбыть",
    );
    assert!(refused.decision.is_none());
    assert_eq!(
        refused.no_apply_stage,
        Some(AutocorrectNoApplyStage::Verifier)
    );
}
