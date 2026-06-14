use crate::microbrain::{
    decide, generate_candidates, CorrectionCandidate, MicroAction, MicroContext,
    MicroDecisionTrace, MicrobrainOptions,
};

use super::types::{TypingCandidate, TypingCandidateDecision};

const NANDA_GENERATED_PRIORITY: i32 = 88;

pub fn rank_with_nanda<I>(original: &str, candidates: I) -> Option<TypingCandidateDecision>
where
    I: IntoIterator<Item = TypingCandidate>,
{
    rank_with_microbrain(original, candidates, &MicrobrainOptions::default())
        .map(|(decision, _trace)| decision)
}

pub fn rank_with_microbrain<I>(
    original: &str,
    candidates: I,
    options: &MicrobrainOptions,
) -> Option<(TypingCandidateDecision, MicroDecisionTrace)>
where
    I: IntoIterator<Item = TypingCandidate>,
{
    let (decision, trace) = rank_with_microbrain_trace(original, candidates, options);
    decision.map(|decision| (decision, trace))
}

pub fn rank_with_microbrain_trace<I>(
    original: &str,
    candidates: I,
    options: &MicrobrainOptions,
) -> (Option<TypingCandidateDecision>, MicroDecisionTrace)
where
    I: IntoIterator<Item = TypingCandidate>,
{
    let mut candidates: Vec<TypingCandidate> = candidates.into_iter().collect();
    let ctx = MicroContext::new(original);
    let generated: Vec<_> = generate_candidates(&ctx, options)
        .into_iter()
        .filter(|candidate| {
            !candidates
                .iter()
                .any(|existing| existing.replacement == candidate.text)
        })
        .collect();
    for candidate in &generated {
        if !candidates
            .iter()
            .any(|existing| existing.replacement == candidate.text)
        {
            candidates.push(TypingCandidate::new(
                candidate.source,
                NANDA_GENERATED_PRIORITY,
                original,
                candidate.text.clone(),
            ));
        }
    }
    let micro_candidates: Vec<CorrectionCandidate> =
        candidates.iter().map(to_micro_candidate).collect();
    let mut trace = decide(&ctx, &micro_candidates, options);
    trace.generated = generated;
    let Some(chosen_text) = trace.chosen.as_deref() else {
        return (None, trace);
    };
    let Some(best_idx) = candidates
        .iter()
        .position(|candidate| candidate.replacement == chosen_text)
    else {
        return (None, trace);
    };
    let best = candidates[best_idx].clone();

    let second = trace
        .candidates
        .iter()
        .filter(|candidate| candidate.candidate != chosen_text)
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
        .and_then(|micro| {
            candidates
                .iter()
                .find(|candidate| candidate.replacement == micro.candidate)
                .cloned()
        });

    let best_confidence = trace
        .candidates
        .iter()
        .find(|candidate| candidate.candidate == chosen_text)
        .map(|candidate| candidate.confidence as f64)
        .unwrap_or(0.0);
    let second_confidence = trace
        .candidates
        .iter()
        .filter(|candidate| candidate.candidate != chosen_text)
        .map(|candidate| candidate.confidence as f64)
        .fold(0.0f64, f64::max);

    (
        Some(TypingCandidateDecision {
            best,
            second,
            margin: best_confidence - second_confidence,
        }),
        trace,
    )
}

fn to_micro_candidate(candidate: &TypingCandidate) -> CorrectionCandidate {
    CorrectionCandidate {
        action: action_from_rule(&candidate.rule_id),
        text: candidate.replacement.clone(),
        source: candidate.rule_id.clone(),
        engine_score: Some(candidate.score.total),
    }
}

