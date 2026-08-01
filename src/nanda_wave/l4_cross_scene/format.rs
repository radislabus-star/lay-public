use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::model::{
    L4CrossScenePackage, L4CrossScenePairProfile, L4CrossSceneProfile, L4CrossSceneProfileKey,
};
use super::{
    CELLS, ENCODER_HASH, ENCODER_VERSION, MAX_AMBIGUITY_CENTERS_PER_BANK, MAX_CENTERS_PER_BANK,
    MAX_HARD_CENTERS_PER_BANK, MAX_PAIR_PROFILES, MAX_PROFILES,
};
use crate::nanda_wave::phase_field::PhaseCenter;
use crate::transition_relation::TransitionOperatorKind;
use crate::typing_memory::{LayoutProjectionDirection, LayoutProjectionScope};

const MAGIC: &[u8; 8] = b"LAYL4CS\0";
const VERSION: u16 = 1;
const HEADER_BYTES: usize = 60;
const PROFILE_HEADER_BYTES: usize = 32;
const PAIR_HEADER_BYTES: usize = 36;
const CENTER_BYTES: usize = 4 + CELLS * 2;
const CHECKSUM_BYTES: usize = 8;
const MAX_PACKAGE_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn write_package(path: &Path, package: &L4CrossScenePackage) -> io::Result<()> {
    let bytes = encode_package(package);
    let temporary = temporary_path(path);
    crate::private_file::write_private_bytes(&temporary, &bytes)?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

pub(crate) fn read_package(path: &Path) -> io::Result<L4CrossScenePackage> {
    let bytes = fs::read(path)?;
    decode_package(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub(crate) fn encode_package(package: &L4CrossScenePackage) -> Vec<u8> {
    let center_count = package
        .profiles
        .iter()
        .map(|profile| {
            profile.positive.len()
                + profile.negative.len()
                + profile.hard_negative.len()
                + profile.ambiguity.len()
        })
        .chain(package.pair_profiles.iter().map(|pair| {
            pair.low_wins.len()
                + pair.high_wins.len()
                + pair.hard_low_wins.len()
                + pair.hard_high_wins.len()
                + pair.ambiguity.len()
        }))
        .sum::<usize>();
    let mut bytes = Vec::with_capacity(
        HEADER_BYTES
            + package.profiles.len() * PROFILE_HEADER_BYTES
            + package.pair_profiles.len() * PAIR_HEADER_BYTES
            + center_count * CENTER_BYTES
            + CHECKSUM_BYTES,
    );
    bytes.extend_from_slice(MAGIC);
    put_u16(&mut bytes, VERSION);
    put_u16(&mut bytes, CELLS as u16);
    put_u32(&mut bytes, ENCODER_VERSION);
    put_u64(&mut bytes, ENCODER_HASH);
    put_u32(&mut bytes, package.profiles.len() as u32);
    put_u32(&mut bytes, package.pair_profiles.len() as u32);
    put_u32(&mut bytes, package.source_observations);
    put_u32(&mut bytes, package.joined_observations);
    put_u32(&mut bytes, package.positive_observations);
    put_u32(&mut bytes, package.negative_observations);
    put_u32(&mut bytes, package.reverted_observations);
    put_u32(&mut bytes, package.ambiguity_observations);
    put_u32(&mut bytes, package.censored_observations);
    debug_assert_eq!(bytes.len(), HEADER_BYTES);

    for profile in &package.profiles {
        write_profile(&mut bytes, profile);
    }
    for pair in &package.pair_profiles {
        write_pair(&mut bytes, pair);
    }
    let checksum = checksum64(&bytes);
    put_u64(&mut bytes, checksum);
    bytes
}

/// Returns the exact compact representation used by runtime readout.
///
/// Calibration and proof must not observe the higher-precision learner
/// centers after the package format has quantized them to signed bytes.
pub(crate) fn canonical_runtime_package(
    package: &L4CrossScenePackage,
) -> Result<L4CrossScenePackage, String> {
    decode_package(encode_package(package))
}

fn decode_package(bytes: Vec<u8>) -> Result<L4CrossScenePackage, String> {
    if bytes.len() < HEADER_BYTES + CHECKSUM_BYTES {
        return Err("truncated L4 cross-scene package".to_string());
    }
    if bytes.len() > MAX_PACKAGE_BYTES {
        return Err("L4 cross-scene package exceeds size budget".to_string());
    }
    let checksum_offset = bytes.len() - CHECKSUM_BYTES;
    let expected_checksum = u64::from_le_bytes(
        bytes[checksum_offset..]
            .try_into()
            .map_err(|_| "invalid L4 cross-scene checksum".to_string())?,
    );
    if checksum64(&bytes[..checksum_offset]) != expected_checksum {
        return Err("L4 cross-scene checksum mismatch".to_string());
    }
    let shared: Arc<[u8]> = bytes.into();
    let mut cursor = Cursor::new(shared.clone(), checksum_offset);
    if cursor.take::<8>()?.as_slice() != MAGIC {
        return Err("invalid L4 cross-scene magic".to_string());
    }
    if cursor.u16()? != VERSION {
        return Err("unsupported L4 cross-scene version".to_string());
    }
    if cursor.u16()? as usize != CELLS {
        return Err("L4 cross-scene phase width mismatch".to_string());
    }
    if cursor.u32()? != ENCODER_VERSION || cursor.u64()? != ENCODER_HASH {
        return Err("L4 cross-scene encoder identity mismatch".to_string());
    }
    let profile_count = cursor.u32()? as usize;
    let pair_count = cursor.u32()? as usize;
    if profile_count > MAX_PROFILES || pair_count > MAX_PAIR_PROFILES {
        return Err("L4 cross-scene profile budget exceeded".to_string());
    }
    let mut package = L4CrossScenePackage {
        profiles: Vec::with_capacity(profile_count),
        pair_profiles: Vec::with_capacity(pair_count),
        source_observations: cursor.u32()?,
        joined_observations: cursor.u32()?,
        positive_observations: cursor.u32()?,
        negative_observations: cursor.u32()?,
        reverted_observations: cursor.u32()?,
        ambiguity_observations: cursor.u32()?,
        censored_observations: cursor.u32()?,
    };
    for _ in 0..profile_count {
        package.profiles.push(read_profile(&mut cursor)?);
    }
    for _ in 0..pair_count {
        package.pair_profiles.push(read_pair(&mut cursor)?);
    }
    if cursor.offset != checksum_offset {
        return Err("trailing or truncated L4 cross-scene payload".to_string());
    }
    validate_order(&package)?;
    Ok(package)
}

fn write_profile(bytes: &mut Vec<u8>, profile: &L4CrossSceneProfile) {
    write_key(bytes, profile.key);
    put_i32(bytes, profile.threshold_micro);
    put_u32(bytes, profile.positive_examples);
    put_u32(bytes, profile.negative_examples);
    put_u32(bytes, profile.reverted_examples);
    put_u32(bytes, profile.ambiguity_examples);
    put_u32(bytes, profile.censored_examples);
    bytes.push(profile.positive.len() as u8);
    bytes.push(profile.negative.len() as u8);
    bytes.push(profile.hard_negative.len() as u8);
    bytes.push(profile.ambiguity.len() as u8);
    for center in profile
        .positive
        .iter()
        .chain(&profile.negative)
        .chain(&profile.hard_negative)
        .chain(&profile.ambiguity)
    {
        write_center(bytes, center);
    }
}

fn read_profile(cursor: &mut Cursor) -> Result<L4CrossSceneProfile, String> {
    let key = read_key(cursor)?;
    let threshold_micro = cursor.i32()?;
    let positive_examples = cursor.u32()?;
    let negative_examples = cursor.u32()?;
    let reverted_examples = cursor.u32()?;
    let ambiguity_examples = cursor.u32()?;
    let censored_examples = cursor.u32()?;
    let positive_count = cursor.u8()? as usize;
    let negative_count = cursor.u8()? as usize;
    let hard_count = cursor.u8()? as usize;
    let ambiguity_count = cursor.u8()? as usize;
    require_center_count(positive_count, MAX_CENTERS_PER_BANK)?;
    require_center_count(negative_count, MAX_CENTERS_PER_BANK)?;
    require_center_count(hard_count, MAX_HARD_CENTERS_PER_BANK)?;
    require_center_count(ambiguity_count, MAX_AMBIGUITY_CENTERS_PER_BANK)?;
    Ok(L4CrossSceneProfile {
        key,
        threshold_micro,
        positive: read_centers(cursor, positive_count)?,
        negative: read_centers(cursor, negative_count)?,
        hard_negative: read_centers(cursor, hard_count)?,
        ambiguity: read_centers(cursor, ambiguity_count)?,
        positive_examples,
        negative_examples,
        reverted_examples,
        ambiguity_examples,
        censored_examples,
    })
}

fn write_pair(bytes: &mut Vec<u8>, pair: &L4CrossScenePairProfile) {
    write_key(bytes, pair.key);
    put_u64(bytes, pair.low_relation);
    put_u64(bytes, pair.high_relation);
    put_i32(bytes, pair.threshold_micro);
    put_u32(bytes, pair.observations);
    bytes.push(pair.low_wins.len() as u8);
    bytes.push(pair.high_wins.len() as u8);
    bytes.push(pair.hard_low_wins.len() as u8);
    bytes.push(pair.hard_high_wins.len() as u8);
    bytes.push(pair.ambiguity.len() as u8);
    bytes.extend_from_slice(&[0; 3]);
    for center in pair
        .low_wins
        .iter()
        .chain(&pair.high_wins)
        .chain(&pair.hard_low_wins)
        .chain(&pair.hard_high_wins)
        .chain(&pair.ambiguity)
    {
        write_center(bytes, center);
    }
}

fn read_pair(cursor: &mut Cursor) -> Result<L4CrossScenePairProfile, String> {
    let key = read_key(cursor)?;
    let low_relation = cursor.u64()?;
    let high_relation = cursor.u64()?;
    if low_relation >= high_relation {
        return Err("L4 cross-scene pair relation order is invalid".to_string());
    }
    let threshold_micro = cursor.i32()?;
    let observations = cursor.u32()?;
    let low_count = cursor.u8()? as usize;
    let high_count = cursor.u8()? as usize;
    let hard_low_count = cursor.u8()? as usize;
    let hard_high_count = cursor.u8()? as usize;
    let ambiguity_count = cursor.u8()? as usize;
    cursor.skip(3)?;
    require_center_count(low_count, MAX_CENTERS_PER_BANK)?;
    require_center_count(high_count, MAX_CENTERS_PER_BANK)?;
    require_center_count(hard_low_count, MAX_HARD_CENTERS_PER_BANK)?;
    require_center_count(hard_high_count, MAX_HARD_CENTERS_PER_BANK)?;
    require_center_count(ambiguity_count, MAX_AMBIGUITY_CENTERS_PER_BANK)?;
    Ok(L4CrossScenePairProfile {
        key,
        low_relation,
        high_relation,
        threshold_micro,
        low_wins: read_centers(cursor, low_count)?,
        high_wins: read_centers(cursor, high_count)?,
        hard_low_wins: read_centers(cursor, hard_low_count)?,
        hard_high_wins: read_centers(cursor, hard_high_count)?,
        ambiguity: read_centers(cursor, ambiguity_count)?,
        observations,
    })
}

fn write_key(bytes: &mut Vec<u8>, key: L4CrossSceneProfileKey) {
    bytes.push(key.operator as u8);
    bytes.push(key.direction.map_or(0, LayoutProjectionDirection::code));
    bytes.push(key.scope.map_or(0, LayoutProjectionScope::code));
    bytes.push(0);
}

fn read_key(cursor: &mut Cursor) -> Result<L4CrossSceneProfileKey, String> {
    let operator = TransitionOperatorKind::from_code(cursor.u8()?)
        .ok_or_else(|| "unknown L4 cross-scene operator".to_string())?;
    let direction = match cursor.u8()? {
        0 => None,
        code => Some(
            LayoutProjectionDirection::from_code(code)
                .ok_or_else(|| "unknown L4 cross-scene direction".to_string())?,
        ),
    };
    let scope = match cursor.u8()? {
        0 => None,
        code => Some(
            LayoutProjectionScope::from_code(code)
                .ok_or_else(|| "unknown L4 cross-scene scope".to_string())?,
        ),
    };
    if cursor.u8()? != 0 {
        return Err("non-zero L4 cross-scene key padding".to_string());
    }
    if operator != TransitionOperatorKind::LayoutProjection
        && (direction.is_some() || scope.is_some())
    {
        return Err("non-layout L4 profile carries layout identity".to_string());
    }
    Ok(L4CrossSceneProfileKey::new(operator, direction, scope))
}

fn write_center(bytes: &mut Vec<u8>, center: &PhaseCenter) {
    put_u32(bytes, center.support);
    center.write_compact(bytes);
}

fn read_centers(cursor: &mut Cursor, count: usize) -> Result<Vec<PhaseCenter>, String> {
    let mut centers = Vec::with_capacity(count);
    for _ in 0..count {
        let support = cursor.u32()?;
        if support == 0 {
            return Err("zero-support L4 cross-scene center".to_string());
        }
        let offset = cursor.offset;
        cursor.skip(CELLS * 2)?;
        centers.push(PhaseCenter::from_serialized(
            cursor.bytes.clone(),
            offset,
            support,
        ));
    }
    Ok(centers)
}

fn require_center_count(count: usize, maximum: usize) -> Result<(), String> {
    if count > maximum {
        Err("L4 cross-scene center budget exceeded".to_string())
    } else {
        Ok(())
    }
}

fn validate_order(package: &L4CrossScenePackage) -> Result<(), String> {
    if package
        .profiles
        .windows(2)
        .any(|pair| pair[0].key >= pair[1].key)
    {
        return Err("L4 cross-scene profiles are not strictly ordered".to_string());
    }
    if package.pair_profiles.windows(2).any(|items| {
        (items[0].key, items[0].low_relation, items[0].high_relation)
            >= (items[1].key, items[1].low_relation, items[1].high_relation)
    }) {
        return Err("L4 cross-scene pair profiles are not strictly ordered".to_string());
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_else(|| "l4-cross-scene".into());
    name.push(format!(".{}.tmp", std::process::id()));
    path.with_file_name(name)
}

fn checksum64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |state, byte| {
        state.wrapping_mul(0x0000_0100_0000_01b3) ^ u64::from(*byte)
    })
}

fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Cursor {
    bytes: Arc<[u8]>,
    offset: usize,
    limit: usize,
}

impl Cursor {
    fn new(bytes: Arc<[u8]>, limit: usize) -> Self {
        Self {
            bytes,
            offset: 0,
            limit,
        }
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| "L4 cross-scene offset overflow".to_string())?;
        if end > self.limit {
            return Err("truncated L4 cross-scene section".to_string());
        }
        let value = self.bytes[self.offset..end]
            .try_into()
            .map_err(|_| "invalid L4 cross-scene field width".to_string())?;
        self.offset = end;
        Ok(value)
    }

    fn skip(&mut self, count: usize) -> Result<(), String> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| "L4 cross-scene offset overflow".to_string())?;
        if end > self.limit {
            return Err("truncated L4 cross-scene vector".to_string());
        }
        self.offset = end;
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take()?))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take()?))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_package_roundtrips_and_detects_corruption() {
        let package = L4CrossScenePackage::default();
        let bytes = encode_package(&package);
        let restored = decode_package(bytes.clone()).unwrap();
        assert_eq!(restored, package);

        let mut corrupt = bytes;
        corrupt[24] ^= 1;
        assert!(decode_package(corrupt).is_err());
    }

    #[test]
    fn compact_center_roundtrip_preserves_runtime_coherence() {
        let center = PhaseCenter::from_center(
            vec![crate::nanda_wave::phase_field::PhaseCell { re: 1.0, im: 0.0 }; CELLS],
            3,
        );
        let package = L4CrossScenePackage {
            profiles: vec![L4CrossSceneProfile {
                key: L4CrossSceneProfileKey::new(TransitionOperatorKind::Other, None, None),
                threshold_micro: 10,
                positive: vec![center],
                negative: Vec::new(),
                hard_negative: Vec::new(),
                ambiguity: Vec::new(),
                positive_examples: 3,
                negative_examples: 0,
                reverted_examples: 0,
                ambiguity_examples: 0,
                censored_examples: 0,
            }],
            ..L4CrossScenePackage::default()
        };
        let restored = decode_package(encode_package(&package)).unwrap();
        assert!(
            restored.profiles[0].positive[0].coherence(&vec![
                    crate::nanda_wave::phase_field::PhaseCell { re: 1.0, im: 0.0 };
                    CELLS
                ])
                > 0.99
        );
    }
}
