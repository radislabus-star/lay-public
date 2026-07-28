use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::{
    CandidateEvidence, CorrectionDecisionSource, TypingErrorClass, UnifiedCorrectionCandidate,
};
use crate::text_case::apply_word_case;
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::{replace_last_text_word, split_last_alphabetic_token};

use super::runtime::{L2FieldBridgeKind, L2FieldShadowReadout, L2LexicalSeed, L2LocalVerdict};

#[cfg(test)]
const SHADOW_DONOR_WINNER_WEIGHT: i64 = 5;
#[cfg(test)]
const SAME_LEMMA_DONOR_TIED_BONUS: i64 = 72;
#[cfg(test)]
const SAME_LEMMA_DONOR_ABSTAIN_BONUS: i64 = 24;
#[cfg(test)]
const NEAR_NEIGHBOR_DONOR_TIED_BONUS: i64 = 56;
#[cfg(test)]
const NEAR_NEIGHBOR_DONOR_ABSTAIN_BONUS: i64 = 12;

pub(crate) fn shadow_text_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    let mut candidates = shadow_owned_text_candidates(original).candidates;
    if let Some(candidate) = short_layout_candidate(original) {
        if let Some(existing) = candidates
            .iter_mut()
            .find(|existing| existing.replacement == candidate.replacement)
        {
            existing.merge_evidence(candidate);
        } else {
            candidates.push(candidate);
        }
    }
    candidates
}

pub(crate) fn cold_probe_surfaces(context_prefix: &str, damaged_surface: &str) -> Vec<String> {
    let original = if context_prefix.trim().is_empty() {
        damaged_surface.to_string()
    } else {
        format!("{} {}", context_prefix.trim(), damaged_surface)
    };
    // L3 learns over the bounded L2 lattice, not over L2's already-settled
    // winner. The live edit route still consumes the winner below.
    let mut surfaces: Vec<String> = standalone_surface_field_readout(&original, false)
        .map(|field| {
            field
                .surface_candidates
                .into_iter()
                .map(|candidate| candidate.surface.to_lowercase())
                .collect()
        })
        .unwrap_or_default();
    surfaces.sort();
    surfaces.dedup();
    surfaces
}

fn short_layout_candidate(original: &str) -> Option<UnifiedCorrectionCandidate> {
    let (_, token) = split_last_alphabetic_token(original)?;
    if token.chars().count() != 1 || original.split_whitespace().count() < 2 {
        return None;
    }
    let candidate = crate::nanda_wave::l2::hot_layout_candidate(original)?;
    let replacement = candidate.text;
    let origin = candidate.origin;
    let error_class = action_operator::classify_token_transition(
        original,
        &replacement,
        origin,
        TypingErrorClass::WrongLayout,
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
        candidate.source,
        error_class,
        gate,
    ))
}

fn shadow_owned_text_candidates(original: &str) -> L2FieldShadowReadout {
    let started = std::time::Instant::now();
    let Some(field) = standalone_surface_field_readout(original, true) else {
        return L2FieldShadowReadout::default();
    };
    let mut candidates =
        l2_surface_unified_candidates(original, &field.token, &field.surface_candidates);
    promote_shadow_local_readout(&mut candidates, &field.local_readout);
    demote_shadow_local_surface_cohort(&mut candidates, &field.token, &field.local_readout);
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        let finished = std::time::Instant::now();
        eprintln!(
            "l2_field_trace seeds_us={} field_us={} materialize_us={} total_us={} seeds={} surfaces={} candidates={}",
            field.seed_duration.as_micros(),
            field.field_duration.as_micros(),
            finished
                .duration_since(started)
                .saturating_sub(field.seed_duration)
                .saturating_sub(field.field_duration)
                .as_micros(),
            finished.duration_since(started).as_micros(),
            field.seed_count,
            field.surface_count,
            candidates.len(),
        );
    }

    L2FieldShadowReadout::new(candidates)
}

struct StandaloneSurfaceFieldReadout {
    token: String,
    surface_candidates: Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    local_readout: ShadowCohortReadout,
    seed_count: usize,
    surface_count: usize,
    seed_duration: std::time::Duration,
    field_duration: std::time::Duration,
}

