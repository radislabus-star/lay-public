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

pub(super) fn boundary_split_has_structural_evidence(token: &str) -> bool {
    if !(4..=18).contains(&token.chars().count()) || !token.chars().all(is_cyrillic_letter) {
        return false;
    }
    let normalized = token.to_lowercase();
    let chars = normalized.chars().collect::<Vec<_>>();
    (1..chars.len()).any(|split| {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        let leading_function = left.chars().count() <= 3
            && (is_ru_one_letter_function_word(&left)
                || crate::lexicon::is_ru_short_pronoun(&left)
                || is_common_ru_word(&left))
            && stable_boundary_right_center(&right);
        let trailing_function =
            trailing_short_function_center(&right) && stable_boundary_left_center(&left);
        leading_function || trailing_function || independent_content_boundary_centers(&left, &right)
    })
}

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
    let whole_surface_known = is_known_russian_word_or_form(&normalized);
    if is_common_ru_word(&normalized)
        || is_ru_live_protected_word(&normalized)
        || surface_motif_strict_known_surface(&normalized)
    {
        return Vec::new();
    }
    // A decoded motif may be a broad form-only surface rather than a stable
    // lexical state. Let the compact two-center boundary readout compete
    // before that broad surface suppresses every split proposal.
    let mut fuzzy_typo_candidates: Option<Vec<String>> = None;
    let light_started = std::time::Instant::now();
    let light_replacement =
        light_boundary_replacement_with_fuzzy(&normalized, &mut fuzzy_typo_candidates);
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_boundary_light_trace elapsed_us={} found={}",
            light_started.elapsed().as_micros(),
            light_replacement.is_some(),
        );
    }
    if let Some(replacement) = light_replacement {
        if boundary_replacement_beats_known_whole(
            &normalized,
            &replacement,
            whole_surface_known,
            context,
        ) {
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
    }
    if normalized.chars().count() < 6 {
        return Vec::new();
    }
    if surface_motif_known_surface(&normalized) {
        return Vec::new();
    }
    let phrase_started = std::time::Instant::now();
    let phrase_replacement = crate::phrase_reader::correct_glued_russian_phrase(&normalized);
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_boundary_phrase_trace elapsed_us={} found={}",
            phrase_started.elapsed().as_micros(),
            phrase_replacement.is_some(),
        );
    }
    if let Some(replacement) = phrase_replacement {
        if replacement != normalized
            && boundary_replacement_beats_known_whole(
                &normalized,
                &replacement,
                whole_surface_known,
                context,
            )
        {
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
    let mut candidates = Vec::new();
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        let trailing_short_function =
            !whole_surface_known && trailing_short_function_center(&right);
        if trailing_short_function
            && fuzzy_typo_candidates
                .get_or_insert_with(|| boundary_fuzzy_candidates(&normalized))
                .iter()
                .any(|candidate| damerau_levenshtein(&normalized, candidate) == 1)
        {
            continue;
        }
        if left.chars().count() > 2 && right.chars().count() < 3 && !trailing_short_function {
            continue;
        }
        let short_function_boundary =
            left.chars().count() == 1 && is_ru_one_letter_function_word(&left);
        if short_function_boundary && fuzzy_typo_candidates.is_none() {
            fuzzy_typo_candidates = Some(boundary_fuzzy_candidates(&normalized));
        }
        if short_function_boundary
            && fuzzy_typo_candidates
                .as_ref()
                .is_some_and(|candidates| !candidates.is_empty())
            && !strong_boundary_right_anchor(&right)
        {
            continue;
        }
        // Boundary field is bidirectional: an attached short function word or
        // pronoun can be on either side of a stable lexical center.
        // Both halves still need compact lexical evidence; this is not a
        // phrase-specific rewrite table.
        let known_left = short_function_boundary
            || surface_motif_known_surface(&left)
            || (trailing_short_function && stable_boundary_left_center(&left));
        let known_right = trailing_short_function || surface_motif_known_surface(&right);
        let two_content_centers =
            !whole_surface_known && independent_content_boundary_centers(&left, &right);
        if (!known_left || !known_right) && !two_content_centers {
            continue;
        }
        // Two ordinary words are not enough evidence for an automatic split:
        // they create many plausible but false segmentations. A lexical
        // boundary needs an explicit short functional anchor; richer
        // multiword repairs stay in the typed phrase/boundary operator route.
        if !short_function_boundary && !trailing_short_function {
            if !two_content_centers {
                continue;
            }
            candidates.push(WordCandidate {
                text: format!(
                    "{prefix}{}",
                    apply_word_case(token, &format!("{left} {right}"))
                ),
                origin: CandidateOrigin::Boundary,
                source: "BoundaryCell32",
                energy: l1_energy(l1, "BoundaryCell32").max(0.96),
                risk: 0.08,
                support: vec![
                    "two-content-center-boundary".to_string(),
                    format!("left={left:?} right={right:?}"),
                ],
            });
            if candidates.len() >= 3 {
                break;
            }
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

fn boundary_replacement_beats_known_whole(
    word: &str,
    replacement: &str,
    whole_surface_known: bool,
    context: &TailContext,
) -> bool {
    if !whole_surface_known {
        return true;
    }

    // The broad morphology corpus can contain a glued surface as a form-only
    // center. A short pronoun followed by an independently stable lexical
    // center is stronger boundary evidence than that broad whole-word hit.
    // Common/protected whole words have already returned before this point.
    if trusted_leading_pronoun_boundary(replacement) {
        return true;
    }

    contextual_boundary_replacement_for_word(
        word,
        context.previous().map(|token| token.text.as_str()),
    )
    .is_some_and(|contextual| contextual == replacement)
}

fn trusted_leading_pronoun_boundary(replacement: &str) -> bool {
    let Some((left, right)) = replacement.split_once(' ') else {
        return false;
    };
    !left.contains(char::is_whitespace)
        && !right.contains(char::is_whitespace)
        && left.chars().count() <= 3
        && right.chars().count() >= 4
        && crate::lexicon::is_ru_short_pronoun(left)
        && stable_boundary_right_center(right)
}

fn stable_boundary_right_center(word: &str) -> bool {
    is_common_ru_word(word)
        || surface_motif_strict_known_surface(word)
        || is_known_russian_word_or_form(word)
}

fn stable_boundary_left_center(word: &str) -> bool {
    word.chars().count() >= 4 && stable_boundary_right_center(word)
}

fn trailing_short_function_center(word: &str) -> bool {
    (1..=3).contains(&word.chars().count())
        && (crate::phrase_lexicon::is_short_russian_function_word(word)
            || crate::lexicon::is_ru_short_pronoun(word))
}

fn independent_content_boundary_centers(left: &str, right: &str) -> bool {
    if left.chars().count() < 4 || right.chars().count() < 4 {
        return false;
    }
    let left_surface_center = surface_motif_known_surface(left);
    let right_surface_center = surface_motif_known_surface(right);
    let left_known = left_surface_center || is_known_russian_word_or_form(left);
    let right_known = right_surface_center || is_known_russian_word_or_form(right);

    left_known && right_known && (left_surface_center || right_surface_center)
}

fn light_boundary_replacement(word: &str) -> Option<String> {
    light_boundary_replacement_with_fuzzy(word, &mut None)
}

fn light_boundary_replacement_with_fuzzy(
    word: &str,
    fuzzy_typo_candidates: &mut Option<Vec<String>>,
) -> Option<String> {
    let chars = word.chars().collect::<Vec<_>>();
    let mut best = None::<(usize, String)>;
    for split in 1..chars.len() {
        let left = chars[..split].iter().collect::<String>();
        let right = chars[split..].iter().collect::<String>();
        let trailing_short_function =
            trailing_short_function_center(&right) && stable_boundary_left_center(&left);
        if trailing_short_function
            && fuzzy_typo_candidates
                .get_or_insert_with(|| boundary_fuzzy_candidates(word))
                .iter()
                .any(|candidate| damerau_levenshtein(word, candidate) == 1)
        {
            continue;
        }
        let leading_short_center = left.chars().count() <= 3 && right.chars().count() >= 3;
        if !leading_short_center && !trailing_short_function {
            continue;
        }
        if leading_short_center
            && left.chars().count() > 1
            && crate::lexicon::is_ru_short_preposition(&left)
        {
            continue;
        }
        let known_left_function = is_ru_one_letter_function_word(&left);
        let known_left_pronoun = crate::lexicon::is_ru_single_letter_pronoun(&left)
            || crate::lexicon::is_ru_short_pronoun(&left);
        let known_left_common = is_common_ru_word(&left);
        let known_left = known_left_function
            || known_left_pronoun
            || known_left_common
            || trailing_short_function;
        let known_right = surface_motif_known_surface(&right)
            || (known_left_pronoun
                && right.chars().count() >= 4
                && stable_boundary_right_center(&right))
            || trailing_short_function;
        if known_left_function && !known_left_pronoun {
            let fuzzy =
                fuzzy_typo_candidates.get_or_insert_with(|| boundary_fuzzy_candidates(word));
            if !fuzzy.is_empty() && !strong_boundary_right_anchor(&right) {
                continue;
            }
        }
        if !trailing_short_function
            && word.chars().count() < 6
            && (!(known_left_function || known_left_pronoun) || !is_common_ru_word(&right))
        {
            continue;
        }
        if known_left && known_right {
            let score = boundary_split_score(
                left.chars().count(),
                right.chars().count(),
                known_left_function,
                known_left_pronoun || known_left_common || trailing_short_function,
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

fn boundary_fuzzy_candidates(word: &str) -> Vec<String> {
    let started = std::time::Instant::now();
    let candidates = crate::ru_typo::fuzzy_known_word_candidates(word);
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_boundary_fuzzy_trace elapsed_us={} candidates={}",
            started.elapsed().as_micros(),
            candidates.len(),
        );
    }
    candidates
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
    let lower = word.to_lowercase();
    if is_common_ru_word(&lower)
        || is_ru_live_protected_word(&lower)
        || surface_motif_strict_known_surface(&lower)
    {
        return None;
    }
    let whole_surface_known = is_known_russian_word_or_form(&lower);
    if let Some(replacement) = light_boundary_replacement(&lower) {
        if !whole_surface_known || trusted_leading_pronoun_boundary(&replacement) {
            return Some(apply_word_case(word, &replacement));
        }
    }
    if lower.chars().count() < 6 || whole_surface_known {
        return None;
    }
    if let Some(replacement) = crate::phrase_reader::correct_glued_russian_phrase(word) {
        return Some(replacement);
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
