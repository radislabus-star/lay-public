use super::*;
use crate::candidate_contract::CandidateOrigin;
use crate::nanda_wave::{llmwave, usage_prior};

pub(super) fn settle_russian_surface(surface: &str) -> Option<String> {
    let (_, word, _) = split_word_punctuation(surface);
    if !(4..=18).contains(&word.chars().count()) || !word.chars().all(is_cyrillic_letter) {
        return None;
    }
    let normalized = word.to_lowercase();

    let context = TailContext::from_text(surface);
    let l1 = crate::nanda_wave::l1::run_l1(surface);
    form_attractor_word_candidates("", surface, &context, &l1, &WaveOptions::default())
        .into_iter()
        .find(|candidate| {
            candidate.energy > candidate.risk
                && projected_layout_settle_shape_allowed(&normalized, &candidate.text)
        })
        .map(|candidate| candidate.text)
}

fn projected_layout_settle_shape_allowed(input: &str, candidate_text: &str) -> bool {
    let (_, candidate_word, _) = split_word_punctuation(candidate_text);
    let input_len = input.chars().count();
    let candidate_len = candidate_word.chars().count();
    candidate_len >= input_len || input_len.saturating_sub(candidate_len) <= 1
}

pub(super) fn surface_motif_word_candidates(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let (leading, word, trailing) = split_word_punctuation(token);
    let normalized = word.to_lowercase();
    let len = normalized.chars().count();
    if !(2..=18).contains(&len) {
        return Vec::new();
    }
    if normalized.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return latin_surface_word_candidates(
            prefix,
            leading,
            word,
            trailing,
            &normalized,
            l1,
            context,
            options,
        );
    }
    if !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }

    let memory = surface_motif_memory();
    let stable_input = surface_motif_stable_existing_word(&normalized);
    let mut surface_candidates = memory.surface_candidates(&normalized, 24);
    if options.is_enabled(L2_SURFACE_MOTIF_CELL) {
        surface_candidates.extend(memory.adjacent_transposition_candidates(&normalized));
        surface_candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.rank.cmp(&right.rank))
                .then_with(|| left.word.cmp(&right.word))
        });
        surface_candidates.dedup_by(|left, right| left.word == right.word);
        preserve_typed_damage_frontier(&normalized, &mut surface_candidates, 32);
    }
    if options.is_enabled(L2_SURFACE_COMPLETION_CELL) && !stable_input {
        surface_candidates.extend(memory.completion_candidates(&normalized, 24, 144));
        surface_candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.rank.cmp(&right.rank))
                .then_with(|| left.word.cmp(&right.word))
        });
        surface_candidates.dedup_by(|left, right| left.word == right.word);
        preserve_typed_damage_frontier(&normalized, &mut surface_candidates, 32);
    }
    let phase_deltas = l2_birth_phase_deltas(&normalized, &surface_candidates, options);
    if std::env::var_os("LAY_NANDA_L2_TIMING").is_some() {
        eprintln!(
            "lay_nanda_l2_surface input={normalized:?} stable={} candidates={:?}",
            stable_input,
            surface_candidates
                .iter()
                .take(12)
                .map(|candidate| {
                    (
                        &candidate.word,
                        candidate.score,
                        candidate.rank,
                        candidate.prefix_match,
                        candidate.reconstructed,
                    )
                })
                .collect::<Vec<_>>()
        );
    }
    let mut out = Vec::new();
    if options.is_enabled(L2_SURFACE_MOTIF_CELL) && surface_candidates.is_empty() {
        if let Some(candidate) = repeated_letter_surface_candidate(
            prefix,
            leading,
            word,
            trailing,
            &normalized,
            l1,
            context,
        ) {
            out.push(candidate);
        }
    }
    let empty_fuzzy_authority = Vec::new();
    for candidate in &surface_candidates {
        let candidate_len = candidate.word.chars().count();
        let distance = damerau_levenshtein(&normalized, &candidate.word);
        let base_score = surface_attractor_score(candidate.score, &candidate.word).saturating_add(
            typed_damage_score_bonus(&normalized, &candidate.word, distance),
        );
        let phase_delta = phase_deltas
            .get(&candidate.word)
            .copied()
            .unwrap_or_default();
        // Local authority remains a property of the L2 center and its surface
        // geometry. The learned phase field only redistributes energy between
        // already admissible centers in this bounded lattice.
        let ranked_score = add_phase_birth_delta(base_score, phase_delta);
        let is_completion = candidate.word.starts_with(&normalized) && candidate_len > len;
        let stable_input_allows_typed_damage =
            stable_input_allows_typed_damage_candidate(&normalized, &candidate.word, distance);

        if options.is_enabled(L2_SURFACE_MOTIF_CELL)
            && len >= 4
            && !is_common_ru_word(&normalized)
            && (!surface_motif_stable_existing_word(&normalized)
                || stable_input_allows_typed_damage)
            && surface_motif_typo_has_authority(
                &normalized,
                &candidate.word,
                base_score,
                &surface_candidates,
                &empty_fuzzy_authority,
            )
            && (!fuzzy_surface_candidate_blocked(word, &normalized, &candidate.word)
                || repeated_all_caps_surface_allowed(word, &normalized, &candidate.word))
            && surface_motif_typo_allowed(&normalized, &candidate.word, len, distance, base_score)
        {
            let mut word_candidate = surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &candidate.word,
                source: L2_SURFACE_MOTIF_CELL,
                score: ranked_score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: typed_damage_risk(context, &normalized, &candidate.word, distance),
                l1,
                context,
            });
            annotate_typed_damage_support(&mut word_candidate, &normalized, &candidate.word);
            record_birth_phase(&mut word_candidate, phase_delta);
            out.push(word_candidate);
            if out.len() >= 8 {
                break;
            }
            continue;
        }

        if !stable_input
            && is_completion
            && options.is_enabled(L2_SURFACE_COMPLETION_CELL)
            && len >= 2
            && candidate_len.saturating_sub(len) <= 10
        {
            out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &candidate.word,
                source: L2_SURFACE_COMPLETION_CELL,
                score: ranked_score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: 0.06,
                l1,
                context,
            }));
        }
    }
    if options.is_enabled(L2_SURFACE_MOTIF_CELL)
        && out.is_empty()
        && len >= 4
        && !surface_motif_stable_existing_word(&normalized)
    {
        let mut fuzzy_authority = crate::ru_typo::fuzzy_known_word_candidates(&normalized);
        fuzzy_authority.sort_by(|left, right| {
            let left_distance = damerau_levenshtein(&normalized, left);
            let right_distance = damerau_levenshtein(&normalized, right);
            left_distance
                .cmp(&right_distance)
                .then_with(|| left.chars().count().cmp(&right.chars().count()))
                .then_with(|| left.cmp(right))
        });
        for replacement_lower in fuzzy_authority.iter().take(4) {
            let distance = damerau_levenshtein(&normalized, replacement_lower);
            let score = 940u32.saturating_sub(distance.min(4) as u32 * 40);
            if surface_motif_typo_has_authority(
                &normalized,
                replacement_lower,
                score,
                &surface_candidates,
                &fuzzy_authority,
            ) && !fuzzy_surface_candidate_blocked(word, &normalized, replacement_lower)
                && surface_motif_typo_allowed(&normalized, replacement_lower, len, distance, score)
            {
                out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                    prefix,
                    leading,
                    word,
                    trailing,
                    replacement_lower,
                    source: L2_SURFACE_MOTIF_CELL,
                    score,
                    l1_overlap: 0,
                    l2_overlap: 0,
                    motif_overlap: 0,
                    prefix_match: false,
                    distance,
                    risk: typed_damage_risk(context, &normalized, replacement_lower, distance),
                    l1,
                    context,
                }));
                break;
            }
        }
    }
    if options.is_enabled(L2_SURFACE_MOTIF_CELL) {
        for replacement_lower in orthographic_sign_frontier(&normalized) {
            if out.iter().any(|candidate| {
                candidate
                    .text
                    .split_whitespace()
                    .last()
                    .is_some_and(|word| word.eq_ignore_ascii_case(&replacement_lower))
            }) {
                continue;
            }
            let distance = damerau_levenshtein(&normalized, &replacement_lower);
            let mut candidate = surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &replacement_lower,
                source: L2_SURFACE_MOTIF_CELL,
                score: 1_520_u32.saturating_add(typed_damage_score_bonus(
                    &normalized,
                    &replacement_lower,
                    distance,
                )),
                l1_overlap: len.saturating_sub(1),
                l2_overlap: len.saturating_sub(2),
                motif_overlap: len.saturating_sub(2),
                prefix_match: false,
                distance,
                risk: typed_damage_risk(context, &normalized, &replacement_lower, distance),
                l1,
                context,
            });
            candidate
                .support
                .push("l2-operator:orthographic-sign-repair".to_string());
            out.push(candidate);
        }
    }
    out
}