fn standalone_surface_field_readout(
    original: &str,
    settle_winner: bool,
) -> Option<StandaloneSurfaceFieldReadout> {
    const HOT_L2_CANDIDATE_LIMIT: usize = 8;
    const SHADOW_SURFACE_MATERIAL_LIMIT: usize = 16;

    let started = std::time::Instant::now();
    let (context_prefix, token) = split_last_alphabetic_token(original)?;
    let normalized_token = token.to_lowercase();
    if normalized_token.chars().count() < 2
        || !normalized_token
            .chars()
            .all(crate::keyboard::is_cyrillic_letter)
    {
        return None;
    }
    let (surface_candidates, l11_seeds) =
        shadow_surface_seed_candidates(token, SHADOW_SURFACE_MATERIAL_LIMIT);
    let seeds_ready = std::time::Instant::now();
    let seed_count = l11_seeds.len();
    let surface_count = surface_candidates.len();
    let mut bounded_surface_candidates = surface_candidates
        .iter()
        .take(HOT_L2_CANDIDATE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let local_readout = apply_standalone_l2_field(
        context_prefix,
        token,
        &mut bounded_surface_candidates,
        &l11_seeds,
        settle_winner,
    )?;
    let field_ready = std::time::Instant::now();
    Some(StandaloneSurfaceFieldReadout {
        token: token.to_string(),
        surface_candidates: bounded_surface_candidates,
        local_readout,
        seed_count,
        surface_count,
        seed_duration: seeds_ready.duration_since(started),
        field_duration: field_ready.duration_since(seeds_ready),
    })
}

fn shadow_surface_seed_candidates(
    token: &str,
    material_limit: usize,
) -> (
    Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    Vec<crate::nanda_wave::L11SeedSurface>,
) {
    let socket_path = crate::nanda_wave::default_l11_socket_path();
    if !socket_path.exists() {
        return (Vec::new(), Vec::new());
    }
    let seeds = crate::nanda_wave::request_l11_seed_surfaces(
        &socket_path,
        token,
        material_limit.max(1),
        l11_service_timeout(),
    )
    .ok()
    .unwrap_or_default();
    if seeds.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let normalized_token = token.to_lowercase();
    let mut candidates = seeds
        .iter()
        .filter(|seed| !seed.surface.chars().any(char::is_whitespace))
        .map(|seed| l11_seed_only_candidate(&normalized_token, seed))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| {
                crate::text_metrics::damerau_levenshtein(&normalized_token, &left.surface).cmp(
                    &crate::text_metrics::damerau_levenshtein(&normalized_token, &right.surface),
                )
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface.eq_ignore_ascii_case(&right.surface));
    candidates.truncate(material_limit);
    (candidates, seeds)
}

fn l11_seed_only_candidate(
    token: &str,
    seed: &crate::nanda_wave::L11SeedSurface,
) -> crate::nanda_wave::l2::L2ImeWordCandidate {
    let surface = seed.surface.to_lowercase();
    let overlap = seed_surface_overlap(token, &seed.surface) as u32;
    let motif = seed_surface_motif_overlap(token, &seed.surface) as u32;
    let score = 420_u32
        .saturating_add(overlap.saturating_mul(72))
        .saturating_add(motif.saturating_mul(48))
        .saturating_add(seed.score_milli.checked_div(24).unwrap_or(0).min(96))
        .saturating_add(if seed.authority { 48 } else { 0 });
    crate::nanda_wave::l2::L2ImeWordCandidate {
        surface,
        kind: seeded_candidate_kind(token, &seed.surface),
        source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
        score,
        l1_overlap: seed_surface_overlap(token, &seed.surface),
        l2_overlap: if seed.authority { 4 } else { 3 },
        motif_overlap: seed_surface_motif_overlap(token, &seed.surface),
        usage_prior: 0.0,
        context_prior: 0.0,
        accepted_count: u32::from(seed.authority),
    }
}

fn seeded_candidate_kind(
    token: &str,
    surface: &str,
) -> crate::nanda_wave::l2::L2ImeWordCandidateKind {
    let token = token.to_lowercase();
    let surface = surface.to_lowercase();
    if crate::text_metrics::is_adjacent_transposition(&token, &surface) {
        crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition
    } else if surface.starts_with(&token) && surface.chars().count() > token.chars().count() {
        crate::nanda_wave::l2::L2ImeWordCandidateKind::Completion
    } else {
        crate::nanda_wave::l2::L2ImeWordCandidateKind::Replacement
    }
}

fn seed_surface_overlap(token: &str, surface: &str) -> usize {
    let token_len = token.chars().count();
    let surface_len = surface.chars().count();
    let distance =
        crate::text_metrics::damerau_levenshtein(&token.to_lowercase(), &surface.to_lowercase());
    token_len.min(surface_len).saturating_sub(distance).min(10)
}

fn seed_surface_motif_overlap(token: &str, surface: &str) -> usize {
    let distance =
        crate::text_metrics::damerau_levenshtein(&token.to_lowercase(), &surface.to_lowercase());
    match distance {
        0 => 4,
        1 => 3,
        2 => 2,
        3 => 1,
        _ => 0,
    }
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

fn apply_standalone_l2_field(
    context_prefix: &str,
    token: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    seeds: &[crate::nanda_wave::L11SeedSurface],
    settle_winner: bool,
) -> Option<ShadowCohortReadout> {
    let field = super::installed_l2_field().ok()?;
    let lexical_seeds = seeds
        .iter()
        .filter_map(|seed| {
            Some(L2LexicalSeed {
                terminal_id: seed.terminal_id?,
                evidence_milli: i32::try_from(seed.score_milli.min(i32::MAX as u32)).ok()?,
            })
        })
        .collect::<Vec<_>>();
    if lexical_seeds.is_empty() {
        return None;
    }
    let context = format!("{} _", context_prefix.trim());
    let readout = field.readout(&context, &lexical_seeds, 8);
    let terminal_ids = readout
        .candidates
        .iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    let decoded = crate::nanda_wave::request_l11_decoded_surfaces(
        &crate::nanda_wave::default_l11_socket_path(),
        &terminal_ids,
        l11_service_timeout(),
    )
    .ok()?
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_field_readout verdict={:?} candidates={:?}",
            readout.verdict,
            readout
                .candidates
                .iter()
                .map(|candidate| (
                    candidate.terminal_id,
                    decoded.get(&candidate.terminal_id),
                    candidate.local_score,
                    candidate.slot_phase_milli,
                    candidate.neighbor_pressure,
                    candidate.competition_pressure,
                ))
                .collect::<Vec<_>>(),
        );
    }
    let local_by_terminal = readout
        .candidates
        .iter()
        .map(|candidate| (candidate.terminal_id, candidate))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut surfaces_by_terminal = std::collections::BTreeMap::new();
    for terminal_id in &terminal_ids {
        let Some(surface) = decoded.get(terminal_id).map(|value| value.to_lowercase()) else {
            continue;
        };
        surfaces_by_terminal.insert(*terminal_id, surface.clone());
        if surface_candidates
            .iter()
            .any(|candidate| candidate.surface.eq_ignore_ascii_case(&surface))
        {
            continue;
        }
        let Some(local) = local_by_terminal.get(terminal_id) else {
            continue;
        };
        surface_candidates.push(crate::nanda_wave::l2::L2ImeWordCandidate {
            surface: surface.clone(),
            kind: seeded_candidate_kind(token, &surface),
            source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
            score: local.local_score.max(0).min(u32::MAX as i32) as u32,
            l1_overlap: seed_surface_overlap(token, &surface),
            l2_overlap: usize::from(local.slot_phase_milli > 0)
                + usize::from(local.neighbor_pressure > 0)
                + usize::from(local.competition_pressure > 0),
            motif_overlap: seed_surface_motif_overlap(token, &surface),
            usage_prior: 0.0,
            context_prior: (local.slot_phase_milli.max(0) as f32 / 1_000.0).min(1.0),
            accepted_count: u32::from(local.local_score > 0),
        });
    }
    match readout.verdict {
        L2LocalVerdict::Winner { terminal_id } => {
            let winner_surface = surfaces_by_terminal.get(&terminal_id)?.clone();
            let cohort_surfaces = surfaces_by_terminal.into_values().collect::<Vec<_>>();
            if settle_winner {
                surface_candidates
                    .retain(|candidate| candidate.surface.eq_ignore_ascii_case(&winner_surface));
            }
            Some(ShadowCohortReadout::Winner {
                winner_surface,
                cohort_surfaces,
            })
        }
        L2LocalVerdict::Tied { terminal_ids } => {
            let cohort_surfaces = terminal_ids
                .into_iter()
                .filter_map(|terminal_id| surfaces_by_terminal.get(&terminal_id).cloned())
                .collect::<Vec<_>>();
            (!cohort_surfaces.is_empty()).then_some(ShadowCohortReadout::Tied { cohort_surfaces })
        }
        L2LocalVerdict::Abstain => Some(ShadowCohortReadout::Abstain {
            cohort_surfaces: surfaces_by_terminal.into_values().collect(),
        }),
    }
}

