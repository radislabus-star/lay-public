use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::super::phase_field::{dequantize, quantize, PhaseCell, PhaseCenter};
use super::{
    ContextCandidateProfile, ContextPairPhaseProfile, ContextPhasePackage, TokenSemanticState,
    CELLS, MAGIC, MAX_HARD_PAIR_CENTERS_PER_BANK, MAX_PAIR_CENTERS_PER_BANK, MAX_PAIR_PROFILES,
    MAX_SIGNATURE_PROFILES,
};

const VERSION: u16 = 5;
const HEADER_BYTES_V1_TO_V3: usize = 48;
const HEADER_BYTES_V4: usize = 52;
const HEADER_BYTES_V5: usize = 56;
const SEMANTIC_HEADER_BYTES: usize = 16;
const PROFILE_HEADER_BYTES_V1: usize = 24;
const PROFILE_HEADER_BYTES_V2: usize = 28;
const CENTER_HEADER_BYTES: usize = 4;
const PAIR_PROFILE_HEADER_BYTES: usize = 24;
const VECTOR_BYTES: usize = CELLS * 2;

pub(crate) fn write_package(path: &Path, package: &ContextPhasePackage) -> io::Result<()> {
    let bytes = encode_package(package);
    let temporary = temporary_package_path(path);
    crate::private_file::write_private_bytes(&temporary, &bytes)?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

pub(crate) fn encode_package(package: &ContextPhasePackage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(
        HEADER_BYTES_V5
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
                .sum::<usize>()
            + package
                .pair_profiles
                .iter()
                .map(|pair| {
                    PAIR_PROFILE_HEADER_BYTES
                        + (pair.low_wins.len()
                            + pair.high_wins.len()
                            + pair.hard_low_wins.len()
                            + pair.hard_high_wins.len())
                            * (CENTER_HEADER_BYTES + VECTOR_BYTES)
                })
                .sum::<usize>()
            + package
                .signature_profiles
                .iter()
                .map(profile_encoded_bytes)
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
    bytes.extend_from_slice(&(package.pair_profiles.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&package.pairwise_threshold_micro.to_le_bytes());
    bytes.extend_from_slice(&(package.signature_profiles.len() as u32).to_le_bytes());
    let signature_schema = if package.signature_schema == 0 {
        super::SIGNATURE_SCHEMA_MORPHOLOGY_PHASE
    } else {
        package.signature_schema
    };
    bytes.extend_from_slice(&signature_schema.to_le_bytes());
    debug_assert_eq!(bytes.len(), HEADER_BYTES_V5);

    for state in &package.semantic_states {
        bytes.extend_from_slice(&state.token_hash.to_le_bytes());
        bytes.extend_from_slice(&state.support.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        write_vector(&mut bytes, &state.center);
    }
    for profile in &package.profiles {
        write_profile(&mut bytes, profile);
    }
    for pair in &package.pair_profiles {
        write_pair_profile(&mut bytes, pair);
    }
    for profile in &package.signature_profiles {
        write_profile(&mut bytes, profile);
    }
    bytes
}

fn write_pair_profile(bytes: &mut Vec<u8>, pair: &ContextPairPhaseProfile) {
    bytes.extend_from_slice(&pair.low_hash.to_le_bytes());
    bytes.extend_from_slice(&pair.high_hash.to_le_bytes());
    for count in [
        pair.low_wins.len(),
        pair.high_wins.len(),
        pair.hard_low_wins.len(),
        pair.hard_high_wins.len(),
    ] {
        bytes.extend_from_slice(&(count as u16).to_le_bytes());
    }
    for center in pair
        .low_wins
        .iter()
        .chain(&pair.high_wins)
        .chain(&pair.hard_low_wins)
        .chain(&pair.hard_high_wins)
    {
        bytes.extend_from_slice(&center.support.to_le_bytes());
        center.write_compact(bytes);
    }
}

fn temporary_package_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("context-phase.nwpc");
    path.with_file_name(format!(".{name}.{}.tmp", std::process::id()))
}

fn write_profile(bytes: &mut Vec<u8>, profile: &ContextCandidateProfile) {
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
        center.write_compact(bytes);
    }
}

fn profile_encoded_bytes(profile: &ContextCandidateProfile) -> usize {
    PROFILE_HEADER_BYTES_V2
        + (profile.positive.len() + profile.negative.len() + profile.hard_negative.len())
            * (CENTER_HEADER_BYTES + VECTOR_BYTES)
}

pub(crate) fn read_package(path: &Path) -> io::Result<ContextPhasePackage> {
    decode_package_owned(fs::read(path)?.into())
}

#[cfg(test)]
fn decode_package(bytes: &[u8]) -> io::Result<ContextPhasePackage> {
    decode_package_owned(Arc::from(bytes))
}

fn decode_package_owned(backing: Arc<[u8]>) -> io::Result<ContextPhasePackage> {
    let bytes = backing.as_ref();
    if bytes.len() < HEADER_BYTES_V1_TO_V3 || &bytes[..8] != MAGIC {
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
    let pair_profile_count = if version >= 3 {
        read_u32(bytes, 40)? as usize
    } else {
        0
    };
    let pairwise_threshold_micro = if version >= 3 {
        read_i32(bytes, 44)?
    } else {
        0
    };
    let signature_profile_count = if version >= 4 {
        read_u32(bytes, 48)? as usize
    } else {
        0
    };
    let signature_schema = if version >= 5 {
        read_u32(bytes, 52)?
    } else {
        super::SIGNATURE_SCHEMA_LEGACY
    };
    if version >= 5
        && signature_schema != super::SIGNATURE_SCHEMA_LEGACY
        && signature_schema != super::SIGNATURE_SCHEMA_MORPHOLOGY_ENDING
        && signature_schema != super::SIGNATURE_SCHEMA_MORPHOLOGY_PHASE
        && signature_schema != super::SIGNATURE_SCHEMA_RELATION_ROLES
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported L3 context phase signature schema",
        ));
    }
    let mut offset = if version >= 5 {
        HEADER_BYTES_V5
    } else if version >= 4 {
        HEADER_BYTES_V4
    } else {
        HEADER_BYTES_V1_TO_V3
    };

    if pair_profile_count > MAX_PAIR_PROFILES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L3 context phase package exceeds pair profile budget",
        ));
    }
    if signature_profile_count > MAX_SIGNATURE_PROFILES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L3 context phase package exceeds signature profile budget",
        ));
    }

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
        profiles.push(read_profile(
            bytes,
            Arc::clone(&backing),
            &mut offset,
            version,
            profile_header_bytes,
        )?);
    }
    let mut pair_profiles = Vec::with_capacity(pair_profile_count);
    let mut previous_pair = None;
    for _ in 0..pair_profile_count {
        let pair = read_pair_profile(bytes, Arc::clone(&backing), &mut offset)?;
        let key = (pair.low_hash, pair.high_hash);
        if pair.low_hash >= pair.high_hash || previous_pair.is_some_and(|previous| previous >= key)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid L3 pair profile key",
            ));
        }
        previous_pair = Some(key);
        pair_profiles.push(pair);
    }
    let mut signature_profiles = Vec::with_capacity(signature_profile_count);
    let mut previous_signature = None;
    for _ in 0..signature_profile_count {
        let profile = read_profile(
            bytes,
            Arc::clone(&backing),
            &mut offset,
            version,
            PROFILE_HEADER_BYTES_V2,
        )?;
        if previous_signature.is_some_and(|previous| previous >= profile.token_hash) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid L3 signature profile key",
            ));
        }
        previous_signature = Some(profile.token_hash);
        signature_profiles.push(profile);
    }
    if offset != bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L3 context phase package has trailing bytes",
        ));
    }
    semantic_states.sort_by_key(|state| state.token_hash);
    profiles.sort_by_key(|profile| profile.token_hash);
    signature_profiles.sort_by_key(|profile| profile.token_hash);
    Ok(ContextPhasePackage {
        semantic_states,
        profiles,
        signature_profiles,
        pair_profiles,
        transitions,
        corpus_fragments,
        global_threshold_micro,
        competition_threshold_micro,
        pairwise_threshold_micro,
        signature_schema,
    })
}