fn preserve_typed_damage_frontier(
    input: &str,
    candidates: &mut Vec<LexicalPhaseCandidate>,
    limit: usize,
) {
    let reserved = candidates
        .iter()
        .filter(|candidate| {
            let distance = damerau_levenshtein(input, &candidate.word);
            sparse_internal_multi_omission(input, &candidate.word)
                || single_internal_missing_letter(input, &candidate.word)
                || internal_extra_fragment(input, &candidate.word)
                || repeated_letter_collapse(input, &candidate.word)
                || is_single_adjacent_transposition(input, &candidate.word)
                || single_letter_substitution(input, &candidate.word)
                || orthographic_sign_repair(input, &candidate.word)
                || distance == 1
        })
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    candidates.truncate(limit);
    for candidate in reserved {
        if candidates.iter().any(|item| item.word == candidate.word) {
            continue;
        }
        if candidates.len() == limit {
            candidates.pop();
        }
        candidates.push(candidate);
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
}

#[allow(clippy::too_many_arguments)]
fn latin_surface_word_candidates(
    prefix: &str,
    leading: &str,
    word: &str,
    trailing: &str,
    normalized: &str,
    l1: &[WavePacket],
    context: &TailContext,
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    let len = normalized.chars().count();
    let memory = surface_motif_memory();
    let stable_input = memory.contains_decoded_surface(normalized);
    let mut surface_candidates = memory.surface_candidates(normalized, 24);
    if options.is_enabled(L2_SURFACE_COMPLETION_CELL) && !stable_input {
        surface_candidates.extend(memory.completion_candidates(normalized, 24, 144));
    }
    surface_candidates
        .retain(|candidate| candidate.word.chars().all(|ch| ch.is_ascii_alphabetic()));
    surface_candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
    surface_candidates.dedup_by(|left, right| left.word == right.word);

    let mut out = Vec::new();
    for candidate in surface_candidates {
        let candidate_len = candidate.word.chars().count();
        let distance = damerau_levenshtein(normalized, &candidate.word);
        let is_completion = candidate.word.starts_with(normalized) && candidate_len > len;
        if !stable_input
            && options.is_enabled(L2_SURFACE_MOTIF_CELL)
            && len >= 4
            && !is_completion
            && surface_motif_typo_allowed(
                normalized,
                &candidate.word,
                len,
                distance,
                candidate.score,
            )
        {
            out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &candidate.word,
                source: L2_SURFACE_MOTIF_CELL,
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: surface_motif_typo_risk(context, distance),
                l1,
                context,
            }));
        } else if !stable_input
            && is_completion
            && options.is_enabled(L2_SURFACE_COMPLETION_CELL)
            && candidate_len.saturating_sub(len) <= 10
        {
            out.push(surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &candidate.word,
                source: L2_SURFACE_COMPLETION_CELL,
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: 0.06,
                l1,
                context,
            }));
        }
        if out.len() >= 8 {
            break;
        }
    }
    out
}

