//! Portable COMPLETED_101 ordering over already admitted native surfaces.
//!
//! Native scores and the canonical full-mask dictionary are a new provider
//! scope, not the research LightGBM/OLDTRAIN/OpenCorpora quality receipt. This
//! module cannot admit a surface, alter a score, or turn Keep into Edit.

use super::*;
use crate::correction_core::{MorphologySlotIdentity, MorphologySlotIdentityDomain};
use std::collections::BTreeSet;

const MAX_CANDIDATES: usize = 64;
const MAX_READINGS: usize = 64;
const MAX_SURFACE_CHARS: usize = 64;
const LEFT_WINDOW: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Verb,
    Adjective,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Signature {
    PersonNumber(u8, u8),
    CaseNumberGender(u8, u8, u8),
}

#[derive(Clone, Copy)]
struct Reading {
    family: (u32, u16),
    verb: Option<Signature>,
    adjective: Option<Signature>,
    subject: Option<Signature>,
}

fn readings(identities: Vec<MorphologySlotIdentity>) -> Option<Vec<Reading>> {
    if identities.len() > MAX_READINGS {
        return None;
    }
    identities
        .into_iter()
        .map(|identity| {
            // Productive slot IDs must never be interpreted as feature bits.
            if identity.domain != MorphologySlotIdentityDomain::CanonicalFeature {
                return None;
            }
            let labels =
                crate::nanda_wave::l2_field::canonical_morphology_feature_labels(identity.slot_id)
                    .ok()?;
            let has = |label| labels.contains(&label);
            let number = if has("sg") {
                Some(1)
            } else if has("pl") {
                Some(2)
            } else {
                None
            };
            let person = if has("p1") {
                Some(1)
            } else if has("p2") {
                Some(2)
            } else if has("p3") {
                Some(3)
            } else {
                None
            };
            let case = ["nom", "gen", "dat", "acc", "ins", "prep", "voc"]
                .iter()
                .position(|label| has(label))
                .map(|i| i as u8 + 1)
                .or_else(|| {
                    if has("part") {
                        Some(2)
                    } else if has("loc2") {
                        Some(6)
                    } else {
                        None
                    }
                });
            let gender = if number == Some(2) {
                Some(0)
            } else if has("masc") {
                Some(1)
            } else if has("fem") {
                Some(2)
            } else if has("neut") {
                Some(3)
            } else {
                None
            };
            let pos =
                crate::nanda_wave::l2_field::canonical_morphology_primary_pos(identity.slot_id);
            let agreement = || Some(Signature::CaseNumberGender(case?, number?, gender?));
            Some(Reading {
                family: (identity.lemma_id, pos),
                verb: if pos == 2 && !has("inf") && !has("ger") {
                    person
                        .zip(number)
                        .map(|(p, n)| Signature::PersonNumber(p, n))
                } else {
                    None
                },
                adjective: if pos == 3 { agreement() } else { None },
                subject: if matches!(pos, 1 | 4) && case == Some(1) {
                    number.map(|n| {
                        Signature::PersonNumber(if pos == 4 { person.unwrap_or(3) } else { 3 }, n)
                    })
                } else {
                    None
                },
            })
        })
        .collect()
}

fn signatures(
    readings: &[Reading],
    role: Role,
    families: &BTreeSet<(u32, u16)>,
) -> BTreeSet<Signature> {
    readings
        .iter()
        .filter(|r| families.contains(&r.family))
        .filter_map(|r| match role {
            Role::Verb => r.verb,
            Role::Adjective => r.adjective,
        })
        .collect()
}

fn is_personal(signatures: &BTreeSet<Signature>) -> bool {
    // Literal research exception: any retained PN reading, not a new
    // same-reading/subject claim. Its original ambiguity limit is preserved.
    signatures
        .iter()
        .any(|s| matches!(s, Signature::PersonNumber(1 | 2, _)))
}

struct OptionReading {
    index: usize,
    score: f32,
    readings: Vec<Reading>,
}

