use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

const PROMOTION_MAGIC: [u8; 8] = *b"LAYP2PR1";
const PROMOTION_VERSION: u16 = 1;
const PROMOTION_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub(super) enum DeltaRecordKindV1 {
    LemmaBindingSupport = 1,
    ParadigmSupport = 2,
    ExplicitAnti = 3,
    AmbiguityObservation = 4,
    DirectionalResidual = 5,
    ModelCoefficients = 6,
    CalibrationGeneration = 7,
    ExactLemmaLocalAllomorph = 8,
    Supersedes = 9,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeltaRecordV1 {
    pub(super) kind: DeltaRecordKindV1,
    pub(super) event_identity: [u8; 32],
    pub(super) typed_key_hash: u64,
    pub(super) payload: Vec<u8>,
    pub(super) supersedes_record_identity: Option<[u8; 32]>,
}

impl DeltaRecordV1 {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(52 + self.payload.len());
        bytes.extend_from_slice(&(self.kind as u16).to_le_bytes());
        bytes.extend_from_slice(&self.typed_key_hash.to_le_bytes());
        bytes.extend_from_slice(&self.event_identity);
        match self.supersedes_record_identity {
            Some(identity) => {
                bytes.push(1);
                bytes.extend_from_slice(&identity);
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.event_identity == [0; 32] {
            return Err("productive delta record has a zero event identity");
        }
        if self.payload.len() > u32::MAX as usize {
            return Err("productive delta payload exceeds u32");
        }
        if matches!(self.kind, DeltaRecordKindV1::Supersedes)
            != self.supersedes_record_identity.is_some()
        {
            return Err("productive SUPERSEDES record identity contract is invalid");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeltaManifestV1 {
    pub(super) base_package_sha256: [u8; 32],
    pub(super) previous_generation_sha256: [u8; 32],
    pub(super) generation: u64,
    pub(super) event_start: u64,
    pub(super) event_end: u64,
    pub(super) coefficient_generation: u64,
    pub(super) calibration_generation: u64,
    pub(super) proof_receipt_sha256: [u8; 32],
    pub(super) requested_authority_scope: AuthorityScopeV1,
    pub(super) payload_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeltaGenerationV1 {
    pub(super) manifest: DeltaManifestV1,
    pub(super) records: Vec<DeltaRecordV1>,
    pub(super) generation_sha256: [u8; 32],
}

impl DeltaGenerationV1 {
    pub(super) fn build(
        mut manifest: DeltaManifestV1,
        mut records: Vec<DeltaRecordV1>,
    ) -> Result<Self, &'static str> {
        for record in &records {
            record.validate()?;
        }
        records.sort_by(|left, right| {
            left.typed_key_hash
                .cmp(&right.typed_key_hash)
                .then_with(|| left.event_identity.cmp(&right.event_identity))
                .then_with(|| (left.kind as u16).cmp(&(right.kind as u16)))
                .then_with(|| left.payload.cmp(&right.payload))
        });
        let payload = canonical_record_payload(&records);
        manifest.payload_sha256 = Sha256::digest(&payload).into();
        let generation_sha256 = generation_hash(&manifest, &payload);
        Ok(Self {
            manifest,
            records,
            generation_sha256,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct DeltaChainStateV1 {
    pub(super) generation: u64,
    pub(super) generation_sha256: [u8; 32],
    pub(super) coefficient_generation: u64,
    pub(super) calibration_generation: u64,
    pub(super) semantic_sha256: [u8; 32],
    active_records: BTreeMap<[u8; 32], DeltaRecordV1>,
    seen_events: BTreeMap<[u8; 32], Vec<u8>>,
}

pub(super) fn validate_delta_chain(
    base_package_sha256: [u8; 32],
    split_seed_unchanged: bool,
    generations: &[DeltaGenerationV1],
) -> Result<DeltaChainStateV1, &'static str> {
    if !split_seed_unchanged {
        return Err("productive delta changes the base split seed");
    }
    let mut state = DeltaChainStateV1::default();
    let mut previous_hash = [0_u8; 32];
    let mut previous_event_end = 0_u64;
    for generation in generations {
        let manifest = generation.manifest;
        if manifest.base_package_sha256 != base_package_sha256 {
            return Err("productive delta base package fingerprint mismatch");
        }
        if manifest.generation != state.generation + 1
            || manifest.previous_generation_sha256 != previous_hash
        {
            return Err("productive delta generation chain has a gap or wrong predecessor");
        }
        if manifest.event_start != previous_event_end || manifest.event_end < manifest.event_start {
            return Err("productive delta event range is not contiguous");
        }
        let payload = canonical_record_payload(&generation.records);
        if <[u8; 32]>::from(Sha256::digest(&payload)) != manifest.payload_sha256
            || generation_hash(&manifest, &payload) != generation.generation_sha256
        {
            return Err("productive delta payload or generation SHA-256 mismatch");
        }
        validate_atomic_model_generation(&state, generation)?;
        for record in &generation.records {
            record.validate()?;
            let bytes = record.canonical_bytes();
            if let Some(existing) = state.seen_events.get(&record.event_identity) {
                if existing != &bytes {
                    return Err("productive delta repeats an event identity with different bytes");
                }
                continue;
            }
            state.seen_events.insert(record.event_identity, bytes);
            if let Some(superseded) = record.supersedes_record_identity {
                if state.active_records.remove(&superseded).is_none() {
                    return Err("productive SUPERSEDES references no active prior record");
                }
            }
            state
                .active_records
                .insert(record.event_identity, record.clone());
        }
        state.generation = manifest.generation;
        state.generation_sha256 = generation.generation_sha256;
        state.coefficient_generation = manifest.coefficient_generation;
        state.calibration_generation = manifest.calibration_generation;
        previous_hash = generation.generation_sha256;
        previous_event_end = manifest.event_end;
    }
    state.semantic_sha256 = semantic_chain_hash(&state);
    Ok(state)
}

fn validate_atomic_model_generation(
    state: &DeltaChainStateV1,
    generation: &DeltaGenerationV1,
) -> Result<(), &'static str> {
    let manifest = generation.manifest;
    let coefficient_changed = manifest.coefficient_generation != state.coefficient_generation;
    let calibration_changed = manifest.calibration_generation != state.calibration_generation;
    if coefficient_changed != calibration_changed
        || manifest.coefficient_generation != manifest.calibration_generation
    {
        return Err("productive coefficient and calibration generations are not atomic");
    }
    if coefficient_changed {
        if manifest.coefficient_generation <= state.coefficient_generation
            || manifest.proof_receipt_sha256 == [0; 32]
        {
            return Err("productive model generation lacks monotonic proof ownership");
        }
        let kinds = generation
            .records
            .iter()
            .map(|record| record.kind)
            .collect::<BTreeSet<_>>();
        if !kinds.contains(&DeltaRecordKindV1::ModelCoefficients)
            || !kinds.contains(&DeltaRecordKindV1::CalibrationGeneration)
        {
            return Err("productive model generation lacks coefficients or calibration records");
        }
    }
    Ok(())
}

pub(super) fn verify_compaction_semantic_parity(
    chain: &DeltaChainStateV1,
    compacted_records: &[DeltaRecordV1],
) -> Result<(), &'static str> {
    let mut compacted = compacted_records.to_vec();
    compacted.sort_by(|left, right| {
        left.typed_key_hash
            .cmp(&right.typed_key_hash)
            .then_with(|| left.event_identity.cmp(&right.event_identity))
    });
    let mut hasher = Sha256::new();
    hasher.update(chain.coefficient_generation.to_le_bytes());
    hasher.update(chain.calibration_generation.to_le_bytes());
    for record in compacted {
        record.validate()?;
        hasher.update(record.canonical_bytes());
    }
    if <[u8; 32]>::from(hasher.finalize()) != chain.semantic_sha256 {
        return Err("productive compacted delta semantic hash differs from the live chain");
    }
    Ok(())
}

fn semantic_chain_hash(state: &DeltaChainStateV1) -> [u8; 32] {
    let mut records = state.active_records.values().collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.typed_key_hash
            .cmp(&right.typed_key_hash)
            .then_with(|| left.event_identity.cmp(&right.event_identity))
    });
    let mut hasher = Sha256::new();
    hasher.update(state.coefficient_generation.to_le_bytes());
    hasher.update(state.calibration_generation.to_le_bytes());
    for record in records {
        hasher.update(record.canonical_bytes());
    }
    hasher.finalize().into()
}

fn canonical_record_payload(records: &[DeltaRecordV1]) -> Vec<u8> {
    let mut payload = Vec::new();
    for record in records {
        let bytes = record.canonical_bytes();
        payload.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(&bytes);
    }
    payload
}

fn generation_hash(manifest: &DeltaManifestV1, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(manifest.base_package_sha256);
    hasher.update(manifest.previous_generation_sha256);
    hasher.update(manifest.generation.to_le_bytes());
    hasher.update(manifest.event_start.to_le_bytes());
    hasher.update(manifest.event_end.to_le_bytes());
    hasher.update(manifest.coefficient_generation.to_le_bytes());
    hasher.update(manifest.calibration_generation.to_le_bytes());
    hasher.update(manifest.proof_receipt_sha256);
    hasher.update((manifest.requested_authority_scope as u32).to_le_bytes());
    hasher.update(manifest.payload_sha256);
    hasher.update(payload);
    hasher.finalize().into()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub(super) enum AuthorityScopeV1 {
    #[default]
    Invalid = 0,
    Shadow = 1,
    SuggestOnly = 2,
    ApplyAllowed = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PromotionManifestV1 {
    pub(super) authority_scope: AuthorityScopeV1,
    pub(super) base_package_sha256: [u8; 32],
    pub(super) semantic_chain_sha256: [u8; 32],
    pub(super) installed_binary_sha256: [u8; 32],
    pub(super) l11_package_sha256: [u8; 32],
    pub(super) canonical_l2_package_sha256: [u8; 32],
    pub(super) offline_receipt_bundle_sha256: [u8; 32],
    pub(super) physical_product_matrix_sha256: [u8; 32],
    pub(super) model_generation: u64,
    pub(super) flags: u32,
}

impl PromotionManifestV1 {
    pub(super) fn encode(self) -> Result<[u8; PROMOTION_BYTES], &'static str> {
        if self.flags != 0 || self.authority_scope == AuthorityScopeV1::Invalid {
            return Err("productive promotion manifest has invalid flags or scope");
        }
        let mut bytes = [0_u8; PROMOTION_BYTES];
        bytes[0..8].copy_from_slice(&PROMOTION_MAGIC);
        bytes[8..10].copy_from_slice(&PROMOTION_VERSION.to_le_bytes());
        bytes[10..12].copy_from_slice(&(PROMOTION_BYTES as u16).to_le_bytes());
        bytes[12..16].copy_from_slice(&(self.authority_scope as u32).to_le_bytes());
        bytes[16..48].copy_from_slice(&self.base_package_sha256);
        bytes[48..80].copy_from_slice(&self.semantic_chain_sha256);
        bytes[80..112].copy_from_slice(&self.installed_binary_sha256);
        bytes[112..144].copy_from_slice(&self.l11_package_sha256);
        bytes[144..176].copy_from_slice(&self.canonical_l2_package_sha256);
        bytes[176..208].copy_from_slice(&self.offline_receipt_bundle_sha256);
        bytes[208..240].copy_from_slice(&self.physical_product_matrix_sha256);
        bytes[240..248].copy_from_slice(&self.model_generation.to_le_bytes());
        bytes[248..252].copy_from_slice(&self.flags.to_le_bytes());
        let crc = crc32(&bytes);
        bytes[252..256].copy_from_slice(&crc.to_le_bytes());
        Ok(bytes)
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != PROMOTION_BYTES
            || bytes[0..8] != PROMOTION_MAGIC
            || u16::from_le_bytes(bytes[8..10].try_into().expect("fixed slice"))
                != PROMOTION_VERSION
            || u16::from_le_bytes(bytes[10..12].try_into().expect("fixed slice")) as usize
                != PROMOTION_BYTES
        {
            return Err("productive promotion manifest magic, version, or width mismatch");
        }
        let expected_crc = u32::from_le_bytes(bytes[252..256].try_into().expect("fixed slice"));
        let mut crc_bytes = bytes.to_vec();
        crc_bytes[252..256].fill(0);
        if crc32(&crc_bytes) != expected_crc {
            return Err("productive promotion manifest CRC mismatch");
        }
        let authority_scope =
            match u32::from_le_bytes(bytes[12..16].try_into().expect("fixed slice")) {
                1 => AuthorityScopeV1::Shadow,
                2 => AuthorityScopeV1::SuggestOnly,
                3 => AuthorityScopeV1::ApplyAllowed,
                _ => return Err("productive promotion manifest scope is invalid"),
            };
        let manifest = Self {
            authority_scope,
            base_package_sha256: bytes[16..48].try_into().expect("fixed slice"),
            semantic_chain_sha256: bytes[48..80].try_into().expect("fixed slice"),
            installed_binary_sha256: bytes[80..112].try_into().expect("fixed slice"),
            l11_package_sha256: bytes[112..144].try_into().expect("fixed slice"),
            canonical_l2_package_sha256: bytes[144..176].try_into().expect("fixed slice"),
            offline_receipt_bundle_sha256: bytes[176..208].try_into().expect("fixed slice"),
            physical_product_matrix_sha256: bytes[208..240].try_into().expect("fixed slice"),
            model_generation: u64::from_le_bytes(bytes[240..248].try_into().expect("fixed slice")),
            flags: u32::from_le_bytes(bytes[248..252].try_into().expect("fixed slice")),
        };
        if manifest.flags != 0 {
            return Err("productive promotion manifest has unknown flags");
        }
        Ok(manifest)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PromotionExpectationV1 {
    pub(super) base_package_sha256: [u8; 32],
    pub(super) semantic_chain_sha256: [u8; 32],
    pub(super) installed_binary_sha256: [u8; 32],
    pub(super) l11_package_sha256: [u8; 32],
    pub(super) canonical_l2_package_sha256: [u8; 32],
    pub(super) model_generation: u64,
}

pub(super) fn resolve_authority_scope(
    manifest_bytes: Option<&[u8]>,
    expected: PromotionExpectationV1,
) -> AuthorityScopeV1 {
    let Ok(manifest) = manifest_bytes
        .ok_or("missing")
        .and_then(PromotionManifestV1::decode)
    else {
        return AuthorityScopeV1::Shadow;
    };
    if manifest.base_package_sha256 != expected.base_package_sha256
        || manifest.semantic_chain_sha256 != expected.semantic_chain_sha256
        || manifest.installed_binary_sha256 != expected.installed_binary_sha256
        || manifest.l11_package_sha256 != expected.l11_package_sha256
        || manifest.canonical_l2_package_sha256 != expected.canonical_l2_package_sha256
        || manifest.model_generation != expected.model_generation
    {
        return AuthorityScopeV1::Shadow;
    }
    if manifest.authority_scope == AuthorityScopeV1::ApplyAllowed
        && (manifest.offline_receipt_bundle_sha256 == [0; 32]
            || manifest.physical_product_matrix_sha256 == [0; 32])
    {
        return AuthorityScopeV1::Shadow;
    }
    manifest.authority_scope
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(index: u8, kind: DeltaRecordKindV1) -> DeltaRecordV1 {
        let mut identity = [0_u8; 32];
        identity[0] = index;
        DeltaRecordV1 {
            kind,
            event_identity: identity,
            typed_key_hash: u64::from(index),
            payload: vec![index],
            supersedes_record_identity: None,
        }
    }

    fn manifest(generation: u64, previous: [u8; 32]) -> DeltaManifestV1 {
        DeltaManifestV1 {
            base_package_sha256: [1; 32],
            previous_generation_sha256: previous,
            generation,
            event_start: generation - 1,
            event_end: generation,
            coefficient_generation: 0,
            calibration_generation: 0,
            proof_receipt_sha256: [0; 32],
            requested_authority_scope: AuthorityScopeV1::Shadow,
            payload_sha256: [0; 32],
        }
    }

    #[test]
    fn delta_chain_rejects_gap_and_conflicting_duplicate_event() {
        let first = DeltaGenerationV1::build(
            manifest(1, [0; 32]),
            vec![record(1, DeltaRecordKindV1::LemmaBindingSupport)],
        )
        .expect("first");
        let mut gap_manifest = manifest(3, first.generation_sha256);
        gap_manifest.event_start = 1;
        gap_manifest.event_end = 2;
        let gap = DeltaGenerationV1::build(
            gap_manifest,
            vec![record(2, DeltaRecordKindV1::ParadigmSupport)],
        )
        .expect("gap package");
        assert!(validate_delta_chain([1; 32], true, &[first.clone(), gap]).is_err());

        let mut conflicting = record(1, DeltaRecordKindV1::LemmaBindingSupport);
        conflicting.payload = vec![9];
        let second =
            DeltaGenerationV1::build(manifest(2, first.generation_sha256), vec![conflicting])
                .expect("second");
        assert!(validate_delta_chain([1; 32], true, &[first, second]).is_err());
    }

    #[test]
    fn coefficient_and_calibration_generations_are_atomic() {
        let mut model_manifest = manifest(1, [0; 32]);
        model_manifest.coefficient_generation = 1;
        model_manifest.calibration_generation = 0;
        let generation = DeltaGenerationV1::build(
            model_manifest,
            vec![record(1, DeltaRecordKindV1::ModelCoefficients)],
        )
        .expect("generation");
        assert!(validate_delta_chain([1; 32], true, &[generation]).is_err());
    }

    #[test]
    fn promotion_manifest_is_256_bytes_and_fails_closed() {
        let manifest = PromotionManifestV1 {
            authority_scope: AuthorityScopeV1::ApplyAllowed,
            base_package_sha256: [1; 32],
            semantic_chain_sha256: [2; 32],
            installed_binary_sha256: [3; 32],
            l11_package_sha256: [4; 32],
            canonical_l2_package_sha256: [5; 32],
            offline_receipt_bundle_sha256: [6; 32],
            physical_product_matrix_sha256: [7; 32],
            model_generation: 9,
            flags: 0,
        };
        let bytes = manifest.encode().expect("encode");
        assert_eq!(bytes.len(), 256);
        assert_eq!(
            PromotionManifestV1::decode(&bytes).expect("decode"),
            manifest
        );
        let expected = PromotionExpectationV1 {
            base_package_sha256: [1; 32],
            semantic_chain_sha256: [2; 32],
            installed_binary_sha256: [3; 32],
            l11_package_sha256: [4; 32],
            canonical_l2_package_sha256: [5; 32],
            model_generation: 9,
        };
        assert_eq!(
            resolve_authority_scope(Some(&bytes), expected),
            AuthorityScopeV1::ApplyAllowed
        );
        let mut stale = expected;
        stale.model_generation = 8;
        assert_eq!(
            resolve_authority_scope(Some(&bytes), stale),
            AuthorityScopeV1::Shadow
        );
        assert_eq!(
            resolve_authority_scope(None, expected),
            AuthorityScopeV1::Shadow
        );
    }
}