pub(super) fn form_attractor_word_candidates(
    prefix: &str,
    token: &str,
    context: &TailContext,
    l1: &[WavePacket],
    options: &WaveOptions,
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let (leading, word, trailing) = split_word_punctuation(token);
    let normalized = word.to_lowercase();
    let len = normalized.chars().count();
    if !(4..=18).contains(&len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let stable_input = surface_motif_stable_existing_word(&normalized);
    let extra_letter_frontier = leading_extra_letter_center_frontier(&normalized);
    if stable_input && (context.token_count() < 2 || extra_letter_frontier.is_empty()) {
        return Vec::new();
    }

    let context_tokens = llmwave::tokenize(prefix);
    let usage = usage_prior::cached_usage_prior_snapshot();
    let transition_state =
        crate::transition_relation::signed_memory_state_id(&format!("{prefix}{token}"));
    let surface_candidates = surface_motif_memory().surface_candidates(&normalized, 32);
    let phase_deltas = l2_birth_phase_deltas(&normalized, &surface_candidates, options);
    let mut out = surface_candidates
        .into_iter()
        .filter_map(|candidate| {
            let replacement_lower = candidate.word;
            if replacement_lower == normalized {
                return None;
            }
            if stable_input && !extra_letter_frontier.contains(&replacement_lower) {
                return None;
            }
            let distance = damerau_levenshtein(&normalized, &replacement_lower);
            let sparse_internal_omission = crate::text_metrics::sparse_internal_omission_count(
                &normalized,
                &replacement_lower,
            )
            .is_some();
            let ranked_score =
                surface_attractor_score(candidate.score, &replacement_lower).saturating_add(
                    typed_damage_score_bonus(&normalized, &replacement_lower, distance),
                );
            let hot = usage.hot_readout(
                &context_tokens,
                LEXICAL_ATTRACTOR_CELL,
                "replacement",
                &transition_state,
                &replacement_lower,
            );
            let signed_bonus =
                ((hot.transition.signed_weight * 180.0).round() as i32).clamp(-180, 180);
            let usage_bonus = ((hot.word_prior + hot.context_prior) * 1_600.0)
                .round()
                .clamp(0.0, 260.0) as u32;
            let rejected_penalty = ((hot.rejected_prior + hot.context_rejected) * 1_400.0)
                .round()
                .clamp(0.0, 220.0) as u32;
            let authority_score = if signed_bonus.is_negative() {
                ranked_score.saturating_sub(signed_bonus.unsigned_abs())
            } else {
                ranked_score.saturating_add(signed_bonus as u32)
            }
            .saturating_add(usage_bonus)
            .saturating_sub(rejected_penalty);

            if !form_attractor_has_authority(
                &normalized,
                &replacement_lower,
                len,
                distance,
                authority_score,
            ) || fuzzy_surface_candidate_blocked(word, &normalized, &replacement_lower)
            {
                return None;
            }

            let phase_delta = phase_deltas
                .get(&replacement_lower)
                .copied()
                .unwrap_or_default();
            let birth_score = add_phase_birth_delta(authority_score, phase_delta);

            let risk = typed_damage_risk(context, &normalized, &replacement_lower, distance)
                + ((hot.rejected_prior + hot.context_rejected) * 0.24).clamp(0.0, 0.16)
                - (hot.transition.attraction * 0.08).clamp(0.0, 0.06)
                + (hot.transition.repulsion * 0.12).clamp(0.0, 0.10);
            let mut candidate = surface_motif_candidate(SurfaceMotifCandidateInput {
                prefix,
                leading,
                word,
                trailing,
                replacement_lower: &replacement_lower,
                source: LEXICAL_ATTRACTOR_CELL,
                score: birth_score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                prefix_match: candidate.prefix_match,
                distance,
                risk: risk.clamp(0.04, 0.40),
                l1,
                context,
            });
            if stable_input {
                candidate
                    .support
                    .push("stable-input-requires-context-proof".to_string());
            }
            if sparse_internal_omission {
                candidate
                    .support
                    .push("l2-operator:sparse-internal-multi-omission".to_string());
            }
            annotate_typed_damage_support(&mut candidate, &normalized, &replacement_lower);
            apply_learned_transition_pressure(&mut candidate, &hot.transition);
            record_birth_phase(&mut candidate, phase_delta);
            Some(candidate)
        })
        .collect::<Vec<_>>();

    for replacement_lower in extra_letter_frontier {
        if out.iter().any(|candidate| {
            candidate
                .text
                .split_whitespace()
                .last()
                .is_some_and(|word| word.eq_ignore_ascii_case(&replacement_lower))
        }) {
            continue;
        }
        let mut candidate = surface_motif_candidate(SurfaceMotifCandidateInput {
            prefix,
            leading,
            word,
            trailing,
            replacement_lower: &replacement_lower,
            source: LEXICAL_ATTRACTOR_CELL,
            score: 1_440,
            l1_overlap: len.saturating_sub(1),
            l2_overlap: len.saturating_sub(2),
            motif_overlap: len.saturating_sub(3),
            prefix_match: false,
            distance: 1,
            risk: typed_damage_risk(context, &normalized, &replacement_lower, 1),
            l1,
            context,
        });
        candidate
            .support
            .push("l2-minimal-transition:extra-letter".to_string());
        if stable_input {
            candidate
                .support
                .push("stable-input-requires-context-proof".to_string());
        }
        out.push(candidate);
    }

    out.sort_by(|left, right| {
        (right.energy - right.risk)
            .total_cmp(&(left.energy - left.risk))
            .then_with(|| left.text.cmp(&right.text))
    });
    out.dedup_by(|left, right| left.text == right.text);
    let reserved = out
        .iter()
        .filter(|candidate| {
            candidate
                .support
                .iter()
                .any(|item| item == "l2-operator:sparse-internal-multi-omission")
        })
        .take(4)
        .cloned()
        .collect::<Vec<_>>();
    out.truncate(L2_FORM_ATTRACTOR_LIMIT);
    for candidate in reserved {
        if out.iter().any(|item| item.text == candidate.text) {
            continue;
        }
        if out.len() == L2_FORM_ATTRACTOR_LIMIT {
            out.pop();
        }
        out.push(candidate);
    }
    out
}

// The field may resolve a close L2 tie, but it must not dominate the surface
// center. A bounded transfer keeps phase as interference, not a second
// authority scale.
const L2_BIRTH_PHASE_MAX_DELTA: i32 = 192;

/// Read phase centers before local lattice ordering, not after DecisionCore has
/// already received a narrowed candidate set. A delta is emitted only for a
/// promoted lexical competition profile and is normalized across this one L2
/// lattice. It never changes structural admission eligibility.
fn l2_birth_phase_deltas(
    original: &str,
    candidates: &[LexicalPhaseCandidate],
    options: &WaveOptions,
) -> std::collections::BTreeMap<String, i32> {
    if !options.l2_phase_lattice_apply() || candidates.len() < 2 {
        return std::collections::BTreeMap::new();
    }
    let strengths = candidates
        .iter()
        .filter_map(|candidate| {
            let readout = crate::nanda_wave::l2_transition_phase_shadow_readout(
                original,
                &candidate.word,
                "typo",
                None,
            );
            (readout.package_loaded
                && readout.operator_present
                && readout.operator_promoted
                && readout.lexical_competition_ready)
                .then_some((candidate.word.clone(), readout.lexical_margin_micro))
        })
        .collect::<Vec<_>>();
    let Some(minimum) = strengths.iter().map(|(_, strength)| *strength).min() else {
        return std::collections::BTreeMap::new();
    };
    let Some(maximum) = strengths.iter().map(|(_, strength)| *strength).max() else {
        return std::collections::BTreeMap::new();
    };
    if minimum == maximum {
        return std::collections::BTreeMap::new();
    }
    strengths
        .into_iter()
        .map(|(word, strength)| {
            let normalized = (strength - minimum) as f64 / (maximum - minimum) as f64;
            let delta = (normalized.mul_add(2.0, -1.0) * f64::from(L2_BIRTH_PHASE_MAX_DELTA))
                .round() as i32;
            (
                word,
                delta.clamp(-L2_BIRTH_PHASE_MAX_DELTA, L2_BIRTH_PHASE_MAX_DELTA),
            )
        })
        .collect()
}

fn add_phase_birth_delta(score: u32, delta: i32) -> u32 {
    if delta.is_negative() {
        score.saturating_sub(delta.unsigned_abs())
    } else {
        score.saturating_add(delta as u32)
    }
}

fn record_birth_phase(candidate: &mut WordCandidate, delta: i32) {
    if delta != 0 {
        candidate
            .support
            .push(format!("l2-phase-birth:energy-delta={delta}"));
    }
}

fn leading_extra_letter_center_frontier(input: &str) -> Vec<String> {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.len() < 4 {
        return Vec::new();
    }
    let candidate = chars.into_iter().skip(1).collect::<String>();
    l2_center_contains_surface(&candidate)
        .then_some(candidate)
        .into_iter()
        .collect()
}

fn apply_learned_transition_pressure(
    candidate: &mut WordCandidate,
    transition: &usage_prior::UsageTransitionSignal,
) {
    let pressure = (transition.attraction - transition.repulsion).clamp(-0.32, 0.32);
    if pressure == 0.0 {
        return;
    }
    candidate.energy = (candidate.energy + pressure * 0.85).clamp(0.0, 1.0);
    candidate.risk = (candidate.risk - pressure * 0.85).clamp(0.02, 0.48);
    candidate.support.push(format!(
        "l4-transition:pressure={pressure:.4} attract={} repel={} state_specific={}",
        transition.attract_count, transition.repel_count, transition.state_specific
    ));
}

pub(super) fn form_attractor_has_authority(
    input: &str,
    candidate: &str,
    input_len: usize,
    distance: usize,
    score: u32,
) -> bool {
    if score >= 1_420 && distance <= 3 {
        return true;
    }
    if surface_motif_typo_allowed(input, candidate, input_len, distance, score) {
        return true;
    }
    let corpus_prior = surface_corpus_prior(candidate);
    input_len >= 6 && distance <= 3 && score >= 1_120 && corpus_prior >= 0.36
}

pub(super) fn surface_attractor_score(base: u32, word: &str) -> u32 {
    base.saturating_add(surface_corpus_score_boost(word))
}

pub(super) fn surface_corpus_score_boost(word: &str) -> u32 {
    let prior = surface_corpus_prior(word);
    (prior * 520.0).round().clamp(0.0, 520.0) as u32
}

pub(super) fn surface_attractor_risk(context: &TailContext, distance: usize, word: &str) -> f32 {
    let base = surface_motif_typo_risk(context, distance);
    (base - surface_corpus_prior(word) * 0.10).clamp(0.04, 0.40)
}

fn typed_damage_risk(context: &TailContext, input: &str, candidate: &str, distance: usize) -> f32 {
    let credit = if sparse_internal_multi_omission(input, candidate) {
        0.115
    } else if single_internal_missing_letter(input, candidate)
        || repeated_letter_collapse(input, candidate)
    {
        0.085
    } else if single_missing_letter(input, candidate) || internal_extra_fragment(input, candidate) {
        0.075
    } else if is_single_adjacent_transposition(input, candidate) {
        0.110
    } else if single_letter_substitution(input, candidate) {
        0.045
    } else if orthographic_sign_repair(input, candidate) {
        0.035
    } else {
        0.0
    };
    (surface_attractor_risk(context, distance, candidate) - credit).clamp(0.04, 0.40)
}

fn typed_damage_score_bonus(input: &str, candidate: &str, distance: usize) -> u32 {
    if sparse_internal_multi_omission(input, candidate) {
        760
    } else if single_internal_missing_letter(input, candidate) {
        680
    } else if single_missing_letter(input, candidate) {
        600
    } else if repeated_letter_collapse(input, candidate) {
        620
    } else if internal_extra_fragment(input, candidate) {
        560
    } else if is_single_adjacent_transposition(input, candidate) {
        700
    } else if single_letter_substitution(input, candidate)
        || orthographic_sign_repair(input, candidate)
    {
        360
    } else if distance == 1 {
        180
    } else {
        0
    }
}

fn stable_input_allows_typed_damage_candidate(
    input: &str,
    candidate: &str,
    distance: usize,
) -> bool {
    if is_common_ru_word(input) || is_user_protected_word(input) || is_ru_live_protected_word(input)
    {
        return false;
    }
    if typed_damage_score_bonus(input, candidate, distance) < 360 {
        return false;
    }
    let candidate_has_reference_authority =
        is_common_ru_word(candidate) || l2_surface_foundation_has_authority(candidate);
    let strong_single_missing =
        single_missing_letter(input, candidate) && candidate_has_reference_authority;
    if crate::russian_lexicon::is_reference_backed_russian_form(input) && !strong_single_missing {
        return false;
    }
    surface_corpus_prior(candidate) > surface_corpus_prior(input) || strong_single_missing
}

fn annotate_typed_damage_support(candidate: &mut WordCandidate, input: &str, replacement: &str) {
    let label = if sparse_internal_multi_omission(input, replacement) {
        "l2-operator:sparse-internal-multi-omission"
    } else if single_internal_missing_letter(input, replacement) {
        "l2-operator:single-internal-missing-letter"
    } else if single_missing_letter(input, replacement) {
        "l2-operator:single-missing-letter"
    } else if repeated_letter_collapse(input, replacement) {
        "l2-operator:repeated-letter-collapse"
    } else if internal_extra_fragment(input, replacement) {
        "l2-operator:internal-extra-fragment"
    } else if single_letter_substitution(input, replacement) {
        "l2-operator:single-letter-substitution"
    } else if is_single_adjacent_transposition(input, replacement) {
        "l2-operator:adjacent-transposition"
    } else if orthographic_sign_repair(input, replacement) {
        "l2-operator:orthographic-sign-repair"
    } else {
        return;
    };
    if !candidate.support.iter().any(|item| item == label) {
        candidate.support.push(label.to_string());
    }
}

fn sparse_internal_multi_omission(input: &str, candidate: &str) -> bool {
    crate::text_metrics::sparse_internal_omission_count(input, candidate).is_some()
}

fn internal_extra_fragment(input: &str, candidate: &str) -> bool {
    let input_chars = input.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    let extra = input_chars.len().saturating_sub(candidate_chars.len());
    if !(1..=3).contains(&extra)
        || input_chars.len() < 5
        || input_chars.first() != candidate_chars.first()
        || input_chars.last() != candidate_chars.last()
    {
        return false;
    }

    let mut candidate_index = 0usize;
    let mut removed = Vec::with_capacity(extra);
    for (input_index, ch) in input_chars.iter().enumerate() {
        if candidate_chars.get(candidate_index) == Some(ch) {
            candidate_index += 1;
        } else {
            removed.push(input_index);
        }
    }
    candidate_index == candidate_chars.len()
        && removed.len() == extra
        && removed
            .iter()
            .all(|index| *index > 0 && *index + 1 < input_chars.len())
}

fn repeated_letter_collapse(input: &str, candidate: &str) -> bool {
    let input_chars = input.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if input_chars.len() != candidate_chars.len() + 1
        || input_chars.len() < 4
        || input_chars.first() != candidate_chars.first()
        || input_chars.last() != candidate_chars.last()
    {
        return false;
    }

    (1..input_chars.len() - 1).any(|skip| {
        let repeated = input_chars.get(skip - 1) == input_chars.get(skip)
            || input_chars.get(skip + 1) == input_chars.get(skip);
        repeated
            && input_chars
                .iter()
                .enumerate()
                .filter_map(|(index, ch)| (index != skip).then_some(*ch))
                .eq(candidate_chars.iter().copied())
    })
}

fn single_letter_substitution(input: &str, candidate: &str) -> bool {
    let input_chars = input.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    input_chars.len() == candidate_chars.len()
        && input_chars.len() >= 4
        && input_chars
            .iter()
            .zip(candidate_chars.iter())
            .filter(|(left, right)| left != right)
            .count()
            == 1
}

fn single_internal_missing_letter(input: &str, candidate: &str) -> bool {
    let input_chars = input.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if input_chars.len() + 1 != candidate_chars.len()
        || input_chars.len() < 4
        || input_chars.first() != candidate_chars.first()
        || input_chars.last() != candidate_chars.last()
        || damerau_levenshtein(input, candidate) != 1
    {
        return false;
    }

    let mut input_index = 0usize;
    let mut inserted_index = None;
    for (candidate_index, ch) in candidate_chars.iter().enumerate() {
        if input_chars.get(input_index) == Some(ch) {
            input_index += 1;
        } else if inserted_index.is_none() {
            inserted_index = Some(candidate_index);
        } else {
            return false;
        }
    }
    input_index == input_chars.len()
        && inserted_index.is_some_and(|index| index > 0 && index + 1 < candidate_chars.len())
}

fn single_missing_letter(input: &str, candidate: &str) -> bool {
    let input_chars = input.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if input_chars.len() + 1 != candidate_chars.len()
        || input_chars.len() < 3
        || input_chars.first() != candidate_chars.first()
        || damerau_levenshtein(input, candidate) != 1
    {
        return false;
    }
    let mut input_index = 0usize;
    for ch in &candidate_chars {
        if input_chars.get(input_index) == Some(ch) {
            input_index += 1;
        }
    }
    input_index == input_chars.len()
}

fn orthographic_sign_repair(input: &str, candidate: &str) -> bool {
    let input_chars = input.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if input_chars.len() != candidate_chars.len() || input_chars.len() < 3 {
        return false;
    }
    let mut changed = None;
    for (index, (left, right)) in input_chars.iter().zip(&candidate_chars).enumerate() {
        if left == right {
            continue;
        }
        if changed.is_some() || *left != 'ь' || *right != 'ъ' {
            return false;
        }
        changed = Some(index);
    }
    changed.is_some_and(|index| {
        input_chars
            .get(index + 1)
            .is_some_and(|next| matches!(next, 'е' | 'ё' | 'ю' | 'я'))
    })
}

fn orthographic_sign_frontier(input: &str) -> Vec<String> {
    crate::russian_typo_candidates::generate_hard_sign_candidates(input)
        .filter(|candidate| {
            l2_decoder_contains_surface(candidate)
                || crate::russian_lexicon::is_known_russian_word_or_form(candidate)
        })
        .collect()
}

pub(super) fn surface_corpus_prior(word: &str) -> f32 {
    if is_common_ru_word(word) {
        return 1.0;
    }
    if let Some(rank) = l2_surface_foundation_rank(word) {
        return match rank {
            0..=999 => 0.82,
            1_000..=4_999 => 0.66,
            5_000..=19_999 => 0.44,
            20_000..=59_999 => 0.24,
            _ => 0.10,
        };
    }
    if crate::russian_lexicon::is_known_russian_word_or_form(word) {
        0.18
    } else {
        0.0
    }
}

pub(super) fn repeated_letter_surface_candidate(
    prefix: &str,
    leading: &str,
    word: &str,
    trailing: &str,
    normalized: &str,
    l1: &[WavePacket],
    context: &TailContext,
) -> Option<WordCandidate> {
    if normalized.chars().count() < 3 || is_common_ru_word(normalized) {
        return None;
    }
    if !has_adjacent_repeated_char(normalized) {
        return None;
    }
    let replacement = crate::ru_typo::correct_repeated_letter(word)?;
    let replacement_lower = replacement.to_lowercase();
    if replacement_lower == normalized || !is_known_russian_word_or_form(&replacement_lower) {
        return None;
    }
    let distance = damerau_levenshtein(normalized, &replacement_lower);
    if distance == 0 || distance > 3 {
        return None;
    }
    Some(surface_motif_candidate(SurfaceMotifCandidateInput {
        prefix,
        leading,
        word,
        trailing,
        replacement_lower: &replacement_lower,
        source: L2_SURFACE_MOTIF_CELL,
        score: 940,
        l1_overlap: 0,
        l2_overlap: 0,
        motif_overlap: 0,
        prefix_match: false,
        distance,
        risk: 0.08,
        l1,
        context,
    }))
}

pub(super) fn has_adjacent_repeated_char(word: &str) -> bool {
    let mut prev = None;
    for ch in word.chars() {
        if prev == Some(ch) {
            return true;
        }
        prev = Some(ch);
    }
    false
}

pub(super) fn surface_motif_typo_has_authority(
    original: &str,
    candidate: &str,
    score: u32,
    surface_candidates: &[LexicalPhaseCandidate],
    fuzzy_authority: &[String],
) -> bool {
    let candidate_distance = damerau_levenshtein(original, candidate);
    let l2_surface_match = surface_candidates
        .iter()
        .any(|surface| surface.word == candidate);
    if l2_surface_match
        && surface_motif_typo_allowed(
            original,
            candidate,
            original.chars().count(),
            candidate_distance,
            score,
        )
    {
        return true;
    }
    if surface_candidates.is_empty()
        && fuzzy_authority.len() == 1
        && fuzzy_authority
            .first()
            .is_some_and(|word| word == candidate)
        && candidate_distance == 1
    {
        return true;
    }
    if score < 880 || !fuzzy_authority.iter().any(|word| word == candidate) {
        return false;
    }
    let original_len = original.chars().count();
    !surface_candidates.iter().any(|other| {
        if other.word == candidate {
            return false;
        }
        let other_distance = damerau_levenshtein(original, &other.word);
        surface_motif_typo_allowed(
            original,
            &other.word,
            original_len,
            other_distance,
            other.score,
        ) && other_distance <= candidate_distance
            && other.score.saturating_add(24) >= score
    })
}

pub(super) fn repeated_all_caps_surface_allowed(
    original_word: &str,
    original_lower: &str,
    candidate: &str,
) -> bool {
    original_word
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
        && has_adjacent_repeated_char(original_lower)
        && damerau_levenshtein(original_lower, candidate) <= 3
}

pub(super) fn fuzzy_surface_candidate_blocked(
    original_word: &str,
    original_lower: &str,
    candidate: &str,
) -> bool {
    if is_user_protected_word(original_lower) || is_ru_live_protected_word(original_lower) {
        return true;
    }
    if original_word
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        return true;
    }
    if crate::ru_typo::rewrites_protected_pattern_term_stem(original_lower, candidate) {
        return true;
    }
    if same_stem_inflection_rewrite(original_lower, candidate)
        && surface_motif_stable_existing_word(original_lower)
    {
        return true;
    }
    looks_like_live_russian_it_verb(original_lower) && !candidate.ends_with("ит")
}

