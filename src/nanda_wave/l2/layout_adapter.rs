use super::super::context::{TailContext, TokenKind};
use super::super::signal::{WavePacket, WordCandidate};
use super::surface::surface_motif_memory;
use super::{candidate_support, l1_energy};
use crate::candidate_contract::CandidateOrigin;
use crate::dict::{convert, detect_direction};
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    is_common_en_guard_prefix, is_common_en_technical_word, is_ru_live_protected_word,
    is_user_protected_word, visual_b_after_ascii_replacement, visual_b_default_replacement,
};
use crate::text_case::apply_word_case;
use crate::text_metrics::damerau_levenshtein;

const MAX_LAYOUT_SCAN_CANDIDATES: usize = 4;
pub(super) const LAYOUT_THEN_L2_WORD_CENTER: &str = "layout_then_l2_word_center";
pub(super) const LAYOUT_SEQUENCE_CELL: &str = "LayoutSequenceCell32";

pub(super) fn layout_sequence_candidate(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Option<WordCandidate> {
    if !(2..=15).contains(&context.token_count()) {
        return None;
    }
    let converted = crate::layout_autoswitch::correct_wrong_layout_ascii_phrase(tail)?;
    if converted == tail {
        return None;
    }

    let raw_projection = convert(tail, crate::dict::Direction::Us2Ru);
    let center_settled = raw_projection != converted;
    let mut support = candidate_support(l1, context);
    support.push("layout-sequence:all-token-projection".to_string());
    if center_settled {
        support.push("layout-sequence:l2-form-center".to_string());
    }
    Some(WordCandidate {
        text: converted,
        origin: if center_settled {
            CandidateOrigin::LayoutThenTypo
        } else {
            CandidateOrigin::Layout
        },
        source: LAYOUT_SEQUENCE_CELL,
        energy: l1_energy(l1, "KeyboardCell32").max(0.92),
        risk: if center_settled { 0.08 } else { 0.04 },
        support,
    })
}

pub(super) fn layout_candidate(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Option<WordCandidate> {
    layout_candidate_with_projection_policy(prefix, token, context, l1, true)
}

pub(super) fn layout_candidate_with_projection_policy(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
    allow_noisy_projection: bool,
) -> Option<WordCandidate> {
    if token.chars().count() < 2 {
        return None;
    }
    if is_common_en_technical_word(&token.to_ascii_lowercase()) {
        return None;
    }
    if technical_context_blocks_layout(prefix, token) {
        return None;
    }
    let (converted, strong_autoswitch, word_center_settled) =
        layout_converted_token(token, allow_noisy_projection)?;
    if converted == token {
        return None;
    }
    // L4 records rank candidates after they exist; it is not lexical evidence.
    // In particular, a historical accept must never create a short layout
    // candidate whose target has no compact L1/L2 center.
    let projection_supported = strong_autoswitch || word_center_settled;
    // A clean, already-known English surface is its own stable L2 center. Do
    // not let an accidental keyboard projection pull it into a Russian center
    // unless accepted transition memory or a strong layout signal says so.
    if input_has_settled_phase(token) && !strong_autoswitch {
        return None;
    }
    if context.token_count() < 2
        && token.chars().count() > 3
        && !is_common_en_technical_word(&converted.to_ascii_lowercase())
        && !strong_autoswitch
        && !word_center_settled
    {
        return None;
    }
    if !layout_candidate_allowed(token, &converted, strong_autoswitch, word_center_settled) {
        return None;
    }
    if !language_allows_layout(token, &converted, projection_supported) {
        return None;
    }
    let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
    let risk = if strong_autoswitch {
        layout_risk(token, &converted, context).min(0.05)
    } else if word_center_settled {
        (layout_risk(token, &converted, context) + 0.06).min(0.24)
    } else {
        layout_risk(token, &converted, context)
    };
    if energy <= risk {
        return None;
    }
    Some(WordCandidate {
        text: format!("{prefix}{converted}"),
        origin: if word_center_settled {
            CandidateOrigin::LayoutThenTypo
        } else {
            CandidateOrigin::Layout
        },
        source: if word_center_settled {
            LAYOUT_THEN_L2_WORD_CENTER
        } else {
            "LayoutWordCell32"
        },
        energy,
        risk,
        support: candidate_support(l1, context),
    })
}

pub(super) fn layout_scan_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    let tokens = tail.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 || tokens.len() > 15 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for idx in (0..tokens.len()).rev() {
        let token = tokens[idx];
        if token.chars().count() < 2 {
            continue;
        }
        if is_common_en_technical_word(&token.to_ascii_lowercase()) {
            continue;
        }
        let prefix = if idx == 0 { "" } else { tokens[idx - 1] };
        if technical_context_blocks_layout(prefix, token) {
            continue;
        }
        let Some((converted, strong_autoswitch, word_center_settled)) =
            layout_converted_token(token, true)
        else {
            continue;
        };
        let projection_supported = strong_autoswitch || word_center_settled;
        if input_has_settled_phase(token) && !strong_autoswitch {
            continue;
        }
        if converted == token
            || !layout_candidate_allowed(token, &converted, strong_autoswitch, word_center_settled)
            || !language_allows_layout(token, &converted, projection_supported)
        {
            continue;
        }
        let mut replaced = tokens
            .iter()
            .map(|item| (*item).to_string())
            .collect::<Vec<_>>();
        replaced[idx] = converted;
        let text = replaced.join(" ");
        let energy = l1_energy(l1, "KeyboardCell32").max(0.35);
        let base_risk = layout_risk(token, &replaced[idx], context);
        let risk = if strong_autoswitch {
            base_risk.min(0.08)
        } else if word_center_settled {
            (base_risk + 0.08).min(0.28)
        } else {
            (base_risk + 0.08).min(0.90)
        };
        if energy <= risk {
            continue;
        }
        candidates.push(WordCandidate {
            text,
            origin: if word_center_settled {
                CandidateOrigin::LayoutThenTypo
            } else {
                CandidateOrigin::Layout
            },
            source: if word_center_settled {
                LAYOUT_THEN_L2_WORD_CENTER
            } else {
                "LayoutWordCell32"
            },
            energy,
            risk,
            support: candidate_support(l1, context),
        });
        if candidates.len() >= MAX_LAYOUT_SCAN_CANDIDATES {
            break;
        }
    }
    candidates
}

fn layout_converted_token(
    token: &str,
    allow_noisy_projection: bool,
) -> Option<(String, bool, bool)> {
    if token.chars().any(is_cyrillic_letter) {
        let raw_converted = convert(token, detect_direction(token));
        if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
            eprintln!(
                "layout-center token={token:?} raw={raw_converted:?} blocked={}",
                cyrillic_layout_word_center_blocked(token)
            );
        }
        if allow_noisy_projection
            && token.chars().all(is_cyrillic_letter)
            && !cyrillic_layout_word_center_blocked(token)
            && raw_converted != token
            && raw_converted.chars().all(|ch| ch.is_ascii_alphabetic())
        {
            if let Some(center) = settle_english_word_center(&raw_converted) {
                let center = apply_word_case(token, &center);
                return Some((center, false, true));
            }
        }
        if allow_noisy_projection {
            if let Some(converted) =
                crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(token)
            {
                return Some((converted, true, false));
            }
        }
        // Cyrillic-to-ASCII requires a proven target; raw projection is not a
        // fallback because it turns valid Russian words into keyboard noise.
        return None;
    }
    // An all-caps alphabetic token is an explicit keyboard-mode signal from
    // L1. The older autoswitch path only recognized shifted punctuation, so a
    // CapsLock projection could be rejected before it reached the field.
    // Preserve the raw projection as a verified strong layout transition;
    // known technical ASCII tokens were already rejected above.
    let ascii_letters = token
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect::<Vec<_>>();
    if ascii_letters.len() >= 4 && ascii_letters.iter().all(|ch| ch.is_ascii_uppercase()) {
        let converted = convert(token, crate::dict::Direction::Us2Ru);
        if converted != token && converted.chars().all(is_cyrillic_letter) {
            return Some((converted, true, false));
        }
    }
    let converted = convert(token, detect_direction(token));
    if converted == token {
        return None;
    }
    // A projected first word has no phrase context yet. Its compact L2 lexical
    // center is sufficient surface evidence: the artifact can reconstruct the
    // target, while random keyboard noise has no terminal center and remains
    // blocked by the caller's first-token guard.
    let converted_lower = converted.to_lowercase();
    if token.chars().all(|ch| ch.is_ascii_alphabetic())
        && token.chars().count() >= 3
        && converted.chars().all(is_cyrillic_letter)
        && surface_motif_memory().contains_surface(&converted_lower)
    {
        return Some((apply_word_case(token, &converted_lower), false, true));
    }
    // Lexical centers are case-normalized. Case is a surface property that is
    // restored only after the same center has admitted the layout projection.
    // Without this, an all-caps wrong-layout word bypasses its known Russian
    // center solely because the keyboard projection is all caps too.
    let converted_center_form = converted_lower;
    // Keyboard projection is only a surface proposal. A target becomes a
    // strong L1/L2 signal when its compact phase readout settles; broad
    // reference morphology alone cannot promote an accidental projection.
    let exact_projection_has_center = token.chars().all(|ch| ch.is_ascii_alphabetic())
        && converted.chars().all(is_cyrillic_letter)
        && crate::hot_field::HotFieldSnapshot::current()
            .layout_projection_has_phase_authority(&converted_center_form);
    Some((converted, exact_projection_has_center, false))
}

