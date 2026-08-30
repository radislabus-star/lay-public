use crate::candidate_contract::CandidateOrigin;
use crate::candidate_contract::CorrectionSourceRole;
use crate::correction_core::{
    CandidateEvidence, CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource,
    MorphologySlotEvidence, TypingErrorClass, UnifiedCorrectionCandidate,
};
use crate::text_case::apply_word_case;
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::{
    replace_last_text_word, split_last_alphabetic_token, split_last_trimmed_ws_token,
    split_word_punctuation,
};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

use super::productive_v1::{
    CanonicalContourRelation, CanonicalContourSeed, CanonicalFormGrounding,
    CanonicalSurfaceGrounding,
};
use super::runtime::{
    CanonicalFieldCacheDisposition, CanonicalFieldTelemetry, CanonicalL2FieldReadout,
    CompositionalFormBirth, CompositionalLemmaBirth, L2FieldAuthority, L2FieldAvailability,
    L2LexicalSeed, L2LexicalSeedOrigin, L2LocalVerdict, ObservedCanonicalL2FieldReadout,
    ProductiveL2FormBirth, StandaloneL2Field, StandaloneL2Readout,
    CANONICAL_L2_PRODUCTIVE_SOURCE_ID, CANONICAL_L2_READOUT_SOURCE_ID,
    CANONICAL_L2_SURFACE_SOURCE_ID,
};

pub(crate) fn canonical_text_candidates(original: &str) -> Vec<UnifiedCorrectionCandidate> {
    canonical_text_readout(original).candidates
}

pub(crate) fn canonical_text_readout(original: &str) -> CanonicalL2FieldReadout {
    canonical_text_readout_observed(original).readout
}

pub(crate) fn canonical_text_readout_observed(original: &str) -> ObservedCanonicalL2FieldReadout {
    canonical_text_readout_observed_with_frame(original, None)
}

pub(crate) fn canonical_text_readout_observed_with_frame(
    original: &str,
    lexical_frame: Option<&crate::lexical_authority_frame::LexicalAuthorityFrameV1>,
) -> ObservedCanonicalL2FieldReadout {
    let (mut observed, boundary_candidates) = rayon::join(
        || canonical_owned_text_candidates_observed(original, lexical_frame),
        || boundary_text_candidates(original),
    );
    for candidate in boundary_candidates {
        if let Some(existing) = observed
            .readout
            .candidates
            .iter_mut()
            .find(|existing| existing.replacement == candidate.replacement)
        {
            existing.merge_evidence(candidate);
        } else {
            observed.readout.candidates.push(candidate);
        }
    }
    observed
}

/// Projects the one canonical L1.1 -> Productive V90 field into the live IME
/// lattice. The shared candidate gate remains the only ranking owner and Tab
/// remains the only mutation authority for whole-token replacements.
pub(crate) fn canonical_ime_candidates_observed(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> (
    L2FieldAvailability,
    Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    CanonicalFieldTelemetry,
) {
    if limit == 0 {
        return (
            L2FieldAvailability::UnsupportedInput,
            Vec::new(),
            CanonicalFieldTelemetry::default(),
        );
    }
    let original = if context_prefix.is_empty() {
        token.to_string()
    } else {
        format!("{context_prefix}{token}")
    };
    let observed = canonical_owned_text_candidates_observed(&original, None);
    let telemetry = observed.telemetry;
    let readout = observed.readout;
    let availability = readout.availability;
    if availability != L2FieldAvailability::Ready {
        return (availability, Vec::new(), telemetry);
    }

    let normalized = token.to_lowercase();
    let usage = super::super::usage_prior::cached_usage_prior_snapshot();
    let context = super::super::llmwave::tokenize(context_prefix);
    let prepared_usage = usage.prepare_hot_context(&context);
    let authoritative_surface = match &readout.authority {
        L2FieldAuthority::Winner { surface } => Some(surface.to_lowercase()),
        _ => None,
    };
    let mut candidates = readout
        .candidates
        .into_iter()
        .filter_map(|candidate| {
            let (_, surface) = split_last_alphabetic_token(&candidate.replacement)?;
            if surface.eq_ignore_ascii_case(&normalized) || surface.chars().any(char::is_whitespace)
            {
                return None;
            }
            let source = match candidate.origin {
                CandidateOrigin::Layout => {
                    crate::nanda_wave::l2::L2ImeWordCandidateSource::ExactLayoutPhase
                }
                CandidateOrigin::LayoutThenTypo => {
                    crate::nanda_wave::l2::L2ImeWordCandidateSource::LayoutThenTypoPhase
                }
                _ => crate::nanda_wave::l2::L2ImeWordCandidateSource::CanonicalField,
            };
            let prior = usage.candidate_prior_prepared(&prepared_usage, surface);
            let morphology_slots = candidate
                .morphology_slot_evidence
                .iter()
                .map(|evidence| crate::correction_core::MorphologySlotIdentity {
                    domain: crate::correction_core::MorphologySlotIdentityDomain::ProductiveV1,
                    lemma_id: evidence.lemma_id,
                    slot_id: evidence.target_feature_mask,
                })
                .collect::<Vec<_>>();
            let seed = crate::nanda_wave::L11SeedSurface {
                terminal_id: None,
                surface: surface.to_string(),
                authority: authoritative_surface
                    .as_deref()
                    .is_some_and(|winner| winner.eq_ignore_ascii_case(surface)),
                score_milli: canonical_ime_score_milli(&candidate),
            };
            let mut projected = l11_seed_only_candidate(&normalized, &seed);
            projected.source = source;
            projected.usage_prior = prior.word_prior;
            projected.context_prior = prior.context_prior;
            projected.accepted_count = prior.accepted_count;
            projected.morphology_slots = morphology_slots;
            Some(projected)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        canonical_ime_source_priority(right.source)
            .cmp(&canonical_ime_source_priority(left.source))
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface.eq_ignore_ascii_case(&right.surface));
    candidates.truncate(limit);
    (availability, candidates, telemetry)
}

fn canonical_ime_score_milli(candidate: &UnifiedCorrectionCandidate) -> u32 {
    let gate = match candidate.gate.action {
        CandidateGateAction::Eligible => 1_000,
        CandidateGateAction::SuggestOnly => 760,
        CandidateGateAction::KeepOriginal | CandidateGateAction::Veto => 0,
    };
    let source = match candidate.origin {
        CandidateOrigin::Layout => 320,
        CandidateOrigin::LayoutThenTypo => 160,
        CandidateOrigin::L2Surface => 240,
        _ => 80,
    };
    gate + source
}

fn canonical_ime_source_priority(source: crate::nanda_wave::l2::L2ImeWordCandidateSource) -> u8 {
    match source {
        crate::nanda_wave::l2::L2ImeWordCandidateSource::ExactLayoutPhase => 3,
        crate::nanda_wave::l2::L2ImeWordCandidateSource::CanonicalField => 2,
        crate::nanda_wave::l2::L2ImeWordCandidateSource::LayoutThenTypoPhase => 1,
        _ => 0,
    }
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
    surfaces.sort();
    surfaces.dedup();
    surfaces
}

fn canonical_owned_text_candidates(original: &str) -> CanonicalL2FieldReadout {
    canonical_owned_text_candidates_observed(original, None).readout
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanonicalInputTokenKind {
    Cyrillic,
    PhysicalLayout,
    MixedLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CanonicalInputToken<'a> {
    context_prefix: &'a str,
    surface: &'a str,
    kind: CanonicalInputTokenKind,
}

fn canonical_input_token(original: &str) -> Option<CanonicalInputToken<'_>> {
    if let Some((context_prefix, surface)) = split_last_trimmed_ws_token(original) {
        if admitted_physical_layout_surface(surface)
            || admitted_contextual_short_layout_surface(context_prefix, surface)
        {
            return Some(CanonicalInputToken {
                context_prefix,
                surface,
                kind: CanonicalInputTokenKind::PhysicalLayout,
            });
        }
        if admitted_mixed_layout_surface(surface) {
            return Some(CanonicalInputToken {
                context_prefix,
                surface,
                kind: CanonicalInputTokenKind::MixedLayout,
            });
        }
    }

    let (context_prefix, surface) = split_last_alphabetic_token(original)?;
    surface
        .chars()
        .all(crate::keyboard::is_cyrillic_letter)
        .then_some(CanonicalInputToken {
            context_prefix,
            surface,
            kind: CanonicalInputTokenKind::Cyrillic,
        })
}

pub(super) fn canonical_input_lexical_parts(original: &str) -> Option<(&str, &str)> {
    canonical_input_token(original).map(|input| (input.context_prefix, input.surface))
}

fn admitted_contextual_short_layout_surface(context_prefix: &str, surface: &str) -> bool {
    if context_prefix.trim().is_empty()
        || surface.chars().count() != 1
        || !crate::layout_autoswitch::is_ascii_layout_letter_surface(surface)
    {
        return false;
    }
    let projected = crate::dict::convert(surface, crate::dict::Direction::Us2Ru);
    projected.chars().all(crate::keyboard::is_cyrillic_letter)
        && (crate::russian_lexicon::is_known_russian_word_or_form(&projected)
            || crate::lexicon::is_common_ru_word(&projected)
            || crate::nanda_wave::l2::l2_surface_foundation_contains(&projected))
}

fn admitted_mixed_layout_surface(surface: &str) -> bool {
    if surface.chars().count() < 2
        || crate::word_recognizer::is_cli_option_token(surface)
        || surface.chars().any(|character| !character.is_alphabetic())
    {
        return false;
    }
    let has_ascii = surface
        .chars()
        .any(|character| character.is_ascii_alphabetic());
    let has_cyrillic = surface.chars().any(crate::keyboard::is_cyrillic_letter);
    has_ascii && has_cyrillic
}

fn admitted_physical_layout_surface(surface: &str) -> bool {
    if !crate::layout_autoswitch::is_ascii_layout_letter_surface(surface)
        || crate::word_recognizer::is_cli_option_token(surface)
    {
        return false;
    }

    let (_, lexical_core, _) = split_word_punctuation(surface);
    if lexical_core.is_empty()
        || crate::word_recognizer::is_ascii_technical_or_brand_token(surface)
        || crate::layout_autoswitch::is_protected_ascii_layout_token(surface)
        || crate::layout_autoswitch::is_protected_ascii_layout_token(lexical_core)
    {
        return false;
    }

    let admitted_short_projection = surface.chars().count() <= 3
        && exact_layout_surface_has_independent_lexical_support(&crate::dict::convert(
            surface,
            crate::dict::Direction::Us2Ru,
        ));
    admitted_short_projection
        || !crate::layout_autoswitch::is_known_english_layout_autoswitch_word(
            &lexical_core.to_ascii_lowercase(),
        )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CanonicalQueryContour {
    surface: String,
    relation: CanonicalContourRelation,
}

#[derive(Clone, Debug)]
struct CanonicalQueryResult {
    contour: CanonicalQueryContour,
    seeds: Vec<crate::nanda_wave::L11SeedSurface>,
}

fn canonical_query_contours(input: CanonicalInputToken<'_>) -> Vec<CanonicalQueryContour> {
    let mut contours = vec![CanonicalQueryContour {
        surface: input.surface.to_string(),
        relation: CanonicalContourRelation::Identity,
    }];
    match input.kind {
        CanonicalInputTokenKind::PhysicalLayout => push_query_contour(
            &mut contours,
            crate::dict::convert(input.surface, crate::dict::Direction::Us2Ru),
            CanonicalContourRelation::ExactLayout,
        ),
        CanonicalInputTokenKind::Cyrillic => {
            let identity = crate::word_recognizer::recognize_token(input.surface);
            if !identity.known_ru {
                let projected = crate::dict::convert(input.surface, crate::dict::Direction::Ru2Us);
                if admitted_ascii_projection(&projected) {
                    push_query_contour(
                        &mut contours,
                        projected,
                        CanonicalContourRelation::ExactLayout,
                    );
                }
            }
        }
        CanonicalInputTokenKind::MixedLayout => {
            let ascii_count = input
                .surface
                .chars()
                .filter(|character| character.is_ascii_alphabetic())
                .count();
            let cyrillic_count = input
                .surface
                .chars()
                .filter(|character| crate::keyboard::is_cyrillic_letter(*character))
                .count();
            if cyrillic_count >= ascii_count {
                push_query_contour(
                    &mut contours,
                    crate::dict::convert(input.surface, crate::dict::Direction::Us2Ru),
                    CanonicalContourRelation::ExactLayout,
                );
            }
            if ascii_count >= cyrillic_count {
                push_query_contour(
                    &mut contours,
                    crate::dict::convert(input.surface, crate::dict::Direction::Ru2Us),
                    CanonicalContourRelation::ExactLayout,
                );
            }
        }
    }
    contours
}

fn push_query_contour(
    contours: &mut Vec<CanonicalQueryContour>,
    surface: String,
    relation: CanonicalContourRelation,
) {
    if surface.is_empty()
        || contours
            .iter()
            .any(|contour| contour.surface.eq_ignore_ascii_case(&surface))
    {
        return;
    }
    contours.push(CanonicalQueryContour { surface, relation });
}

fn admitted_ascii_projection(surface: &str) -> bool {
    let identity = crate::word_recognizer::recognize_token(surface);
    identity.known_en
        || crate::lexicon::is_common_en_technical_word(&surface.to_ascii_lowercase())
        || crate::word_recognizer::is_ascii_technical_or_brand_token(surface)
        || crate::layout_autoswitch::is_known_english_layout_autoswitch_word(
            &surface.to_ascii_lowercase(),
        )
}

fn live_l11_contour_results(
    contours: &[CanonicalQueryContour],
    material_limit: usize,
) -> std::io::Result<Vec<CanonicalQueryResult>> {
    let socket_path = crate::nanda_wave::default_l11_socket_path();
    if !socket_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "L1.1 service socket is unavailable",
        ));
    }
    contours
        .par_iter()
        .map(|contour| {
            let seeds = crate::nanda_wave::request_l11_seed_surfaces(
                &socket_path,
                &contour.surface,
                material_limit.max(1),
                l11_service_timeout(),
            )?
            .into_iter()
            .filter(|seed| !seed.surface.chars().any(char::is_whitespace))
            .take(material_limit)
            .collect();
            Ok(CanonicalQueryResult {
                contour: contour.clone(),
                seeds,
            })
        })
        .collect()
}

