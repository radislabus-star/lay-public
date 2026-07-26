use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::{
    CandidateEvidence, CorrectionDecisionSource, TypingErrorClass, UnifiedCorrectionCandidate,
};
use crate::text_case::apply_word_case;
use crate::typing_transition::{
    action as action_operator,
    decision::TransitionDecisionCore,
};
use crate::word_reader::{replace_last_text_word, split_last_alphabetic_token};

use super::runtime::{L2FieldBridgeKind, L2FieldShadowReadout};

pub(crate) fn compact_text_candidates(
    original: &str,
    l2_peak_context: &crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext,
) -> Vec<UnifiedCorrectionCandidate> {
    donor_text_candidates(L2FieldBridgeKind::CompactL2, original, l2_peak_context).candidates
}

pub(crate) fn shadow_text_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    shadow_owned_text_candidates(original).candidates
}

#[cfg(test)]
pub(crate) fn compact_l11_restore_candidate(
    original: &str,
    token: &str,
) -> Option<UnifiedCorrectionCandidate> {
    l11_restore_candidate(L2FieldBridgeKind::CompactL2, original, token)
}

fn donor_text_candidates(
    kind: L2FieldBridgeKind,
    original: &str,
    l2_peak_context: &crate::nanda_wave::l2_wave_peak::L2CorrectionPeakContext,
) -> L2FieldShadowReadout {
    const HOT_L2_CANDIDATE_LIMIT: usize = 8;

    let Some((context_prefix, token)) = split_last_alphabetic_token(original) else {
        return L2FieldShadowReadout::default();
    };
    let surface_candidates = l2_peak_context
        .center_candidates()
        .iter()
        .take(HOT_L2_CANDIDATE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let mut candidates = l2_surface_unified_candidates(
        kind,
        original,
        token,
        &surface_candidates,
    );

    let allow_noisy_layout_projection = !l2_peak_context.has_local_single_edit_peak();
    let layout_candidate = if allow_noisy_layout_projection {
        crate::nanda_wave::l2::hot_layout_candidate(original)
    } else {
        crate::nanda_wave::l2::hot_layout_candidate_with_noisy_projection(original, false)
    };
    if let Some(layout) = layout_candidate
        .as_ref()
        .and_then(|candidate| nanda_word_candidate(original, candidate))
    {
        candidates.push(layout);
    }

    candidates.extend(
        crate::nanda_wave::l2::ime_l2_boundary_candidates(context_prefix, token, 2)
            .into_iter()
            .filter_map(|candidate| {
                if !boundary_surface_splits_current_token(token, &candidate.surface) {
                    return None;
                }
                let replacement = replace_last_text_word(original, &candidate.surface)?;
                let origin = CandidateOrigin::Boundary;
                let error_class = action_operator::classify_token_transition(
                    original,
                    &replacement,
                    origin,
                    TypingErrorClass::GluedWords,
                );
                let local_original = format!("{token} ");
                let local_replacement = format!("{} ", candidate.surface);
                let gate = TransitionDecisionCore::admit_candidate_proposal(
                    &local_original,
                    &local_replacement,
                    error_class,
                    origin,
                );
                Some(UnifiedCorrectionCandidate::new(
                    replacement,
                    CorrectionDecisionSource::Nanda,
                    origin,
                    kind.boundary_source_id(),
                    error_class,
                    gate,
                ))
            }),
    );

    if let Some(candidate) = l11_restore_candidate(kind, original, token) {
        candidates.push(candidate);
    }

    L2FieldShadowReadout::new(candidates)
}

fn shadow_owned_text_candidates(original: &str) -> L2FieldShadowReadout {
    const HOT_L2_CANDIDATE_LIMIT: usize = 8;
    const SHADOW_SURFACE_MATERIAL_LIMIT: usize = 16;

    let Some((context_prefix, token)) = split_last_alphabetic_token(original) else {
        return L2FieldShadowReadout::default();
    };
    let normalized_token = token.to_lowercase();
    let surface_candidates = if normalized_token.chars().count() >= 2
        && normalized_token
            .chars()
            .all(crate::keyboard::is_cyrillic_letter)
    {
        crate::nanda_wave::l2::correction_l2_word_candidates(
            context_prefix,
            token,
            SHADOW_SURFACE_MATERIAL_LIMIT,
        )
    } else {
        Vec::new()
    };
    let mut bounded_surface_candidates = surface_candidates
        .iter()
        .take(HOT_L2_CANDIDATE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let same_lemma_morphology =
        apply_shadow_same_lemma_morphology(context_prefix, &mut bounded_surface_candidates);
    let near_neighbor_lexical =
        apply_shadow_near_neighbor_lexical(token, &mut bounded_surface_candidates);
    let mut candidates = l2_surface_unified_candidates(
        L2FieldBridgeKind::Shadow,
        original,
        token,
        &bounded_surface_candidates,
    );
    if let Some(readout) = same_lemma_morphology.as_ref() {
        promote_shadow_same_lemma_morphology(&mut candidates, readout);
    }
    if let Some(readout) = near_neighbor_lexical.as_ref() {
        promote_shadow_near_neighbor_lexical(&mut candidates, readout);
    }

    let allow_noisy_layout_projection = !shadow_has_local_single_edit_peak(token, &surface_candidates);
    let layout_candidate = if allow_noisy_layout_projection {
        crate::nanda_wave::l2::hot_layout_candidate(original)
    } else {
        crate::nanda_wave::l2::hot_layout_candidate_with_noisy_projection(original, false)
    };
    if let Some(layout) = layout_candidate
        .as_ref()
        .and_then(|candidate| nanda_word_candidate(original, candidate))
    {
        candidates.push(layout);
    }

    candidates.extend(
        crate::nanda_wave::l2::ime_l2_boundary_candidates(context_prefix, token, 2)
            .into_iter()
            .filter_map(|candidate| {
                if !boundary_surface_splits_current_token(token, &candidate.surface) {
                    return None;
                }
                let replacement = replace_last_text_word(original, &candidate.surface)?;
                let origin = CandidateOrigin::Boundary;
                let error_class = action_operator::classify_token_transition(
                    original,
                    &replacement,
                    origin,
                    TypingErrorClass::GluedWords,
                );
                let local_original = format!("{token} ");
                let local_replacement = format!("{} ", candidate.surface);
                let gate = TransitionDecisionCore::admit_candidate_proposal(
                    &local_original,
                    &local_replacement,
                    error_class,
                    origin,
                );
                Some(UnifiedCorrectionCandidate::new(
                    replacement,
                    CorrectionDecisionSource::Nanda,
                    origin,
                    L2FieldBridgeKind::Shadow.boundary_source_id(),
                    error_class,
                    gate,
                ))
            }),
    );

    if let Some(candidate) = l11_restore_candidate(L2FieldBridgeKind::Shadow, original, token) {
        candidates.push(candidate);
    }

    L2FieldShadowReadout::new(candidates)
}

fn l11_restore_candidate(
    kind: L2FieldBridgeKind,
    original: &str,
    token: &str,
) -> Option<UnifiedCorrectionCandidate> {
    const HOT_L11_LIMIT: usize = 4;

    let socket_path = crate::nanda_wave::default_l11_socket_path();
    if !socket_path.exists() {
        return None;
    }
    let restored = crate::nanda_wave::request_l11_authoritative_surface(
        &socket_path,
        token,
        HOT_L11_LIMIT,
        l11_service_timeout(),
    )
    .ok()
    .flatten()?;
    if restored.chars().any(char::is_whitespace) || restored.to_lowercase() == token.to_lowercase()
    {
        return None;
    }
    let restored_word = apply_word_case(token, &restored);
    let replacement = replace_last_text_word(original, &restored_word)?;
    let origin = CandidateOrigin::L2Surface;
    let error_class = action_operator::classify_token_transition(
        original,
        &replacement,
        origin,
        TypingErrorClass::Unknown,
    );
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        original,
        &replacement,
        error_class,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
        origin,
        kind.l11_source_id(),
        error_class,
        gate,
    ))
}

