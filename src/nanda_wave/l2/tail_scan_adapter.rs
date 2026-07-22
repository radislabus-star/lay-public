use super::super::context::TailContext;
use super::super::signal::{WavePacket, WordCandidate};
use super::surface::{
    surface_motif_known_surface, surface_motif_strict_known_surface, surface_motif_typo_risk,
};
use super::{candidate_support, l1_energy, L2_SURFACE_MOTIF_CELL};
use crate::candidate_contract::CandidateOrigin;
use crate::keyboard::is_cyrillic_letter;
use crate::lexicon::{
    is_common_ru_word, is_ru_live_protected_word, is_ru_one_letter_function_word,
};
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::text_metrics::damerau_levenshtein;
use crate::word_reader::{split_word_punctuation, split_ws_segments};

pub(super) fn boundary_split_candidates(
    prefix: &str,
    token: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() || !token.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    if is_common_ru_word(&normalized)
        || is_ru_live_protected_word(&normalized)
        || surface_motif_strict_known_surface(&normalized)
    {
        return Vec::new();
    }
    // A decoded motif may be a broad form-only surface rather than a stable
    // lexical state. Let the compact two-center boundary readout compete
    // before that broad surface suppresses every split proposal.
    if let Some(replacement) = light_boundary_replacement(&normalized) {
        return vec![WordCandidate {
            text: format!("{prefix}{}", apply_word_case(token, &replacement)),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.99),
            risk: 0.04,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("light-boundary-split".to_string());
                support.push(format!("word={normalized:?} replacement={replacement:?}"));
                support
            },
        }];
    }
    if normalized.chars().count() < 6 {
        return Vec::new();
    }
    if surface_motif_known_surface(&normalized) {
        return Vec::new();
    }
    if let Some(replacement) = crate::phrase_reader::correct_glued_russian_phrase(&normalized) {
        if replacement != normalized {
            return vec![WordCandidate {
                text: format!("{prefix}{}", apply_word_case(token, &replacement)),
                origin: CandidateOrigin::Boundary,
                source: "BoundaryCell32",
                energy: l1_energy(l1, "BoundaryCell32").max(0.99),
                risk: 0.04,
                support: {
                    let mut support = candidate_support(l1, context);
                    support.push("direct-glued-phrase-boundary".to_string());
                    support.push(format!("word={normalized:?} replacement={replacement:?}"));
                    support
                },
            }];
        }
    }
    let chars = normalized.chars().collect::<Vec<_>>();
    let mut fuzzy_typo_candidates: Option<Vec<String>> = None;
    let mut candidates = Vec::new();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        let trailing_short_pronoun =
            right.chars().count() == 1 && crate::lexicon::is_ru_short_pronoun(&right);
        if left.chars().count() > 2 && right.chars().count() < 3 && !trailing_short_pronoun {
            continue;
        }
        let short_function_boundary =
            left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
        if short_function_boundary && fuzzy_typo_candidates.is_none() {
            fuzzy_typo_candidates = Some(crate::ru_typo::fuzzy_known_word_candidates(&normalized));
        }
        if short_function_boundary
            && fuzzy_typo_candidates
                .as_ref()
                .is_some_and(|candidates| !candidates.is_empty())
            && !strong_boundary_right_anchor(&right)
        {
            continue;
        }
        // Boundary field is bidirectional: an attached one-letter function
        // word or pronoun can be on either side of a stable lexical center.
        // Both halves still need compact lexical evidence; this is not a
        // phrase-specific rewrite table.
        let known_left = short_function_boundary || surface_motif_known_surface(&left);
        let known_right = trailing_short_pronoun || surface_motif_known_surface(&right);
        if !known_left || !known_right {
            continue;
        }
        // Two ordinary words are not enough evidence for an automatic split:
        // they create many plausible but false segmentations. A lexical
        // boundary needs an explicit short functional anchor; richer
        // multiword repairs stay in the typed phrase/boundary operator route.
        if !short_function_boundary && !trailing_short_pronoun {
            continue;
        }
        let (energy, risk, reason) = (
            l1_energy(l1, "BoundaryCell32").max(0.99),
            0.04,
            "hidden-short-function-boundary",
        );
        candidates.push(WordCandidate {
            text: format!("{prefix}{left} {right}"),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy,
            risk,
            support: vec![reason.to_string(), format!("left={left:?} right={right:?}")],
        });
        if candidates.len() >= 3 {
            break;
        }
    }
    candidates
}

