use std::fs;
use std::io;
use std::path::Path;

use super::super::phase_field::{dequantize, quantize, PhaseCell, PhaseCenter};
use super::{ContextCandidateProfile, ContextPhasePackage, TokenSemanticState, CELLS, MAGIC};

const VERSION: u16 = 2;
const HEADER_BYTES: usize = 48;
const SEMANTIC_HEADER_BYTES: usize = 16;
const PROFILE_HEADER_BYTES_V1: usize = 24;
const PROFILE_HEADER_BYTES_V2: usize = 28;
const CENTER_HEADER_BYTES: usize = 4;
const VECTOR_BYTES: usize = CELLS * 2;

pub(crate) fn write_package(path: &Path, package: &ContextPhasePackage) -> io::Result<()> {
    let mut bytes = Vec::with_capacity(
        HEADER_BYTES
            + package.semantic_states.len() * (SEMANTIC_HEADER_BYTES + VECTOR_BYTES)
            + package
                .profiles
                .iter()
                .map(|profile| {
                    PROFILE_HEADER_BYTES_V2
                        + (profile.positive.len()
                            + profile.negative.len()
                            + profile.hard_negative.len())
                            * (CENTER_HEADER_BYTES + VECTOR_BYTES)
                })
                .sum::<usize>(),
    );
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&(CELLS as u16).to_le_bytes());
    bytes.extend_from_slice(&(package.semantic_states.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(package.profiles.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&package.transitions.to_le_bytes());
    bytes.extend_from_slice(&package.corpus_fragments.to_le_bytes());
    bytes.extend_from_slice(&package.global_threshold_micro.to_le_bytes());
    bytes.extend_from_slice(&package.competition_threshold_micro.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    debug_assert_eq!(bytes.len(), HEADER_BYTES);

    for state in &package.semantic_states {
        bytes.extend_from_slice(&state.token_hash.to_le_bytes());
        bytes.extend_from_slice(&state.support.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        write_vector(&mut bytes, &state.center);
    }
    for profile in &package.profiles {
        bytes.extend_from_slice(&profile.token_hash.to_le_bytes());
        bytes.extend_from_slice(&profile.positive_examples.to_le_bytes());
        bytes.extend_from_slice(&profile.negative_examples.to_le_bytes());
        bytes.extend_from_slice(&profile.threshold_micro.to_le_bytes());
        bytes.extend_from_slice(&(profile.positive.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&(profile.negative.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&(profile.hard_negative.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        for center in profile
            .positive
            .iter()
            .chain(&profile.negative)
            .chain(&profile.hard_negative)
        {
            bytes.extend_from_slice(&center.support.to_le_bytes());
            write_vector(&mut bytes, &center.center);
        }
    }
    crate::private_file::write_private_bytes(path, &bytes)
}

pub(crate) fn read_package(path: &Path) -> io::Result<ContextPhasePackage> {
    decode_package(&fs::read(path)?)
}

fn decode_package(bytes: &[u8]) -> io::Result<ContextPhasePackage> {
    if bytes.len() < HEADER_BYTES || &bytes[..8] != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid L3 context phase package magic",
        ));
    }
    let version = read_u16(bytes, 8)?;
    let cells = read_u16(bytes, 10)? as usize;
    if !(1..=VERSION).contains(&version) || cells != CELLS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported L3 context phase package version",
        ));
    }
    let semantic_count = read_u32(bytes, 12)? as usize;
    let profile_count = read_u32(bytes, 16)? as usize;
    let transitions = read_u64(bytes, 20)?;
    let corpus_fragments = read_u32(bytes, 28)?;
    let global_threshold_micro = read_i32(bytes, 32)?;
    let competition_threshold_micro = read_i32(bytes, 36)?;
    let mut offset = HEADER_BYTES;

    let mut semantic_states = Vec::with_capacity(semantic_count);
    for _ in 0..semantic_count {
        require(bytes, offset, SEMANTIC_HEADER_BYTES + VECTOR_BYTES)?;
        let token_hash = read_u64(bytes, offset)?;
        let support = read_u32(bytes, offset + 8)?;
        offset += SEMANTIC_HEADER_BYTES;
        let center = read_vector(bytes, &mut offset)?;
        semantic_states.push(TokenSemanticState {
            token_hash,
            support,
            center,
        });
    }

    let profile_header_bytes = if version == 1 {
        PROFILE_HEADER_BYTES_V1
    } else {
        PROFILE_HEADER_BYTES_V2
    };
    let mut profiles = Vec::with_capacity(profile_count);
    for _ in 0..profile_count {
        require(bytes, offset, profile_header_bytes)?;
        let token_hash = read_u64(bytes, offset)?;
        let positive_examples = read_u32(bytes, offset + 8)?;
        let negative_examples = read_u32(bytes, offset + 12)?;
        let threshold_micro = read_i32(bytes, offset + 16)?;
        let positive_count = read_u16(bytes, offset + 20)? as usize;
        let negative_count = read_u16(bytes, offset + 22)? as usize;
        let hard_negative_count = if version == 1 {
            0
        } else {
            read_u16(bytes, offset + 24)? as usize
        };
        offset += profile_header_bytes;
        let positive = read_centers(bytes, &mut offset, positive_count)?;
        let negative = read_centers(bytes, &mut offset, negative_count)?;
        let hard_negative = read_centers(bytes, &mut offset, hard_negative_count)?;
        profiles.push(ContextCandidateProfile {
            token_hash,
            positive_examples,
            negative_examples,
            threshold_micro,
            positive,
            negative,
            hard_negative,
        });
    }
    if offset != bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L3 context phase package has trailing bytes",
        ));
    }
    semantic_states.sort_by_key(|state| state.token_hash);
    profiles.sort_by_key(|profile| profile.token_hash);
    Ok(ContextPhasePackage {
        semantic_states,
        profiles,
        transitions,
        corpus_fragments,
        global_threshold_micro,
        competition_threshold_micro,
    })
}