fn merge_contour_seeds(results: &[CanonicalQueryResult]) -> Vec<CanonicalContourSeed> {
    let exact_layout_surfaces = results
        .iter()
        .filter(|result| result.contour.relation == CanonicalContourRelation::ExactLayout)
        .map(|result| result.contour.surface.to_lowercase())
        .collect::<BTreeSet<_>>();
    let mut by_identity = BTreeMap::<(Option<u32>, String), CanonicalContourSeed>::new();
    for result in results {
        for seed in &result.seeds {
            let normalized = seed.surface.to_lowercase();
            let relation = if exact_layout_surfaces.contains(&normalized) {
                CanonicalContourRelation::ExactLayout
            } else if result.contour.relation == CanonicalContourRelation::ExactLayout {
                CanonicalContourRelation::LayoutThenTypo
            } else {
                CanonicalContourRelation::Identity
            };
            let query_surface = if relation == CanonicalContourRelation::ExactLayout {
                results
                    .iter()
                    .find(|candidate| {
                        candidate.contour.relation == CanonicalContourRelation::ExactLayout
                            && candidate
                                .contour
                                .surface
                                .eq_ignore_ascii_case(&seed.surface)
                    })
                    .map(|candidate| candidate.contour.surface.clone())
                    .unwrap_or_else(|| result.contour.surface.clone())
            } else {
                result.contour.surface.clone()
            };
            let key = (seed.terminal_id, normalized);
            by_identity
                .entry(key)
                .and_modify(|retained| {
                    retained.seed.authority |= seed.authority;
                    retained.seed.score_milli = retained.seed.score_milli.max(seed.score_milli);
                    if relation > retained.relation {
                        retained.relation = relation;
                        retained.query_surface.clone_from(&query_surface);
                    }
                })
                .or_insert_with(|| CanonicalContourSeed {
                    query_surface,
                    seed: seed.clone(),
                    relation,
                });
        }
    }
    let mut merged = by_identity.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .seed
            .authority
            .cmp(&left.seed.authority)
            .then_with(|| {
                contour_seed_priority(right.relation).cmp(&contour_seed_priority(left.relation))
            })
            .then_with(|| right.seed.score_milli.cmp(&left.seed.score_milli))
            .then_with(|| left.seed.terminal_id.cmp(&right.seed.terminal_id))
            .then_with(|| left.seed.surface.cmp(&right.seed.surface))
    });
    merged.truncate(super::super::lexical_grokking::L11_LIVE_LATTICE_LIMIT);
    merged
}