pub(super) fn surface_motif_stable_existing_word(word: &str) -> bool {
    is_common_ru_word(word)
        || is_user_protected_word(word)
        || is_ru_live_protected_word(word)
        // This query follows the active hot-field policy. In IME mode it stays
        // compact; in compiler/test mode it can also recognize attested forms.
        || is_known_russian_word_or_form(word)
        // A derived form is stable when its lexical stem is present in the
        // compact L2 foundation. This is reference evidence, not a per-word
        // correction rule, and keeps post-space completion from extending a
        // completed inflection.
        || crate::russian_lexicon::is_reference_backed_russian_form(word)
        || surface_motif_runtime_known_surface(word)
}

pub(super) fn surface_motif_runtime_known_surface(word: &str) -> bool {
    crate::hot_field::HotFieldSnapshot::current()
        .input_surface_readout(word)
        .is_known()
}

pub(super) fn surface_motif_known_surface(word: &str) -> bool {
    surface_motif_strict_known_surface(word) || l2_center_contains_surface(word)
}

pub(super) fn surface_motif_strict_known_surface(word: &str) -> bool {
    is_common_ru_word(word)
        || is_ru_live_protected_word(word)
        || is_user_protected_word(word)
        || crate::russian_lexicon::russian_dictionary().contains(word)
}