fn l2_surface_unified_candidates(
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
            let gate = short_surface_gate_guard(token, error_class, gate);
            Some(UnifiedCorrectionCandidate::new(
                replacement,
                CorrectionDecisionSource::Nanda,
                origin,
                L2FieldBridgeKind::Shadow.surface_source_id(),
                error_class,
                gate,
            ))
        })
        .collect()
}

fn short_surface_gate_guard(
    token: &str,
    error_class: TypingErrorClass,
    gate: crate::correction_core::CandidateGateDecision,
) -> crate::correction_core::CandidateGateDecision {
    if token.chars().count() <= 3
        && error_class == TypingErrorClass::SparseInternalMultiOmission
        && gate.action == crate::correction_core::CandidateGateAction::Eligible
    {
        return crate::correction_core::CandidateGateDecision {
            action: crate::correction_core::CandidateGateAction::SuggestOnly,
            reason: "short_sparse_multi_omission_requires_tie_or_context",
        };
    }
    gate
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ShadowCohortReadout {
    Winner {
        winner_surface: String,
        cohort_surfaces: Vec<String>,
    },
    Tied {
        cohort_surfaces: Vec<String>,
    },
    Abstain {
        cohort_surfaces: Vec<String>,
    },
}

impl ShadowCohortReadout {
    fn winner_surface(&self) -> Option<&str> {
        match self {
            Self::Winner { winner_surface, .. } => Some(winner_surface.as_str()),
            Self::Tied { .. } | Self::Abstain { .. } => None,
        }
    }
}

#[cfg(test)]
fn apply_shadow_same_lemma_morphology(
    context_prefix: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
) -> Option<ShadowCohortReadout> {
    let candidate_surfaces = surface_candidates
        .iter()
        .map(|candidate| candidate.surface.to_lowercase())
        .collect::<Vec<_>>();
    let readout = crate::nanda_wave::morphology_phase::shadow_same_lemma_surface_readout(
        context_prefix,
        &candidate_surfaces,
    )?;
    match readout {
        crate::nanda_wave::morphology_phase::SameLemmaSurfaceReadout::Winner {
            winner_surface,
            cohort_surfaces,
        } => {
            let cohort = cohort_surfaces
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>();
            surface_candidates.retain(|candidate| {
                let surface = candidate.surface.to_lowercase();
                !cohort.contains(&surface) || surface == winner_surface
            });
            Some(ShadowCohortReadout::Winner {
                winner_surface,
                cohort_surfaces,
            })
        }
        crate::nanda_wave::morphology_phase::SameLemmaSurfaceReadout::Tied { cohort_surfaces } => {
            Some(ShadowCohortReadout::Tied { cohort_surfaces })
        }
        crate::nanda_wave::morphology_phase::SameLemmaSurfaceReadout::Abstain {
            cohort_surfaces,
        } => Some(ShadowCohortReadout::Abstain { cohort_surfaces }),
    }
}

#[cfg(test)]
fn promote_shadow_same_lemma_morphology(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &ShadowCohortReadout,
) {
    let Some(winner_surface) = readout.winner_surface() else {
        return;
    };
    for candidate in candidates {
        let Some((_, word)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)
        else {
            continue;
        };
        let (_, word, _) = crate::word_reader::split_word_punctuation(word);
        if word.to_lowercase() != winner_surface {
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

#[cfg(test)]
fn apply_shadow_near_neighbor_lexical(
    token: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
) -> Option<ShadowCohortReadout> {
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
            let input_gap = crate::text_metrics::damerau_levenshtein(&normalized_token, &surface);
            ((surface == leader_surface)
                || ((1..=2).contains(&leader_gap)
                    && surface
                        .chars()
                        .count()
                        .abs_diff(leader_surface.chars().count())
                        <= 2
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
    let runner_up = ranked.get(1)?;
    let margin = winner.1.saturating_sub(runner_up.1);
    let cohort = cohort_indices
        .iter()
        .filter_map(|index| surface_candidates.get(*index))
        .map(|candidate| candidate.surface.to_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    let cohort_surfaces = cohort.into_iter().collect::<Vec<_>>();
    if normalized_token.chars().count() <= 3 {
        return Some(ShadowCohortReadout::Tied { cohort_surfaces });
    }
    if winner.0 != leader_surface {
        return Some(ShadowCohortReadout::Tied { cohort_surfaces });
    }
    if winner.1 < 1_050 || winner.2 > leader_distance.saturating_add(1) {
        return Some(ShadowCohortReadout::Abstain { cohort_surfaces });
    }
    if margin < 220 || winner.2 > runner_up.2 || winner.3 < runner_up.3 {
        return Some(ShadowCohortReadout::Tied { cohort_surfaces });
    }
    surface_candidates.retain(|candidate| {
        let surface = candidate.surface.to_lowercase();
        !cohort_surfaces.iter().any(|value| value == &surface) || surface == winner.0
    });
    Some(ShadowCohortReadout::Winner {
        winner_surface: winner.0.clone(),
        cohort_surfaces,
    })
}

#[cfg(test)]
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

#[cfg(test)]
fn promote_shadow_near_neighbor_lexical(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &ShadowCohortReadout,
) {
    let Some(winner_surface) = readout.winner_surface() else {
        return;
    };
    for candidate in candidates {
        let Some((_, word)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)
        else {
            continue;
        };
        let (_, word, _) = crate::word_reader::split_word_punctuation(word);
        if word.to_lowercase() != winner_surface {
            continue;
        }
        let source_id = L2FieldBridgeKind::Shadow
            .near_neighbor_source_id()
            .to_string();
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

#[cfg(test)]
fn apply_shadow_local_readout(
    token: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    same_lemma: Option<&ShadowCohortReadout>,
    near_neighbor: Option<&ShadowCohortReadout>,
) -> Option<ShadowCohortReadout> {
    if surface_candidates.is_empty() {
        return None;
    }
    let normalized_token = token.to_lowercase();
    let mut ranked = surface_candidates
        .iter()
        .map(|candidate| {
            let surface = candidate.surface.to_lowercase();
            let score = shadow_local_readout_strength(
                &normalized_token,
                candidate,
                same_lemma,
                near_neighbor,
            );
            let distance = crate::text_metrics::damerau_levenshtein(&normalized_token, &surface);
            (
                surface,
                score,
                distance,
                candidate.kind,
                candidate.motif_overlap,
                candidate.l2_overlap,
                candidate.l1_overlap,
            )
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| {
                shadow_candidate_kind_rank(right.3).cmp(&shadow_candidate_kind_rank(left.3))
            })
            .then_with(|| right.4.cmp(&left.4))
            .then_with(|| right.5.cmp(&left.5))
            .then_with(|| right.6.cmp(&left.6))
            .then_with(|| left.0.cmp(&right.0))
    });
    let best = ranked.first()?;
    let cohort_surfaces = ranked
        .iter()
        .map(|candidate| candidate.0.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let support_floor = shadow_local_support_floor(&normalized_token, same_lemma, near_neighbor);
    if best.1 < support_floor {
        return Some(ShadowCohortReadout::Abstain { cohort_surfaces });
    }
    let tie_window = shadow_local_tie_window(same_lemma, near_neighbor);
    let tied_surfaces = ranked
        .iter()
        .take_while(|candidate| best.1.saturating_sub(candidate.1) < tie_window)
        .map(|candidate| candidate.0.clone())
        .collect::<Vec<_>>();
    if tied_surfaces.len() > 1 {
        return Some(ShadowCohortReadout::Tied {
            cohort_surfaces: tied_surfaces,
        });
    }
    if matches!(near_neighbor, Some(ShadowCohortReadout::Tied { .. }))
        && best.0.chars().count() == normalized_token.chars().count()
    {
        let competitive_missing_letter_surfaces = ranked
            .iter()
            .filter(|candidate| {
                candidate.0.chars().count() == normalized_token.chars().count() + 1
                    && candidate.2 <= best.2
                    && candidate.4 >= best.4
            })
            .map(|candidate| candidate.0.clone())
            .collect::<Vec<_>>();
        if !competitive_missing_letter_surfaces.is_empty() {
            let mut cohort_surfaces = vec![best.0.clone()];
            cohort_surfaces.extend(competitive_missing_letter_surfaces);
            cohort_surfaces.sort();
            cohort_surfaces.dedup();
            return Some(ShadowCohortReadout::Tied { cohort_surfaces });
        }
    }
    if let Some(cohort_surfaces) =
        shadow_local_growth_vs_same_length_conflict(&normalized_token, &ranked, tie_window)
    {
        return Some(ShadowCohortReadout::Tied { cohort_surfaces });
    }
    let donor_backed_winner = donor_winner_matches(best.0.as_str(), same_lemma)
        || compact_donor_winner_matches(best.0.as_str(), near_neighbor, 3);
    let transposition_backed_winner = matches!(
        best.3,
        crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition
    ) && best.2 <= 1;
    if !donor_backed_winner && !transposition_backed_winner {
        if let Some(cohort_surfaces) = shadow_local_competitive_cluster(&ranked, tie_window) {
            return Some(ShadowCohortReadout::Tied { cohort_surfaces });
        }
    }
    let self_backed_winner =
        shadow_local_self_backed_winner(token, &ranked, support_floor, tie_window);
    if !donor_backed_winner && !transposition_backed_winner && !self_backed_winner {
        return Some(ShadowCohortReadout::Abstain { cohort_surfaces });
    }
    surface_candidates.retain(|candidate| candidate.surface.to_lowercase() == best.0);
    Some(ShadowCohortReadout::Winner {
        winner_surface: best.0.clone(),
        cohort_surfaces,
    })
}

#[cfg(test)]
fn shadow_local_readout_strength(
    token: &str,
    candidate: &crate::nanda_wave::l2::L2ImeWordCandidate,
    same_lemma: Option<&ShadowCohortReadout>,
    near_neighbor: Option<&ShadowCohortReadout>,
) -> i64 {
    let surface = candidate.surface.to_lowercase();
    let mut score = shadow_near_neighbor_strength(token, candidate);
    score += donor_surface_bias(
        surface.as_str(),
        same_lemma,
        SAME_LEMMA_DONOR_TIED_BONUS * SHADOW_DONOR_WINNER_WEIGHT,
        SAME_LEMMA_DONOR_TIED_BONUS,
        SAME_LEMMA_DONOR_ABSTAIN_BONUS,
    );
    score += donor_surface_bias(
        surface.as_str(),
        near_neighbor,
        NEAR_NEIGHBOR_DONOR_TIED_BONUS * SHADOW_DONOR_WINNER_WEIGHT,
        NEAR_NEIGHBOR_DONOR_TIED_BONUS,
        NEAR_NEIGHBOR_DONOR_ABSTAIN_BONUS,
    );
    score
}

#[cfg(test)]
fn donor_winner_matches(surface: &str, readout: Option<&ShadowCohortReadout>) -> bool {
    matches!(
        readout,
        Some(ShadowCohortReadout::Winner { winner_surface, .. }) if winner_surface == surface
    )
}

#[cfg(test)]
fn compact_donor_winner_matches(
    surface: &str,
    readout: Option<&ShadowCohortReadout>,
    max_cohort: usize,
) -> bool {
    matches!(
        readout,
        Some(ShadowCohortReadout::Winner {
            winner_surface,
            cohort_surfaces,
        }) if winner_surface == surface && cohort_surfaces.len() <= max_cohort
    )
}

#[cfg(test)]
fn donor_surface_bias(
    surface: &str,
    readout: Option<&ShadowCohortReadout>,
    winner_bonus: i64,
    tied_bonus: i64,
    abstain_bonus: i64,
) -> i64 {
    match readout {
        Some(ShadowCohortReadout::Winner {
            winner_surface,
            cohort_surfaces,
        }) if winner_surface == surface => winner_bonus,
        Some(ShadowCohortReadout::Winner {
            cohort_surfaces, ..
        }) if cohort_surfaces.iter().any(|value| value == surface) => tied_bonus,
        Some(ShadowCohortReadout::Tied { cohort_surfaces })
            if cohort_surfaces.iter().any(|value| value == surface) =>
        {
            tied_bonus
        }
        Some(ShadowCohortReadout::Abstain { cohort_surfaces })
            if cohort_surfaces.iter().any(|value| value == surface) =>
        {
            abstain_bonus
        }
        _ => 0,
    }
}

#[cfg(test)]
fn shadow_local_support_floor(
    token: &str,
    same_lemma: Option<&ShadowCohortReadout>,
    near_neighbor: Option<&ShadowCohortReadout>,
) -> i64 {
    let len = token.chars().count();
    let mut floor = if len <= 3 { 1_260 } else { 1_020 };
    if matches!(same_lemma, Some(ShadowCohortReadout::Tied { .. })) {
        floor += 120;
    }
    if matches!(near_neighbor, Some(ShadowCohortReadout::Tied { .. })) {
        floor += 80;
    }
    floor
}

#[cfg(test)]
fn shadow_local_tie_window(
    same_lemma: Option<&ShadowCohortReadout>,
    near_neighbor: Option<&ShadowCohortReadout>,
) -> i64 {
    let mut window = 220;
    if matches!(same_lemma, Some(ShadowCohortReadout::Tied { .. })) {
        window += 110;
    }
    if matches!(near_neighbor, Some(ShadowCohortReadout::Tied { .. })) {
        window += 70;
    }
    window
}

#[cfg(test)]
fn shadow_candidate_kind_rank(kind: crate::nanda_wave::l2::L2ImeWordCandidateKind) -> u8 {
    match kind {
        crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition => 3,
        crate::nanda_wave::l2::L2ImeWordCandidateKind::Replacement => 2,
        crate::nanda_wave::l2::L2ImeWordCandidateKind::Completion => 1,
    }
}

#[cfg(test)]
fn shadow_local_self_backed_winner(
    token: &str,
    ranked: &[(
        String,
        i64,
        usize,
        crate::nanda_wave::l2::L2ImeWordCandidateKind,
        usize,
        usize,
        usize,
    )],
    support_floor: i64,
    tie_window: i64,
) -> bool {
    let Some(best) = ranked.first() else {
        return false;
    };
    let token_len = token.chars().count();
    let best_len = best.0.chars().count();
    if token_len <= 3
        || best.3 == crate::nanda_wave::l2::L2ImeWordCandidateKind::Completion
        || best.1 < support_floor.saturating_add(180)
        || best.4 == 0
        || best.5 == 0
    {
        return false;
    }
    let max_distance = if token_len >= 7 { 2 } else { 1 };
    if best.2 > max_distance {
        return false;
    }
    let Some(runner_up) = ranked.get(1) else {
        return true;
    };
    let margin = best.1.saturating_sub(runner_up.1);
    if margin < tie_window.saturating_add(160) {
        return false;
    }
    let competing_single_missing_letter = best_len == token_len
        && ranked.iter().skip(1).any(|candidate| {
            candidate.3 != crate::nanda_wave::l2::L2ImeWordCandidateKind::Completion
                && candidate.0.chars().count() == token_len + 1
                && candidate.2 == 1
                && best.1.saturating_sub(candidate.1) < tie_window.saturating_add(260)
                && candidate.4 > 0
                && candidate.5 > 0
        });
    if competing_single_missing_letter {
        return false;
    }
    best.2 < runner_up.2
        || (best.2 == runner_up.2
            && best.4 > runner_up.4
            && (best.5 > runner_up.5 || best.6 > runner_up.6))
}

#[cfg(test)]
fn shadow_local_competitive_cluster(
    ranked: &[(
        String,
        i64,
        usize,
        crate::nanda_wave::l2::L2ImeWordCandidateKind,
        usize,
        usize,
        usize,
    )],
    tie_window: i64,
) -> Option<Vec<String>> {
    let best = ranked.first()?;
    let competitive = ranked
        .iter()
        .take_while(|candidate| best.1.saturating_sub(candidate.1) < tie_window.saturating_add(260))
        .filter(|candidate| {
            candidate.3 != crate::nanda_wave::l2::L2ImeWordCandidateKind::Completion
                && candidate.2 <= best.2.saturating_add(1)
                && candidate.4 > 0
                && candidate.5 > 0
        })
        .collect::<Vec<_>>();
    if competitive.len() < 3 {
        return None;
    }
    let same_distance = competitive
        .iter()
        .filter(|candidate| candidate.2 == best.2)
        .count();
    let same_length = competitive
        .iter()
        .filter(|candidate| candidate.0.chars().count() == best.0.chars().count())
        .count();
    let mixed_lengths = competitive
        .iter()
        .any(|candidate| candidate.0.chars().count() != best.0.chars().count());
    if same_distance < 2 && !(same_length >= 2 && mixed_lengths) {
        return None;
    }
    let mut cohort_surfaces = competitive
        .iter()
        .map(|candidate| candidate.0.clone())
        .collect::<Vec<_>>();
    cohort_surfaces.sort();
    cohort_surfaces.dedup();
    Some(cohort_surfaces)
}

#[cfg(test)]
fn shadow_local_growth_vs_same_length_conflict(
    token: &str,
    ranked: &[(
        String,
        i64,
        usize,
        crate::nanda_wave::l2::L2ImeWordCandidateKind,
        usize,
        usize,
        usize,
    )],
    tie_window: i64,
) -> Option<Vec<String>> {
    let best = ranked.first()?;
    let token_len = token.chars().count();
    if token_len > 4 {
        return None;
    }
    if best.0.chars().count() != token_len + 1 {
        return None;
    }
    let growth_candidates = ranked
        .iter()
        .take_while(|candidate| best.1.saturating_sub(candidate.1) < tie_window.saturating_add(260))
        .filter(|candidate| {
            candidate.0.chars().count() == token_len + 1 && candidate.2 <= best.2.saturating_add(1)
        })
        .collect::<Vec<_>>();
    if growth_candidates.len() < 3 {
        return None;
    }
    let same_length_competitor = ranked.iter().find(|candidate| {
        candidate.0.chars().count() == token_len
            && candidate.2 <= best.2
            && best.1.saturating_sub(candidate.1) < tie_window.saturating_add(320)
    })?;
    if same_length_competitor.2 >= best.2 && growth_candidates.len() < 4 {
        return None;
    }
    let mut cohort_surfaces = growth_candidates
        .iter()
        .map(|candidate| candidate.0.clone())
        .collect::<Vec<_>>();
    cohort_surfaces.push(same_length_competitor.0.clone());
    cohort_surfaces.sort();
    cohort_surfaces.dedup();
    Some(cohort_surfaces)
}

fn promote_shadow_local_readout(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &ShadowCohortReadout,
) {
    let Some(winner_surface) = readout.winner_surface() else {
        return;
    };
    for candidate in candidates {
        let Some((_, word)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)
        else {
            continue;
        };
        let (_, word, _) = crate::word_reader::split_word_punctuation(word);
        if word.to_lowercase() != winner_surface
            || candidate.gate.action != crate::correction_core::CandidateGateAction::Eligible
        {
            continue;
        }
        let source_id = L2FieldBridgeKind::Shadow.readout_source_id().to_string();
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

fn demote_shadow_local_surface_cohort(
    candidates: &mut [UnifiedCorrectionCandidate],
    token: &str,
    readout: &ShadowCohortReadout,
) {
    let (cohort_surfaces, reason) = match readout {
        ShadowCohortReadout::Winner { .. } => return,
        ShadowCohortReadout::Tied { cohort_surfaces } => {
            (cohort_surfaces.as_slice(), "l2_field_shadow_local_tie")
        }
        ShadowCohortReadout::Abstain { cohort_surfaces } => {
            (cohort_surfaces.as_slice(), "l2_field_shadow_local_abstain")
        }
    };
    let token_len = token.chars().count();
    let short_growth_cohort = token_len <= 4
        && cohort_surfaces
            .iter()
            .filter(|surface| surface.chars().count() == token_len + 1)
            .count()
            >= 3;
    for candidate in candidates {
        if candidate.source_id != L2FieldBridgeKind::Shadow.surface_source_id() {
            continue;
        }
        let Some((_, word)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)
        else {
            continue;
        };
        let (_, word, _) = crate::word_reader::split_word_punctuation(word);
        let word_lower = word.to_lowercase();
        let candidate_len = word_lower.chars().count();
        let in_growth_tail = short_growth_cohort
            && matches!(
                candidate.error_class,
                TypingErrorClass::MissingLetter | TypingErrorClass::AdjacentTransposition
            )
            && (candidate_len == token_len || candidate_len == token_len + 1);
        if !in_growth_tail && !cohort_surfaces.iter().any(|surface| surface == &word_lower) {
            continue;
        }
        if candidate.gate.action == crate::correction_core::CandidateGateAction::Eligible {
            candidate.gate = crate::correction_core::CandidateGateDecision {
                action: crate::correction_core::CandidateGateAction::SuggestOnly,
                reason,
            };
        }
        for evidence in &mut candidate.evidence {
            if evidence.source_id == L2FieldBridgeKind::Shadow.surface_source_id()
                && evidence.gate.action == crate::correction_core::CandidateGateAction::Eligible
            {
                evidence.gate = crate::correction_core::CandidateGateDecision {
                    action: crate::correction_core::CandidateGateAction::SuggestOnly,
                    reason,
                };
            }
        }
    }
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

    fn lexical_candidate_with_kind(
        surface: &str,
        score: u32,
        l1_overlap: usize,
        l2_overlap: usize,
        motif_overlap: usize,
        kind: crate::nanda_wave::l2::L2ImeWordCandidateKind,
    ) -> crate::nanda_wave::l2::L2ImeWordCandidate {
        let mut candidate =
            lexical_candidate(surface, score, l1_overlap, l2_overlap, motif_overlap);
        candidate.kind = kind;
        candidate
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
            Some(ShadowCohortReadout::Winner {
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

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Tied {
                cohort_surfaces: vec!["посмотри".to_string(), "просмотри".to_string()],
            })
        );
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn shadow_near_neighbor_lexical_does_not_reselect_non_leader() {
        let mut candidates = vec![
            lexical_candidate("всю", 1220, 5, 3, 2),
            lexical_candidate("васю", 1540, 8, 6, 4),
        ];

        let readout = apply_shadow_near_neighbor_lexical("всю", &mut candidates);

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Tied {
                cohort_surfaces: vec!["васю".to_string(), "всю".to_string()],
            })
        );
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn shadow_local_readout_filters_clear_winner_with_general_donor_backing() {
        let mut candidates = vec![
            lexical_candidate("васю", 1540, 8, 6, 4),
            lexical_candidate("всю", 1220, 5, 3, 2),
        ];
        let near_neighbor = ShadowCohortReadout::Winner {
            winner_surface: "васю".to_string(),
            cohort_surfaces: vec!["васю".to_string(), "всю".to_string()],
        };

        let readout =
            apply_shadow_local_readout("всю", &mut candidates, None, Some(&near_neighbor));

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Winner {
                winner_surface: "васю".to_string(),
                cohort_surfaces: vec!["васю".to_string(), "всю".to_string()],
            })
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].surface, "васю");
    }

    #[test]
    fn shadow_local_readout_preserves_tied_local_field() {
        let mut candidates = vec![
            lexical_candidate("васю", 1360, 6, 4, 3),
            lexical_candidate("всю", 1348, 6, 4, 3),
        ];

        let readout = apply_shadow_local_readout("всю", &mut candidates, None, None);

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Tied {
                cohort_surfaces: vec!["всю".to_string(), "васю".to_string()],
            })
        );
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn shadow_local_readout_ties_dense_missing_letter_cluster_without_donor() {
        let mut candidates = vec![
            lexical_candidate_with_kind(
                "соли",
                1500,
                7,
                5,
                3,
                crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition,
            ),
            lexical_candidate("слови", 1490, 7, 5, 3),
            lexical_candidate("слоги", 1482, 7, 5, 3),
            lexical_candidate("сложи", 1476, 7, 5, 3),
        ];

        let readout = apply_shadow_local_readout("слои", &mut candidates, None, None);

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Tied {
                cohort_surfaces: vec![
                    "соли".to_string(),
                    "слови".to_string(),
                    "слоги".to_string(),
                    "сложи".to_string(),
                ],
            })
        );
        assert_eq!(candidates.len(), 4);
    }

    #[test]
    fn shadow_local_readout_ties_dense_long_form_cluster_without_donor() {
        let mut candidates = vec![
            lexical_candidate("докуривать", 2120, 14, 10, 8),
            lexical_candidate("докручивать", 2106, 14, 10, 8),
            lexical_candidate("докручивал", 2094, 14, 10, 8),
            lexical_candidate("докручивает", 2088, 14, 10, 8),
        ];

        let readout = apply_shadow_local_readout("докурчиват", &mut candidates, None, None);

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Tied {
                cohort_surfaces: vec![
                    "докуривать".to_string(),
                    "докручивать".to_string(),
                    "докручивал".to_string(),
                    "докручивает".to_string(),
                ],
            })
        );
        assert_eq!(candidates.len(), 4);
    }

    #[test]
    fn shadow_local_readout_ties_growth_cluster_against_same_length_competitor() {
        let mut candidates = vec![
            lexical_candidate("слови", 1540, 7, 5, 3),
            lexical_candidate("слоги", 1532, 7, 5, 3),
            lexical_candidate("сложи", 1526, 7, 5, 3),
            lexical_candidate_with_kind(
                "соли",
                1518,
                7,
                5,
                3,
                crate::nanda_wave::l2::L2ImeWordCandidateKind::AdjacentTransposition,
            ),
        ];

        let readout = apply_shadow_local_readout("слои", &mut candidates, None, None);

        assert_eq!(
            readout,
            Some(ShadowCohortReadout::Tied {
                cohort_surfaces: vec![
                    "соли".to_string(),
                    "слови".to_string(),
                    "слоги".to_string(),
                    "сложи".to_string(),
                ],
            })
        );
        assert_eq!(candidates.len(), 4);
    }
}
