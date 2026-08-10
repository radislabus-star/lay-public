use crate::candidate_contract::CandidateOrigin;
use crate::candidate_contract::CorrectionSourceRole;
use crate::correction_core::{
    CandidateEvidence, CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource,
    TypingErrorClass, UnifiedCorrectionCandidate,
};
use crate::text_case::apply_word_case;
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::{replace_last_text_word, split_last_alphabetic_token};
use rayon::prelude::*;

use super::runtime::{
    CanonicalL2FieldReadout, CompositionalFormBirth, CompositionalLemmaBirth, L2FieldAuthority,
    L2LexicalSeed, L2LexicalSeedOrigin, L2LocalVerdict, StandaloneL2Field,
    CANONICAL_L2_READOUT_SOURCE_ID, CANONICAL_L2_SURFACE_SOURCE_ID,
};

pub(crate) fn canonical_text_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    canonical_text_readout(original).candidates
}

pub(crate) fn canonical_text_readout(original: &str) -> CanonicalL2FieldReadout {
    if let Some(readout) = super::cache::get(original) {
        return readout;
    }
    let (mut readout, boundary_candidates) = rayon::join(
        || canonical_owned_text_candidates(original),
        || boundary_text_candidates(original),
    );
    for candidate in boundary_candidates
        .into_iter()
        .chain(short_layout_candidates(original))
    {
        if let Some(existing) = readout
            .candidates
            .iter_mut()
            .find(|existing| existing.replacement == candidate.replacement)
        {
            existing.merge_evidence(candidate);
        } else {
            readout.candidates.push(candidate);
        }
    }
    super::cache::store(original, &readout);
    readout
}

fn boundary_text_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    const MAX_BOUNDARY_CANDIDATES: usize = 2;
    const CANONICAL_L2_BOUNDARY_SOURCE_ID: &str = "CanonicalL2FieldBoundary";

    let Some((context_prefix, token)) = split_last_alphabetic_token(original) else {
        return Vec::new();
    };
    crate::nanda_wave::l2::ime_l2_boundary_candidates(
        context_prefix,
        token,
        MAX_BOUNDARY_CANDIDATES,
    )
    .into_iter()
    .filter_map(|candidate| {
        let replacement = replace_last_text_word(original, &candidate.surface)?;
        let origin = CandidateOrigin::Boundary;
        let error_class = action_operator::classify_token_transition(
            original,
            &replacement,
            origin,
            TypingErrorClass::GluedWords,
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
            CANONICAL_L2_BOUNDARY_SOURCE_ID,
            error_class,
            gate,
        ))
    })
    .collect()
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
    surfaces.extend(
        short_layout_candidates(&original)
            .into_iter()
            .filter_map(|candidate| {
                split_last_alphabetic_token(&candidate.replacement)
                    .map(|(_, token)| token.to_lowercase())
            }),
    );
    surfaces.sort();
    surfaces.dedup();
    surfaces
}

fn short_layout_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    let Some((_, token)) = split_last_alphabetic_token(original) else {
        return Vec::new();
    };
    if token.chars().count() != 1 || original.split_whitespace().count() < 2 {
        return Vec::new();
    }
    crate::nanda_wave::l2::hot_short_layout_candidates(original)
        .into_iter()
        .filter_map(|candidate| {
            let (_, projected_token) = split_last_alphabetic_token(&candidate.text)?;
            let replacement = replace_last_text_word(original, projected_token)?;
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
                CANONICAL_L2_SURFACE_SOURCE_ID,
                error_class,
                gate,
            ))
        })
        .collect()
}

fn canonical_owned_text_candidates(original: &str) -> CanonicalL2FieldReadout {
    let started = std::time::Instant::now();
    // L2 owns its Winner/Tied/Abstain verdict, but L3 must observe the whole
    // bounded cohort. Non-winners remain SuggestOnly below; retaining them is
    // evidence visibility, not lexical apply authority.
    let Some(field) = standalone_surface_field_readout(original, false) else {
        return CanonicalL2FieldReadout::default();
    };
    let materialize_started = std::time::Instant::now();
    let mut candidates =
        l2_surface_unified_candidates(original, &field.token, &field.surface_candidates);
    let unified_ready = std::time::Instant::now();
    promote_canonical_local_readout(&mut candidates, &field.local_readout);
    demote_canonical_local_surface_cohort(&mut candidates, &field.local_readout);
    let authority = field.local_readout.authority();
    apply_authority_to_candidate_lattice(&mut candidates, &authority);
    let authority_ready = std::time::Instant::now();
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        let finished = std::time::Instant::now();
        eprintln!(
            "l2_field_trace seeds_us={} field_us={} materialize_us={} authority_us={} total_us={} seeds={} surfaces={} candidates={}",
            field.seed_duration.as_micros(),
            field.field_duration.as_micros(),
            unified_ready.duration_since(materialize_started).as_micros(),
            authority_ready.duration_since(unified_ready).as_micros(),
            finished.duration_since(started).as_micros(),
            field.seed_count,
            field.surface_count,
            candidates.len(),
        );
    }

    CanonicalL2FieldReadout::new(candidates, authority)
}

