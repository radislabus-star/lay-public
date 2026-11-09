//! Boundary rules always retain their original policy. The optional letter
//! detector lives in layout_fragments::on_letter and cannot veto these rules.

use crate::typing_rule_graph::TypingRuleContext;

type Rule = for<'a> fn(&TypingRuleContext<'a>) -> Option<String>;

pub(crate) fn apply_automatic_layout_rule(
    ctx: &TypingRuleContext<'_>,
    _id: &str,
    legacy: Rule,
) -> Option<String> {
    legacy(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{default_typing_assist_pipeline, publish_runtime_config, LayConfig};
    use crate::correction_core::{
        resolve_text_correction, CandidateReadoutRoute, CorrectionMode, CorrectionRequest,
    };
    use crate::typing_rule_graph::ids;
    use crate::word_reader::split_word_punctuation;

    #[test]
    fn fragment_option_cannot_replace_or_veto_space_layout_rules() {
        fn original(_: &TypingRuleContext<'_>) -> Option<String> {
            Some("unchanged-policy".into())
        }
        let core = "ghbdtn";
        let (token_leading, word, token_trailing) = split_word_punctuation(core);
        let context = TypingRuleContext {
            core,
            word,
            token_leading,
            token_trailing,
            allow_layout_auto: true,
        };
        for enabled in [false, true] {
            publish_runtime_config(&LayConfig {
                fragment_layout_autoflip: enabled,
                ..LayConfig::default()
            });
            for id in [
                ids::FAST_LAYOUT_EN_TO_RU,
                ids::LAYOUT_EN_TO_RU,
                ids::LAYOUT_RU_TO_EN,
                ids::CONTEXTUAL_LAYOUT_EN_TO_RU,
            ] {
                assert_eq!(
                    apply_automatic_layout_rule(&context, id, original).as_deref(),
                    Some("unchanged-policy")
                );
            }
        }
        publish_runtime_config(&LayConfig::default());
    }

    #[test]
    fn fragment_option_preserves_complete_boundary_decisions() {
        fn correction(text: &str, enabled: bool) -> Option<String> {
            publish_runtime_config(&LayConfig {
                fragment_layout_autoflip: enabled,
                ..LayConfig::default()
            });
            let pipeline = default_typing_assist_pipeline();
            resolve_text_correction(CorrectionRequest {
                text,
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
        for input in [
            "ghbdtn",
            "hello",
            "привет",
            "руддщ",
            "фиикум",
            "злцн",
            "ручнд",
            "цгтве",
            "rdy",
            "зфьудшф",
            "гайдуке",
            "abbrev",
            "фее",
            "пше",
            "снова ghbdtn",
            "етбя",
        ] {
            assert_eq!(
                correction(input, false),
                correction(input, true),
                "Space policy changed: {input}"
            );
        }
        publish_runtime_config(&LayConfig::default());
    }
}