const fn contour_seed_priority(relation: CanonicalContourRelation) -> u8 {
    match relation {
        CanonicalContourRelation::ExactLayout => 3,
        CanonicalContourRelation::Identity => 2,
        CanonicalContourRelation::LayoutThenTypo => 1,
        CanonicalContourRelation::InverseGeometry => 0,
    }
}

fn canonical_form_groundings(
    canonical_index: &StandaloneL2Field,
    results: &[CanonicalQueryResult],
) -> Vec<CanonicalFormGrounding> {
    const MAX_FORM_GROUNDINGS: usize = 32;
    const MAX_INVERSE_FORMS_PER_CONTOUR: usize = 16;

    let mut by_form = BTreeMap::<u32, CanonicalFormGrounding>::new();
    for result in results {
        let query_peak = result.seeds.iter().map(|seed| seed.score_milli).max();
        if result.contour.relation == CanonicalContourRelation::ExactLayout {
            if let Some(form_ref) = canonical_index.form_ref_for_surface(&result.contour.surface) {
                merge_form_grounding(
                    &mut by_form,
                    CanonicalFormGrounding {
                        form_ref,
                        normalized_surface: result.contour.surface.to_lowercase(),
                        support_milli: query_peak.unwrap_or(1_000).max(1_000),
                        relation: CanonicalContourRelation::ExactLayout,
                    },
                );
            }
        }
        let inverse_relation = if result.contour.relation == CanonicalContourRelation::Identity {
            CanonicalContourRelation::InverseGeometry
        } else {
            CanonicalContourRelation::LayoutThenTypo
        };
        // Canonical inverse geometry is an independent bounded lookup lane. A
        // missing L1.1 seed must not disable it, but its minimal support never
        // turns it into an authoritative L1.1 observation.
        let inverse_support = query_peak.unwrap_or(1).max(1);
        for form_ref in canonical_index
            .single_edit_form_refs(&result.contour.surface, MAX_INVERSE_FORMS_PER_CONTOUR)
        {
            let Some(surface) = canonical_index
                .decode_form_ref(form_ref)
                .map(std::borrow::Cow::into_owned)
            else {
                continue;
            };
            merge_form_grounding(
                &mut by_form,
                CanonicalFormGrounding {
                    form_ref,
                    normalized_surface: surface,
                    support_milli: inverse_support,
                    relation: inverse_relation,
                },
            );
        }
    }
    let mut groundings = by_form.into_values().collect::<Vec<_>>();
    groundings.sort_by(|left, right| {
        right
            .relation
            .cmp(&left.relation)
            .then_with(|| right.support_milli.cmp(&left.support_milli))
            .then_with(|| left.form_ref.cmp(&right.form_ref))
    });
    groundings.truncate(MAX_FORM_GROUNDINGS);
    groundings
}

fn canonical_surface_groundings(
    canonical_index: &StandaloneL2Field,
    results: &[CanonicalQueryResult],
) -> Vec<CanonicalSurfaceGrounding> {
    const MAX_DIRECT_SURFACE_GROUNDINGS: usize = 4;
    const MAX_REFERENCE_SURFACE_GROUNDINGS_PER_CONTOUR: usize = 2;

    let mut by_surface = BTreeMap::<String, CanonicalSurfaceGrounding>::new();
    for result in results {
        let query_peak = result.seeds.iter().map(|seed| seed.score_milli).max();
        if result.contour.relation == CanonicalContourRelation::ExactLayout
            && exact_layout_surface_has_independent_lexical_support(&result.contour.surface)
        {
            let normalized_surface = result.contour.surface.to_lowercase();
            by_surface.insert(
                normalized_surface.clone(),
                CanonicalSurfaceGrounding {
                    normalized_surface,
                    support_milli: query_peak.unwrap_or(1_000).max(1_000),
                    relation: CanonicalContourRelation::ExactLayout,
                },
            );
        }
        let relation = if result.contour.relation == CanonicalContourRelation::Identity {
            CanonicalContourRelation::InverseGeometry
        } else {
            CanonicalContourRelation::LayoutThenTypo
        };
        let support_milli = query_peak.unwrap_or(1).max(1);
        for surface in reference_backed_missing_letter_surfaces(
            &result.contour.surface,
            MAX_REFERENCE_SURFACE_GROUNDINGS_PER_CONTOUR,
        ) {
            if canonical_index.form_ref_for_surface(&surface).is_some() {
                continue;
            }
            let normalized_surface = surface.to_lowercase();
            by_surface
                .entry(normalized_surface.clone())
                .and_modify(|retained| {
                    retained.support_milli = retained.support_milli.max(support_milli);
                    retained.relation = retained.relation.max(relation);
                })
                .or_insert(CanonicalSurfaceGrounding {
                    normalized_surface,
                    support_milli,
                    relation,
                });
        }
    }
    let mut groundings = by_surface.into_values().collect::<Vec<_>>();
    groundings.sort_by(|left, right| {
        right
            .relation
            .cmp(&left.relation)
            .then_with(|| right.support_milli.cmp(&left.support_milli))
            .then_with(|| left.normalized_surface.cmp(&right.normalized_surface))
    });
    groundings.truncate(MAX_DIRECT_SURFACE_GROUNDINGS);
    groundings
}

fn exact_layout_surface_has_independent_lexical_support(surface: &str) -> bool {
    let lower = surface.to_lowercase();
    if lower.chars().all(crate::keyboard::is_cyrillic_letter) {
        return crate::russian_lexicon::is_reference_known_russian_word_or_form(&lower)
            || crate::russian_lexicon::is_known_russian_word_or_form(&lower)
            || crate::lexicon::is_common_ru_word(&lower)
            || crate::nanda_wave::l2::l2_surface_foundation_contains(&lower);
    }
    if lower
        .chars()
        .all(|character| character.is_ascii_alphabetic())
    {
        let identity = crate::word_recognizer::recognize_token(&lower);
        return identity.known_en
            || crate::lexicon::is_common_en_technical_word(&lower)
            || crate::word_recognizer::is_ascii_technical_or_brand_token(&lower)
            || crate::layout_autoswitch::is_known_english_layout_autoswitch_word(&lower);
    }
    false
}

fn merge_form_grounding(
    by_form: &mut BTreeMap<u32, CanonicalFormGrounding>,
    grounding: CanonicalFormGrounding,
) {
    by_form
        .entry(grounding.form_ref)
        .and_modify(|retained| {
            retained.support_milli = retained.support_milli.max(grounding.support_milli);
            retained.relation = retained.relation.max(grounding.relation);
        })
        .or_insert(grounding);
}

fn canonical_contour_identity_bytes(
    results: &[CanonicalQueryResult],
    seeds: &[CanonicalContourSeed],
    forms: &[CanonicalFormGrounding],
    surfaces: &[CanonicalSurfaceGrounding],
) -> Vec<u8> {
    let mut bytes = b"LAY-CONTOUR-FIELD-V2\0".to_vec();
    push_identity_count(&mut bytes, results.len());
    for result in results {
        bytes.push(result.contour.relation.tag());
        push_identity_text(&mut bytes, &result.contour.surface);
        push_identity_count(&mut bytes, result.seeds.len());
        for seed in &result.seeds {
            bytes.extend_from_slice(&seed.terminal_id.unwrap_or(u32::MAX).to_le_bytes());
            bytes.push(u8::from(seed.authority));
            bytes.extend_from_slice(&seed.score_milli.to_le_bytes());
            push_identity_text(&mut bytes, &seed.surface);
        }
    }
    push_identity_count(&mut bytes, seeds.len());
    for evidence in seeds {
        bytes.push(evidence.relation.tag());
        push_identity_text(&mut bytes, &evidence.query_surface);
        bytes.extend_from_slice(&evidence.seed.terminal_id.unwrap_or(u32::MAX).to_le_bytes());
        bytes.push(u8::from(evidence.seed.authority));
        bytes.extend_from_slice(&evidence.seed.score_milli.to_le_bytes());
        push_identity_text(&mut bytes, &evidence.seed.surface);
    }
    push_identity_count(&mut bytes, forms.len());
    for grounding in forms {
        bytes.push(grounding.relation.tag());
        bytes.extend_from_slice(&grounding.form_ref.to_le_bytes());
        bytes.extend_from_slice(&grounding.support_milli.to_le_bytes());
        push_identity_text(&mut bytes, &grounding.normalized_surface);
    }
    push_identity_count(&mut bytes, surfaces.len());
    for grounding in surfaces {
        bytes.push(grounding.relation.tag());
        bytes.extend_from_slice(&grounding.support_milli.to_le_bytes());
        push_identity_text(&mut bytes, &grounding.normalized_surface);
    }
    bytes
}

