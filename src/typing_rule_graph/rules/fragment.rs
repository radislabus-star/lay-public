//! Automatic candidate policy only. No manual conversion or field delivery.

use crate::dict::{convert, Direction};
use crate::layout_fragments::{analyze, Boundary, Selection};
use crate::typing_rule_graph::{ids, TypingRuleContext};

use super::helpers::{
    apply_last_physical_layout_token_rule, apply_last_trailing_layout_token_rule,
    apply_last_word_rule, last_word_has_protected_ascii_context, TextRule,
};

type Rule = for<'a> fn(&TypingRuleContext<'a>) -> Option<String>;

pub(crate) fn apply_automatic_layout_rule(
    ctx: &TypingRuleContext<'_>,
    id: &str,
    legacy: Rule,
) -> Option<String> {
    apply_with_mode(
        ctx,
        id,
        legacy,
        crate::config::runtime_fragment_layout_enabled(),
    )
}

fn apply_with_mode(
    ctx: &TypingRuleContext<'_>,
    id: &str,
    legacy: Rule,
    enabled: bool,
) -> Option<String> {
    if !enabled {
        return legacy(ctx);
    }
    let (rule, direction): (TextRule, Direction) = match id {
        ids::FAST_LAYOUT_EN_TO_RU
        | ids::LAYOUT_EN_TO_RU
        | ids::CONTEXTUAL_LAYOUT_EN_TO_RU
        | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU => (en_to_ru, Direction::Us2Ru),
        ids::LAYOUT_RU_TO_EN | ids::EXPERIMENTAL_LAYOUT_RU_TO_EN => (ru_to_en, Direction::Ru2Us),
        _ => return legacy(ctx),
    };
    if !ctx.allow_layout_auto || last_word_has_protected_ascii_context(ctx) {
        return None;
    }
    apply_last_physical_layout_token_rule(ctx, rule)
        .or_else(|| apply_last_trailing_layout_token_rule(ctx, rule))
        .or_else(|| apply_last_word_rule(ctx, rule))
        .or_else(|| {
            let replacement = legacy(ctx)?;
            safe_exact_legacy_candidate(ctx, &replacement, direction).then_some(replacement)
        })
}

fn safe_exact_legacy_candidate(
    ctx: &TypingRuleContext<'_>,
    replacement: &str,
    direction: Direction,
) -> bool {
    let Some(token) = ctx.core.split_whitespace().next_back() else {
        return false;
    };
    let Some(prefix) = ctx.core.strip_suffix(token) else {
        return false;
    };
    let Some(replacement_token) = replacement.strip_prefix(prefix) else {
        return false;
    };
    replacement_token == convert(token, direction)
        && !crate::word_recognizer::is_plain_layout_autocorrect_risky(token, replacement_token)
}

fn propose(token: &str, direction: Selection) -> Option<String> {
    if crate::word_recognizer::is_protected_ascii_token(token) {
        return None;
    }
    let analysis = analyze(token, Boundary::Closed)?;
    if direction == Selection::English
        && crate::layout_autoswitch::english_layout_target_requires_context(&analysis.english)
    {
        return None;
    }
    (analysis.selection == direction)
        .then(|| analysis.replacement(token))
        .flatten()
}