pub(super) fn same_stem_inflection_rewrite(original: &str, candidate: &str) -> bool {
    same_stem_suffix_rewrite(
        original,
        candidate,
        &[
            "ыми", "ими", "ого", "его", "ому", "ему", "ом", "ем", "ой", "ей", "ая", "яя", "ое",
            "ее", "ые", "ие", "ых", "их", "ый", "ий",
        ],
    ) || same_stem_suffix_rewrite(
        original,
        candidate,
        &[
            "ешься",
            "ишься",
            "ется",
            "ются",
            "ался",
            "алась",
            "ались",
            "алось",
            "ился",
            "илась",
            "ились",
            "илось",
            "аете",
            "аешь",
            "айте",
            "ают",
            "ешь",
            "ишь",
            "ает",
            "ать",
            "ить",
            "еть",
            "уть",
            "нут",
            "ют",
            "ут",
            "ет",
            "ит",
            "ай",
            "ил",
            "ла",
            "ли",
            "ло",
            "у",
        ],
    )
}

pub(super) fn same_stem_suffix_rewrite(
    original: &str,
    candidate: &str,
    suffixes: &[&'static str],
) -> bool {
    let Some((original_stem, original_suffix)) = split_known_suffix(original, suffixes) else {
        return false;
    };
    let Some((candidate_stem, candidate_suffix)) = split_known_suffix(candidate, suffixes) else {
        return false;
    };
    original_suffix != candidate_suffix
        && original_stem == candidate_stem
        && original_stem.chars().count() >= 3
}

pub(super) fn split_known_suffix<'a>(
    word: &'a str,
    suffixes: &[&'static str],
) -> Option<(&'a str, &'static str)> {
    suffixes.iter().find_map(|suffix| {
        let stem = word.strip_suffix(suffix)?;
        (!stem.is_empty()).then_some((stem, *suffix))
    })
}

pub(super) fn looks_like_live_russian_it_verb(word: &str) -> bool {
    word.chars().count() >= 5
        && word.ends_with("ит")
        && !word.ends_with("оит")
        && !word.ends_with("еит")
        && !word.ends_with("аит")
}

pub(super) struct SurfaceMotifCandidateInput<'a> {
    prefix: &'a str,
    leading: &'a str,
    word: &'a str,
    trailing: &'a str,
    replacement_lower: &'a str,
    source: &'static str,
    score: u32,
    l1_overlap: usize,
    l2_overlap: usize,
    motif_overlap: usize,
    prefix_match: bool,
    distance: usize,
    risk: f32,
    l1: &'a [WavePacket],
    context: &'a TailContext,
}