fn push_identity_count(bytes: &mut Vec<u8>, count: usize) {
    bytes.extend_from_slice(&(count.min(u32::MAX as usize) as u32).to_le_bytes());
}

fn push_identity_text(bytes: &mut Vec<u8>, text: &str) {
    push_identity_count(bytes, text.len());
    bytes.extend_from_slice(text.as_bytes());
}

fn canonical_owned_text_candidates_observed(
    original: &str,
    lexical_frame: Option<&crate::lexical_authority_frame::LexicalAuthorityFrameV1>,
) -> ObservedCanonicalL2FieldReadout {
    let started = std::time::Instant::now();
    let Some(input) = canonical_input_token(original) else {
        return observed_field_readout(
            CanonicalL2FieldReadout::abstain(L2FieldAvailability::UnsupportedInput),
            started,
            0,
            CanonicalFieldCacheDisposition::NotRequested,
            0,
        );
    };
    let token = input.surface;
    let query_contours = canonical_query_contours(input);
    if token.chars().count() < 2 && query_contours.len() == 1 {
        return observed_field_readout(
            CanonicalL2FieldReadout::abstain(L2FieldAvailability::UnsupportedInput),
            started,
            0,
            CanonicalFieldCacheDisposition::NotRequested,
            0,
        );
    }
    let seed_started = std::time::Instant::now();
    let query_results = match live_l11_contour_results(
        &query_contours,
        super::super::lexical_grokking::L11_LIVE_LATTICE_LIMIT,
    ) {
        Ok(results) => results,
        Err(_) => {
            return observed_field_readout(
                CanonicalL2FieldReadout::unavailable(L2FieldAvailability::L11ServiceUnavailable),
                started,
                elapsed_us(seed_started.elapsed()),
                CanonicalFieldCacheDisposition::NotRequested,
                0,
            )
        }
    };
    let contour_seeds = merge_contour_seeds(&query_results);
    let l11_seeds = contour_seeds
        .iter()
        .map(|evidence| evidence.seed.clone())
        .collect::<Vec<_>>();
    let seed_duration = seed_started.elapsed();
    // Productive V90 consumes one typed contour field. Canonical inverse forms
    // ground that field but never become independent L1.1 authority.
    let (readout, cache_disposition, field_generation, cohort_compare) =
        match super::installed_l2_field() {
            Err(_) => (
                CanonicalL2FieldReadout::unavailable(
                    L2FieldAvailability::CanonicalPackageUnavailable,
                ),
                CanonicalFieldCacheDisposition::NotRequested,
                0,
                None,
            ),
            Ok(canonical_index) => {
                let form_groundings = canonical_form_groundings(canonical_index, &query_results);
                let surface_groundings =
                    canonical_surface_groundings(canonical_index, &query_results);
                let exact_generation = match super::installed_exact_v13(canonical_index) {
                    Ok(generation) => generation,
                    Err(error) => {
                        if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
                            eprintln!("l2_field_trace exact_v13_admission_error={error:?}");
                        }
                        None
                    }
                };
                if contour_seeds.is_empty()
                    && form_groundings.is_empty()
                    && surface_groundings.is_empty()
                    && exact_generation.is_none()
                {
                    (
                        CanonicalL2FieldReadout::abstain(L2FieldAvailability::EmptyL11Lattice),
                        CanonicalFieldCacheDisposition::NotRequested,
                        0,
                        None,
                    )
                } else {
                    match super::installed_productive_l2_v1() {
                        Err(error) => {
                            if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
                                eprintln!("l2_field_trace productive_admission_error={error:?}");
                            }
                            (
                                CanonicalL2FieldReadout::unavailable(
                                    L2FieldAvailability::ProductivePackageUnavailable,
                                ),
                                CanonicalFieldCacheDisposition::NotRequested,
                                0,
                                None,
                            )
                        }
                        Ok(runtime) => {
                            let contour_identity = canonical_contour_identity_bytes(
                                &query_results,
                                &contour_seeds,
                                &form_groundings,
                                &surface_groundings,
                            );
                            let key = super::cache::CanonicalTokenKey::new(
                                super::productive_v1::canonical_live_scene_bytes(
                                    input.context_prefix,
                                    token,
                                    canonical_index,
                                ),
                                &contour_identity,
                                &l11_seeds,
                                runtime.l11_package_sha256(),
                                runtime.canonical_l2_package_sha256(),
                                runtime.package_sha256(),
                                exact_generation
                                    .as_ref()
                                    .map(|generation| generation.sidecar_sha256())
                                    .unwrap_or([0; 32]),
                            );
                            match super::cache::get_or_prepare(key, || {
                                let exact_peaks = exact_generation
                                    .as_ref()
                                    .map_or_else(
                                        || {
                                            Ok(super::productive_v1::ExactPeakBirthEnumerationV1::complete_empty())
                                        },
                                        |generation| {
                                            generation.exact_peaks(canonical_index, token)
                                        },
                                    )
                                    .unwrap_or_else(|error| {
                                        if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
                                            eprintln!(
                                                "l2_field_trace exact_v13_query_error={error:?}"
                                            );
                                        }
                                        super::productive_v1::ExactPeakBirthEnumerationV1::incomplete(
                                            crate::typing_transition::target_evidence::IncompletenessReasonV1::IntegrityFailure,
                                        )
                                    });
                                super::productive_v1::prepare_live_productive_v1_field_with_exact_peaks(
                                    input.context_prefix,
                                    token,
                                    canonical_index,
                                    &runtime,
                                    &contour_seeds,
                                    &form_groundings,
                                    &surface_groundings,
                                    exact_peaks,
                                )
                            }) {
                                Ok(prepared) => {
                                    let disposition = match prepared.disposition {
                                        super::cache::FieldCacheDisposition::Produced => {
                                            CanonicalFieldCacheDisposition::Produced
                                        }
                                        super::cache::FieldCacheDisposition::Waited => {
                                            CanonicalFieldCacheDisposition::Waited
                                        }
                                        super::cache::FieldCacheDisposition::ReadyHit => {
                                            CanonicalFieldCacheDisposition::ReadyHit
                                        }
                                    };
                                    let current =
                                        super::cache::generation_is_current(prepared.generation)
                                            && prepared.field.productive_package_sha256()
                                                == runtime.package_sha256();
                                    let readout = if !current {
                                        CanonicalL2FieldReadout::unavailable(
                                            L2FieldAvailability::ProductiveReadoutError,
                                        )
                                    } else {
                                        super::productive_v1::materialize_live_productive_v1_field(
                                        original,
                                        token,
                                        &prepared.field,
                                    )
                                    .unwrap_or_else(|error| {
                                        if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
                                            eprintln!(
                                                "l2_field_trace productive_materialization_error={error:?}"
                                            );
                                        }
                                        CanonicalL2FieldReadout::unavailable(
                                            L2FieldAvailability::ProductiveReadoutError,
                                        )
                                    })
                                    };
                                    let cohort_compare = current.then(|| {
                                        super::productive_v1::compare_shared_canonical_cohort(
                                            &prepared.field,
                                            lexical_frame,
                                            prepared.generation,
                                        )
                                    });
                                    (readout, disposition, prepared.generation, cohort_compare)
                                }
                                Err(error) => {
                                    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
                                        eprintln!("l2_field_trace field_cache_error={error:?}");
                                    }
                                    (
                                        CanonicalL2FieldReadout::unavailable(
                                            L2FieldAvailability::ProductiveReadoutError,
                                        ),
                                        CanonicalFieldCacheDisposition::Failed,
                                        0,
                                        None,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        };
    let mut observed = observed_field_readout(
        readout,
        started,
        elapsed_us(seed_duration),
        cache_disposition,
        field_generation,
    );
    observed.cohort_compare = cohort_compare;
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "l2_field_trace owner=productive_v90 seeds_us={} productive_us={} total_us={} contours={} seeds={} candidates={} field_cache={:?} field_generation={}",
            observed.telemetry.l11_us,
            observed.telemetry.productive_v90_us,
            observed.telemetry.total_us,
            query_contours.len(),
            l11_seeds.len(),
            observed.readout.candidates.len(),
            observed.telemetry.cache_disposition,
            observed.telemetry.field_generation,
        );
        if let Some(compare) = &observed.cohort_compare {
            eprintln!(
                "l2_cohort_compare status={:?} material_scope={:?} legacy={:?} cohort={:?} field_candidates={} material_targets={} retained={} grounded_l11_loss={} complete_for_authority={} first_divergence={:?}",
                compare.status,
                compare.material_scope,
                compare.legacy,
                compare.cohort,
                compare.field_candidate_count,
                compare.material_target_count,
                compare.retained_field_candidate_count,
                compare.grounded_l11_loss_count,
                compare.complete_for_authority,
                compare.first_divergence,
            );
        }
    }
    observed
}

