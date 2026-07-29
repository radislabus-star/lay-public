//! Proof codec for scale-complete forward postings.
//!
//! Forward relations are sorted by terminal ID and split into independently
//! skippable blocks. Phase is derived from position, while the forward lane has
//! no per-relation flags, so neither value needs to occupy package bytes.

use std::io;
use std::path::Path;

use super::format;
use super::model::WaveCoupling;
use super::runtime::{LexicalGrokkingMemory, ReadoutMode};

const POSTING_BLOCK_RELATIONS: usize = 32;
const BLOCK_HEADER_BYTES: usize = 8;
const RAW_COUPLING_BYTES: usize = 8;

pub fn analyze_package(path: &Path) -> io::Result<serde_json::Value> {
    let package_bytes = std::fs::read(path)?;
    let package = format::decode(&package_bytes).map_err(io::Error::other)?;
    let mut encoded_bytes = 0_usize;
    let mut block_count = 0_usize;
    let mut parity = true;

    for atom in &package.atoms {
        let start = atom.coupling_start as usize;
        let end = start.saturating_add(atom.coupling_count as usize);
        let relations = package
            .forward_couplings
            .get(start..end)
            .ok_or_else(|| io::Error::other("invalid forward posting range"))?;
        let encoded = encode_posting(relations).map_err(io::Error::other)?;
        block_count = block_count.saturating_add(encoded.block_count);
        encoded_bytes = encoded_bytes.saturating_add(encoded.bytes.len());
        let decoded = decode_posting(&encoded.bytes, relations.len()).map_err(io::Error::other)?;
        parity &= canonical_relations(relations) == decoded;
    }

    let relation_count = package.forward_couplings.len();
    let raw_bytes = relation_count.saturating_mul(RAW_COUPLING_BYTES);
    let projected_package_bytes = package_bytes
        .len()
        .saturating_sub(raw_bytes)
        .saturating_add(encoded_bytes);
    let compression_ratio = if encoded_bytes == 0 {
        1.0
    } else {
        raw_bytes as f64 / encoded_bytes as f64
    };
    let v3_bytes = format::encode(&package).map_err(io::Error::other)?;
    let migrated = LexicalGrokkingMemory::from_bytes(&v3_bytes).map_err(io::Error::other)?;
    let original = LexicalGrokkingMemory::from_package(package.clone());
    let parity_samples = package.terminal_count().min(512) as usize;
    let sample_stride = (package.terminal_count() as usize)
        .div_ceil(parity_samples.max(1))
        .max(1);
    let mut exact_topk_parity = true;
    let mut checked_samples = 0_usize;
    for terminal_id in (0..package.terminal_count() as usize)
        .step_by(sample_stride)
        .take(parity_samples)
    {
        let Some(surface) = original.decode_terminal(terminal_id as u32) else {
            exact_topk_parity = false;
            break;
        };
        exact_topk_parity &= original.readout(&surface, 64, ReadoutMode::Full)
            == migrated.readout(&surface, 64, ReadoutMode::Full);
        checked_samples += 1;
        if !exact_topk_parity {
            break;
        }
    }
    let verdict = if parity && exact_topk_parity && compression_ratio >= 1.8 {
        "PASS_codec_candidate"
    } else {
        "WATCH_codec_candidate"
    };

    Ok(serde_json::json!({
        "package": path,
        "terminal_count": package.terminal_count(),
        "atom_count": package.atoms.len(),
        "forward_relations": relation_count,
        "posting_block_relations": POSTING_BLOCK_RELATIONS,
        "posting_blocks": block_count,
        "raw_forward_bytes": raw_bytes,
        "compressed_forward_bytes": encoded_bytes,
        "compression_ratio": compression_ratio,
        "average_bytes_per_relation": if relation_count == 0 {
            0.0
        } else {
            encoded_bytes as f64 / relation_count as f64
        },
        "original_package_bytes": package_bytes.len(),
        "projected_package_bytes": projected_package_bytes,
        "actual_v3_package_bytes": v3_bytes.len(),
        "coupling_roundtrip_parity": parity,
        "exact_topk_parity": exact_topk_parity,
        "exact_topk_parity_samples": checked_samples,
        "minimum_required_ratio": 1.8,
        "verdict": verdict,
    }))
}

pub(super) struct EncodedPosting {
    pub(super) bytes: Vec<u8>,
    pub(super) block_count: usize,
}

pub(super) fn encode_posting(relations: &[WaveCoupling]) -> Result<EncodedPosting, String> {
    let relations = canonical_relations(relations);
    let mut bytes = Vec::new();
    let mut block_count = 0_usize;
    for block in relations.chunks(POSTING_BLOCK_RELATIONS) {
        let first_peer = block.first().map(|item| item.peer_id).unwrap_or_default();
        let max_strength = block
            .iter()
            .map(|item| item.strength)
            .max()
            .unwrap_or_default();
        let mut payload = Vec::new();
        let mut previous = first_peer;
        for (index, relation) in block.iter().copied().enumerate() {
            validate_compact_relation(relation)?;
            let delta = if index == 0 {
                0
            } else {
                relation
                    .peer_id
                    .checked_sub(previous)
                    .ok_or_else(|| "forward posting peer IDs are not ordered".to_string())?
            };
            write_var_u32(&mut payload, delta);
            payload.push(relation.strength);
            payload.push(relation.position_mode);
            previous = relation.peer_id;
        }
        let payload_len = u16::try_from(payload.len())
            .map_err(|_| "compressed posting block exceeds u16".to_string())?;
        bytes.extend_from_slice(&first_peer.to_le_bytes());
        bytes.extend_from_slice(&payload_len.to_le_bytes());
        bytes.push(block.len() as u8);
        bytes.push(max_strength);
        bytes.extend_from_slice(&payload);
        block_count += 1;
    }
    Ok(EncodedPosting { bytes, block_count })
}