fn light_boundary_replacement(word: &str) -> Option<String> {
    let chars = word.chars().collect::<Vec<_>>();
    let mut fuzzy_typo_candidates: Option<Vec<String>> = None;
    let mut best = None::<(usize, String)>;
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if left.chars().count() > 3 || right.chars().count() < 3 {
            continue;
        }
        if left.chars().count() > 1 && crate::lexicon::is_ru_short_preposition(&left) {
            continue;
        }
        let known_left_function = is_ru_one_letter_function_word(&left);
        let known_left_pronoun = crate::lexicon::is_ru_single_letter_pronoun(&left)
            || crate::lexicon::is_ru_short_pronoun(&left);
        let known_left_common = is_common_ru_word(&left);
        let known_left = known_left_function || known_left_pronoun || known_left_common;
        let known_right = surface_motif_known_surface(&right);
        if known_left_function && !known_left_pronoun {
            let fuzzy = fuzzy_typo_candidates
                .get_or_insert_with(|| crate::ru_typo::fuzzy_known_word_candidates(word));
            if !fuzzy.is_empty() && !strong_boundary_right_anchor(&right) {
                continue;
            }
        }
        if word.chars().count() < 6
            && (!(known_left_function || known_left_pronoun) || !is_common_ru_word(&right))
        {
            continue;
        }
        if known_left && known_right {
            let score = boundary_split_score(
                left.chars().count(),
                right.chars().count(),
                known_left_function,
                known_left_pronoun || known_left_common,
                is_common_ru_word(&right),
            );
            let replacement = format!("{left} {right}");
            let replace_best = match best.as_ref() {
                Some((best_score, _)) => score > *best_score,
                None => true,
            };
            if replace_best {
                best = Some((score, replacement));
            }
        }
    }
    best.map(|(_, replacement)| replacement)
}

fn boundary_split_score(
    left_len: usize,
    right_len: usize,
    left_function: bool,
    left_common: bool,
    right_common: bool,
) -> usize {
    let mut score = right_len.min(12);
    if left_common {
        score += 20;
    }
    if right_common {
        score += 10;
    }
    if left_function {
        score += 4;
    }
    score + left_len.min(4) * 3
}

pub(super) fn boundary_scan_candidates(
    tail: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let segments = split_ws_segments(tail);
    if segments.len() < 3 || context.token_count() > 15 {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for (idx, (segment, is_ws)) in segments.iter().enumerate().rev() {
        if *is_ws {
            continue;
        }
        let (leading, word, trailing) = split_word_punctuation(segment);
        if word.is_empty() || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        let previous = previous_word_segment(&segments, idx);
        let Some(replacement) = contextual_boundary_replacement_for_word(word, previous)
            .or_else(|| boundary_replacement_for_word(word))
        else {
            continue;
        };
        if replacement == word {
            continue;
        }
        let text = replace_segment_word(&segments, idx, leading, &replacement, trailing);
        candidates.push(WordCandidate {
            text,
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: l1_energy(l1, "BoundaryCell32").max(0.82),
            risk: 0.10,
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-boundary-scan".to_string());
                support.push(format!("word={word:?} replacement={replacement:?}"));
                support
            },
        });
        if candidates.len() >= 4 {
            return candidates;
        }
    }

    for window in word_segment_windows(&segments).into_iter().rev() {
        let pair_text = format!(
            "{}{}{}",
            segments[window.left_idx].0, segments[window.ws_idx].0, segments[window.right_idx].0
        );
        let Some((replacement, repair_kind, energy, risk)) =
            crate::phrase_reader::propose_moved_prefix_letter_pair(&pair_text)
                .map(|replacement| {
                    (
                        replacement,
                        "tail-moved-prefix-pair-scan",
                        l1_energy(l1, "BoundaryShiftCell32").max(0.92),
                        0.06,
                    )
                })
                .or_else(|| {
                    crate::phrase_reader::correct_split_word_pair(&pair_text).map(|replacement| {
                        (
                            replacement,
                            "tail-split-pair-scan",
                            l1_energy(l1, "BoundaryCell32").max(0.80),
                            0.12,
                        )
                    })
                })
        else {
            continue;
        };
        if replacement == pair_text {
            continue;
        }
        candidates.push(WordCandidate {
            text: replace_segment_window(
                &segments,
                window.left_idx,
                window.right_idx,
                &replacement,
            ),
            origin: CandidateOrigin::Boundary,
            source: if repair_kind == "tail-moved-prefix-pair-scan" {
                "BoundaryShiftCell32"
            } else {
                "BoundaryCell32"
            },
            energy,
            risk,
            support: {
                let mut support = candidate_support(l1, context);
                support.push(repair_kind.to_string());
                support.push(format!("pair={pair_text:?} replacement={replacement:?}"));
                support
            },
        });
        if candidates.len() >= 4 {
            break;
        }
    }

    candidates
}

fn boundary_replacement_for_word(word: &str) -> Option<String> {
    crate::phrase_reader::correct_glued_russian_phrase(word).or_else(|| {
        let lower = word.to_lowercase();
        if lower.chars().count() < 6
            || is_common_ru_word(&lower)
            || is_known_russian_word_or_form(&lower)
        {
            return None;
        }
        let chars = lower.chars().collect::<Vec<_>>();
        for split in 1..chars.len() {
            let left = chars[..split].iter().collect::<String>();
            let right = chars[split..].iter().collect::<String>();
            if left.chars().count() > 2 && right.chars().count() < 3 {
                continue;
            }
            let known_left = left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
            let known_right = is_common_ru_word(&right) || is_known_russian_word_or_form(&right);
            if known_left && known_right {
                let replacement = format!("{left} {right}");
                return Some(apply_word_case(word, &replacement));
            }
        }
        None
    })
}

