use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::model::{
    L4CrossScenePackage, L4CrossScenePairProfile, L4CrossSceneProfile, L4CrossSceneProfileKey,
};
use super::{
    CELLS, ENCODER_HASH, ENCODER_VERSION, MAX_AMBIGUITY_CENTERS_PER_BANK, MAX_CENTERS_PER_BANK,
    MAX_HARD_CENTERS_PER_BANK, MAX_PAIR_PROFILES, MAX_PROFILES, MAX_SYMBOLS, V1_ENCODER_HASH,
    V1_ENCODER_VERSION,
};
use crate::nanda_wave::phase_field::PhaseCenter;
use crate::transition_relation::TransitionOperatorKind;
use crate::typing_memory::{LayoutProjectionDirection, LayoutProjectionScope};
use crate::typing_scene::{
    KeyboardGeometryId, LanguageId, LanguageSceneIdentity, LayoutId, SceneIdentityEvidence,
    SceneSymbol, SceneSymbolKind, ScriptFamily,
};

const MAGIC: &[u8; 8] = b"LAYL4CS\0";
const VERSION_V1: u16 = 1;
const VERSION_V2: u16 = 2;
const HEADER_V1_BYTES: usize = 60;
const HEADER_V2_BYTES: usize = 76;
const KEY_V1_BYTES: usize = 4;
const KEY_V2_BYTES: usize = 56;
const PROFILE_PAYLOAD_BYTES: usize = 28;
const PAIR_PAYLOAD_BYTES: usize = 32;
const CENTER_BYTES: usize = 4 + CELLS * 2;
const SYMBOL_HEADER_BYTES: usize = 12;
const CHECKSUM_BYTES: usize = 8;
const MAX_PACKAGE_BYTES: usize = 16 * 1024 * 1024;
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write_package(path: &Path, package: &L4CrossScenePackage) -> io::Result<()> {
    let bytes = encode_package_checked(package).map_err(invalid_data)?;
    decode_package(bytes.clone()).map_err(invalid_data)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = temporary_path(path);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let mut file = options.open(&temporary)?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        let candidate = fs::read(&temporary)?;
        decode_package(candidate).map_err(invalid_data)?;
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        if let Some(parent) = path.parent() {
            fs::File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn read_package(path: &Path) -> io::Result<L4CrossScenePackage> {
    let bytes = fs::read(path)?;
    decode_package(bytes).map_err(invalid_data)
}

pub(crate) fn encode_package(package: &L4CrossScenePackage) -> Vec<u8> {
    encode_package_checked(package).expect("validated L4 cross-scene package")
}

fn encode_package_checked(package: &L4CrossScenePackage) -> Result<Vec<u8>, String> {
    validate_package(package)?;
    let version = format_version(package)?;
    let center_count = center_count(package);
    let symbol_bytes = if version == VERSION_V2 {
        package
            .symbols
            .iter()
            .map(|symbol| SYMBOL_HEADER_BYTES + symbol.label.len())
            .sum()
    } else {
        0
    };
    let key_bytes = if version == VERSION_V1 {
        KEY_V1_BYTES
    } else {
        KEY_V2_BYTES
    };
    let header_bytes = if version == VERSION_V1 {
        HEADER_V1_BYTES
    } else {
        HEADER_V2_BYTES
    };
    let mut bytes = Vec::with_capacity(
        header_bytes
            + symbol_bytes
            + package.profiles.len() * (key_bytes + PROFILE_PAYLOAD_BYTES)
            + package.pair_profiles.len() * (key_bytes + PAIR_PAYLOAD_BYTES)
            + center_count * CENTER_BYTES
            + CHECKSUM_BYTES,
    );
    write_header(&mut bytes, package, version);
    debug_assert_eq!(bytes.len(), header_bytes);
    if version == VERSION_V2 {
        for symbol in &package.symbols {
            write_symbol(&mut bytes, symbol);
        }
    }
    for profile in &package.profiles {
        write_profile(&mut bytes, profile, version);
    }
    for pair in &package.pair_profiles {
        write_pair(&mut bytes, pair, version);
    }
    if bytes.len() + CHECKSUM_BYTES > MAX_PACKAGE_BYTES {
        return Err("L4 cross-scene package exceeds size budget".to_string());
    }
    let checksum = checksum64(&bytes);
    put_u64(&mut bytes, checksum);
    Ok(bytes)
}

/// Returns the exact compact representation used by runtime readout.
pub(crate) fn canonical_runtime_package(
    package: &L4CrossScenePackage,
) -> Result<L4CrossScenePackage, String> {
    decode_package(encode_package_checked(package)?)
}

fn write_header(bytes: &mut Vec<u8>, package: &L4CrossScenePackage, version: u16) {
    bytes.extend_from_slice(MAGIC);
    put_u16(bytes, version);
    put_u16(bytes, CELLS as u16);
    put_u32(bytes, package.encoder_version);
    put_u64(bytes, package.encoder_hash);
    put_u32(bytes, package.profiles.len() as u32);
    put_u32(bytes, package.pair_profiles.len() as u32);
    if version == VERSION_V2 {
        put_u32(bytes, package.symbols.len() as u32);
        put_u32(bytes, 0);
        put_u64(bytes, package.applied_segment);
    }
    put_u32(bytes, package.source_observations);
    put_u32(bytes, package.joined_observations);
    put_u32(bytes, package.positive_observations);
    put_u32(bytes, package.negative_observations);
    put_u32(bytes, package.reverted_observations);
    put_u32(bytes, package.ambiguity_observations);
    put_u32(bytes, package.censored_observations);
}

fn decode_package(bytes: Vec<u8>) -> Result<L4CrossScenePackage, String> {
    if bytes.len() < HEADER_V1_BYTES + CHECKSUM_BYTES {
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
    let mut cursor = Cursor::new(shared, checksum_offset);
    if cursor.take::<8>()?.as_slice() != MAGIC {
        return Err("invalid L4 cross-scene magic".to_string());
    }
    let version = cursor.u16()?;
    if !matches!(version, VERSION_V1 | VERSION_V2) {
        return Err("unsupported L4 cross-scene version".to_string());
    }
    if cursor.u16()? as usize != CELLS {
        return Err("L4 cross-scene phase width mismatch".to_string());
    }
    let encoder_version = cursor.u32()?;
    let encoder_hash = cursor.u64()?;
    let expected_encoder = match version {
        VERSION_V1 => (V1_ENCODER_VERSION, V1_ENCODER_HASH),
        VERSION_V2 => (ENCODER_VERSION, ENCODER_HASH),
        _ => unreachable!(),
    };
    if (encoder_version, encoder_hash) != expected_encoder {
        return Err("L4 cross-scene encoder identity mismatch".to_string());
    }
    let profile_count = cursor.u32()? as usize;
    let pair_count = cursor.u32()? as usize;
    if profile_count > MAX_PROFILES || pair_count > MAX_PAIR_PROFILES {
        return Err("L4 cross-scene profile budget exceeded".to_string());
    }
    let (symbol_count, applied_segment) = if version == VERSION_V2 {
        let symbol_count = cursor.u32()? as usize;
        if symbol_count > MAX_SYMBOLS {
            return Err("L4 cross-scene symbol budget exceeded".to_string());
        }
        if cursor.u32()? != 0 {
            return Err("non-zero L4 cross-scene header padding".to_string());
        }
        (symbol_count, cursor.u64()?)
    } else {
        (0, 0)
    };
    let mut package = L4CrossScenePackage {
        encoder_version,
        encoder_hash,
        applied_segment,
        symbols: Vec::with_capacity(symbol_count),
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
    let expected_header = if version == VERSION_V1 {
        HEADER_V1_BYTES
    } else {
        HEADER_V2_BYTES
    };
    if cursor.offset != expected_header {
        return Err("invalid L4 cross-scene header width".to_string());
    }
    for _ in 0..symbol_count {
        package.symbols.push(read_symbol(&mut cursor)?);
    }
    for _ in 0..profile_count {
        package.profiles.push(read_profile(&mut cursor, version)?);
    }
    for _ in 0..pair_count {
        package.pair_profiles.push(read_pair(&mut cursor, version)?);
    }
    if cursor.offset != checksum_offset {
        return Err("trailing or truncated L4 cross-scene payload".to_string());
    }
    validate_package(&package)?;
    Ok(package)
}

fn write_symbol(bytes: &mut Vec<u8>, symbol: &SceneSymbol) {
    bytes.push(symbol.kind as u8);
    bytes.push(symbol.label.len() as u8);
    put_u16(bytes, 0);
    put_u64(bytes, symbol.id);
    bytes.extend_from_slice(symbol.label.as_bytes());
}

fn read_symbol(cursor: &mut Cursor) -> Result<SceneSymbol, String> {
    let kind = SceneSymbolKind::from_code(cursor.u8()?)
        .ok_or_else(|| "unknown L4 cross-scene symbol kind".to_string())?;
    let label_len = cursor.u8()? as usize;
    if label_len == 0 || cursor.u16()? != 0 {
        return Err("invalid L4 cross-scene symbol header".to_string());
    }
    let id = cursor.u64()?;
    let label = std::str::from_utf8(cursor.slice(label_len)?)
        .map_err(|_| "non-UTF8 L4 cross-scene symbol".to_string())?
        .to_string();
    Ok(SceneSymbol { kind, id, label })
}

fn write_profile(bytes: &mut Vec<u8>, profile: &L4CrossSceneProfile, version: u16) {
    write_key(bytes, profile.key, version);
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

fn read_profile(cursor: &mut Cursor, version: u16) -> Result<L4CrossSceneProfile, String> {
    let key = read_key(cursor, version)?;
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

fn write_pair(bytes: &mut Vec<u8>, pair: &L4CrossScenePairProfile, version: u16) {
    write_key(bytes, pair.key, version);
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

fn read_pair(cursor: &mut Cursor, version: u16) -> Result<L4CrossScenePairProfile, String> {
    let key = read_key(cursor, version)?;
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
    if cursor.take::<3>()? != [0; 3] {
        return Err("non-zero L4 cross-scene pair padding".to_string());
    }
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

fn write_key(bytes: &mut Vec<u8>, key: L4CrossSceneProfileKey, version: u16) {
    bytes.push(key.operator as u8);
    bytes.push(key.direction.map_or(0, LayoutProjectionDirection::code));
    bytes.push(key.scope.map_or(0, LayoutProjectionScope::code));
    if version == VERSION_V1 {
        bytes.push(0);
        return;
    }
    bytes.push(key.scene.evidence.code());
    bytes.push(key.scene.source_script.code());
    bytes.push(key.scene.target_script.code());
    bytes.push(key.sentence_evidence_bucket);
    bytes.push(0);
    put_u64(bytes, key.scene.source_language.code());
    put_u64(bytes, key.scene.target_language.code());
    put_u64(bytes, key.scene.source_layout.code());
    put_u64(bytes, key.scene.target_layout.code());
    put_u64(bytes, key.scene.keyboard_geometry.code());
    put_u64(bytes, key.sentence_language.code());
}

fn read_key(cursor: &mut Cursor, version: u16) -> Result<L4CrossSceneProfileKey, String> {
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
    if version == VERSION_V1 {
        if cursor.u8()? != 0 {
            return Err("non-zero L4 cross-scene key padding".to_string());
        }
        return Ok(L4CrossSceneProfileKey::new(operator, direction, scope));
    }
    let evidence = SceneIdentityEvidence::from_code(cursor.u8()?)
        .ok_or_else(|| "unknown L4 cross-scene identity evidence".to_string())?;
    let source_script = ScriptFamily::from_code(cursor.u8()?)
        .ok_or_else(|| "unknown L4 cross-scene source script".to_string())?;
    let target_script = ScriptFamily::from_code(cursor.u8()?)
        .ok_or_else(|| "unknown L4 cross-scene target script".to_string())?;
    let sentence_evidence_bucket = cursor.u8()?;
    if cursor.u8()? != 0 {
        return Err("non-zero L4 cross-scene V2 key padding".to_string());
    }
    Ok(L4CrossSceneProfileKey {
        operator,
        direction,
        scope,
        scene: LanguageSceneIdentity {
            source_language: LanguageId::from_code(cursor.u64()?),
            target_language: LanguageId::from_code(cursor.u64()?),
            source_layout: LayoutId::from_code(cursor.u64()?),
            target_layout: LayoutId::from_code(cursor.u64()?),
            source_script,
            target_script,
            keyboard_geometry: KeyboardGeometryId::from_code(cursor.u64()?),
            evidence,
        },
        sentence_language: LanguageId::from_code(cursor.u64()?),
        sentence_evidence_bucket,
    })
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

fn validate_package(package: &L4CrossScenePackage) -> Result<(), String> {
    let version = format_version(package)?;
    if package.profiles.len() > MAX_PROFILES || package.pair_profiles.len() > MAX_PAIR_PROFILES {
        return Err("L4 cross-scene profile budget exceeded".to_string());
    }
    validate_order(package)?;
    if version == VERSION_V1 {
        if package.applied_segment != 0 || !package.symbols.is_empty() {
            return Err("V1 L4 cross-scene package carries V2 state".to_string());
        }
        if package
            .profiles
            .iter()
            .map(|profile| profile.key)
            .chain(package.pair_profiles.iter().map(|pair| pair.key))
            .any(|key| key != key.legacy_v1())
        {
            return Err("V1 L4 cross-scene package carries a V2 key".to_string());
        }
    } else {
        validate_symbol_registry(&package.symbols)?;
        for key in package
            .profiles
            .iter()
            .map(|profile| profile.key)
            .chain(package.pair_profiles.iter().map(|pair| pair.key))
        {
            validate_key_symbols(key, &package.symbols)?;
        }
    }
    for profile in &package.profiles {
        require_center_count(profile.positive.len(), MAX_CENTERS_PER_BANK)?;
        require_center_count(profile.negative.len(), MAX_CENTERS_PER_BANK)?;
        require_center_count(profile.hard_negative.len(), MAX_HARD_CENTERS_PER_BANK)?;
        require_center_count(profile.ambiguity.len(), MAX_AMBIGUITY_CENTERS_PER_BANK)?;
        validate_key(profile.key)?;
    }
    for pair in &package.pair_profiles {
        if pair.low_relation >= pair.high_relation {
            return Err("L4 cross-scene pair relation order is invalid".to_string());
        }
        require_center_count(pair.low_wins.len(), MAX_CENTERS_PER_BANK)?;
        require_center_count(pair.high_wins.len(), MAX_CENTERS_PER_BANK)?;
        require_center_count(pair.hard_low_wins.len(), MAX_HARD_CENTERS_PER_BANK)?;
        require_center_count(pair.hard_high_wins.len(), MAX_HARD_CENTERS_PER_BANK)?;
        require_center_count(pair.ambiguity.len(), MAX_AMBIGUITY_CENTERS_PER_BANK)?;
        validate_key(pair.key)?;
    }
    Ok(())
}

fn validate_key(key: L4CrossSceneProfileKey) -> Result<(), String> {
    if key.operator != TransitionOperatorKind::LayoutProjection
        && (key.direction.is_some() || key.scope.is_some())
    {
        return Err("non-layout L4 profile carries layout identity".to_string());
    }
    if key.sentence_evidence_bucket > 5 {
        return Err("unknown L4 sentence evidence bucket".to_string());
    }
    Ok(())
}

fn validate_symbol_registry(symbols: &[SceneSymbol]) -> Result<(), String> {
    if symbols.len() > MAX_SYMBOLS {
        return Err("L4 cross-scene symbol budget exceeded".to_string());
    }
    if symbols.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("L4 cross-scene symbols are not strictly ordered".to_string());
    }
    let mut ids = BTreeMap::new();
    let mut labels = BTreeSet::new();
    for symbol in symbols {
        if symbol.id == 0 || !symbol.validate() {
            return Err("invalid L4 cross-scene symbol identity".to_string());
        }
        if let Some(previous) = ids.insert((symbol.kind, symbol.id), symbol.label.as_str()) {
            if previous != symbol.label {
                return Err("L4 cross-scene symbol identity collision".to_string());
            }
        }
        if !labels.insert((symbol.kind, symbol.label.as_str())) {
            return Err("duplicate L4 cross-scene symbol label".to_string());
        }
    }
    Ok(())
}

fn validate_key_symbols(
    key: L4CrossSceneProfileKey,
    symbols: &[SceneSymbol],
) -> Result<(), String> {
    for (kind, id) in [
        (SceneSymbolKind::Language, key.scene.source_language.code()),
        (SceneSymbolKind::Language, key.scene.target_language.code()),
        (SceneSymbolKind::Layout, key.scene.source_layout.code()),
        (SceneSymbolKind::Layout, key.scene.target_layout.code()),
        (
            SceneSymbolKind::KeyboardGeometry,
            key.scene.keyboard_geometry.code(),
        ),
        (SceneSymbolKind::Language, key.sentence_language.code()),
    ] {
        if id != 0
            && !symbols
                .iter()
                .any(|symbol| symbol.kind == kind && symbol.id == id)
        {
            return Err("L4 cross-scene key references an absent symbol".to_string());
        }
    }
    Ok(())
}

fn format_version(package: &L4CrossScenePackage) -> Result<u16, String> {
    match (package.encoder_version, package.encoder_hash) {
        (V1_ENCODER_VERSION, V1_ENCODER_HASH) => Ok(VERSION_V1),
        (ENCODER_VERSION, ENCODER_HASH) => Ok(VERSION_V2),
        _ => Err("L4 cross-scene encoder identity mismatch".to_string()),
    }
}

fn center_count(package: &L4CrossScenePackage) -> usize {
    package
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
        .sum()
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
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("l4-cross-scene");
    path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), sequence))
}

fn invalid_data(error: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
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
        let slice = self.slice(N)?;
        slice
            .try_into()
            .map_err(|_| "invalid L4 cross-scene field width".to_string())
    }

    fn slice(&mut self, count: usize) -> Result<&[u8], String> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| "L4 cross-scene offset overflow".to_string())?;
        if end > self.limit {
            return Err("truncated L4 cross-scene section".to_string());
        }
        let start = self.offset;
        self.offset = end;
        Ok(&self.bytes[start..end])
    }

    fn skip(&mut self, count: usize) -> Result<(), String> {
        self.slice(count).map(|_| ())
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
    fn empty_v2_package_roundtrips_and_detects_corruption() {
        let package = L4CrossScenePackage::default();
        let bytes = encode_package(&package);
        let restored = decode_package(bytes.clone()).unwrap();
        assert_eq!(restored, package);
        assert_eq!(
            u16::from_le_bytes(bytes[8..10].try_into().unwrap()),
            VERSION_V2
        );

        let mut corrupt = bytes;
        corrupt[24] ^= 1;
        assert!(decode_package(corrupt).is_err());
    }

    #[test]
    fn v1_package_remains_byte_stable_and_erases_v2_identity() {
        let package = L4CrossScenePackage {
            encoder_version: V1_ENCODER_VERSION,
            encoder_hash: V1_ENCODER_HASH,
            ..L4CrossScenePackage::default()
        };
        let first = encode_package(&package);
        let restored = decode_package(first.clone()).unwrap();
        assert_eq!(restored.encoder_version, V1_ENCODER_VERSION);
        assert_eq!(restored.encoder_hash, V1_ENCODER_HASH);
        assert_eq!(first, encode_package(&restored));
        assert_eq!(
            u16::from_le_bytes(first[8..10].try_into().unwrap()),
            VERSION_V1
        );
    }

    #[test]
    fn v2_registry_and_typed_key_roundtrip() {
        let sentence = crate::typing_scene::SentenceLanguageEvidence {
            language: LanguageId::RUSSIAN,
            support_milli: 900,
            alternative_milli: 100,
            observed_tokens: 3,
        };
        let scene = LanguageSceneIdentity::observed("ghbdtn", "привет").with_legacy_ru_en_layout(
            LayoutId::XKB_US,
            LayoutId::XKB_RU,
            LanguageId::RUSSIAN,
        );
        let key = L4CrossSceneProfileKey::new(
            TransitionOperatorKind::LayoutProjection,
            Some(LayoutProjectionDirection::EnToRu),
            Some(LayoutProjectionScope::CurrentToken),
        )
        .with_scene(scene, sentence);
        let mut symbols = scene.known_symbols();
        symbols.push(SceneSymbol::language("ru").unwrap());
        symbols.sort();
        symbols.dedup();
        let package = L4CrossScenePackage {
            symbols,
            profiles: vec![L4CrossSceneProfile {
                key,
                threshold_micro: 10,
                positive: Vec::new(),
                negative: Vec::new(),
                hard_negative: Vec::new(),
                ambiguity: Vec::new(),
                positive_examples: 1,
                negative_examples: 0,
                reverted_examples: 0,
                ambiguity_examples: 0,
                censored_examples: 0,
            }],
            applied_segment: 7,
            ..L4CrossScenePackage::default()
        };
        let first = encode_package(&package);
        let restored = decode_package(first.clone()).unwrap();
        assert_eq!(restored.applied_segment, 7);
        assert_eq!(restored.profiles[0].key, key);
        assert_eq!(first, encode_package(&restored));
    }

    #[test]
    fn absent_symbol_reference_fails_closed() {
        let sentence = crate::typing_scene::SentenceLanguageEvidence {
            language: LanguageId::RUSSIAN,
            support_milli: 1_000,
            alternative_milli: 0,
            observed_tokens: 1,
        };
        let package = L4CrossScenePackage {
            profiles: vec![L4CrossSceneProfile {
                key: L4CrossSceneProfileKey::new(TransitionOperatorKind::Other, None, None)
                    .with_scene(LanguageSceneIdentity::default(), sentence),
                threshold_micro: 0,
                positive: Vec::new(),
                negative: Vec::new(),
                hard_negative: Vec::new(),
                ambiguity: Vec::new(),
                positive_examples: 0,
                negative_examples: 0,
                reverted_examples: 0,
                ambiguity_examples: 0,
                censored_examples: 0,
            }],
            ..L4CrossScenePackage::default()
        };
        assert!(encode_package_checked(&package).is_err());
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
            ]) > 0.99
        );
    }
}
