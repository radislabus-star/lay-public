use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::events::{
    ContextContradictionEventV1, ContextOccurrenceEventV1, LemmaSplitKeyV1, MorphologyEventV1,
    ProofEventV1, TypedEventSpoolConfigV1, TypedEventSpoolManifestV1, TypedEventSpoolWriterV1,
    TypedProductiveEventV1,
};
use super::reduce::ReducedMorphologyManifestV1;
use super::scene::{
    BoundaryKindV1, L2LocalSceneV1, LocalTokenObservationV1, TypedLocalMorphologyObservationV1,
};
use super::transition_reduce::{MorphologyAxisLabelV1, MorphologyAxisSchemaV1};
use super::types::{
    CanonicalL2BindingIdentityV1, ImportedCanonicalL2FormRefV1, ImportedCanonicalL2LemmaRefV1,
    MorphologyApplicabilityMaskV1, MorphologySlotKeyV1,
};

const RAW_CONTEXT_MAGIC: [u8; 4] = *b"P2R1";
const RAW_CONTEXT_HEADER_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum RawContextKindV1 {
    Train = 1,
    Heldout = 2,
    NeighborTrain = 3,
    NeighborHeldout = 4,
}

impl RawContextKindV1 {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Train),
            2 => Ok(Self::Heldout),
            3 => Ok(Self::NeighborTrain),
            4 => Ok(Self::NeighborHeldout),
            _ => Err("productive raw context kind is invalid".to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RawContextRowV1 {
    pub(super) kind: RawContextKindV1,
    pub(super) lemma: LemmaSplitKeyV1,
    pub(super) normalized_surface: String,
    pub(super) canonical_form_ref: ImportedCanonicalL2FormRefV1,
    pub(super) canonical_feature_mask: u32,
    pub(super) slot: MorphologySlotKeyV1,
    pub(super) context: String,
    pub(super) competitor_surfaces: Vec<String>,
    pub(super) provenance: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(super) struct ProductiveRawCorpusConfigV1 {
    pub(super) corpus_path: PathBuf,
    pub(super) canonical_l2_path: PathBuf,
    pub(super) axis_schema_path: PathBuf,
    pub(super) raw_context_path: PathBuf,
    pub(super) source_role: String,
    pub(super) expected_corpus_sha256: [u8; 32],
    pub(super) expected_corpus_bytes: u64,
    pub(super) morphology_spool: TypedEventSpoolConfigV1,
    pub(super) raw_context_write_buffer_bytes: usize,
}

#[derive(Clone, Debug)]
pub(super) struct ProductiveRawCorpusManifestV1 {
    pub(super) corpus_sha256: [u8; 32],
    pub(super) corpus_bytes: u64,
    pub(super) canonical_l2_sha256: [u8; 32],
    pub(super) morphology_rows: u64,
    pub(super) admitted_morphology_rows: u64,
    pub(super) ungrounded_morphology_rows: u64,
    pub(super) context_rows: u64,
    pub(super) admitted_context_rows: u64,
    pub(super) ungrounded_context_rows: u64,
    pub(super) raw_context_path: PathBuf,
    pub(super) morphology_spool: TypedEventSpoolManifestV1,
    pub(super) axis_schema: MorphologyAxisSchemaV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveContextReplayManifestV1 {
    pub(super) source_rows: u64,
    pub(super) context_occurrence_events: u64,
    pub(super) direct_contradiction_events: u64,
    pub(super) proof_events: u64,
    pub(super) event_spool: TypedEventSpoolManifestV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AxisSchemaDocumentV1 {
    schema_version: u32,
    pos_applicability: Vec<AxisApplicabilityDocumentV1>,
    labels: Vec<AxisLabelDocumentV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AxisApplicabilityDocumentV1 {
    pos: u8,
    mask: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AxisLabelDocumentV1 {
    axis: u8,
    value: u8,
    label: String,
}

pub(super) fn load_axis_schema(path: &Path) -> Result<MorphologyAxisSchemaV1, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let document: AxisSchemaDocumentV1 =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let mut pos_applicability = BTreeMap::new();
    for row in document.pos_applicability {
        let mask =
            MorphologyApplicabilityMaskV1::new(row.mask).map_err(|error| error.to_string())?;
        if row.pos < 2 || pos_applicability.insert(row.pos, mask).is_some() {
            return Err("productive axis schema repeats or invalidates a POS".to_string());
        }
    }
    let labels = document
        .labels
        .into_iter()
        .map(|row| MorphologyAxisLabelV1 {
            axis: row.axis,
            value: row.value,
            label: row.label,
        })
        .collect::<Vec<_>>();
    if labels.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("productive axis schema labels are not canonically ordered".to_string());
    }
    Ok(MorphologyAxisSchemaV1 {
        schema_version: document.schema_version,
        pos_applicability,
        labels,
    })
}

pub(super) fn run_productive_raw_corpus_pass(
    config: &ProductiveRawCorpusConfigV1,
) -> Result<ProductiveRawCorpusManifestV1, String> {
    if config.source_role.is_empty()
        || config.expected_corpus_sha256 == [0; 32]
        || config.expected_corpus_bytes == 0
        || config.raw_context_write_buffer_bytes < RAW_CONTEXT_HEADER_BYTES
    {
        return Err("productive raw corpus pass has an invalid manifest contract".to_string());
    }
    let axis_schema = load_axis_schema(&config.axis_schema_path)?;
    let canonical_l2 = super::super::runtime::StandaloneL2Field::load(&config.canonical_l2_path)?;
    let canonical_l2_sha256 = sha256_file(&config.canonical_l2_path)?;
    let mut event_writer = TypedEventSpoolWriterV1::create(config.morphology_spool.clone())?;
    let raw_context_file =
        File::create(&config.raw_context_path).map_err(|error| error.to_string())?;
    let mut context_writer = RawContextWriterV1 {
        writer: BufWriter::with_capacity(config.raw_context_write_buffer_bytes, raw_context_file),
        sequence: 0,
    };
    let source = File::open(&config.corpus_path).map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(source);
    let mut raw = Vec::new();
    let mut source_hasher = Sha256::new();
    let mut source_bytes = 0_u64;
    let mut line_number = 0_u64;
    let mut morphology_rows = 0_u64;
    let mut admitted_morphology_rows = 0_u64;
    let mut ungrounded_morphology_rows = 0_u64;
    let mut context_rows = 0_u64;
    let mut admitted_context_rows = 0_u64;
    let mut ungrounded_context_rows = 0_u64;
    loop {
        raw.clear();
        let read = reader
            .read_until(b'\n', &mut raw)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        line_number = line_number
            .checked_add(1)
            .ok_or_else(|| "productive source line count overflow".to_string())?;
        source_bytes = source_bytes
            .checked_add(read as u64)
            .ok_or_else(|| "productive source byte count overflow".to_string())?;
        source_hasher.update(&raw);
        while matches!(raw.last(), Some(b'\n' | b'\r')) {
            raw.pop();
        }
        let line = std::str::from_utf8(&raw)
            .map_err(|_| format!("line {line_number}: corpus row is not UTF-8"))?
            .trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.as_slice() {
            ["F", lemma, surface, features] => {
                morphology_rows += 1;
                let normalized_surface = normalize(surface);
                let Some(form_ref) = canonical_l2.form_ref_for_surface(&normalized_surface) else {
                    ungrounded_morphology_rows += 1;
                    continue;
                };
                let canonical_feature_mask =
                    crate::nanda_wave::morphology_phase::parse_features(features)
                        .map_err(|error| format!("line {line_number}: {error}"))?;
                let slot = axis_schema
                    .parse_feature_labels(features)
                    .map_err(|error| format!("line {line_number}: {error}"))?;
                event_writer.append(&TypedProductiveEventV1::Morphology(MorphologyEventV1 {
                    lemma: LemmaSplitKeyV1 {
                        language: "ru".to_string(),
                        normalized_lemma: normalize(lemma),
                    },
                    normalized_surface,
                    canonical_form_ref: ImportedCanonicalL2FormRefV1(form_ref),
                    canonical_feature_mask,
                    slot,
                    support: 1,
                    provenance: provenance(&config.source_role, line_number),
                }))?;
                admitted_morphology_rows += 1;
            }
            [kind @ ("T" | "H"), lemma, surface, features, context] => {
                context_rows += 1;
                let row_kind = if *kind == "T" {
                    RawContextKindV1::Train
                } else {
                    RawContextKindV1::Heldout
                };
                if append_raw_context(
                    &mut context_writer,
                    &canonical_l2,
                    &axis_schema,
                    row_kind,
                    lemma,
                    surface,
                    features,
                    context,
                    &[],
                    &config.source_role,
                    line_number,
                )? {
                    admitted_context_rows += 1;
                } else {
                    ungrounded_context_rows += 1;
                }
            }
            [kind @ ("NT" | "NH"), lemma, surface, features, context, competitors] => {
                context_rows += 1;
                let row_kind = if *kind == "NT" {
                    RawContextKindV1::NeighborTrain
                } else {
                    RawContextKindV1::NeighborHeldout
                };
                let competitor_surfaces = competitors
                    .split(',')
                    .map(normalize)
                    .filter(|surface| !surface.is_empty())
                    .collect::<Vec<_>>();
                if competitor_surfaces.is_empty() {
                    return Err(format!(
                        "line {line_number}: neighbor row has no competitors"
                    ));
                }
                if append_raw_context(
                    &mut context_writer,
                    &canonical_l2,
                    &axis_schema,
                    row_kind,
                    lemma,
                    surface,
                    features,
                    context,
                    &competitor_surfaces,
                    &config.source_role,
                    line_number,
                )? {
                    admitted_context_rows += 1;
                } else {
                    ungrounded_context_rows += 1;
                }
            }
            _ => {
                return Err(format!(
                    "line {line_number}: expected F/T/H/NT/NH typed morphology row"
                ));
            }
        }
    }
    let corpus_sha256: [u8; 32] = source_hasher.finalize().into();
    if source_bytes != config.expected_corpus_bytes
        || corpus_sha256 != config.expected_corpus_sha256
    {
        return Err(
            "productive corpus bytes or SHA-256 disagree with the source manifest".to_string(),
        );
    }
    context_writer.finish()?;
    let morphology_spool = event_writer.finish()?;
    Ok(ProductiveRawCorpusManifestV1 {
        corpus_sha256,
        corpus_bytes: source_bytes,
        canonical_l2_sha256,
        morphology_rows,
        admitted_morphology_rows,
        ungrounded_morphology_rows,
        context_rows,
        admitted_context_rows,
        ungrounded_context_rows,
        raw_context_path: config.raw_context_path.clone(),
        morphology_spool,
        axis_schema,
    })
}

pub(super) fn replay_productive_context_spool(
    raw_context_path: &Path,
    reduced: &ReducedMorphologyManifestV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &MorphologyAxisSchemaV1,
    output: TypedEventSpoolConfigV1,
) -> Result<ProductiveContextReplayManifestV1, String> {
    if !reduced.imported_identity_verified || reduced.imported_lemma_refs.is_empty() {
        return Err("productive context replay requires verified imported identities".to_string());
    }
    let mut reader = RawContextReaderV1::open(raw_context_path)?;
    let mut writer = TypedEventSpoolWriterV1::create(output)?;
    let mut source_rows = 0_u64;
    let mut context_occurrence_events = 0_u64;
    let mut direct_contradiction_events = 0_u64;
    let mut proof_events = 0_u64;
    while let Some(row) = reader.next()? {
        source_rows = source_rows
            .checked_add(1)
            .ok_or_else(|| "productive context replay row count overflow".to_string())?;
        let lemma_ref = *reduced
            .imported_lemma_refs
            .get(&(
                row.lemma.language.clone(),
                row.lemma.normalized_lemma.clone(),
            ))
            .ok_or_else(|| {
                format!(
                    "productive context target lemma {:?} lacks imported ownership",
                    row.lemma.normalized_lemma
                )
            })?;
        let target = CanonicalL2BindingIdentityV1 {
            lemma_ref: ImportedCanonicalL2LemmaRefV1(lemma_ref),
            form_ref: row.canonical_form_ref,
            legacy_feature_mask: row.canonical_feature_mask,
        };
        let target_bindings = canonical_l2
            .imported_binding_identities_for_form(row.canonical_form_ref.0)
            .into_iter()
            .collect::<Vec<_>>();
        if !target_bindings.contains(&(lemma_ref, row.canonical_feature_mask)) {
            return Err(
                "productive context target disagrees with canonical L2 binding".to_string(),
            );
        }
        let scene = build_grounded_scene(&row, canonical_l2, axis_schema)?;
        let competitors = resolve_competitors(&row, canonical_l2)?;
        let source_event_identity: [u8; 32] = Sha256::digest(encode_raw_context(&row)?).into();
        match row.kind {
            RawContextKindV1::Train | RawContextKindV1::NeighborTrain => {
                writer.append(&TypedProductiveEventV1::ContextOccurrence(
                    ContextOccurrenceEventV1 {
                        lemma: row.lemma.clone(),
                        normalized_surface: row.normalized_surface.clone(),
                        canonical_form_ref: row.canonical_form_ref,
                        canonical_feature_mask: row.canonical_feature_mask,
                        slot: row.slot,
                        scene: scene.clone(),
                        source_event_identity: source_event_identity.to_vec(),
                        support: 1,
                        provenance: row.provenance.clone(),
                    },
                ))?;
                context_occurrence_events += 1;
                if row.kind == RawContextKindV1::NeighborTrain {
                    if competitors.is_empty() {
                        return Err(
                            "productive NT context lost every explicit competitor".to_string()
                        );
                    }
                    writer.append(&TypedProductiveEventV1::ContextContradiction(
                        ContextContradictionEventV1 {
                            lemma: row.lemma,
                            normalized_surface: row.normalized_surface,
                            canonical_form_ref: row.canonical_form_ref,
                            canonical_feature_mask: row.canonical_feature_mask,
                            slot: row.slot,
                            scene,
                            competitors,
                            source_event_identity: source_event_identity.to_vec(),
                            support: 1,
                            provenance: row.provenance,
                        },
                    ))?;
                    direct_contradiction_events += 1;
                }
            }
            RawContextKindV1::Heldout | RawContextKindV1::NeighborHeldout => {
                writer.append(&TypedProductiveEventV1::Proof(ProofEventV1 {
                    lemma: row.lemma,
                    proof_identity: source_event_identity,
                    observed_surface: row.normalized_surface,
                    valid_targets: vec![target],
                    explicit_invalid_competitors: competitors,
                    scene,
                    provenance: row.provenance,
                }))?;
                proof_events += 1;
            }
        }
    }
    Ok(ProductiveContextReplayManifestV1 {
        source_rows,
        context_occurrence_events,
        direct_contradiction_events,
        proof_events,
        event_spool: writer.finish()?,
    })
}

fn resolve_competitors(
    row: &RawContextRowV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
) -> Result<Vec<CanonicalL2BindingIdentityV1>, String> {
    let mut identities = Vec::new();
    for surface in &row.competitor_surfaces {
        let form_ref = canonical_l2.form_ref_for_surface(surface).ok_or_else(|| {
            "productive explicit competitor disappeared from canonical L2".to_string()
        })?;
        let bindings = canonical_l2.imported_binding_identities_for_form(form_ref);
        if bindings.is_empty() {
            return Err("productive explicit competitor has no canonical binding".to_string());
        }
        identities.extend(bindings.into_iter().map(|(lemma_ref, feature_mask)| {
            CanonicalL2BindingIdentityV1 {
                lemma_ref: ImportedCanonicalL2LemmaRefV1(lemma_ref),
                form_ref: ImportedCanonicalL2FormRefV1(form_ref),
                legacy_feature_mask: feature_mask,
            }
        }));
    }
    identities.sort_unstable();
    identities.dedup();
    Ok(identities)
}

fn build_grounded_scene(
    row: &RawContextRowV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &MorphologyAxisSchemaV1,
) -> Result<L2LocalSceneV1, String> {
    let tokens = row.context.split_whitespace().collect::<Vec<_>>();
    let hole = tokens
        .iter()
        .position(|token| *token == "_")
        .ok_or_else(|| "productive context replay lost its _ slot".to_string())?;
    let left = tokens[..hole]
        .iter()
        .rev()
        .take(2)
        .map(|token| normalize_context_token(token))
        .collect::<Vec<_>>();
    let right = tokens[hole + 1..]
        .iter()
        .take(2)
        .map(|token| normalize_context_token(token))
        .collect::<Vec<_>>();
    let left_near = left.first().cloned().filter(|token| !token.is_empty());
    let left_far = left.get(1).cloned().filter(|token| !token.is_empty());
    let right_near = right.first().cloned().filter(|token| !token.is_empty());
    let right_far = right.get(1).cloned().filter(|token| !token.is_empty());
    let positioned = [
        (-2_i8, left_far.clone()),
        (-1_i8, left_near.clone()),
        (1_i8, right_near.clone()),
        (2_i8, right_far.clone()),
    ];
    let mut morphology = Vec::new();
    let mut observations = BTreeMap::new();
    for (position, surface) in positioned
        .iter()
        .filter_map(|(position, surface)| surface.as_ref().map(|surface| (*position, surface)))
    {
        let Some(form_ref) = canonical_l2.form_ref_for_surface(surface) else {
            continue;
        };
        let bindings = canonical_l2.imported_binding_identities_for_form(form_ref);
        let mut typed = Vec::new();
        for (lemma_ref, feature_mask) in bindings {
            let labels =
                crate::nanda_wave::morphology_phase::canonical_feature_labels(feature_mask)?;
            let slot = axis_schema.parse_feature_labels(&labels.join(":"))?;
            morphology.push(TypedLocalMorphologyObservationV1 {
                position,
                lemma_id: Some(lemma_ref),
                slot,
            });
            typed.push((lemma_ref, slot));
        }
        observations.insert(position, typed);
    }
    morphology.sort_unstable();
    morphology.dedup();
    let token_observation = |position: i8, surface: Option<String>| {
        surface.map(|normalized_surface| {
            let typed = observations.get(&position).cloned().unwrap_or_default();
            let lemma_ids = typed
                .iter()
                .map(|(lemma_ref, _)| *lemma_ref)
                .collect::<std::collections::BTreeSet<_>>();
            let slots = typed
                .iter()
                .map(|(_, slot)| *slot)
                .collect::<std::collections::BTreeSet<_>>();
            LocalTokenObservationV1 {
                normalized_surface,
                lemma_id: (lemma_ids.len() == 1).then(|| *lemma_ids.first().expect("one lemma")),
                morphology_slot: (slots.len() == 1).then(|| *slots.first().expect("one slot")),
            }
        })
    };
    let scene = L2LocalSceneV1 {
        current_token: row.normalized_surface.clone(),
        current_normalized_scalars: row.normalized_surface.chars().map(u32::from).collect(),
        left_tokens: [
            token_observation(-2, left_far),
            token_observation(-1, left_near),
        ],
        right_tokens: [
            token_observation(1, right_near),
            token_observation(2, right_far),
        ],
        boundary_before: BoundaryKindV1::Token,
        boundary_after: BoundaryKindV1::Token,
        morphology,
        ..L2LocalSceneV1::default()
    };
    scene.validate().map_err(str::to_string)?;
    Ok(scene)
}

fn normalize_context_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
        .to_lowercase()
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn append_raw_context(
    writer: &mut RawContextWriterV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &MorphologyAxisSchemaV1,
    kind: RawContextKindV1,
    lemma: &str,
    surface: &str,
    features: &str,
    context: &str,
    competitor_surfaces: &[String],
    source_role: &str,
    line_number: u64,
) -> Result<bool, String> {
    if context
        .split_whitespace()
        .filter(|token| *token == "_")
        .count()
        != 1
    {
        return Err(format!(
            "line {line_number}: context requires exactly one _ slot"
        ));
    }
    let normalized_surface = normalize(surface);
    let Some(form_ref) = canonical_l2.form_ref_for_surface(&normalized_surface) else {
        return Ok(false);
    };
    for competitor in competitor_surfaces {
        if canonical_l2.form_ref_for_surface(competitor).is_none() {
            return Err(format!(
                "line {line_number}: explicit competitor {competitor:?} is absent from canonical L2"
            ));
        }
    }
    let canonical_feature_mask = crate::nanda_wave::morphology_phase::parse_features(features)
        .map_err(|error| format!("line {line_number}: {error}"))?;
    let slot = axis_schema
        .parse_feature_labels(features)
        .map_err(|error| format!("line {line_number}: {error}"))?;
    writer.append(&RawContextRowV1 {
        kind,
        lemma: LemmaSplitKeyV1 {
            language: "ru".to_string(),
            normalized_lemma: normalize(lemma),
        },
        normalized_surface,
        canonical_form_ref: ImportedCanonicalL2FormRefV1(form_ref),
        canonical_feature_mask,
        slot,
        context: normalize(context),
        competitor_surfaces: competitor_surfaces.to_vec(),
        provenance: provenance(source_role, line_number),
    })?;
    Ok(true)
}

struct RawContextWriterV1 {
    writer: BufWriter<File>,
    sequence: u64,
}

impl RawContextWriterV1 {
    fn append(&mut self, row: &RawContextRowV1) -> Result<(), String> {
        let payload = encode_raw_context(row)?;
        let mut header = [0_u8; RAW_CONTEXT_HEADER_BYTES];
        header[0..4].copy_from_slice(&RAW_CONTEXT_MAGIC);
        header[4..8].copy_from_slice(
            &u32::try_from(payload.len())
                .map_err(|_| "productive raw context row exceeds u32".to_string())?
                .to_le_bytes(),
        );
        header[8..12].copy_from_slice(&(self.sequence as u32).to_le_bytes());
        header[12..16].copy_from_slice(&crc32(&payload).to_le_bytes());
        self.writer
            .write_all(&header)
            .and_then(|_| self.writer.write_all(&payload))
            .map_err(|error| error.to_string())?;
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| "productive raw context sequence overflow".to_string())?;
        if self.sequence > u64::from(u32::MAX) {
            return Err("productive raw context row count exceeds u32".to_string());
        }
        Ok(())
    }

    fn finish(mut self) -> Result<u64, String> {
        self.writer.flush().map_err(|error| error.to_string())?;
        Ok(self.sequence)
    }
}

pub(super) struct RawContextReaderV1 {
    reader: BufReader<File>,
    expected_sequence: u32,
}

impl RawContextReaderV1 {
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        Ok(Self {
            reader: BufReader::new(File::open(path).map_err(|error| error.to_string())?),
            expected_sequence: 0,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<RawContextRowV1>, String> {
        let mut header = [0_u8; RAW_CONTEXT_HEADER_BYTES];
        match self.reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(1) => self
                .reader
                .read_exact(&mut header[1..])
                .map_err(|_| "productive raw context header is truncated".to_string())?,
            Ok(_) => unreachable!("one-byte raw context read returned more than one byte"),
            Err(error) => return Err(error.to_string()),
        }
        if header[0..4] != RAW_CONTEXT_MAGIC {
            return Err("productive raw context magic is invalid".to_string());
        }
        let payload_bytes =
            u32::from_le_bytes(header[4..8].try_into().expect("fixed raw context")) as usize;
        let sequence = u32::from_le_bytes(header[8..12].try_into().expect("fixed raw context"));
        let expected_crc =
            u32::from_le_bytes(header[12..16].try_into().expect("fixed raw context"));
        if sequence != self.expected_sequence {
            return Err("productive raw context sequence is not monotonic".to_string());
        }
        let mut payload = vec![0_u8; payload_bytes];
        self.reader
            .read_exact(&mut payload)
            .map_err(|_| "productive raw context payload is truncated".to_string())?;
        if crc32(&payload) != expected_crc {
            return Err("productive raw context CRC mismatch".to_string());
        }
        self.expected_sequence = self
            .expected_sequence
            .checked_add(1)
            .ok_or_else(|| "productive raw context sequence overflow".to_string())?;
        decode_raw_context(&payload).map(Some)
    }
}

fn encode_raw_context(row: &RawContextRowV1) -> Result<Vec<u8>, String> {
    let mut bytes = vec![row.kind as u8, 0, 0, 0];
    push_string(&mut bytes, &row.lemma.language)?;
    push_string(&mut bytes, &row.lemma.normalized_lemma)?;
    push_string(&mut bytes, &row.normalized_surface)?;
    bytes.extend_from_slice(&row.canonical_form_ref.0.to_le_bytes());
    bytes.extend_from_slice(&row.canonical_feature_mask.to_le_bytes());
    bytes.extend_from_slice(&row.slot.to_bytes());
    push_string(&mut bytes, &row.context)?;
    push_u32(&mut bytes, row.competitor_surfaces.len())?;
    for competitor in &row.competitor_surfaces {
        push_string(&mut bytes, competitor)?;
    }
    push_bytes(&mut bytes, &row.provenance)?;
    Ok(bytes)
}

fn decode_raw_context(bytes: &[u8]) -> Result<RawContextRowV1, String> {
    let mut input = RawInputV1::new(bytes);
    let kind = RawContextKindV1::decode(input.u8()?)?;
    if input.bytes(3)? != [0; 3] {
        return Err("productive raw context reserved bytes are not zero".to_string());
    }
    let lemma = LemmaSplitKeyV1 {
        language: input.string()?,
        normalized_lemma: input.string()?,
    };
    let normalized_surface = input.string()?;
    let canonical_form_ref = ImportedCanonicalL2FormRefV1(input.u32()?);
    let canonical_feature_mask = input.u32()?;
    let slot = MorphologySlotKeyV1::from_bytes(input.array()?).map_err(str::to_string)?;
    let context = input.string()?;
    let competitor_count = input.u32()? as usize;
    let mut competitor_surfaces = Vec::with_capacity(competitor_count);
    for _ in 0..competitor_count {
        competitor_surfaces.push(input.string()?);
    }
    let provenance = input.length_prefixed_bytes()?.to_vec();
    if !input.is_empty() {
        return Err("productive raw context row has an unowned suffix".to_string());
    }
    Ok(RawContextRowV1 {
        kind,
        lemma,
        normalized_surface,
        canonical_form_ref,
        canonical_feature_mask,
        slot,
        context,
        competitor_surfaces,
        provenance,
    })
}

fn sha256_file(path: &Path) -> Result<[u8; 32], String> {
    let mut reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

fn provenance(source_role: &str, line_number: u64) -> Vec<u8> {
    format!("{source_role}:{line_number}").into_bytes()
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn push_u32(bytes: &mut Vec<u8>, value: usize) -> Result<(), String> {
    bytes.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| "productive raw context sequence exceeds u32".to_string())?
            .to_le_bytes(),
    );
    Ok(())
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), String> {
    push_u32(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), String> {
    push_bytes(bytes, value.as_bytes())
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

struct RawInputV1<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> RawInputV1<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.bytes(N)?
            .try_into()
            .map_err(|_| "productive raw context fixed field is truncated".to_string())
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| "productive raw context read overflow".to_string())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "productive raw context row is truncated".to_string())?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.array::<1>()?[0])
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn string(&mut self) -> Result<String, String> {
        std::str::from_utf8(self.length_prefixed_bytes()?)
            .map(str::to_owned)
            .map_err(|_| "productive raw context string is not UTF-8".to_string())
    }

    fn length_prefixed_bytes(&mut self) -> Result<&'a [u8], String> {
        let count = self.u32()? as usize;
        self.bytes(count)
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::events::{
        decode_verified_spool_record, read_verified_spool_shard, ProductiveEventKindV1,
    };
    use super::super::reduce::ReducedMorphologyManifestV1;
    use super::super::types::MorphologySlotKeyV1;
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lay-productive-v1-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    fn row(kind: RawContextKindV1) -> RawContextRowV1 {
        RawContextRowV1 {
            kind,
            lemma: LemmaSplitKeyV1 {
                language: "ru".to_string(),
                normalized_lemma: "lemma-a".to_string(),
            },
            normalized_surface: "alpha".to_string(),
            canonical_form_ref: ImportedCanonicalL2FormRefV1(0),
            canonical_feature_mask: crate::nanda_wave::morphology_phase::parse_features(
                "noun:nom:sg",
            )
            .expect("feature mask"),
            slot: MorphologySlotKeyV1::new(2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0),
            context: "_ middle".to_string(),
            competitor_surfaces: Vec::new(),
            provenance: b"fixture:1".to_vec(),
        }
    }

    fn write_rows(path: &Path, rows: &[RawContextRowV1]) {
        let file = File::create(path).expect("raw context file");
        let mut writer = RawContextWriterV1 {
            writer: BufWriter::with_capacity(4096, file),
            sequence: 0,
        };
        for row in rows {
            writer.append(row).expect("raw context row");
        }
        assert_eq!(
            writer.finish().expect("finish raw context"),
            rows.len() as u64
        );
    }

    fn canonical_field() -> super::super::super::runtime::StandaloneL2Field {
        let corpus = super::super::super::teacher::L2TeacherCorpus::parse_tsv(
            "F\tlemma-a\talpha\tnoun:nom:sg\n\
             F\tlemma-a\tomega\tnoun:gen:sg\n\
             F\tlemma-b\tmiddle\tnoun:nom:sg\n\
             T\tlemma-a\talpha\tnoun:nom:sg\t_ middle\n\
             H\tlemma-a\tomega\tnoun:gen:sg\tmiddle _\n",
        )
        .expect("canonical corpus");
        let terminals = BTreeMap::from([("alpha", 7), ("middle", 11), ("omega", 13)]);
        let (package, _) =
            super::super::super::compiler::compile_l2_package(&corpus, 99, |surface| {
                terminals.get(surface).copied()
            })
            .expect("canonical package");
        super::super::super::runtime::StandaloneL2Field::from_package(package)
            .expect("canonical field")
    }

    #[test]
    fn russian_axis_schema_loads_and_parses_applicable_unknowns() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/morphology/productive_v1_ru_axis_schema.json");
        let schema = load_axis_schema(&path).expect("axis schema");
        let slot = schema
            .parse_feature_labels("verb:sg:p2:imp:perf")
            .expect("verb slot");

        assert_eq!(schema.schema_version, 1);
        assert_eq!(schema.pos_applicability.len(), 4);
        assert_eq!(slot.pos_domain(), 3);
        assert_eq!(slot.axes()[1], 2);
        assert_eq!(slot.axes()[4], 3);
        assert_eq!(slot.axes()[6], 3);
        assert_eq!(slot.axes()[7], 2);
        assert_eq!(
            slot.axes()[5],
            super::super::types::AXIS_UNKNOWN_OR_UNANNOTATED
        );
        let exclusive = schema
            .parse_feature_labels("verb:imp_excl:sg:imp:perf")
            .expect("exclusive imperative slot");
        let inclusive = schema
            .parse_feature_labels("verb:imp_incl:pl:imp:perf")
            .expect("inclusive imperative slot");
        assert_eq!(exclusive.axes()[12], 2);
        assert_eq!(inclusive.axes()[12], 3);
    }

    #[test]
    fn raw_context_spool_roundtrips_and_rejects_crc_corruption() {
        let root = temp_root("raw-context");
        std::fs::create_dir_all(&root).expect("temp root");
        let path = root.join("context.p2r");
        let expected = row(RawContextKindV1::NeighborTrain);
        write_rows(&path, std::slice::from_ref(&expected));

        let mut reader = RawContextReaderV1::open(&path).expect("reader");
        assert_eq!(reader.next().expect("row"), Some(expected));
        assert_eq!(reader.next().expect("eof"), None);

        let mut bytes = std::fs::read(&path).expect("raw bytes");
        bytes[RAW_CONTEXT_HEADER_BYTES] ^= 0x01;
        std::fs::write(&path, bytes).expect("corrupt raw context");
        assert!(RawContextReaderV1::open(&path)
            .expect("corrupt reader")
            .next()
            .expect_err("CRC rejection")
            .contains("CRC mismatch"));
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn context_replay_separates_train_contradiction_and_read_only_proof_events() {
        let root = temp_root("context-replay");
        std::fs::create_dir_all(&root).expect("temp root");
        let raw_path = root.join("context.p2r");
        let mut rows = Vec::new();
        for kind in [
            RawContextKindV1::Train,
            RawContextKindV1::NeighborTrain,
            RawContextKindV1::Heldout,
            RawContextKindV1::NeighborHeldout,
        ] {
            let mut event = row(kind);
            if matches!(
                kind,
                RawContextKindV1::NeighborTrain | RawContextKindV1::NeighborHeldout
            ) {
                event.competitor_surfaces.push("middle".to_string());
            }
            rows.push(event);
        }
        write_rows(&raw_path, &rows);

        let field = canonical_field();
        let schema = load_axis_schema(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("data/morphology/productive_v1_ru_axis_schema.json"),
        )
        .expect("axis schema");
        let reduced = ReducedMorphologyManifestV1 {
            path: root.join("unused.p2l"),
            split_seed: 17,
            compiler_version: 1,
            normalization_version: 1,
            lemma_count: 2,
            form_count: 3,
            train_event_count: 3,
            morphology_slots: Vec::new(),
            maximum_observed_scalars: 6,
            payload_sha256: [1; 32],
            imported_identity_verified: true,
            imported_lemma_refs: BTreeMap::from([
                (("ru".to_string(), "lemma-a".to_string()), 0),
                (("ru".to_string(), "lemma-b".to_string()), 1),
            ]),
        };
        let replay = replay_productive_context_spool(
            &raw_path,
            &reduced,
            &field,
            &schema,
            TypedEventSpoolConfigV1 {
                root: root.join("events"),
                shard_count: 2,
                split_seed: 17,
                compiler_version: 1,
                normalization_version: 1,
                write_buffer_bytes: 4096,
            },
        )
        .expect("context replay");

        assert_eq!(replay.source_rows, 4);
        assert_eq!(replay.context_occurrence_events, 2);
        assert_eq!(replay.direct_contradiction_events, 1);
        assert_eq!(replay.proof_events, 2);
        let mut kinds = replay
            .event_spool
            .shards
            .iter()
            .flat_map(|shard| read_verified_spool_shard(&shard.path).expect("verified shard"))
            .map(|record| {
                decode_verified_spool_record(&record, replay.event_spool.split_seed)
                    .expect("typed replay event")
                    .kind()
            })
            .collect::<Vec<_>>();
        kinds.sort_unstable();
        assert_eq!(
            kinds,
            vec![
                ProductiveEventKindV1::ContextOccurrence,
                ProductiveEventKindV1::ContextOccurrence,
                ProductiveEventKindV1::Proof,
                ProductiveEventKindV1::Proof,
                ProductiveEventKindV1::ContextContradiction,
            ]
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
