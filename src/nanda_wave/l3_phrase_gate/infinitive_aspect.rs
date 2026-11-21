use super::{
    context_certificate_report, ContextCertificate, L3PhraseGateDecision, L3PhraseGateReport,
};
use crate::keyboard::is_cyrillic_letter;
use crate::nanda_wave::l2_field::{
    cached_morphology_slot_identities_for_surface, canonical_morphology_feature_labels,
    canonical_morphology_primary_pos,
};
use crate::russian_lexicon::has_clean_russian_surface_certificate;
use crate::russian_typo_candidates::inserted_char_position_for_missing_letter;
use std::collections::BTreeSet;

const PROFILES: &str = include_str!("../../../data/lexicon/russian_infinitive_aspect_context.tsv");
const MAX_READINGS: usize = 16;

pub(super) fn apply(
    original: &str,
    replacements: &[&str],
    reports: &mut [Option<L3PhraseGateReport>],
) {
    if replacements.len() != reports.len()
        || reports
            .iter()
            .flatten()
            .any(|report| report.decision != L3PhraseGateDecision::Neutral)
    {
        return;
    }
    let trimmed = original.trim_end();
    let Some((prefix, damaged)) = trimmed.rsplit_once(' ') else {
        return;
    };
    let left = prefix.trim_end_matches(' ');
    if left.ends_with(char::is_whitespace) {
        return;
    }
    let mut left_words = left
        .rsplit(['\n', '\r', '\t'])
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .rev();
    let Some(governor) = left_words.next() else {
        return;
    };
    let subject = left_words.next();
    let damaged_len = damaged.chars().count();
    if !(4..=32).contains(&damaged_len)
        || !damaged.chars().all(is_cyrillic_letter)
        || !governor.chars().all(is_cyrillic_letter)
    {
        return;
    }
    let stable_prefix = &trimmed[..trimmed.len() - damaged.len()];
    let mut restored = BTreeSet::new();
    for replacement in replacements {
        let Some(word) = replacement.trim_end().strip_prefix(stable_prefix) else {
            continue;
        };
        if word.chars().count() != damaged_len + 1 || !word.chars().all(is_cyrillic_letter) {
            continue;
        }
        if inserted_char_position_for_missing_letter(damaged, word)
            .is_some_and(|(position, _)| position > 0 && position < damaged_len)
        {
            restored.insert(word);
            if restored.len() > 2 {
                return;
            }
        }
    }
    if restored.len() != 2 || has_clean_russian_surface_certificate(&damaged.to_lowercase()) {
        return;
    }
    let Some((expected, support)) = governor_profile(&governor.to_lowercase(), subject) else {
        return;
    };
    let mut supported = restored.into_iter().map(|word| {
        let aspect = canonical_infinitive_aspect(&word.to_lowercase())?;
        Some((word, aspect == expected))
    });
    let (Some((first, first_supported)), Some((second, second_supported))) =
        (supported.next().flatten(), supported.next().flatten())
    else {
        return;
    };
    if first_supported == second_supported {
        return;
    }
    let winner = if first_supported { first } else { second };
    for (replacement, report) in replacements.iter().zip(reports) {
        if replacement.trim_end().strip_prefix(stable_prefix) == Some(winner) {
            *report = Some(context_certificate_report(
                report.as_ref(),
                support,
                1,
                ContextCertificate::InfinitiveAspect,
            ));
        }
    }
}

fn governor_profile(surface: &str, subject: Option<&str>) -> Option<(&'static str, usize)> {
    let lemma_id = governor_lemma(surface, subject)?;
    PROFILES.lines().find_map(|line| {
        let (lemma, profile) = line.split_once('\t')?;
        let (aspect, support) = profile.split_once('\t')?;
        if unambiguous_verb_lemma(lemma) != Some(lemma_id) {
            return None;
        }
        Some((aspect, support.parse().ok()?))
    })
}

fn unambiguous_verb_lemma(surface: &str) -> Option<u32> {
    let readings = cached_morphology_slot_identities_for_surface(surface, MAX_READINGS)?;
    let lemma = readings.first()?.lemma_id;
    readings
        .iter()
        .all(|reading| {
            reading.lemma_id == lemma && canonical_morphology_primary_pos(reading.slot_id) == 2
        })
        .then_some(lemma)
}

// V13 can encode a conjugated form and a short participle under one lemma.
// A plural personal subject can resolve this specific disagreement. Other
// homographs stay unresolved.
fn governor_lemma(surface: &str, subject: Option<&str>) -> Option<u32> {
    let readings = cached_morphology_slot_identities_for_surface(surface, MAX_READINGS)?;
    let lemma = readings.first()?.lemma_id;
    if readings.iter().any(|reading| reading.lemma_id != lemma) {
        return None;
    }
    if readings
        .iter()
        .all(|reading| canonical_morphology_primary_pos(reading.slot_id) == 2)
    {
        return Some(lemma);
    }

    let person = plural_personal_subject(&subject?.to_lowercase())?;
    let mut agrees = false;
    for reading in readings {
        let pos = canonical_morphology_primary_pos(reading.slot_id);
        let labels = canonical_morphology_feature_labels(reading.slot_id).ok()?;
        match pos {
            2 => agrees |= labels.contains(&"pl") && labels.contains(&person),
            3 if labels.contains(&"short") && labels.contains(&"sg") => {}
            _ => return None,
        }
    }
    agrees.then_some(lemma)
}

fn plural_personal_subject(surface: &str) -> Option<&'static str> {
    let readings = cached_morphology_slot_identities_for_surface(surface, MAX_READINGS)?;
    let mut person = None;
    for reading in readings {
        let labels = canonical_morphology_feature_labels(reading.slot_id).ok()?;
        if !["pron", "nom", "pl"]
            .iter()
            .all(|label| labels.contains(label))
        {
            return None;
        }
        let value = ["p1", "p2", "p3"]
            .into_iter()
            .find(|label| labels.contains(label))?;
        if person.is_some_and(|known| known != value) {
            return None;
        }
        person = Some(value);
    }
    person
}

fn canonical_infinitive_aspect(surface: &str) -> Option<&'static str> {
    let readings = cached_morphology_slot_identities_for_surface(surface, MAX_READINGS)?;
    let mut aspect = None;
    for reading in readings {
        if canonical_morphology_primary_pos(reading.slot_id) != 2 {
            return None;
        }
        let labels = canonical_morphology_feature_labels(reading.slot_id).ok()?;
        if !labels.contains(&"inf") {
            return None;
        }
        let value = match (labels.contains(&"perf"), labels.contains(&"imperf")) {
            (true, false) => "perf",
            (false, true) => "imperf",
            _ => return None,
        };
        if aspect.is_some_and(|known| known != value) {
            return None;
        }
        aspect = Some(value);
    }
    aspect
}
