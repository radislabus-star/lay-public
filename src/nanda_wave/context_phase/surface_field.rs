//! Cold evidence field for damaged lexical surfaces.
//!
//! This is deliberately not a word-replacement table.  It reduces verified
//! `from -> to` receipts to repeated edit geometry: operation, relative
//! position and length band.  The compiler keeps only those compact modes;
//! raw corrections stay in the cold JSONL source.

use std::collections::BTreeMap;
use std::io;

use serde::Deserialize;

const POSITION_BUCKETS: usize = 8;
const MAX_MODES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum SurfaceMutationKind {
    MissingFromTyped,
    AdjacentSwap,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct SurfaceMutationMode {
    kind: SurfaceMutationKind,
    position_bucket: u8,
    length_bucket: u8,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SurfaceMutationField {
    modes: Vec<SurfaceMutationMode>,
    source_rows: usize,
    admitted_rows: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SurfaceMutationFieldReport {
    pub(crate) source_rows: usize,
    pub(crate) admitted_rows: usize,
    pub(crate) mode_count: usize,
}

#[derive(Deserialize)]
struct CorrectionReceipt {
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: String,
}

impl SurfaceMutationField {
    pub(crate) fn from_corrections_jsonl(text: &str, min_support: u32) -> io::Result<Self> {
        let min_support = min_support.max(1);
        let mut counts = BTreeMap::<SurfaceMutationMode, u32>::new();
        let mut source_rows = 0_usize;
        let mut admitted_rows = 0_usize;
        for (line_number, raw) in text.lines().enumerate() {
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            let receipt: CorrectionReceipt = serde_json::from_str(raw).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid correction JSONL at line {}: {error}",
                        line_number + 1
                    ),
                )
            })?;
            source_rows = source_rows.saturating_add(1);
            let Some(mode) = mutation_mode(&receipt.from, &receipt.to) else {
                continue;
            };
            admitted_rows = admitted_rows.saturating_add(1);
            let count = counts.entry(mode).or_default();
            *count = count.saturating_add(1);
        }
        let mut modes = counts
            .into_iter()
            .filter_map(|(mode, support)| (support >= min_support).then_some((mode, support)))
            .collect::<Vec<_>>();
        // Support first makes the field stable under log ordering. The compact
        // tie-break is the phase mode itself, never an observed word.
        modes.sort_by(|(left_mode, left_support), (right_mode, right_support)| {
            right_support
                .cmp(left_support)
                .then_with(|| left_mode.cmp(right_mode))
        });
        modes.truncate(MAX_MODES);
        Ok(Self {
            modes: modes.into_iter().map(|(mode, _)| mode).collect(),
            source_rows,
            admitted_rows,
        })
    }

    pub(crate) fn report(&self) -> SurfaceMutationFieldReport {
        SurfaceMutationFieldReport {
            source_rows: self.source_rows,
            admitted_rows: self.admitted_rows,
            mode_count: self.modes.len(),
        }
    }

    /// Produces only damage geometries observed repeatedly in the cold log.
    /// Candidate identity still comes exclusively from the real L2 readout.
    pub(super) fn damaged_surfaces(&self, target: &str, limit: usize) -> Vec<String> {
        if limit == 0 || !is_single_word(target) {
            return Vec::new();
        }
        let chars = target.chars().collect::<Vec<_>>();
        if chars.len() < 3 {
            return Vec::new();
        }
        let mut surfaces = Vec::with_capacity(limit);
        for mode in &self.modes {
            if length_bucket(chars.len()) != mode.length_bucket {
                continue;
            }
            let position = bucket_position(chars.len(), mode.position_bucket);
            let surface = match mode.kind {
                SurfaceMutationKind::MissingFromTyped => chars
                    .iter()
                    .enumerate()
                    .filter_map(|(index, ch)| (index != position).then_some(*ch))
                    .collect(),
                SurfaceMutationKind::AdjacentSwap => {
                    if position + 1 >= chars.len() {
                        continue;
                    }
                    let mut swapped = chars.clone();
                    swapped.swap(position, position + 1);
                    swapped.into_iter().collect()
                }
            };
            if surface != target && !surfaces.contains(&surface) {
                surfaces.push(surface);
                if surfaces.len() >= limit {
                    break;
                }
            }
        }
        surfaces
    }
}