struct StandaloneSurfaceFieldReadout {
    token: String,
    surface_candidates: Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    local_readout: CanonicalCohortReadout,
    seed_count: usize,
    surface_count: usize,
    seed_duration: std::time::Duration,
    field_duration: std::time::Duration,
}

struct PreparedCompositionalField {
    broad_lemma_births: Vec<CompositionalLemmaBirth>,
    active_lemma_births: Vec<CompositionalLemmaBirth>,
    form_births: Vec<CompositionalFormBirth>,
    broad_duration: std::time::Duration,
    active_duration: std::time::Duration,
    form_duration: std::time::Duration,
    total_duration: std::time::Duration,
}

fn standalone_surface_field_readout(
    original: &str,
    settle_winner: bool,
) -> Option<StandaloneSurfaceFieldReadout> {
    const HOT_L2_CANDIDATE_LIMIT: usize = 8;
    const SPARSE_OMISSION_RESERVE: usize = 2;
    const SHADOW_SURFACE_MATERIAL_LIMIT: usize = 16;

    let (context_prefix, token) = split_last_alphabetic_token(original)?;
    let normalized_token = token.to_lowercase();
    if normalized_token.chars().count() < 2
        || !normalized_token
            .chars()
            .all(crate::keyboard::is_cyrillic_letter)
    {
        return None;
    }
    let field = super::installed_l2_field().ok()?;
    let context = format!("{} _", context_prefix.trim());
    let ((surface_candidates, l11_seeds), seed_duration, prepared) = std::thread::scope(|scope| {
        let prepared = scope.spawn(|| prepare_compositional_field(field, &context, token));
        let seed_started = std::time::Instant::now();
        let seeds = l11_surface_seed_candidates(token, SHADOW_SURFACE_MATERIAL_LIMIT);
        let seed_duration = seed_started.elapsed();
        Some((seeds, seed_duration, prepared.join().ok()?))
    })?;
    let seed_count = l11_seeds.len();
    let surface_count = surface_candidates.len();
    let mut bounded_surface_candidates = surface_candidates
        .iter()
        .take(HOT_L2_CANDIDATE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    for candidate in surface_candidates
        .iter()
        .filter(|candidate| {
            crate::text_metrics::sparse_internal_omission_count(
                &normalized_token,
                &candidate.surface,
            )
            .is_some()
        })
        .take(SPARSE_OMISSION_RESERVE)
    {
        if !bounded_surface_candidates
            .iter()
            .any(|existing| existing.surface.eq_ignore_ascii_case(&candidate.surface))
        {
            bounded_surface_candidates.push(candidate.clone());
        }
    }
    let l11_geometry_candidates = bounded_surface_candidates.clone();
    for candidate in reference_backed_missing_letter_candidates(&normalized_token, 2) {
        if !bounded_surface_candidates
            .iter()
            .any(|existing| existing.surface.eq_ignore_ascii_case(&candidate.surface))
        {
            bounded_surface_candidates.push(candidate);
        }
    }
    let prepared_duration = prepared.total_duration;
    let apply_started = std::time::Instant::now();
    let local_readout = apply_standalone_l2_field(
        field,
        &context,
        token,
        &mut bounded_surface_candidates,
        &l11_geometry_candidates,
        &l11_seeds,
        prepared,
        settle_winner,
    )?;
    let field_duration = prepared_duration.saturating_add(apply_started.elapsed());
    let local_readout = retain_reference_backed_geometry_ambiguity(
        &normalized_token,
        local_readout,
        &bounded_surface_candidates,
    );
    Some(StandaloneSurfaceFieldReadout {
        token: token.to_string(),
        surface_candidates: bounded_surface_candidates,
        local_readout,
        seed_count,
        surface_count,
        seed_duration,
        field_duration,
    })
}

fn prepare_compositional_field(
    field: &StandaloneL2Field,
    context: &str,
    token: &str,
) -> PreparedCompositionalField {
    let started = std::time::Instant::now();
    let broad_lemma_births =
        field.compositional_lemma_births(token, super::CANONICAL_L2_LEMMA_FRONTIER);
    let broad_ready = std::time::Instant::now();
    let active_lemma_births = field.contextual_compositional_lemma_births(
        context,
        &broad_lemma_births,
        super::CANONICAL_L2_ACTIVE_LEMMA_LIMIT,
    );
    let active_ready = std::time::Instant::now();
    let form_births = field.contextual_compositional_form_births_from_lemmas(
        context,
        token,
        &active_lemma_births,
        super::CANONICAL_L2_FEATURE_LIMIT,
        super::CANONICAL_L2_FORM_LIMIT,
    );
    let form_ready = std::time::Instant::now();
    PreparedCompositionalField {
        broad_lemma_births,
        active_lemma_births,
        form_births,
        broad_duration: broad_ready.duration_since(started),
        active_duration: active_ready.duration_since(broad_ready),
        form_duration: form_ready.duration_since(active_ready),
        total_duration: form_ready.duration_since(started),
    }
}

fn reference_backed_missing_letter_candidates(
    token: &str,
    limit: usize,
) -> Vec<crate::nanda_wave::l2::L2ImeWordCandidate> {
    crate::ru_typo::safe_missing_letter_candidates(token)
        .filter(|candidate| {
            crate::russian_lexicon::is_reference_backed_short_passive_participle(candidate)
        })
        .take(limit)
        .map(|surface| {
            let seed = crate::nanda_wave::L11SeedSurface {
                terminal_id: None,
                surface,
                authority: false,
                score_milli: 0,
            };
            l11_seed_only_candidate(token, &seed)
        })
        .collect()
}

fn retain_reference_backed_geometry_ambiguity(
    token: &str,
    readout: CanonicalCohortReadout,
    candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
) -> CanonicalCohortReadout {
    let CanonicalCohortReadout::Winner {
        winner_surface,
        mut cohort_surfaces,
    } = readout
    else {
        return readout;
    };
    let mut unresolved = candidates
        .iter()
        .map(|candidate| candidate.surface.to_lowercase())
        .filter(|surface| !surface.eq_ignore_ascii_case(&winner_surface))
        .filter(|surface| crate::text_metrics::damerau_levenshtein(token, surface) == 1)
        .filter(|surface| {
            crate::russian_lexicon::is_reference_backed_short_passive_participle(surface)
        })
        .collect::<Vec<_>>();
    if unresolved.is_empty() {
        return CanonicalCohortReadout::Winner {
            winner_surface,
            cohort_surfaces,
        };
    }
    cohort_surfaces.push(winner_surface);
    cohort_surfaces.append(&mut unresolved);
    cohort_surfaces.sort();
    cohort_surfaces.dedup();
    CanonicalCohortReadout::Tied { cohort_surfaces }
}

fn l11_surface_seed_candidates(
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
            .unwrap_or(24)
            .clamp(1, 250),
    )
}