pub(super) fn settle_english_word_center(token: &str) -> Option<String> {
    let normalized = token.to_ascii_lowercase();
    if !(4..=18).contains(&normalized.chars().count())
        || !normalized.chars().all(|ch| ch.is_ascii_alphabetic())
    {
        return None;
    }
    let mut candidates = surface_motif_memory()
        .field_surface_candidates(&normalized, 8)
        .into_iter()
        .filter(|candidate| {
            candidate.word.chars().count() >= 4
                && candidate.word.chars().all(|ch| ch.is_ascii_alphabetic())
                && crate::layout_autoswitch::is_known_english_layout_autoswitch_word(
                    &candidate.word,
                )
        })
        .collect::<Vec<_>>();
    if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
        eprintln!("english-center token={normalized:?} candidates={candidates:?}");
    }
    candidates.sort_by(|left, right| {
        left.word
            .cmp(&right.word)
            .then_with(|| right.score.cmp(&left.score))
    });
    candidates.dedup_by(|left, right| left.word == right.word);
    crate::candidate_ranker::choose_best_with_gap(candidates, 64.0, |candidate| {
        let (_, transition) = crate::transition_relation::TransitionRelationAtoms::inferred(
            &normalized,
            &candidate.word,
            "",
        );
        if !transition.verifier_passed() {
            return None;
        }
        let operator_cost = damerau_levenshtein(&normalized, &candidate.word) as f64 * 256.0;
        Some(candidate.score as f64 - operator_cost)
    })
    .map(|(candidate, _)| candidate.word)
}

