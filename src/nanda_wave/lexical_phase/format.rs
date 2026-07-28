use crate::lexical_surface_atoms::{
    surface_atom_projection, visit_surface_atoms, EncodedSurfaceField,
};
use crate::stable_hash::mix64_golden;

pub(super) const MAGIC: &[u8; 8] = b"LAYLPH02";
pub(super) const VERSION: u32 = 2;
pub(super) const HEADER_BYTES: usize = 128;
pub(super) const NODE_BYTES: usize = 24;
pub(super) const ARC_BYTES: usize = 8;
pub(super) const PHASE_CELLS: usize = 32;
pub(super) const TERMINAL_BYTES: usize = 16 + PHASE_CELLS;
pub(super) const CENTER_BYTES: usize = 16;
pub(super) const POSTING_BYTES: usize = 4;
pub(super) const DECODER_STATE_BYTES: usize = 8;
pub(super) const DECODER_ARC_BYTES: usize = 8;
pub(super) const NO_INDEX: u32 = u32::MAX;
pub(super) const MAX_WORD_CHARS: usize = 24;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ArtifactHeader {
    pub(super) file_bytes: u64,
    pub(super) node_count: u32,
    pub(super) arc_count: u32,
    pub(super) terminal_count: u32,
    pub(super) center_count: u32,
    pub(super) posting_count: u32,
    pub(super) nodes_offset: u64,
    pub(super) arcs_offset: u64,
    pub(super) terminals_offset: u64,
    pub(super) centers_offset: u64,
    pub(super) postings_offset: u64,
    pub(super) checksum: u64,
    pub(super) corpus_hash: u64,
    pub(super) source_words: u32,
    pub(super) decoder_state_count: u32,
    pub(super) decoder_arc_count: u32,
    pub(super) decoder_states_offset: u64,
    pub(super) training_surfaces: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct NodeRecord {
    pub(super) parent: u32,
    pub(super) incoming: u32,
    pub(super) first_arc: u32,
    pub(super) arc_len: u16,
    pub(super) depth: u16,
    pub(super) terminal: u32,
    pub(super) best_terminal: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ArcRecord {
    pub(super) ch: u32,
    pub(super) child: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TerminalRecord {
    pub(super) node: u32,
    pub(super) rank: u32,
    pub(super) support: u32,
    pub(super) char_len: u16,
    pub(super) atom_count: u16,
    pub(super) phase: [i8; PHASE_CELLS],
}

impl Default for TerminalRecord {
    fn default() -> Self {
        Self {
            node: 0,
            rank: 0,
            support: 0,
            char_len: 0,
            atom_count: 0,
            phase: [0; PHASE_CELLS],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CenterRecord {
    pub(super) key: u64,
    pub(super) posting_start: u32,
    pub(super) posting_len: u16,
    pub(super) support: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DecoderStateRecord {
    pub(super) first_arc: u32,
    pub(super) arc_len: u16,
    pub(super) flags: u16,
}

impl DecoderStateRecord {
    pub(super) fn is_final(self) -> bool {
        self.flags & 1 != 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DecoderArcRecord {
    pub(super) ch: u32,
    pub(super) child: u32,
}

pub(super) fn normalize_surface(text: &str) -> Option<String> {
    let normalized = text.trim().to_lowercase();
    let len = normalized.chars().count();
    if !(1..=MAX_WORD_CHARS).contains(&len)
        || !normalized
            .chars()
            .all(|ch| ch.is_alphabetic() || ch == '-' || ch == '\'')
    {
        return None;
    }
    Some(normalized)
}

pub(super) fn atom_center_keys(field: &EncodedSurfaceField) -> Vec<u64> {
    let mut keys = Vec::new();
    for atom in field.atoms() {
        keys.push(atom_key(0x4c31_504f_5349_5449, atom.position, &atom.bytes));
        keys.push(atom_key(0x4c31_5245_4c41_5845, 0, &atom.bytes));
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

pub(super) fn surface_phase(field: &EncodedSurfaceField) -> ([i8; PHASE_CELLS], u16) {
    let mut sums = [0i16; PHASE_CELLS];
    for atom in field.atoms() {
        for trit in surface_atom_projection(atom.position, &atom.bytes) {
            let cell = usize::from(trit.lane) % PHASE_CELLS;
            sums[cell] = sums[cell].saturating_add(i16::from(trit.value));
        }
    }
    let mut phase = [0i8; PHASE_CELLS];
    for (target, value) in phase.iter_mut().zip(sums) {
        *target = value.clamp(i8::MIN as i16, i8::MAX as i16) as i8;
    }
    (phase, field.atoms().len().min(u16::MAX as usize) as u16)
}

pub(super) fn surface_phase_and_atom_center_keys(text: &str) -> ([i8; PHASE_CELLS], u16, Vec<u64>) {
    let mut sums = [0i16; PHASE_CELLS];
    let mut keys = Vec::new();
    let mut atom_count = 0usize;
    visit_surface_atoms(text, |position, bytes| {
        atom_count = atom_count.saturating_add(1);
        keys.push(atom_key(0x4c31_504f_5349_5449, position, bytes));
        keys.push(atom_key(0x4c31_5245_4c41_5845, 0, bytes));
        for trit in surface_atom_projection(position, bytes) {
            let cell = usize::from(trit.lane) % PHASE_CELLS;
            sums[cell] = sums[cell].saturating_add(i16::from(trit.value));
        }
    });
    keys.sort_unstable();
    keys.dedup();
    let mut phase = [0i8; PHASE_CELLS];
    for (target, value) in phase.iter_mut().zip(sums) {
        *target = value.clamp(i8::MIN as i16, i8::MAX as i16) as i8;
    }
    (phase, atom_count.min(u16::MAX as usize) as u16, keys)
}

pub(super) fn phase_coherence_milli(left: &[i8; PHASE_CELLS], right: &[i8; PHASE_CELLS]) -> u16 {
    let mut dot = 0f64;
    let mut left_norm = 0f64;
    let mut right_norm = 0f64;
    for (left, right) in left.iter().zip(right) {
        let left = f64::from(*left);
        let right = f64::from(*right);
        dot += left * right;
        left_norm += left * left;
        right_norm += right * right;
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        return 0;
    }
    (((dot / (left_norm.sqrt() * right_norm.sqrt()) + 1.0) * 500.0)
        .round()
        .clamp(0.0, 1_000.0)) as u16
}

fn atom_key(seed: u64, position: u64, bytes: &[u8]) -> u64 {
    let mut state = seed ^ position.rotate_left(17);
    for byte in bytes {
        state ^= u64::from(*byte).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        state = mix64_golden(state);
    }
    mix64_golden(state ^ bytes.len() as u64)
}

pub(super) fn checksum(bytes: &[u8]) -> u64 {
    let mut state = 0x4c45_5849_4341_4c31u64;
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100_0000_01b3);
    }
    mix64_golden(state ^ bytes.len() as u64)
}

pub(super) fn read_header(bytes: &[u8]) -> Result<ArtifactHeader, String> {
    if bytes.len() < HEADER_BYTES || bytes.get(..8) != Some(MAGIC.as_slice()) {
        return Err("invalid lexical phase artifact magic".to_string());
    }
    if read_u32(bytes, 8)? != VERSION || read_u32(bytes, 12)? as usize != HEADER_BYTES {
        return Err("unsupported lexical phase artifact version".to_string());
    }
    let header = ArtifactHeader {
        file_bytes: read_u64(bytes, 16)?,
        node_count: read_u32(bytes, 24)?,
        arc_count: read_u32(bytes, 28)?,
        terminal_count: read_u32(bytes, 32)?,
        center_count: read_u32(bytes, 36)?,
        posting_count: read_u32(bytes, 40)?,
        nodes_offset: read_u64(bytes, 48)?,
        arcs_offset: read_u64(bytes, 56)?,
        terminals_offset: read_u64(bytes, 64)?,
        centers_offset: read_u64(bytes, 72)?,
        postings_offset: read_u64(bytes, 80)?,
        checksum: read_u64(bytes, 88)?,
        corpus_hash: read_u64(bytes, 96)?,
        source_words: read_u32(bytes, 104)?,
        decoder_state_count: read_u32(bytes, 108)?,
        decoder_arc_count: read_u32(bytes, 112)?,
        decoder_states_offset: read_u64(bytes, 116)?,
        training_surfaces: read_u32(bytes, 124)?,
    };
    if header.file_bytes as usize != bytes.len()
        || header.nodes_offset as usize != HEADER_BYTES
        || header.arcs_offset as usize != HEADER_BYTES + header.node_count as usize * NODE_BYTES
        || header.terminals_offset as usize
            != header.arcs_offset as usize + header.arc_count as usize * ARC_BYTES
        || header.centers_offset as usize
            != header.terminals_offset as usize + header.terminal_count as usize * TERMINAL_BYTES
        || header.postings_offset as usize
            != header.centers_offset as usize + header.center_count as usize * CENTER_BYTES
        || header.decoder_states_offset as usize
            != header.postings_offset as usize + header.posting_count as usize * POSTING_BYTES
        || header.decoder_states_offset as usize
            + header.decoder_state_count as usize * DECODER_STATE_BYTES
            + header.decoder_arc_count as usize * DECODER_ARC_BYTES
            != bytes.len()
    {
        return Err("invalid lexical phase artifact offsets".to_string());
    }
    if checksum(&bytes[HEADER_BYTES..]) != header.checksum {
        return Err("lexical phase artifact checksum mismatch".to_string());
    }
    Ok(header)
}

pub(super) fn read_node(bytes: &[u8], header: ArtifactHeader, index: u32) -> Option<NodeRecord> {
    if index >= header.node_count {
        return None;
    }
    let offset = header.nodes_offset as usize + index as usize * NODE_BYTES;
    Some(NodeRecord {
        parent: read_u32(bytes, offset).ok()?,
        incoming: read_u32(bytes, offset + 4).ok()?,
        first_arc: read_u32(bytes, offset + 8).ok()?,
        arc_len: read_u16(bytes, offset + 12).ok()?,
        depth: read_u16(bytes, offset + 14).ok()?,
        terminal: read_u32(bytes, offset + 16).ok()?,
        best_terminal: read_u32(bytes, offset + 20).ok()?,
    })
}

pub(super) fn read_arc(bytes: &[u8], header: ArtifactHeader, index: u32) -> Option<ArcRecord> {
    if index >= header.arc_count {
        return None;
    }
    let offset = header.arcs_offset as usize + index as usize * ARC_BYTES;
    Some(ArcRecord {
        ch: read_u32(bytes, offset).ok()?,
        child: read_u32(bytes, offset + 4).ok()?,
    })
}

pub(super) fn read_terminal(
    bytes: &[u8],
    header: ArtifactHeader,
    index: u32,
) -> Option<TerminalRecord> {
    if index >= header.terminal_count {
        return None;
    }
    let offset = header.terminals_offset as usize + index as usize * TERMINAL_BYTES;
    let mut phase = [0i8; PHASE_CELLS];
    for (target, source) in phase
        .iter_mut()
        .zip(bytes.get(offset + 16..offset + TERMINAL_BYTES)?)
    {
        *target = *source as i8;
    }
    Some(TerminalRecord {
        node: read_u32(bytes, offset).ok()?,
        rank: read_u32(bytes, offset + 4).ok()?,
        support: read_u32(bytes, offset + 8).ok()?,
        char_len: read_u16(bytes, offset + 12).ok()?,
        atom_count: read_u16(bytes, offset + 14).ok()?,
        phase,
    })
}

pub(super) fn read_center(
    bytes: &[u8],
    header: ArtifactHeader,
    index: u32,
) -> Option<CenterRecord> {
    if index >= header.center_count {
        return None;
    }
    let offset = header.centers_offset as usize + index as usize * CENTER_BYTES;
    Some(CenterRecord {
        key: read_u64(bytes, offset).ok()?,
        posting_start: read_u32(bytes, offset + 8).ok()?,
        posting_len: read_u16(bytes, offset + 12).ok()?,
        support: read_u16(bytes, offset + 14).ok()?,
    })
}

pub(super) fn read_posting(bytes: &[u8], header: ArtifactHeader, index: u32) -> Option<u32> {
    if index >= header.posting_count {
        return None;
    }
    read_u32(
        bytes,
        header.postings_offset as usize + index as usize * POSTING_BYTES,
    )
    .ok()
}

pub(super) fn read_decoder_state(
    bytes: &[u8],
    header: ArtifactHeader,
    index: u32,
) -> Option<DecoderStateRecord> {
    if index >= header.decoder_state_count {
        return None;
    }
    let offset = header.decoder_states_offset as usize + index as usize * DECODER_STATE_BYTES;
    Some(DecoderStateRecord {
        first_arc: read_u32(bytes, offset).ok()?,
        arc_len: read_u16(bytes, offset + 4).ok()?,
        flags: read_u16(bytes, offset + 6).ok()?,
    })
}

pub(super) fn read_decoder_arc(
    bytes: &[u8],
    header: ArtifactHeader,
    index: u32,
) -> Option<DecoderArcRecord> {
    if index >= header.decoder_arc_count {
        return None;
    }
    let states_bytes = header.decoder_state_count as usize * DECODER_STATE_BYTES;
    let offset =
        header.decoder_states_offset as usize + states_bytes + index as usize * DECODER_ARC_BYTES;
    Some(DecoderArcRecord {
        ch: read_u32(bytes, offset).ok()?,
        child: read_u32(bytes, offset + 4).ok()?,
    })
}

pub(super) fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "truncated lexical phase artifact".to_string())?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "truncated lexical phase artifact".to_string())?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| "truncated lexical phase artifact".to_string())?;
    Ok(u64::from_le_bytes([
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexical_surface_atoms::SurfaceFieldEncoder;

    #[test]
    fn streaming_surface_summary_matches_materialized_atoms() {
        for surface in ["п", "пров", "перезагрузки", "file", "don't", "слово-форма"]
        {
            let field = SurfaceFieldEncoder::encode(surface);
            let expected_phase = surface_phase(&field);
            let expected_keys = atom_center_keys(&field);
            let (phase, atom_count, keys) = surface_phase_and_atom_center_keys(surface);

            assert_eq!((phase, atom_count), expected_phase, "surface={surface}");
            assert_eq!(keys, expected_keys, "surface={surface}");
        }
    }
}