fn write_vector(bytes: &mut Vec<u8>, vector: &[PhaseCell]) {
    for index in 0..CELLS {
        let cell = vector.get(index).copied().unwrap_or_default();
        bytes.push(quantize(cell.re) as u8);
        bytes.push(quantize(cell.im) as u8);
    }
}

fn read_vector(bytes: &[u8], offset: &mut usize) -> io::Result<Vec<PhaseCell>> {
    require(bytes, *offset, VECTOR_BYTES)?;
    let mut vector = Vec::with_capacity(CELLS);
    for _ in 0..CELLS {
        vector.push(PhaseCell {
            re: dequantize(bytes[*offset] as i8),
            im: dequantize(bytes[*offset + 1] as i8),
        });
        *offset += 2;
    }
    Ok(vector)
}

fn read_centers(bytes: &[u8], offset: &mut usize, count: usize) -> io::Result<Vec<PhaseCenter>> {
    let mut centers = Vec::with_capacity(count);
    for _ in 0..count {
        require(bytes, *offset, CENTER_HEADER_BYTES + VECTOR_BYTES)?;
        let support = read_u32(bytes, *offset)?;
        *offset += CENTER_HEADER_BYTES;
        centers.push(PhaseCenter::from_center(
            read_vector(bytes, offset)?,
            support,
        ));
    }
    Ok(centers)
}

fn require(bytes: &[u8], offset: usize, width: usize) -> io::Result<()> {
    if offset.saturating_add(width) > bytes.len() {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated L3 context phase package",
        ))
    } else {
        Ok(())
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    require(bytes, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    require(bytes, offset, 4)?;
    Ok(u32::from_le_bytes(
        bytes[offset..offset + 4].try_into().unwrap(),
    ))
}

fn read_i32(bytes: &[u8], offset: usize) -> io::Result<i32> {
    require(bytes, offset, 4)?;
    Ok(i32::from_le_bytes(
        bytes[offset..offset + 4].try_into().unwrap(),
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> io::Result<u64> {
    require(bytes, offset, 8)?;
    Ok(u64::from_le_bytes(
        bytes[offset..offset + 8].try_into().unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::phase_field::PhaseCenter;

    #[test]
    fn package_roundtrip_keeps_compact_centers_without_words() {
        let package = ContextPhasePackage {
            semantic_states: vec![TokenSemanticState {
                token_hash: 11,
                support: 4,
                center: vec![PhaseCell { re: 1.0, im: 0.0 }; CELLS],
            }],
            profiles: vec![ContextCandidateProfile {
                token_hash: 17,
                positive_examples: 5,
                negative_examples: 2,
                threshold_micro: 25_000,
                positive: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: 0.0, im: 1.0 }; CELLS],
                    5,
                )],
                negative: Vec::new(),
                hard_negative: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: -1.0, im: 0.0 }; CELLS],
                    2,
                )],
            }],
            transitions: 9,
            corpus_fragments: 3,
            global_threshold_micro: 10_000,
            competition_threshold_micro: 20_000,
        };
        let dir = std::env::temp_dir().join(format!("lay-l3-phase-{}", std::process::id()));
        let path = dir.join("memory.nwpc");
        std::fs::create_dir_all(&dir).unwrap();
        write_package(&path, &package).unwrap();
        let decoded = read_package(&path).unwrap();

        assert_eq!(decoded.semantic_states.len(), 1);
        assert_eq!(decoded.profiles.len(), 1);
        assert_eq!(decoded.profiles[0].hard_negative.len(), 1);
        assert_eq!(decoded.transitions, 9);
        assert!(!std::fs::read(&path)
            .unwrap()
            .windows("word".len())
            .any(|window| window == b"word"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
