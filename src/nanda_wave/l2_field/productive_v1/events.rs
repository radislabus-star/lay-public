use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::scene::L2LocalSceneV1;
use super::types::{
    CanonicalL2BindingIdentityV1, ImportedCanonicalL2FormRefV1, MorphologySlotKeyV1,
    ProductiveCandidateIdentityV1,
};
use super::{PRODUCTIVE_V1_INNER_FOLDS, PRODUCTIVE_V1_SCHEMA_VERSION};

const SPOOL_MAGIC: [u8; 4] = *b"P2S1";
const SPOOL_RECORD_HEADER_BYTES: usize = 56;
const INNER_FOLD_LABEL: &[u8] = b"productive-v1-inner";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum ProductiveSplitV1 {
    Train = 0,
    Calibration = 1,
    HeldoutLemma = 2,
    Proof = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum ProductiveEventKindV1 {
    Morphology = 1,
    ContextOccurrence = 2,
    Feedback = 3,
    Proof = 4,
    ContextContradiction = 5,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct LemmaSplitKeyV1 {
    pub(super) language: String,
    pub(super) normalized_lemma: String,
}

impl LemmaSplitKeyV1 {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        if self.language.is_empty() {
            return Err("lemma split key has empty language");
        }
        if self.normalized_lemma.is_empty() {
            return Err("lemma split key has empty normalized lemma");
        }
        if self.language.len() > u32::MAX as usize
            || self.normalized_lemma.len() > u32::MAX as usize
        {
            return Err("lemma split key exceeds canonical string width");
        }
        Ok(())
    }

    pub(super) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.language.len() + self.normalized_lemma.len());
        push_string(&mut bytes, &self.language);
        push_string(&mut bytes, &self.normalized_lemma);
        bytes
    }
}

pub(super) fn deterministic_productive_split(
    key: &LemmaSplitKeyV1,
    split_seed: u64,
) -> ProductiveSplitV1 {
    let bucket = crate::nanda_wave::phase_field::stable_hash64(&key.canonical_bytes(), split_seed)
        % super::PRODUCTIVE_V1_SPLIT_BUCKETS;
    match bucket {
        0..=7_999 => ProductiveSplitV1::Train,
        8_000..=8_999 => ProductiveSplitV1::Calibration,
        _ => ProductiveSplitV1::HeldoutLemma,
    }
}

