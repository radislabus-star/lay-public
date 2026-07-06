use lay::manual_toggle::{plan_manual_toggle, ManualTogglePlan, ManualToggleRequest};
use lay::text_edit::{VisibleTail, VisibleTailSource};

use super::{ime_bridge, switch_to_target_layout};

pub(super) fn try_manual_toggle(ime_enabled: bool) -> Result<Option<bool>, String> {
    if !ime_enabled {
        return Ok(None);
    }
    let Some(plan) = build_plan_from_visible_tail(ime_bridge::visible_tail()?) else {
        return Ok(None);
    };
    if !ime_bridge::replace_tail_plan(plan.backspaces, &plan.replacement, "ime-manual-toggle")? {
        return Ok(None);
    }
    switch_to_target_layout(plan.target_layout_is_ru)?;
    Ok(Some(plan.target_layout_is_ru))
}

fn build_plan_from_visible_tail(
    (state, text, layout_is_ru): (String, String, bool),
) -> Option<ManualTogglePlan> {
    let source = VisibleTailSource::from_bridge_state(&state)?;
    let request = match source {
        VisibleTailSource::ImeActiveComposition => ManualToggleRequest {
            visible_tail: VisibleTail {
                text: &text,
                source,
            },
            current_layout_is_ru: layout_is_ru,
            recover_missing_initial: false,
            preserve_trailing_whitespace: false,
        },
        VisibleTailSource::ImeCommittedTail => ManualToggleRequest {
            visible_tail: VisibleTail {
                text: &text,
                source,
            },
            current_layout_is_ru: layout_is_ru,
            recover_missing_initial: true,
            preserve_trailing_whitespace: true,
        },
        VisibleTailSource::DaemonWordBuffer => return None,
    };
    plan_manual_toggle(request)
}

#[cfg(test)]
mod tests {
    use super::build_plan_from_visible_tail;

    #[test]
    fn daemon_plans_active_ime_composition_from_visible_tail() {
        let plan = build_plan_from_visible_tail((
            "active:composition".to_string(),
            "ghbdtn".to_string(),
            false,
        ))
        .expect("plan");

        assert_eq!(plan.backspaces, 6);
        assert_eq!(plan.replacement, "привет");
        assert!(plan.target_layout_is_ru);
    }

    #[test]
    fn daemon_plans_committed_tail_with_separator() {
        let plan = build_plan_from_visible_tail((
            "passive:committed-tail".to_string(),
            "работает ".to_string(),
            true,
        ))
        .expect("plan");

        assert_eq!(plan.backspaces, 9);
        assert_eq!(plan.replacement, "hf,jnftn ");
        assert!(!plan.target_layout_is_ru);
    }

    #[test]
    fn daemon_ignores_ime_when_visible_tail_belongs_to_word_buffer() {
        assert!(build_plan_from_visible_tail((
            "passive:daemon-word-buffer".to_string(),
            String::new(),
            true,
        ))
        .is_none());
    }
}