fn contextual_boundary_replacement_for_word(word: &str, previous: Option<&str>) -> Option<String> {
    let previous = previous?.to_lowercase();
    if !crate::phrase_lexicon::is_short_russian_function_word(&previous) {
        return None;
    }

    let lower = word.to_lowercase();
    let chars = lower.chars().collect::<Vec<_>>();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        if !crate::lexicon::is_ru_short_pronoun(&left) {
            continue;
        }
        if !(right == "есть" || is_common_ru_word(&right) || is_known_russian_word_or_form(&right))
        {
            continue;
        }
        let replacement = format!("{left} {right}");
        return Some(apply_word_case(word, &replacement));
    }
    None
}

pub(super) fn surface_motif_scan_candidates(
    tail: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Vec<WordCandidate> {
    if context.has_technical_context() || context.token_count() > 15 {
        return Vec::new();
    }
    let segments = split_ws_segments(tail);
    if segments.len() < 3 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for (idx, (segment, is_ws)) in segments.iter().enumerate().rev() {
        if *is_ws {
            continue;
        }
        let (leading, word, trailing) = split_word_punctuation(segment);
        if word.is_empty() || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        let Some(replacement) = surface_replacement_for_word(word) else {
            continue;
        };
        if replacement == word {
            continue;
        }
        let lower = word.to_lowercase();
        let replacement_lower = replacement.to_lowercase();
        let distance = damerau_levenshtein(&lower, &replacement_lower);
        if distance == 0 || distance > 3 {
            continue;
        }
        candidates.push(WordCandidate {
            text: replace_segment_word(&segments, idx, leading, &replacement, trailing),
            origin: CandidateOrigin::L2Surface,
            source: L2_SURFACE_MOTIF_CELL,
            energy: l1_energy(l1, "ScriptCell32").max(0.78),
            risk: surface_motif_typo_risk(context, distance),
            support: {
                let mut support = candidate_support(l1, context);
                support.push("tail-surface-scan".to_string());
                support.push(format!(
                    "word={word:?} replacement={replacement:?} distance={distance}"
                ));
                support
            },
        });
        if candidates.len() >= 4 {
            break;
        }
    }
    candidates
}

fn surface_replacement_for_word(word: &str) -> Option<String> {
    crate::ru_typo::correct_repeated_letter(word)
        .or_else(|| crate::ru_typo::correct_adjacent_transposition(word))
        .or_else(|| crate::ru_typo::correct_missing_letter(word))
}

struct SegmentWindow {
    left_idx: usize,
    ws_idx: usize,
    right_idx: usize,
}

fn word_segment_windows(segments: &[(&str, bool)]) -> Vec<SegmentWindow> {
    segments
        .windows(3)
        .enumerate()
        .filter_map(|(idx, window)| {
            let [left, ws, right] = window else {
                return None;
            };
            (!left.1 && ws.1 && !right.1).then_some(SegmentWindow {
                left_idx: idx,
                ws_idx: idx + 1,
                right_idx: idx + 2,
            })
        })
        .collect()
}

fn replace_segment_word(
    segments: &[(&str, bool)],
    target_idx: usize,
    leading: &str,
    replacement: &str,
    trailing: &str,
) -> String {
    let mut out = String::new();
    for (idx, (segment, _)) in segments.iter().enumerate() {
        if idx == target_idx {
            out.push_str(leading);
            out.push_str(replacement);
            out.push_str(trailing);
        } else {
            out.push_str(segment);
        }
    }
    out
}

fn replace_segment_window(
    segments: &[(&str, bool)],
    left_idx: usize,
    right_idx: usize,
    replacement: &str,
) -> String {
    let mut out = String::new();
    let mut idx = 0;
    while idx < segments.len() {
        if idx == left_idx {
            out.push_str(replacement);
            idx = right_idx + 1;
        } else {
            out.push_str(segments[idx].0);
            idx += 1;
        }
    }
    out
}

fn previous_word_segment<'a>(
    segments: &'a [(&'a str, bool)],
    before_idx: usize,
) -> Option<&'a str> {
    let segment = crate::word_reader::previous_non_whitespace_segment(segments, before_idx)?;
    let (_, word, _) = split_word_punctuation(segment);
    (!word.is_empty()).then_some(word)
}

fn strong_boundary_right_anchor(lower: &str) -> bool {
    lower.chars().count() >= 5
        && (lower.ends_with("ах") || lower.ends_with("ях"))
        && (is_common_ru_word(lower) || is_known_russian_word_or_form(lower))
}