pub(super) fn deterministic_inner_fold(key: &LemmaSplitKeyV1, split_seed: u64) -> u8 {
    let mut bytes = key.canonical_bytes();
    bytes.extend_from_slice(INNER_FOLD_LABEL);
    (crate::nanda_wave::phase_field::stable_hash64(&bytes, split_seed) % PRODUCTIVE_V1_INNER_FOLDS)
        as u8
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MorphologyEventV1 {
    pub(super) lemma: LemmaSplitKeyV1,
    pub(super) normalized_surface: String,
    pub(super) canonical_form_ref: ImportedCanonicalL2FormRefV1,
    pub(super) canonical_feature_mask: u32,
    pub(super) slot: MorphologySlotKeyV1,
    pub(super) support: u32,
    pub(super) provenance: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ContextOccurrenceEventV1 {
    pub(super) lemma: LemmaSplitKeyV1,
    pub(super) normalized_surface: String,
    pub(super) canonical_form_ref: ImportedCanonicalL2FormRefV1,
    pub(super) canonical_feature_mask: u32,
    pub(super) slot: MorphologySlotKeyV1,
    pub(super) scene: L2LocalSceneV1,
    pub(super) source_event_identity: Vec<u8>,
    pub(super) support: u32,
    pub(super) provenance: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ContextContradictionEventV1 {
    pub(super) lemma: LemmaSplitKeyV1,
    pub(super) normalized_surface: String,
    pub(super) canonical_form_ref: ImportedCanonicalL2FormRefV1,
    pub(super) canonical_feature_mask: u32,
    pub(super) slot: MorphologySlotKeyV1,
    pub(super) scene: L2LocalSceneV1,
    pub(super) competitors: Vec<CanonicalL2BindingIdentityV1>,
    pub(super) source_event_identity: Vec<u8>,
    pub(super) support: u32,
    pub(super) provenance: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum FeedbackOutcomeV1 {
    Accept = 1,
    Continue = 2,
    Revert = 3,
    Replace = 4,
    Ignore = 5,
}

impl FeedbackOutcomeV1 {
    pub(super) const fn is_explicit_anti(self) -> bool {
        matches!(self, Self::Revert | Self::Replace)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FeedbackEventV1 {
    pub(super) lemma: LemmaSplitKeyV1,
    pub(super) proposal_identity: [u8; 32],
    pub(super) package_generation: u64,
    pub(super) visible_input: String,
    pub(super) proposed_form: ProductiveCandidateIdentityV1,
    pub(super) outcome: FeedbackOutcomeV1,
    pub(super) resulting_committed_surface: Option<String>,
    pub(super) scene: L2LocalSceneV1,
    pub(super) timestamp_bucket: u64,
    pub(super) provenance: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProofEventV1 {
    pub(super) lemma: LemmaSplitKeyV1,
    pub(super) proof_identity: [u8; 32],
    pub(super) observed_surface: String,
    pub(super) valid_targets: Vec<CanonicalL2BindingIdentityV1>,
    pub(super) explicit_invalid_competitors: Vec<CanonicalL2BindingIdentityV1>,
    pub(super) scene: L2LocalSceneV1,
    pub(super) provenance: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TypedProductiveEventV1 {
    Morphology(MorphologyEventV1),
    ContextOccurrence(ContextOccurrenceEventV1),
    Feedback(FeedbackEventV1),
    Proof(ProofEventV1),
    ContextContradiction(ContextContradictionEventV1),
}

impl TypedProductiveEventV1 {
    pub(super) const fn kind(&self) -> ProductiveEventKindV1 {
        match self {
            Self::Morphology(_) => ProductiveEventKindV1::Morphology,
            Self::ContextOccurrence(_) => ProductiveEventKindV1::ContextOccurrence,
            Self::Feedback(_) => ProductiveEventKindV1::Feedback,
            Self::Proof(_) => ProductiveEventKindV1::Proof,
            Self::ContextContradiction(_) => ProductiveEventKindV1::ContextContradiction,
        }
    }

    pub(super) fn lemma(&self) -> &LemmaSplitKeyV1 {
        match self {
            Self::Morphology(event) => &event.lemma,
            Self::ContextOccurrence(event) => &event.lemma,
            Self::Feedback(event) => &event.lemma,
            Self::Proof(event) => &event.lemma,
            Self::ContextContradiction(event) => &event.lemma,
        }
    }

    pub(super) fn provenance(&self) -> &[u8] {
        match self {
            Self::Morphology(event) => &event.provenance,
            Self::ContextOccurrence(event) => &event.provenance,
            Self::Feedback(event) => &event.provenance,
            Self::Proof(event) => &event.provenance,
            Self::ContextContradiction(event) => &event.provenance,
        }
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        self.lemma().validate().map_err(str::to_string)?;
        let (surface, support) = match self {
            Self::Morphology(event) => {
                (Some(event.normalized_surface.as_str()), Some(event.support))
            }
            Self::ContextOccurrence(event) => {
                event.scene.validate().map_err(str::to_string)?;
                (Some(event.normalized_surface.as_str()), Some(event.support))
            }
            Self::ContextContradiction(event) => {
                event.scene.validate().map_err(str::to_string)?;
                if event.competitors.is_empty() {
                    return Err(
                        "context contradiction event has no explicit competitor".to_string()
                    );
                }
                (Some(event.normalized_surface.as_str()), Some(event.support))
            }
            Self::Feedback(event) => {
                event.scene.validate().map_err(str::to_string)?;
                (Some(event.visible_input.as_str()), None)
            }
            Self::Proof(event) => {
                event.scene.validate().map_err(str::to_string)?;
                if event.valid_targets.is_empty() {
                    return Err("proof event has no valid target identity".to_string());
                }
                (Some(event.observed_surface.as_str()), None)
            }
        };
        if surface.is_some_and(str::is_empty) {
            return Err("typed productive event has empty surface".to_string());
        }
        if support.is_some_and(|support| support == 0) {
            return Err("typed productive event has zero support".to_string());
        }
        Ok(())
    }

    pub(super) fn envelope(&self, split_seed: u64) -> Result<EventEnvelopeV1, String> {
        self.validate()?;
        let kind = self.kind();
        let split = if matches!(self, Self::Proof(_)) {
            ProductiveSplitV1::Proof
        } else {
            deterministic_productive_split(self.lemma(), split_seed)
        };
        let primary_identity = self.lemma().canonical_bytes();
        let canonical_fields = self.canonical_field_bytes()?;
        let mut hash_input =
            Vec::with_capacity(3 + canonical_fields.len() + self.provenance().len());
        hash_input.extend_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
        hash_input.push(kind as u8);
        hash_input.extend_from_slice(&canonical_fields);
        hash_input.extend_from_slice(self.provenance());
        let event_sha256: [u8; 32] = Sha256::digest(&hash_input).into();
        Ok(EventEnvelopeV1 {
            split,
            inner_fold: (split == ProductiveSplitV1::Train)
                .then(|| deterministic_inner_fold(self.lemma(), split_seed)),
            kind,
            primary_identity,
            event_sha256,
            canonical_event_bytes: hash_input,
        })
    }

    fn canonical_field_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = self.lemma().canonical_bytes();
        match self {
            Self::Morphology(event) => {
                push_checked_string(&mut bytes, &event.normalized_surface)?;
                bytes.extend_from_slice(&event.canonical_form_ref.0.to_le_bytes());
                bytes.extend_from_slice(&event.canonical_feature_mask.to_le_bytes());
                bytes.extend_from_slice(&event.slot.to_bytes());
                bytes.extend_from_slice(&event.support.to_le_bytes());
            }
            Self::ContextOccurrence(event) => {
                push_checked_string(&mut bytes, &event.normalized_surface)?;
                bytes.extend_from_slice(&event.canonical_form_ref.0.to_le_bytes());
                bytes.extend_from_slice(&event.canonical_feature_mask.to_le_bytes());
                bytes.extend_from_slice(&event.slot.to_bytes());
                push_checked_bytes(&mut bytes, &event.scene.canonical_bytes())?;
                push_checked_bytes(&mut bytes, &event.source_event_identity)?;
                bytes.extend_from_slice(&event.support.to_le_bytes());
            }
            Self::ContextContradiction(event) => {
                push_checked_string(&mut bytes, &event.normalized_surface)?;
                bytes.extend_from_slice(&event.canonical_form_ref.0.to_le_bytes());
                bytes.extend_from_slice(&event.canonical_feature_mask.to_le_bytes());
                bytes.extend_from_slice(&event.slot.to_bytes());
                push_checked_bytes(&mut bytes, &event.scene.canonical_bytes())?;
                let mut competitors = event.competitors.clone();
                competitors.sort_unstable();
                competitors.dedup();
                push_u32(&mut bytes, checked_u32(competitors.len())?);
                for competitor in competitors {
                    push_binding_identity(&mut bytes, competitor);
                }
                push_checked_bytes(&mut bytes, &event.source_event_identity)?;
                bytes.extend_from_slice(&event.support.to_le_bytes());
            }
            Self::Feedback(event) => {
                bytes.extend_from_slice(&event.proposal_identity);
                bytes.extend_from_slice(&event.package_generation.to_le_bytes());
                push_checked_string(&mut bytes, &event.visible_input)?;
                push_candidate_identity(&mut bytes, event.proposed_form);
                bytes.push(event.outcome as u8);
                push_optional_string(&mut bytes, event.resulting_committed_surface.as_deref())?;
                push_checked_bytes(&mut bytes, &event.scene.canonical_bytes())?;
                bytes.extend_from_slice(&event.timestamp_bucket.to_le_bytes());
            }
            Self::Proof(event) => {
                bytes.extend_from_slice(&event.proof_identity);
                push_checked_string(&mut bytes, &event.observed_surface)?;
                let mut targets = event.valid_targets.clone();
                targets.sort_unstable();
                targets.dedup();
                push_u32(&mut bytes, checked_u32(targets.len())?);
                for target in targets {
                    push_binding_identity(&mut bytes, target);
                }
                let mut competitors = event.explicit_invalid_competitors.clone();
                competitors.sort_unstable();
                competitors.dedup();
                push_u32(&mut bytes, checked_u32(competitors.len())?);
                for competitor in competitors {
                    push_binding_identity(&mut bytes, competitor);
                }
                push_checked_bytes(&mut bytes, &event.scene.canonical_bytes())?;
            }
        }
        Ok(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EventEnvelopeV1 {
    pub(super) split: ProductiveSplitV1,
    pub(super) inner_fold: Option<u8>,
    pub(super) kind: ProductiveEventKindV1,
    pub(super) primary_identity: Vec<u8>,
    pub(super) event_sha256: [u8; 32],
    pub(super) canonical_event_bytes: Vec<u8>,
}

pub(super) fn sort_and_deduplicate_events(events: &mut Vec<EventEnvelopeV1>) {
    events.sort_by(|left, right| {
        left.split
            .cmp(&right.split)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.primary_identity.cmp(&right.primary_identity))
            .then_with(|| left.event_sha256.cmp(&right.event_sha256))
            .then_with(|| left.canonical_event_bytes.cmp(&right.canonical_event_bytes))
    });
    events.dedup_by(|left, right| {
        left.event_sha256 == right.event_sha256
            && left.canonical_event_bytes == right.canonical_event_bytes
    });
}

#[derive(Clone, Debug)]
pub(super) struct TypedEventSpoolConfigV1 {
    pub(super) root: PathBuf,
    pub(super) shard_count: usize,
    pub(super) split_seed: u64,
    pub(super) compiler_version: u32,
    pub(super) normalization_version: u32,
    pub(super) write_buffer_bytes: usize,
}

pub(super) struct TypedEventSpoolWriterV1 {
    config: TypedEventSpoolConfigV1,
    writers: Vec<BufWriter<File>>,
    sequences: Vec<u64>,
    counts: Vec<u64>,
}

impl TypedEventSpoolWriterV1 {
    pub(super) fn create(config: TypedEventSpoolConfigV1) -> Result<Self, String> {
        if config.shard_count == 0 {
            return Err("typed event spool requires at least one shard".to_string());
        }
        if config.write_buffer_bytes < config.shard_count * SPOOL_RECORD_HEADER_BYTES {
            return Err("typed event spool write-buffer budget is too small".to_string());
        }
        fs::create_dir_all(&config.root).map_err(|error| error.to_string())?;
        let per_shard_capacity = config.write_buffer_bytes / config.shard_count;
        let mut writers = Vec::with_capacity(config.shard_count);
        for shard in 0..config.shard_count {
            let path = shard_path(&config.root, shard);
            let file = File::create(path).map_err(|error| error.to_string())?;
            writers.push(BufWriter::with_capacity(per_shard_capacity, file));
        }
        let shard_count = config.shard_count;
        Ok(Self {
            config,
            writers,
            sequences: vec![0; shard_count],
            counts: vec![0; shard_count],
        })
    }

    pub(super) fn append(&mut self, event: &TypedProductiveEventV1) -> Result<[u8; 32], String> {
        let envelope = event.envelope(self.config.split_seed)?;
        let shard = (crate::nanda_wave::phase_field::stable_hash64(
            &envelope.primary_identity,
            self.config.split_seed,
        ) as usize)
            % self.config.shard_count;
        let sequence = self.sequences[shard];
        self.sequences[shard] = sequence
            .checked_add(1)
            .ok_or_else(|| "typed spool sequence overflow".to_string())?;
        let payload_bytes = checked_u32(envelope.canonical_event_bytes.len())?;
        let mut header = [0_u8; SPOOL_RECORD_HEADER_BYTES];
        header[0..4].copy_from_slice(&SPOOL_MAGIC);
        header[4..6].copy_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
        header[6] = envelope.kind as u8;
        header[7] = envelope.split as u8;
        header[8..16].copy_from_slice(&sequence.to_le_bytes());
        header[16..20].copy_from_slice(&payload_bytes.to_le_bytes());
        header[20..52].copy_from_slice(&envelope.event_sha256);
        let crc = crc32_parts(&header, &envelope.canonical_event_bytes);
        header[52..56].copy_from_slice(&crc.to_le_bytes());
        self.writers[shard]
            .write_all(&header)
            .and_then(|_| self.writers[shard].write_all(&envelope.canonical_event_bytes))
            .map_err(|error| error.to_string())?;
        self.counts[shard] += 1;
        Ok(envelope.event_sha256)
    }

    pub(super) fn finish(mut self) -> Result<TypedEventSpoolManifestV1, String> {
        for writer in &mut self.writers {
            writer.flush().map_err(|error| error.to_string())?;
        }
        let shards = self
            .counts
            .into_iter()
            .enumerate()
            .map(|(shard, record_count)| TypedEventSpoolShardV1 {
                path: shard_path(&self.config.root, shard),
                record_count,
            })
            .collect();
        Ok(TypedEventSpoolManifestV1 {
            schema_version: PRODUCTIVE_V1_SCHEMA_VERSION,
            split_seed: self.config.split_seed,
            compiler_version: self.config.compiler_version,
            normalization_version: self.config.normalization_version,
            shards,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TypedEventSpoolShardV1 {
    pub(super) path: PathBuf,
    pub(super) record_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TypedEventSpoolManifestV1 {
    pub(super) schema_version: u16,
    pub(super) split_seed: u64,
    pub(super) compiler_version: u32,
    pub(super) normalization_version: u32,
    pub(super) shards: Vec<TypedEventSpoolShardV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SpoolRecordV1 {
    pub(super) kind: ProductiveEventKindV1,
    pub(super) split: ProductiveSplitV1,
    pub(super) sequence: u64,
    pub(super) event_sha256: [u8; 32],
    pub(super) canonical_event_bytes: Vec<u8>,
}

pub(super) struct VerifiedSpoolShardReaderV1 {
    reader: BufReader<File>,
    expected_sequence: u64,
}

impl VerifiedSpoolShardReaderV1 {
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        Ok(Self {
            reader: BufReader::new(File::open(path).map_err(|error| error.to_string())?),
            expected_sequence: 0,
        })
    }

    pub(super) fn next_record(&mut self) -> Result<Option<SpoolRecordV1>, String> {
        let mut header = [0_u8; SPOOL_RECORD_HEADER_BYTES];
        match self.reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(1) => self
                .reader
                .read_exact(&mut header[1..])
                .map_err(|_| "truncated typed spool header".to_string())?,
            Ok(_) => unreachable!("one-byte read returned more than one byte"),
            Err(error) => return Err(error.to_string()),
        }
        if header[0..4] != SPOOL_MAGIC {
            return Err("typed spool magic mismatch".to_string());
        }
        if u16::from_le_bytes(header[4..6].try_into().expect("fixed slice"))
            != PRODUCTIVE_V1_SCHEMA_VERSION
        {
            return Err("typed spool schema version mismatch".to_string());
        }
        let kind = decode_event_kind(header[6])?;
        let split = decode_split(header[7])?;
        let sequence = u64::from_le_bytes(header[8..16].try_into().expect("fixed slice"));
        if sequence != self.expected_sequence {
            return Err("typed spool sequence is not monotonic".to_string());
        }
        self.expected_sequence = self
            .expected_sequence
            .checked_add(1)
            .ok_or_else(|| "typed spool sequence overflows u64".to_string())?;
        let payload_bytes =
            u32::from_le_bytes(header[16..20].try_into().expect("fixed slice")) as usize;
        let event_sha256 = header[20..52].try_into().expect("fixed slice");
        let expected_crc = u32::from_le_bytes(header[52..56].try_into().expect("fixed slice"));
        let mut payload = vec![0_u8; payload_bytes];
        self.reader
            .read_exact(&mut payload)
            .map_err(|error| error.to_string())?;
        let mut zeroed_header = header;
        zeroed_header[52..56].fill(0);
        if crc32_parts(&zeroed_header, &payload) != expected_crc {
            return Err("typed spool CRC mismatch".to_string());
        }
        if <[u8; 32]>::from(Sha256::digest(&payload)) != event_sha256 {
            return Err("typed spool event SHA-256 mismatch".to_string());
        }
        Ok(Some(SpoolRecordV1 {
            kind,
            split,
            sequence,
            event_sha256,
            canonical_event_bytes: payload,
        }))
    }
}

pub(super) struct VerifiedSpoolShardWriterV1 {
    writer: BufWriter<File>,
    sequence: u64,
}

impl VerifiedSpoolShardWriterV1 {
    pub(super) fn create(path: &Path, buffer_bytes: usize) -> Result<Self, String> {
        if buffer_bytes < SPOOL_RECORD_HEADER_BYTES {
            return Err("typed spool output buffer is smaller than one record header".to_string());
        }
        Ok(Self {
            writer: BufWriter::with_capacity(
                buffer_bytes,
                File::create(path).map_err(|error| error.to_string())?,
            ),
            sequence: 0,
        })
    }

    pub(super) fn append(&mut self, record: &SpoolRecordV1) -> Result<(), String> {
        if <[u8; 32]>::from(Sha256::digest(&record.canonical_event_bytes)) != record.event_sha256 {
            return Err("typed spool writer received a payload/hash mismatch".to_string());
        }
        let payload_bytes = checked_u32(record.canonical_event_bytes.len())?;
        let mut header = [0_u8; SPOOL_RECORD_HEADER_BYTES];
        header[0..4].copy_from_slice(&SPOOL_MAGIC);
        header[4..6].copy_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
        header[6] = record.kind as u8;
        header[7] = record.split as u8;
        header[8..16].copy_from_slice(&self.sequence.to_le_bytes());
        header[16..20].copy_from_slice(&payload_bytes.to_le_bytes());
        header[20..52].copy_from_slice(&record.event_sha256);
        let crc = crc32_parts(&header, &record.canonical_event_bytes);
        header[52..56].copy_from_slice(&crc.to_le_bytes());
        self.writer
            .write_all(&header)
            .and_then(|_| self.writer.write_all(&record.canonical_event_bytes))
            .map_err(|error| error.to_string())?;
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| "typed spool output sequence overflows u64".to_string())?;
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<u64, String> {
        self.writer.flush().map_err(|error| error.to_string())?;
        Ok(self.sequence)
    }
}

pub(super) fn read_verified_spool_shard(path: &Path) -> Result<Vec<SpoolRecordV1>, String> {
    let mut reader = VerifiedSpoolShardReaderV1::open(path)?;
    let mut records = Vec::new();
    while let Some(record) = reader.next_record()? {
        records.push(record);
    }
    Ok(records)
}

pub(super) fn decode_verified_spool_record(
    record: &SpoolRecordV1,
    split_seed: u64,
) -> Result<TypedProductiveEventV1, String> {
    let event = decode_canonical_event(record.kind, &record.canonical_event_bytes)?;
    let envelope = event.envelope(split_seed)?;
    if envelope.kind != record.kind
        || envelope.split != record.split
        || envelope.event_sha256 != record.event_sha256
        || envelope.canonical_event_bytes != record.canonical_event_bytes
    {
        return Err("decoded typed spool event disagrees with its canonical envelope".to_string());
    }
    Ok(event)
}

fn decode_canonical_event(
    expected_kind: ProductiveEventKindV1,
    bytes: &[u8],
) -> Result<TypedProductiveEventV1, String> {
    let mut input = CanonicalEventInputV1::new(bytes);
    if input.u16()? != PRODUCTIVE_V1_SCHEMA_VERSION {
        return Err("typed spool payload schema version mismatch".to_string());
    }
    if decode_event_kind(input.u8()?)? != expected_kind {
        return Err("typed spool payload kind disagrees with record header".to_string());
    }
    let lemma = LemmaSplitKeyV1 {
        language: input.string()?,
        normalized_lemma: input.string()?,
    };
    let event = match expected_kind {
        ProductiveEventKindV1::Morphology => {
            let normalized_surface = input.string()?;
            let canonical_form_ref = ImportedCanonicalL2FormRefV1(input.u32()?);
            let canonical_feature_mask = input.u32()?;
            let slot = input.slot()?;
            let support = input.u32()?;
            TypedProductiveEventV1::Morphology(MorphologyEventV1 {
                lemma,
                normalized_surface,
                canonical_form_ref,
                canonical_feature_mask,
                slot,
                support,
                provenance: input.remaining().to_vec(),
            })
        }
        ProductiveEventKindV1::ContextOccurrence => {
            let normalized_surface = input.string()?;
            let canonical_form_ref = ImportedCanonicalL2FormRefV1(input.u32()?);
            let canonical_feature_mask = input.u32()?;
            let slot = input.slot()?;
            let scene = L2LocalSceneV1::decode_canonical_bytes(input.length_prefixed_bytes()?)
                .map_err(str::to_string)?;
            let source_event_identity = input.length_prefixed_bytes()?.to_vec();
            let support = input.u32()?;
            TypedProductiveEventV1::ContextOccurrence(ContextOccurrenceEventV1 {
                lemma,
                normalized_surface,
                canonical_form_ref,
                canonical_feature_mask,
                slot,
                scene,
                source_event_identity,
                support,
                provenance: input.remaining().to_vec(),
            })
        }
        ProductiveEventKindV1::ContextContradiction => {
            let normalized_surface = input.string()?;
            let canonical_form_ref = ImportedCanonicalL2FormRefV1(input.u32()?);
            let canonical_feature_mask = input.u32()?;
            let slot = input.slot()?;
            let scene = L2LocalSceneV1::decode_canonical_bytes(input.length_prefixed_bytes()?)
                .map_err(str::to_string)?;
            let competitor_count = input.u32()? as usize;
            let mut competitors = Vec::with_capacity(competitor_count);
            for _ in 0..competitor_count {
                competitors.push(input.binding_identity()?);
            }
            let source_event_identity = input.length_prefixed_bytes()?.to_vec();
            let support = input.u32()?;
            TypedProductiveEventV1::ContextContradiction(ContextContradictionEventV1 {
                lemma,
                normalized_surface,
                canonical_form_ref,
                canonical_feature_mask,
                slot,
                scene,
                competitors,
                source_event_identity,
                support,
                provenance: input.remaining().to_vec(),
            })
        }
        ProductiveEventKindV1::Feedback => {
            let proposal_identity = input.array()?;
            let package_generation = input.u64()?;
            let visible_input = input.string()?;
            let proposed_form = input.candidate_identity()?;
            let outcome = match input.u8()? {
                1 => FeedbackOutcomeV1::Accept,
                2 => FeedbackOutcomeV1::Continue,
                3 => FeedbackOutcomeV1::Revert,
                4 => FeedbackOutcomeV1::Replace,
                5 => FeedbackOutcomeV1::Ignore,
                _ => return Err("typed spool feedback outcome is invalid".to_string()),
            };
            let resulting_committed_surface = input.optional_string()?;
            let scene = L2LocalSceneV1::decode_canonical_bytes(input.length_prefixed_bytes()?)
                .map_err(str::to_string)?;
            let timestamp_bucket = input.u64()?;
            TypedProductiveEventV1::Feedback(FeedbackEventV1 {
                lemma,
                proposal_identity,
                package_generation,
                visible_input,
                proposed_form,
                outcome,
                resulting_committed_surface,
                scene,
                timestamp_bucket,
                provenance: input.remaining().to_vec(),
            })
        }
        ProductiveEventKindV1::Proof => {
            let proof_identity = input.array()?;
            let observed_surface = input.string()?;
            let target_count = input.u32()? as usize;
            let mut valid_targets = Vec::with_capacity(target_count);
            for _ in 0..target_count {
                valid_targets.push(input.binding_identity()?);
            }
            let competitor_count = input.u32()? as usize;
            let mut explicit_invalid_competitors = Vec::with_capacity(competitor_count);
            for _ in 0..competitor_count {
                explicit_invalid_competitors.push(input.binding_identity()?);
            }
            let scene = L2LocalSceneV1::decode_canonical_bytes(input.length_prefixed_bytes()?)
                .map_err(str::to_string)?;
            TypedProductiveEventV1::Proof(ProofEventV1 {
                lemma,
                proof_identity,
                observed_surface,
                valid_targets,
                explicit_invalid_competitors,
                scene,
                provenance: input.remaining().to_vec(),
            })
        }
    };
    event.validate()?;
    Ok(event)
}

struct CanonicalEventInputV1<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> CanonicalEventInputV1<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| "typed spool canonical read overflow".to_string())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "typed spool canonical event is truncated".to_string())?
            .try_into()
            .expect("fixed typed event field");
        self.offset = end;
        Ok(value)
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| "typed spool canonical variable range overflow".to_string())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "typed spool canonical variable field is truncated".to_string())?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn string(&mut self) -> Result<String, String> {
        let count = self.u32()? as usize;
        std::str::from_utf8(self.bytes(count)?)
            .map(str::to_owned)
            .map_err(|_| "typed spool canonical string is not UTF-8".to_string())
    }

    fn optional_string(&mut self) -> Result<Option<String>, String> {
        match self.u8()? {
            0 => Ok(None),
            1 => self.string().map(Some),
            _ => Err("typed spool optional string presence flag is invalid".to_string()),
        }
    }

    fn length_prefixed_bytes(&mut self) -> Result<&'a [u8], String> {
        let count = self.u32()? as usize;
        self.bytes(count)
    }

    fn slot(&mut self) -> Result<MorphologySlotKeyV1, String> {
        MorphologySlotKeyV1::from_bytes(self.array()?).map_err(str::to_string)
    }

    fn candidate_identity(&mut self) -> Result<ProductiveCandidateIdentityV1, String> {
        Ok(ProductiveCandidateIdentityV1 {
            lemma_id: self.u32()?,
            paradigm_id: self.u32()?,
            program_id: self.u32()?,
            target_slot_id: self.u32()?,
            normalized_surface_id: self.u32()?,
            variant_id: self.u16()?,
        })
    }

    fn binding_identity(&mut self) -> Result<CanonicalL2BindingIdentityV1, String> {
        Ok(CanonicalL2BindingIdentityV1 {
            lemma_ref: super::types::ImportedCanonicalL2LemmaRefV1(self.u32()?),
            form_ref: ImportedCanonicalL2FormRefV1(self.u32()?),
            legacy_feature_mask: self.u32()?,
        })
    }

    fn remaining(&mut self) -> &'a [u8] {
        let value = &self.bytes[self.offset..];
        self.offset = self.bytes.len();
        value
    }
}

fn shard_path(root: &Path, shard: usize) -> PathBuf {
    root.join(format!("events-{shard:05}.p2s"))
}

fn decode_event_kind(value: u8) -> Result<ProductiveEventKindV1, String> {
    match value {
        1 => Ok(ProductiveEventKindV1::Morphology),
        2 => Ok(ProductiveEventKindV1::ContextOccurrence),
        3 => Ok(ProductiveEventKindV1::Feedback),
        4 => Ok(ProductiveEventKindV1::Proof),
        5 => Ok(ProductiveEventKindV1::ContextContradiction),
        _ => Err("unknown typed spool event kind".to_string()),
    }
}

fn decode_split(value: u8) -> Result<ProductiveSplitV1, String> {
    match value {
        0 => Ok(ProductiveSplitV1::Train),
        1 => Ok(ProductiveSplitV1::Calibration),
        2 => Ok(ProductiveSplitV1::HeldoutLemma),
        3 => Ok(ProductiveSplitV1::Proof),
        _ => Err("unknown typed spool split".to_string()),
    }
}

fn crc32_parts(header_with_zero_crc: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in header_with_zero_crc.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn checked_u32(value: usize) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| "canonical sequence exceeds u32".to_string())
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_string(bytes: &mut Vec<u8>, value: &str) {
    push_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_checked_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), String> {
    push_u32(bytes, checked_u32(value.len())?);
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_checked_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), String> {
    push_u32(bytes, checked_u32(value.len())?);
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_optional_string(bytes: &mut Vec<u8>, value: Option<&str>) -> Result<(), String> {
    match value {
        Some(value) => {
            bytes.push(1);
            push_checked_string(bytes, value)?;
        }
        None => bytes.push(0),
    }
    Ok(())
}

fn push_binding_identity(bytes: &mut Vec<u8>, identity: CanonicalL2BindingIdentityV1) {
    bytes.extend_from_slice(&identity.lemma_ref.0.to_le_bytes());
    bytes.extend_from_slice(&identity.form_ref.0.to_le_bytes());
    bytes.extend_from_slice(&identity.legacy_feature_mask.to_le_bytes());
}

fn push_candidate_identity(bytes: &mut Vec<u8>, identity: ProductiveCandidateIdentityV1) {
    bytes.extend_from_slice(&identity.lemma_id.to_le_bytes());
    bytes.extend_from_slice(&identity.paradigm_id.to_le_bytes());
    bytes.extend_from_slice(&identity.program_id.to_le_bytes());
    bytes.extend_from_slice(&identity.target_slot_id.to_le_bytes());
    bytes.extend_from_slice(&identity.normalized_surface_id.to_le_bytes());
    bytes.extend_from_slice(&identity.variant_id.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn morphology_event(provenance: &[u8]) -> TypedProductiveEventV1 {
        TypedProductiveEventV1::Morphology(MorphologyEventV1 {
            lemma: LemmaSplitKeyV1 {
                language: "ru".to_string(),
                normalized_lemma: "lemma".to_string(),
            },
            normalized_surface: "surface".to_string(),
            canonical_form_ref: ImportedCanonicalL2FormRefV1(0),
            canonical_feature_mask: 1,
            slot: MorphologySlotKeyV1::new(2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            support: 1,
            provenance: provenance.to_vec(),
        })
    }

    fn spool_record(event: &TypedProductiveEventV1) -> SpoolRecordV1 {
        let envelope = event.envelope(17).expect("envelope");
        SpoolRecordV1 {
            kind: envelope.kind,
            split: envelope.split,
            sequence: 0,
            event_sha256: envelope.event_sha256,
            canonical_event_bytes: envelope.canonical_event_bytes,
        }
    }

    #[test]
    fn split_and_inner_fold_are_lemma_owned() {
        let key = morphology_event(b"a").lemma().clone();
        assert_eq!(
            deterministic_productive_split(&key, 17),
            deterministic_productive_split(&key, 17)
        );
        assert!(deterministic_inner_fold(&key, 17) < 5);
    }

    #[test]
    fn full_event_identity_is_idempotent_but_keeps_provenance_distinct() {
        let first = morphology_event(b"source-a").envelope(17).expect("event");
        let duplicate = morphology_event(b"source-a").envelope(17).expect("event");
        let independent = morphology_event(b"source-b").envelope(17).expect("event");
        assert_eq!(first.event_sha256, duplicate.event_sha256);
        assert_ne!(first.event_sha256, independent.event_sha256);
    }

    #[test]
    fn sharded_spool_roundtrip_checks_sequence_crc_and_hash() {
        let root = std::env::temp_dir().join(format!(
            "lay-productive-v1-spool-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.clone(),
            shard_count: 2,
            split_seed: 17,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("writer");
        writer
            .append(&morphology_event(b"source-a"))
            .expect("append");
        writer
            .append(&morphology_event(b"source-b"))
            .expect("append");
        let manifest = writer.finish().expect("finish");
        let records = manifest
            .shards
            .iter()
            .flat_map(|shard| read_verified_spool_shard(&shard.path).expect("verified shard"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);
        assert!(records
            .iter()
            .all(|record| decode_verified_spool_record(record, 17).is_ok()));
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn canonical_spool_decoder_roundtrips_every_typed_event_kind() {
        let lemma = LemmaSplitKeyV1 {
            language: "ru".to_string(),
            normalized_lemma: "lemma".to_string(),
        };
        let slot = MorphologySlotKeyV1::new(2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        let scene = L2LocalSceneV1::default();
        let events = vec![
            morphology_event(b"morphology"),
            TypedProductiveEventV1::ContextOccurrence(ContextOccurrenceEventV1 {
                lemma: lemma.clone(),
                normalized_surface: "surface".to_string(),
                canonical_form_ref: ImportedCanonicalL2FormRefV1(0),
                canonical_feature_mask: 1,
                slot,
                scene: scene.clone(),
                source_event_identity: b"source-event".to_vec(),
                support: 2,
                provenance: b"context".to_vec(),
            }),
            TypedProductiveEventV1::ContextContradiction(ContextContradictionEventV1 {
                lemma: lemma.clone(),
                normalized_surface: "surface".to_string(),
                canonical_form_ref: ImportedCanonicalL2FormRefV1(0),
                canonical_feature_mask: 1,
                slot,
                scene: scene.clone(),
                competitors: vec![CanonicalL2BindingIdentityV1 {
                    lemma_ref: super::super::types::ImportedCanonicalL2LemmaRefV1(1),
                    form_ref: ImportedCanonicalL2FormRefV1(2),
                    legacy_feature_mask: 3,
                }],
                source_event_identity: b"source-event".to_vec(),
                support: 2,
                provenance: b"contradiction".to_vec(),
            }),
            TypedProductiveEventV1::Feedback(FeedbackEventV1 {
                lemma: lemma.clone(),
                proposal_identity: [7; 32],
                package_generation: 3,
                visible_input: "surface".to_string(),
                proposed_form: ProductiveCandidateIdentityV1 {
                    lemma_id: 1,
                    paradigm_id: 1,
                    program_id: 1,
                    target_slot_id: 1,
                    normalized_surface_id: 1,
                    variant_id: 1,
                },
                outcome: FeedbackOutcomeV1::Replace,
                resulting_committed_surface: Some("replacement".to_string()),
                scene: scene.clone(),
                timestamp_bucket: 9,
                provenance: b"feedback".to_vec(),
            }),
            TypedProductiveEventV1::Proof(ProofEventV1 {
                lemma,
                proof_identity: [8; 32],
                observed_surface: "damaged".to_string(),
                valid_targets: vec![CanonicalL2BindingIdentityV1 {
                    lemma_ref: super::super::types::ImportedCanonicalL2LemmaRefV1(0),
                    form_ref: ImportedCanonicalL2FormRefV1(0),
                    legacy_feature_mask: 1,
                }],
                explicit_invalid_competitors: Vec::new(),
                scene,
                provenance: b"proof".to_vec(),
            }),
        ];
        for event in events {
            assert_eq!(
                decode_verified_spool_record(&spool_record(&event), 17).expect("decode"),
                event
            );
        }
    }

    #[test]
    fn event_sort_deduplicates_only_identical_full_events() {
        let mut events = vec![
            morphology_event(b"source-b").envelope(17).expect("event"),
            morphology_event(b"source-a").envelope(17).expect("event"),
            morphology_event(b"source-a").envelope(17).expect("event"),
        ];
        sort_and_deduplicate_events(&mut events);
        assert_eq!(events.len(), 2);
        assert_ne!(
            events[0].canonical_event_bytes,
            events[1].canonical_event_bytes
        );
    }

    #[test]
    fn feedback_non_selection_is_not_anti_evidence() {
        assert!(!FeedbackOutcomeV1::Ignore.is_explicit_anti());
        assert!(!FeedbackOutcomeV1::Continue.is_explicit_anti());
        assert!(FeedbackOutcomeV1::Revert.is_explicit_anti());
        assert!(FeedbackOutcomeV1::Replace.is_explicit_anti());
    }
}