fn read_pair_profile(
    bytes: &[u8],
    backing: Arc<[u8]>,
    offset: &mut usize,
) -> io::Result<ContextPairPhaseProfile> {
    require(bytes, *offset, PAIR_PROFILE_HEADER_BYTES)?;
    let low_hash = read_u64(bytes, *offset)?;
    let high_hash = read_u64(bytes, *offset + 8)?;
    let low_count = read_u16(bytes, *offset + 16)? as usize;
    let high_count = read_u16(bytes, *offset + 18)? as usize;
    let hard_low_count = read_u16(bytes, *offset + 20)? as usize;
    let hard_high_count = read_u16(bytes, *offset + 22)? as usize;
    *offset += PAIR_PROFILE_HEADER_BYTES;
    if low_count > MAX_PAIR_CENTERS_PER_BANK
        || high_count > MAX_PAIR_CENTERS_PER_BANK
        || hard_low_count > MAX_HARD_PAIR_CENTERS_PER_BANK
        || hard_high_count > MAX_HARD_PAIR_CENTERS_PER_BANK
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L3 context phase pair bank exceeds center budget",
        ));
    }
    Ok(ContextPairPhaseProfile {
        low_hash,
        high_hash,
        low_wins: read_centers(bytes, Arc::clone(&backing), offset, low_count)?,
        high_wins: read_centers(bytes, Arc::clone(&backing), offset, high_count)?,
        hard_low_wins: read_centers(bytes, Arc::clone(&backing), offset, hard_low_count)?,
        hard_high_wins: read_centers(bytes, backing, offset, hard_high_count)?,
    })
}