fn observed_field_readout(
    readout: CanonicalL2FieldReadout,
    started: std::time::Instant,
    l11_us: u64,
    cache_disposition: CanonicalFieldCacheDisposition,
    field_generation: u64,
) -> ObservedCanonicalL2FieldReadout {
    let total_us = elapsed_us(started.elapsed());
    ObservedCanonicalL2FieldReadout {
        readout,
        telemetry: CanonicalFieldTelemetry {
            l11_us,
            productive_v90_us: total_us.saturating_sub(l11_us),
            total_us,
            field_producer_count: cache_disposition.producer_count(),
            cache_disposition,
            field_generation,
        },
        cohort_compare: None,
    }
}

fn elapsed_us(duration: std::time::Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

pub(super) fn query_live_canonical_l2(
    original: &str,
    repeat: usize,
    productive_lemma_limit: usize,
) -> serde_json::Value {
    let mut iteration_us = Vec::with_capacity(repeat);
    let mut field_us = Vec::with_capacity(repeat);
    let mut productive_us = Vec::with_capacity(repeat);
    let mut last_readout = None;
    let mut last_seed_count = 0;
    let mut last_surface_count = 0;
    let mut last_productive_surfaces = std::collections::BTreeSet::new();
    let mut prepared_cache_hits = 0_usize;

    for _ in 0..repeat {
        let started = std::time::Instant::now();
        let Some(field) = standalone_surface_field_readout_with_productive_limit(
            original,
            false,
            productive_lemma_limit,
        ) else {
            return serde_json::json!({
                "kind": "canonical_l2_live_query",
                "status": "unavailable",
                "text": original,
                "repeat": repeat,
                "productive_lemma_limit": productive_lemma_limit,
                "runtime_authority_changed": false,
            });
        };
        let mut candidates =
            l2_surface_unified_candidates(original, &field.token, &field.surface_candidates);
        mark_productive_surface_candidates(
            &mut candidates,
            &field.productive_surfaces,
            &field.morphology_evidence_by_surface,
        );
        promote_canonical_local_readout(&mut candidates, &field.local_readout);
        demote_canonical_local_surface_cohort(&mut candidates, &field.local_readout);
        let authority = field.local_readout.authority();
        apply_authority_to_candidate_lattice(&mut candidates, &authority);
        iteration_us.push(started.elapsed().as_micros() as u64);
        field_us.push(field.field_duration.as_micros() as u64);
        productive_us.push(field.productive_duration.as_micros() as u64);
        prepared_cache_hits += usize::from(field.prepared_cache_hit);
        last_seed_count = field.seed_count;
        last_surface_count = field.surface_count;
        last_productive_surfaces = field.productive_surfaces;
        last_readout = Some(CanonicalL2FieldReadout::new(candidates, authority));
    }

    let readout = last_readout.expect("validated non-zero repeat");
    let authority = match &readout.authority {
        L2FieldAuthority::Winner { surface } => serde_json::json!({
            "kind": "winner",
            "surfaces": [surface],
        }),
        L2FieldAuthority::Tied { surfaces } => serde_json::json!({
            "kind": "tied",
            "surfaces": surfaces,
        }),
        L2FieldAuthority::Abstain => serde_json::json!({
            "kind": "abstain",
            "surfaces": [],
        }),
        L2FieldAuthority::Unavailable => serde_json::json!({
            "kind": "unavailable",
            "surfaces": [],
        }),
    };
    let cold_us = iteration_us.first().copied().unwrap_or_default();
    let hot_us = iteration_us.iter().skip(1).copied().collect::<Vec<_>>();
    serde_json::json!({
        "kind": "canonical_l2_live_query",
        "status": "ready",
        "text": original,
        "repeat": repeat,
        "productive_lemma_limit": productive_lemma_limit,
        "productive_form_limit": super::CANONICAL_L2_PRODUCTIVE_FORM_LIMIT,
        "cache_mode": "bounded_prepared_frontier_lru",
        "prepared_cache_hits": prepared_cache_hits,
        "cold_us": cold_us,
        "hot": latency_summary(&hot_us),
        "all_iterations": latency_summary(&iteration_us),
        "field": latency_summary(&field_us),
        "productive": latency_summary(&productive_us),
        "seed_count": last_seed_count,
        "pre_productive_surface_count": last_surface_count,
        "productive_surfaces": last_productive_surfaces,
        "authority": authority,
        "candidates": readout.candidates.iter().map(|candidate| serde_json::json!({
            "replacement": candidate.replacement,
            "source_id": candidate.source_id,
            "origin": format!("{:?}", candidate.origin),
            "error_class": candidate.error_class.as_str(),
            "gate_action": format!("{:?}", candidate.gate.action),
            "gate_reason": candidate.gate.reason,
            "productive": candidate.source_id == CANONICAL_L2_PRODUCTIVE_SOURCE_ID,
        })).collect::<Vec<_>>(),
        "peak_rss_kib": process_status_kib("VmHWM:"),
        "resident_rss_kib": process_status_kib("VmRSS:"),
        "runtime_authority_changed": false,
    })
}

/// Benchmarks the exact Productive V90 producer used by the live text and IME
/// projections. This deliberately bypasses both the legacy compositional query
/// helper and the outer materialized-readout cache.
pub(super) fn query_live_productive_v90(original: &str, repeat: usize) -> serde_json::Value {
    let cache_before = super::cache::stats();
    let mut iteration_us = Vec::with_capacity(repeat);
    let mut last_readout = None;

    for _ in 0..repeat {
        let started = std::time::Instant::now();
        let readout = canonical_owned_text_candidates(original);
        iteration_us.push(started.elapsed().as_micros() as u64);
        if readout.availability != L2FieldAvailability::Ready {
            let cache_delta = super::cache::stats().delta(cache_before);
            return serde_json::json!({
                "kind": "productive_v90_live_owner_query",
                "status": "unavailable",
                "text": original,
                "repeat": repeat,
                "producer_symbol": "canonical_owned_text_candidates",
                "owner_symbol": "prepare_live_productive_v1_field",
                "materializer_symbol": "materialize_live_productive_v1_field",
                "outer_readout_cache_bypassed": true,
                "availability": format!("{:?}", readout.availability),
                "completed_iterations": iteration_us.len(),
                "all_iterations": latency_summary(&iteration_us),
                "field_cache": field_cache_stats_json(cache_delta),
                "runtime_authority_changed": false,
            });
        }
        last_readout = Some(readout);
    }

    let readout = last_readout.expect("validated non-zero repeat");
    let authority = match &readout.authority {
        L2FieldAuthority::Winner { surface } => serde_json::json!({
            "kind": "winner",
            "surfaces": [surface],
        }),
        L2FieldAuthority::Tied { surfaces } => serde_json::json!({
            "kind": "tied",
            "surfaces": surfaces,
        }),
        L2FieldAuthority::Abstain => serde_json::json!({
            "kind": "abstain",
            "surfaces": [],
        }),
        L2FieldAuthority::Unavailable => serde_json::json!({
            "kind": "unavailable",
            "surfaces": [],
        }),
    };
    let cold_us = iteration_us.first().copied().unwrap_or_default();
    let hot_us = iteration_us.iter().skip(1).copied().collect::<Vec<_>>();
    let cache_delta = super::cache::stats().delta(cache_before);
    serde_json::json!({
        "kind": "productive_v90_live_owner_query",
        "status": "ready",
        "text": original,
        "repeat": repeat,
        "producer_symbol": "canonical_owned_text_candidates",
        "owner_symbol": "prepare_live_productive_v1_field",
        "materializer_symbol": "materialize_live_productive_v1_field",
        "outer_readout_cache_bypassed": true,
        "producer_calls": cache_delta.producer_computations,
        "cold_us": cold_us,
        "hot": latency_summary(&hot_us),
        "all_iterations": latency_summary(&iteration_us),
        "availability": format!("{:?}", readout.availability),
        "authority": authority,
        "field_cache": field_cache_stats_json(cache_delta),
        "candidates": readout.candidates.iter().map(|candidate| serde_json::json!({
            "replacement": candidate.replacement,
            "source_id": candidate.source_id,
            "origin": format!("{:?}", candidate.origin),
            "error_class": candidate.error_class.as_str(),
            "gate_action": format!("{:?}", candidate.gate.action),
            "gate_reason": candidate.gate.reason,
        })).collect::<Vec<_>>(),
        "peak_rss_kib": process_status_kib("VmHWM:"),
        "resident_rss_kib": process_status_kib("VmRSS:"),
        "runtime_authority_changed": false,
    })
}

fn field_cache_stats_json(stats: super::cache::FieldCacheStatsSnapshot) -> serde_json::Value {
    serde_json::json!({
        "generation": stats.generation,
        "ready_entries": stats.ready_entries,
        "in_flight_entries": stats.in_flight_entries,
        "producer_computations": stats.producer_computations,
        "waiter_joins": stats.waiter_joins,
        "ready_hits": stats.ready_hits,
        "transient_failures": stats.transient_failures,
        "superseded_publications": stats.superseded_publications,
        "saturated_requests": stats.saturated_requests,
        "invalidations": stats.invalidations,
    })
}

fn latency_summary(samples: &[u64]) -> serde_json::Value {
    if samples.is_empty() {
        return serde_json::json!({
            "samples": 0,
            "min_us": null,
            "p50_us": null,
            "p95_us": null,
            "p99_us": null,
            "max_us": null,
            "mean_us": null,
        });
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let percentile = |numerator: usize, denominator: usize| {
        let index = (sorted.len() * numerator)
            .div_ceil(denominator)
            .saturating_sub(1);
        sorted[index.min(sorted.len() - 1)]
    };
    let total = sorted.iter().copied().map(u128::from).sum::<u128>();
    serde_json::json!({
        "samples": sorted.len(),
        "min_us": sorted[0],
        "p50_us": percentile(50, 100),
        "p95_us": percentile(95, 100),
        "p99_us": percentile(99, 100),
        "max_us": sorted[sorted.len() - 1],
        "mean_us": (total / sorted.len() as u128) as u64,
    })
}

fn process_status_kib(prefix: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix(prefix)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .unwrap_or_default()
}

struct StandaloneSurfaceFieldReadout {
    token: String,
    surface_candidates: Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    local_readout: CanonicalCohortReadout,
    productive_surfaces: std::collections::BTreeSet<String>,
    morphology_evidence_by_surface: std::collections::BTreeMap<String, Vec<MorphologySlotEvidence>>,
    seed_count: usize,
    surface_count: usize,
    field_duration: std::time::Duration,
    productive_duration: std::time::Duration,
    prepared_cache_hit: bool,
}

#[derive(Clone)]
struct PreparedCompositionalField {
    broad_lemma_births: Vec<CompositionalLemmaBirth>,
    active_lemma_births: Vec<CompositionalLemmaBirth>,
    form_births: Vec<CompositionalFormBirth>,
    productive_form_births: Vec<ProductiveL2FormBirth>,
    broad_duration: std::time::Duration,
    active_duration: std::time::Duration,
    form_duration: std::time::Duration,
    productive_duration: std::time::Duration,
    total_duration: std::time::Duration,
    cache_hit: bool,
}

const PREPARED_FIELD_CACHE_LIMIT: usize = 64;

struct PreparedFieldCacheEntry {
    context: String,
    token: String,
    productive_lemma_limit: usize,
    field: PreparedCompositionalField,
}

fn prepared_field_cache(
) -> &'static std::sync::Mutex<std::collections::VecDeque<PreparedFieldCacheEntry>> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::VecDeque<PreparedFieldCacheEntry>>,
    > = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::VecDeque::new()))
}