pub(super) fn surface_motif_candidate(input: SurfaceMotifCandidateInput<'_>) -> WordCandidate {
    let replacement_word = apply_word_case(input.word, input.replacement_lower);
    // Preserve the field ordering for DecisionCore. The old 900 divisor
    // saturated nearly every serious candidate at 0.95, turning ties into
    // accidental lexical ordering.
    let field_energy = (0.42 + input.score as f32 / 6_000.0).clamp(0.42, 0.95);
    let sensor_floor = l1_energy(input.l1, "ScriptCell32") * 0.72;
    let energy = field_energy.max(sensor_floor);
    WordCandidate {
        text: format!(
            "{}{}{}{}",
            input.prefix, input.leading, replacement_word, input.trailing
        ),
        origin: if input.source == L2_SURFACE_COMPLETION_CELL {
            CandidateOrigin::Completion
        } else {
            CandidateOrigin::L2Surface
        },
        source: input.source,
        energy,
        risk: input.risk,
        support: {
            let mut support = candidate_support(input.l1, input.context);
            support.push(format!(
                "l2-surface:score={} l1_overlap={} l2_overlap={} motif_overlap={} prefix={} distance={}",
                input.score,
                input.l1_overlap,
                input.l2_overlap,
                input.motif_overlap,
                input.prefix_match,
                input.distance
            ));
            support
        },
    }
}

