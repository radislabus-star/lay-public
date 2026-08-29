use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::anchor_recovery_reduce::{
    induce_anchor_recovery_field, reopen_anchor_recovery_field, AnchorRecoveryManifestV1,
};
use super::induce::{
    derive_edit_template, select_canonical_anchor, CanonicalFormObservationV1, EditOperationV1,
    EditTemplateV1, ParadigmSignatureV1, ParadigmTransitionKeyV1, SourceAnchorV1,
};
use super::reduce::{ReducedLemmaReaderV1, ReducedMorphologyManifestV1};
use super::types::{MorphologyApplicabilityMaskV1, MorphologySlotKeyV1, MORPHOLOGY_AXIS_COUNT};
use super::PRODUCTIVE_V1_SCHEMA_VERSION;

const TRANSITION_MAGIC: [u8; 4] = *b"P2T1";
const SUPPORT_MAGIC: [u8; 4] = *b"P2S2";
const CLASSIFIED_MAGIC: [u8; 4] = *b"P2C1";
const SIGNATURE_MAGIC: [u8; 4] = *b"P2G1";
const PARADIGM_MAGIC: [u8; 4] = *b"P2P1";
const BINDING_MAGIC: [u8; 4] = *b"P2B1";
const COMPATIBILITY_OBSERVATION_MAGIC: [u8; 4] = *b"P2K1";
const COMPATIBILITY_INDEX_MAGIC: [u8; 4] = *b"P2X1";
const PARADIGM_POSTING_MAGIC: [u8; 4] = *b"P2O1";
const WIRE_HEADER_BYTES: usize = 16;
const WIRE_ACCOUNTING_BYTES: usize = 48;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MorphologyAxisLabelV1 {
    pub(super) axis: u8,
    pub(super) value: u8,
    pub(super) label: String,
}

#[derive(Clone, Debug)]
pub(super) struct MorphologyAxisSchemaV1 {
    pub(super) schema_version: u32,
    pub(super) pos_applicability: BTreeMap<u8, MorphologyApplicabilityMaskV1>,
    pub(super) labels: Vec<MorphologyAxisLabelV1>,
}