fn action_from_rule(rule_id: &str) -> MicroAction {
    if rule_id.contains("protect") {
        MicroAction::Protect
    } else if rule_id.contains("ru_to_en") || rule_id.contains("technical") {
        MicroAction::LayoutRuToEn
    } else if rule_id.contains("en_to_ru") {
        MicroAction::LayoutEnToRu
    } else if rule_id.contains("split") || rule_id.contains("glued") || rule_id.contains("prefix") {
        MicroAction::SplitGlue
    } else if rule_id.contains("personal") || rule_id.contains("exact") {
        MicroAction::Keep
    } else {
        MicroAction::TypoFix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_candidate::TypingCandidate;
    use crate::typing_rule_graph::ids;

    #[test]
    fn nanda_accepts_direct_layout_prediction() {
        let candidate =
            TypingCandidate::new(ids::EXPERIMENTAL_LAYOUT_EN_TO_RU, 1, "djn ", "вот ".into());
        let decision = rank_with_nanda("djn ", [candidate]).expect("decision");
        assert_eq!(decision.best.replacement, "вот ");
    }

    #[test]
    fn nanda_rejects_boundary_removal_without_strong_signal() {
        let candidate = TypingCandidate::new(ids::GLUED_PHRASE, 1, "слов и ", "слови ".into());
        assert!(rank_with_nanda("слов и ", [candidate]).is_none());
    }

    #[test]
    fn nanda_supports_expert_ablation() {
        let candidate =
            TypingCandidate::new(ids::EXPERIMENTAL_LAYOUT_EN_TO_RU, 1, "djn ", "вот ".into());
        let (_decision, trace) = rank_with_microbrain(
            "djn ",
            [candidate],
            &MicrobrainOptions::with_disabled(&["cli_guard_16k_stub".to_string()]),
        )
        .expect("decision");
        assert_eq!(trace.disabled_experts, vec!["cli_guard_16k_stub"]);
    }

    #[test]
    fn nanda_writer_can_create_layout_candidate_without_deterministic_input() {
        let (decision, trace) =
            rank_with_microbrain("djn ", std::iter::empty(), &MicrobrainOptions::default())
                .expect("writer candidate");
        assert_eq!(decision.best.replacement, "вот ");
        assert_eq!(trace.generated.len(), 1);
        assert_eq!(
            trace.generated[0].source,
            "nanda_writer_layout_en_to_ru_64k"
        );
    }

    #[test]
    fn nanda_guard_allows_short_russian_function_layout_candidate() {
        let candidate =
            TypingCandidate::new(ids::EXPERIMENTAL_LAYOUT_EN_TO_RU, 1, "yt ", "не ".into());
        let (decision, _trace) =
            rank_with_microbrain("yt ", [candidate], &MicrobrainOptions::default())
                .expect("short function layout");
        assert_eq!(decision.best.replacement, "не ");
    }

    #[test]
    fn nanda_writer_does_not_flip_known_ascii_technical_tokens_to_cyrillic() {
        for original in [
            "apt-get",
            "systemctl",
            "journalctl",
            "ssh-keygen",
            "python3",
        ] {
            assert!(
                rank_with_microbrain(original, std::iter::empty(), &MicrobrainOptions::default())
                    .is_none(),
                "original={original:?}"
            );
        }
    }

    #[test]
    fn nanda_guard_vetoes_existing_layout_candidate_for_known_ascii_token() {
        let candidate = TypingCandidate::new(
            ids::EXPERIMENTAL_LAYOUT_EN_TO_RU,
            1,
            "systemctl",
            "ыныеуьсед".into(),
        );
        assert!(
            rank_with_microbrain("systemctl", [candidate], &MicrobrainOptions::default()).is_none()
        );
    }

    #[test]
    fn nanda_writer_can_create_long_russian_layout_candidate() {
        let (decision, trace) = rank_with_microbrain(
            "gthtdfhfxbdftn ",
            std::iter::empty(),
            &MicrobrainOptions::default(),
        )
        .expect("writer candidate");
        assert_eq!(decision.best.replacement, "переварачивает ");
        assert_eq!(trace.generated.len(), 1);
    }

    #[test]
    fn nanda_writer_does_not_flip_normal_russian_words_to_ascii() {
        for original in ["весь ", "написан ", "влиянием ", "почему ", "строке "]
        {
            assert!(
                rank_with_microbrain(original, std::iter::empty(), &MicrobrainOptions::default(),)
                    .is_none(),
                "original={original:?}"
            );
        }
    }

    #[test]
    fn nanda_writer_does_not_flip_cyrillic_acronyms_to_ascii() {
        for original in ["ИНН ", "НДС ", "ОГРН "] {
            assert!(
                rank_with_microbrain(original, std::iter::empty(), &MicrobrainOptions::default(),)
                    .is_none(),
                "original={original:?}"
            );
        }
    }

    #[test]
    fn nanda_writer_can_recover_known_ascii_token_typed_in_ru_layout() {
        let (decision, trace) =
            rank_with_microbrain("цусрфе ", std::iter::empty(), &MicrobrainOptions::default())
                .expect("writer candidate");
        assert_eq!(decision.best.replacement, "wechat ");
        assert_eq!(
            trace.generated[0].source,
            "nanda_writer_layout_ru_to_en_64k"
        );
    }

    #[test]
    fn nanda_writer_does_not_flip_cyrillic_noise_to_unknown_ascii_noise() {
        for original in ["свло ", "м ", "кция ", "ть "] {
            assert!(
                rank_with_microbrain(original, std::iter::empty(), &MicrobrainOptions::default(),)
                    .is_none(),
                "original={original:?}"
            );
        }
    }

    #[test]
    fn nanda_writer_does_not_guess_multiword_phrase_yet() {
        assert!(rank_with_microbrain(
            "ghjujyzq ntcns ",
            std::iter::empty(),
            &MicrobrainOptions::default(),
        )
        .is_none());
    }
}
