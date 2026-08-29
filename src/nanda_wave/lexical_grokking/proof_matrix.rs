use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::atoms::{encode_wave_surface, normalize_lexical_surface, AtomChannel};
use super::corruption::DamageExample;
use super::restoration::RestorationReadout;
use super::runtime::{GrokkingCandidate, LexicalGrokkingMemory};

const LANGUAGES: [&str; 2] = ["ru", "en"];
const LENGTHS: [&str; 4] = ["2_4", "5_8", "9_16", "17_32"];
const FREQUENCIES: [&str; 3] = ["head", "middle", "tail"];
pub(super) const DAMAGE_CLASSES: [&str; 13] = [
    "missing_letter",
    "extra_letter",
    "adjacent_transposition",
    "letter_substitution",
    "sparse_multi_omission",
    "non_adjacent_transposition",
    "double_substitution",
    "omission_transposition",
    "repeated_fragment",
    "prefix_truncation",
    "suffix_truncation",
    "layout_projection",
    "punctuation_suffix",
];
const AMBIGUITIES: [&str; 2] = ["objective_unique", "objective_tied"];
const LOSS_BOUNDARIES: [&str; 8] = [
    "query_encoding",
    "posting_evidence_availability",
    "typed_edit_reachability",
    "independent_frontier_eligibility",
    "typed_dependency_resolution",
    "nonlinear_settlement_rank",
    "restoration_authority",
    "contract_satisfied",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PositionBin {
    P0,
    P25,
    P50,
    P75,
    P100,
}

impl PositionBin {
    const ALL: [Self; 5] = [Self::P0, Self::P25, Self::P50, Self::P75, Self::P100];

    const fn label(self) -> &'static str {
        match self {
            Self::P0 => "p0",
            Self::P25 => "p25",
            Self::P50 => "p50",
            Self::P75 => "p75",
            Self::P100 => "p100",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct FrequencyEntry {
    language: &'static str,
    bucket: &'static str,
}

pub(super) struct FrequencyProfile {
    entries: Vec<FrequencyEntry>,
    unsupported_source_surfaces: usize,
}

impl FrequencyProfile {
    pub(super) fn from_words(words: &[String]) -> Self {
        let languages = words
            .iter()
            .map(|surface| language(surface))
            .collect::<Vec<_>>();
        let mut totals = BTreeMap::<&'static str, usize>::new();
        for language in languages.iter().copied().flatten() {
            *totals.entry(language).or_default() += 1;
        }
        let mut seen = BTreeMap::<&'static str, usize>::new();
        let entries = languages
            .into_iter()
            .map(|language| {
                let Some(language) = language else {
                    return FrequencyEntry {
                        language: "unsupported",
                        bucket: "unsupported",
                    };
                };
                let rank = seen.entry(language).or_default();
                let total = totals.get(language).copied().unwrap_or(1).max(1);
                let bucket = match rank.saturating_mul(3) / total {
                    0 => "head",
                    1 => "middle",
                    _ => "tail",
                };
                *rank += 1;
                FrequencyEntry { language, bucket }
            })
            .collect();
        Self {
            entries,
            unsupported_source_surfaces: words.len().saturating_sub(totals.values().sum()),
        }
    }

    fn get(&self, terminal_id: u32) -> FrequencyEntry {
        self.entries
            .get(terminal_id as usize)
            .copied()
            .unwrap_or(FrequencyEntry {
                language: "unsupported",
                bucket: "unsupported",
            })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct MatrixKey {
    language: &'static str,
    length: &'static str,
    frequency: &'static str,
    position: &'static str,
    position_pair: String,
    class: &'static str,
    ambiguity: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
struct MatrixMetrics {
    cases: usize,
    target_retained: usize,
    objective_top1: usize,
    authority_in_objective: usize,
    false_authority: usize,
    first_loss: BTreeMap<&'static str, usize>,
}

impl MatrixMetrics {
    fn merge(&mut self, source: Self) {
        self.cases += source.cases;
        self.target_retained += source.target_retained;
        self.objective_top1 += source.objective_top1;
        self.authority_in_objective += source.authority_in_objective;
        self.false_authority += source.false_authority;
        merge_counts(&mut self.first_loss, source.first_loss);
    }
}

#[derive(Clone, Debug, Serialize)]
struct MatrixRow {
    language: &'static str,
    length: &'static str,
    frequency: &'static str,
    position: &'static str,
    position_pair: String,
    class: &'static str,
    ambiguity: &'static str,
    #[serde(flatten)]
    metrics: MatrixMetrics,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct ProofMatrix {
    cases: usize,
    unsupported_source_surfaces: usize,
    unsupported_cases: usize,
    by_language: BTreeMap<&'static str, usize>,
    by_length: BTreeMap<&'static str, usize>,
    by_frequency: BTreeMap<&'static str, usize>,
    by_position: BTreeMap<&'static str, usize>,
    by_position_pair: BTreeMap<String, usize>,
    by_class: BTreeMap<&'static str, usize>,
    by_ambiguity: BTreeMap<&'static str, usize>,
    first_loss: BTreeMap<&'static str, usize>,
    #[serde(skip)]
    strata: BTreeMap<MatrixKey, MatrixMetrics>,
    requested_languages: Vec<&'static str>,
    requested_lengths: Vec<&'static str>,
    requested_frequencies: Vec<&'static str>,
    requested_positions: Vec<&'static str>,
    requested_position_pairs: Vec<String>,
    requested_classes: Vec<&'static str>,
    requested_ambiguity: Vec<&'static str>,
    requested_first_loss_boundaries: Vec<&'static str>,
    populated_strata: Vec<MatrixRow>,
    populated_requested_strata: usize,
    empty_requested_strata: usize,
    axis_totals_match_cases: bool,
    first_loss_total_matches_cases: bool,
    fixed_class_denominator: bool,
}

impl ProofMatrix {
    #[expect(
        clippy::too_many_arguments,
        reason = "existing explicit boundary contract"
    )]
    pub(super) fn record(
        &mut self,
        memory: &LexicalGrokkingMemory,
        frequency_profile: &FrequencyProfile,
        example: &DamageExample,
        target: u32,
        objective: &BTreeSet<u32>,
        candidates: &[GrokkingCandidate],
        restoration_candidates: &[GrokkingCandidate],
        restoration: &RestorationReadout,
    ) {
        let target_surface = memory.decode_terminal(target).unwrap_or_default();
        let frequency = frequency_profile.get(target);
        let length = length_bucket(target_surface.chars().count()).unwrap_or("unsupported");
        let positions = edit_position_buckets(example.class, &target_surface, &example.surface);
        let position = positions.first().copied().unwrap_or(PositionBin::P0);
        let pair_end = positions.last().copied().unwrap_or(position);
        let position_pair = format!("{}_{}", position.label(), pair_end.label());
        let ambiguity = if objective.len() == 1 {
            "objective_unique"
        } else {
            "objective_tied"
        };
        let selected = candidates.first().map(|candidate| candidate.terminal_id);
        let authority = authority_terminal(restoration);
        let target_retained = candidates
            .iter()
            .any(|candidate| candidate.terminal_id == target);
        let objective_top1 = selected.is_some_and(|terminal| objective.contains(&terminal));
        let authority_in_objective =
            authority.is_some_and(|terminal| objective.contains(&terminal));
        let false_authority = authority.is_some_and(|terminal| !objective.contains(&terminal));
        let boundary = first_loss_boundary(
            memory,
            example,
            objective,
            candidates,
            restoration_candidates,
            restoration,
        );
        let key = MatrixKey {
            language: frequency.language,
            length,
            frequency: frequency.bucket,
            position: position.label(),
            position_pair: position_pair.clone(),
            class: example.class,
            ambiguity,
        };
        let metrics = self.strata.entry(key).or_default();
        metrics.cases += 1;
        metrics.target_retained += usize::from(target_retained);
        metrics.objective_top1 += usize::from(objective_top1);
        metrics.authority_in_objective += usize::from(authority_in_objective);
        metrics.false_authority += usize::from(false_authority);
        *metrics.first_loss.entry(boundary).or_default() += 1;
        self.cases += 1;
        *self.by_language.entry(frequency.language).or_default() += 1;
        *self.by_length.entry(length).or_default() += 1;
        *self.by_frequency.entry(frequency.bucket).or_default() += 1;
        *self.by_position.entry(position.label()).or_default() += 1;
        *self.by_position_pair.entry(position_pair).or_default() += 1;
        *self.by_class.entry(example.class).or_default() += 1;
        *self.by_ambiguity.entry(ambiguity).or_default() += 1;
        *self.first_loss.entry(boundary).or_default() += 1;
        self.unsupported_cases +=
            usize::from(frequency.language == "unsupported" || length == "unsupported");
    }

    pub(super) fn merge(&mut self, source: Self) {
        self.cases += source.cases;
        self.unsupported_cases += source.unsupported_cases;
        merge_counts(&mut self.by_language, source.by_language);
        merge_counts(&mut self.by_length, source.by_length);
        merge_counts(&mut self.by_frequency, source.by_frequency);
        merge_counts(&mut self.by_position, source.by_position);
        merge_counts(&mut self.by_position_pair, source.by_position_pair);
        merge_counts(&mut self.by_class, source.by_class);
        merge_counts(&mut self.by_ambiguity, source.by_ambiguity);
        merge_counts(&mut self.first_loss, source.first_loss);
        for (key, metrics) in source.strata {
            self.strata.entry(key).or_default().merge(metrics);
        }
    }

    pub(super) fn finalize(&mut self, frequency_profile: &FrequencyProfile) {
        self.unsupported_source_surfaces = frequency_profile.unsupported_source_surfaces;
        self.requested_languages = LANGUAGES.to_vec();
        self.requested_lengths = LENGTHS.to_vec();
        self.requested_frequencies = FREQUENCIES.to_vec();
        self.requested_positions = PositionBin::ALL
            .into_iter()
            .map(PositionBin::label)
            .collect();
        self.requested_position_pairs = PositionBin::ALL
            .into_iter()
            .flat_map(|left| {
                PositionBin::ALL
                    .into_iter()
                    .map(move |right| format!("{}_{}", left.label(), right.label()))
            })
            .collect();
        self.requested_classes = DAMAGE_CLASSES.to_vec();
        self.requested_ambiguity = AMBIGUITIES.to_vec();
        self.requested_first_loss_boundaries = LOSS_BOUNDARIES.to_vec();
        self.populated_strata = std::mem::take(&mut self.strata)
            .into_iter()
            .map(|(key, metrics)| MatrixRow {
                language: key.language,
                length: key.length,
                frequency: key.frequency,
                position: key.position,
                position_pair: key.position_pair,
                class: key.class,
                ambiguity: key.ambiguity,
                metrics,
            })
            .collect();
        self.populated_requested_strata = self
            .populated_strata
            .iter()
            .filter(|row| {
                LANGUAGES.contains(&row.language)
                    && LENGTHS.contains(&row.length)
                    && FREQUENCIES.contains(&row.frequency)
                    && PositionBin::ALL
                        .into_iter()
                        .any(|position| position.label() == row.position)
                    && self.requested_position_pairs.contains(&row.position_pair)
                    && DAMAGE_CLASSES.contains(&row.class)
                    && AMBIGUITIES.contains(&row.ambiguity)
            })
            .count();
        let requested = LANGUAGES.len()
            * LENGTHS.len()
            * FREQUENCIES.len()
            * PositionBin::ALL.len()
            * PositionBin::ALL.len()
            * DAMAGE_CLASSES.len()
            * AMBIGUITIES.len();
        self.empty_requested_strata = requested.saturating_sub(self.populated_requested_strata);
        self.axis_totals_match_cases = [
            self.by_language.values().sum::<usize>(),
            self.by_length.values().sum::<usize>(),
            self.by_frequency.values().sum::<usize>(),
            self.by_position.values().sum::<usize>(),
            self.by_position_pair.values().sum::<usize>(),
            self.by_class.values().sum::<usize>(),
            self.by_ambiguity.values().sum::<usize>(),
        ]
        .into_iter()
        .all(|total| total == self.cases);
        self.first_loss_total_matches_cases = self.first_loss.values().sum::<usize>() == self.cases;
        self.fixed_class_denominator = self.by_class.len() == DAMAGE_CLASSES.len()
            && DAMAGE_CLASSES
                .into_iter()
                .all(|class| self.by_class.contains_key(class));
    }
}

fn first_loss_boundary(
    memory: &LexicalGrokkingMemory,
    example: &DamageExample,
    objective: &BTreeSet<u32>,
    candidates: &[GrokkingCandidate],
    restoration_candidates: &[GrokkingCandidate],
    restoration: &RestorationReadout,
) -> &'static str {
    if encode_wave_surface(&example.surface).is_empty() {
        return "query_encoding";
    }
    let observed = memory.resolve_surface(&example.surface);
    let lexical = observed
        .into_iter()
        .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
        .collect::<BTreeMap<_, _>>();
    if !objective
        .iter()
        .any(|terminal| memory.activation_for_terminal(*terminal, &lexical).hits != 0)
    {
        return "posting_evidence_availability";
    }
    if !objective.iter().any(|terminal| {
        memory
            .decode_terminal(*terminal)
            .is_some_and(|target_surface| {
                typed_edit_reachable(example.class, &example.surface, &target_surface)
            })
    }) {
        return "typed_edit_reachability";
    }
    if !candidates
        .iter()
        .any(|candidate| objective.contains(&candidate.terminal_id))
    {
        return "independent_frontier_eligibility";
    }
    if !super::restoration::geometric_basin(restoration_candidates)
        .into_iter()
        .any(|candidate| objective.contains(&candidate.terminal_id))
    {
        return "typed_dependency_resolution";
    }
    if !candidates
        .first()
        .is_some_and(|candidate| objective.contains(&candidate.terminal_id))
    {
        return "nonlinear_settlement_rank";
    }
    if !restoration_contract_satisfied(restoration, objective) {
        return "restoration_authority";
    }
    "contract_satisfied"
}

fn authority_terminal(readout: &RestorationReadout) -> Option<u32> {
    match readout {
        RestorationReadout::Winner { candidate } => Some(candidate.terminal_id),
        _ => None,
    }
}

fn restoration_contract_satisfied(readout: &RestorationReadout, objective: &BTreeSet<u32>) -> bool {
    match authority_terminal(readout) {
        Some(terminal) => objective.contains(&terminal),
        None => objective.len() > 1,
    }
}

fn typed_edit_reachable(class: &str, damaged: &str, target: &str) -> bool {
    let normalize = |surface: &str| {
        if class == "layout_projection" {
            surface.trim().to_lowercase()
        } else {
            normalize_lexical_surface(surface)
        }
    };
    let damaged = normalize(damaged).chars().collect::<Vec<_>>();
    let target = normalize(target).chars().collect::<Vec<_>>();
    let mismatches = damaged
        .iter()
        .zip(&target)
        .filter(|(left, right)| left != right)
        .count();
    match class {
        "missing_letter" => target.len() == damaged.len() + 1 && is_subsequence(&damaged, &target),
        "extra_letter" => damaged.len() == target.len() + 1 && is_subsequence(&target, &damaged),
        "adjacent_transposition" => is_single_adjacent_transposition(&damaged, &target),
        "letter_substitution" => damaged.len() == target.len() && mismatches == 1,
        "sparse_multi_omission" => {
            target.len() == damaged.len() + 2 && is_subsequence(&damaged, &target)
        }
        "non_adjacent_transposition" => is_single_non_adjacent_transposition(&damaged, &target),
        "double_substitution" => damaged.len() == target.len() && mismatches == 2,
        "omission_transposition" => omission_transposition_reachable(&damaged, &target),
        "repeated_fragment" => damaged.len() > target.len() && is_subsequence(&target, &damaged),
        "prefix_truncation" => {
            target.len() == damaged.len() + 1 && target.get(1..) == Some(damaged.as_slice())
        }
        "suffix_truncation" => {
            target.len() == damaged.len() + 1
                && target.get(..target.len() - 1) == Some(damaged.as_slice())
        }
        "layout_projection" => layout_projection_reachable(damaged.as_slice(), target.as_slice()),
        "punctuation_suffix" => damaged == target,
        _ => false,
    }
}

fn is_subsequence(needle: &[char], haystack: &[char]) -> bool {
    let mut next = 0;
    for character in haystack {
        if needle.get(next) == Some(character) {
            next += 1;
        }
    }
    next == needle.len()
}

fn is_single_adjacent_transposition(left: &[char], right: &[char]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mismatches = left
        .iter()
        .zip(right)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    matches!(mismatches.as_slice(), [first, second]
        if *second == *first + 1
            && left[*first] == right[*second]
            && left[*second] == right[*first])
}

fn is_single_non_adjacent_transposition(left: &[char], right: &[char]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mismatches = left
        .iter()
        .zip(right)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    matches!(mismatches.as_slice(), [first, second]
        if *second > *first + 1
            && left[*first] == right[*second]
            && left[*second] == right[*first])
}

fn omission_transposition_reachable(damaged: &[char], target: &[char]) -> bool {
    if target.len() != damaged.len() + 1 {
        return false;
    }
    (0..target.len()).any(|omitted| {
        let mut remainder = target.to_vec();
        remainder.remove(omitted);
        is_single_adjacent_transposition(damaged, &remainder)
    })
}

fn layout_projection_reachable(damaged: &[char], target: &[char]) -> bool {
    use crate::dict::{convert, Direction};

    let damaged = damaged.iter().collect::<String>();
    let target = target.iter().collect::<String>();
    convert(&target, Direction::Ru2Us) == damaged || convert(&target, Direction::Us2Ru) == damaged
}

fn language(surface: &str) -> Option<&'static str> {
    let mut ascii = false;
    let mut cyrillic = false;
    for character in surface.chars() {
        ascii |= character.is_ascii_alphabetic();
        cyrillic |= matches!(character, '\u{0400}'..='\u{052f}');
    }
    match (ascii, cyrillic) {
        (true, false) => Some("en"),
        (false, true) => Some("ru"),
        _ => None,
    }
}

fn length_bucket(length: usize) -> Option<&'static str> {
    match length {
        2..=4 => Some("2_4"),
        5..=8 => Some("5_8"),
        9..=16 => Some("9_16"),
        17..=32 => Some("17_32"),
        _ => None,
    }
}

fn edit_position_buckets(class: &str, target: &str, damaged: &str) -> Vec<PositionBin> {
    if class == "punctuation_suffix" {
        return vec![PositionBin::P100];
    }
    let normalize = |surface: &str| {
        if class == "layout_projection" {
            surface.trim().to_lowercase()
        } else {
            normalize_lexical_surface(surface)
        }
    };
    let target = normalize(target).chars().collect::<Vec<_>>();
    let damaged = normalize(damaged).chars().collect::<Vec<_>>();
    let mut positions = lcs_edit_positions(&target, &damaged);
    if positions.is_empty() {
        positions.push(0);
    }
    positions.sort_unstable();
    positions.dedup();
    let mut bins = Vec::new();
    for position in positions {
        let bin = relative_position_bucket(position, target.len());
        if bins.last() != Some(&bin) {
            bins.push(bin);
        }
    }
    bins
}

fn lcs_edit_positions(target: &[char], damaged: &[char]) -> Vec<usize> {
    let width = damaged.len() + 1;
    let mut lcs = vec![0_u16; (target.len() + 1) * width];
    for left in 0..target.len() {
        for right in 0..damaged.len() {
            lcs[(left + 1) * width + right + 1] = if target[left] == damaged[right] {
                lcs[left * width + right].saturating_add(1)
            } else {
                lcs[left * width + right + 1].max(lcs[(left + 1) * width + right])
            };
        }
    }
    let mut left = target.len();
    let mut right = damaged.len();
    let mut matched_target = BTreeSet::new();
    let mut matched_damaged = BTreeSet::new();
    let mut matched_pairs = Vec::new();
    while left != 0 && right != 0 {
        if target[left - 1] == damaged[right - 1] {
            matched_target.insert(left - 1);
            matched_damaged.insert(right - 1);
            matched_pairs.push((left - 1, right - 1));
            left -= 1;
            right -= 1;
        } else if lcs[(left - 1) * width + right] >= lcs[left * width + right - 1] {
            left -= 1;
        } else {
            right -= 1;
        }
    }
    matched_pairs.reverse();
    let mut positions = (0..target.len())
        .filter(|position| !matched_target.contains(position))
        .collect::<Vec<_>>();
    for damaged_position in
        (0..damaged.len()).filter(|position| !matched_damaged.contains(position))
    {
        let insertion_position = matched_pairs
            .iter()
            .find_map(|(target_position, matched_damaged_position)| {
                (*matched_damaged_position > damaged_position).then_some(*target_position)
            })
            .unwrap_or_else(|| target.len().saturating_sub(1));
        positions.push(insertion_position);
    }
    positions
}

fn relative_position_bucket(position: usize, length: usize) -> PositionBin {
    if length <= 1 {
        return PositionBin::P0;
    }
    match position.saturating_mul(4).saturating_add((length - 1) / 2) / (length - 1) {
        0 => PositionBin::P0,
        1 => PositionBin::P25,
        2 => PositionBin::P50,
        3 => PositionBin::P75,
        _ => PositionBin::P100,
    }
}

fn merge_counts<K: Ord>(target: &mut BTreeMap<K, usize>, source: BTreeMap<K, usize>) {
    for (key, count) in source {
        *target.entry(key).or_default() += count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_edits_keep_distinct_position_bins() {
        assert_eq!(
            edit_position_buckets("sparse_multi_omission", "abcdefghi", "abdefgi"),
            vec![PositionBin::P25, PositionBin::P100]
        );
    }

    #[test]
    fn insertion_and_punctuation_positions_are_not_forced_to_zero_or_end() {
        assert_eq!(
            edit_position_buckets("extra_letter", "abcdef", "abxcdef"),
            vec![PositionBin::P50]
        );
        assert_eq!(
            edit_position_buckets("punctuation_suffix", "abcdef", "abcdef!"),
            vec![PositionBin::P100]
        );
    }

    #[test]
    fn matrix_merge_is_permutation_invariant() {
        let key = MatrixKey {
            language: "ru",
            length: "5_8",
            frequency: "head",
            position: "p25",
            position_pair: "p25_p75".to_string(),
            class: "missing_letter",
            ambiguity: "objective_unique",
        };
        let mut left = ProofMatrix {
            cases: 1,
            ..ProofMatrix::default()
        };
        left.by_language.insert("ru", 1);
        left.by_length.insert("5_8", 1);
        left.by_frequency.insert("head", 1);
        left.by_position.insert("p25", 1);
        left.by_position_pair.insert("p25_p75".to_string(), 1);
        left.by_class.insert("missing_letter", 1);
        left.by_ambiguity.insert("objective_unique", 1);
        left.first_loss.insert("contract_satisfied", 1);
        left.strata.entry(key.clone()).or_default().cases = 1;
        let mut right = ProofMatrix {
            cases: 2,
            ..ProofMatrix::default()
        };
        right.by_language.insert("ru", 2);
        right.by_length.insert("5_8", 2);
        right.by_frequency.insert("head", 2);
        right.by_position.insert("p25", 2);
        right.by_position_pair.insert("p25_p75".to_string(), 2);
        right.by_class.insert("missing_letter", 2);
        right.by_ambiguity.insert("objective_unique", 2);
        right.first_loss.insert("contract_satisfied", 2);
        right.strata.entry(key).or_default().cases = 2;
        let mut forward = left.clone();
        forward.merge(right.clone());
        let mut reverse = right;
        reverse.merge(left);
        let profile = FrequencyProfile::from_words(&[]);
        forward.finalize(&profile);
        reverse.finalize(&profile);
        assert_eq!(
            serde_json::to_vec(&forward).unwrap(),
            serde_json::to_vec(&reverse).unwrap()
        );
    }

    #[test]
    fn matrix_finalize_exposes_fixed_axes_and_exact_totals() {
        let mut matrix = ProofMatrix {
            cases: 2,
            ..ProofMatrix::default()
        };
        for counts in [
            &mut matrix.by_language,
            &mut matrix.by_length,
            &mut matrix.by_frequency,
            &mut matrix.by_position,
            &mut matrix.by_class,
            &mut matrix.by_ambiguity,
        ] {
            counts.insert("first", 1);
            counts.insert("second", 1);
        }
        matrix.by_position_pair.insert("p0_p25".to_string(), 2);
        matrix.first_loss.insert("contract_satisfied", 2);
        matrix.finalize(&FrequencyProfile::from_words(&[]));

        assert_eq!(matrix.requested_classes, DAMAGE_CLASSES);
        assert_eq!(
            matrix.requested_positions,
            ["p0", "p25", "p50", "p75", "p100"]
        );
        assert!(matrix.axis_totals_match_cases);
        assert!(matrix.first_loss_total_matches_cases);
        assert!(!matrix.fixed_class_denominator);
    }

    #[test]
    fn unsupported_lengths_do_not_enter_requested_strata() {
        assert_eq!(length_bucket(1), None);
        assert_eq!(length_bucket(2), Some("2_4"));
        assert_eq!(length_bucket(32), Some("17_32"));
        assert_eq!(length_bucket(33), None);
    }

    #[test]
    fn typed_reachability_covers_each_fixed_operator_family() {
        let cases = [
            ("missing_letter", "ac", "abc"),
            ("extra_letter", "abbc", "abc"),
            ("adjacent_transposition", "acb", "abc"),
            ("letter_substitution", "axc", "abc"),
            ("sparse_multi_omission", "ac", "abcd"),
            ("non_adjacent_transposition", "dbca", "abcd"),
            ("double_substitution", "axyd", "abcd"),
            ("omission_transposition", "acb", "abcd"),
            ("repeated_fragment", "abcbcd", "abcd"),
            ("prefix_truncation", "bcd", "abcd"),
            ("suffix_truncation", "abc", "abcd"),
            ("layout_projection", "ghbdtn", "привет"),
            ("layout_projection", ",", "б"),
            ("punctuation_suffix", "word!", "word"),
        ];
        for (class, damaged, target) in cases {
            assert!(
                typed_edit_reachable(class, damaged, target),
                "{class}: {damaged:?} -> {target:?}"
            );
        }
    }

    #[test]
    fn generated_layout_projection_has_typed_witness_on_external_corpus() {
        let Ok(path) = std::env::var("LAY_L11_MATRIX_CORPUS") else {
            return;
        };
        let text = std::fs::read_to_string(path).unwrap();
        for target in text
            .lines()
            .map(str::trim)
            .filter(|word| !word.is_empty())
            .take(10_000)
        {
            let (training, heldout) = super::super::corruption::split_scale_damages(target, false);
            for example in training.into_iter().chain(heldout) {
                if example.class == "layout_projection" {
                    assert!(
                        typed_edit_reachable(example.class, &example.surface, target),
                        "layout projection lacks typed witness: {:?} -> {:?}",
                        example.surface,
                        target
                    );
                }
            }
        }
    }

    #[test]
    fn production_search_does_not_import_proof_matrix() {
        for source in [
            include_str!("runtime.rs"),
            include_str!("peak_search/mod.rs"),
        ] {
            assert!(!source.contains("proof_matrix"));
        }
    }
}