pub(super) fn decode_posting(
    bytes: &[u8],
    expected_count: usize,
) -> Result<Vec<WaveCoupling>, String> {
    let mut cursor = 0_usize;
    let mut relations = Vec::with_capacity(expected_count);
    while relations.len() < expected_count {
        let header = bytes
            .get(cursor..cursor + BLOCK_HEADER_BYTES)
            .ok_or_else(|| "truncated posting block header".to_string())?;
        let first_peer = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let payload_len = u16::from_le_bytes([header[4], header[5]]) as usize;
        let count = header[6] as usize;
        let stored_max_strength = header[7];
        if count == 0 || count > POSTING_BLOCK_RELATIONS {
            return Err("invalid posting block relation count".to_string());
        }
        cursor += BLOCK_HEADER_BYTES;
        let payload_end = cursor.saturating_add(payload_len);
        let payload = bytes
            .get(cursor..payload_end)
            .ok_or_else(|| "truncated posting block payload".to_string())?;
        let mut payload_cursor = 0_usize;
        let mut previous = first_peer;
        let mut decoded_max_strength = 0_u8;
        for index in 0..count {
            let delta = read_var_u32(payload, &mut payload_cursor)?;
            let peer_id = if index == 0 {
                if delta != 0 {
                    return Err("first posting delta must be zero".to_string());
                }
                first_peer
            } else {
                previous
                    .checked_add(delta)
                    .ok_or_else(|| "posting peer delta overflow".to_string())?
            };
            let strength = *payload
                .get(payload_cursor)
                .ok_or_else(|| "truncated posting strength".to_string())?;
            let position_mode = *payload
                .get(payload_cursor + 1)
                .ok_or_else(|| "truncated posting position".to_string())?;
            payload_cursor += 2;
            decoded_max_strength = decoded_max_strength.max(strength);
            relations.push(WaveCoupling {
                peer_id,
                strength,
                phase_relation: phase_from_position(position_mode),
                position_mode,
                flags: 0,
            });
            previous = peer_id;
        }
        if payload_cursor != payload.len() || decoded_max_strength != stored_max_strength {
            return Err("posting block metadata mismatch".to_string());
        }
        cursor = payload_end;
    }
    if relations.len() != expected_count || cursor != bytes.len() {
        return Err("compressed posting relation count mismatch".to_string());
    }
    Ok(relations)
}

pub(super) fn canonical_relations(relations: &[WaveCoupling]) -> Vec<WaveCoupling> {
    let mut canonical = relations.to_vec();
    canonical.sort_unstable_by_key(|item| item.peer_id);
    canonical
}

fn validate_compact_relation(relation: WaveCoupling) -> Result<(), String> {
    if relation.flags != 0 {
        return Err("forward posting codec cannot discard relation flags".to_string());
    }
    if relation.phase_relation != phase_from_position(relation.position_mode) {
        return Err("forward posting phase is not derivable from position".to_string());
    }
    Ok(())
}

fn phase_from_position(position: u8) -> i8 {
    (i16::from(position) - 128).clamp(-127, 127) as i8
}

fn write_var_u32(bytes: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        bytes.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

fn read_var_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, String> {
    let mut value = 0_u32;
    for shift in (0..35).step_by(7) {
        let byte = *bytes
            .get(*cursor)
            .ok_or_else(|| "truncated posting varint".to_string())?;
        *cursor += 1;
        if shift == 28 && byte > 0x0f {
            return Err("posting varint overflows u32".to_string());
        }
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("posting varint exceeds five bytes".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relation(peer_id: u32, strength: u8, position_mode: u8) -> WaveCoupling {
        WaveCoupling {
            peer_id,
            strength,
            phase_relation: phase_from_position(position_mode),
            position_mode,
            flags: 0,
        }
    }

    #[test]
    fn posting_blocks_roundtrip_exact_relations() {
        let relations = (0..91_u32)
            .map(|index| {
                relation(
                    index.saturating_mul(index + 3),
                    255 - index as u8,
                    index as u8,
                )
            })
            .rev()
            .collect::<Vec<_>>();
        let encoded = encode_posting(&relations).expect("encode posting");
        let decoded = decode_posting(&encoded.bytes, relations.len()).expect("decode posting");
        assert_eq!(decoded, canonical_relations(&relations));
        assert_eq!(encoded.block_count, 3);
    }

    #[test]
    fn posting_codec_refuses_non_derivable_metadata() {
        let mut invalid = relation(7, 200, 12);
        invalid.phase_relation = 99;
        assert!(encode_posting(&[invalid]).is_err());
        invalid.phase_relation = phase_from_position(invalid.position_mode);
        invalid.flags = 1;
        assert!(encode_posting(&[invalid]).is_err());
    }
}