fn apply_standalone_l2_field(
    field: &StandaloneL2Field,
    context: &str,
    token: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    l11_geometry_candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
    seeds: &[crate::nanda_wave::L11SeedSurface],
    prepared: PreparedCompositionalField,
    settle_winner: bool,
) -> Option<CanonicalCohortReadout> {
    let input_surface_reserve = surface_candidates
        .iter()
        .map(|candidate| candidate.surface.to_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    let mut lexical_seeds = seeds
        .iter()
        .filter_map(|seed| {
            Some(L2LexicalSeed {
                terminal_id: seed.terminal_id,
                surface: Some(seed.surface.to_lowercase()),
                evidence_milli: i32::try_from(seed.score_milli.min(i32::MAX as u32)).ok()?,
                origin: L2LexicalSeedOrigin::GroundedL11,
            })
        })
        .collect::<Vec<_>>();
    let l11_peak = lexical_seeds.iter().map(|seed| seed.evidence_milli).max();
    let PreparedCompositionalField {
        broad_lemma_births,
        active_lemma_births,
        form_births: compositional_form_births,
        broad_duration,
        active_duration,
        form_duration,
        total_duration: _,
    } = prepared;

    // The wave lane is allowed to recover an exact paradigm surface that has
    // no L1 terminal. Its evidence remains relative to the observed L1 peak;
    // without any L1 seed it can populate the lattice but cannot own authority.
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_field_birth_trace token={token:?} l11_seeds={:?} broad_lemmas={} active_lemmas={} active_lemma_head={:?} form_births={:?}",
            lexical_seeds
                .iter()
                .map(|seed| (
                    seed.terminal_id,
                    seed.surface.as_deref(),
                    seed.evidence_milli,
                    seed.origin,
                ))
                .collect::<Vec<_>>(),
            broad_lemma_births.len(),
            active_lemma_births.len(),
            active_lemma_births
                .iter()
                .take(32)
                .map(|birth| (
                    birth.lemma_id,
                    birth.atom_evidence,
                    birth.atom_evidence_milli,
                    birth.wave_distance,
                ))
                .collect::<Vec<_>>(),
            compositional_form_births
                .iter()
                .map(|birth| (
                    birth.form_ref,
                    field
                        .decode_form_ref(birth.form_ref)
                        .map(std::borrow::Cow::into_owned),
                    birth.lemma_id,
                    birth.evidence_milli,
                    birth.geometry_evidence_milli,
                    birth.atom_evidence_milli,
                    birth.lemma_evidence_milli,
                    birth.wave_distance,
                ))
                .collect::<Vec<_>>(),
        );
    }
    let apply_started = std::time::Instant::now();
    for birth in compositional_form_births {
        let Some(surface) = field
            .decode_form_ref(birth.form_ref)
            .map(std::borrow::Cow::into_owned)
        else {
            continue;
        };
        if lexical_seeds.iter().any(|seed| {
            seed.surface
                .as_deref()
                .is_some_and(|seed_surface| seed_surface.eq_ignore_ascii_case(&surface))
        }) {
            continue;
        }
        let evidence_milli = compositional_evidence_milli(l11_peak, birth.evidence_milli);
        lexical_seeds.push(L2LexicalSeed {
            terminal_id: field.l1_terminal_for_form_ref(birth.form_ref),
            surface: Some(surface.clone()),
            evidence_milli,
            origin: L2LexicalSeedOrigin::CompositionalMorphology,
        });
        if !surface_candidates
            .iter()
            .any(|candidate| candidate.surface.eq_ignore_ascii_case(&surface))
        {
            surface_candidates.push(crate::nanda_wave::l2::L2ImeWordCandidate {
                surface: surface.clone(),
                kind: seeded_candidate_kind(token, &surface),
                source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
                score: evidence_milli.max(0) as u32,
                l1_overlap: seed_surface_overlap(token, &surface),
                l2_overlap: 2,
                motif_overlap: seed_surface_motif_overlap(token, &surface),
                usage_prior: 0.0,
                context_prior: 0.0,
                accepted_count: 0,
            });
        }
    }
    let form_seeds_duration = std::time::Instant::now().duration_since(apply_started);
    // Inverse geometry may birth an exact package form, but it is not an
    // independent L1.1 observation. Runtime caps this raw proposal at the
    // inherited L1 floor before context and competition are evaluated.
    let inverse_started = std::time::Instant::now();
    let inverse_proposal_milli = l11_peak.unwrap_or_default();
    for form_ref in l11_peak
        .into_iter()
        .flat_map(|_| field.single_edit_form_refs(token, 16))
    {
        let Some(surface) = field
            .decode_form_ref(form_ref)
            .map(std::borrow::Cow::into_owned)
        else {
            continue;
        };
        if lexical_seeds.iter().any(|seed| {
            seed.surface
                .as_deref()
                .is_some_and(|seed_surface| seed_surface.eq_ignore_ascii_case(&surface))
        }) {
            continue;
        }
        lexical_seeds.push(L2LexicalSeed {
            terminal_id: field.l1_terminal_for_form_ref(form_ref),
            surface: Some(surface.clone()),
            evidence_milli: inverse_proposal_milli,
            origin: L2LexicalSeedOrigin::InverseGeometry,
        });
        if !surface_candidates
            .iter()
            .any(|candidate| candidate.surface.eq_ignore_ascii_case(&surface))
        {
            surface_candidates.push(crate::nanda_wave::l2::L2ImeWordCandidate {
                surface: surface.clone(),
                kind: seeded_candidate_kind(token, &surface),
                source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
                score: inverse_proposal_milli.max(0) as u32,
                l1_overlap: seed_surface_overlap(token, &surface),
                l2_overlap: 1,
                motif_overlap: seed_surface_motif_overlap(token, &surface),
                usage_prior: 0.0,
                context_prior: 0.0,
                accepted_count: 0,
            });
        }
    }
    let inverse_ready = std::time::Instant::now();
    if lexical_seeds.is_empty() {
        return None;
    }
    let readout = field.readout_observed(
        &context,
        token,
        &lexical_seeds,
        super::CANONICAL_L2_FORM_LIMIT,
    );
    let readout_ready = std::time::Instant::now();
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_field_stage_trace broad_us={} active_us={} form_birth_us={} form_seed_us={} inverse_us={} readout_us={}",
            broad_duration.as_micros(),
            active_duration.as_micros(),
            form_duration.as_micros(),
            form_seeds_duration.as_micros(),
            inverse_ready.duration_since(inverse_started).as_micros(),
            readout_ready.duration_since(inverse_ready).as_micros(),
        );
        eprintln!(
            "l2_field_readout verdict={:?} candidates={:?}",
            readout.verdict,
            readout
                .candidates
                .iter()
                .map(|candidate| (
                    candidate.form_ref,
                    candidate.surface.as_str(),
                    candidate.l1_terminal_id,
                    candidate.local_score,
                    candidate.slot_phase_milli,
                    candidate.neighbor_pressure,
                    candidate.competition_pressure,
                ))
                .collect::<Vec<_>>(),
        );
    }
    let local_by_form = readout
        .candidates
        .iter()
        .map(|candidate| (candidate.form_ref, candidate))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut surfaces_by_form = std::collections::BTreeMap::new();
    for local in &readout.candidates {
        let surface = local.surface.to_lowercase();
        surfaces_by_form.insert(local.form_ref, surface.clone());
        if surface_candidates
            .iter()
            .any(|candidate| candidate.surface.eq_ignore_ascii_case(&surface))
        {
            continue;
        }
        let Some(local) = local_by_form.get(&local.form_ref) else {
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
    let mut retained_surfaces = input_surface_reserve;
    retained_surfaces.extend(surfaces_by_form.values().cloned());
    surface_candidates
        .retain(|candidate| retained_surfaces.contains(&candidate.surface.to_lowercase()));
    match readout.verdict {
        L2LocalVerdict::Winner { form_ref } => {
            let winner_surface = surfaces_by_form.get(&form_ref)?.clone();
            let cohort_surfaces = surfaces_by_form.into_values().collect::<Vec<_>>();
            if settle_winner {
                surface_candidates
                    .retain(|candidate| candidate.surface.eq_ignore_ascii_case(&winner_surface));
            }
            Some(CanonicalCohortReadout::Winner {
                winner_surface,
                cohort_surfaces,
            })
        }
        L2LocalVerdict::Tied { form_refs } => {
            if let Some(readout) = settle_unique_l1_geometry(
                token,
                l11_geometry_candidates,
                surface_candidates,
                settle_winner,
            ) {
                return Some(readout);
            }
            let cohort_surfaces = form_refs
                .into_iter()
                .filter_map(|form_ref| surfaces_by_form.get(&form_ref).cloned())
                .collect::<Vec<_>>();
            (!cohort_surfaces.is_empty())
                .then_some(CanonicalCohortReadout::Tied { cohort_surfaces })
        }
        L2LocalVerdict::Abstain => {
            if let Some(readout) = settle_unique_l1_geometry(
                token,
                l11_geometry_candidates,
                surface_candidates,
                settle_winner,
            ) {
                return Some(readout);
            }
            Some(CanonicalCohortReadout::Abstain {
                cohort_surfaces: surfaces_by_form.into_values().collect(),
            })
        }
    }
}

fn compositional_evidence_milli(l11_peak: Option<i32>, similarity_milli: u16) -> i32 {
    let evidence_basis = i64::from(l11_peak.unwrap_or(1_000).max(0));
    let scaled = evidence_basis.saturating_mul(i64::from(similarity_milli)) / 1_000;
    scaled.min(i64::from(i32::MAX)) as i32
}

fn settle_unique_l1_geometry(
    token: &str,
    lexical_candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    settle_winner: bool,
) -> Option<CanonicalCohortReadout> {
    let normalized_token = token.to_lowercase();
    let mut nearest = lexical_candidates
        .iter()
        .filter(|candidate| {
            crate::text_metrics::damerau_levenshtein(
                &normalized_token,
                &candidate.surface.to_lowercase(),
            ) == 1
        })
        .map(|candidate| candidate.surface.to_lowercase())
        .collect::<Vec<_>>();
    nearest.sort();
    nearest.dedup();
    if nearest.is_empty() {
        return None;
    }
    if nearest.len() == 1 {
        let winner_surface = nearest[0].clone();
        if settle_winner {
            surface_candidates
                .retain(|candidate| candidate.surface.eq_ignore_ascii_case(&winner_surface));
        }
        return Some(CanonicalCohortReadout::Winner {
            winner_surface,
            cohort_surfaces: nearest,
        });
    }
    Some(CanonicalCohortReadout::Tied {
        cohort_surfaces: nearest,
    })
}

fn l2_surface_unified_candidates(
    original: &str,
    token: &str,
    candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
) -> Vec<UnifiedCorrectionCandidate> {
    let normalized_token = token.to_lowercase();
    candidates
        .par_iter()
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
                CANONICAL_L2_SURFACE_SOURCE_ID,
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
enum CanonicalCohortReadout {
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

impl CanonicalCohortReadout {
    fn winner_surface(&self) -> Option<&str> {
        match self {
            Self::Winner { winner_surface, .. } => Some(winner_surface.as_str()),
            Self::Tied { .. } | Self::Abstain { .. } => None,
        }
    }

    fn authority(&self) -> L2FieldAuthority {
        match self {
            Self::Winner { winner_surface, .. } => L2FieldAuthority::Winner {
                surface: winner_surface.clone(),
            },
            Self::Tied { cohort_surfaces } => L2FieldAuthority::Tied {
                surfaces: cohort_surfaces.clone(),
            },
            Self::Abstain { .. } => L2FieldAuthority::Abstain,
        }
    }
}

fn promote_canonical_local_readout(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &CanonicalCohortReadout,
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
        let source_id = CANONICAL_L2_READOUT_SOURCE_ID.to_string();
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

fn demote_canonical_local_surface_cohort(
    candidates: &mut [UnifiedCorrectionCandidate],
    readout: &CanonicalCohortReadout,
) {
    let reason = match readout {
        CanonicalCohortReadout::Winner { .. } => return,
        CanonicalCohortReadout::Tied { .. } => "canonical_l2_field_local_tie",
        CanonicalCohortReadout::Abstain { .. } => "canonical_l2_field_local_abstain",
    };
    for candidate in candidates {
        if candidate.source_id != CANONICAL_L2_SURFACE_SOURCE_ID {
            continue;
        }
        if candidate.gate.action == CandidateGateAction::Eligible {
            candidate.gate = CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason,
            };
        }
        for evidence in &mut candidate.evidence {
            if evidence.source_id == CANONICAL_L2_SURFACE_SOURCE_ID
                && evidence.gate.action == CandidateGateAction::Eligible
            {
                evidence.gate = CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason,
                };
            }
        }
    }
}

pub(crate) fn apply_authority_to_candidate_lattice(
    candidates: &mut [UnifiedCorrectionCandidate],
    authority: &L2FieldAuthority,
) {
    let (winner_surface, tied_surfaces, reason) = match authority {
        L2FieldAuthority::Unavailable => (None, None, "l2_field_unavailable_requires_suggestion"),
        L2FieldAuthority::Winner { surface } => (
            Some(surface.as_str()),
            None,
            "l2_field_winner_owns_lexical_authority",
        ),
        L2FieldAuthority::Tied { surfaces } => (
            None,
            Some(surfaces.as_slice()),
            "l2_field_tie_requires_context",
        ),
        L2FieldAuthority::Abstain => (None, None, "l2_field_abstain_requires_context"),
    };

    for candidate in candidates {
        let role = candidate.origin.source_role();
        if !matches!(
            role,
            CorrectionSourceRole::DeterministicTypo | CorrectionSourceRole::L2Surface
        ) || has_independent_apply_evidence(candidate)
        {
            continue;
        }
        if winner_surface
            .is_some_and(|winner| candidate_last_word_lower(candidate).as_deref() == Some(winner))
        {
            continue;
        }
        if tied_surfaces.is_some_and(|surfaces| {
            candidate_last_word_lower(candidate).is_some_and(|word| {
                surfaces
                    .iter()
                    .any(|surface| surface.eq_ignore_ascii_case(&word))
            })
        }) {
            // A tied length-changing repair remains ambiguous even when a
            // deterministic producer independently found one cohort member.
            // Substitution/transposition evidence can retain its own
            // authority; insertion/deletion needs context to choose which
            // neighboring surface was intended.
            if !matches!(
                candidate.error_class,
                TypingErrorClass::MissingLetter | TypingErrorClass::ExtraLetter
            ) {
                continue;
            }
        }
        if candidate.gate.action == CandidateGateAction::Eligible {
            candidate.gate = CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason,
            };
        }
        for evidence in &mut candidate.evidence {
            if matches!(
                evidence.origin.source_role(),
                CorrectionSourceRole::DeterministicTypo | CorrectionSourceRole::L2Surface
            ) && evidence.gate.action == CandidateGateAction::Eligible
            {
                evidence.gate = CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason,
                };
            }
        }
    }
}