pub(super) fn clear_prepared_field_cache() {
    if let Ok(mut cache) = prepared_field_cache().lock() {
        cache.clear();
    }
}

fn cached_prepared_field(
    context: &str,
    token: &str,
    productive_lemma_limit: usize,
) -> Option<PreparedCompositionalField> {
    let Ok(mut cache) = prepared_field_cache().lock() else {
        return None;
    };
    let index = cache.iter().position(|entry| {
        entry.context == context
            && entry.token == token
            && entry.productive_lemma_limit == productive_lemma_limit
    })?;
    let entry = cache.remove(index)?;
    let mut field = entry.field.clone();
    field.broad_duration = std::time::Duration::ZERO;
    field.active_duration = std::time::Duration::ZERO;
    field.form_duration = std::time::Duration::ZERO;
    field.productive_duration = std::time::Duration::ZERO;
    field.total_duration = std::time::Duration::ZERO;
    field.cache_hit = true;
    cache.push_back(entry);
    Some(field)
}

fn store_prepared_field(
    context: &str,
    token: &str,
    productive_lemma_limit: usize,
    field: &PreparedCompositionalField,
) {
    let Ok(mut cache) = prepared_field_cache().lock() else {
        return;
    };
    if let Some(index) = cache.iter().position(|entry| {
        entry.context == context
            && entry.token == token
            && entry.productive_lemma_limit == productive_lemma_limit
    }) {
        cache.remove(index);
    }
    cache.push_back(PreparedFieldCacheEntry {
        context: context.to_string(),
        token: token.to_string(),
        productive_lemma_limit,
        field: field.clone(),
    });
    while cache.len() > PREPARED_FIELD_CACHE_LIMIT {
        cache.pop_front();
    }
}

fn standalone_surface_field_readout(
    original: &str,
    settle_winner: bool,
) -> Option<StandaloneSurfaceFieldReadout> {
    standalone_surface_field_readout_with_productive_limit(
        original,
        settle_winner,
        super::CANONICAL_L2_PRODUCTIVE_LEMMA_LIMIT,
    )
}

