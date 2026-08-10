use std::borrow::Cow;
use std::collections::{BTreeSet, BinaryHeap, HashMap};
use std::ops::Range;

use super::model::{L2FieldPackage, LemmaCenter, MorphBinding};
use super::package_bytes::PackageBytes;
use super::runtime_storage::RuntimeL2Package;
use super::CANONICAL_L2_ATOM_RELATION_LIMIT;

const WAVE_BAND_BITS: usize = 16;
const WAVE_BANDS: usize = 128 / WAVE_BAND_BITS;
const WAVE_BAND_VALUES: usize = 1 << WAVE_BAND_BITS;
const WAVE_BAND_KEYS: usize = WAVE_BANDS * WAVE_BAND_VALUES;
const MAX_LEMMA_WAVE_CENTERS: usize = 8;
const ATOM_HASH_MASK: u64 = (1_u64 << 56) - 1;
const ATOM_CHARACTER_BIGRAM: u8 = 1;
const ATOM_CHARACTER_TRIGRAM: u8 = 2;
const ATOM_KEYBOARD_BIGRAM: u8 = 3;
const ATOM_KEYBOARD_TRIGRAM: u8 = 4;
const ATOM_CHARACTER_BAG_TRIGRAM: u8 = 5;
const ATOM_KEYBOARD_BAG_TRIGRAM: u8 = 6;
const ATOM_CHARACTER_SKIP_GRAM: u8 = 7;
const ATOM_KEYBOARD_SKIP_GRAM: u8 = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SurfaceWaveCode {
    pub(super) character: u64,
    pub(super) keyboard: u64,
}