fn cyrillic_layout_word_center_blocked(token: &str) -> bool {
    let lower = token.to_lowercase();
    is_user_protected_word(&lower)
        || is_ru_live_protected_word(&lower)
        || input_has_settled_phase(&lower)
}

fn language_allows_layout(token: &str, converted: &str, learned_transition: bool) -> bool {
    if learned_transition {
        return true;
    }
    let switched_script = (token.chars().all(|ch| ch.is_ascii_alphabetic())
        && converted.chars().all(is_cyrillic_letter))
        || (token.chars().all(is_cyrillic_letter)
            && converted.chars().all(|ch| ch.is_ascii_alphabetic()));
    !switched_script
        || crate::hot_field::HotFieldSnapshot::current()
            .layout_projection_has_phase_authority(converted)
}

pub(super) fn short_token_candidates(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    let clean = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    if clean.chars().count() != 1 || !clean.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Vec::new();
    }
    if context.token_count() < 2 || technical_context_blocks_layout(prefix, token) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let converted = convert(clean, detect_direction(clean));
    if converted != clean
        && converted.chars().all(is_cyrillic_letter)
        && crate::hot_field::HotFieldSnapshot::current()
            .layout_projection_has_phase_authority(&converted)
    {
        candidates.push(short_token_candidate(ShortTokenCandidateInput {
            prefix,
            token,
            replacement: &converted,
            reason: "keyboard-short-token",
            energy_floor: 0.90,
            risk: short_token_risk(context, token, "keyboard"),
            l1,
            context,
        }));
    }

    if clean.eq_ignore_ascii_case("b") {
        for (replacement, reason) in [
            (visual_b_default_replacement(), "visual-b-default"),
            (visual_b_after_ascii_replacement(), "visual-b-after-ascii"),
        ] {
            if replacement != converted
                && !candidates.iter().any(|item| {
                    item.text
                        .split_whitespace()
                        .last()
                        .is_some_and(|last| last == replacement)
                })
            {
                candidates.push(short_token_candidate(ShortTokenCandidateInput {
                    prefix,
                    token,
                    replacement,
                    reason,
                    energy_floor: 0.76,
                    risk: short_token_risk(context, token, "visual"),
                    l1,
                    context,
                }));
            }
        }
    }
    candidates
}

struct ShortTokenCandidateInput<'a> {
    prefix: &'a str,
    token: &'a str,
    replacement: &'a str,
    reason: &'a str,
    energy_floor: f32,
    risk: f32,
    l1: &'a [WavePacket],
    context: &'a TailContext,
}