fn standalone_surface_field_readout_with_productive_limit(
    original: &str,
    settle_winner: bool,
    productive_lemma_limit: usize,
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
    let ((surface_candidates, l11_seeds), mut prepared) = std::thread::scope(|scope| {
        let prepared = scope
            .spawn(|| prepare_compositional_field(field, &context, token, productive_lemma_limit));
        let seeds = l11_surface_seed_candidates(token, SHADOW_SURFACE_MATERIAL_LIMIT);
        Some((seeds, prepared.join().ok()?))
    })?;
    let seed_count = l11_seeds.len();
    let surface_count = surface_candidates.len();
    let mut bounded_surface_candidates = bounded_surface_candidates_with_sparse_reserve(
        &normalized_token,
        &surface_candidates,
        HOT_L2_CANDIDATE_LIMIT,
        SPARSE_OMISSION_RESERVE,
    );
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
    let productive_duration = prepared.productive_duration;
    let prepared_cache_hit = prepared.cache_hit;
    let productive_form_births = std::mem::take(&mut prepared.productive_form_births);
    let apply_started = std::time::Instant::now();
    let (local_readout, _typed_local_readout) = apply_standalone_l2_field(
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
    let mut productive_projection = append_productive_surface_candidates(
        &normalized_token,
        productive_form_births,
        &mut bounded_surface_candidates,
    );
    if let Ok(index) = super::installed_productive_l2_sidecar() {
        if index.l2_fingerprint() == field.l1_package_fingerprint() {
            append_exact_morphology_evidence(
                field,
                index.as_ref(),
                &context,
                &bounded_surface_candidates,
                &mut productive_projection.evidence_by_surface,
            );
        }
    }
    Some(StandaloneSurfaceFieldReadout {
        token: token.to_string(),
        surface_candidates: bounded_surface_candidates,
        local_readout,
        productive_surfaces: productive_projection.added_surfaces,
        morphology_evidence_by_surface: productive_projection.evidence_by_surface,
        seed_count,
        surface_count,
        field_duration,
        productive_duration,
        prepared_cache_hit,
    })
}

fn bounded_surface_candidates_with_sparse_reserve(
    normalized_token: &str,
    surface_candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
    hot_candidate_limit: usize,
    sparse_omission_reserve: usize,
) -> Vec<crate::nanda_wave::l2::L2ImeWordCandidate> {
    let mut bounded = surface_candidates
        .iter()
        .take(hot_candidate_limit)
        .cloned()
        .collect::<Vec<_>>();
    for candidate in surface_candidates
        .iter()
        .filter(|candidate| {
            crate::text_metrics::sparse_internal_omission_count(
                normalized_token,
                &candidate.surface,
            )
            .is_some()
        })
        .take(sparse_omission_reserve)
    {
        if !bounded
            .iter()
            .any(|existing| existing.surface.eq_ignore_ascii_case(&candidate.surface))
        {
            bounded.push(candidate.clone());
        }
    }
    bounded
}

fn prepare_compositional_field(
    field: &StandaloneL2Field,
    context: &str,
    token: &str,
    productive_lemma_limit: usize,
) -> PreparedCompositionalField {
    if let Some(field) = cached_prepared_field(context, token, productive_lemma_limit) {
        return field;
    }
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
    let productive_started = std::time::Instant::now();
    let productive_form_births = super::installed_productive_l2_sidecar()
        .ok()
        .filter(|index| index.l2_fingerprint() == field.l1_package_fingerprint())
        .map(|index| {
            field.productive_form_births_from_lemmas(
                index.as_ref(),
                context,
                token,
                &active_lemma_births[..active_lemma_births.len().min(productive_lemma_limit)],
                super::CANONICAL_L2_FEATURE_LIMIT,
                super::CANONICAL_L2_PRODUCTIVE_FORM_LIMIT,
            )
        })
        .unwrap_or_default();
    let productive_ready = std::time::Instant::now();
    let prepared = PreparedCompositionalField {
        broad_lemma_births,
        active_lemma_births,
        form_births,
        productive_form_births,
        broad_duration: broad_ready.duration_since(started),
        active_duration: active_ready.duration_since(broad_ready),
        form_duration: form_ready.duration_since(active_ready),
        productive_duration: productive_ready.duration_since(productive_started),
        total_duration: productive_ready.duration_since(started),
        cache_hit: false,
    };
    store_prepared_field(context, token, productive_lemma_limit, &prepared);
    prepared
}

fn append_productive_surface_candidates(
    token: &str,
    births: Vec<ProductiveL2FormBirth>,
    candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
) -> ProductiveSurfaceProjection {
    let mut projection = ProductiveSurfaceProjection::default();
    for birth in births {
        let surface = birth.surface.to_lowercase();
        let generated = birth.exact_surface_form_ref.is_none();
        projection
            .evidence_by_surface
            .entry(surface.clone())
            .or_default()
            .push(MorphologySlotEvidence {
                lemma_id: birth.lemma_id,
                source_feature_mask:
                    crate::nanda_wave::morphology_phase::productive_context_slot_features(
                        birth.source_feature_mask,
                    ),
                target_feature_mask:
                    crate::nanda_wave::morphology_phase::productive_context_slot_features(
                        birth.target_feature_mask,
                    ),
                context_positive_support: birth.context_positive_support,
                context_alternative_support: birth.context_unlabeled_alternative_support,
                context_posterior_milli: birth.context_posterior_milli,
                slot_evidence_milli: birth.slot_evidence_milli,
                joint_evidence_milli: birth.joint_evidence_milli,
                generated,
            });
        if surface == token
            || candidates
                .iter()
                .any(|candidate| candidate.surface.eq_ignore_ascii_case(&surface))
        {
            continue;
        }
        candidates.push(crate::nanda_wave::l2::L2ImeWordCandidate {
            surface: surface.clone(),
            kind: seeded_candidate_kind(token, &surface),
            source: crate::nanda_wave::l2::L2ImeWordCandidateSource::LexicalPhase,
            score: u32::from(birth.joint_evidence_milli),
            l1_overlap: seed_surface_overlap(token, &surface),
            l2_overlap: usize::from(birth.context_positive_support > 0)
                + usize::from(birth.family_specificity > 0),
            motif_overlap: seed_surface_motif_overlap(token, &surface),
            usage_prior: 0.0,
            context_prior: if birth.context_positive_support > 0 {
                1.0
            } else {
                0.0
            },
            accepted_count: 0,
            target_evidence: crate::nanda_wave::l2::L2ImeTargetEvidence::None,
            morphology_slots: vec![crate::correction_core::MorphologySlotIdentity {
                domain: crate::correction_core::MorphologySlotIdentityDomain::CanonicalFeature,
                lemma_id: birth.lemma_id,
                slot_id: birth.target_feature_mask,
            }],
        });
        projection.added_surfaces.insert(surface);
    }
    projection
}

#[derive(Default)]
struct ProductiveSurfaceProjection {
    added_surfaces: std::collections::BTreeSet<String>,
    evidence_by_surface: std::collections::BTreeMap<String, Vec<MorphologySlotEvidence>>,
}

fn append_exact_morphology_evidence(
    field: &StandaloneL2Field,
    index: &impl super::productive::ProductiveMorphologySource,
    context: &str,
    candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
    evidence_by_surface: &mut std::collections::BTreeMap<String, Vec<MorphologySlotEvidence>>,
) {
    for candidate in candidates {
        let surface = candidate.surface.to_lowercase();
        let slot_evidence = field.exact_morphology_slot_evidence(index, context, &surface);
        let retained = evidence_by_surface.entry(surface).or_default();
        for evidence in slot_evidence {
            if !retained.iter().any(|existing| {
                existing.lemma_id == evidence.lemma_id
                    && existing.source_feature_mask == evidence.source_feature_mask
                    && existing.target_feature_mask == evidence.target_feature_mask
            }) {
                retained.push(evidence);
            }
        }
    }
}

fn mark_productive_surface_candidates(
    candidates: &mut [UnifiedCorrectionCandidate],
    productive_surfaces: &std::collections::BTreeSet<String>,
    evidence_by_surface: &std::collections::BTreeMap<String, Vec<MorphologySlotEvidence>>,
) {
    for candidate in candidates {
        let Some(surface) = candidate_last_word_lower(candidate) else {
            continue;
        };
        if let Some(evidence) = evidence_by_surface.get(&surface) {
            candidate.extend_morphology_slot_evidence(evidence.iter().copied());
        }
        if !productive_surfaces.contains(&surface) {
            continue;
        }
        candidate.source_id = CANONICAL_L2_PRODUCTIVE_SOURCE_ID.to_string();
        candidate.gate = CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "productive_morphology_requires_l3",
        };
        for evidence in &mut candidate.evidence {
            evidence.source_id = CANONICAL_L2_PRODUCTIVE_SOURCE_ID.to_string();
            evidence.gate = candidate.gate.clone();
        }
    }
}

