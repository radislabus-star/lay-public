use std::collections::{BTreeMap, BTreeSet};

use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::{
    CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, MorphologySlotEvidence,
    TypingErrorClass, UnifiedCorrectionCandidate,
};
use crate::nanda_wave::l2_field::runtime::{
    CanonicalL2FieldReadout, L2FieldAuthority, StandaloneL2Field,
};
use crate::nanda_wave::lexical_grokking::restoration::{
    AbstainReason, RestorationCandidate, RestorationEvidence, RestorationReadout,
};
use crate::nanda_wave::L11SeedSurface;
use crate::text_case::apply_word_case;
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::replace_last_text_word;

use super::calibrate::{CandidateProvenanceClassV1, ProductiveCalibratedVerdictV1};
use super::composite::{CompositeGroundedVerdictV1, CompositeL2LatticeV1};
use super::packaged_runtime::{
    PackagedGroundedLemmaV1, PackagedProductiveCandidateV1, PackagedProductiveRuntimeV1,
};
use super::scene::{BoundaryKindV1, L2LocalSceneV1, LocalTokenObservationV1};

pub(super) const PRODUCTIVE_V90_SURFACE_SOURCE_ID: &str = "ProductiveL2V90Surface";
pub(super) const PRODUCTIVE_V90_GROUNDED_SOURCE_ID: &str = "ProductiveL2V90Grounded";
pub(super) const PRODUCTIVE_V90_GROUNDED_WINNER_SOURCE_ID: &str = "ProductiveL2V90GroundedWinner";
const MAX_ACTIVE_PACKAGE_LEMMAS: usize = 32;