pub(super) fn surface_motif_typo_allowed(
    input: &str,
    candidate: &str,
    input_len: usize,
    distance: usize,
    score: u32,
) -> bool {
    distance == 1
        || is_single_adjacent_transposition(input, candidate)
        || single_missing_letter(input, candidate)
        || sparse_internal_multi_omission(input, candidate)
        || internal_extra_fragment(input, candidate)
        || repeated_letter_collapse(input, candidate)
        || single_letter_substitution(input, candidate)
        || orthographic_sign_repair(input, candidate)
        || (input_len >= 6 && distance == 2 && score >= 300)
        || (input_len >= 8 && distance == 3 && score >= 380)
}

pub(super) fn is_single_adjacent_transposition(input: &str, candidate: &str) -> bool {
    let mut left = input.chars().collect::<Vec<_>>();
    let right = candidate.chars().collect::<Vec<_>>();
    if left.len() != right.len() || left.len() < 2 || left == right {
        return false;
    }
    for index in 0..left.len() - 1 {
        left.swap(index, index + 1);
        if left == right {
            return true;
        }
        left.swap(index, index + 1);
    }
    false
}

pub(super) fn surface_motif_typo_risk(context: &TailContext, distance: usize) -> f32 {
    let phrase_bonus = if context.token_count() >= 2 {
        -0.03
    } else {
        0.05
    };
    (0.10 + distance as f32 * 0.06 + phrase_bonus).clamp(0.06, 0.40)
}