/// The pure policy sees observed readings, original scores and partner
/// signatures only. Gold roles, reference surfaces and outcomes are absent.
fn choose(
    source: &[Reading],
    options: &[OptionReading],
    verb_partners: &[BTreeSet<Signature>],
    adjective_partners: &[BTreeSet<Signature>],
) -> (Option<usize>, &'static str) {
    let families = source.iter().map(|r| r.family).collect();
    let verb = signatures(source, Role::Verb, &families);
    let adjective = signatures(source, Role::Adjective, &families);
    let (role, partners) = match (verb.is_empty(), adjective.is_empty()) {
        (false, true) => (Role::Verb, verb_partners),
        (true, false) => (Role::Adjective, adjective_partners),
        _ => return (None, "agreement_unknown_source_role"),
    };
    let Some(nearest) = partners.first() else {
        return (None, "agreement_no_observed_partner");
    };
    let Some(old) = options.iter().max_by(|a, b| a.score.total_cmp(&b.score)) else {
        return (None, "agreement_no_admitted_option");
    };
    if options.iter().filter(|o| o.score == old.score).count() != 1 {
        return (None, "agreement_tied_old_top");
    }
    let option_signatures = options
        .iter()
        .map(|o| signatures(&o.readings, role, &families))
        .collect::<Vec<_>>();
    // C remains the legacy singleton-candidate AND singleton-nearest gate.
    let supported = options
        .iter()
        .zip(&option_signatures)
        .filter(|(_, s)| nearest.len() == 1 && s.len() == 1 && !s.is_disjoint(nearest))
        .collect::<Vec<_>>();
    let Some((proposed, proposed_signatures)) = supported
        .iter()
        .max_by(|a, b| a.0.score.total_cmp(&b.0.score))
        .copied()
    else {
        return (None, "agreement_no_supported_option");
    };
    if supported
        .iter()
        .filter(|(o, _)| o.score == proposed.score)
        .count()
        != 1
    {
        return (None, "agreement_tied_supported_top");
    }
    if proposed.index == old.index {
        return (None, "agreement_old_top_supported");
    }
    let old_signatures = &option_signatures[options
        .iter()
        .position(|o| o.index == old.index)
        .expect("old option retained")];
    let personal =
        role == Role::Verb && is_personal(proposed_signatures) && !is_personal(old_signatures);
    if partners.iter().any(|p| !old_signatures.is_disjoint(p)) && !personal {
        return (None, "agreement_old_top_has_partner_support");
    }
    (
        Some(proposed.index),
        "agreement_compatible_signature_preferred",
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AgreementOrder {
    pub(super) selected: Option<usize>,
    pub(super) reason: &'static str,
}

fn boundary(text: &str) -> bool {
    text.chars().any(|c| ",.;:!?—–()[]{}".contains(c))
}

fn bounded_surface(surface: &str) -> bool {
    !surface.is_empty()
        && surface.chars().take(MAX_SURFACE_CHARS + 1).count() <= MAX_SURFACE_CHARS
        && surface.chars().all(is_cyrillic_letter)
}

pub(super) fn select(
    event: &TypingErrorEvent,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
    admissions: &[Option<AuthorityLaneAdmission>],
) -> AgreementOrder {
    select_with_readings(event, candidates, evaluations, admissions, |surface| {
        crate::nanda_wave::l2_field::cached_morphology_slot_identities_for_surface(
            surface,
            MAX_READINGS,
        )
    })
}

fn select_with_readings(
    event: &TypingErrorEvent,
    candidates: &[UnifiedCorrectionCandidate],
    evaluations: &[CandidateDecisionEvaluation],
    admissions: &[Option<AuthorityLaneAdmission>],
    mut lookup: impl FnMut(&str) -> Option<Vec<MorphologySlotIdentity>>,
) -> AgreementOrder {
    let original = admissions
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.as_ref().map(|_| i))
        .max_by(|a, b| compare_candidate_decision_order(*a, *b, candidates, evaluations));
    let unchanged = |reason| AgreementOrder {
        selected: original,
        reason,
    };
    if candidates.len() > MAX_CANDIDATES {
        return unchanged("agreement_work_budget");
    }
    let admitted = admissions
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.as_ref().map(|_| i))
        .collect::<Vec<_>>();
    if admitted.len() < 2 {
        return unchanged("agreement_fewer_than_two_admissions");
    }
    let Some((prefix, token)) = crate::word_reader::split_last_trimmed_ws_token(&event.original)
    else {
        return unchanged("agreement_unknown_focus");
    };
    let (leading, source_surface, trailing) = split_word_punctuation(token);
    let (_, current_word, _) = split_word_punctuation(&event.current_word);
    if current_word != source_surface || !bounded_surface(source_surface) || boundary(leading) {
        return unchanged("agreement_unknown_focus");
    }
    let Some(source) = lookup(&source_surface.to_lowercase()).and_then(readings) else {
        return unchanged("agreement_provider_unavailable");
    };
    let mut options = Vec::with_capacity(admitted.len());
    for index in admitted {
        let Some((candidate_prefix, candidate_token)) =
            crate::word_reader::split_last_trimmed_ws_token(&candidates[index].replacement)
        else {
            return unchanged("agreement_not_current_word_replacement");
        };
        let (candidate_leading, surface, candidate_trailing) =
            split_word_punctuation(candidate_token);
        if candidate_prefix != prefix
            || candidate_leading != leading
            || candidate_trailing != trailing
            || !bounded_surface(surface)
            || surface == source_surface
        {
            return unchanged("agreement_not_current_word_replacement");
        }
        let score = evaluations[index].signals.rank_score;
        if !score.is_finite() {
            return unchanged("agreement_nonfinite_score");
        }
        let Some(readings) = lookup(&surface.to_lowercase()).and_then(readings) else {
            return unchanged("agreement_provider_unavailable");
        };
        options.push(OptionReading {
            index,
            score,
            readings,
        });
    }
    let mut verb_partners = Vec::new();
    for token in prefix.split_whitespace().rev().take(LEFT_WINDOW) {
        // Attached punctuation is a real boundary, not a reason to invent an
        // adjacent subject. Never reach around it to an earlier token.
        if boundary(token) {
            break;
        }
        let (_, surface, _) = split_word_punctuation(token);
        if !bounded_surface(surface) {
            return unchanged("agreement_unknown_context_token");
        }
        let Some(readings) = lookup(&surface.to_lowercase()).and_then(readings) else {
            return unchanged("agreement_provider_unavailable");
        };
        let signatures = readings
            .iter()
            .filter_map(|r| r.subject)
            .collect::<BTreeSet<_>>();
        if !signatures.is_empty() {
            verb_partners.push(signatures);
        }
    }
    // The actual correction event focuses its last token. Thus there are no
    // observed +1..+4 noun partners: the adjective arm must remain unknown.
    let (preferred, reason) = choose(&source, &options, &verb_partners, &[]);
    AgreementOrder {
        selected: preferred.or(original),
        reason,
    }
}

#[cfg(test)]
mod tests;
