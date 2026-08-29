use super::compositional::{LemmaWaveIndex, LemmaWaveRange, SurfaceWaveCode};
use super::format::{self, Cursor};

pub(super) const LEMMA_WAVE_RANGE_BYTES: usize = 8;
pub(super) const SURFACE_WAVE_CODE_BYTES: usize = 16;
pub(super) const WAVE_BAND_OFFSET_BYTES: usize = 4;
pub(super) const WAVE_BAND_POSTING_BYTES: usize = 4;
pub(super) const ATOM_KEY_BYTES: usize = 8;
pub(super) const ATOM_OFFSET_BYTES: usize = 4;
pub(super) const ATOM_POSTING_BYTES: usize = 1;

pub(super) fn encode(index: &LemmaWaveIndex, out: &mut Vec<u8>) {
    for range in index.ranges() {
        format::put_u32(out, range.start);
        format::put_u16(out, range.count);
        out.push(range.minimum_length);
        out.push(range.maximum_length);
    }
    for center in index.centers() {
        format::put_u64(out, center.character);
        format::put_u64(out, center.keyboard);
    }
    for offset in index.band_offsets() {
        format::put_u32(out, *offset);
    }
    for lemma_id in index.band_postings() {
        format::put_u32(out, *lemma_id);
    }
    for atom in index.atom_keys() {
        format::put_u64(out, *atom);
    }
    for offset in index.atom_offsets() {
        format::put_u32(out, *offset);
    }
    out.extend_from_slice(index.atom_postings());
}

#[expect(
    clippy::too_many_arguments,
    reason = "sealed format sections remain explicit"
)]
pub(super) fn decode(
    range_bytes: &[u8],
    range_count: usize,
    center_bytes: &[u8],
    center_count: usize,
    band_offset_bytes: &[u8],
    band_offset_count: usize,
    band_posting_bytes: &[u8],
    band_posting_count: usize,
    atom_key_bytes: &[u8],
    atom_key_count: usize,
    atom_offset_bytes: &[u8],
    atom_offset_count: usize,
    atom_posting_bytes: &[u8],
    atom_posting_count: usize,
) -> Result<LemmaWaveIndex, String> {
    if range_bytes.len() != range_count.saturating_mul(LEMMA_WAVE_RANGE_BYTES)
        || center_bytes.len() != center_count.saturating_mul(SURFACE_WAVE_CODE_BYTES)
        || band_offset_bytes.len() != band_offset_count.saturating_mul(WAVE_BAND_OFFSET_BYTES)
        || band_posting_bytes.len() != band_posting_count.saturating_mul(WAVE_BAND_POSTING_BYTES)
        || atom_key_bytes.len() != atom_key_count.saturating_mul(ATOM_KEY_BYTES)
        || atom_offset_bytes.len() != atom_offset_count.saturating_mul(ATOM_OFFSET_BYTES)
        || atom_posting_bytes.len() != atom_posting_count.saturating_mul(ATOM_POSTING_BYTES)
    {
        return Err("compact L2 lemma wave section width mismatch".to_string());
    }
    let mut range_cursor = Cursor::new(range_bytes);
    let mut ranges = Vec::with_capacity(range_count);
    for _ in 0..range_count {
        ranges.push(LemmaWaveRange {
            start: range_cursor.u32()?,
            count: range_cursor.u16()?,
            minimum_length: range_cursor.u8()?,
            maximum_length: range_cursor.u8()?,
        });
    }
    let mut center_cursor = Cursor::new(center_bytes);
    let mut centers = Vec::with_capacity(center_count);
    for _ in 0..center_count {
        centers.push(SurfaceWaveCode {
            character: center_cursor.u64()?,
            keyboard: center_cursor.u64()?,
        });
    }
    let mut offset_cursor = Cursor::new(band_offset_bytes);
    let mut band_offsets = Vec::with_capacity(band_offset_count);
    for _ in 0..band_offset_count {
        band_offsets.push(offset_cursor.u32()?);
    }
    let mut posting_cursor = Cursor::new(band_posting_bytes);
    let mut band_postings = Vec::with_capacity(band_posting_count);
    for _ in 0..band_posting_count {
        band_postings.push(posting_cursor.u32()?);
    }
    let mut atom_key_cursor = Cursor::new(atom_key_bytes);
    let mut atom_keys = Vec::with_capacity(atom_key_count);
    for _ in 0..atom_key_count {
        atom_keys.push(atom_key_cursor.u64()?);
    }
    let mut atom_offset_cursor = Cursor::new(atom_offset_bytes);
    let mut atom_offsets = Vec::with_capacity(atom_offset_count);
    for _ in 0..atom_offset_count {
        atom_offsets.push(atom_offset_cursor.u32()?);
    }
    LemmaWaveIndex::from_parts(
        ranges,
        centers,
        band_offsets,
        band_postings,
        atom_keys,
        atom_offsets,
        atom_posting_bytes.to_vec(),
    )
}