fn mutation_mode(from: &str, to: &str) -> Option<SurfaceMutationMode> {
    let from = normalized_single_word(from)?;
    let to = normalized_single_word(to)?;
    if from == to || !same_script(&from, &to) {
        return None;
    }
    let source = from.chars().collect::<Vec<_>>();
    let target = to.chars().collect::<Vec<_>>();
    if target.len() == source.len() + 1 {
        let position = one_missing_position(&source, &target)?;
        return Some(SurfaceMutationMode {
            kind: SurfaceMutationKind::MissingFromTyped,
            position_bucket: position_bucket(position, target.len()),
            length_bucket: length_bucket(target.len()),
        });
    }
    if target.len() == source.len() {
        let position = adjacent_swap_position(&source, &target)?;
        return Some(SurfaceMutationMode {
            kind: SurfaceMutationKind::AdjacentSwap,
            position_bucket: position_bucket(position, target.len()),
            length_bucket: length_bucket(target.len()),
        });
    }
    None
}

fn normalized_single_word(value: &str) -> Option<String> {
    let value = crate::typing_memory::normalize_memory_word(value);
    is_single_word(&value).then_some(value)
}

fn is_single_word(value: &str) -> bool {
    !value.is_empty() && value.chars().all(char::is_alphabetic)
}

fn same_script(left: &str, right: &str) -> bool {
    let left_cyrillic = left.chars().all(crate::keyboard::is_cyrillic_letter);
    let right_cyrillic = right.chars().all(crate::keyboard::is_cyrillic_letter);
    let left_ascii = left.chars().all(|ch| ch.is_ascii_alphabetic());
    let right_ascii = right.chars().all(|ch| ch.is_ascii_alphabetic());
    (left_cyrillic && right_cyrillic) || (left_ascii && right_ascii)
}

fn one_missing_position(source: &[char], target: &[char]) -> Option<usize> {
    let mut source_index = 0_usize;
    let mut skipped = None;
    for (target_index, ch) in target.iter().enumerate() {
        if source.get(source_index) == Some(ch) {
            source_index += 1;
        } else if skipped.replace(target_index).is_some() {
            return None;
        }
    }
    (source_index == source.len()).then_some(skipped.unwrap_or(target.len() - 1))
}

fn adjacent_swap_position(source: &[char], target: &[char]) -> Option<usize> {
    let mismatches = source
        .iter()
        .zip(target)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    if mismatches.len() != 2 || mismatches[1] != mismatches[0] + 1 {
        return None;
    }
    let index = mismatches[0];
    (source[index] == target[index + 1] && source[index + 1] == target[index]).then_some(index)
}

fn position_bucket(position: usize, length: usize) -> u8 {
    ((position.saturating_mul(POSITION_BUCKETS) / length.max(1)).min(POSITION_BUCKETS - 1)) as u8
}

fn bucket_position(length: usize, bucket: u8) -> usize {
    // A bucket records an interval. Reconstruct its midpoint instead of its
    // left edge, otherwise early buckets systematically move edits left.
    (usize::from(bucket)
        .saturating_mul(length)
        .saturating_add(POSITION_BUCKETS / 2)
        / POSITION_BUCKETS)
        .min(length.saturating_sub(1))
}

fn length_bucket(length: usize) -> u8 {
    length.min(18) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_repeated_generic_missing_geometry() {
        let field = SurfaceMutationField::from_corrections_jsonl(
            concat!(
                r#"{"from":"время","to":"времени"}"#,
                "\n",
                r#"{"from":"форм","to":"форма"}"#,
                "\n",
                r#"{"from":"карт","to":"карта"}"#,
            ),
            2,
        )
        .unwrap();
        assert_eq!(field.report().source_rows, 3);
        assert_eq!(field.report().admitted_rows, 2);
        assert_eq!(field.report().mode_count, 1);
        assert!(field.damaged_surfaces("переподключаю", 4).len() <= 1);
    }

    #[test]
    fn derives_transposition_without_storing_words() {
        let field = SurfaceMutationField::from_corrections_jsonl(
            concat!(
                r#"{"from":"врмея","to":"время"}"#,
                "\n",
                r#"{"from":"срко","to":"срок"}"#,
            ),
            1,
        )
        .unwrap();
        assert!(field
            .damaged_surfaces("время", 4)
            .iter()
            .any(|surface| surface == "врмея"));
    }

    #[test]
    fn excludes_layout_and_multiword_receipts() {
        let field = SurfaceMutationField::from_corrections_jsonl(
            concat!(
                r#"{"from":"djn","to":"вот"}"#,
                "\n",
                r#"{"from":"я был","to":"ябыл"}"#,
            ),
            1,
        )
        .unwrap();
        assert_eq!(field.report().admitted_rows, 0);
        assert_eq!(field.report().mode_count, 0);
    }
}