fn reference_backed_missing_letter_candidates(
    token: &str,
    limit: usize,
) -> Vec<crate::nanda_wave::l2::L2ImeWordCandidate> {
    reference_backed_missing_letter_surfaces(token, limit)
        .into_iter()
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

fn reference_backed_missing_letter_surfaces(token: &str, limit: usize) -> Vec<String> {
    crate::ru_typo::safe_missing_letter_candidates(token)
        .filter(|candidate| {
            crate::russian_lexicon::is_reference_backed_short_passive_participle(candidate)
        })
        .take(limit)
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
        target_evidence: if seed.authority {
            crate::nanda_wave::l2::L2ImeTargetEvidence::CanonicalWinner
        } else {
            crate::nanda_wave::l2::L2ImeTargetEvidence::None
        },
        morphology_slots: super::morphology_slot_identities_for_surface(&seed.surface),
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

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn apply_standalone_l2_field(
    field: &StandaloneL2Field,
    context: &str,
    token: &str,
    surface_candidates: &mut Vec<crate::nanda_wave::l2::L2ImeWordCandidate>,
    l11_geometry_candidates: &[crate::nanda_wave::l2::L2ImeWordCandidate],
    seeds: &[crate::nanda_wave::L11SeedSurface],
    prepared: PreparedCompositionalField,
    settle_winner: bool,
) -> Option<(CanonicalCohortReadout, StandaloneL2Readout)> {
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
        productive_form_births: _,
        broad_duration,
        active_duration,
        form_duration,
        productive_duration: _,
        total_duration: _,
        cache_hit: _,
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
                target_evidence: crate::nanda_wave::l2::L2ImeTargetEvidence::None,
                morphology_slots: super::morphology_slot_identities_for_surface(&surface),
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
                target_evidence: crate::nanda_wave::l2::L2ImeTargetEvidence::None,
                morphology_slots: super::morphology_slot_identities_for_surface(&surface),
            });
        }
    }
    let inverse_ready = std::time::Instant::now();
    if lexical_seeds.is_empty() {
        return None;
    }
    let readout = field.readout_observed(
        context,
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
            target_evidence: crate::nanda_wave::l2::L2ImeTargetEvidence::None,
            morphology_slots: local
                .lemma_ids
                .iter()
                .copied()
                .zip(local.feature_masks.iter().copied())
                .map(
                    |(lemma_id, slot_id)| crate::correction_core::MorphologySlotIdentity {
                        domain:
                            crate::correction_core::MorphologySlotIdentityDomain::CanonicalFeature,
                        lemma_id,
                        slot_id,
                    },
                )
                .collect(),
        });
    }
    let mut retained_surfaces = input_surface_reserve;
    retained_surfaces.extend(surfaces_by_form.values().cloned());
    surface_candidates
        .retain(|candidate| retained_surfaces.contains(&candidate.surface.to_lowercase()));
    let typed_readout = readout.clone();
    let cohort = match readout.verdict {
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
                return Some((readout, typed_readout));
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
                return Some((readout, typed_readout));
            }
            Some(CanonicalCohortReadout::Abstain {
                cohort_surfaces: surfaces_by_form.into_values().collect(),
            })
        }
    }?;
    Some((cohort, typed_readout))
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

    #[test]
    fn canonical_token_preserves_complete_physical_layout_surface() {
        assert_eq!(
            canonical_input_token("введи cj,frf "),
            Some(CanonicalInputToken {
                context_prefix: "введи ",
                surface: "cj,frf",
                kind: CanonicalInputTokenKind::PhysicalLayout,
            })
        );
        assert_eq!(
            canonical_input_token("ytn "),
            Some(CanonicalInputToken {
                context_prefix: "",
                surface: "ytn",
                kind: CanonicalInputTokenKind::PhysicalLayout,
            })
        );
    }

    #[test]
    fn canonical_token_admits_lexically_supported_short_layout_surface() {
        assert_eq!(
            crate::dict::convert("yt", crate::dict::Direction::Us2Ru),
            "не"
        );
        assert_eq!(
            canonical_input_token("yt "),
            Some(CanonicalInputToken {
                context_prefix: "",
                surface: "yt",
                kind: CanonicalInputTokenKind::PhysicalLayout,
            })
        );
    }

    #[test]
    fn canonical_token_preserves_cyrillic_context_identity() {
        assert_eq!(
            canonical_input_token("проверь врмея, "),
            Some(CanonicalInputToken {
                context_prefix: "проверь ",
                surface: "врмея",
                kind: CanonicalInputTokenKind::Cyrillic,
            })
        );
        assert_eq!(
            canonical_input_lexical_parts("проверь врмея, "),
            Some(("проверь ", "врмея"))
        );
    }

    #[test]
    fn canonical_token_rejects_protected_ascii_classes() {
        for text in [
            "https://example.com ",
            "--help ",
            "release-build ",
            "PDF ",
            "Apple ",
        ] {
            assert_eq!(
                canonical_input_token(text),
                None,
                "protected ASCII token reached L1.1: {text:?}"
            );
        }
    }

    #[test]
    fn canonical_token_rejects_known_english_plain_words() {
        assert!(crate::layout_autoswitch::is_known_english_layout_autoswitch_word("hello"));
        assert_eq!(canonical_input_token("hello "), None);
    }

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
            target_evidence: crate::nanda_wave::l2::L2ImeTargetEvidence::None,
            morphology_slots: Vec::new(),
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
    fn canonical_readout_reserves_a_strong_short_left_boundary_candidate() {
        let readout = canonical_text_readout("данорм ");
        let candidate = readout
            .candidates
            .iter()
            .find(|candidate| candidate.replacement == "да норм ")
            .expect("bounded short-left boundary reserve");

        assert_eq!(candidate.origin, CandidateOrigin::Boundary);
        assert_eq!(candidate.source_id, "CanonicalL2FieldBoundary");
        assert_eq!(candidate.error_class, TypingErrorClass::GluedWords);
        assert_eq!(candidate.gate.action, CandidateGateAction::Eligible);
    }

    #[test]
    fn sparse_omission_reserve_survives_below_the_general_frontier() {
        let surfaces = [
            "форма",
            "сигнал",
            "контур",
            "слово",
            "пакет",
            "дерево",
            "сцена",
            "волна",
            "центр",
            "подтвердить",
        ]
        .into_iter()
        .map(|surface| lexical_candidate(surface, 1_000, 1, 1, 1))
        .collect::<Vec<_>>();

        let bounded = bounded_surface_candidates_with_sparse_reserve("подврдить", &surfaces, 8, 2);

        assert_eq!(
            bounded
                .iter()
                .map(|candidate| candidate.surface.as_str())
                .collect::<Vec<_>>(),
            [
                "форма",
                "сигнал",
                "контур",
                "слово",
                "пакет",
                "дерево",
                "сцена",
                "волна",
                "подтвердить",
            ]
        );
        let mut unified = l2_surface_unified_candidates(
            "на компанию Хунлу можем подврдить ",
            "подврдить",
            &bounded,
        );
        apply_authority_to_candidate_lattice(&mut unified, &L2FieldAuthority::Abstain);
        let candidate = unified
            .iter()
            .find(|candidate| candidate.replacement.ends_with("подтвердить "))
            .expect("reserved sparse omission must reach the unified candidate lattice");
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
    fn reference_backed_short_participle_ambiguity_blocks_false_singleton() {
        let mut surfaces = crate::ru_typo::fuzzy_known_word_candidates("подлючен")
            .into_iter()
            .filter(|surface| {
                crate::russian_lexicon::is_reference_backed_short_passive_participle(surface)
                    && crate::text_metrics::damerau_levenshtein("подлючен", surface) == 1
            })
            .collect::<Vec<_>>();
        surfaces.sort();
        surfaces.dedup();
        assert!(surfaces.iter().any(|surface| surface == "подключен"));
        assert!(surfaces.iter().any(|surface| surface == "подлечен"));
        assert!(surfaces.len() >= 2);
        let candidates = surfaces
            .iter()
            .map(|surface| lexical_candidate(surface, 900, 6, 3, 3))
            .collect::<Vec<_>>();

        let readout = retain_reference_backed_geometry_ambiguity(
            "подлючен",
            CanonicalCohortReadout::Winner {
                winner_surface: "подключен".to_string(),
                cohort_surfaces: vec!["подключен".to_string()],
            },
            &candidates,
        );
        let CanonicalCohortReadout::Tied {
            mut cohort_surfaces,
        } = readout
        else {
            panic!("reference-backed one-edit ambiguity must not remain a singleton");
        };
        cohort_surfaces.sort();
        assert!(cohort_surfaces.iter().any(|surface| surface == "подключен"));
        assert!(cohort_surfaces.iter().any(|surface| surface == "подлечен"));
        assert!(cohort_surfaces.len() >= 2);

        let mut unified = l2_surface_unified_candidates("подлючен ", "подлючен", &candidates);
        demote_canonical_local_surface_cohort(
            &mut unified,
            &CanonicalCohortReadout::Tied { cohort_surfaces },
        );
        assert_eq!(unified.len(), candidates.len());
        assert!(unified
            .iter()
            .all(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly));
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

    fn morphology_evidence(
        lemma_id: u32,
        target_feature_mask: u32,
        generated: bool,
    ) -> MorphologySlotEvidence {
        MorphologySlotEvidence {
            lemma_id,
            source_feature_mask: 1,
            target_feature_mask,
            context_positive_support: 4,
            context_alternative_support: 1,
            context_posterior_milli: 800,
            slot_evidence_milli: 600,
            joint_evidence_milli: 900,
            generated,
        }
    }

    #[test]
    fn typed_morphology_evidence_reaches_exact_and_generated_surfaces() {
        let mut candidates = vec![
            unified_candidate(
                "вы принуждаете ",
                CandidateOrigin::L2Surface,
                CANONICAL_L2_SURFACE_SOURCE_ID,
            ),
            unified_candidate(
                "вы принуждаетеся ",
                CandidateOrigin::L2Surface,
                CANONICAL_L2_SURFACE_SOURCE_ID,
            ),
        ];
        let productive_surfaces = ["принуждаетеся".to_string()].into_iter().collect();
        let evidence_by_surface = [
            (
                "принуждаете".to_string(),
                vec![morphology_evidence(17, 10, false)],
            ),
            (
                "принуждаетеся".to_string(),
                vec![morphology_evidence(17, 11, true)],
            ),
        ]
        .into_iter()
        .collect();

        mark_productive_surface_candidates(
            &mut candidates,
            &productive_surfaces,
            &evidence_by_surface,
        );

        assert_eq!(candidates[0].source_id, CANONICAL_L2_SURFACE_SOURCE_ID);
        assert_eq!(candidates[0].morphology_slot_evidence.len(), 1);
        assert!(!candidates[0].morphology_slot_evidence[0].generated);
        assert_eq!(candidates[1].source_id, CANONICAL_L2_PRODUCTIVE_SOURCE_ID);
        assert_eq!(candidates[1].gate.action, CandidateGateAction::SuggestOnly);
        assert!(candidates[1].morphology_slot_evidence[0].generated);
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
