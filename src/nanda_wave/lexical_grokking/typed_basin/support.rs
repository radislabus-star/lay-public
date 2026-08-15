use rayon::prelude::*;

use super::super::atoms::encode_wave_surface;
use super::super::format;
use super::super::model::LexicalGrokkingPackage;

const OVERFLOW_HEADER_BYTES: usize = 32;
const OVERFLOW_ENTRY_BYTES: usize = 8;

#[derive(Clone, Debug)]
pub(in crate::nanda_wave::lexical_grokking) struct ExactSupportField {
    values: Vec<u32>,
    pub(in crate::nanda_wave::lexical_grokking) metrics: ExactSupportMetrics,
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::nanda_wave::lexical_grokking) struct ExactSupportMetrics {
    pub(in crate::nanda_wave::lexical_grokking) centers_decoded: usize,
    pub(in crate::nanda_wave::lexical_grokking) corpus_surface_mismatches: usize,
    pub(in crate::nanda_wave::lexical_grokking) encoded_atom_occurrences: u64,
    pub(in crate::nanda_wave::lexical_grokking) stored_saturated_atoms: usize,
    pub(in crate::nanda_wave::lexical_grokking) exact_overflow_atoms: usize,
    pub(in crate::nanda_wave::lexical_grokking) stored_support_mismatches: usize,
    pub(in crate::nanda_wave::lexical_grokking) maximum_exact_support: u32,
    pub(in crate::nanda_wave::lexical_grokking) projected_overflow_bytes: usize,
}

#[derive(Debug)]
struct SupportShard {
    values: Vec<u32>,
    centers_decoded: usize,
    corpus_surface_mismatches: usize,
    encoded_atom_occurrences: u64,
}

impl ExactSupportField {
    pub(in crate::nanda_wave::lexical_grokking) fn rebuild(
        package: &LexicalGrokkingPackage,
        expected_surfaces: &[String],
    ) -> Result<Self, String> {
        Self::rebuild_inner(package, Some(expected_surfaces))
    }

    pub(in crate::nanda_wave::lexical_grokking) fn rebuild_decoded(
        package: &LexicalGrokkingPackage,
    ) -> Result<Self, String> {
        Self::rebuild_inner(package, None)
    }

