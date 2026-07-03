#[cfg(not(test))]
use super::super::super::{
    active_auto_replace, active_auto_switch_layout, active_correction_safety,
    active_nanda_autocorrect, active_typing_assist, active_typing_assist_pipeline_for_auto_replace,
};

struct GateRuntimeConfig {
    auto_replace: bool,
    typing_assist: bool,
    auto_switch_layout: bool,
    nanda_autocorrect: bool,
    correction_safety: lay::config::CorrectionSafety,
}

impl GateRuntimeConfig {
    fn active(allow_layout_auto: bool) -> Self {
        Self {
            auto_replace: active_auto_replace_for_gate(),
            typing_assist: active_typing_assist_for_gate(),
            auto_switch_layout: allow_layout_auto && active_auto_switch_layout_for_gate(),
            nanda_autocorrect: active_nanda_autocorrect_for_gate(),
            correction_safety: active_correction_safety_for_gate(),
        }
    }
}

#[cfg(test)]
fn active_pipeline(context: &str) -> Vec<TypingAssistRuleConfig> {
    lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &lay::config::default_typing_assist_pipeline(),
        context,
    )
}

#[cfg(not(test))]
fn active_pipeline(context: &str) -> Vec<TypingAssistRuleConfig> {
    active_typing_assist_pipeline_for_auto_replace(context)
}

#[cfg(test)]
fn active_auto_replace_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_auto_replace_for_gate() -> bool {
    active_auto_replace()
}

#[cfg(test)]
fn active_typing_assist_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_typing_assist_for_gate() -> bool {
    active_typing_assist()
}

#[cfg(test)]
fn active_auto_switch_layout_for_gate() -> bool {
    true
}

#[cfg(not(test))]
fn active_auto_switch_layout_for_gate() -> bool {
    active_auto_switch_layout()
}

#[cfg(test)]
fn active_nanda_autocorrect_for_gate() -> bool {
    false
}

#[cfg(not(test))]
fn active_nanda_autocorrect_for_gate() -> bool {
    active_nanda_autocorrect()
}

#[cfg(test)]
fn active_correction_safety_for_gate() -> lay::config::CorrectionSafety {
    lay::config::CorrectionSafety::Normal
}

#[cfg(not(test))]
fn active_correction_safety_for_gate() -> lay::config::CorrectionSafety {
    active_correction_safety()
}