impl MorphologyAxisSchemaV1 {
    pub(super) fn parse_feature_labels(&self, raw: &str) -> Result<MorphologySlotKeyV1, String> {
        let mut by_label = BTreeMap::<&str, (u8, u8)>::new();
        for label in &self.labels {
            if by_label
                .insert(label.label.as_str(), (label.axis, label.value))
                .is_some()
            {
                return Err("productive axis schema repeats a textual label".to_string());
            }
        }
        let encoded = raw
            .split(':')
            .map(|label| {
                by_label
                    .get(label)
                    .copied()
                    .ok_or_else(|| format!("productive axis schema has no label {label:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let pos_values = encoded
            .iter()
            .filter_map(|(axis, value)| (*axis == 0).then_some(*value))
            .collect::<BTreeSet<_>>();
        if pos_values.len() != 1 {
            return Err("productive feature row requires exactly one POS label".to_string());
        }
        let pos = *pos_values.first().expect("one POS value");
        let applicability = self.applicability(pos)?;
        let mut axes = [0_u8; MORPHOLOGY_AXIS_COUNT];
        for (axis, value) in axes.iter_mut().enumerate() {
            if applicability.contains(axis) {
                *value = super::types::AXIS_UNKNOWN_OR_UNANNOTATED;
            }
        }
        for (axis, value) in encoded {
            let axis = axis as usize;
            if axis >= MORPHOLOGY_AXIS_COUNT || !applicability.contains(axis) {
                return Err("productive feature label is inapplicable to its POS".to_string());
            }
            if axes[axis] >= 2 && axes[axis] != value {
                return Err(
                    "productive feature row repeats an axis with conflicting values".to_string(),
                );
            }
            axes[axis] = value;
        }
        let slot = MorphologySlotKeyV1::new(
            axes[0], axes[1], axes[2], axes[3], axes[4], axes[5], axes[6], axes[7], axes[8],
            axes[9], axes[10], axes[11], axes[12],
        );
        slot.validate(applicability)
            .map_err(|error| error.to_string())?;
        Ok(slot)
    }

    pub(super) fn validate_for_slots(
        &self,
        slots: &[MorphologySlotKeyV1],
    ) -> Result<[u8; 32], String> {
        if self.schema_version == 0 || slots.is_empty() {
            return Err("productive axis schema version or slot set is empty".to_string());
        }
        let mut used_pos = BTreeSet::new();
        let mut used_labels = BTreeSet::new();
        for slot in slots {
            let pos = slot.pos_domain();
            if pos < 2 {
                return Err("productive slot has no concrete POS value".to_string());
            }
            let mask = self.pos_applicability.get(&pos).copied().ok_or_else(|| {
                "productive axis schema lacks a POS applicability mask".to_string()
            })?;
            slot.validate(mask).map_err(|error| error.to_string())?;
            used_pos.insert(pos);
            for (axis, value) in slot.axes().into_iter().enumerate() {
                if value >= 2 {
                    used_labels.insert((axis as u8, value));
                }
            }
        }
        if self
            .pos_applicability
            .keys()
            .copied()
            .collect::<BTreeSet<_>>()
            != used_pos
        {
            return Err(
                "productive axis schema contains an unused POS applicability mask".to_string(),
            );
        }
        let mut labels = self.labels.clone();
        labels.sort_unstable();
        if labels != self.labels
            || labels
                .windows(2)
                .any(|pair| (pair[0].axis, pair[0].value) == (pair[1].axis, pair[1].value))
        {
            return Err("productive axis labels are not strictly canonical".to_string());
        }
        let encoded_labels = labels
            .iter()
            .map(|entry| (entry.axis, entry.value))
            .collect::<BTreeSet<_>>();
        if encoded_labels != used_labels
            || labels.iter().any(|entry| {
                entry.axis as usize >= MORPHOLOGY_AXIS_COUNT
                    || entry.value < 2
                    || entry.label.is_empty()
                    || entry.label.chars().any(char::is_control)
            })
        {
            return Err("productive axis labels are missing, unused, or invalid".to_string());
        }
        let mut canonical = Vec::new();
        canonical.extend_from_slice(&self.schema_version.to_le_bytes());
        canonical.extend_from_slice(&(self.pos_applicability.len() as u32).to_le_bytes());
        for (pos, mask) in &self.pos_applicability {
            canonical.push(*pos);
            canonical.extend_from_slice(&mask.bits().to_le_bytes());
        }
        canonical.extend_from_slice(&(labels.len() as u32).to_le_bytes());
        for entry in labels {
            canonical.extend_from_slice(&[entry.axis, entry.value]);
            push_bytes(&mut canonical, entry.label.as_bytes())?;
        }
        Ok(Sha256::digest(canonical).into())
    }

    pub(super) fn applicability(&self, pos: u8) -> Result<MorphologyApplicabilityMaskV1, String> {
        self.pos_applicability
            .get(&pos)
            .copied()
            .ok_or_else(|| "productive axis schema lacks the lemma POS".to_string())
    }
}

#[derive(Clone, Debug)]
pub(super) struct TransitionReduceConfigV1 {
    pub(super) root: PathBuf,
    pub(super) maximum_buffer_bytes: usize,
    pub(super) maximum_open_runs: usize,
    pub(super) write_buffer_bytes: usize,
    pub(super) maximum_record_bytes: usize,
    pub(super) maximum_lemma_transitions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TransitionInductionManifestV1 {
    pub(super) classified_transitions_path: PathBuf,
    pub(super) paradigms_path: PathBuf,
    pub(super) lemma_bindings_path: PathBuf,
    pub(super) compatibility_index_path: PathBuf,
    pub(super) paradigm_postings_path: PathBuf,
    pub(super) axis_schema_sha256: [u8; 32],
    pub(super) transition_observations: u64,
    pub(super) transferable_observations: u64,
    pub(super) exact_allomorph_observations: u64,
    pub(super) paradigm_count: u32,
    pub(super) bound_lemma_count: u32,
    pub(super) compatibility_index_count: u32,
    pub(super) compatibility_posting_count: u64,
    pub(super) maximum_program_operations: u16,
    pub(super) anchor_recovery: Option<AnchorRecoveryManifestV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ClassifiedTransitionV1 {
    pub(super) lemma_id: u32,
    pub(super) source_form_ref: u32,
    pub(super) target_form_ref: u32,
    pub(super) source_slot_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) variant_id: u16,
    pub(super) target_support: u32,
    pub(super) source_surface: String,
    pub(super) target_surface: String,
    pub(super) transition: ParadigmTransitionKeyV1,
    pub(super) transfer_lemma_support: u32,
}

impl ClassifiedTransitionV1 {
    pub(super) fn template(&self) -> EditTemplateV1 {
        let transferable = self.transfer_lemma_support >= 2;
        let operations = if transferable {
            self.transition
                .operations
                .iter()
                .cloned()
                .chain([EditOperationV1::Terminate {
                    slot_id: self.target_slot_id,
                    variant_id: self.variant_id,
                }])
                .collect()
        } else {
            vec![
                EditOperationV1::EmitExactAllomorph {
                    form_ref: self.target_form_ref,
                },
                EditOperationV1::Terminate {
                    slot_id: self.target_slot_id,
                    variant_id: self.variant_id,
                },
            ]
        };
        EditTemplateV1 {
            source_slot_id: self.source_slot_id,
            target_slot_id: self.target_slot_id,
            source_slot: self.transition.source_slot,
            target_slot: self.transition.target_slot,
            variant_id: self.variant_id,
            operations,
            transferable,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TransitionObservationV1 {
    lemma_id: u32,
    source_form_ref: u32,
    target_form_ref: u32,
    source_slot_id: u32,
    target_slot_id: u32,
    variant_id: u16,
    target_support: u32,
    source_surface: String,
    target_surface: String,
    transition: ParadigmTransitionKeyV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ParadigmDefinitionV1 {
    pub(super) paradigm_id: u32,
    pub(super) lemma_support: u32,
    pub(super) signature: ParadigmSignatureV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LemmaParadigmAssignmentV1 {
    pub(super) lemma_id: u32,
    pub(super) pos_domain: u8,
    pub(super) paradigm_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ParadigmCompatibilityIndexV1 {
    pub(super) pos_domain: u8,
    pub(super) source_slot_id: u32,
    pub(super) posting_start: u64,
    pub(super) posting_count: u32,
}

pub(super) fn induce_transition_field(
    reduced: &ReducedMorphologyManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    config: &TransitionReduceConfigV1,
) -> Result<TransitionInductionManifestV1, String> {
    validate_config(config)?;
    fs::create_dir_all(&config.root).map_err(|error| error.to_string())?;
    let axis_schema_sha256 = axis_schema.validate_for_slots(&reduced.morphology_slots)?;
    let slot_ids = reduced
        .morphology_slots
        .iter()
        .copied()
        .enumerate()
        .map(|(index, slot)| (slot, index as u32 + 1))
        .collect::<BTreeMap<_, _>>();

    let raw_transitions = config.root.join("transitions-raw.p2t");
    let (transition_observations, maximum_program_operations) =
        emit_transitions(reduced, axis_schema, &slot_ids, &raw_transitions, config)?;
    let sorted_transitions = config.root.join("transitions-by-key.p2t");
    external_sort_wire(
        &raw_transitions,
        &sorted_transitions,
        TRANSITION_MAGIC,
        "transition-key",
        config,
        |payload| {
            let transition = decode_transition(payload)?;
            Ok(transition_sort_key(&transition))
        },
    )?;

    let support_path = config.root.join("transition-support.p2s");
    reduce_transition_support(&sorted_transitions, &support_path, config)?;
    let classified_by_key = config.root.join("classified-by-key.p2c");
    let (transferable_observations, exact_allomorph_observations) = join_transition_support(
        &sorted_transitions,
        &support_path,
        &classified_by_key,
        config,
    )?;
    let classified_by_lemma = config.root.join("classified-by-lemma.p2c");
    external_sort_wire(
        &classified_by_key,
        &classified_by_lemma,
        CLASSIFIED_MAGIC,
        "transition-lemma",
        config,
        |payload| {
            let transition = decode_classified(payload)?;
            Ok(classified_lemma_sort_key(&transition))
        },
    )?;

    let raw_signatures = config.root.join("lemma-signatures-raw.p2g");
    emit_lemma_signatures(&classified_by_lemma, &raw_signatures, config)?;
    let sorted_signatures = config.root.join("lemma-signatures-by-key.p2g");
    external_sort_wire(
        &raw_signatures,
        &sorted_signatures,
        SIGNATURE_MAGIC,
        "signature-key",
        config,
        signature_sort_key,
    )?;
    let paradigms_path = config.root.join("paradigms.p2p");
    let raw_bindings = config.root.join("lemma-bindings-raw.p2b");
    let (paradigm_count, bound_lemma_count) =
        reduce_paradigms(&sorted_signatures, &paradigms_path, &raw_bindings, config)?;
    let lemma_bindings_path = config.root.join("lemma-bindings.p2b");
    external_sort_wire(
        &raw_bindings,
        &lemma_bindings_path,
        BINDING_MAGIC,
        "binding-lemma",
        config,
        |payload| {
            let assignment = decode_binding(payload)?;
            let mut key = assignment.lemma_id.to_be_bytes().to_vec();
            key.push(assignment.pos_domain);
            Ok(key)
        },
    )?;

    let raw_compatibility = config.root.join("compatibility-raw.p2k");
    emit_compatibility_observations(&paradigms_path, &raw_compatibility, &slot_ids, config)?;
    let sorted_compatibility = config.root.join("compatibility-sorted.p2k");
    external_sort_wire(
        &raw_compatibility,
        &sorted_compatibility,
        COMPATIBILITY_OBSERVATION_MAGIC,
        "compatibility-key",
        config,
        |payload| {
            let (pos, slot_id, paradigm_id) = decode_compatibility_observation(payload)?;
            let mut key = vec![pos];
            key.extend_from_slice(&slot_id.to_be_bytes());
            key.extend_from_slice(&paradigm_id.to_be_bytes());
            Ok(key)
        },
    )?;
    let compatibility_index_path = config.root.join("compatibility-index.p2x");
    let paradigm_postings_path = config.root.join("paradigm-postings.p2o");
    let (compatibility_index_count, compatibility_posting_count) = reduce_compatibility_index(
        &sorted_compatibility,
        &compatibility_index_path,
        &paradigm_postings_path,
        config,
    )?;

    let mut manifest = TransitionInductionManifestV1 {
        classified_transitions_path: classified_by_lemma,
        paradigms_path,
        lemma_bindings_path,
        compatibility_index_path,
        paradigm_postings_path,
        axis_schema_sha256,
        transition_observations,
        transferable_observations,
        exact_allomorph_observations,
        paradigm_count,
        bound_lemma_count,
        compatibility_index_count,
        compatibility_posting_count,
        maximum_program_operations: maximum_program_operations.max(2),
        anchor_recovery: None,
    };
    manifest.anchor_recovery = Some(induce_anchor_recovery_field(
        &manifest,
        axis_schema,
        config,
    )?);
    Ok(manifest)
}

pub(super) fn reopen_transition_induction(
    reduced: &ReducedMorphologyManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    root: &Path,
    maximum_record_bytes: usize,
) -> Result<TransitionInductionManifestV1, String> {
    if maximum_record_bytes == 0 {
        return Err("productive induction reopen has no record bound".to_string());
    }
    let classified_transitions_path = root.join("classified-by-lemma.p2c");
    let paradigms_path = root.join("paradigms.p2p");
    let lemma_bindings_path = root.join("lemma-bindings.p2b");
    let compatibility_index_path = root.join("compatibility-index.p2x");
    let paradigm_postings_path = root.join("paradigm-postings.p2o");
    for path in [
        &classified_transitions_path,
        &paradigms_path,
        &lemma_bindings_path,
        &compatibility_index_path,
        &paradigm_postings_path,
    ] {
        if !path.is_file() {
            return Err(format!(
                "productive induction reopen is missing {}",
                path.display()
            ));
        }
    }

    let mut classified =
        ClassifiedTransitionReaderV1::open(&classified_transitions_path, maximum_record_bytes)?;
    let mut transition_observations = 0_u64;
    let mut transferable_observations = 0_u64;
    let mut exact_allomorph_observations = 0_u64;
    let mut maximum_program_operations = 0_u16;
    while let Some(row) = classified.next()? {
        transition_observations = transition_observations
            .checked_add(1)
            .ok_or_else(|| "productive reopened transition count overflow".to_string())?;
        if row.transfer_lemma_support >= 2 {
            transferable_observations = transferable_observations
                .checked_add(1)
                .ok_or_else(|| "productive reopened transfer count overflow".to_string())?;
        } else {
            exact_allomorph_observations = exact_allomorph_observations
                .checked_add(1)
                .ok_or_else(|| "productive reopened exact count overflow".to_string())?;
        }
        maximum_program_operations = maximum_program_operations.max(
            u16::try_from(row.template().operations.len())
                .map_err(|_| "productive reopened program exceeds u16".to_string())?,
        );
    }

    let mut paradigm_reader =
        ParadigmDefinitionReaderV1::open(&paradigms_path, maximum_record_bytes)?;
    let mut paradigm_count = 0_u32;
    while let Some(paradigm) = paradigm_reader.next()? {
        paradigm_count = paradigm_count
            .checked_add(1)
            .ok_or_else(|| "productive reopened paradigm count overflow".to_string())?;
        if paradigm.paradigm_id != paradigm_count {
            return Err("productive reopened paradigms are not contiguous".to_string());
        }
    }

    let mut binding_reader =
        LemmaParadigmAssignmentReaderV1::open(&lemma_bindings_path, maximum_record_bytes)?;
    let mut bound_lemma_count = 0_u32;
    while binding_reader.next()?.is_some() {
        bound_lemma_count = bound_lemma_count
            .checked_add(1)
            .ok_or_else(|| "productive reopened binding count overflow".to_string())?;
    }

    let mut compatibility_reader =
        ParadigmCompatibilityIndexReaderV1::open(&compatibility_index_path, maximum_record_bytes)?;
    let mut compatibility_index_count = 0_u32;
    while compatibility_reader.next()?.is_some() {
        compatibility_index_count = compatibility_index_count
            .checked_add(1)
            .ok_or_else(|| "productive reopened compatibility count overflow".to_string())?;
    }
    let mut posting_reader =
        ParadigmPostingReaderV1::open(&paradigm_postings_path, maximum_record_bytes)?;
    let mut compatibility_posting_count = 0_u64;
    while posting_reader.next()?.is_some() {
        compatibility_posting_count = compatibility_posting_count
            .checked_add(1)
            .ok_or_else(|| "productive reopened posting count overflow".to_string())?;
    }

    let recovery_path = root.join("anchor-recovery-definitions.p2r");
    let anchor_recovery = recovery_path
        .is_file()
        .then(|| reopen_anchor_recovery_field(&recovery_path))
        .transpose()?;
    Ok(TransitionInductionManifestV1 {
        classified_transitions_path,
        paradigms_path,
        lemma_bindings_path,
        compatibility_index_path,
        paradigm_postings_path,
        axis_schema_sha256: axis_schema.validate_for_slots(&reduced.morphology_slots)?,
        transition_observations,
        transferable_observations,
        exact_allomorph_observations,
        paradigm_count,
        bound_lemma_count,
        compatibility_index_count,
        compatibility_posting_count,
        maximum_program_operations: maximum_program_operations.max(2),
        anchor_recovery,
    })
}

fn validate_config(config: &TransitionReduceConfigV1) -> Result<(), String> {
    if config.maximum_buffer_bytes < WIRE_ACCOUNTING_BYTES
        || config.maximum_open_runs < 2
        || config.write_buffer_bytes < WIRE_HEADER_BYTES
        || config.maximum_record_bytes < WIRE_ACCOUNTING_BYTES
        || config.maximum_record_bytes > config.maximum_buffer_bytes
        || config.maximum_lemma_transitions == 0
    {
        return Err("productive transition reduce has an invalid bounded budget".to_string());
    }
    Ok(())
}

fn emit_transitions(
    reduced: &ReducedMorphologyManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    slot_ids: &BTreeMap<MorphologySlotKeyV1, u32>,
    output_path: &Path,
    config: &TransitionReduceConfigV1,
) -> Result<(u64, u16), String> {
    let mut reader = ReducedLemmaReaderV1::open(&reduced.path)?;
    let mut writer = WireWriterV1::create(
        output_path,
        TRANSITION_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut count = 0_u64;
    let mut maximum_program_operations = 0_u16;
    while let Some(lemma) = reader.next_lemma()? {
        if lemma.forms.len() > config.maximum_lemma_transitions {
            return Err("productive lemma exceeds the transition-count budget".to_string());
        }
        let mut forms_by_pos = BTreeMap::<u8, Vec<CanonicalFormObservationV1>>::new();
        for form in &lemma.forms {
            let pos_domain = form.slot.pos_domain();
            let applicability = axis_schema.applicability(pos_domain)?;
            form.slot
                .validate(applicability)
                .map_err(|error| error.to_string())?;
            let slot_id = *slot_ids
                .get(&form.slot)
                .ok_or_else(|| "productive reduced form references an unknown slot".to_string())?;
            forms_by_pos
                .entry(pos_domain)
                .or_default()
                .push(CanonicalFormObservationV1 {
                    form_ref: form.form_ref,
                    slot_id,
                    slot: form.slot,
                    applicability,
                    normalized_surface: form.normalized_surface.clone(),
                    support: form.support,
                    // Reduced form identity is canonical and only breaks a comparator tie
                    // after slot and surface identity, where two reduced forms cannot differ.
                    provenance_id: form.form_ref,
                    variant_id: form.variant_id,
                });
        }
        for forms in forms_by_pos.values() {
            let anchor = select_canonical_anchor(forms).map_err(str::to_string)?;
            for target in forms {
                let template = derive_edit_template(anchor, target).map_err(str::to_string)?;
                maximum_program_operations = maximum_program_operations.max(
                    u16::try_from(template.operations.len()).map_err(|_| {
                        "productive program operation count exceeds u16".to_string()
                    })?,
                );
                let observation = TransitionObservationV1 {
                    lemma_id: lemma.lemma_id,
                    source_form_ref: anchor.form_ref,
                    target_form_ref: target.form_ref,
                    source_slot_id: anchor.slot_id,
                    target_slot_id: target.slot_id,
                    variant_id: target.variant_id,
                    target_support: target.support,
                    source_surface: anchor.normalized_surface.clone(),
                    target_surface: target.normalized_surface.clone(),
                    transition: template.transition_key(),
                };
                writer.write(&encode_transition(&observation)?)?;
                count = count
                    .checked_add(1)
                    .ok_or_else(|| "productive transition count overflow".to_string())?;
            }
        }
    }
    if reader.payload_sha256() != reduced.payload_sha256 {
        return Err("productive reduced lemma replay hash mismatch".to_string());
    }
    if writer.finish()? != count {
        return Err("productive transition writer count mismatch".to_string());
    }
    Ok((count, maximum_program_operations))
}

fn reduce_transition_support(
    sorted_path: &Path,
    output_path: &Path,
    config: &TransitionReduceConfigV1,
) -> Result<(), String> {
    let mut reader =
        WireReaderV1::open(sorted_path, TRANSITION_MAGIC, config.maximum_record_bytes)?;
    let mut writer = WireWriterV1::create(
        output_path,
        SUPPORT_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut current_key: Option<Vec<u8>> = None;
    let mut previous_lemma = None;
    let mut support = 0_u32;
    while let Some(payload) = reader.next()? {
        let observation = decode_transition(&payload)?;
        let key = observation.transition.canonical_bytes();
        if current_key.as_ref().is_some_and(|current| current != &key) {
            writer.write(&encode_support(
                current_key.take().expect("support key"),
                support,
            )?)?;
            previous_lemma = None;
            support = 0;
        }
        if current_key.is_none() {
            current_key = Some(key);
        }
        if previous_lemma != Some(observation.lemma_id) {
            support = support
                .checked_add(1)
                .ok_or_else(|| "productive transition support exceeds u32".to_string())?;
            previous_lemma = Some(observation.lemma_id);
        }
    }
    if let Some(key) = current_key {
        writer.write(&encode_support(key, support)?)?;
    }
    writer.finish()?;
    Ok(())
}

fn join_transition_support(
    transitions_path: &Path,
    support_path: &Path,
    output_path: &Path,
    config: &TransitionReduceConfigV1,
) -> Result<(u64, u64), String> {
    let mut transitions = WireReaderV1::open(
        transitions_path,
        TRANSITION_MAGIC,
        config.maximum_record_bytes,
    )?;
    let mut supports =
        WireReaderV1::open(support_path, SUPPORT_MAGIC, config.maximum_record_bytes)?;
    let mut current_support = supports
        .next()?
        .map(|payload| decode_support(&payload))
        .transpose()?;
    let mut writer = WireWriterV1::create(
        output_path,
        CLASSIFIED_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut transferable = 0_u64;
    let mut exact = 0_u64;
    while let Some(payload) = transitions.next()? {
        let observation = decode_transition(&payload)?;
        let key = observation.transition.canonical_bytes();
        while current_support
            .as_ref()
            .is_some_and(|(support_key, _)| support_key < &key)
        {
            current_support = supports
                .next()?
                .map(|payload| decode_support(&payload))
                .transpose()?;
        }
        let support = current_support
            .as_ref()
            .filter(|(support_key, _)| support_key == &key)
            .map(|(_, support)| *support)
            .ok_or_else(|| "productive transition support merge join lost a key".to_string())?;
        let classified = ClassifiedTransitionV1 {
            lemma_id: observation.lemma_id,
            source_form_ref: observation.source_form_ref,
            target_form_ref: observation.target_form_ref,
            source_slot_id: observation.source_slot_id,
            target_slot_id: observation.target_slot_id,
            variant_id: observation.variant_id,
            target_support: observation.target_support,
            source_surface: observation.source_surface,
            target_surface: observation.target_surface,
            transition: observation.transition,
            transfer_lemma_support: support,
        };
        if support >= 2 {
            transferable += 1;
        } else {
            exact += 1;
        }
        writer.write(&encode_classified(&classified)?)?;
    }
    if supports.next()?.is_some() {
        return Err("productive transition support merge join left unmatched keys".to_string());
    }
    writer.finish()?;
    Ok((transferable, exact))
}

fn emit_lemma_signatures(
    classified_path: &Path,
    output_path: &Path,
    config: &TransitionReduceConfigV1,
) -> Result<(), String> {
    let mut reader = WireReaderV1::open(
        classified_path,
        CLASSIFIED_MAGIC,
        config.maximum_record_bytes,
    )?;
    let mut writer = WireWriterV1::create(
        output_path,
        SIGNATURE_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut current_basin = None;
    let mut transitions = BTreeSet::new();
    let mut count = 0_usize;
    while let Some(payload) = reader.next()? {
        let classified = decode_classified(&payload)?;
        let pos = classified.transition.source_slot.pos_domain();
        if classified.transition.target_slot.pos_domain() != pos {
            return Err("productive classified transition crosses POS domains".to_string());
        }
        let basin = (classified.lemma_id, pos);
        if current_basin.is_some_and(|current| basin != current) {
            let (lemma_id, pos_domain) = current_basin.expect("current classified basin");
            write_signature(&mut writer, lemma_id, pos_domain, &transitions)?;
            transitions.clear();
            count = 0;
        }
        current_basin = Some(basin);
        count += 1;
        if count > config.maximum_lemma_transitions {
            return Err("productive classified lemma exceeds its transition budget".to_string());
        }
        if classified.transfer_lemma_support >= 2 {
            transitions.insert(classified.transition);
        }
    }
    if let Some((lemma_id, pos_domain)) = current_basin {
        write_signature(&mut writer, lemma_id, pos_domain, &transitions)?;
    }
    writer.finish()?;
    Ok(())
}

fn write_signature(
    writer: &mut WireWriterV1,
    lemma_id: u32,
    pos: u8,
    transitions: &BTreeSet<ParadigmTransitionKeyV1>,
) -> Result<(), String> {
    if transitions.is_empty() {
        return Ok(());
    }
    let signature =
        ParadigmSignatureV1::new(pos, transitions.iter().cloned()).map_err(str::to_string)?;
    writer.write(&encode_signature_observation(lemma_id, &signature)?)?;
    Ok(())
}

fn reduce_paradigms(
    signatures_path: &Path,
    paradigms_path: &Path,
    raw_bindings_path: &Path,
    config: &TransitionReduceConfigV1,
) -> Result<(u32, u32), String> {
    let mut reader = WireReaderV1::open(
        signatures_path,
        SIGNATURE_MAGIC,
        config.maximum_record_bytes,
    )?;
    let mut paradigms = WireWriterV1::create(
        paradigms_path,
        PARADIGM_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut bindings = WireWriterV1::create(
        raw_bindings_path,
        BINDING_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut current_key: Option<Vec<u8>> = None;
    let mut current_signature: Option<ParadigmSignatureV1> = None;
    let mut current_support = 0_u32;
    let mut previous_lemma = None;
    let mut paradigm_id = 0_u32;
    let mut bound_lemma_count = 0_u32;
    while let Some(payload) = reader.next()? {
        let (lemma_id, signature) = decode_signature_observation(&payload)?;
        let pos_domain = signature.pos_domain;
        let key = signature_bytes(&signature)?;
        if current_key.as_ref().is_some_and(|current| current != &key) {
            flush_paradigm(
                &mut paradigms,
                paradigm_id,
                current_signature.take().expect("paradigm signature"),
                current_support,
            )?;
            current_key = None;
            current_support = 0;
            previous_lemma = None;
        }
        if current_key.is_none() {
            paradigm_id = paradigm_id
                .checked_add(1)
                .ok_or_else(|| "productive paradigm identity overflow".to_string())?;
            current_key = Some(key);
            current_signature = Some(signature);
        }
        if previous_lemma.is_some_and(|previous| lemma_id <= previous) {
            return Err(
                "productive paradigm lemma support is repeated or noncanonical".to_string(),
            );
        }
        bindings.write(&encode_binding(LemmaParadigmAssignmentV1 {
            lemma_id,
            pos_domain,
            paradigm_id,
        }))?;
        current_support = current_support
            .checked_add(1)
            .ok_or_else(|| "productive paradigm support exceeds u32".to_string())?;
        bound_lemma_count = bound_lemma_count
            .checked_add(1)
            .ok_or_else(|| "productive bound lemma count overflow".to_string())?;
        previous_lemma = Some(lemma_id);
    }
    if let Some(signature) = current_signature {
        flush_paradigm(&mut paradigms, paradigm_id, signature, current_support)?;
    }
    paradigms.finish()?;
    bindings.finish()?;
    Ok((paradigm_id, bound_lemma_count))
}

fn flush_paradigm(
    paradigms: &mut WireWriterV1,
    paradigm_id: u32,
    signature: ParadigmSignatureV1,
    lemma_support: u32,
) -> Result<(), String> {
    if paradigm_id == 0 || lemma_support == 0 {
        return Err("productive paradigm identity or support is zero".to_string());
    }
    paradigms.write(&encode_paradigm(&ParadigmDefinitionV1 {
        paradigm_id,
        lemma_support,
        signature,
    })?)?;
    Ok(())
}

fn emit_compatibility_observations(
    paradigms_path: &Path,
    output_path: &Path,
    slot_ids: &BTreeMap<MorphologySlotKeyV1, u32>,
    config: &TransitionReduceConfigV1,
) -> Result<(), String> {
    let mut reader = ParadigmDefinitionReaderV1::open(paradigms_path, config.maximum_record_bytes)?;
    let mut writer = WireWriterV1::create(
        output_path,
        COMPATIBILITY_OBSERVATION_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    while let Some(paradigm) = reader.next()? {
        let source_slots = paradigm
            .signature
            .transitions
            .iter()
            .map(|transition| transition.source_slot)
            .collect::<BTreeSet<_>>();
        for source_slot in source_slots {
            let source_slot_id = *slot_ids
                .get(&source_slot)
                .ok_or_else(|| "productive paradigm source slot has no canonical ID".to_string())?;
            writer.write(&encode_compatibility_observation(
                paradigm.signature.pos_domain,
                source_slot_id,
                paradigm.paradigm_id,
            ))?;
        }
    }
    writer.finish()?;
    Ok(())
}

fn reduce_compatibility_index(
    observations_path: &Path,
    index_path: &Path,
    postings_path: &Path,
    config: &TransitionReduceConfigV1,
) -> Result<(u32, u64), String> {
    let mut observations = WireReaderV1::open(
        observations_path,
        COMPATIBILITY_OBSERVATION_MAGIC,
        config.maximum_record_bytes,
    )?;
    let mut indexes = WireWriterV1::create(
        index_path,
        COMPATIBILITY_INDEX_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut postings = WireWriterV1::create(
        postings_path,
        PARADIGM_POSTING_MAGIC,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut current: Option<(u8, u32)> = None;
    let mut posting_start = 0_u64;
    let mut posting_count = 0_u32;
    let mut index_count = 0_u32;
    let mut total_postings = 0_u64;
    let mut previous_paradigm = 0_u32;
    while let Some(payload) = observations.next()? {
        let (pos, slot_id, paradigm_id) = decode_compatibility_observation(&payload)?;
        let key = (pos, slot_id);
        if current.is_some_and(|current| current != key) {
            indexes.write(&encode_compatibility_index(ParadigmCompatibilityIndexV1 {
                pos_domain: current.expect("compatibility key").0,
                source_slot_id: current.expect("compatibility key").1,
                posting_start,
                posting_count,
            }))?;
            index_count = index_count
                .checked_add(1)
                .ok_or_else(|| "productive compatibility index count overflow".to_string())?;
            posting_start = total_postings;
            posting_count = 0;
            previous_paradigm = 0;
        }
        if current != Some(key) {
            current = Some(key);
        }
        if paradigm_id <= previous_paradigm {
            return Err("productive compatibility postings are repeated or unsorted".to_string());
        }
        postings.write(&paradigm_id.to_le_bytes())?;
        posting_count = posting_count
            .checked_add(1)
            .ok_or_else(|| "productive compatibility posting range exceeds u32".to_string())?;
        total_postings = total_postings
            .checked_add(1)
            .ok_or_else(|| "productive compatibility posting count overflow".to_string())?;
        previous_paradigm = paradigm_id;
    }
    if let Some((pos_domain, source_slot_id)) = current {
        indexes.write(&encode_compatibility_index(ParadigmCompatibilityIndexV1 {
            pos_domain,
            source_slot_id,
            posting_start,
            posting_count,
        }))?;
        index_count = index_count
            .checked_add(1)
            .ok_or_else(|| "productive compatibility index count overflow".to_string())?;
    }
    if indexes.finish()? != u64::from(index_count) || postings.finish()? != total_postings {
        return Err("productive compatibility output denominator mismatch".to_string());
    }
    Ok((index_count, total_postings))
}

fn encode_compatibility_observation(pos: u8, slot_id: u32, paradigm_id: u32) -> Vec<u8> {
    let mut bytes = vec![pos, 0, 0, 0];
    bytes.extend_from_slice(&slot_id.to_le_bytes());
    bytes.extend_from_slice(&paradigm_id.to_le_bytes());
    bytes
}

fn decode_compatibility_observation(bytes: &[u8]) -> Result<(u8, u32, u32), String> {
    if bytes.len() != 12 || bytes[1..4] != [0; 3] {
        return Err("productive compatibility observation width or flags are invalid".to_string());
    }
    let pos = bytes[0];
    let slot_id = u32::from_le_bytes(bytes[4..8].try_into().expect("fixed slot ID"));
    let paradigm_id = u32::from_le_bytes(bytes[8..12].try_into().expect("fixed paradigm ID"));
    if pos < 2 || slot_id == 0 || paradigm_id == 0 {
        return Err("productive compatibility observation contains zero".to_string());
    }
    Ok((pos, slot_id, paradigm_id))
}

fn encode_compatibility_index(index: ParadigmCompatibilityIndexV1) -> Vec<u8> {
    let mut bytes = vec![index.pos_domain, 0, 0, 0];
    bytes.extend_from_slice(&index.source_slot_id.to_le_bytes());
    bytes.extend_from_slice(&index.posting_start.to_le_bytes());
    bytes.extend_from_slice(&index.posting_count.to_le_bytes());
    bytes
}

fn decode_compatibility_index(bytes: &[u8]) -> Result<ParadigmCompatibilityIndexV1, String> {
    if bytes.len() != 20 || bytes[1..4] != [0; 3] {
        return Err("productive compatibility index width or flags are invalid".to_string());
    }
    let index = ParadigmCompatibilityIndexV1 {
        pos_domain: bytes[0],
        source_slot_id: u32::from_le_bytes(bytes[4..8].try_into().expect("fixed slot ID")),
        posting_start: u64::from_le_bytes(bytes[8..16].try_into().expect("fixed posting start")),
        posting_count: u32::from_le_bytes(bytes[16..20].try_into().expect("fixed posting count")),
    };
    if index.pos_domain < 2 || index.source_slot_id == 0 || index.posting_count == 0 {
        return Err("productive compatibility index contains zero".to_string());
    }
    Ok(index)
}

pub(super) struct ClassifiedTransitionReaderV1 {
    reader: WireReaderV1,
}

impl ClassifiedTransitionReaderV1 {
    pub(super) fn open(path: &Path, maximum_record_bytes: usize) -> Result<Self, String> {
        Ok(Self {
            reader: WireReaderV1::open(path, CLASSIFIED_MAGIC, maximum_record_bytes)?,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<ClassifiedTransitionV1>, String> {
        self.reader
            .next()?
            .map(|payload| decode_classified(&payload))
            .transpose()
    }
}

pub(super) struct ParadigmDefinitionReaderV1 {
    reader: WireReaderV1,
}

impl ParadigmDefinitionReaderV1 {
    pub(super) fn open(path: &Path, maximum_record_bytes: usize) -> Result<Self, String> {
        Ok(Self {
            reader: WireReaderV1::open(path, PARADIGM_MAGIC, maximum_record_bytes)?,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<ParadigmDefinitionV1>, String> {
        self.reader
            .next()?
            .map(|payload| decode_paradigm(&payload))
            .transpose()
    }
}

pub(super) struct LemmaParadigmAssignmentReaderV1 {
    reader: WireReaderV1,
}

impl LemmaParadigmAssignmentReaderV1 {
    pub(super) fn open(path: &Path, maximum_record_bytes: usize) -> Result<Self, String> {
        Ok(Self {
            reader: WireReaderV1::open(path, BINDING_MAGIC, maximum_record_bytes)?,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<LemmaParadigmAssignmentV1>, String> {
        self.reader
            .next()?
            .map(|payload| decode_binding(&payload))
            .transpose()
    }
}

pub(super) struct ParadigmCompatibilityIndexReaderV1 {
    reader: WireReaderV1,
}

impl ParadigmCompatibilityIndexReaderV1 {
    pub(super) fn open(path: &Path, maximum_record_bytes: usize) -> Result<Self, String> {
        Ok(Self {
            reader: WireReaderV1::open(path, COMPATIBILITY_INDEX_MAGIC, maximum_record_bytes)?,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<ParadigmCompatibilityIndexV1>, String> {
        self.reader
            .next()?
            .map(|payload| decode_compatibility_index(&payload))
            .transpose()
    }
}

pub(super) struct ParadigmPostingReaderV1 {
    reader: WireReaderV1,
}

impl ParadigmPostingReaderV1 {
    pub(super) fn open(path: &Path, maximum_record_bytes: usize) -> Result<Self, String> {
        Ok(Self {
            reader: WireReaderV1::open(path, PARADIGM_POSTING_MAGIC, maximum_record_bytes)?,
        })
    }

    pub(super) fn next(&mut self) -> Result<Option<u32>, String> {
        self.reader
            .next()?
            .map(|payload| {
                if payload.len() != 4 {
                    return Err("productive paradigm posting width is invalid".to_string());
                }
                let paradigm_id = u32::from_le_bytes(payload.try_into().expect("fixed posting"));
                if paradigm_id == 0 {
                    return Err("productive paradigm posting is zero".to_string());
                }
                Ok(paradigm_id)
            })
            .transpose()
    }
}

#[derive(Clone, Debug)]
struct SortItemV1 {
    key: Vec<u8>,
    payload: Vec<u8>,
}

fn external_sort_wire(
    input_path: &Path,
    output_path: &Path,
    magic: [u8; 4],
    label: &str,
    config: &TransitionReduceConfigV1,
    key_fn: impl Fn(&[u8]) -> Result<Vec<u8>, String> + Copy,
) -> Result<u64, String> {
    let mut reader = WireReaderV1::open(input_path, magic, config.maximum_record_bytes)?;
    let mut chunk = Vec::new();
    let mut chunk_bytes = 0_usize;
    let mut runs = Vec::new();
    while let Some(payload) = reader.next()? {
        let key = key_fn(&payload)?;
        let accounted = WIRE_ACCOUNTING_BYTES
            .checked_add(key.len())
            .and_then(|bytes| bytes.checked_add(payload.len()))
            .ok_or_else(|| "productive wire sort accounting overflow".to_string())?;
        if accounted > config.maximum_buffer_bytes {
            return Err("productive wire record exceeds the sort buffer budget".to_string());
        }
        if !chunk.is_empty() && chunk_bytes + accounted > config.maximum_buffer_bytes {
            runs.push(write_wire_run(
                &config.root,
                label,
                runs.len(),
                magic,
                &mut chunk,
                config,
            )?);
            chunk_bytes = 0;
        }
        chunk_bytes += accounted;
        chunk.push(SortItemV1 { key, payload });
    }
    if !chunk.is_empty() {
        runs.push(write_wire_run(
            &config.root,
            label,
            runs.len(),
            magic,
            &mut chunk,
            config,
        )?);
    }
    merge_wire_runs(runs, output_path, magic, label, config, key_fn)
}

fn write_wire_run(
    root: &Path,
    label: &str,
    index: usize,
    magic: [u8; 4],
    chunk: &mut Vec<SortItemV1>,
    config: &TransitionReduceConfigV1,
) -> Result<PathBuf, String> {
    chunk.sort_unstable_by(sort_item_order);
    chunk.dedup_by(|left, right| left.payload == right.payload);
    let path = root.join(format!("{label}-run-{index:08}.bin"));
    let mut writer = WireWriterV1::create(
        &path,
        magic,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    for item in chunk.iter() {
        writer.write(&item.payload)?;
    }
    writer.finish()?;
    chunk.clear();
    Ok(path)
}

fn merge_wire_runs(
    mut runs: Vec<PathBuf>,
    output_path: &Path,
    magic: [u8; 4],
    label: &str,
    config: &TransitionReduceConfigV1,
    key_fn: impl Fn(&[u8]) -> Result<Vec<u8>, String> + Copy,
) -> Result<u64, String> {
    if runs.is_empty() {
        return WireWriterV1::create(
            output_path,
            magic,
            config.write_buffer_bytes,
            config.maximum_record_bytes,
        )?
        .finish();
    }
    let mut pass = 0_usize;
    while runs.len() > 1 {
        let mut next = Vec::new();
        for (group, paths) in runs.chunks(config.maximum_open_runs).enumerate() {
            let path = config
                .root
                .join(format!("{label}-merge-{pass:04}-{group:08}.bin"));
            merge_wire_group(paths, &path, magic, config, key_fn)?;
            next.push(path);
        }
        for path in &runs {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
        runs = next;
        pass += 1;
    }
    if output_path.exists() {
        fs::remove_file(output_path).map_err(|error| error.to_string())?;
    }
    fs::rename(&runs[0], output_path).map_err(|error| error.to_string())?;
    count_wire_records(output_path, magic, config.maximum_record_bytes)
}

fn merge_wire_group(
    paths: &[PathBuf],
    output_path: &Path,
    magic: [u8; 4],
    config: &TransitionReduceConfigV1,
    key_fn: impl Fn(&[u8]) -> Result<Vec<u8>, String> + Copy,
) -> Result<(), String> {
    let mut readers = paths
        .iter()
        .map(|path| WireReaderV1::open(path, magic, config.maximum_record_bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let mut heads = readers
        .iter_mut()
        .map(|reader| {
            reader
                .next()?
                .map(|payload| {
                    Ok(SortItemV1 {
                        key: key_fn(&payload)?,
                        payload,
                    })
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut writer = WireWriterV1::create(
        output_path,
        magic,
        config.write_buffer_bytes,
        config.maximum_record_bytes,
    )?;
    let mut previous: Option<Vec<u8>> = None;
    while let Some(index) = heads
        .iter()
        .enumerate()
        .filter_map(|(index, item)| item.as_ref().map(|item| (index, item)))
        .min_by(|(_, left), (_, right)| sort_item_order(left, right))
        .map(|(index, _)| index)
    {
        let item = heads[index].take().expect("selected wire merge head");
        if previous.as_ref() != Some(&item.payload) {
            writer.write(&item.payload)?;
            previous = Some(item.payload);
        }
        heads[index] = readers[index]
            .next()?
            .map(|payload| {
                Ok::<SortItemV1, String>(SortItemV1 {
                    key: key_fn(&payload)?,
                    payload,
                })
            })
            .transpose()?;
    }
    writer.finish()?;
    Ok(())
}

fn sort_item_order(left: &SortItemV1, right: &SortItemV1) -> Ordering {
    left.key
        .cmp(&right.key)
        .then_with(|| left.payload.cmp(&right.payload))
}

struct WireWriterV1 {
    writer: BufWriter<File>,
    magic: [u8; 4],
    maximum_record_bytes: usize,
    count: u64,
}

impl WireWriterV1 {
    fn create(
        path: &Path,
        magic: [u8; 4],
        write_buffer_bytes: usize,
        maximum_record_bytes: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            writer: BufWriter::with_capacity(
                write_buffer_bytes,
                File::create(path).map_err(|error| error.to_string())?,
            ),
            magic,
            maximum_record_bytes,
            count: 0,
        })
    }

    fn write(&mut self, payload: &[u8]) -> Result<(), String> {
        if payload.is_empty() || payload.len() > self.maximum_record_bytes {
            return Err("productive wire payload is empty or exceeds its bound".to_string());
        }
        let payload_bytes = u32::try_from(payload.len())
            .map_err(|_| "productive wire payload exceeds u32".to_string())?;
        let mut header = [0_u8; WIRE_HEADER_BYTES];
        header[0..4].copy_from_slice(&self.magic);
        header[4..6].copy_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
        header[8..12].copy_from_slice(&payload_bytes.to_le_bytes());
        header[12..16].copy_from_slice(&crc32(payload).to_le_bytes());
        self.writer
            .write_all(&header)
            .and_then(|_| self.writer.write_all(payload))
            .map_err(|error| error.to_string())?;
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| "productive wire record count overflow".to_string())?;
        Ok(())
    }

    fn finish(mut self) -> Result<u64, String> {
        self.writer.flush().map_err(|error| error.to_string())?;
        Ok(self.count)
    }
}

struct WireReaderV1 {
    reader: BufReader<File>,
    magic: [u8; 4],
    maximum_record_bytes: usize,
}

impl WireReaderV1 {
    fn open(path: &Path, magic: [u8; 4], maximum_record_bytes: usize) -> Result<Self, String> {
        Ok(Self {
            reader: BufReader::new(File::open(path).map_err(|error| error.to_string())?),
            magic,
            maximum_record_bytes,
        })
    }

    fn next(&mut self) -> Result<Option<Vec<u8>>, String> {
        let mut header = [0_u8; WIRE_HEADER_BYTES];
        match self.reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(1) => self
                .reader
                .read_exact(&mut header[1..])
                .map_err(|_| "productive wire header is truncated".to_string())?,
            Ok(_) => unreachable!("one-byte wire read returned more than one byte"),
            Err(error) => return Err(error.to_string()),
        }
        if header[0..4] != self.magic
            || u16::from_le_bytes(header[4..6].try_into().expect("fixed slice"))
                != PRODUCTIVE_V1_SCHEMA_VERSION
            || header[6..8] != [0; 2]
        {
            return Err("productive wire magic, version, or flags are invalid".to_string());
        }
        let payload_bytes =
            u32::from_le_bytes(header[8..12].try_into().expect("fixed slice")) as usize;
        if payload_bytes == 0 || payload_bytes > self.maximum_record_bytes {
            return Err("productive wire payload exceeds its configured bound".to_string());
        }
        let expected_crc = u32::from_le_bytes(header[12..16].try_into().expect("fixed slice"));
        let mut payload = vec![0_u8; payload_bytes];
        self.reader
            .read_exact(&mut payload)
            .map_err(|_| "productive wire payload is truncated".to_string())?;
        if crc32(&payload) != expected_crc {
            return Err("productive wire payload CRC mismatch".to_string());
        }
        Ok(Some(payload))
    }
}

fn count_wire_records(path: &Path, magic: [u8; 4], maximum: usize) -> Result<u64, String> {
    let mut reader = WireReaderV1::open(path, magic, maximum)?;
    let mut count = 0_u64;
    while reader.next()?.is_some() {
        count += 1;
    }
    Ok(count)
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

fn encode_transition(observation: &TransitionObservationV1) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, &observation.transition.canonical_bytes())?;
    bytes.extend_from_slice(&observation.lemma_id.to_le_bytes());
    bytes.extend_from_slice(&observation.source_form_ref.to_le_bytes());
    bytes.extend_from_slice(&observation.target_form_ref.to_le_bytes());
    bytes.extend_from_slice(&observation.source_slot_id.to_le_bytes());
    bytes.extend_from_slice(&observation.target_slot_id.to_le_bytes());
    bytes.extend_from_slice(&observation.variant_id.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(&observation.target_support.to_le_bytes());
    push_bytes(&mut bytes, observation.source_surface.as_bytes())?;
    push_bytes(&mut bytes, observation.target_surface.as_bytes())?;
    Ok(bytes)
}

fn decode_transition(bytes: &[u8]) -> Result<TransitionObservationV1, String> {
    let mut input = InputV1::new(bytes);
    let transition = decode_transition_key(input.bytes_field()?)?;
    let lemma_id = input.u32()?;
    let source_form_ref = input.u32()?;
    let target_form_ref = input.u32()?;
    let source_slot_id = input.u32()?;
    let target_slot_id = input.u32()?;
    let variant_id = input.u16()?;
    if input.u16()? != 0 {
        return Err("productive transition reserved field is not zero".to_string());
    }
    let target_support = input.u32()?;
    let source_surface = input.string()?;
    let target_surface = input.string()?;
    if !input.is_empty()
        || source_slot_id == 0
        || target_slot_id == 0
        || variant_id == 0
        || target_support == 0
        || source_surface.is_empty()
        || target_surface.is_empty()
    {
        return Err("productive transition identity or payload is invalid".to_string());
    }
    Ok(TransitionObservationV1 {
        lemma_id,
        source_form_ref,
        target_form_ref,
        source_slot_id,
        target_slot_id,
        variant_id,
        target_support,
        source_surface,
        target_surface,
        transition,
    })
}

fn encode_classified(classified: &ClassifiedTransitionV1) -> Result<Vec<u8>, String> {
    let observation = TransitionObservationV1 {
        lemma_id: classified.lemma_id,
        source_form_ref: classified.source_form_ref,
        target_form_ref: classified.target_form_ref,
        source_slot_id: classified.source_slot_id,
        target_slot_id: classified.target_slot_id,
        variant_id: classified.variant_id,
        target_support: classified.target_support,
        source_surface: classified.source_surface.clone(),
        target_surface: classified.target_surface.clone(),
        transition: classified.transition.clone(),
    };
    let mut bytes = encode_transition(&observation)?;
    bytes.extend_from_slice(&classified.transfer_lemma_support.to_le_bytes());
    Ok(bytes)
}

fn decode_classified(bytes: &[u8]) -> Result<ClassifiedTransitionV1, String> {
    if bytes.len() < 4 {
        return Err("productive classified transition is truncated".to_string());
    }
    let split = bytes.len() - 4;
    let observation = decode_transition(&bytes[..split])?;
    let transfer_lemma_support =
        u32::from_le_bytes(bytes[split..].try_into().expect("fixed support"));
    if transfer_lemma_support == 0 {
        return Err("productive classified transition has zero support".to_string());
    }
    Ok(ClassifiedTransitionV1 {
        lemma_id: observation.lemma_id,
        source_form_ref: observation.source_form_ref,
        target_form_ref: observation.target_form_ref,
        source_slot_id: observation.source_slot_id,
        target_slot_id: observation.target_slot_id,
        variant_id: observation.variant_id,
        target_support: observation.target_support,
        source_surface: observation.source_surface,
        target_surface: observation.target_surface,
        transition: observation.transition,
        transfer_lemma_support,
    })
}

fn encode_support(key: Vec<u8>, support: u32) -> Result<Vec<u8>, String> {
    if support == 0 {
        return Err("productive transition support is zero".to_string());
    }
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, &key)?;
    bytes.extend_from_slice(&support.to_le_bytes());
    Ok(bytes)
}

fn decode_support(bytes: &[u8]) -> Result<(Vec<u8>, u32), String> {
    let mut input = InputV1::new(bytes);
    let key = input.bytes_field()?.to_vec();
    let support = input.u32()?;
    if !input.is_empty() || key.is_empty() || support == 0 {
        return Err("productive transition support payload is invalid".to_string());
    }
    Ok((key, support))
}

fn encode_signature_observation(
    lemma_id: u32,
    signature: &ParadigmSignatureV1,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, &signature_bytes(signature)?)?;
    bytes.extend_from_slice(&lemma_id.to_le_bytes());
    Ok(bytes)
}

fn decode_signature_observation(bytes: &[u8]) -> Result<(u32, ParadigmSignatureV1), String> {
    let mut input = InputV1::new(bytes);
    let signature = decode_signature(input.bytes_field()?)?;
    let lemma_id = input.u32()?;
    if !input.is_empty() {
        return Err("productive signature observation is invalid".to_string());
    }
    Ok((lemma_id, signature))
}

fn signature_sort_key(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let (lemma_id, signature) = decode_signature_observation(bytes)?;
    let mut key = signature_bytes(&signature)?;
    key.extend_from_slice(&lemma_id.to_be_bytes());
    Ok(key)
}

fn encode_paradigm(paradigm: &ParadigmDefinitionV1) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&paradigm.paradigm_id.to_le_bytes());
    bytes.extend_from_slice(&paradigm.lemma_support.to_le_bytes());
    push_bytes(&mut bytes, &signature_bytes(&paradigm.signature)?)?;
    Ok(bytes)
}

fn decode_paradigm(bytes: &[u8]) -> Result<ParadigmDefinitionV1, String> {
    let mut input = InputV1::new(bytes);
    let paradigm_id = input.u32()?;
    let lemma_support = input.u32()?;
    let signature = decode_signature(input.bytes_field()?)?;
    if !input.is_empty() || paradigm_id == 0 || lemma_support == 0 {
        return Err("productive paradigm definition is invalid".to_string());
    }
    Ok(ParadigmDefinitionV1 {
        paradigm_id,
        lemma_support,
        signature,
    })
}

fn encode_binding(binding: LemmaParadigmAssignmentV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12);
    bytes.extend_from_slice(&binding.lemma_id.to_le_bytes());
    bytes.push(binding.pos_domain);
    bytes.extend_from_slice(&[0; 3]);
    bytes.extend_from_slice(&binding.paradigm_id.to_le_bytes());
    bytes
}

fn decode_binding(bytes: &[u8]) -> Result<LemmaParadigmAssignmentV1, String> {
    if bytes.len() != 12 || bytes[5..8] != [0; 3] {
        return Err("productive lemma assignment width is invalid".to_string());
    }
    let binding = LemmaParadigmAssignmentV1 {
        lemma_id: u32::from_le_bytes(bytes[0..4].try_into().expect("fixed lemma ID")),
        pos_domain: bytes[4],
        paradigm_id: u32::from_le_bytes(bytes[8..12].try_into().expect("fixed paradigm ID")),
    };
    if binding.pos_domain < 2 || binding.paradigm_id == 0 {
        return Err("productive lemma assignment contains a zero paradigm".to_string());
    }
    Ok(binding)
}

fn transition_sort_key(observation: &TransitionObservationV1) -> Vec<u8> {
    let mut key = observation.transition.canonical_bytes();
    key.extend_from_slice(&observation.lemma_id.to_be_bytes());
    key.extend_from_slice(&observation.target_form_ref.to_be_bytes());
    key
}

fn classified_lemma_sort_key(classified: &ClassifiedTransitionV1) -> Vec<u8> {
    let mut key = classified.lemma_id.to_be_bytes().to_vec();
    key.push(classified.transition.source_slot.pos_domain());
    key.extend_from_slice(&classified.target_form_ref.to_be_bytes());
    key.extend_from_slice(&classified.transition.canonical_bytes());
    key
}

pub(super) fn signature_bytes(signature: &ParadigmSignatureV1) -> Result<Vec<u8>, String> {
    if signature.pos_domain < 2 || signature.transitions.is_empty() {
        return Err("productive paradigm signature is empty or lacks POS".to_string());
    }
    let mut bytes = vec![signature.pos_domain, 0, 0, 0];
    bytes.extend_from_slice(
        &u32::try_from(signature.transitions.len())
            .map_err(|_| "productive paradigm signature exceeds u32".to_string())?
            .to_le_bytes(),
    );
    for transition in &signature.transitions {
        push_bytes(&mut bytes, &transition.canonical_bytes())?;
    }
    Ok(bytes)
}

fn decode_signature(bytes: &[u8]) -> Result<ParadigmSignatureV1, String> {
    let mut input = InputV1::new(bytes);
    let pos = input.u8()?;
    if input.bytes(3)? != [0; 3] {
        return Err("productive paradigm signature reserved bytes are nonzero".to_string());
    }
    let count = input.u32()? as usize;
    let mut transitions = Vec::with_capacity(count);
    for _ in 0..count {
        transitions.push(decode_transition_key(input.bytes_field()?)?);
    }
    if !input.is_empty() || count == 0 {
        return Err("productive paradigm signature is empty or has a suffix".to_string());
    }
    ParadigmSignatureV1::new(pos, transitions).map_err(str::to_string)
}

pub(super) fn decode_transition_key(bytes: &[u8]) -> Result<ParadigmTransitionKeyV1, String> {
    let mut input = InputV1::new(bytes);
    let source_slot = MorphologySlotKeyV1::from_bytes(input.array()?).map_err(str::to_string)?;
    let target_slot = MorphologySlotKeyV1::from_bytes(input.array()?).map_err(str::to_string)?;
    let count = input.u32()? as usize;
    let mut operations = Vec::with_capacity(count);
    for _ in 0..count {
        operations.push(decode_edit_operation(input.bytes_field()?)?);
    }
    if !input.is_empty()
        || operations.is_empty()
        || operations
            .iter()
            .any(|operation| matches!(operation, EditOperationV1::Terminate { .. }))
    {
        return Err("productive transition key is empty or has an invalid suffix".to_string());
    }
    Ok(ParadigmTransitionKeyV1 {
        source_slot,
        target_slot,
        operations,
    })
}

fn decode_edit_operation(bytes: &[u8]) -> Result<EditOperationV1, String> {
    let mut input = InputV1::new(bytes);
    let opcode = input.u8()?;
    let operation = match opcode {
        1 => EditOperationV1::CopySourceRange {
            start_anchor: match input.u8()? {
                1 => SourceAnchorV1::Start,
                2 => SourceAnchorV1::End,
                _ => return Err("productive transition copy anchor is invalid".to_string()),
            },
            start_delta: input.i16()?,
            scalar_count: input.u16()?,
        },
        2 => EditOperationV1::DropSourcePrefix {
            scalar_count: input.u16()?,
        },
        3 => EditOperationV1::DropSourceSuffix {
            scalar_count: input.u16()?,
        },
        4 => EditOperationV1::EmitSegment {
            segment: input.string()?,
        },
        5 => EditOperationV1::ReplaceSourceRange {
            end_relative_offset: input.i16()?,
            delete_count: input.u16()?,
            segment: input.string()?,
        },
        6 => EditOperationV1::EmitExactAllomorph {
            form_ref: input.u32()?,
        },
        7 => EditOperationV1::Terminate {
            slot_id: input.u32()?,
            variant_id: input.u16()?,
        },
        _ => return Err("productive transition operation opcode is invalid".to_string()),
    };
    if !input.is_empty() {
        return Err("productive transition operation has a suffix".to_string());
    }
    Ok(operation)
}

fn push_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), String> {
    output.extend_from_slice(
        &u32::try_from(bytes.len())
            .map_err(|_| "productive variable field exceeds u32".to_string())?
            .to_le_bytes(),
    );
    output.extend_from_slice(bytes);
    Ok(())
}

struct InputV1<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> InputV1<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| "productive transition read overflow".to_string())?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "productive transition payload is truncated".to_string())?;
        self.offset = end;
        Ok(bytes)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        Ok(self
            .bytes(N)?
            .try_into()
            .expect("fixed productive transition field"))
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn bytes_field(&mut self) -> Result<&'a [u8], String> {
        let count = self.u32()? as usize;
        self.bytes(count)
    }

    fn string(&mut self) -> Result<String, String> {
        std::str::from_utf8(self.bytes_field()?)
            .map(str::to_owned)
            .map_err(|_| "productive transition string is not UTF-8".to_string())
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::super::events::{
        deterministic_productive_split, LemmaSplitKeyV1, MorphologyEventV1, ProductiveSplitV1,
        TypedEventSpoolConfigV1, TypedEventSpoolWriterV1, TypedProductiveEventV1,
    };
    use super::super::reduce::{reduce_train_morphology, TrainMorphologyReduceConfigV1};
    use super::super::spool_sort::{external_sort_verified_spool, ExternalSpoolSortConfigV1};
    use super::super::types::AXIS_INAPPLICABLE;
    use super::*;

    fn slot_for_pos(pos: u8, number: u8) -> MorphologySlotKeyV1 {
        MorphologySlotKeyV1::new(
            pos,
            number,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
        )
    }

    fn slot(number: u8) -> MorphologySlotKeyV1 {
        slot_for_pos(2, number)
    }

    fn event(lemma: &str, surface: &str, number: u8) -> TypedProductiveEventV1 {
        event_for_pos(lemma, surface, 2, number)
    }

    fn event_for_pos(lemma: &str, surface: &str, pos: u8, number: u8) -> TypedProductiveEventV1 {
        TypedProductiveEventV1::Morphology(MorphologyEventV1 {
            lemma: LemmaSplitKeyV1 {
                language: "ru".to_string(),
                normalized_lemma: lemma.to_string(),
            },
            normalized_surface: surface.to_string(),
            canonical_form_ref: super::super::types::ImportedCanonicalL2FormRefV1(1),
            canonical_feature_mask: 1,
            slot: slot_for_pos(pos, number),
            support: 3,
            provenance: format!("fixture:{lemma}:{surface}").into_bytes(),
        })
    }

    fn schema() -> MorphologyAxisSchemaV1 {
        MorphologyAxisSchemaV1 {
            schema_version: 1,
            pos_applicability: [(2, MorphologyApplicabilityMaskV1::new(0b11).expect("mask"))]
                .into_iter()
                .collect(),
            labels: vec![
                MorphologyAxisLabelV1 {
                    axis: 0,
                    value: 2,
                    label: "noun".to_string(),
                },
                MorphologyAxisLabelV1 {
                    axis: 1,
                    value: 2,
                    label: "singular".to_string(),
                },
                MorphologyAxisLabelV1 {
                    axis: 1,
                    value: 3,
                    label: "plural".to_string(),
                },
            ],
        }
    }

    fn multi_pos_schema() -> MorphologyAxisSchemaV1 {
        let mut schema = schema();
        schema.pos_applicability.insert(
            3,
            MorphologyApplicabilityMaskV1::new(0b11).expect("verb mask"),
        );
        schema.labels.insert(
            1,
            MorphologyAxisLabelV1 {
                axis: 0,
                value: 3,
                label: "verb".to_string(),
            },
        );
        schema
    }

    #[test]
    fn transition_reduce_is_bounded_and_counts_distinct_train_lemmas() {
        let root = std::env::temp_dir().join(format!(
            "lay-productive-transition-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.join("raw"),
            shard_count: 2,
            split_seed: 17,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 1024,
        })
        .expect("event spool");
        let mut admitted = 0;
        for index in 0..1000 {
            let lemma = format!("lemma-{index:04}");
            if deterministic_productive_split(
                &LemmaSplitKeyV1 {
                    language: "ru".to_string(),
                    normalized_lemma: lemma.clone(),
                },
                17,
            ) != ProductiveSplitV1::Train
            {
                continue;
            }
            let singular = format!("stem{index:04}a");
            let plural = format!("stem{index:04}y");
            writer
                .append(&event(&lemma, &singular, 2))
                .expect("singular");
            writer.append(&event(&lemma, &plural, 3)).expect("plural");
            admitted += 1;
            if admitted == 300 {
                break;
            }
        }
        assert_eq!(admitted, 300);
        let raw = writer.finish().expect("raw");
        let sorted = external_sort_verified_spool(
            &raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("sorted"),
                maximum_buffer_bytes: 512,
                maximum_open_runs: 3,
                write_buffer_bytes: 1024,
            },
        )
        .expect("event sort");
        let reduced = reduce_train_morphology(
            &sorted,
            &TrainMorphologyReduceConfigV1 {
                output_path: root.join("lemmas.p2l"),
                write_buffer_bytes: 1024,
                maximum_lemma_bytes: 4096,
            },
        )
        .expect("lemma reduce");
        let config = TransitionReduceConfigV1 {
            root: root.join("induction"),
            maximum_buffer_bytes: 1024,
            maximum_open_runs: 3,
            write_buffer_bytes: 1024,
            maximum_record_bytes: 1024,
            maximum_lemma_transitions: 16,
        };
        let manifest = induce_transition_field(&reduced, &schema(), &config).expect("induction");
        assert_eq!(manifest.transition_observations, admitted * 2);
        assert_eq!(manifest.transferable_observations, admitted * 2);
        assert_eq!(manifest.exact_allomorph_observations, 0);
        assert_eq!(manifest.paradigm_count, 1);
        assert_eq!(manifest.bound_lemma_count as u64, admitted);
        assert_eq!(manifest.compatibility_index_count, 1);
        assert_eq!(manifest.compatibility_posting_count, 1);

        let mut classified = ClassifiedTransitionReaderV1::open(
            &manifest.classified_transitions_path,
            config.maximum_record_bytes,
        )
        .expect("classified reader");
        let mut rows = Vec::new();
        while let Some(row) = classified.next().expect("classified") {
            assert!(row.template().transferable);
            assert_eq!(row.transfer_lemma_support, admitted as u32);
            rows.push(row);
        }
        assert_eq!(rows.len() as u64, admitted * 2);

        let mut paradigms =
            ParadigmDefinitionReaderV1::open(&manifest.paradigms_path, config.maximum_record_bytes)
                .expect("paradigm reader");
        let paradigm = paradigms.next().expect("paradigm").expect("one paradigm");
        assert_eq!(paradigm.paradigm_id, 1);
        assert_eq!(paradigm.lemma_support, admitted as u32);
        assert_eq!(paradigm.signature.transitions.len(), 2);
        assert!(paradigms.next().expect("end").is_none());

        let mut bindings = LemmaParadigmAssignmentReaderV1::open(
            &manifest.lemma_bindings_path,
            config.maximum_record_bytes,
        )
        .expect("binding reader");
        let mut prior = 0;
        let mut count = 0;
        while let Some(binding) = bindings.next().expect("binding") {
            assert!(binding.lemma_id > prior);
            assert_eq!(binding.paradigm_id, 1);
            prior = binding.lemma_id;
            count += 1;
        }
        assert_eq!(count, admitted);

        let mut compatibility = ParadigmCompatibilityIndexReaderV1::open(
            &manifest.compatibility_index_path,
            config.maximum_record_bytes,
        )
        .expect("compatibility reader");
        let index = compatibility
            .next()
            .expect("compatibility")
            .expect("one compatibility row");
        assert_eq!(index.pos_domain, 2);
        assert_eq!(index.source_slot_id, 1);
        assert_eq!(index.posting_start, 0);
        assert_eq!(index.posting_count, 1);
        assert!(compatibility.next().expect("compatibility end").is_none());
        let mut postings = ParadigmPostingReaderV1::open(
            &manifest.paradigm_postings_path,
            config.maximum_record_bytes,
        )
        .expect("posting reader");
        assert_eq!(postings.next().expect("posting"), Some(1));
        assert!(postings.next().expect("posting end").is_none());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn transition_reduce_separates_pos_basins_for_one_canonical_lemma() {
        let root = std::env::temp_dir().join(format!(
            "lay-productive-transition-multi-pos-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.join("raw"),
            shard_count: 2,
            split_seed: 17,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 1024,
        })
        .expect("event spool");
        let mut admitted = 0_u32;
        for index in 0..1000 {
            let lemma = format!("mixed-{index:04}");
            if deterministic_productive_split(
                &LemmaSplitKeyV1 {
                    language: "ru".to_string(),
                    normalized_lemma: lemma.clone(),
                },
                17,
            ) != ProductiveSplitV1::Train
            {
                continue;
            }
            for event in [
                event_for_pos(&lemma, &format!("stem{index:04}a"), 2, 2),
                event_for_pos(&lemma, &format!("stem{index:04}y"), 2, 3),
                event_for_pos(&lemma, &format!("stem{index:04}it"), 3, 2),
                event_for_pos(&lemma, &format!("stem{index:04}ila"), 3, 3),
            ] {
                writer.append(&event).expect("mixed POS form");
            }
            admitted += 1;
            if admitted == 2 {
                break;
            }
        }
        assert_eq!(admitted, 2);
        let raw = writer.finish().expect("raw");
        let sorted = external_sort_verified_spool(
            &raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("sorted"),
                maximum_buffer_bytes: 512,
                maximum_open_runs: 3,
                write_buffer_bytes: 1024,
            },
        )
        .expect("event sort");
        let reduced = reduce_train_morphology(
            &sorted,
            &TrainMorphologyReduceConfigV1 {
                output_path: root.join("lemmas.p2l"),
                write_buffer_bytes: 1024,
                maximum_lemma_bytes: 4096,
            },
        )
        .expect("lemma reduce");
        let config = TransitionReduceConfigV1 {
            root: root.join("induction"),
            maximum_buffer_bytes: 1024,
            maximum_open_runs: 3,
            write_buffer_bytes: 1024,
            maximum_record_bytes: 1024,
            maximum_lemma_transitions: 16,
        };
        let manifest = induce_transition_field(&reduced, &multi_pos_schema(), &config)
            .expect("multi-POS induction");
        assert_eq!(manifest.transition_observations, 8);
        assert_eq!(manifest.paradigm_count, 2);
        assert_eq!(manifest.bound_lemma_count, 4);

        let mut classified = ClassifiedTransitionReaderV1::open(
            &manifest.classified_transitions_path,
            config.maximum_record_bytes,
        )
        .expect("classified reader");
        let mut basin_sources = BTreeMap::<(u32, u8), u32>::new();
        while let Some(row) = classified.next().expect("classified row") {
            let source_pos = row.transition.source_slot.pos_domain();
            assert_eq!(source_pos, row.transition.target_slot.pos_domain());
            assert_eq!(
                basin_sources
                    .entry((row.lemma_id, source_pos))
                    .or_insert(row.source_form_ref),
                &row.source_form_ref
            );
        }
        assert_eq!(basin_sources.len(), 4);

        let mut bindings = LemmaParadigmAssignmentReaderV1::open(
            &manifest.lemma_bindings_path,
            config.maximum_record_bytes,
        )
        .expect("binding reader");
        let mut assignment_keys = BTreeSet::new();
        while let Some(binding) = bindings.next().expect("binding") {
            assert!(assignment_keys.insert((binding.lemma_id, binding.pos_domain)));
        }
        assert_eq!(assignment_keys.len(), 4);
        assert_eq!(
            assignment_keys
                .iter()
                .map(|(_, pos)| *pos)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([2, 3])
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn axis_schema_rejects_implicit_or_unused_dictionary_values() {
        let slots = [slot(2), slot(3)];
        assert!(schema().validate_for_slots(&slots).is_ok());
        let mut missing = schema();
        missing.labels.pop();
        assert!(missing.validate_for_slots(&slots).is_err());
        let mut unused = schema();
        unused.labels.push(MorphologyAxisLabelV1 {
            axis: 1,
            value: 4,
            label: "dual".to_string(),
        });
        unused.labels.sort_unstable();
        assert!(unused.validate_for_slots(&slots).is_err());
    }
}