impl SurfaceWaveCode {
    pub(super) fn distance(self, other: Self) -> u16 {
        ((self.character ^ other.character).count_ones()
            + (self.keyboard ^ other.keyboard).count_ones()) as u16
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct LemmaWaveRange {
    pub(super) start: u32,
    pub(super) count: u16,
    pub(super) minimum_length: u8,
    pub(super) maximum_length: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LemmaWaveMatch {
    pub(super) lemma_id: u32,
    pub(super) wave_distance: u16,
    pub(super) atom_evidence: u32,
    pub(super) atom_evidence_milli: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SurfaceAtomProfile {
    atoms: Vec<u64>,
    total_weight: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SurfaceScoringProfile {
    normalized: String,
    characters: Vec<u32>,
    keyboard: Vec<u32>,
}

impl SurfaceScoringProfile {
    pub(super) fn normalized(&self) -> &str {
        &self.normalized
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LemmaWaveIndex {
    ranges: Vec<LemmaWaveRange>,
    centers: Vec<SurfaceWaveCode>,
    band_offsets: Vec<u32>,
    band_postings: Vec<u32>,
    atom_keys: Vec<u64>,
    atom_offsets: Vec<u32>,
    atom_postings: Vec<u8>,
    atom_degrees: Vec<u32>,
}

#[derive(Clone, Debug)]
pub(super) struct CompactLemmaWaveIndexView {
    bytes: PackageBytes,
    range_section: Range<usize>,
    center_section: Range<usize>,
    band_offset_section: Range<usize>,
    band_posting_section: Range<usize>,
    atom_key_section: Range<usize>,
    atom_offset_section: Range<usize>,
    atom_posting_section: Range<usize>,
    atom_degrees: Vec<u32>,
}

#[derive(Clone, Debug)]
pub(super) enum RuntimeLemmaWaveIndex {
    Owned(LemmaWaveIndex),
    Compact(CompactLemmaWaveIndexView),
}

impl RuntimeLemmaWaveIndex {
    pub(super) fn from_owned(index: LemmaWaveIndex) -> Self {
        Self::Owned(index)
    }

    pub(super) fn rank_lemmas_with_atom_relation_limit(
        &self,
        surface: &str,
        limit: usize,
        atom_relation_limit: usize,
    ) -> Vec<LemmaWaveMatch> {
        match self {
            Self::Owned(index) => {
                index.rank_lemmas_with_atom_relation_limit(surface, limit, atom_relation_limit)
            }
            Self::Compact(index) => {
                rank_lemmas_with_atom_relation_limit(index, surface, limit, atom_relation_limit)
            }
        }
    }

    pub(super) fn owned_resident_bytes(&self) -> usize {
        match self {
            Self::Owned(index) => index.resident_bytes(),
            Self::Compact(index) => index.owned_resident_bytes(),
        }
    }

    pub(super) fn backing_view_bytes(&self) -> usize {
        match self {
            Self::Owned(_) => 0,
            Self::Compact(index) => index.view_bytes(),
        }
    }

    pub(super) fn atom_key_count(&self) -> usize {
        match self {
            Self::Owned(index) => index.atom_keys.len(),
            Self::Compact(index) => index.atom_key_count(),
        }
    }
}

impl LemmaWaveIndex {
    pub(super) fn build(package: &RuntimeL2Package) -> Result<Self, String> {
        Self::build_from_source(package)
    }

    pub(super) fn build_reference(package: &L2FieldPackage) -> Result<Self, String> {
        Self::build_from_source(package)
    }

    fn build_from_source(source: &impl LemmaWaveSource) -> Result<Self, String> {
        let lemmas = source.lemma_centers();
        let workers = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .min(lemmas.len().max(1));
        let chunk_size = lemmas.len().div_ceil(workers).max(1);
        let mut built = std::thread::scope(|scope| {
            let handles = lemmas
                .chunks(chunk_size)
                .enumerate()
                .map(|(chunk_index, chunk)| {
                    let first_lemma = chunk_index * chunk_size;
                    scope.spawn(move || {
                        chunk
                            .iter()
                            .enumerate()
                            .map(|(offset, lemma)| {
                                build_lemma_wave_row(source, first_lemma + offset, lemma)
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                })
                .collect::<Vec<_>>();
            let mut built = Vec::with_capacity(lemmas.len());
            for handle in handles {
                let rows = handle
                    .join()
                    .map_err(|_| "L2 lemma wave build worker panicked".to_string())??;
                built.extend(rows);
            }
            Ok::<_, String>(built)
        })?;
        built.sort_unstable_by_key(|row| row.lemma_id);

        let mut ranges = Vec::with_capacity(lemmas.len());
        let mut centers = Vec::new();
        for row in &built {
            let range_start = u32::try_from(centers.len())
                .map_err(|_| "L2 lemma wave center index exceeds u32".to_string())?;
            let count = u16::try_from(row.centers.len())
                .map_err(|_| "L2 lemma wave center count exceeds u16".to_string())?;
            centers.extend_from_slice(&row.centers);
            ranges.push(LemmaWaveRange {
                start: range_start,
                count,
                minimum_length: row.minimum_length,
                maximum_length: row.maximum_length,
            });
        }
        let (atom_keys, atom_offsets, atom_postings) = build_atom_postings(&built)?;
        let (band_offsets, band_postings) = build_band_postings(&ranges, &centers)?;
        Self::from_parts(
            ranges,
            centers,
            band_offsets,
            band_postings,
            atom_keys,
            atom_offsets,
            atom_postings,
        )
    }

    pub(super) fn from_parts(
        ranges: Vec<LemmaWaveRange>,
        centers: Vec<SurfaceWaveCode>,
        band_offsets: Vec<u32>,
        band_postings: Vec<u32>,
        atom_keys: Vec<u64>,
        atom_offsets: Vec<u32>,
        atom_postings: Vec<u8>,
    ) -> Result<Self, String> {
        let mut expected_start = 0_usize;
        for (lemma_id, range) in ranges.iter().enumerate() {
            if range.count == 0 || range.minimum_length > range.maximum_length {
                return Err(format!("L2 lemma wave range {lemma_id} is invalid"));
            }
            if range.start as usize != expected_start {
                return Err(format!(
                    "L2 lemma wave range {lemma_id} starts at {}, expected {expected_start}",
                    range.start
                ));
            }
            expected_start = expected_start
                .checked_add(range.count as usize)
                .ok_or_else(|| "L2 lemma wave center range overflow".to_string())?;
            if expected_start > centers.len() {
                return Err(format!(
                    "L2 lemma wave range {lemma_id} exceeds center section"
                ));
            }
        }
        if expected_start != centers.len() {
            return Err(format!(
                "L2 lemma wave ranges cover {expected_start} of {} centers",
                centers.len()
            ));
        }
        validate_band_postings(ranges.len(), &band_offsets, &band_postings)?;
        let atom_degrees =
            validate_atom_postings(ranges.len(), &atom_keys, &atom_offsets, &atom_postings)?;
        Ok(Self {
            ranges,
            centers,
            band_offsets,
            band_postings,
            atom_keys,
            atom_offsets,
            atom_postings,
            atom_degrees,
        })
    }

    pub(super) fn ranges(&self) -> &[LemmaWaveRange] {
        &self.ranges
    }

    pub(super) fn centers(&self) -> &[SurfaceWaveCode] {
        &self.centers
    }

    pub(super) fn band_offsets(&self) -> &[u32] {
        &self.band_offsets
    }

    pub(super) fn band_postings(&self) -> &[u32] {
        &self.band_postings
    }

    pub(super) fn atom_keys(&self) -> &[u64] {
        &self.atom_keys
    }

    pub(super) fn atom_offsets(&self) -> &[u32] {
        &self.atom_offsets
    }

    pub(super) fn atom_postings(&self) -> &[u8] {
        &self.atom_postings
    }

    pub(super) fn rank_lemmas(&self, surface: &str, limit: usize) -> Vec<LemmaWaveMatch> {
        self.rank_lemmas_with_atom_relation_limit(surface, limit, CANONICAL_L2_ATOM_RELATION_LIMIT)
    }

    pub(super) fn rank_lemmas_with_atom_relation_limit(
        &self,
        surface: &str,
        limit: usize,
        atom_relation_limit: usize,
    ) -> Vec<LemmaWaveMatch> {
        rank_lemmas_with_atom_relation_limit(self, surface, limit, atom_relation_limit)
    }

    pub(super) fn resident_bytes(&self) -> usize {
        self.ranges.capacity() * std::mem::size_of::<LemmaWaveRange>()
            + self.centers.capacity() * std::mem::size_of::<SurfaceWaveCode>()
            + self.band_offsets.capacity() * std::mem::size_of::<u32>()
            + self.band_postings.capacity() * std::mem::size_of::<u32>()
            + self.atom_keys.capacity() * std::mem::size_of::<u64>()
            + self.atom_offsets.capacity() * std::mem::size_of::<u32>()
            + self.atom_postings.capacity()
            + self.atom_degrees.capacity() * std::mem::size_of::<u32>()
    }

    #[cfg(test)]
    fn center_count(&self) -> usize {
        self.centers.len()
    }

    #[cfg(test)]
    fn rank_lemmas_exhaustive(&self, surface: &str, limit: usize) -> Vec<LemmaWaveMatch> {
        let normalized = normalize_surface(surface);
        let input_length = normalized.chars().count().min(u8::MAX as usize) as u8;
        let input = surface_wave_code(&normalized);
        rank_candidate_lemmas(
            self,
            input,
            input_length,
            limit,
            &(0..self.ranges.len() as u32).collect::<Vec<_>>(),
        )
    }

    #[cfg(test)]
    fn band_candidate_lemmas(&self, input: SurfaceWaveCode, radius: usize) -> Vec<u32> {
        band_candidate_lemmas(self, input, radius)
    }
}

impl CompactLemmaWaveIndexView {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_sections(
        bytes: PackageBytes,
        range_section: Range<usize>,
        center_section: Range<usize>,
        band_offset_section: Range<usize>,
        band_posting_section: Range<usize>,
        atom_key_section: Range<usize>,
        atom_offset_section: Range<usize>,
        atom_posting_section: Range<usize>,
    ) -> Result<Self, String> {
        let mut view = Self {
            bytes,
            range_section,
            center_section,
            band_offset_section,
            band_posting_section,
            atom_key_section,
            atom_offset_section,
            atom_posting_section,
            atom_degrees: Vec::new(),
        };
        view.validate_section_widths()?;
        view.atom_degrees = validate_read_index(&view)?;
        Ok(view)
    }

    pub(super) fn range_count(&self) -> usize {
        self.range_section.len() / super::compositional_format::LEMMA_WAVE_RANGE_BYTES
    }

    pub(super) fn atom_key_count(&self) -> usize {
        self.atom_key_section.len() / super::compositional_format::ATOM_KEY_BYTES
    }

    pub(super) fn view_bytes(&self) -> usize {
        self.range_section.len()
            + self.center_section.len()
            + self.band_offset_section.len()
            + self.band_posting_section.len()
            + self.atom_key_section.len()
            + self.atom_offset_section.len()
            + self.atom_posting_section.len()
    }

    pub(super) fn owned_resident_bytes(&self) -> usize {
        self.atom_degrees.capacity() * std::mem::size_of::<u32>()
    }

    fn center_count(&self) -> usize {
        self.center_section.len() / super::compositional_format::SURFACE_WAVE_CODE_BYTES
    }

    fn band_offset_count(&self) -> usize {
        self.band_offset_section.len() / super::compositional_format::WAVE_BAND_OFFSET_BYTES
    }

    fn band_posting_count(&self) -> usize {
        self.band_posting_section.len() / super::compositional_format::WAVE_BAND_POSTING_BYTES
    }

    fn atom_offset_count(&self) -> usize {
        self.atom_offset_section.len() / super::compositional_format::ATOM_OFFSET_BYTES
    }

    fn validate_section_widths(&self) -> Result<(), String> {
        let sections = [
            (
                "lemma wave ranges",
                self.range_section.len(),
                super::compositional_format::LEMMA_WAVE_RANGE_BYTES,
            ),
            (
                "surface wave codes",
                self.center_section.len(),
                super::compositional_format::SURFACE_WAVE_CODE_BYTES,
            ),
            (
                "wave band offsets",
                self.band_offset_section.len(),
                super::compositional_format::WAVE_BAND_OFFSET_BYTES,
            ),
            (
                "wave band postings",
                self.band_posting_section.len(),
                super::compositional_format::WAVE_BAND_POSTING_BYTES,
            ),
            (
                "typed atom keys",
                self.atom_key_section.len(),
                super::compositional_format::ATOM_KEY_BYTES,
            ),
            (
                "typed atom offsets",
                self.atom_offset_section.len(),
                super::compositional_format::ATOM_OFFSET_BYTES,
            ),
            (
                "typed atom postings",
                self.atom_posting_section.len(),
                super::compositional_format::ATOM_POSTING_BYTES,
            ),
        ];
        for (name, bytes, width) in sections {
            if bytes % width != 0 {
                return Err(format!("compact L2 {name} section width mismatch"));
            }
        }
        Ok(())
    }

    fn item(&self, section: &Range<usize>, index: usize, width: usize) -> Option<&[u8]> {
        let start = section.start.checked_add(index.checked_mul(width)?)?;
        let end = start.checked_add(width)?;
        if end > section.end {
            return None;
        }
        self.bytes.as_slice().get(start..end)
    }
}

trait LemmaWaveReadIndex {
    fn lemma_count(&self) -> usize;
    fn center_count(&self) -> usize;
    fn range(&self, index: usize) -> Option<LemmaWaveRange>;
    fn center(&self, index: usize) -> Option<SurfaceWaveCode>;
    fn band_offset_count(&self) -> usize;
    fn band_posting_count(&self) -> usize;
    fn band_offset(&self, index: usize) -> Option<u32>;
    fn band_posting(&self, index: usize) -> Option<u32>;
    fn atom_key_count(&self) -> usize;
    fn atom_offset_count(&self) -> usize;
    fn atom_posting_bytes(&self) -> usize;
    fn atom_key(&self, index: usize) -> Option<u64>;
    fn atom_offset(&self, index: usize) -> Option<u32>;
    fn atom_posting_slice(&self, start: usize, end: usize) -> Option<&[u8]>;

    fn atom_posting(&self, index: usize) -> Option<&[u8]> {
        let start = self.atom_offset(index)? as usize;
        let end = self.atom_offset(index + 1)? as usize;
        self.atom_posting_slice(start, end)
    }

    fn atom_degree(&self, index: usize) -> Option<usize> {
        let mut degree = 0_usize;
        decode_delta_postings(self.atom_posting(index)?, |_| {
            degree = degree.saturating_add(1);
        })
        .ok()?;
        Some(degree)
    }
}

impl LemmaWaveReadIndex for LemmaWaveIndex {
    fn lemma_count(&self) -> usize {
        self.ranges.len()
    }

    fn center_count(&self) -> usize {
        self.centers.len()
    }

    fn range(&self, index: usize) -> Option<LemmaWaveRange> {
        self.ranges.get(index).copied()
    }

    fn center(&self, index: usize) -> Option<SurfaceWaveCode> {
        self.centers.get(index).copied()
    }

    fn band_offset_count(&self) -> usize {
        self.band_offsets.len()
    }

    fn band_posting_count(&self) -> usize {
        self.band_postings.len()
    }

    fn band_offset(&self, index: usize) -> Option<u32> {
        self.band_offsets.get(index).copied()
    }

    fn band_posting(&self, index: usize) -> Option<u32> {
        self.band_postings.get(index).copied()
    }

    fn atom_key_count(&self) -> usize {
        self.atom_keys.len()
    }

    fn atom_offset_count(&self) -> usize {
        self.atom_offsets.len()
    }

    fn atom_posting_bytes(&self) -> usize {
        self.atom_postings.len()
    }

    fn atom_key(&self, index: usize) -> Option<u64> {
        self.atom_keys.get(index).copied()
    }

    fn atom_offset(&self, index: usize) -> Option<u32> {
        self.atom_offsets.get(index).copied()
    }

    fn atom_posting_slice(&self, start: usize, end: usize) -> Option<&[u8]> {
        self.atom_postings.get(start..end)
    }

    fn atom_degree(&self, index: usize) -> Option<usize> {
        usize::try_from(*self.atom_degrees.get(index)?).ok()
    }
}

impl LemmaWaveReadIndex for CompactLemmaWaveIndexView {
    fn lemma_count(&self) -> usize {
        self.range_count()
    }

    fn center_count(&self) -> usize {
        self.center_count()
    }

    fn range(&self, index: usize) -> Option<LemmaWaveRange> {
        let bytes = self.item(
            &self.range_section,
            index,
            super::compositional_format::LEMMA_WAVE_RANGE_BYTES,
        )?;
        Some(LemmaWaveRange {
            start: u32::from_le_bytes(bytes[0..4].try_into().ok()?),
            count: u16::from_le_bytes(bytes[4..6].try_into().ok()?),
            minimum_length: bytes[6],
            maximum_length: bytes[7],
        })
    }

    fn center(&self, index: usize) -> Option<SurfaceWaveCode> {
        let bytes = self.item(
            &self.center_section,
            index,
            super::compositional_format::SURFACE_WAVE_CODE_BYTES,
        )?;
        Some(SurfaceWaveCode {
            character: u64::from_le_bytes(bytes[0..8].try_into().ok()?),
            keyboard: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        })
    }

    fn band_offset_count(&self) -> usize {
        self.band_offset_count()
    }

    fn band_posting_count(&self) -> usize {
        self.band_posting_count()
    }

    fn band_offset(&self, index: usize) -> Option<u32> {
        let bytes = self.item(
            &self.band_offset_section,
            index,
            super::compositional_format::WAVE_BAND_OFFSET_BYTES,
        )?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    }

    fn band_posting(&self, index: usize) -> Option<u32> {
        let bytes = self.item(
            &self.band_posting_section,
            index,
            super::compositional_format::WAVE_BAND_POSTING_BYTES,
        )?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    }

    fn atom_key_count(&self) -> usize {
        self.atom_key_count()
    }

    fn atom_offset_count(&self) -> usize {
        self.atom_offset_count()
    }

    fn atom_posting_bytes(&self) -> usize {
        self.atom_posting_section.len()
    }

    fn atom_key(&self, index: usize) -> Option<u64> {
        let bytes = self.item(
            &self.atom_key_section,
            index,
            super::compositional_format::ATOM_KEY_BYTES,
        )?;
        Some(u64::from_le_bytes(bytes.try_into().ok()?))
    }

    fn atom_offset(&self, index: usize) -> Option<u32> {
        let bytes = self.item(
            &self.atom_offset_section,
            index,
            super::compositional_format::ATOM_OFFSET_BYTES,
        )?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    }

    fn atom_posting_slice(&self, start: usize, end: usize) -> Option<&[u8]> {
        if start > end || end > self.atom_posting_section.len() {
            return None;
        }
        let absolute_start = self.atom_posting_section.start.checked_add(start)?;
        let absolute_end = self.atom_posting_section.start.checked_add(end)?;
        self.bytes.as_slice().get(absolute_start..absolute_end)
    }

    fn atom_degree(&self, index: usize) -> Option<usize> {
        usize::try_from(*self.atom_degrees.get(index)?).ok()
    }
}

fn rank_lemmas_with_atom_relation_limit(
    index: &impl LemmaWaveReadIndex,
    surface: &str,
    limit: usize,
    atom_relation_limit: usize,
) -> Vec<LemmaWaveMatch> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = normalize_surface(surface);
    let input_length = normalized.chars().count().min(u8::MAX as usize) as u8;
    if input_length < 2 {
        return Vec::new();
    }
    let input = surface_wave_code(&normalized);
    let atom_ranked = rank_atom_lemmas(
        index,
        &normalized,
        input,
        input_length,
        limit,
        atom_relation_limit,
    );
    if atom_ranked.len() >= limit {
        return atom_ranked;
    }
    let mut candidates = band_candidate_lemmas(index, input, 2);
    let mut ranked = rank_candidate_lemmas(index, input, input_length, limit, &candidates);
    if ranked.len() >= limit && ranked.last().is_some_and(|item| item.wave_distance <= 23) {
        return merge_lemma_matches(atom_ranked, ranked, limit);
    }
    candidates = band_candidate_lemmas(index, input, 3);
    ranked = rank_candidate_lemmas(index, input, input_length, limit, &candidates);
    if ranked.len() >= limit && ranked.last().is_some_and(|item| item.wave_distance <= 31) {
        return merge_lemma_matches(atom_ranked, ranked, limit);
    }
    let exhaustive = rank_candidate_lemmas(
        index,
        input,
        input_length,
        limit,
        &(0..index.lemma_count() as u32).collect::<Vec<_>>(),
    );
    merge_lemma_matches(atom_ranked, exhaustive, limit)
}

fn rank_atom_lemmas(
    index: &impl LemmaWaveReadIndex,
    surface: &str,
    input: SurfaceWaveCode,
    input_length: u8,
    limit: usize,
    atom_relation_limit: usize,
) -> Vec<LemmaWaveMatch> {
    let atoms = surface_atom_keys(surface);
    let mut active_atoms = atoms
        .into_iter()
        .filter_map(|atom| {
            let atom_index = find_atom_key(index, atom)?;
            let degree = index.atom_degree(atom_index)?.max(1);
            let inverse_degree = (index.lemma_count().max(1) / degree).max(1);
            let evidence = u32::from(atom_weight(atom))
                .saturating_mul(inverse_degree.ilog2().saturating_add(1));
            Some((atom, atom_index, degree, evidence))
        })
        .collect::<Vec<_>>();
    active_atoms.sort_unstable_by(|left, right| {
        let left_density = u64::from(left.3).saturating_mul(right.2 as u64);
        let right_density = u64::from(right.3).saturating_mul(left.2 as u64);
        right_density
            .cmp(&left_density)
            .then_with(|| right.3.cmp(&left.3))
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut selected_relations = 0_usize;
    active_atoms.retain(|(_, _, degree, _)| {
        let next = selected_relations.saturating_add(*degree);
        if next > atom_relation_limit {
            false
        } else {
            selected_relations = next;
            true
        }
    });
    if active_atoms.is_empty() {
        return Vec::new();
    }

    let total_evidence = active_atoms
        .iter()
        .map(|(_, _, _, evidence)| *evidence)
        .sum::<u32>()
        .max(1);
    let mut scores = vec![0_u32; index.lemma_count()];
    let mut touched = Vec::with_capacity(selected_relations.min(index.lemma_count()));
    for (_, atom_index, _, evidence) in active_atoms {
        let Some(posting) = index.atom_posting(atom_index) else {
            continue;
        };
        let _ = decode_delta_postings(posting, |lemma_id| {
            let score = &mut scores[lemma_id as usize];
            if *score == 0 {
                touched.push(lemma_id);
            }
            *score = score.saturating_add(evidence);
        });
    }
    let mut ranked = touched
        .into_iter()
        .map(|lemma_id| (lemma_id, scores[lemma_id as usize]))
        .collect::<Vec<_>>();
    ranked.sort_unstable_by(|(left_id, left_score), (right_id, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_id.cmp(right_id))
    });
    ranked.truncate(limit);
    let mut resolved = ranked
        .into_iter()
        .filter_map(|(lemma_id, atom_evidence)| {
            let range = index.range(lemma_id as usize)?;
            let wave_distance = minimum_wave_distance(index, range, input)?;
            let length_gap = if input_length < range.minimum_length {
                range.minimum_length - input_length
            } else {
                input_length.saturating_sub(range.maximum_length)
            };
            let atom_evidence_milli = atom_evidence
                .saturating_mul(1_000)
                .checked_div(total_evidence)
                .unwrap_or_default()
                .min(1_000) as u16;
            Some((
                lemma_id,
                atom_evidence,
                atom_evidence_milli,
                wave_distance,
                length_gap,
            ))
        })
        .collect::<Vec<_>>();
    resolved.sort_unstable_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.4.cmp(&right.4))
            .then_with(|| left.3.cmp(&right.3))
            .then_with(|| left.0.cmp(&right.0))
    });
    resolved
        .into_iter()
        .map(
            |(lemma_id, atom_evidence, atom_evidence_milli, wave_distance, _)| LemmaWaveMatch {
                lemma_id,
                wave_distance,
                atom_evidence,
                atom_evidence_milli,
            },
        )
        .collect()
}

fn rank_candidate_lemmas(
    index: &impl LemmaWaveReadIndex,
    input: SurfaceWaveCode,
    input_length: u8,
    limit: usize,
    lemma_ids: &[u32],
) -> Vec<LemmaWaveMatch> {
    let mut best = BinaryHeap::<(u16, u8, u32)>::with_capacity(limit + 1);
    for lemma_id in lemma_ids.iter().copied() {
        let Some(range) = index.range(lemma_id as usize) else {
            continue;
        };
        let Some(wave_distance) = minimum_wave_distance(index, range, input) else {
            continue;
        };
        let length_gap = if input_length < range.minimum_length {
            range.minimum_length - input_length
        } else {
            input_length.saturating_sub(range.maximum_length)
        };
        let rank = (wave_distance, length_gap, lemma_id);
        if best.len() < limit {
            best.push(rank);
        } else if best.peek().is_some_and(|worst| rank < *worst) {
            best.pop();
            best.push(rank);
        }
    }
    let mut ranked = best.into_vec();
    ranked.sort_unstable();
    ranked
        .into_iter()
        .map(|(wave_distance, _, lemma_id)| LemmaWaveMatch {
            lemma_id,
            wave_distance,
            atom_evidence: 0,
            atom_evidence_milli: 0,
        })
        .collect()
}

fn minimum_wave_distance(
    index: &impl LemmaWaveReadIndex,
    range: LemmaWaveRange,
    input: SurfaceWaveCode,
) -> Option<u16> {
    let start = range.start as usize;
    let end = start.checked_add(range.count as usize)?;
    (start..end)
        .filter_map(|center_index| index.center(center_index))
        .map(|center| center.distance(input))
        .min()
}

fn band_candidate_lemmas(
    index: &impl LemmaWaveReadIndex,
    input: SurfaceWaveCode,
    radius: usize,
) -> Vec<u32> {
    let masks = probe_masks(radius);
    let mut candidates = Vec::new();
    for band in 0..WAVE_BANDS {
        let value = wave_band(input, band);
        for mask in masks {
            let key = band * WAVE_BAND_VALUES + usize::from(value ^ mask);
            let Some((start, end)) = index.band_offset(key).zip(index.band_offset(key + 1)) else {
                continue;
            };
            candidates.extend(
                (start as usize..end as usize)
                    .filter_map(|posting_index| index.band_posting(posting_index)),
            );
        }
    }
    candidates.sort_unstable();
    candidates.dedup();
    candidates
}

fn find_atom_key(index: &impl LemmaWaveReadIndex, target: u64) -> Option<usize> {
    let mut left = 0_usize;
    let mut right = index.atom_key_count();
    while left < right {
        let middle = left + (right - left) / 2;
        match index.atom_key(middle)?.cmp(&target) {
            std::cmp::Ordering::Less => left = middle + 1,
            std::cmp::Ordering::Greater => right = middle,
            std::cmp::Ordering::Equal => return Some(middle),
        }
    }
    None
}

fn validate_read_index(index: &impl LemmaWaveReadIndex) -> Result<Vec<u32>, String> {
    let mut expected_center = 0_usize;
    for lemma_id in 0..index.lemma_count() {
        let range = index
            .range(lemma_id)
            .ok_or_else(|| format!("L2 lemma wave range {lemma_id} is missing"))?;
        if range.count == 0 || range.minimum_length > range.maximum_length {
            return Err(format!("L2 lemma wave range {lemma_id} is invalid"));
        }
        if range.start as usize != expected_center {
            return Err(format!(
                "L2 lemma wave range {lemma_id} starts at {}, expected {expected_center}",
                range.start
            ));
        }
        expected_center = expected_center
            .checked_add(range.count as usize)
            .ok_or_else(|| "L2 lemma wave center range overflow".to_string())?;
        if expected_center > index.center_count() {
            return Err(format!(
                "L2 lemma wave range {lemma_id} exceeds center section"
            ));
        }
    }
    if expected_center != index.center_count() {
        return Err(format!(
            "L2 lemma wave ranges cover {expected_center} of {} centers",
            index.center_count()
        ));
    }

    if index.band_offset_count() != WAVE_BAND_KEYS + 1
        || index.band_offset(0) != Some(0)
        || index.band_offset(WAVE_BAND_KEYS) != Some(index.band_posting_count() as u32)
    {
        return Err("L2 lemma wave band offsets are invalid".to_string());
    }
    for key in 0..WAVE_BAND_KEYS {
        let start = index
            .band_offset(key)
            .ok_or_else(|| "L2 lemma wave band offset is missing".to_string())?
            as usize;
        let end = index
            .band_offset(key + 1)
            .ok_or_else(|| "L2 lemma wave band offset is missing".to_string())?
            as usize;
        if start > end || end > index.band_posting_count() {
            return Err("L2 lemma wave band posting range is invalid".to_string());
        }
        let mut previous = None;
        for posting_index in start..end {
            let lemma_id = index
                .band_posting(posting_index)
                .ok_or_else(|| "L2 lemma wave band posting is missing".to_string())?;
            if lemma_id as usize >= index.lemma_count()
                || previous.is_some_and(|previous| previous >= lemma_id)
            {
                return Err(format!("L2 lemma wave band bucket {key} is invalid"));
            }
            previous = Some(lemma_id);
        }
    }

    if index.atom_key_count() == 0
        && index.atom_offset_count() == 0
        && index.atom_posting_bytes() == 0
    {
        return Ok(Vec::new());
    }
    if index.atom_offset_count() != index.atom_key_count() + 1
        || index.atom_offset(0) != Some(0)
        || index.atom_offset(index.atom_key_count()) != Some(index.atom_posting_bytes() as u32)
    {
        return Err("L2 typed atom key or offset section is invalid".to_string());
    }
    let mut previous_key = None;
    let mut atom_degrees = Vec::with_capacity(index.atom_key_count());
    for atom_index in 0..index.atom_key_count() {
        let atom = index
            .atom_key(atom_index)
            .ok_or_else(|| format!("L2 typed atom key {atom_index} is missing"))?;
        let start = index
            .atom_offset(atom_index)
            .ok_or_else(|| "L2 typed atom offset is missing".to_string())?
            as usize;
        let end = index
            .atom_offset(atom_index + 1)
            .ok_or_else(|| "L2 typed atom offset is missing".to_string())?
            as usize;
        if previous_key.is_some_and(|previous| previous >= atom)
            || atom_weight(atom) == 0
            || start >= end
            || end > index.atom_posting_bytes()
        {
            return Err("L2 typed atom key or offset section is invalid".to_string());
        }
        let posting = index
            .atom_posting_slice(start, end)
            .ok_or_else(|| format!("L2 typed atom posting {atom_index} is out of range"))?;
        let mut previous_lemma = None;
        let mut degree = 0_u32;
        decode_delta_postings(posting, |lemma_id| {
            degree = degree.saturating_add(1);
            if lemma_id as usize >= index.lemma_count()
                || previous_lemma.is_some_and(|previous| previous >= lemma_id)
            {
                previous_lemma = Some(u32::MAX);
            } else {
                previous_lemma = Some(lemma_id);
            }
        })?;
        if previous_lemma.is_none() || previous_lemma == Some(u32::MAX) {
            return Err(format!("L2 typed atom posting {atom_index} is invalid"));
        }
        atom_degrees.push(degree);
        previous_key = Some(atom);
    }
    Ok(atom_degrees)
}

fn merge_lemma_matches(
    primary: Vec<LemmaWaveMatch>,
    fallback: Vec<LemmaWaveMatch>,
    limit: usize,
) -> Vec<LemmaWaveMatch> {
    let mut merged = HashMap::<u32, LemmaWaveMatch>::with_capacity(primary.len() + fallback.len());
    for candidate in primary.into_iter().chain(fallback) {
        merged
            .entry(candidate.lemma_id)
            .and_modify(|current| {
                if (
                    candidate.atom_evidence,
                    std::cmp::Reverse(candidate.wave_distance),
                ) > (
                    current.atom_evidence,
                    std::cmp::Reverse(current.wave_distance),
                ) {
                    *current = candidate;
                }
            })
            .or_insert(candidate);
    }
    let mut merged = merged.into_values().collect::<Vec<_>>();
    merged.sort_unstable_by(|left, right| {
        right
            .atom_evidence
            .cmp(&left.atom_evidence)
            .then_with(|| left.wave_distance.cmp(&right.wave_distance))
            .then_with(|| left.lemma_id.cmp(&right.lemma_id))
    });
    merged.truncate(limit);
    merged
}

struct LemmaWaveBuildRow {
    lemma_id: usize,
    minimum_length: u8,
    maximum_length: u8,
    centers: Vec<SurfaceWaveCode>,
    atoms: Vec<u64>,
}

fn build_lemma_wave_row(
    source: &impl LemmaWaveSource,
    lemma_id: usize,
    lemma: &LemmaCenter,
) -> Result<LemmaWaveBuildRow, String> {
    let start = lemma.form_start as usize;
    let end = start.saturating_add(lemma.form_count as usize);
    let mut form_refs = (start..end)
        .filter_map(|binding_index| source.binding(binding_index))
        .map(|binding| binding.form_center_ref)
        .collect::<Vec<_>>();
    form_refs.sort_unstable();
    form_refs.dedup();
    if form_refs.is_empty() {
        return Err(format!("L2 lemma {lemma_id} has no exact surface bindings"));
    }

    let mut codes = Vec::with_capacity(form_refs.len());
    let mut atoms = BTreeSet::new();
    let mut minimum_length = u8::MAX;
    let mut maximum_length = 0_u8;
    for form_ref in form_refs {
        let surface = source
            .surface(form_ref as usize)
            .ok_or_else(|| format!("L2 lemma {lemma_id} references missing form {form_ref}"))?;
        let length = surface.chars().count().min(u8::MAX as usize) as u8;
        minimum_length = minimum_length.min(length);
        maximum_length = maximum_length.max(length);
        codes.push(surface_wave_code(&surface));
        atoms.extend(surface_atom_keys(&surface));
    }
    codes.sort_unstable_by_key(|code| (code.character, code.keyboard));
    codes.dedup();
    Ok(LemmaWaveBuildRow {
        lemma_id,
        minimum_length,
        maximum_length,
        centers: select_multimodal_centers(&codes),
        atoms: atoms.into_iter().collect(),
    })
}

trait LemmaWaveSource: Sync {
    fn lemma_centers(&self) -> &[LemmaCenter];
    fn binding(&self, index: usize) -> Option<MorphBinding>;
    fn surface(&self, form_ref: usize) -> Option<Cow<'_, str>>;
}

impl LemmaWaveSource for RuntimeL2Package {
    fn lemma_centers(&self) -> &[LemmaCenter] {
        self.lemma_centers()
    }

    fn binding(&self, index: usize) -> Option<MorphBinding> {
        self.binding(index)
    }

    fn surface(&self, form_ref: usize) -> Option<Cow<'_, str>> {
        self.surface(form_ref)
    }
}

impl LemmaWaveSource for L2FieldPackage {
    fn lemma_centers(&self) -> &[LemmaCenter] {
        &self.lemma_centers
    }

    fn binding(&self, index: usize) -> Option<MorphBinding> {
        self.morph_bindings.get(index).copied()
    }

    fn surface(&self, form_ref: usize) -> Option<Cow<'_, str>> {
        let form = self.form_refs.get(form_ref)?;
        super::format::decoder_surface(&self.decoder_bytes, form.decoder_ref)
            .ok()
            .map(Cow::Borrowed)
    }
}

fn build_atom_postings(
    rows: &[LemmaWaveBuildRow],
) -> Result<(Vec<u64>, Vec<u32>, Vec<u8>), String> {
    let relation_count = rows.iter().try_fold(0_usize, |total, row| {
        total
            .checked_add(row.atoms.len())
            .ok_or_else(|| "L2 typed atom relation count overflow".to_string())
    })?;
    let mut relations = Vec::<(u64, u32)>::with_capacity(relation_count);
    for row in rows {
        let lemma_id = u32::try_from(row.lemma_id)
            .map_err(|_| "L2 typed atom lemma ID exceeds u32".to_string())?;
        relations.extend(row.atoms.iter().copied().map(|atom| (atom, lemma_id)));
    }
    relations.sort_unstable();
    relations.dedup();

    let mut keys = Vec::new();
    let mut offsets = vec![0_u32];
    let mut postings = Vec::new();
    let mut relation_index = 0_usize;
    while relation_index < relations.len() {
        let atom = relations[relation_index].0;
        keys.push(atom);
        let mut previous = 0_u32;
        let mut first = true;
        while relation_index < relations.len() && relations[relation_index].0 == atom {
            let lemma_id = relations[relation_index].1;
            let delta = if first {
                lemma_id
            } else {
                lemma_id
                    .checked_sub(previous)
                    .ok_or_else(|| "L2 typed atom postings are not ordered".to_string())?
            };
            put_var_u32(&mut postings, delta);
            previous = lemma_id;
            first = false;
            relation_index += 1;
        }
        offsets.push(
            u32::try_from(postings.len())
                .map_err(|_| "L2 typed atom posting bytes exceed u32".to_string())?,
        );
    }
    Ok((keys, offsets, postings))
}

fn validate_atom_postings(
    lemma_count: usize,
    keys: &[u64],
    offsets: &[u32],
    postings: &[u8],
) -> Result<Vec<u32>, String> {
    if keys.is_empty() && offsets.is_empty() && postings.is_empty() {
        return Ok(Vec::new());
    }
    if offsets.len() != keys.len() + 1
        || offsets.first().copied() != Some(0)
        || offsets.last().copied() != Some(postings.len() as u32)
        || offsets.windows(2).any(|pair| pair[0] >= pair[1])
        || keys.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err("L2 typed atom key or offset section is invalid".to_string());
    }
    let mut degrees = Vec::with_capacity(keys.len());
    for (index, atom) in keys.iter().copied().enumerate() {
        if atom_weight(atom) == 0 {
            return Err(format!("L2 typed atom key {index} has an invalid channel"));
        }
        let posting = postings
            .get(offsets[index] as usize..offsets[index + 1] as usize)
            .ok_or_else(|| format!("L2 typed atom posting {index} is out of range"))?;
        let mut previous = None;
        let mut degree = 0_u32;
        decode_delta_postings(posting, |lemma_id| {
            degree = degree.saturating_add(1);
            if lemma_id as usize >= lemma_count
                || previous.is_some_and(|previous| previous >= lemma_id)
            {
                previous = Some(u32::MAX);
            } else {
                previous = Some(lemma_id);
            }
        })?;
        if previous.is_none() || previous == Some(u32::MAX) {
            return Err(format!("L2 typed atom posting {index} is invalid"));
        }
        degrees.push(degree);
    }
    Ok(degrees)
}

fn decode_delta_postings(bytes: &[u8], mut emit: impl FnMut(u32)) -> Result<(), String> {
    let mut offset = 0_usize;
    let mut previous = 0_u32;
    let mut first = true;
    while offset < bytes.len() {
        let delta = read_var_u32(bytes, &mut offset)?;
        if !first && delta == 0 {
            return Err("L2 typed atom posting contains a duplicate lemma".to_string());
        }
        let lemma_id = if first {
            delta
        } else {
            previous
                .checked_add(delta)
                .ok_or_else(|| "L2 typed atom lemma delta overflows u32".to_string())?
        };
        emit(lemma_id);
        previous = lemma_id;
        first = false;
    }
    if first {
        return Err("L2 typed atom posting is empty".to_string());
    }
    Ok(())
}

fn put_var_u32(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn read_var_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let mut value = 0_u32;
    for shift in (0..35).step_by(7) {
        let byte = *bytes
            .get(*offset)
            .ok_or_else(|| "L2 typed atom varint is truncated".to_string())?;
        *offset += 1;
        if shift == 28 && byte > 0x0f {
            return Err("L2 typed atom varint overflows u32".to_string());
        }
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("L2 typed atom varint is too long".to_string())
}

fn build_band_postings(
    ranges: &[LemmaWaveRange],
    centers: &[SurfaceWaveCode],
) -> Result<(Vec<u32>, Vec<u32>), String> {
    let mut buckets = vec![Vec::<u32>::new(); WAVE_BAND_KEYS];
    for (lemma_id, range) in ranges.iter().enumerate() {
        let start = range.start as usize;
        let end = start.saturating_add(range.count as usize);
        for center in centers.get(start..end).unwrap_or_default() {
            for band in 0..WAVE_BANDS {
                let key = band * WAVE_BAND_VALUES + usize::from(wave_band(*center, band));
                buckets[key].push(lemma_id as u32);
            }
        }
    }
    let mut offsets = Vec::with_capacity(WAVE_BAND_KEYS + 1);
    let mut postings = Vec::new();
    offsets.push(0);
    for bucket in &mut buckets {
        bucket.sort_unstable();
        bucket.dedup();
        postings.extend_from_slice(bucket);
        offsets.push(
            u32::try_from(postings.len())
                .map_err(|_| "L2 lemma wave band postings exceed u32".to_string())?,
        );
    }
    Ok((offsets, postings))
}

fn validate_band_postings(
    lemma_count: usize,
    offsets: &[u32],
    postings: &[u32],
) -> Result<(), String> {
    if offsets.len() != WAVE_BAND_KEYS + 1
        || offsets.first().copied() != Some(0)
        || offsets.last().copied() != Some(postings.len() as u32)
        || offsets.windows(2).any(|pair| pair[0] > pair[1])
    {
        return Err("L2 lemma wave band offsets are invalid".to_string());
    }
    for key in 0..WAVE_BAND_KEYS {
        let start = offsets[key] as usize;
        let end = offsets[key + 1] as usize;
        let bucket = postings
            .get(start..end)
            .ok_or_else(|| "L2 lemma wave band posting range is invalid".to_string())?;
        if bucket.iter().any(|lemma| *lemma as usize >= lemma_count)
            || bucket.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(format!("L2 lemma wave band bucket {key} is invalid"));
        }
    }
    Ok(())
}

fn wave_band(code: SurfaceWaveCode, band: usize) -> u16 {
    let (bits, local_band) = if band < 4 {
        (code.character, band)
    } else {
        (code.keyboard, band - 4)
    };
    ((bits >> (local_band * WAVE_BAND_BITS)) & 0xffff) as u16
}

fn probe_masks(radius: usize) -> &'static [u16] {
    static RADIUS_TWO: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    static RADIUS_THREE: std::sync::OnceLock<Vec<u16>> = std::sync::OnceLock::new();
    match radius {
        2 => RADIUS_TWO.get_or_init(|| hamming_masks(2)),
        3 => RADIUS_THREE.get_or_init(|| hamming_masks(3)),
        _ => &[],
    }
}

fn hamming_masks(radius: usize) -> Vec<u16> {
    let mut masks = vec![0_u16];
    for first in 0..WAVE_BAND_BITS {
        masks.push(1_u16 << first);
    }
    if radius >= 2 {
        for first in 0..WAVE_BAND_BITS {
            for second in first + 1..WAVE_BAND_BITS {
                masks.push((1_u16 << first) | (1_u16 << second));
            }
        }
    }
    if radius >= 3 {
        for first in 0..WAVE_BAND_BITS {
            for second in first + 1..WAVE_BAND_BITS {
                for third in second + 1..WAVE_BAND_BITS {
                    masks.push((1_u16 << first) | (1_u16 << second) | (1_u16 << third));
                }
            }
        }
    }
    masks
}

pub(super) fn normalized_similarity_milli(observed: &str, expected: &str) -> u16 {
    let observed = surface_scoring_profile(observed);
    let expected = surface_scoring_profile(expected);
    prepared_normalized_similarity_milli(&observed, &expected)
}

pub(super) fn surface_scoring_profile(surface: &str) -> SurfaceScoringProfile {
    let normalized = normalize_surface(surface);
    let characters = normalized.chars().map(|ch| ch as u32).collect::<Vec<_>>();
    let keyboard = physical_keys(&normalized);
    SurfaceScoringProfile {
        normalized,
        characters,
        keyboard,
    }
}

pub(super) fn prepared_normalized_similarity_milli(
    observed: &SurfaceScoringProfile,
    expected: &SurfaceScoringProfile,
) -> u16 {
    let character = normalized_distance_similarity(&observed.characters, &expected.characters);
    let keyboard = normalized_distance_similarity(&observed.keyboard, &expected.keyboard);
    character.max(keyboard)
}

pub(super) fn prepared_normalized_similarity_at_least_milli(
    observed: &SurfaceScoringProfile,
    expected: &SurfaceScoringProfile,
    minimum_milli: u16,
) -> u16 {
    let character = normalized_distance_similarity_at_least(
        &observed.characters,
        &expected.characters,
        minimum_milli,
    );
    let keyboard = normalized_distance_similarity_at_least(
        &observed.keyboard,
        &expected.keyboard,
        minimum_milli,
    );
    character.max(keyboard)
}

pub(super) fn surface_wave_code(surface: &str) -> SurfaceWaveCode {
    let normalized = normalize_surface(surface);
    SurfaceWaveCode {
        character: simhash(
            &normalized.chars().map(|ch| ch as u32).collect::<Vec<_>>(),
            1,
        ),
        keyboard: simhash(&physical_keys(&normalized), 2),
    }
}

pub(super) fn surface_atom_profile(surface: &str) -> SurfaceAtomProfile {
    surface_atom_profile_normalized(&normalize_surface(surface))
}

fn surface_atom_profile_normalized(surface: &str) -> SurfaceAtomProfile {
    let atoms = surface_atom_keys_normalized(surface);
    let total_weight = atoms.iter().copied().map(atom_weight).map(u32::from).sum();
    SurfaceAtomProfile {
        atoms,
        total_weight,
    }
}

pub(super) fn surface_atom_similarity_milli(observed: &SurfaceAtomProfile, expected: &str) -> u16 {
    let expected = surface_atom_profile(expected);
    atom_profile_similarity_milli(observed, &expected)
}

pub(super) fn prepared_surface_atom_similarity_milli(
    observed: &SurfaceAtomProfile,
    expected: &SurfaceAtomProfile,
) -> u16 {
    atom_profile_similarity_milli(observed, expected)
}

pub(super) fn prepared_surface_atom_profile(profile: &SurfaceScoringProfile) -> SurfaceAtomProfile {
    surface_atom_profile_normalized(&profile.normalized)
}

fn atom_profile_similarity_milli(
    observed: &SurfaceAtomProfile,
    expected: &SurfaceAtomProfile,
) -> u16 {
    let denominator = observed.total_weight.saturating_add(expected.total_weight);
    if denominator == 0 {
        return u16::from(observed.atoms.is_empty() && expected.atoms.is_empty()) * 1_000;
    }
    let mut observed_index = 0_usize;
    let mut expected_index = 0_usize;
    let mut shared_weight = 0_u32;
    while observed_index < observed.atoms.len() && expected_index < expected.atoms.len() {
        match observed.atoms[observed_index].cmp(&expected.atoms[expected_index]) {
            std::cmp::Ordering::Less => observed_index += 1,
            std::cmp::Ordering::Greater => expected_index += 1,
            std::cmp::Ordering::Equal => {
                shared_weight = shared_weight
                    .saturating_add(u32::from(atom_weight(observed.atoms[observed_index])));
                observed_index += 1;
                expected_index += 1;
            }
        }
    }
    shared_weight
        .saturating_mul(2_000)
        .checked_div(denominator)
        .unwrap_or_default()
        .min(1_000) as u16
}

fn surface_atom_keys(surface: &str) -> Vec<u64> {
    let normalized = normalize_surface(surface);
    surface_atom_keys_normalized(&normalized)
}

fn surface_atom_keys_normalized(surface: &str) -> Vec<u64> {
    let characters = surface.chars().map(|ch| ch as u32).collect::<Vec<_>>();
    let keyboard = physical_keys(surface);
    let mut atoms = Vec::with_capacity(characters.len().saturating_mul(16).saturating_add(32));
    append_atom_family(
        &characters,
        0x11_0001,
        0x11_0002,
        ATOM_CHARACTER_BIGRAM,
        ATOM_CHARACTER_TRIGRAM,
        ATOM_CHARACTER_BAG_TRIGRAM,
        ATOM_CHARACTER_SKIP_GRAM,
        &mut atoms,
    );
    append_atom_family(
        &keyboard,
        0x20_0001,
        0x20_0002,
        ATOM_KEYBOARD_BIGRAM,
        ATOM_KEYBOARD_TRIGRAM,
        ATOM_KEYBOARD_BAG_TRIGRAM,
        ATOM_KEYBOARD_SKIP_GRAM,
        &mut atoms,
    );
    atoms.sort_unstable();
    atoms.dedup();
    atoms
}

#[allow(clippy::too_many_arguments)]
fn append_atom_family(
    units: &[u32],
    start_marker: u32,
    end_marker: u32,
    bigram_channel: u8,
    trigram_channel: u8,
    bag_channel: u8,
    skip_channel: u8,
    output: &mut Vec<u64>,
) {
    if units.is_empty() {
        return;
    }
    let mut padded = Vec::with_capacity(units.len() + 4);
    padded.extend([start_marker, start_marker]);
    padded.extend_from_slice(units);
    padded.extend([end_marker, end_marker]);
    for window in padded.windows(2) {
        output.push(typed_atom_key(
            bigram_channel,
            u64::from(bigram_channel),
            window,
        ));
    }
    for window in padded.windows(3) {
        output.push(typed_atom_key(
            trigram_channel,
            u64::from(trigram_channel),
            window,
        ));
        let mut bag = [window[0], window[1], window[2]];
        bag.sort_unstable();
        output.push(typed_atom_key(bag_channel, u64::from(bag_channel), &bag));
    }
    for distance in 2..=4 {
        for position in 0..padded.len().saturating_sub(distance) {
            output.push(typed_atom_key(
                skip_channel,
                u64::from(skip_channel) * 16 + distance as u64,
                &[padded[position], padded[position + distance]],
            ));
        }
    }
}

fn typed_atom_key(channel: u8, domain: u64, units: &[u32]) -> u64 {
    (u64::from(channel) << 56) | (hash_atom(domain, units) & ATOM_HASH_MASK)
}

fn atom_weight(atom: u64) -> u8 {
    match (atom >> 56) as u8 {
        ATOM_CHARACTER_BIGRAM | ATOM_KEYBOARD_BIGRAM => 1,
        ATOM_CHARACTER_TRIGRAM
        | ATOM_KEYBOARD_TRIGRAM
        | ATOM_CHARACTER_BAG_TRIGRAM
        | ATOM_KEYBOARD_BAG_TRIGRAM => 3,
        ATOM_CHARACTER_SKIP_GRAM | ATOM_KEYBOARD_SKIP_GRAM => 2,
        _ => 0,
    }
}

fn select_multimodal_centers(codes: &[SurfaceWaveCode]) -> Vec<SurfaceWaveCode> {
    if codes.is_empty() {
        return Vec::new();
    }
    let budget = logarithmic_center_budget(codes.len());
    let consensus = consensus_code(codes);
    let first = codes
        .iter()
        .enumerate()
        .min_by_key(|(index, code)| (code.distance(consensus), *index))
        .map(|(index, _)| index)
        .unwrap_or_default();
    let mut selected = vec![first];
    while selected.len() < budget {
        let Some((distance, index)) = codes
            .iter()
            .enumerate()
            .filter(|(index, _)| !selected.contains(index))
            .map(|(index, code)| {
                let distance = selected
                    .iter()
                    .map(|selected| code.distance(codes[*selected]))
                    .min()
                    .unwrap_or_default();
                (distance, index)
            })
            .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1)))
        else {
            break;
        };
        if distance == 0 {
            break;
        }
        selected.push(index);
    }
    selected.sort_unstable();
    selected.into_iter().map(|index| codes[index]).collect()
}

fn logarithmic_center_budget(variant_count: usize) -> usize {
    if variant_count <= 1 {
        return variant_count;
    }
    (usize::BITS - variant_count.leading_zeros()).min(MAX_LEMMA_WAVE_CENTERS as u32) as usize
}

fn consensus_code(codes: &[SurfaceWaveCode]) -> SurfaceWaveCode {
    SurfaceWaveCode {
        character: majority_bits(codes.iter().map(|code| code.character), codes.len()),
        keyboard: majority_bits(codes.iter().map(|code| code.keyboard), codes.len()),
    }
}

fn majority_bits(values: impl Iterator<Item = u64>, count: usize) -> u64 {
    let mut support = [0_u32; 64];
    for value in values {
        for (bit, support) in support.iter_mut().enumerate() {
            *support += u32::from(value & (1_u64 << bit) != 0);
        }
    }
    support
        .into_iter()
        .enumerate()
        .fold(0_u64, |bits, (bit, support)| {
            bits | (u64::from(support as usize * 2 >= count) << bit)
        })
}

fn simhash(units: &[u32], domain: u64) -> u64 {
    if units.is_empty() {
        return 0;
    }
    let mut padded = Vec::with_capacity(units.len() + 2);
    padded.push(0x11_0000 + domain as u32);
    padded.extend_from_slice(units);
    padded.push(0x12_0000 + domain as u32);
    let mut atoms = BTreeSet::<u64>::new();
    for gram in [2_usize, 3] {
        for window in padded.windows(gram) {
            atoms.insert(hash_atom(domain * 16 + gram as u64, window));
        }
    }
    for distance in 2..=4 {
        for position in 0..padded.len().saturating_sub(distance) {
            atoms.insert(hash_atom(
                domain * 16 + 8 + distance as u64,
                &[padded[position], padded[position + distance]],
            ));
        }
    }
    for window in padded.windows(3) {
        let mut bag = [window[0], window[1], window[2]];
        bag.sort_unstable();
        atoms.insert(hash_atom(domain * 16 + 15, &bag));
    }
    let mut support = [0_i16; 64];
    for atom in atoms {
        for (bit, support) in support.iter_mut().enumerate() {
            *support += if atom & (1_u64 << bit) == 0 { -1 } else { 1 };
        }
    }
    support
        .into_iter()
        .enumerate()
        .fold(0_u64, |bits, (bit, support)| {
            bits | (u64::from(support >= 0) << bit)
        })
}

fn hash_atom(channel: u64, units: &[u32]) -> u64 {
    units
        .iter()
        .fold(crate::stable_hash::mix64_golden(channel), |state, unit| {
            crate::stable_hash::mix64_golden(state ^ u64::from(*unit).rotate_left(17))
        })
}

fn physical_keys(surface: &str) -> Vec<u32> {
    crate::keyboard::text_to_key_events(surface, false)
        .unwrap_or_default()
        .into_iter()
        .map(|event| u32::from(event.keycode) | (u32::from(event.shift) << 16))
        .collect()
}

pub(super) fn normalize_surface(surface: &str) -> String {
    surface
        .trim()
        .trim_matches(|ch: char| matches!(ch, '!' | ',' | '.' | '?' | ';' | ':'))
        .to_lowercase()
}

fn normalized_distance_similarity<T: Eq>(left: &[T], right: &[T]) -> u16 {
    let denominator = left.len().max(right.len());
    if denominator == 0 {
        return 1_000;
    }
    let distance = damerau_levenshtein(left, right).min(denominator);
    ((denominator - distance) * 1_000 / denominator) as u16
}

fn normalized_distance_similarity_at_least<T: Eq>(
    left: &[T],
    right: &[T],
    minimum_milli: u16,
) -> u16 {
    let denominator = left.len().max(right.len());
    if denominator == 0 {
        return 1_000;
    }
    let required_matches = usize::from(minimum_milli)
        .saturating_mul(denominator)
        .div_ceil(1_000);
    let maximum_distance = denominator.saturating_sub(required_matches);
    let Some(distance) = damerau_levenshtein_bounded(left, right, maximum_distance) else {
        return 0;
    };
    ((denominator - distance) * 1_000 / denominator) as u16
}

fn damerau_levenshtein<T: Eq>(left: &[T], right: &[T]) -> usize {
    const STACK_ROW_UNITS: usize = 64;
    if right.len() <= STACK_ROW_UNITS {
        let mut previous_previous = [0_usize; STACK_ROW_UNITS + 1];
        let mut previous = [0_usize; STACK_ROW_UNITS + 1];
        let mut current = [0_usize; STACK_ROW_UNITS + 1];
        return damerau_levenshtein_rows(
            left,
            right,
            &mut previous_previous[..=right.len()],
            &mut previous[..=right.len()],
            &mut current[..=right.len()],
        );
    }
    let mut previous_previous = vec![0_usize; right.len() + 1];
    let mut previous = vec![0_usize; right.len() + 1];
    let mut current = vec![0_usize; right.len() + 1];
    damerau_levenshtein_rows(
        left,
        right,
        &mut previous_previous,
        &mut previous,
        &mut current,
    )
}

fn damerau_levenshtein_bounded<T: Eq>(
    left: &[T],
    right: &[T],
    maximum_distance: usize,
) -> Option<usize> {
    const STACK_ROW_UNITS: usize = 64;
    if left.len().abs_diff(right.len()) > maximum_distance {
        return None;
    }
    if maximum_distance >= left.len().max(right.len()) {
        return Some(damerau_levenshtein(left, right));
    }
    if right.len() <= STACK_ROW_UNITS {
        let mut previous_previous = [0_usize; STACK_ROW_UNITS + 1];
        let mut previous = [0_usize; STACK_ROW_UNITS + 1];
        let mut current = [0_usize; STACK_ROW_UNITS + 1];
        return damerau_levenshtein_bounded_rows(
            left,
            right,
            maximum_distance,
            &mut previous_previous[..=right.len()],
            &mut previous[..=right.len()],
            &mut current[..=right.len()],
        );
    }
    let mut previous_previous = vec![0_usize; right.len() + 1];
    let mut previous = vec![0_usize; right.len() + 1];
    let mut current = vec![0_usize; right.len() + 1];
    damerau_levenshtein_bounded_rows(
        left,
        right,
        maximum_distance,
        &mut previous_previous,
        &mut previous,
        &mut current,
    )
}

fn damerau_levenshtein_bounded_rows<'a, T: Eq>(
    left: &[T],
    right: &[T],
    maximum_distance: usize,
    mut previous_previous: &'a mut [usize],
    mut previous: &'a mut [usize],
    mut current: &'a mut [usize],
) -> Option<usize> {
    let outside = maximum_distance.saturating_add(1);
    previous_previous.fill(outside);
    previous.fill(outside);
    current.fill(outside);
    for (index, value) in previous
        .iter_mut()
        .enumerate()
        .take(maximum_distance.saturating_add(1))
    {
        *value = index;
    }
    for left_index in 1..=left.len() {
        current.fill(outside);
        if left_index <= maximum_distance {
            current[0] = left_index;
        }
        let first_right = left_index.saturating_sub(maximum_distance).max(1);
        let last_right = left_index.saturating_add(maximum_distance).min(right.len());
        for right_index in first_right..=last_right {
            let substitution = previous[right_index - 1]
                .saturating_add(usize::from(left[left_index - 1] != right[right_index - 1]));
            let insertion = current[right_index - 1].saturating_add(1);
            let deletion = previous[right_index].saturating_add(1);
            let mut value = substitution.min(insertion).min(deletion);
            if left_index > 1
                && right_index > 1
                && left[left_index - 1] == right[right_index - 2]
                && left[left_index - 2] == right[right_index - 1]
            {
                value = value.min(previous_previous[right_index - 2].saturating_add(1));
            }
            current[right_index] = value.min(outside);
        }
        std::mem::swap(&mut previous_previous, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    (previous[right.len()] <= maximum_distance).then_some(previous[right.len()])
}

fn damerau_levenshtein_rows<'a, T: Eq>(
    left: &[T],
    right: &[T],
    mut previous_previous: &'a mut [usize],
    mut previous: &'a mut [usize],
    mut current: &'a mut [usize],
) -> usize {
    for (index, value) in previous.iter_mut().enumerate() {
        *value = index;
    }
    for (left_index, left_value) in left.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_value) in right.iter().enumerate() {
            let substitution = previous[right_index] + usize::from(left_value != right_value);
            let insertion = current[right_index] + 1;
            let deletion = previous[right_index + 1] + 1;
            let mut value = substitution.min(insertion).min(deletion);
            if left_index > 0
                && right_index > 0
                && left_value == &right[right_index - 1]
                && &left[left_index - 1] == right_value
            {
                value = value.min(previous_previous[right_index - 1] + 1);
            }
            current[right_index + 1] = value;
        }
        std::mem::swap(&mut previous_previous, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l2_field::compiler::compile_l2_package;
    use crate::nanda_wave::l2_field::runtime_storage::RuntimeL2Package;
    use crate::nanda_wave::l2_field::teacher::L2TeacherCorpus;
    use std::collections::BTreeMap;

    fn synthetic_index(input: SurfaceWaveCode, distances: &[usize]) -> LemmaWaveIndex {
        let centers = distances
            .iter()
            .enumerate()
            .map(|(lemma_id, distance)| {
                let mut code = input;
                for offset in 0..*distance {
                    let bit = (offset * 29 + lemma_id * 17) % 128;
                    if bit < 64 {
                        code.character ^= 1_u64 << bit;
                    } else {
                        code.keyboard ^= 1_u64 << (bit - 64);
                    }
                }
                code
            })
            .collect::<Vec<_>>();
        let ranges = centers
            .iter()
            .enumerate()
            .map(|(index, _)| LemmaWaveRange {
                start: index as u32,
                count: 1,
                minimum_length: 2,
                maximum_length: u8::MAX,
            })
            .collect::<Vec<_>>();
        let (offsets, postings) = build_band_postings(&ranges, &centers).expect("postings");
        LemmaWaveIndex::from_parts(
            ranges,
            centers,
            offsets,
            postings,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect("index")
    }

    #[test]
    fn keyboard_wave_is_layout_invariant() {
        assert_eq!(
            surface_wave_code("привет").keyboard,
            surface_wave_code("ghbdtn").keyboard
        );
    }

    #[test]
    fn lemma_wave_index_keeps_multimodal_exact_variants() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tидти\tидти\tverb:inf\n\
             F\tидти\tиду\tverb:sg:p1:pres\n\
             F\tидти\tшел\tverb:sg:past:masc\n\
             F\tидти\tшёл\tverb:sg:past:masc\n\
             T\tидти\tиду\tverb:sg:p1:pres\tя _\n\
             H\tидти\tшёл\tverb:sg:past:masc\tон _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("идти", 17)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let package = RuntimeL2Package::from_reference(package);
        let index = LemmaWaveIndex::build(&package).expect("index");

        assert_eq!(index.ranges.len(), 1);
        assert!(index.center_count() > 1);
        let ranked = index.rank_lemmas("шол", 1);
        assert_eq!(ranked[0].lemma_id, 0);
        assert!(ranked[0].atom_evidence > 0);
        assert!(!index.atom_keys().is_empty());
        assert_eq!(index.atom_offsets().len(), index.atom_keys().len() + 1);
    }

    #[test]
    fn normalized_similarity_uses_character_or_physical_key_geometry() {
        assert_eq!(normalized_similarity_milli("привет", "привет"), 1_000);
        assert_eq!(normalized_similarity_milli("ghbdtn", "привет"), 1_000);
        assert!(normalized_similarity_milli("превет", "привет") >= 800);
    }

    #[test]
    fn prepared_surface_profiles_preserve_geometry_and_atom_scores() {
        for (observed, expected) in [
            ("ПРИВЕТ!", "привет"),
            ("ghbdtn", "привет"),
            ("превет", "привет"),
            ("титрирориваны", "титрированы"),
        ] {
            let normalized_observed = normalize_surface(observed);
            let normalized_expected = normalize_surface(expected);
            let legacy_character = normalized_distance_similarity(
                &normalized_observed
                    .chars()
                    .map(|ch| ch as u32)
                    .collect::<Vec<_>>(),
                &normalized_expected
                    .chars()
                    .map(|ch| ch as u32)
                    .collect::<Vec<_>>(),
            );
            let legacy_keyboard = normalized_distance_similarity(
                &physical_keys(&normalized_observed),
                &physical_keys(&normalized_expected),
            );
            let observed_profile = surface_scoring_profile(observed);
            let expected_profile = surface_scoring_profile(expected);
            assert_eq!(
                prepared_normalized_similarity_milli(&observed_profile, &expected_profile),
                legacy_character.max(legacy_keyboard)
            );
            assert_eq!(
                prepared_surface_atom_similarity_milli(
                    &prepared_surface_atom_profile(&observed_profile),
                    &prepared_surface_atom_profile(&expected_profile),
                ),
                surface_atom_similarity_milli(&surface_atom_profile(observed), expected)
            );
        }
    }

    #[test]
    fn stack_damerau_rows_match_heap_reference() {
        fn heap_reference<T: Eq>(left: &[T], right: &[T]) -> usize {
            let mut previous_previous = vec![0_usize; right.len() + 1];
            let mut previous = (0..=right.len()).collect::<Vec<_>>();
            let mut current = vec![0_usize; right.len() + 1];
            for (left_index, left_value) in left.iter().enumerate() {
                current[0] = left_index + 1;
                for (right_index, right_value) in right.iter().enumerate() {
                    let substitution =
                        previous[right_index] + usize::from(left_value != right_value);
                    let insertion = current[right_index] + 1;
                    let deletion = previous[right_index + 1] + 1;
                    let mut value = substitution.min(insertion).min(deletion);
                    if left_index > 0
                        && right_index > 0
                        && left_value == &right[right_index - 1]
                        && &left[left_index - 1] == right_value
                    {
                        value = value.min(previous_previous[right_index - 1] + 1);
                    }
                    current[right_index + 1] = value;
                }
                std::mem::swap(&mut previous_previous, &mut previous);
                std::mem::swap(&mut previous, &mut current);
            }
            previous[right.len()]
        }

        for (left, right) in [
            ("", ""),
            ("", "дом"),
            ("привет", "привет"),
            ("превет", "привет"),
            ("првиет", "привет"),
            ("титрирориваны", "титрированы"),
            ("nbnhbhjdfys", "титрированы"),
        ] {
            let left = left.chars().collect::<Vec<_>>();
            let right = right.chars().collect::<Vec<_>>();
            let exact = heap_reference(&left, &right);
            assert_eq!(damerau_levenshtein(&left, &right), exact);
            for limit in 0..=left.len().max(right.len()) {
                assert_eq!(
                    damerau_levenshtein_bounded(&left, &right, limit),
                    (exact <= limit).then_some(exact),
                    "bounded mismatch for {left:?} -> {right:?} at {limit}"
                );
            }
        }
    }

    #[test]
    fn keyboard_typed_atoms_are_layout_invariant() {
        let keyboard_atoms = |surface| {
            surface_atom_keys(surface)
                .into_iter()
                .filter(|atom| {
                    matches!(
                        (atom >> 56) as u8,
                        ATOM_KEYBOARD_BIGRAM
                            | ATOM_KEYBOARD_TRIGRAM
                            | ATOM_KEYBOARD_BAG_TRIGRAM
                            | ATOM_KEYBOARD_SKIP_GRAM
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(keyboard_atoms("привет"), keyboard_atoms("ghbdtn"));
    }

    #[test]
    fn delta_postings_roundtrip_strictly_ordered_lemma_ids() {
        let mut encoded = Vec::new();
        for delta in [0, 3, 124, 65_408] {
            put_var_u32(&mut encoded, delta);
        }
        let mut decoded = Vec::new();
        decode_delta_postings(&encoded, |lemma_id| decoded.push(lemma_id)).expect("decode");
        assert_eq!(decoded, [0, 3, 127, 65_535]);

        assert!(decode_delta_postings(&[0, 0], |_| {}).is_err());
        assert!(decode_delta_postings(&[0x80], |_| {}).is_err());
    }

    #[test]
    fn band_probes_cover_every_center_inside_their_exact_hamming_radius() {
        let input = surface_wave_code("системныйконтур");
        let distances = (0..=40).collect::<Vec<_>>();
        let index = synthetic_index(input, &distances);

        let radius_two = index.band_candidate_lemmas(input, 2);
        for lemma_id in 0_u32..=23 {
            assert!(
                radius_two.contains(&lemma_id),
                "missing distance {lemma_id}"
            );
        }
        let radius_three = index.band_candidate_lemmas(input, 3);
        for lemma_id in 0_u32..=31 {
            assert!(
                radius_three.contains(&lemma_id),
                "missing distance {lemma_id}"
            );
        }
    }

    #[test]
    fn banded_ranking_is_identical_to_exhaustive_ranking() {
        let probes = [
            "восстановление",
            "востановление",
            "восстанолвение",
            "ghbdtn",
            "перспективнее",
        ];
        let mut centers = probes
            .iter()
            .flat_map(|probe| {
                let input = surface_wave_code(probe);
                (0..12).map(move |index| SurfaceWaveCode {
                    character: input.character ^ crate::stable_hash::mix64_golden(index + 1),
                    keyboard: input.keyboard ^ crate::stable_hash::mix64_golden(index + 101),
                })
            })
            .collect::<Vec<_>>();
        centers.extend(probes.iter().map(|probe| surface_wave_code(probe)));
        let ranges = centers
            .iter()
            .enumerate()
            .map(|(index, _)| LemmaWaveRange {
                start: index as u32,
                count: 1,
                minimum_length: 2,
                maximum_length: u8::MAX,
            })
            .collect::<Vec<_>>();
        let (offsets, postings) = build_band_postings(&ranges, &centers).expect("postings");
        let index = LemmaWaveIndex::from_parts(
            ranges,
            centers,
            offsets,
            postings,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect("index");

        for probe in probes {
            for limit in [1, 4, 8, 16, 32] {
                assert_eq!(
                    index.rank_lemmas(probe, limit),
                    index.rank_lemmas_exhaustive(probe, limit),
                    "probe={probe:?} limit={limit}"
                );
            }
        }
    }
}
