fn decode_input_gate_tail(
    events: &[KeyEvent],
    context: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<DecodedCompletedTail> {
    let original = lay::keyboard::map_original_events(events);
    let text_tail = if context.ends_with(&original) {
        context
    } else {
        &original
    };
    let gate_config = GateRuntimeConfig::active(allow_layout_auto);
    let decision = lay::input_gate::decide_input_gate(lay::input_gate::InputGateRequest {
        trigger: lay::input_gate::InputGateTrigger::Space,
        text_tail,
        auto_replace: gate_config.auto_replace,
        typing_assist: gate_config.typing_assist,
        auto_switch_layout: gate_config.auto_switch_layout,
        correction_safety: gate_config.correction_safety,
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: gate_config.nanda_autocorrect,
        correction_mode: lay::correction_core::CorrectionMode::DeterministicThenNanda,
    });
    let lay::input_gate::InputGateAction::ApplyReplacement { replacement, .. } = &decision.action
    else {
        return None;
    };
    let replacement = replacement.clone();

    let (original_tail, replacement_tail) = if text_tail == context {
        let prefix = context.strip_suffix(&original)?;
        if prefix.is_empty() {
            (original.as_str(), replacement.as_str())
        } else {
            let replacement_tail = replacement.strip_prefix(prefix)?;
            if prefix.ends_with(char::is_whitespace)
                && replacement_tail.chars().count() > original.chars().count()
            {
                let separator = prefix
                    .chars()
                    .next_back()
                    .expect("prefix ends with whitespace");
                let anchored_original = format!("{separator}{original}");
                let anchored_replacement = format!("{separator}{replacement_tail}");
                return build_input_gate_decoded_tail(
                    decision,
                    &anchored_original,
                    &anchored_replacement,
                );
            }
            (original.as_str(), replacement_tail)
        }
    } else {
        (original.as_str(), replacement.as_str())
    };
    build_input_gate_decoded_tail(decision, original_tail, replacement_tail)
}

fn build_input_gate_decoded_tail(
    decision: lay::input_gate::InputGateDecision,
    original: &str,
    replacement_tail: &str,
) -> Option<DecodedCompletedTail> {
    let mut edit = DecoderEditPlan::committed_tail(
        lay::decoder::CorrectionTrigger::AfterSpace,
        original,
        replacement_tail,
        CorrectionSource::TypingAssist,
    )?;
    if let Some(full_token_plan) = edit.verified_full_token_plan_for_cursor(0) {
        edit.plan = full_token_plan;
    }
    let input_gate = decision
        .trace
        .as_ref()
        .map(lay::action_log::RecentActionGateTrace::from_input_gate);
    let source_id = input_gate
        .as_ref()
        .and_then(|trace| trace.selected_source_id.as_deref());
    let error_class = input_gate
        .as_ref()
        .and_then(|trace| trace.selected_error_class.as_deref());
    let confidence_milli = input_gate
        .as_ref()
        .and_then(|trace| trace.scoreboard.as_ref())
        .and_then(|scoreboard| scoreboard.selected_bayes_posterior_milli)
        .unwrap_or(0);
    let transition = input_gate
        .as_ref()
        .map(lay::action_log::RecentActionGateTrace::selected_transition_audit)
        .unwrap_or_default();
    let edit_action = lay::text_edit::authorize_replacement_with_transition(
        "typing-assist",
        confidence_milli,
        original,
        edit.replacement.as_str(),
        edit.plan.clone(),
        source_id,
        error_class,
        transition,
    );
    if !edit_action.allow_apply() {
        crate::log(&format!(
            "· typing-assist blocked by edit-plan safety: reason={} original={:?} replacement={:?}",
            edit_action.safety_reason(),
            original,
            edit.replacement
        ));
        return None;
    }
    let rule_id = decision
        .correction
        .as_ref()
        .and_then(|resolution| resolution.selected.as_ref())
        .map(|candidate| {
            if candidate.source == lay::correction_core::CorrectionDecisionSource::Nanda {
                lay::typing_assist::NANDA_WAVE_RULE_ID.to_string()
            } else {
                candidate.source_id.clone()
            }
        });
    Some(DecodedCompletedTail::with_input_gate(
        edit, rule_id, input_gate,
    ))
}