pub(super) fn surface_motif_memory() -> &'static LexicalPhaseMemory {
    default_memory()
        .expect("missing L2 lexical phase artifact; run scripts/install-l2-lexical-phase.sh")
}

pub(crate) fn l2_surface_foundation_contains(word: &str) -> bool {
    optional_surface_foundation_contains(super::super::lexical_phase::default_memory(), word)
}

pub(crate) fn l2_surface_foundation_rank(word: &str) -> Option<usize> {
    optional_surface_foundation_rank(super::super::lexical_phase::default_memory(), word)
}

fn optional_surface_foundation_contains(memory: Option<&LexicalPhaseMemory>, word: &str) -> bool {
    memory.is_some_and(|memory| memory.contains_surface(word))
}

fn optional_surface_foundation_rank(
    memory: Option<&LexicalPhaseMemory>,
    word: &str,
) -> Option<usize> {
    memory.and_then(|memory| memory.surface_rank(word))
}

pub(crate) fn l2_surface_foundation_has_authority(word: &str) -> bool {
    l2_surface_foundation_rank(word).is_some_and(|rank| rank < 20_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> WordCandidate {
        WordCandidate {
            text: "исправление".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.80,
            risk: 0.20,
            support: Vec::new(),
        }
    }

    #[test]
    fn missing_surface_artifact_cannot_create_learning_authority() {
        assert!(!optional_surface_foundation_contains(None, "прекрасно"));
        assert_eq!(optional_surface_foundation_rank(None, "прекрасно"), None);
    }

    #[test]
    fn learned_transition_pressure_is_signed_and_symmetric() {
        let mut attracted = candidate();
        apply_learned_transition_pressure(
            &mut attracted,
            &usage_prior::UsageTransitionSignal {
                attraction: 0.10,
                attract_count: 6,
                state_specific: true,
                ..Default::default()
            },
        );

        let mut repelled = candidate();
        apply_learned_transition_pressure(
            &mut repelled,
            &usage_prior::UsageTransitionSignal {
                repulsion: 0.10,
                repel_count: 8,
                state_specific: true,
                ..Default::default()
            },
        );

        assert!(attracted.energy > 0.80);
        assert!(attracted.risk < 0.20);
        assert!(repelled.energy < 0.80);
        assert!(repelled.risk > 0.20);
    }

    #[test]
    fn leading_extra_letter_frontier_resolves_through_l2_centers() {
        let frontier = leading_extra_letter_center_frontier("атак");

        assert_eq!(frontier, vec!["так"]);
        assert!(leading_extra_letter_center_frontier("можем").is_empty());
    }
}
