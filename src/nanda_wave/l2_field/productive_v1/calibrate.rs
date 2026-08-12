use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::TypingErrorClass;

use super::events::ProductiveSplitV1;
use super::records::CalibrationCellRecordV1;
use super::types::{ContradictionCertificateV1, ProductiveCandidateIdentityV1};

const MINIMUM_AUTHORITY_GROUPS: usize = 200;
const MAX_PRODUCTIVE_TIED_OUTPUT: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum CandidateProvenanceClassV1 {
    Exact = 1,
    TrainingSeenGenerated = 2,
    UnobservedLemmaSlot = 3,
    ColdLemmaBinding = 4,
    TrainingUnseen = 5,
}

impl CandidateProvenanceClassV1 {
    const fn generated(self) -> bool {
        !matches!(self, Self::Exact)
    }
}

pub(super) const AMBIGUITY_SYNCRHETIC_SLOT: u8 = 1 << 0;
pub(super) const AMBIGUITY_SAME_LEMMA_MULTI_LABEL: u8 = 1 << 1;
pub(super) const AMBIGUITY_CROSS_LEMMA_BASIN: u8 = 1 << 2;
pub(super) const AMBIGUITY_GENERATED_OVERFLOW: u8 = 1 << 3;
const KNOWN_AMBIGUITY_BITS: u8 = AMBIGUITY_SYNCRHETIC_SLOT
    | AMBIGUITY_SAME_LEMMA_MULTI_LABEL
    | AMBIGUITY_CROSS_LEMMA_BASIN
    | AMBIGUITY_GENERATED_OVERFLOW;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ObservableCalibrationStratumV1 {
    pub(super) transition_class: String,
    pub(super) provenance: CandidateProvenanceClassV1,
    pub(super) support_bin: u8,
    pub(super) ambiguity_kind: u8,
}