fn short_token_candidate(input: ShortTokenCandidateInput<'_>) -> WordCandidate {
    let replacement = if input.token.chars().next().is_some_and(char::is_uppercase) {
        input.replacement.to_uppercase()
    } else {
        input.replacement.to_string()
    };
    WordCandidate {
        text: format!("{}{}", input.prefix, replacement),
        origin: CandidateOrigin::Layout,
        source: "ShortTokenCell32",
        energy: l1_energy(input.l1, "KeyboardCell32").max(input.energy_floor),
        risk: input.risk,
        support: {
            let mut support = candidate_support(input.l1, input.context);
            support.push(input.reason.to_string());
            support
        },
    }
}

fn short_token_risk(context: &TailContext, token: &str, mode: &str) -> f32 {
    let technical_context = context.has_technical_context();
    let ascii_context = context.tokens.iter().any(|item| {
        matches!(item.kind, TokenKind::AsciiWord | TokenKind::TechnicalAscii)
            && !item.text.eq_ignore_ascii_case(token)
    });
    let cyrillic_context = context
        .tokens
        .iter()
        .any(|item| item.kind == TokenKind::CyrillicWord);
    let mut risk: f32 = match mode {
        "visual" => 0.30,
        _ => 0.18,
    };
    if technical_context {
        risk += 0.35;
    }
    if ascii_context && !cyrillic_context {
        risk += 0.28;
    }
    if cyrillic_context {
        risk -= 0.08;
    }
    risk.clamp(0.05, 0.85)
}

fn technical_context_blocks_layout(prefix: &str, token: &str) -> bool {
    if !token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    let Some(previous) = previous_token(prefix) else {
        return false;
    };
    is_common_en_guard_prefix(&previous.to_ascii_lowercase()) && token.chars().count() >= 3
}

fn input_has_settled_phase(token: &str) -> bool {
    crate::hot_field::HotFieldSnapshot::current()
        .input_surface_readout(token)
        .has_phase_authority()
}

pub(super) fn layout_candidate_allowed(
    token: &str,
    converted: &str,
    _strong_autoswitch: bool,
    learned_transition: bool,
) -> bool {
    let switched_script = (token.chars().all(|ch| ch.is_ascii_alphabetic())
        && converted.chars().all(is_cyrillic_letter))
        || (token.chars().all(is_cyrillic_letter)
            && converted.chars().all(|ch| ch.is_ascii_alphabetic()));
    switched_script
        && (learned_transition
            || crate::hot_field::HotFieldSnapshot::current()
                .layout_projection_has_phase_authority(converted))
}

fn previous_token(prefix: &str) -> Option<&str> {
    prefix.split_whitespace().last()
}

fn layout_risk(token: &str, converted: &str, context: &TailContext) -> f32 {
    let short: f32 = if token.chars().count() <= 2 {
        0.35
    } else {
        0.10
    };
    let technical: f32 = if is_common_en_technical_word(&token.to_ascii_lowercase())
        || is_common_en_technical_word(&converted.to_ascii_lowercase())
    {
        0.20
    } else {
        0.0
    };
    let context_bonus = context.mixed_language_score();
    (short + technical - context_bonus).clamp(0.0, 0.85)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyrillic_layout_projection_requires_a_proven_target() {
        assert!(layout_converted_token("давай", true).is_none());
        assert_eq!(
            layout_converted_token("вудуеу", true).map(|candidate| candidate.0),
            Some("delete".to_string())
        );
        assert_eq!(
            layout_converted_token("ytn", true).map(|candidate| candidate.0),
            Some("нет".to_string())
        );
    }

    #[test]
    fn ascii_layout_projection_recovers_reference_backed_forms() {
        assert_eq!(
            layout_converted_token("ltkfq", true).map(|candidate| candidate.0),
            Some("делай".to_string())
        );
        assert_eq!(
            layout_converted_token("Ghjljkbv", true).map(|candidate| candidate.0),
            Some("Продолим".to_string())
        );
    }

    #[test]
    fn short_layout_projection_uses_target_phase_authority() {
        assert!(!layout_candidate_allowed("rt", "ке", true, false));
        assert!(layout_candidate_allowed("rt", "ке", true, true));
        assert!(layout_candidate_allowed("yt", "не", true, false));
        assert!(layout_candidate_allowed("ytn", "нет", true, false));
        assert!(crate::nanda_wave::l2::hot_layout_candidate("rt").is_none());
    }

    #[test]
    fn all_caps_layout_projection_uses_the_same_lexical_center() {
        assert_eq!(
            layout_converted_token("YTGTHTDTHYEKJCM", true).map(|candidate| candidate.0),
            Some("НЕПЕРЕВЕРНУЛОСЬ".to_string())
        );
    }

    #[test]
    fn all_caps_glued_phrase_stays_out_of_direct_lexical_authority() {
        let token = "YTGTHTDTHYEKJCM";
        let context = TailContext::from_text(token);
        assert!(layout_candidate("", token, &context, &[]).is_none());
    }
}