fn en_to_ru(token: &str) -> Option<String> {
    propose(token, Selection::Russian)
}
fn ru_to_en(token: &str) -> Option<String> {
    propose(token, Selection::English)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word_reader::split_word_punctuation;

    #[test]
    fn fragment_mode_retains_safe_exact_ordinary_layout_candidates() {
        use crate::config::{default_typing_assist_pipeline, publish_runtime_config, LayConfig};
        use crate::correction_core::{
            resolve_text_correction, CandidateReadoutRoute, CorrectionMode, CorrectionRequest,
        };
        fn correction(input: &str, fragments: bool) -> Option<String> {
            publish_runtime_config(&LayConfig {
                fragment_layout_autoflip: fragments,
                ..LayConfig::default()
            });
            let pipeline = default_typing_assist_pipeline();
            resolve_text_correction(CorrectionRequest {
                text: input,
                lexical_authority_frame: None,
                auto_replace: false,
                typing_assist: false,
                auto_switch_layout: true,
                correction_safety: crate::config::CorrectionSafety::Normal,
                typing_assist_pipeline: &pipeline,
                nanda_autocorrect: false,
                nanda_candidate_route: CandidateReadoutRoute::FullWave,
                nanda_wave_options: crate::nanda_wave::WaveOptions::default(),
                mode: CorrectionMode::DeterministicOnly,
            })
            .decision
            .map(|decision| decision.replacement)
        }
        for (input, target) in [
            ("фиикум", "abbrev"),
            ("злцн", "pkwy"),
            ("ручнд", "hexyl"),
            ("цгтве", "wundt"),
        ] {
            assert_eq!(correction(input, false).as_deref(), Some(target));
            assert_eq!(
                correction(input, true).as_deref(),
                Some(target),
                "optional fragment abstention must retain safe exact ordinary result: {input}"
            );
        }
        for original in ["rdy", "зфьудшф", "гайдуке", "abbrev"] {
            assert_eq!(
                correction(original, true),
                None,
                "unsafe or nonexact fallback must not gain authority: {original}"
            );
        }
    }

    fn context(core: &str, allow_layout_auto: bool) -> TypingRuleContext<'_> {
        let (token_leading, word, token_trailing) = split_word_punctuation(core);
        TypingRuleContext {
            core,
            word,
            token_leading,
            token_trailing,
            allow_layout_auto,
        }
    }
    fn legacy(_: &TypingRuleContext<'_>) -> Option<String> {
        Some("legacy".into())
    }

    #[test]
    fn fragment_mode_off_preserves_legacy_dispatch_and_other_rules() {
        let ctx = context("ghbdtn", true);
        assert_eq!(
            apply_with_mode(&ctx, ids::FAST_LAYOUT_EN_TO_RU, legacy, false).as_deref(),
            Some("legacy")
        );
        assert_eq!(
            apply_with_mode(&ctx, "non_layout_rule", legacy, true).as_deref(),
            Some("legacy")
        );
    }

    #[test]
    fn fragment_mode_preserves_previous_words_case_and_punctuation() {
        for (input, expected) in [("ghbdtn", "привет"), ("Уже  Ghbdtn!", "Уже  Привет!")]
        {
            assert_eq!(
                apply_with_mode(
                    &context(input, true),
                    ids::FAST_LAYOUT_EN_TO_RU,
                    legacy,
                    true
                )
                .as_deref(),
                Some(expected)
            );
        }
        assert_eq!(
            apply_with_mode(&context("руддщ", true), ids::LAYOUT_RU_TO_EN, legacy, true).as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn fragment_mode_obeys_master_switch_and_protected_context() {
        for (input, allow) in [
            ("ghbdtn", false),
            ("https://example.org", true),
            ("https://example.org ghbdtn", true),
        ] {
            assert_eq!(
                apply_with_mode(
                    &context(input, allow),
                    ids::FAST_LAYOUT_EN_TO_RU,
                    legacy,
                    true
                ),
                None
            );
        }
    }

    #[test]
    fn fragment_mode_projects_physical_edge_letters_before_stripping_punctuation() {
        for word in ["брачку", "живет", "хорошо", "даю", "люблю"] {
            let input = crate::dict::convert(word, crate::dict::Direction::Ru2Us);
            let result = apply_with_mode(
                &context(&input, true),
                ids::FAST_LAYOUT_EN_TO_RU,
                legacy,
                true,
            );
            assert_eq!(result.as_deref(), Some(word), "physical token {input:?}");
        }
    }

    #[test]
    fn fragment_mode_reuses_short_english_target_context_requirement() {
        assert_eq!(
            analyze("фее", Boundary::Closed).unwrap().selection,
            Selection::English
        );
        for input in ["фее", "снова фее"] {
            assert_eq!(
                apply_with_mode(&context(input, true), ids::LAYOUT_RU_TO_EN, legacy, true),
                None
            );
        }
        assert_eq!(ru_to_en("пше").as_deref(), Some("git"));
    }
}