    fn rebuild_inner(
        package: &LexicalGrokkingPackage,
        expected_surfaces: Option<&[String]>,
    ) -> Result<Self, String> {
        if package.centers.is_empty() || package.atoms.is_empty() {
            return Err("exact support requires non-empty center and atom fields".to_string());
        }
        if let Some(expected_surfaces) = expected_surfaces {
            if expected_surfaces.len() != package.centers.len() {
                return Err(format!(
                    "exact support corpus/package terminal count differs: {} != {}",
                    expected_surfaces.len(),
                    package.centers.len()
                ));
            }
        }
        let workers = rayon::current_num_threads()
            .max(1)
            .min(package.centers.len());
        let chunk_size = package.centers.len().div_ceil(workers);
        let shards = package
            .centers
            .par_chunks(chunk_size)
            .enumerate()
            .map(|(shard_index, centers)| {
                let mut values = vec![0_u32; package.atoms.len()];
                let mut corpus_surface_mismatches = 0_usize;
                let mut encoded_atom_occurrences = 0_u64;
                let start = shard_index.saturating_mul(chunk_size);
                for (offset, center) in centers.iter().copied().enumerate() {
                    let surface = format::decode_center_surface(center, &package.decoder_nodes)?;
                    if let Some(expected_surfaces) = expected_surfaces {
                        corpus_surface_mismatches = corpus_surface_mismatches.saturating_add(
                            usize::from(expected_surfaces.get(start + offset) != Some(&surface)),
                        );
                    }
                    for atom in encode_wave_surface(&surface) {
                        let atom_id = package.graph.atom_id(atom.key).ok_or_else(|| {
                            format!(
                                "decoded center atom is absent from NGramGraph: channel={:?}",
                                atom.key.channel
                            )
                        })? as usize;
                        let slot = values.get_mut(atom_id).ok_or_else(|| {
                            "decoded center atom exceeds exact support field".to_string()
                        })?;
                        *slot = slot
                            .checked_add(1)
                            .ok_or_else(|| "exact atom support exceeds u32".to_string())?;
                        encoded_atom_occurrences =
                            encoded_atom_occurrences.checked_add(1).ok_or_else(|| {
                                "encoded atom occurrence count exceeds u64".to_string()
                            })?;
                    }
                }
                Ok(SupportShard {
                    values,
                    centers_decoded: centers.len(),
                    corpus_surface_mismatches,
                    encoded_atom_occurrences,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut values = vec![0_u32; package.atoms.len()];
        let mut centers_decoded = 0_usize;
        let mut corpus_surface_mismatches = 0_usize;
        let mut encoded_atom_occurrences = 0_u64;
        for shard in shards {
            centers_decoded = centers_decoded
                .checked_add(shard.centers_decoded)
                .ok_or_else(|| "exact support center count exceeds usize".to_string())?;
            corpus_surface_mismatches = corpus_surface_mismatches
                .checked_add(shard.corpus_surface_mismatches)
                .ok_or_else(|| "corpus surface mismatch count exceeds usize".to_string())?;
            encoded_atom_occurrences = encoded_atom_occurrences
                .checked_add(shard.encoded_atom_occurrences)
                .ok_or_else(|| "encoded atom occurrence count exceeds u64".to_string())?;
            for (target, value) in values.iter_mut().zip(shard.values) {
                *target = target
                    .checked_add(value)
                    .ok_or_else(|| "merged exact atom support exceeds u32".to_string())?;
            }
        }
        if centers_decoded != package.centers.len() {
            return Err(format!(
                "exact support center count differs: {centers_decoded} != {}",
                package.centers.len()
            ));
        }

        let metrics = metrics_for_values(
            package,
            &values,
            centers_decoded,
            corpus_surface_mismatches,
            encoded_atom_occurrences,
        );
        Ok(Self { values, metrics })
    }

    pub(in crate::nanda_wave::lexical_grokking) fn from_compact_overflow(
        package: &LexicalGrokkingPackage,
        overflow: &[(u32, u32)],
    ) -> Result<Self, String> {
        if package.atoms.is_empty() {
            return Err("exact support requires a non-empty atom field".to_string());
        }
        let mut values = package
            .atoms
            .iter()
            .map(|record| u32::from(record.support))
            .collect::<Vec<_>>();
        let mut previous = None;
        for &(atom_id, exact_support) in overflow {
            if previous.is_some_and(|previous| atom_id <= previous) {
                return Err("V9 exact-support AtomIds are not sorted and unique".to_string());
            }
            previous = Some(atom_id);
            if exact_support <= u32::from(u16::MAX) {
                return Err(format!(
                    "V9 exact-support overflow is not above u16: atom={atom_id} support={exact_support}"
                ));
            }
            let record = package
                .atoms
                .get(atom_id as usize)
                .ok_or_else(|| format!("V9 exact-support AtomId is invalid: {atom_id}"))?;
            if record.support != u16::MAX {
                return Err(format!(
                    "V9 exact-support overflow references an unsaturated atom: {atom_id}"
                ));
            }
            values[atom_id as usize] = exact_support;
        }
        let metrics = metrics_for_values(package, &values, 0, 0, 0);
        Ok(Self { values, metrics })
    }

    pub(in crate::nanda_wave::lexical_grokking) fn overflow_entries(&self) -> Vec<(u32, u32)> {
        self.values
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, support)| *support > u32::from(u16::MAX))
            .map(|(atom_id, support)| (atom_id as u32, support))
            .collect()
    }

    pub(in crate::nanda_wave::lexical_grokking) fn resident_bytes(&self) -> usize {
        self.values
            .capacity()
            .saturating_mul(std::mem::size_of::<u32>())
    }

    pub(in crate::nanda_wave::lexical_grokking) fn get(&self, atom_id: u32) -> Option<u32> {
        self.values.get(atom_id as usize).copied()
    }

    pub(in crate::nanda_wave::lexical_grokking) fn values(&self) -> &[u32] {
        &self.values
    }
}

fn metrics_for_values(
    package: &LexicalGrokkingPackage,
    values: &[u32],
    centers_decoded: usize,
    corpus_surface_mismatches: usize,
    encoded_atom_occurrences: u64,
) -> ExactSupportMetrics {
    let mut metrics = ExactSupportMetrics {
        centers_decoded,
        corpus_surface_mismatches,
        encoded_atom_occurrences,
        ..ExactSupportMetrics::default()
    };
    for (record, exact) in package.atoms.iter().zip(values) {
        metrics.stored_saturated_atoms += usize::from(record.support == u16::MAX);
        metrics.exact_overflow_atoms += usize::from(*exact > u32::from(u16::MAX));
        metrics.maximum_exact_support = metrics.maximum_exact_support.max(*exact);
        let stored_expected = (*exact).min(u32::from(u16::MAX)) as u16;
        metrics.stored_support_mismatches += usize::from(record.support != stored_expected);
    }
    metrics.projected_overflow_bytes = OVERFLOW_HEADER_BYTES.saturating_add(
        metrics
            .exact_overflow_atoms
            .saturating_mul(OVERFLOW_ENTRY_BYTES),
    );
    metrics
}