impl ObservableCalibrationStratumV1 {
    pub(super) fn new(
        observed: &str,
        candidate: &str,
        provenance: CandidateProvenanceClassV1,
        minimum_independent_support: u32,
        ambiguity_kind: u8,
    ) -> Result<Self, &'static str> {
        if ambiguity_kind & !KNOWN_AMBIGUITY_BITS != 0 {
            return Err("calibration stratum contains an unknown ambiguity bit");
        }
        let transition_class = crate::typing_transition::action::classify_token_transition(
            observed,
            candidate,
            CandidateOrigin::L2Surface,
            TypingErrorClass::Unknown,
        )
        .as_str()
        .to_string();
        let support = minimum_independent_support.max(1);
        let support_bin = (u32::BITS - 1 - support.leading_zeros()).min(15) as u8;
        Ok(Self {
            transition_class,
            provenance,
            support_bin,
            ambiguity_kind,
        })
    }

    fn backoff_keys(&self) -> [CalibrationBackoffKeyV1; 5] {
        let merged_provenance = match self.provenance {
            CandidateProvenanceClassV1::UnobservedLemmaSlot
            | CandidateProvenanceClassV1::ColdLemmaBinding => {
                CandidateProvenanceClassV1::TrainingUnseen
            }
            provenance => provenance,
        };
        [
            CalibrationBackoffKeyV1 {
                transition_class: Some(self.transition_class.clone()),
                provenance: self.provenance,
                support_bin: Some(self.support_bin),
                ambiguity_kind: Some(self.ambiguity_kind),
                generated: self.provenance.generated(),
                level: 0,
            },
            CalibrationBackoffKeyV1 {
                transition_class: Some(self.transition_class.clone()),
                provenance: self.provenance,
                support_bin: None,
                ambiguity_kind: Some(self.ambiguity_kind),
                generated: self.provenance.generated(),
                level: 1,
            },
            CalibrationBackoffKeyV1 {
                transition_class: Some(self.transition_class.clone()),
                provenance: merged_provenance,
                support_bin: None,
                ambiguity_kind: Some(self.ambiguity_kind),
                generated: self.provenance.generated(),
                level: 2,
            },
            CalibrationBackoffKeyV1 {
                transition_class: None,
                provenance: merged_provenance,
                support_bin: None,
                ambiguity_kind: Some(self.ambiguity_kind),
                generated: self.provenance.generated(),
                level: 3,
            },
            CalibrationBackoffKeyV1 {
                transition_class: None,
                provenance: if self.provenance.generated() {
                    CandidateProvenanceClassV1::TrainingUnseen
                } else {
                    CandidateProvenanceClassV1::Exact
                },
                support_bin: None,
                ambiguity_kind: None,
                generated: self.provenance.generated(),
                level: 4,
            },
        ]
    }

    pub(super) fn packaged_backoff_key_ids(&self) -> Result<[u32; 5], &'static str> {
        let keys = self.backoff_keys();
        Ok([
            calibration_key_hash(&keys[0])?,
            calibration_key_hash(&keys[1])?,
            calibration_key_hash(&keys[2])?,
            calibration_key_hash(&keys[3])?,
            calibration_key_hash(&keys[4])?,
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CalibrationBackoffKeyV1 {
    transition_class: Option<String>,
    provenance: CandidateProvenanceClassV1,
    support_bin: Option<u8>,
    ambiguity_kind: Option<u8>,
    generated: bool,
    level: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CalibrationCandidateV1 {
    pub(super) identity: ProductiveCandidateIdentityV1,
    pub(super) normalized_surface: String,
    pub(super) score_q16: i64,
    pub(super) grounded_lemma_evidence: u32,
    pub(super) exact_osa_distance: u16,
    pub(super) exact_form: bool,
    pub(super) gold_valid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CalibrationGroupV1 {
    pub(super) split: ProductiveSplitV1,
    pub(super) group_identity: [u8; 32],
    pub(super) stratum: ObservableCalibrationStratumV1,
    pub(super) candidates: Vec<CalibrationCandidateV1>,
    pub(super) false_singleton: bool,
    pub(super) grounded_winner_protection_violation: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CalibrationCellV1 {
    pub(super) stratum_id: u32,
    pub(super) winner_margin_q16: Option<i64>,
    pub(super) tie_radius_q16: i64,
    pub(super) support: u32,
    pub(super) correct_winner_count: u32,
    pub(super) false_winner_count: u32,
    pub(super) tied_count: u32,
}

#[derive(Clone, Debug, Default)]
pub(super) struct CalibrationTableV1 {
    cells: BTreeMap<CalibrationBackoffKeyV1, CalibrationCellV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PackagedCalibrationCellV1 {
    pub(super) stratum_key_id: u32,
    pub(super) cell: CalibrationCellV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackagedCalibrationTableV1 {
    pub(super) cells: Vec<PackagedCalibrationCellV1>,
    pub(super) generated_fallback_row: u16,
}

impl CalibrationTableV1 {
    pub(super) fn package_rows(&self) -> Result<PackagedCalibrationTableV1, &'static str> {
        if self.cells.is_empty() || self.cells.len() > u16::MAX as usize {
            return Err("productive calibration table is empty or exceeds u16 rows");
        }
        let mut rows = Vec::with_capacity(self.cells.len());
        let mut generated_fallback_key = None;
        for (key, cell) in &self.cells {
            let hash = calibration_key_hash(key)?;
            if key.level == 4
                && key.generated
                && key.transition_class.is_none()
                && key.support_bin.is_none()
                && key.ambiguity_kind.is_none()
            {
                if generated_fallback_key.replace(hash).is_some() {
                    return Err("productive calibration has multiple generated fallback keys");
                }
            }
            rows.push(PackagedCalibrationCellV1 {
                stratum_key_id: hash,
                cell: *cell,
            });
        }
        rows.sort_unstable_by_key(|row| row.stratum_key_id);
        if rows
            .windows(2)
            .any(|pair| pair[0].stratum_key_id == pair[1].stratum_key_id)
        {
            return Err("productive calibration stratum hash collision");
        }
        let fallback_key = generated_fallback_key
            .ok_or("productive calibration lacks the generated fallback key")?;
        let generated_fallback_row = rows
            .binary_search_by_key(&fallback_key, |row| row.stratum_key_id)
            .map_err(|_| "productive generated fallback row disappeared")?
            .checked_add(1)
            .and_then(|row| u16::try_from(row).ok())
            .ok_or("productive generated fallback row exceeds u16")?;
        Ok(PackagedCalibrationTableV1 {
            cells: rows,
            generated_fallback_row,
        })
    }
}

fn calibration_key_hash(key: &CalibrationBackoffKeyV1) -> Result<u32, &'static str> {
    let mut canonical = Vec::new();
    canonical.extend_from_slice(b"lay-productive-calibration-v1\0");
    canonical.push(key.level);
    canonical.push(key.provenance as u8);
    canonical.push(u8::from(key.generated));
    match key.support_bin {
        Some(value) => canonical.extend_from_slice(&[1, value]),
        None => canonical.extend_from_slice(&[0, 0]),
    }
    match key.ambiguity_kind {
        Some(value) => canonical.extend_from_slice(&[1, value]),
        None => canonical.extend_from_slice(&[0, 0]),
    }
    match &key.transition_class {
        Some(value) => {
            canonical.push(1);
            canonical.extend_from_slice(
                &u32::try_from(value.len())
                    .map_err(|_| "productive calibration transition class exceeds u32")?
                    .to_le_bytes(),
            );
            canonical.extend_from_slice(value.as_bytes());
        }
        None => canonical.extend_from_slice(&[0, 0, 0, 0, 0]),
    }
    let digest = Sha256::digest(canonical);
    let hash = u32::from_le_bytes(digest[0..4].try_into().expect("SHA-256 prefix"));
    if hash == 0 {
        return Err("productive calibration stratum hash is zero");
    }
    Ok(hash)
}

pub(super) fn fit_calibration_table(
    groups: &[CalibrationGroupV1],
) -> Result<CalibrationTableV1, &'static str> {
    if groups.is_empty() {
        return Err("productive calibration requires disjoint calibration groups");
    }
    let mut ordered = groups.to_vec();
    ordered.sort_by(|left, right| left.group_identity.cmp(&right.group_identity));
    if ordered
        .windows(2)
        .any(|pair| pair[0].group_identity == pair[1].group_identity)
    {
        return Err("productive calibration repeats a group identity");
    }
    for group in &mut ordered {
        if group.split != ProductiveSplitV1::Calibration {
            return Err("productive calibration contains a non-calibration lemma");
        }
        if group.candidates.is_empty() {
            return Err("productive calibration group has no candidates");
        }
        group.candidates.sort_by(candidate_score_order);
        let identities = group
            .candidates
            .iter()
            .map(|candidate| candidate.identity)
            .collect::<BTreeSet<_>>();
        if identities.len() != group.candidates.len() {
            return Err("productive calibration group repeats a candidate identity");
        }
        if !group
            .candidates
            .iter()
            .any(|candidate| candidate.gold_valid)
        {
            return Err("productive calibration group has no valid alternative");
        }
    }

    let mut by_key = BTreeMap::<CalibrationBackoffKeyV1, Vec<CalibrationGroupV1>>::new();
    for group in ordered {
        for key in group.stratum.backoff_keys() {
            by_key.entry(key).or_default().push(group.clone());
        }
    }
    let mut table = CalibrationTableV1::default();
    for (index, (key, groups)) in by_key.into_iter().enumerate() {
        let stratum_id = u32::try_from(index + 1)
            .map_err(|_| "productive calibration stratum count exceeds u32")?;
        table.cells.insert(key, fit_cell(stratum_id, &groups)?);
    }
    Ok(table)
}

fn fit_cell(
    stratum_id: u32,
    groups: &[CalibrationGroupV1],
) -> Result<CalibrationCellV1, &'static str> {
    let mut authority = groups
        .iter()
        .map(|group| {
            let leader = &group.candidates[0];
            let second_score = group
                .candidates
                .get(1)
                .map(|candidate| candidate.score_q16)
                .unwrap_or(i64::MIN);
            let margin = if second_score == i64::MIN {
                i64::MAX
            } else {
                leader
                    .score_q16
                    .checked_sub(second_score)
                    .ok_or("productive calibration leader margin overflow")?
            };
            let unique = group
                .candidates
                .get(1)
                .is_none_or(|second| leader.score_q16 > second.score_q16);
            Ok((
                margin,
                unique && !leader.gold_valid,
                group.false_singleton,
                group.grounded_winner_protection_violation,
                unique && leader.gold_valid,
            ))
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    authority.sort_by(|left, right| right.0.cmp(&left.0));
    let mut wrong = 0_usize;
    let mut false_singleton = 0_usize;
    let mut grounded_violations = 0_usize;
    let mut winner_margin = None;
    for (index, (margin, wrong_unique, false_singleton_group, grounded_violation, _)) in
        authority.iter().enumerate()
    {
        wrong += usize::from(*wrong_unique);
        false_singleton += usize::from(*false_singleton_group);
        grounded_violations += usize::from(*grounded_violation);
        if wrong != 0 || false_singleton != 0 || grounded_violations != 0 {
            break;
        }
        if index + 1 >= MINIMUM_AUTHORITY_GROUPS {
            winner_margin = Some(*margin);
        }
    }

    let mut alternatives = Vec::<(i64, bool)>::new();
    for group in groups {
        let leader_score = group.candidates[0].score_q16;
        for candidate in group.candidates.iter().skip(1) {
            alternatives.push((
                leader_score
                    .checked_sub(candidate.score_q16)
                    .ok_or("productive calibration tie difference overflow")?,
                candidate.gold_valid,
            ));
        }
    }
    alternatives.sort_unstable_by_key(|record| record.0);
    let isotonic = pava_nonincreasing(&alternatives);
    let mut valid_differences = alternatives
        .iter()
        .filter_map(|(difference, valid)| valid.then_some(*difference))
        .collect::<Vec<_>>();
    valid_differences.sort_unstable();
    let empirical_radius = if valid_differences.is_empty() {
        0
    } else {
        let retained = valid_differences.len().saturating_mul(99).div_ceil(100);
        valid_differences[retained.saturating_sub(1)]
    };
    let calibrated_mass = isotonic
        .iter()
        .map(|(_, probability)| u64::from(*probability))
        .sum::<u64>();
    let required_mass = calibrated_mass.saturating_mul(99).div_ceil(100);
    let mut cumulative_mass = 0_u64;
    let mut calibrated_radius = 0_i64;
    for (difference, probability) in isotonic {
        cumulative_mass = cumulative_mass.saturating_add(u64::from(probability));
        calibrated_radius = difference;
        if cumulative_mass >= required_mass {
            break;
        }
    }
    let tie_radius_q16 = empirical_radius.max(calibrated_radius);
    Ok(CalibrationCellV1 {
        stratum_id,
        winner_margin_q16: (groups.len() >= MINIMUM_AUTHORITY_GROUPS)
            .then_some(winner_margin)
            .flatten(),
        tie_radius_q16,
        support: u32::try_from(groups.len()).map_err(|_| "calibration support exceeds u32")?,
        correct_winner_count: authority.iter().filter(|record| record.4).count() as u32,
        false_winner_count: authority.iter().filter(|record| record.1).count() as u32,
        tied_count: groups
            .iter()
            .filter(|group| {
                group
                    .candidates
                    .get(1)
                    .is_some_and(|second| second.score_q16 == group.candidates[0].score_q16)
            })
            .count() as u32,
    })
}

#[derive(Clone, Copy, Debug)]
struct PavaBlockV1 {
    start: usize,
    end: usize,
    positives: u64,
    count: u64,
}

fn pava_nonincreasing(records: &[(i64, bool)]) -> Vec<(i64, u32)> {
    let mut blocks = Vec::<PavaBlockV1>::new();
    for (index, (_, positive)) in records.iter().enumerate() {
        blocks.push(PavaBlockV1 {
            start: index,
            end: index + 1,
            positives: u64::from(*positive),
            count: 1,
        });
        while blocks.len() >= 2 {
            let right = blocks[blocks.len() - 1];
            let left = blocks[blocks.len() - 2];
            if left.positives * right.count >= right.positives * left.count {
                break;
            }
            blocks.pop();
            blocks.pop();
            blocks.push(PavaBlockV1 {
                start: left.start,
                end: right.end,
                positives: left.positives + right.positives,
                count: left.count + right.count,
            });
        }
    }
    let mut calibrated = Vec::with_capacity(records.len());
    for block in blocks {
        let probability_millionths =
            ((block.positives * 1_000_000 + block.count / 2) / block.count) as u32;
        for record in records.iter().take(block.end).skip(block.start) {
            calibrated.push((record.0, probability_millionths));
        }
    }
    calibrated
}

fn candidate_score_order(
    left: &CalibrationCandidateV1,
    right: &CalibrationCandidateV1,
) -> std::cmp::Ordering {
    right
        .score_q16
        .cmp(&left.score_q16)
        .then_with(|| {
            right
                .grounded_lemma_evidence
                .cmp(&left.grounded_lemma_evidence)
        })
        .then_with(|| left.exact_osa_distance.cmp(&right.exact_osa_distance))
        .then_with(|| right.exact_form.cmp(&left.exact_form))
        .then_with(|| {
            candidate_identity_order(
                left.identity,
                &left.normalized_surface,
                right.identity,
                &right.normalized_surface,
            )
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadoutCandidateV1 {
    pub(super) identity: ProductiveCandidateIdentityV1,
    pub(super) equivalent_identities: Vec<ProductiveCandidateIdentityV1>,
    pub(super) normalized_surface: String,
    pub(super) score_q16: i64,
    pub(super) grounded_lemma_evidence: u32,
    pub(super) exact_osa_distance: u16,
    pub(super) exact_form: bool,
    pub(super) cross_lemma_ownership_satisfied: bool,
    pub(super) rank_origin: CandidateRankOriginV1,
    pub(super) cross_lane_certified: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CandidateRankOriginV1 {
    #[default]
    BaseV64,
    RecoveredV66,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ProductiveCalibratedVerdictV1 {
    Winner {
        candidate: ReadoutCandidateV1,
        calibration_stratum_id: u32,
    },
    Tied {
        candidates: Vec<ReadoutCandidateV1>,
        calibration_stratum_id: u32,
    },
    Abstain {
        suggestions: Vec<ReadoutCandidateV1>,
        productive_overflow: bool,
    },
}

#[allow(clippy::too_many_arguments)]
pub(super) fn calibrated_readout(
    table: &CalibrationTableV1,
    stratum: &ObservableCalibrationStratumV1,
    candidates: Vec<ReadoutCandidateV1>,
    logical_productive_overflow: bool,
    grounded_winner_conflict: bool,
    contradiction_certificate: Option<ContradictionCertificateV1>,
) -> ProductiveCalibratedVerdictV1 {
    let selected = stratum
        .backoff_keys()
        .iter()
        .find_map(|key| table.cells.get(key).filter(|cell| cell.support >= 200))
        .map(|cell| (cell.stratum_id, cell.winner_margin_q16, cell.tie_radius_q16));
    calibrated_readout_selected(
        selected,
        candidates,
        logical_productive_overflow,
        grounded_winner_conflict,
        contradiction_certificate,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn calibrated_readout_packaged(
    selected: Option<(u32, CalibrationCellRecordV1)>,
    candidates: Vec<ReadoutCandidateV1>,
    logical_productive_overflow: bool,
    grounded_winner_conflict: bool,
    contradiction_certificate: Option<ContradictionCertificateV1>,
) -> ProductiveCalibratedVerdictV1 {
    let selected = selected
        .filter(|(_, cell)| cell.support >= MINIMUM_AUTHORITY_GROUPS as u32)
        .map(|(row_id, cell)| {
            (
                row_id,
                (cell.winner_margin_q16 != i32::MIN).then_some(i64::from(cell.winner_margin_q16)),
                i64::from(cell.tie_radius_q16),
            )
        });
    calibrated_readout_selected(
        selected,
        candidates,
        logical_productive_overflow,
        grounded_winner_conflict,
        contradiction_certificate,
    )
}

fn calibrated_readout_selected(
    selected: Option<(u32, Option<i64>, i64)>,
    mut candidates: Vec<ReadoutCandidateV1>,
    logical_productive_overflow: bool,
    grounded_winner_conflict: bool,
    contradiction_certificate: Option<ContradictionCertificateV1>,
) -> ProductiveCalibratedVerdictV1 {
    candidates.sort_by(readout_candidate_order);
    candidates.dedup_by(|left, right| left.identity == right.identity);
    if candidates.is_empty() {
        return ProductiveCalibratedVerdictV1::Abstain {
            suggestions: candidates,
            productive_overflow: logical_productive_overflow,
        };
    }
    let Some((calibration_stratum_id, winner_margin_q16, tie_radius_q16)) = selected else {
        candidates.truncate(MAX_PRODUCTIVE_TIED_OUTPUT);
        return ProductiveCalibratedVerdictV1::Abstain {
            suggestions: candidates,
            productive_overflow: logical_productive_overflow,
        };
    };
    let leader_score = candidates[0].score_q16;
    let mut tied = candidates
        .iter()
        .take_while(|candidate| {
            leader_score
                .checked_sub(candidate.score_q16)
                .is_some_and(|difference| difference <= tie_radius_q16)
        })
        .cloned()
        .collect::<Vec<_>>();
    if logical_productive_overflow || tied.len() > MAX_PRODUCTIVE_TIED_OUTPUT {
        candidates.truncate(MAX_PRODUCTIVE_TIED_OUTPUT);
        return ProductiveCalibratedVerdictV1::Abstain {
            suggestions: candidates,
            productive_overflow: true,
        };
    }
    if tied.len() >= 2 {
        return ProductiveCalibratedVerdictV1::Tied {
            candidates: tied,
            calibration_stratum_id,
        };
    }
    let leader = tied.pop().expect("one leader");
    let margin = candidates
        .get(1)
        .and_then(|second| leader_score.checked_sub(second.score_q16))
        .unwrap_or(i64::MAX);
    let contradiction_valid =
        contradiction_certificate.is_some_and(|certificate| certificate.validate().is_ok());
    if winner_margin_q16.is_some_and(|threshold| margin >= threshold)
        && leader.cross_lemma_ownership_satisfied
        && (!grounded_winner_conflict || contradiction_valid)
    {
        ProductiveCalibratedVerdictV1::Winner {
            candidate: leader,
            calibration_stratum_id,
        }
    } else {
        candidates.truncate(MAX_PRODUCTIVE_TIED_OUTPUT);
        ProductiveCalibratedVerdictV1::Abstain {
            suggestions: candidates,
            productive_overflow: false,
        }
    }
}

fn readout_candidate_order(
    left: &ReadoutCandidateV1,
    right: &ReadoutCandidateV1,
) -> std::cmp::Ordering {
    cross_lane_order(left)
        .cmp(&cross_lane_order(right))
        .then_with(|| right.score_q16.cmp(&left.score_q16))
        .then_with(|| {
            right
                .grounded_lemma_evidence
                .cmp(&left.grounded_lemma_evidence)
        })
        .then_with(|| left.exact_osa_distance.cmp(&right.exact_osa_distance))
        .then_with(|| right.exact_form.cmp(&left.exact_form))
        .then_with(|| {
            candidate_identity_order(
                left.identity,
                &left.normalized_surface,
                right.identity,
                &right.normalized_surface,
            )
        })
}

fn cross_lane_order(candidate: &ReadoutCandidateV1) -> u8 {
    match (candidate.rank_origin, candidate.cross_lane_certified) {
        (CandidateRankOriginV1::BaseV64, _) | (CandidateRankOriginV1::RecoveredV66, true) => 0,
        (CandidateRankOriginV1::RecoveredV66, false) => 1,
    }
}

fn candidate_identity_order(
    left: ProductiveCandidateIdentityV1,
    left_surface: &str,
    right: ProductiveCandidateIdentityV1,
    right_surface: &str,
) -> std::cmp::Ordering {
    left.lemma_id
        .cmp(&right.lemma_id)
        .then_with(|| left.paradigm_id.cmp(&right.paradigm_id))
        .then_with(|| left.target_slot_id.cmp(&right.target_slot_id))
        .then_with(|| left.variant_id.cmp(&right.variant_id))
        .then_with(|| left_surface.cmp(right_surface))
        .then_with(|| left.program_id.cmp(&right.program_id))
        .then_with(|| left.normalized_surface_id.cmp(&right.normalized_surface_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(index: u32) -> ProductiveCandidateIdentityV1 {
        ProductiveCandidateIdentityV1 {
            lemma_id: 1,
            paradigm_id: 1,
            program_id: index,
            target_slot_id: index,
            normalized_surface_id: index,
            variant_id: 1,
        }
    }

    fn stratum(support: u32) -> ObservableCalibrationStratumV1 {
        ObservableCalibrationStratumV1::new(
            "наблюдение",
            "кандидат",
            CandidateProvenanceClassV1::TrainingSeenGenerated,
            support,
            AMBIGUITY_SAME_LEMMA_MULTI_LABEL,
        )
        .expect("stratum")
    }

    fn group(index: usize, margin: i64, valid_leader: bool, support: u32) -> CalibrationGroupV1 {
        let mut group_identity = [0_u8; 32];
        group_identity[0..8].copy_from_slice(&(index as u64).to_le_bytes());
        CalibrationGroupV1 {
            split: ProductiveSplitV1::Calibration,
            group_identity,
            stratum: stratum(support),
            candidates: vec![
                CalibrationCandidateV1 {
                    identity: identity(index as u32 + 1),
                    normalized_surface: format!("leader-{index}"),
                    score_q16: 1_000,
                    grounded_lemma_evidence: 0,
                    exact_osa_distance: 0,
                    exact_form: false,
                    gold_valid: valid_leader,
                },
                CalibrationCandidateV1 {
                    identity: identity(index as u32 + 10_000),
                    normalized_surface: format!("alternative-{index}"),
                    score_q16: 1_000 - margin,
                    grounded_lemma_evidence: 0,
                    exact_osa_distance: 0,
                    exact_form: false,
                    gold_valid: !valid_leader || index % 100 == 0,
                },
            ],
            false_singleton: false,
            grounded_winner_protection_violation: false,
        }
    }

    #[test]
    fn winner_threshold_requires_two_hundred_zero_error_groups() {
        let groups = (0..199)
            .map(|index| group(index, 100, true, 8))
            .collect::<Vec<_>>();
        let table = fit_calibration_table(&groups).expect("table");
        assert!(table
            .cells
            .values()
            .all(|cell| cell.winner_margin_q16.is_none()));

        let groups = (0..220)
            .map(|index| group(index, 300 - index as i64, true, 8))
            .collect::<Vec<_>>();
        let table = fit_calibration_table(&groups).expect("table");
        assert!(table
            .cells
            .values()
            .any(|cell| cell.winner_margin_q16.is_some()));
    }

    #[test]
    fn sparse_exact_stratum_backs_off_in_normative_order() {
        let groups = (0..220)
            .map(|index| group(index, 100, true, (index % 16 + 1) as u32))
            .collect::<Vec<_>>();
        let table = fit_calibration_table(&groups).expect("table");
        let runtime_stratum = stratum(1);
        let selected = runtime_stratum
            .backoff_keys()
            .iter()
            .find_map(|key| table.cells.get(key).filter(|cell| cell.support >= 200))
            .expect("backoff cell");
        assert!(selected.support >= 200);
    }

    #[test]
    fn pava_is_nonincreasing_and_tie_radius_retains_valid_alternatives() {
        let records = vec![(0, true), (1, false), (2, true), (3, false)];
        let calibrated = pava_nonincreasing(&records);
        assert!(calibrated.windows(2).all(|pair| pair[0].1 >= pair[1].1));
        let groups = (0..220)
            .map(|index| group(index, (index % 10) as i64, true, 8))
            .collect::<Vec<_>>();
        let table = fit_calibration_table(&groups).expect("table");
        assert!(table.cells.values().all(|cell| cell.tie_radius_q16 >= 0));
    }

    #[test]
    fn grounded_winner_cannot_be_downgraded_without_certificate() {
        let groups = (0..220)
            .map(|index| group(index, 100, true, 8))
            .collect::<Vec<_>>();
        let table = fit_calibration_table(&groups).expect("table");
        let candidate = ReadoutCandidateV1 {
            identity: identity(1),
            equivalent_identities: vec![identity(1)],
            normalized_surface: "кандидат".to_string(),
            score_q16: 1_000,
            grounded_lemma_evidence: 1,
            exact_osa_distance: 0,
            exact_form: false,
            cross_lemma_ownership_satisfied: true,
            rank_origin: CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        };
        let verdict = calibrated_readout(&table, &stratum(8), vec![candidate], false, true, None);
        assert!(matches!(
            verdict,
            ProductiveCalibratedVerdictV1::Abstain { .. }
        ));
    }

    #[test]
    fn readout_total_order_matches_grounding_geometry_and_exact_provenance() {
        let candidate = |index, grounding, distance, exact| ReadoutCandidateV1 {
            identity: identity(index),
            equivalent_identities: vec![identity(index)],
            normalized_surface: format!("candidate-{index}"),
            score_q16: 1_000,
            grounded_lemma_evidence: grounding,
            exact_osa_distance: distance,
            exact_form: exact,
            cross_lemma_ownership_satisfied: true,
            rank_origin: CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        };
        let mut candidates = vec![
            candidate(4, 8, 2, false),
            candidate(3, 9, 3, false),
            candidate(2, 9, 2, false),
            candidate(1, 9, 2, true),
        ];
        candidates.sort_by(readout_candidate_order);
        assert_eq!(candidates[0].identity, identity(1));
        assert_eq!(candidates[1].identity, identity(2));
        assert_eq!(candidates[2].identity, identity(3));
        assert_eq!(candidates[3].identity, identity(4));
    }

    #[test]
    fn recovered_candidate_requires_a_cross_lane_certificate_to_lead() {
        let base = ReadoutCandidateV1 {
            identity: identity(1),
            equivalent_identities: vec![identity(1)],
            normalized_surface: "base".to_string(),
            score_q16: 100,
            grounded_lemma_evidence: 1,
            exact_osa_distance: 2,
            exact_form: false,
            cross_lemma_ownership_satisfied: true,
            rank_origin: CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        };
        let mut recovered = base.clone();
        recovered.identity = identity(2);
        recovered.equivalent_identities = vec![identity(2)];
        recovered.normalized_surface = "recovered".to_string();
        recovered.score_q16 = 10_000;
        recovered.rank_origin = CandidateRankOriginV1::RecoveredV66;
        let mut uncertified = vec![recovered.clone(), base.clone()];
        uncertified.sort_by(readout_candidate_order);
        assert_eq!(uncertified.remove(0), base);

        recovered.cross_lane_certified = true;
        let mut certified = vec![recovered.clone(), base];
        certified.sort_by(readout_candidate_order);
        assert_eq!(certified.remove(0), recovered);
    }
}