/// The only live L2 owner. Canonical L2 is used as a read-only identity index;
/// its historical local verdict is deliberately absent from this path.
pub(in crate::nanda_wave::l2_field) fn live_productive_v1_readout(
    original: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
    runtime: &PackagedProductiveRuntimeV1,
    l11_seeds: &[L11SeedSurface],
) -> Result<CanonicalL2FieldReadout, String> {
    let restoration = l11_restoration_readout(observed, l11_seeds);
    let groundings = package_known_groundings(canonical_index, runtime, l11_seeds)?;
    let scene = live_scene(original, observed, canonical_index);
    let grounded_winner_present = matches!(restoration, RestorationReadout::Winner { .. });
    let productive = runtime.evaluate_shadow_with_cold_bindings(
        observed,
        &scene,
        &groundings,
        &[],
        grounded_winner_present,
    );
    if let Some(error) = productive.integrity_error.as_deref() {
        return Err(format!("productive V90 integrity error: {error}"));
    }

    let surface_by_terminal = l11_seeds
        .iter()
        .filter_map(|seed| {
            seed.terminal_id
                .map(|terminal_id| (terminal_id, seed.surface.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let lattice = CompositeL2LatticeV1::assemble(
        &restoration,
        |terminal_id| surface_by_terminal.get(&terminal_id).cloned(),
        productive,
        None,
    )?;
    if !lattice.grounded_winner_is_preserved() {
        return Err("productive V90 dropped the grounded L1.1 winner".to_string());
    }

    let common_l3_required = productive_surface_count(&lattice) > 1;
    let authority = live_authority(&lattice, common_l3_required);
    let candidates = materialize_live_candidates(original, observed, &lattice, common_l3_required)?;
    Ok(CanonicalL2FieldReadout::new(candidates, authority))
}

fn materialize_live_candidates(
    original: &str,
    observed: &str,
    lattice: &CompositeL2LatticeV1,
    common_l3_required: bool,
) -> Result<Vec<UnifiedCorrectionCandidate>, String> {
    let protected_surface = lattice
        .grounded_candidates
        .iter()
        .find(|candidate| candidate.protected_winner)
        .map(|candidate| candidate.normalized_surface.as_str());
    let productive_winner = match (&lattice.productive_verdict, common_l3_required) {
        (ProductiveCalibratedVerdictV1::Winner { candidate, .. }, false) => {
            Some(candidate.normalized_surface.as_str())
        }
        (ProductiveCalibratedVerdictV1::Winner { .. }, true)
        | (ProductiveCalibratedVerdictV1::Tied { .. }, _)
        | (ProductiveCalibratedVerdictV1::Abstain { .. }, _) => None,
    };
    let productive_by_surface = lattice.productive_candidates.iter().fold(
        BTreeMap::<&str, Vec<&PackagedProductiveCandidateV1>>::new(),
        |mut map, candidate| {
            map.entry(candidate.normalized_surface.as_ref())
                .or_default()
                .push(candidate);
            map
        },
    );
    let grounded_lemmas = lattice
        .productive_candidates
        .iter()
        .filter(|candidate| candidate.grounded_support > 0)
        .map(|candidate| candidate.identity.lemma_id)
        .collect::<BTreeSet<_>>();

    let mut candidates = Vec::with_capacity(lattice.surface_groups.len());
    for group in &lattice.surface_groups {
        if group.normalized_surface.eq_ignore_ascii_case(observed) {
            continue;
        }
        let projected = apply_word_case(observed, &group.normalized_surface);
        let replacement = replace_last_text_word(original, &projected)
            .ok_or_else(|| "productive V90 cannot replace the active word".to_string())?;
        let productive_nodes = productive_by_surface
            .get(group.normalized_surface.as_str())
            .cloned()
            .unwrap_or_default();
        let same_lemma_slot = productive_nodes.iter().any(|candidate| {
            candidate
                .equivalent_identities
                .iter()
                .any(|identity| grounded_lemmas.contains(&identity.lemma_id))
        });
        let declared_class = if same_lemma_slot {
            TypingErrorClass::GrammarAgreement
        } else {
            TypingErrorClass::Unknown
        };
        let error_class = action_operator::classify_token_transition(
            original,
            &replacement,
            CandidateOrigin::L2Surface,
            declared_class,
        );
        let mut gate = TransitionDecisionCore::admit_candidate_proposal(
            original,
            &replacement,
            error_class,
            CandidateOrigin::L2Surface,
        );
        let is_protected = protected_surface == Some(group.normalized_surface.as_str());
        let productive_has_l2_winner =
            productive_winner == Some(group.normalized_surface.as_str()) && !is_protected;
        if !is_protected
            && !productive_nodes.is_empty()
            && !productive_has_l2_winner
            && gate.action == CandidateGateAction::Eligible
        {
            gate = CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "productive_v90_lattice_requires_common_l3",
            };
        }
        let source_id = if is_protected {
            PRODUCTIVE_V90_GROUNDED_WINNER_SOURCE_ID
        } else if !productive_nodes.is_empty() {
            PRODUCTIVE_V90_SURFACE_SOURCE_ID
        } else {
            PRODUCTIVE_V90_GROUNDED_SOURCE_ID
        };
        let mut candidate = UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Nanda,
            CandidateOrigin::L2Surface,
            source_id,
            error_class,
            gate,
        );
        candidate.extend_morphology_slot_evidence(productive_slot_evidence(
            &productive_nodes,
            productive_winner,
        ));
        candidates.push(candidate);
    }
    Ok(candidates)
}

fn productive_slot_evidence(
    candidates: &[&PackagedProductiveCandidateV1],
    productive_winner: Option<&str>,
) -> Vec<MorphologySlotEvidence> {
    let mut identities = BTreeSet::new();
    let mut evidence = Vec::new();
    for candidate in candidates {
        let selected = productive_winner == Some(candidate.normalized_surface.as_ref());
        for identity in &candidate.equivalent_identities {
            if !identities.insert((identity.lemma_id, identity.target_slot_id)) {
                continue;
            }
            evidence.push(MorphologySlotEvidence {
                lemma_id: identity.lemma_id,
                source_feature_mask: 0,
                target_feature_mask: identity.target_slot_id,
                context_positive_support: if selected {
                    candidate.grounded_support.max(1)
                } else {
                    0
                },
                context_alternative_support: if selected { 0 } else { 1 },
                context_posterior_milli: if selected { 1_000 } else { 0 },
                slot_evidence_milli: if selected { 1_000 } else { 0 },
                joint_evidence_milli: if selected { 1_000 } else { 0 },
                generated: candidate.provenance != CandidateProvenanceClassV1::Exact,
            });
        }
    }
    evidence
}

fn productive_surface_count(lattice: &CompositeL2LatticeV1) -> usize {
    lattice
        .productive_candidates
        .iter()
        .map(|candidate| candidate.normalized_surface.as_ref())
        .collect::<BTreeSet<_>>()
        .len()
}

fn live_authority(lattice: &CompositeL2LatticeV1, common_l3_required: bool) -> L2FieldAuthority {
    if let CompositeGroundedVerdictV1::Winner { terminal_id } = lattice.original_l11_verdict {
        if let Some(surface) = lattice
            .grounded_candidates
            .iter()
            .find(|candidate| candidate.candidate.terminal_id == terminal_id)
            .map(|candidate| candidate.normalized_surface.clone())
        {
            return L2FieldAuthority::Winner { surface };
        }
    }
    if common_l3_required {
        return L2FieldAuthority::Tied {
            surfaces: lattice
                .surface_groups
                .iter()
                .map(|group| group.normalized_surface.clone())
                .collect(),
        };
    }
    match &lattice.productive_verdict {
        ProductiveCalibratedVerdictV1::Winner { candidate, .. } => L2FieldAuthority::Winner {
            surface: candidate.normalized_surface.clone(),
        },
        ProductiveCalibratedVerdictV1::Tied { candidates, .. } => L2FieldAuthority::Tied {
            surfaces: candidates
                .iter()
                .map(|candidate| candidate.normalized_surface.clone())
                .collect(),
        },
        ProductiveCalibratedVerdictV1::Abstain { .. } => {
            let surfaces = lattice
                .grounded_candidates
                .iter()
                .map(|candidate| candidate.normalized_surface.clone())
                .collect::<Vec<_>>();
            if matches!(
                lattice.original_l11_verdict,
                CompositeGroundedVerdictV1::Tied { .. }
                    | CompositeGroundedVerdictV1::TiedOverflow { .. }
            ) && !surfaces.is_empty()
            {
                L2FieldAuthority::Tied { surfaces }
            } else {
                L2FieldAuthority::Abstain
            }
        }
    }
}

fn package_known_groundings(
    canonical_index: &StandaloneL2Field,
    runtime: &PackagedProductiveRuntimeV1,
    seeds: &[L11SeedSurface],
) -> Result<Vec<PackagedGroundedLemmaV1>, String> {
    let mut evidence_by_lemma = BTreeMap::<u32, u32>::new();
    for seed in seeds {
        let Some(form_ref) = canonical_index.form_ref_for_surface(&seed.surface) else {
            continue;
        };
        for (lemma_id, _) in canonical_index.imported_binding_identities_for_form(form_ref) {
            evidence_by_lemma
                .entry(lemma_id)
                .and_modify(|evidence| *evidence = (*evidence).max(seed.score_milli.max(1)))
                .or_insert(seed.score_milli.max(1));
        }
    }
    let mut ranked = evidence_by_lemma.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    ranked.truncate(MAX_ACTIVE_PACKAGE_LEMMAS);

    let mut grounded = Vec::new();
    for (lemma_id, seed_support) in ranked {
        for descriptor in runtime.grounding_descriptors(lemma_id)? {
            let normalized_source = canonical_index
                .imported_surface_for_form(descriptor.canonical_source_form_ref)
                .ok_or_else(|| {
                    "productive V90 grounding lacks its canonical source surface".to_string()
                })?;
            grounded.push(PackagedGroundedLemmaV1 {
                lemma_id: descriptor.lemma_id,
                pos_domain: descriptor.pos_domain,
                canonical_source_form_ref: descriptor.canonical_source_form_ref,
                source_slot_id: descriptor.source_slot_id,
                normalized_source,
                grounded_support: descriptor.grounded_support.max(seed_support),
            });
        }
    }
    grounded.sort_by(|left, right| {
        (left.lemma_id, left.pos_domain, left.source_slot_id).cmp(&(
            right.lemma_id,
            right.pos_domain,
            right.source_slot_id,
        ))
    });
    grounded.dedup_by(|left, right| {
        (left.lemma_id, left.pos_domain) == (right.lemma_id, right.pos_domain)
    });
    Ok(grounded)
}

fn l11_restoration_readout(observed: &str, seeds: &[L11SeedSurface]) -> RestorationReadout {
    let mut seen = BTreeSet::new();
    let candidates = seeds
        .iter()
        .filter_map(|seed| {
            let terminal_id = seed.terminal_id?;
            seen.insert(terminal_id).then_some(RestorationCandidate {
                terminal_id,
                evidence: RestorationEvidence {
                    geometry_distance: crate::text_metrics::damerau_levenshtein(
                        observed,
                        &seed.surface,
                    )
                    .min(u8::MAX as usize) as u8,
                    positive_milli: seed.score_milli.min(u32::from(u16::MAX)) as u16,
                    backward_milli: seed.score_milli.min(u32::from(u16::MAX)) as u16,
                    ..RestorationEvidence::default()
                },
            })
        })
        .collect::<Vec<_>>();
    let authoritative = seeds
        .iter()
        .filter(|seed| seed.authority)
        .filter_map(|seed| seed.terminal_id)
        .collect::<BTreeSet<_>>();
    if authoritative.len() == 1 {
        let terminal_id = *authoritative.first().expect("one authoritative terminal");
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| candidate.terminal_id == terminal_id)
        {
            return RestorationReadout::Winner {
                candidate: *candidate,
            };
        }
    }
    let geometry_distance = candidates
        .iter()
        .map(|candidate| candidate.evidence.geometry_distance)
        .min();
    if candidates.len() >= 2 {
        RestorationReadout::Tied {
            geometry_distance: geometry_distance.unwrap_or_default(),
            candidates,
        }
    } else {
        RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance,
            candidates,
        }
    }
}