fn l11_service_timeout() -> std::time::Duration {
    std::time::Duration::from_millis(
        std::env::var("LAY_L11_SERVICE_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(12)
            .clamp(1, 250),
    )
}

fn nanda_word_candidate(
    original: &str,
    candidate: &crate::nanda_wave::WordCandidate,
) -> Option<UnifiedCorrectionCandidate> {
    let replacement = preserve_candidate_trailing_separator(original, &candidate.text);
    if replacement == original {
        return None;
    }
    let origin = candidate.origin;
    let error_class = action_operator::classify_token_transition(
        original,
        &replacement,
        origin,
        TypingErrorClass::Unknown,
    );
    if candidate.source == "BoundaryCell32"
        && origin == CandidateOrigin::Boundary
        && matches!(
            error_class,
            TypingErrorClass::GluedWords | TypingErrorClass::SplitWord
        )
        && !crate::text_metrics::current_token_boundary_split(original, &replacement)
    {
        return None;
    }
    let gate = TransitionDecisionCore::admit_candidate_proposal(
        original,
        &replacement,
        error_class,
        origin,
    );
    Some(UnifiedCorrectionCandidate::new(
        replacement,
        CorrectionDecisionSource::Nanda,
        origin,
        candidate.source,
        error_class,
        gate,
    ))
}

fn l2_surface_unified_candidates(
    kind: L2FieldBridgeKind,
    original: &str,
    token: &str,
    candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
) -> Vec<UnifiedCorrectionCandidate> {
    let normalized_token = token.to_lowercase();
    candidates
        .iter()
        .filter(|candidate| candidate.surface.to_lowercase() != normalized_token)
        .cloned()
        .filter_map(|candidate| {
            let candidate_word = apply_word_case(token, &candidate.surface);
            let replacement = replace_last_text_word(original, &candidate_word)?;
            let origin = CandidateOrigin::L2Surface;
            let error_class = action_operator::classify_token_transition(
                original,
                &replacement,
                origin,
                TypingErrorClass::Unknown,
            );
            let gate = TransitionDecisionCore::admit_candidate_proposal(
                original,
                &replacement,
                error_class,
                origin,
            );
            Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Nanda,
                origin,
                kind.surface_source_id(),
                error_class,
                gate,
            ))
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShadowSameLemmaMorphology {
    winner_surface: String,
    cohort_surfaces: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShadowNearNeighborLexical {
    winner_surface: String,
    cohort_surfaces: Vec<String>,
}

fn apply_shadow_same_lemma_morphology(
    context_prefix: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
) -> Option<ShadowSameLemmaMorphology> {
    let candidate_surfaces = surface_candidates
        .iter()
        .map(|candidate| candidate.surface.to_lowercase())
        .collect::<Vec<_>>();
    let readout =
        crate::nanda_wave::morphology_phase::shadow_same_lemma_surface_readout(
            context_prefix,
            &candidate_surfaces,
        )?;
    let crate::nanda_wave::morphology_phase::SameLemmaSurfaceReadout::Winner {
        winner_surface,
        cohort_surfaces,
    } = readout
    else {
        return None;
    };
    let cohort = cohort_surfaces
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    surface_candidates.retain(|candidate| {
        let surface = candidate.surface.to_lowercase();
        !cohort.contains(&surface) || surface == winner_surface
    });
    Some(ShadowSameLemmaMorphology {
        winner_surface,
        cohort_surfaces,
    })
}

fn promote_shadow_same_lemma_morphology(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &ShadowSameLemmaMorphology,
) {
    for candidate in candidates {
        let Some((_, word)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)
        else {
            continue;
        };
        let (_, word, _) = crate::word_reader::split_word_punctuation(word);
        if word.to_lowercase() != readout.winner_surface {
            continue;
        }
        let source_id = L2FieldBridgeKind::Shadow.morph_source_id().to_string();
        if candidate.source_id == source_id {
            break;
        }
        let evidence_present = candidate
            .evidence
            .iter()
            .any(|evidence| evidence.source_id == source_id);
        candidate.source_id = source_id.clone();
        if !evidence_present {
            candidate.evidence.push(CandidateEvidence {
                source: candidate.source,
                origin: candidate.origin,
                source_id,
                error_class: candidate.error_class,
                gate: candidate.gate.clone(),
            });
        }
        break;
    }
}

fn apply_shadow_near_neighbor_lexical(
    token: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
) -> Option<ShadowNearNeighborLexical> {
    if surface_candidates.len() < 2 || !token.chars().all(crate::keyboard::is_cyrillic_letter) {
        return None;
    }
    let normalized_token = token.to_lowercase();
    let leader = surface_candidates.first()?;
    let leader_surface = leader.surface.to_lowercase();
    let leader_distance =
        crate::text_metrics::damerau_levenshtein(&normalized_token, &leader_surface);
    let cohort_indices = surface_candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            let surface = candidate.surface.to_lowercase();
            let leader_gap = crate::text_metrics::damerau_levenshtein(&leader_surface, &surface);
            let input_gap =
                crate::text_metrics::damerau_levenshtein(&normalized_token, &surface);
            ((surface == leader_surface)
                || ((1..=2).contains(&leader_gap)
                    && surface.chars().count().abs_diff(leader_surface.chars().count()) <= 2
                    && input_gap <= leader_distance.saturating_add(1)))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if cohort_indices.len() < 2 {
        return None;
    }

    let mut ranked = cohort_indices
        .iter()
        .filter_map(|index| {
            let candidate = surface_candidates.get(*index)?;
            let surface = candidate.surface.to_lowercase();
            Some((
                surface,
                shadow_near_neighbor_strength(&normalized_token, candidate),
                crate::text_metrics::damerau_levenshtein(
                    &normalized_token,
                    &candidate.surface.to_lowercase(),
                ),
                candidate.motif_overlap,
                candidate.l2_overlap,
                candidate.l1_overlap,
            ))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| right.3.cmp(&left.3))
            .then_with(|| right.4.cmp(&left.4))
            .then_with(|| right.5.cmp(&left.5))
            .then_with(|| left.0.cmp(&right.0))
    });
    let winner = ranked.first()?;
    if winner.0 != leader_surface {
        return None;
    }
    let runner_up = ranked.get(1)?;
    let margin = winner.1.saturating_sub(runner_up.1);
    if margin < 220 || winner.2 > runner_up.2 || winner.3 < runner_up.3 {
        return None;
    }

    let cohort = cohort_indices
        .iter()
        .filter_map(|index| surface_candidates.get(*index))
        .map(|candidate| candidate.surface.to_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    surface_candidates.retain(|candidate| {
        let surface = candidate.surface.to_lowercase();
        !cohort.contains(&surface) || surface == winner.0
    });
    Some(ShadowNearNeighborLexical {
        winner_surface: winner.0.clone(),
        cohort_surfaces: cohort.into_iter().collect(),
    })
}

fn shadow_near_neighbor_strength(
    token: &str,
    candidate: &crate::nanda_wave::l2::L2ImeWordCandidate,
) -> i64 {
    let surface = candidate.surface.to_lowercase();
    let distance = crate::text_metrics::damerau_levenshtein(token, &surface);
    let distance_bonus = match distance {
        0 => 240,
        1 => 180,
        2 => 80,
        3 => 20,
        _ => 0,
    };
    let transposition_bonus = i64::from(
        candidate.kind == crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition,
    ) * 48;
    i64::from(candidate.score)
        + i64::from((candidate.motif_overlap as u32).saturating_mul(192))
        + i64::from((candidate.l2_overlap as u32).saturating_mul(128))
        + i64::from((candidate.l1_overlap as u32).saturating_mul(64))
        + i64::from(candidate.accepted_count.min(32).saturating_mul(18))
        + (candidate.context_prior * 600.0).round() as i64
        + (candidate.usage_prior * 320.0).round() as i64
        + distance_bonus
        + transposition_bonus
}

fn promote_shadow_near_neighbor_lexical(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &ShadowNearNeighborLexical,
) {
    for candidate in candidates {
        let Some((_, word)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)
        else {
            continue;
        };
        let (_, word, _) = crate::word_reader::split_word_punctuation(word);
        if word.to_lowercase() != readout.winner_surface {
            continue;
        }
        let source_id = L2FieldBridgeKind::Shadow.near_neighbor_source_id().to_string();
        if candidate.source_id == source_id {
            break;
        }
        let evidence_present = candidate
            .evidence
            .iter()
            .any(|evidence| evidence.source_id == source_id);
        candidate.source_id = source_id.clone();
        if !evidence_present {
            candidate.evidence.push(CandidateEvidence {
                source: candidate.source,
                origin: candidate.origin,
                source_id,
                error_class: candidate.error_class,
                gate: candidate.gate.clone(),
            });
        }
        break;
    }
}

fn shadow_has_local_single_edit_peak(
    token: &str,
    candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
) -> bool {
    let original_word = token.to_lowercase();
    candidates.iter().any(|candidate| {
        matches!(
            candidate.kind,
            crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition
                | crate::nanda_wave::l2::L2ImeWordCandidateKind::Replacement
        ) && crate::text_metrics::damerau_levenshtein(&original_word, &candidate.surface) == 1
            && candidate.l1_overlap > 0
            && candidate.motif_overlap > 0
    })
}

fn preserve_candidate_trailing_separator(original: &str, candidate: &str) -> String {
    let mut out = candidate.to_string();
    if original
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
        && !out.chars().next_back().is_some_and(char::is_whitespace)
    {
        out.push(' ');
    }
    out
}

fn boundary_surface_splits_current_token(token: &str, surface: &str) -> bool {
    let token = token.to_lowercase();
    let parts = crate::typing_transition::proposal_admission::normalized_correction_words(surface);
    parts.len() >= 2 && parts.concat() == token
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lexical_candidate(
        surface: &str,
        score: u32,
        l1_overlap: usize,
        l2_overlap: usize,
        motif_overlap: usize,
    ) -> crate::nanda_wave::l2::L2ImeWordCandidate {
        crate::nanda_wave::l2::L2ImeWordCandidate {
            surface: surface.to_string(),
            kind: crate::nanda_wave::l2::L2ImeWordCandidateKind::Replacement,
            source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
            score,
            l1_overlap,
            l2_overlap,
            motif_overlap,
            usage_prior: 0.0,
            context_prior: 0.0,
            accepted_count: 0,
        }
    }

    #[test]
    fn shadow_near_neighbor_lexical_filters_clear_neighbor_drift() {
        let mut candidates = vec![
            lexical_candidate("посмотри", 1480, 8, 6, 4),
            lexical_candidate("просмотри", 1180, 6, 4, 3),
        ];

        let readout = apply_shadow_near_neighbor_lexical("посмтри", &mut candidates);

        assert_eq!(
            readout,
            Some(ShadowNearNeighborLexical {
                winner_surface: "посмотри".to_string(),
                cohort_surfaces: vec!["посмотри".to_string(), "просмотри".to_string()],
            })
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].surface, "посмотри");
    }

    #[test]
    fn shadow_near_neighbor_lexical_abstains_on_small_margin() {
        let mut candidates = vec![
            lexical_candidate("посмотри", 1240, 6, 4, 3),
            lexical_candidate("просмотри", 1210, 6, 4, 3),
        ];

        let readout = apply_shadow_near_neighbor_lexical("посмтри", &mut candidates);

        assert_eq!(readout, None);
        assert_eq!(candidates.len(), 2);
    }
}