fn read_profile(
    bytes: &[u8],
    backing: Arc<[u8]>,
    offset: &mut usize,
    version: u16,
    profile_header_bytes: usize,
) -> io::Result<ContextCandidateProfile> {
    require(bytes, *offset, profile_header_bytes)?;
    let token_hash = read_u64(bytes, *offset)?;
    let positive_examples = read_u32(bytes, *offset + 8)?;
    let negative_examples = read_u32(bytes, *offset + 12)?;
    let threshold_micro = read_i32(bytes, *offset + 16)?;
    let positive_count = read_u16(bytes, *offset + 20)? as usize;
    let negative_count = read_u16(bytes, *offset + 22)? as usize;
    let hard_negative_count = if version == 1 {
        0
    } else {
        read_u16(bytes, *offset + 24)? as usize
    };
    *offset += profile_header_bytes;
    let positive = read_centers(bytes, Arc::clone(&backing), offset, positive_count)?;
    let negative = read_centers(bytes, Arc::clone(&backing), offset, negative_count)?;
    let hard_negative = read_centers(bytes, backing, offset, hard_negative_count)?;
    Ok(ContextCandidateProfile {
        token_hash,
        positive_examples,
        negative_examples,
        threshold_micro,
        positive,
        negative,
        hard_negative,
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

fn read_centers(
    bytes: &[u8],
    backing: Arc<[u8]>,
    offset: &mut usize,
    count: usize,
) -> io::Result<Vec<PhaseCenter>> {
    let mut centers = Vec::with_capacity(count);
    for _ in 0..count {
        require(bytes, *offset, CENTER_HEADER_BYTES + VECTOR_BYTES)?;
        let support = read_u32(bytes, *offset)?;
        *offset += CENTER_HEADER_BYTES;
        let center_offset = *offset;
        *offset += VECTOR_BYTES;
        centers.push(PhaseCenter::from_serialized(
            Arc::clone(&backing),
            center_offset,
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
        let profile = ContextCandidateProfile {
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
        };
        let package = ContextPhasePackage {
            semantic_states: vec![TokenSemanticState {
                token_hash: 11,
                support: 4,
                center: vec![PhaseCell { re: 1.0, im: 0.0 }; CELLS],
            }],
            profiles: vec![profile],
            signature_profiles: vec![ContextCandidateProfile {
                token_hash: 99,
                positive_examples: 3,
                negative_examples: 0,
                threshold_micro: 20_000,
                positive: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: 0.0, im: 1.0 }; CELLS],
                    3,
                )],
                negative: Vec::new(),
                hard_negative: Vec::new(),
            }],
            pair_profiles: vec![ContextPairPhaseProfile {
                low_hash: 17,
                high_hash: 23,
                low_wins: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: 1.0, im: 0.0 }; CELLS],
                    2,
                )],
                high_wins: Vec::new(),
                hard_low_wins: Vec::new(),
                hard_high_wins: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: 0.0, im: 1.0 }; CELLS],
                    1,
                )],
            }],
            transitions: 9,
            corpus_fragments: 3,
            global_threshold_micro: 10_000,
            competition_threshold_micro: 20_000,
            pairwise_threshold_micro: 20_000,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
        };
        let dir = std::env::temp_dir().join(format!("lay-l3-phase-{}", std::process::id()));
        let path = dir.join("memory.nwpc");
        std::fs::create_dir_all(&dir).unwrap();
        write_package(&path, &package).unwrap();
        let decoded = read_package(&path).unwrap();
        let first_bytes = std::fs::read(&path).unwrap();
        write_package(&path, &package).unwrap();
        let second_bytes = std::fs::read(&path).unwrap();

        assert_eq!(decoded.semantic_states.len(), 1);
        assert_eq!(decoded.profiles.len(), 1);
        assert_eq!(decoded.signature_profiles.len(), 1);
        assert_eq!(decoded.signature_profiles[0].token_hash, 99);
        assert_eq!(
            decoded.signature_schema,
            super::super::SIGNATURE_SCHEMA_RELATION_ROLES
        );
        assert_eq!(decoded.profiles[0].hard_negative.len(), 1);
        assert_eq!(decoded.pair_profiles.len(), 1);
        assert_eq!(decoded.pair_profiles[0].low_hash, 17);
        assert_eq!(decoded.pair_profiles[0].hard_high_wins.len(), 1);
        assert_eq!(decoded.transitions, 9);
        assert_eq!(first_bytes, second_bytes);
        assert!(!second_bytes
            .windows("word".len())
            .any(|window| window == b"word"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn v2_header_loads_with_an_empty_pairwise_field() {
        let package = ContextPhasePackage {
            profiles: vec![ContextCandidateProfile {
                token_hash: 17,
                positive_examples: 2,
                negative_examples: 0,
                threshold_micro: 10,
                positive: Vec::new(),
                negative: Vec::new(),
                hard_negative: Vec::new(),
            }],
            ..ContextPhasePackage::default()
        };
        let mut bytes = encode_package(&package);
        bytes[8..10].copy_from_slice(&2_u16.to_le_bytes());
        bytes[40..48].fill(0);
        bytes.drain(48..56);

        let decoded = decode_package(&bytes).unwrap();
        assert_eq!(decoded.profiles.len(), 1);
        assert!(decoded.pair_profiles.is_empty());
        assert_eq!(decoded.pairwise_threshold_micro, 0);
    }

    #[test]
    fn generalized_pair_key_roundtrips_without_candidate_text() {
        let package = ContextPhasePackage {
            pair_profiles: vec![ContextPairPhaseProfile {
                low_hash: 0,
                high_hash: 77,
                low_wins: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: 1.0, im: 0.0 }; CELLS],
                    3,
                )],
                ..ContextPairPhaseProfile::default()
            }],
            ..ContextPhasePackage::default()
        };
        let decoded = decode_package(&encode_package(&package)).unwrap();
        assert_eq!(decoded.pair_profiles[0].low_hash, 0);
        assert_eq!(decoded.pair_profiles[0].high_hash, 77);
        assert_eq!(decoded.pair_profiles[0].low_wins[0].support, 3);
    }

    #[test]
    fn decoder_rejects_pair_bank_over_budget() {
        let center = PhaseCenter::from_center(vec![PhaseCell { re: 1.0, im: 0.0 }; CELLS], 2);
        let package = ContextPhasePackage {
            pair_profiles: vec![ContextPairPhaseProfile {
                low_hash: 17,
                high_hash: 23,
                low_wins: vec![center; MAX_PAIR_CENTERS_PER_BANK + 1],
                ..ContextPairPhaseProfile::default()
            }],
            ..ContextPhasePackage::default()
        };

        let error = decode_package(&encode_package(&package)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