fn live_scene(
    original: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
) -> L2LocalSceneV1 {
    let context = crate::word_reader::split_last_alphabetic_token(original)
        .map(|(context, _)| context)
        .unwrap_or_default();
    let left = context
        .split_whitespace()
        .rev()
        .filter_map(normalize_context_token)
        .take(2)
        .collect::<Vec<_>>();
    let token_observation = |surface: Option<String>| {
        surface.map(|normalized_surface| {
            let lemma_ids = canonical_index
                .form_ref_for_surface(&normalized_surface)
                .into_iter()
                .flat_map(|form_ref| canonical_index.imported_binding_identities_for_form(form_ref))
                .map(|(lemma_id, _)| lemma_id)
                .collect::<BTreeSet<_>>();
            LocalTokenObservationV1 {
                normalized_surface,
                lemma_id: (lemma_ids.len() == 1)
                    .then(|| *lemma_ids.first().expect("one contextual lemma")),
                morphology_slot: None,
            }
        })
    };
    L2LocalSceneV1 {
        current_token: observed.to_string(),
        current_normalized_scalars: observed.chars().map(u32::from).collect(),
        left_tokens: [
            token_observation(left.get(1).cloned()),
            token_observation(left.first().cloned()),
        ],
        boundary_before: if context.trim().is_empty() {
            BoundaryKindV1::None
        } else {
            BoundaryKindV1::Token
        },
        ..L2LocalSceneV1::default()
    }
}