fn has_independent_apply_evidence(candidate: &UnifiedCorrectionCandidate) -> bool {
    candidate.evidence.iter().any(|evidence| {
        evidence.gate.action == CandidateGateAction::Eligible
            && !matches!(
                evidence.origin.source_role(),
                CorrectionSourceRole::DeterministicTypo | CorrectionSourceRole::L2Surface
            )
    })
}

fn candidate_last_word_lower(candidate: &UnifiedCorrectionCandidate) -> Option<String> {
    let (_, word) = crate::word_reader::split_last_trimmed_ws_token(&candidate.replacement)?;
    let (_, word, _) = crate::word_reader::split_word_punctuation(word);
    Some(word.to_lowercase())
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
    fn canonical_readout_reserves_a_verified_two_content_boundary_candidate() {
        let readout = canonical_text_readout("Еленапросит ");
        let candidate = readout
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "Елена просит ")
            .expect("bounded boundary reserve");

        assert_eq!(candidate.origin, CandidateOrigin::Boundary);
        assert_eq!(candidate.source_id, "CanonicalL2FieldBoundary");
        assert_eq!(candidate.error_class, TypingErrorClass::GluedWords);
    }

    #[test]
    fn canonical_readout_retains_sparse_omission_candidate_below_general_frontier() {
        let readout = canonical_text_readout("на компанию Хунлу можем подврдить ");
        let candidate = readout
            .candidates
            .iter()
            .find(|candidate| candidate.replacement.ends_with("подтвердить "))
            .expect("sparse multi-omission reserve must survive the general top-8 frontier");

        assert_eq!(
            candidate.error_class,
            TypingErrorClass::SparseInternalMultiOmission
        );
        assert_eq!(candidate.gate.action, CandidateGateAction::SuggestOnly);
    }

    #[test]
    fn composition_only_lattice_cannot_gain_authority() {
        assert_eq!(
            compositional_evidence_milli(Some(800), 750),
            600,
            "composition evidence must scale with the observed L1 peak"
        );
    }

    #[test]
    fn non_l11_births_cannot_enter_the_l1_geometry_fallback() {
        let l11 = vec![lexical_candidate("форма", 1_000, 4, 0, 2)];
        let mut materialized = vec![
            lexical_candidate("форма", 1_000, 4, 0, 2),
            lexical_candidate("сигнал", 900, 5, 2, 3),
        ];

        assert_eq!(
            settle_unique_l1_geometry("сигна", &l11, &mut materialized, false),
            None
        );
    }

    #[test]
    fn reference_backed_short_participle_blocks_false_singleton() {
        let readout = canonical_text_readout("подлючен ");
        let correct = readout
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "подключен ")
            .expect("reference-backed missing-letter candidate");
        let wrong = readout
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "подлечен ")
            .expect("competing one-edit candidate");

        assert_eq!(correct.gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(wrong.gate.action, CandidateGateAction::SuggestOnly);
        assert!(!matches!(
            readout.authority,
            L2FieldAuthority::Winner { .. }
        ));
    }

    fn unified_candidate(
        replacement: &str,
        origin: CandidateOrigin,
        source_id: &str,
    ) -> UnifiedCorrectionCandidate {
        UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Deterministic,
            origin,
            source_id,
            TypingErrorClass::CompositeTypo,
            CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "test",
            },
        )
    }

    #[test]
    fn l1_geometry_settles_one_single_edit_peak() {
        let lexical = vec![
            lexical_candidate("время", 1889, 5, 0, 3),
            lexical_candidate("змея", 1782, 3, 0, 2),
        ];
        let mut materialized = lexical.clone();

        let readout = settle_unique_l1_geometry("врмея", &lexical, &mut materialized, true);

        assert_eq!(
            readout,
            Some(CanonicalCohortReadout::Winner {
                winner_surface: "время".to_string(),
                cohort_surfaces: vec!["время".to_string()],
            })
        );
        assert_eq!(materialized.len(), 1);
        assert_eq!(materialized[0].surface, "время");
    }

    #[test]
    fn l1_geometry_keeps_multiple_single_edit_peaks_tied() {
        let lexical = vec![
            lexical_candidate("мзс", 1757, 2, 0, 3),
            lexical_candidate("мзд", 1743, 2, 0, 3),
        ];
        let mut materialized = lexical.clone();

        let readout = settle_unique_l1_geometry("мзт", &lexical, &mut materialized, true);

        assert_eq!(
            readout,
            Some(CanonicalCohortReadout::Tied {
                cohort_surfaces: vec!["мзд".to_string(), "мзс".to_string()],
            })
        );
        assert_eq!(materialized.len(), 2);
    }

    #[test]
    fn abstain_demotes_all_owned_surface_candidates_not_only_reported_cohort() {
        let mut candidates = vec![
            unified_candidate(
                "проверка ",
                CandidateOrigin::L2Surface,
                CANONICAL_L2_SURFACE_SOURCE_ID,
            ),
            unified_candidate(
                "проварка ",
                CandidateOrigin::L2Surface,
                CANONICAL_L2_SURFACE_SOURCE_ID,
            ),
        ];
        let readout = CanonicalCohortReadout::Abstain {
            cohort_surfaces: vec!["проверка".to_string()],
        };

        demote_canonical_local_surface_cohort(&mut candidates, &readout);

        assert!(candidates
            .iter()
            .all(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly));
    }

    #[test]
    fn abstain_demotes_lexical_authority_but_preserves_independent_layout() {
        let mut candidates = vec![
            unified_candidate(
                "понимаешь ",
                CandidateOrigin::DeterministicTypo,
                "composite_ru_typo",
            ),
            unified_candidate("vpn ", CandidateOrigin::Layout, "layout_ru_to_en"),
        ];

        apply_authority_to_candidate_lattice(&mut candidates, &L2FieldAuthority::Abstain);

        assert_eq!(candidates[0].gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(candidates[1].gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn unavailable_requested_field_fails_closed_for_lexical_edits() {
        let mut candidates = vec![
            unified_candidate(
                "Приши ",
                CandidateOrigin::DeterministicTypo,
                "missing_letter",
            ),
            unified_candidate("pdf ", CandidateOrigin::Layout, "layout_ru_to_en"),
        ];

        apply_authority_to_candidate_lattice(&mut candidates, &L2FieldAuthority::Unavailable);

        assert_eq!(candidates[0].gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(
            candidates[0].gate.reason,
            "l2_field_unavailable_requires_suggestion"
        );
        assert_eq!(candidates[1].gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn winner_owns_lexical_authority_case_insensitively() {
        let mut candidates = vec![
            unified_candidate(
                "Посмотри ",
                CandidateOrigin::DeterministicTypo,
                "missing_letter",
            ),
            unified_candidate(
                "Посмотреть ",
                CandidateOrigin::DeterministicTypo,
                "missing_letter",
            ),
        ];

        apply_authority_to_candidate_lattice(
            &mut candidates,
            &L2FieldAuthority::Winner {
                surface: "посмотри".to_string(),
            },
        );

        assert_eq!(candidates[0].gate.action, CandidateGateAction::Eligible);
        assert_eq!(candidates[1].gate.action, CandidateGateAction::SuggestOnly);
    }

    #[test]
    fn tie_demotes_ambiguous_length_change_but_preserves_other_verified_member() {
        let mut candidates = vec![
            unified_candidate(
                "перехвачу ",
                CandidateOrigin::DeterministicTypo,
                "missing_letter",
            ),
            unified_candidate(
                "передачу ",
                CandidateOrigin::DeterministicTypo,
                "composite_ru_typo",
            ),
        ];
        candidates[0].error_class = TypingErrorClass::MissingLetter;
        candidates[1].error_class = TypingErrorClass::LetterSubstitution;
        candidates.push(unified_candidate(
            "перехвачу ",
            CandidateOrigin::DeterministicTypo,
            "vowel_confusion",
        ));
        candidates[2].error_class = TypingErrorClass::LetterSubstitution;
        candidates.push(unified_candidate(
            "перехват ",
            CandidateOrigin::DeterministicTypo,
            "missing_letter",
        ));

        apply_authority_to_candidate_lattice(
            &mut candidates,
            &L2FieldAuthority::Tied {
                surfaces: vec!["первачу".to_string(), "перехвачу".to_string()],
            },
        );

        assert_eq!(candidates[0].gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(candidates[1].gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(candidates[2].gate.action, CandidateGateAction::Eligible);
        assert_eq!(candidates[3].gate.action, CandidateGateAction::SuggestOnly);
    }
}