fn normalize_context_token(token: &str) -> Option<String> {
    let normalized = token
        .trim_matches(|character: char| !character.is_alphanumeric() && character != '-')
        .to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::super::calibrate::{
        CandidateProvenanceClassV1, CandidateRankOriginV1, ReadoutCandidateV1,
    };
    use super::super::geometry::GeometryTerminalEvidenceV1;
    use super::super::packaged_runtime::PackagedProductiveReadoutV1;
    use super::super::types::ProductiveCandidateIdentityV1;
    use super::*;

    fn productive_candidate(
        lemma_id: u32,
        target_slot_id: u32,
        normalized_surface_id: u32,
        surface: &str,
    ) -> PackagedProductiveCandidateV1 {
        let identity = ProductiveCandidateIdentityV1 {
            lemma_id,
            paradigm_id: 11,
            program_id: target_slot_id,
            target_slot_id,
            normalized_surface_id,
            variant_id: 1,
        };
        PackagedProductiveCandidateV1 {
            identity,
            equivalent_identities: vec![identity],
            normalized_surface: surface.into(),
            score_q16: 1_000 - i64::from(target_slot_id),
            geometry: GeometryTerminalEvidenceV1::default(),
            provenance: CandidateProvenanceClassV1::TrainingSeenGenerated,
            minimum_independent_support: 2,
            grounded_support: 2,
            ambiguity_center_cosine: 0,
            equivalent_identity_count: 1,
            equivalent_paradigm_count: 1,
            minimum_equivalent_support: 2,
            maximum_equivalent_support: 2,
            rank_origin: CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        }
    }

    fn readout_candidate(candidate: &PackagedProductiveCandidateV1) -> ReadoutCandidateV1 {
        ReadoutCandidateV1 {
            identity: candidate.identity,
            equivalent_identities: candidate.equivalent_identities.clone(),
            normalized_surface: candidate.normalized_surface.to_string(),
            score_q16: candidate.score_q16,
            grounded_lemma_evidence: candidate.grounded_support,
            exact_osa_distance: 0,
            exact_form: false,
            cross_lemma_ownership_satisfied: false,
            rank_origin: candidate.rank_origin,
            cross_lane_certified: candidate.cross_lane_certified,
        }
    }

    #[test]
    fn l11_authority_is_derived_from_typed_seed_not_candidate_order() {
        let readout = l11_restoration_readout(
            "проврека",
            &[
                L11SeedSurface {
                    terminal_id: Some(3),
                    surface: "проверка".to_string(),
                    authority: false,
                    score_milli: 900,
                },
                L11SeedSurface {
                    terminal_id: Some(7),
                    surface: "проврека".to_string(),
                    authority: true,
                    score_milli: 800,
                },
            ],
        );
        assert!(matches!(
            readout,
            RestorationReadout::Winner {
                candidate: RestorationCandidate { terminal_id: 7, .. }
            }
        ));
    }

    #[test]
    fn non_authoritative_single_seed_remains_abstain() {
        let readout = l11_restoration_readout(
            "форма",
            &[L11SeedSurface {
                terminal_id: Some(3),
                surface: "формы".to_string(),
                authority: false,
                score_milli: 700,
            }],
        );
        assert!(matches!(readout, RestorationReadout::Abstain { .. }));
    }

    #[test]
    fn multiple_productive_surfaces_defer_slot_selection_to_common_l3() {
        let nominative = productive_candidate(17, 1, 101, "форма");
        let genitive = productive_candidate(17, 2, 102, "формы");
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&nominative),
                calibration_stratum_id: 1,
            },
            candidates: vec![nominative, genitive],
            logical_terminal_count: 2,
            logical_surface_basin_count: 2,
            integrity_error: None,
        };
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("two-slot productive lattice");

        let common_l3_required = productive_surface_count(&lattice) > 1;
        assert!(common_l3_required);
        assert!(matches!(
            live_authority(&lattice, common_l3_required),
            L2FieldAuthority::Tied { ref surfaces }
                if surfaces == &["форма".to_string(), "формы".to_string()]
        ));

        let candidates =
            materialize_live_candidates("нужна форм", "форм", &lattice, common_l3_required)
                .expect("common L3 candidates");
        assert_eq!(candidates.len(), 2);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly));
        assert!(candidates.iter().all(|candidate| {
            candidate
                .morphology_slot_evidence
                .iter()
                .any(|evidence| evidence.lemma_id == 17)
        }));
    }
}
