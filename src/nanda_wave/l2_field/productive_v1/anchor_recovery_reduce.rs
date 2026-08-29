use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::induce::{
    derive_edit_template, CanonicalFormObservationV1, EditOperationV1, EditTemplateV1,
    ParadigmTransitionKeyV1,
};
use super::transition_reduce::{
    decode_transition_key, ClassifiedTransitionReaderV1, LemmaParadigmAssignmentReaderV1,
    MorphologyAxisSchemaV1, ParadigmDefinitionReaderV1, TransitionInductionManifestV1,
    TransitionReduceConfigV1,
};
use super::PRODUCTIVE_V1_SCHEMA_VERSION;

const MAGIC: [u8; 8] = *b"LAYARSP1";
const HEADER_BYTES: usize = 128;
const ENTRY_ACCOUNTING_BYTES: usize = 160;
const MINIMUM_TRANSFER_LEMMA_SUPPORT: u32 = 2;
pub(super) const ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED: u16 = 1 << 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AnchorRecoveryManifestV1 {
    pub(super) path: PathBuf,
    pub(super) definition_count: u32,
    pub(super) maximum_program_operations: u16,
    pub(super) axis_schema_sha256: [u8; 32],
    pub(super) evidence_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AnchorRecoveryDefinitionV1 {
    pub(super) paradigm_id: u32,
    pub(super) pos_domain: u8,
    pub(super) source_slot_id: u32,
    pub(super) canonical_anchor_slot_id: u32,
    pub(super) transition: ParadigmTransitionKeyV1,
    pub(super) train_lemma_support: u32,
    pub(super) stability: u16,
    pub(super) flags: u16,
    pub(super) provenance_hash_low: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecoveryAggregateKeyV1 {
    paradigm_id: u32,
    source_slot_id: u32,
    canonical_anchor_slot_id: u32,
    transition: ParadigmTransitionKeyV1,
}

#[derive(Clone, Copy, Debug, Default)]
struct RecoveryAggregateV1 {
    last_lemma_id: Option<u32>,
    train_lemma_support: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SharedRecoveryAggregateKeyV1 {
    pos_domain: u8,
    source_slot_id: u32,
    canonical_anchor_slot_id: u32,
    transition: ParadigmTransitionKeyV1,
}

fn observe_lemma_support(aggregate: &mut RecoveryAggregateV1, lemma_id: u32) -> Result<(), String> {
    if aggregate.last_lemma_id != Some(lemma_id) {
        aggregate.train_lemma_support = aggregate
            .train_lemma_support
            .checked_add(1)
            .ok_or_else(|| "anchor recovery lemma support overflow".to_string())?;
        aggregate.last_lemma_id = Some(lemma_id);
    }
    Ok(())
}

fn recovery_template(
    transition: &ParadigmTransitionKeyV1,
    source_slot_id: u32,
    canonical_anchor_slot_id: u32,
    source_slot: super::types::MorphologySlotKeyV1,
    canonical_anchor_slot: super::types::MorphologySlotKeyV1,
) -> EditTemplateV1 {
    EditTemplateV1 {
        source_slot_id,
        target_slot_id: canonical_anchor_slot_id,
        source_slot,
        target_slot: canonical_anchor_slot,
        variant_id: 1,
        operations: transition
            .operations
            .iter()
            .cloned()
            .chain([EditOperationV1::Terminate {
                slot_id: canonical_anchor_slot_id,
                variant_id: 1,
            }])
            .collect(),
        transferable: true,
    }
}

pub(super) fn induce_anchor_recovery_field(
    induction: &TransitionInductionManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    config: &TransitionReduceConfigV1,
) -> Result<AnchorRecoveryManifestV1, String> {
    induce_anchor_recovery_field_inner(induction, axis_schema, config, false, false)
        .map(|(manifest, _)| manifest)
}

pub(super) fn induce_shared_support_anchor_recovery_field(
    induction: &TransitionInductionManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    config: &TransitionReduceConfigV1,
) -> Result<AnchorRecoveryManifestV1, String> {
    induce_anchor_recovery_field_inner(induction, axis_schema, config, true, true)
        .map(|(manifest, _)| manifest)
}

fn induce_anchor_recovery_field_inner(
    induction: &TransitionInductionManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    config: &TransitionReduceConfigV1,
    collect_shared_support: bool,
    admit_shared_support: bool,
) -> Result<(AnchorRecoveryManifestV1, Option<serde_json::Value>), String> {
    fs::create_dir_all(&config.root).map_err(|error| {
        format!(
            "failed to create anchor recovery reduce root {}: {error}",
            config.root.display()
        )
    })?;
    let axis_schema_sha256 = induction.axis_schema_sha256;
    let mut assignment_reader = LemmaParadigmAssignmentReaderV1::open(
        &induction.lemma_bindings_path,
        config.maximum_record_bytes,
    )?;
    let mut assignment = assignment_reader.next()?;
    let mut current_basin = None;
    let mut current_paradigm_id = None;

    let mut paradigm_support = BTreeMap::new();
    let mut paradigm_reader =
        ParadigmDefinitionReaderV1::open(&induction.paradigms_path, config.maximum_record_bytes)?;
    while let Some(paradigm) = paradigm_reader.next()? {
        paradigm_support.insert(paradigm.paradigm_id, paradigm.lemma_support);
    }

    let mut accounted_bytes = paradigm_support
        .len()
        .checked_mul(24)
        .ok_or_else(|| "anchor recovery paradigm accounting overflow".to_string())?;
    if accounted_bytes > config.maximum_buffer_bytes {
        return Err("anchor recovery fixed indexes exceed the bounded reduce budget".to_string());
    }

    let mut aggregates = BTreeMap::<RecoveryAggregateKeyV1, RecoveryAggregateV1>::new();
    let mut shared_aggregates =
        BTreeMap::<SharedRecoveryAggregateKeyV1, RecoveryAggregateV1>::new();
    let mut transitions_by_bucket =
        BTreeMap::<(u32, u32, u32), Vec<ParadigmTransitionKeyV1>>::new();
    let mut classified = ClassifiedTransitionReaderV1::open(
        &induction.classified_transitions_path,
        config.maximum_record_bytes,
    )?;
    while let Some(row) = classified.next()? {
        let pos_domain = row.transition.source_slot.pos_domain();
        if row.transition.target_slot.pos_domain() != pos_domain {
            return Err("anchor recovery classified transition crosses POS domains".to_string());
        }
        let basin = (row.lemma_id, pos_domain);
        if current_basin != Some(basin) {
            if assignment
                .as_ref()
                .is_some_and(|candidate| (candidate.lemma_id, candidate.pos_domain) < basin)
            {
                return Err(
                    "anchor recovery assignment has no classified lemma/POS basin".to_string(),
                );
            }
            let matched =
                assignment.filter(|candidate| (candidate.lemma_id, candidate.pos_domain) == basin);
            current_paradigm_id = matched.map(|candidate| candidate.paradigm_id);
            current_basin = Some(basin);
            if matched.is_some() {
                assignment = assignment_reader.next()?;
            }
        }
        if row.source_slot_id == row.target_slot_id || current_paradigm_id.is_none() {
            continue;
        }
        let paradigm_id = current_paradigm_id.expect("matched recovery paradigm");
        let applicability = axis_schema.applicability(pos_domain)?;
        let exposed = CanonicalFormObservationV1 {
            form_ref: row.target_form_ref,
            slot_id: row.target_slot_id,
            slot: row.transition.target_slot,
            applicability,
            normalized_surface: row.target_surface.clone(),
            support: row.target_support.max(1),
            provenance_id: row.target_form_ref,
            variant_id: row.variant_id.max(1),
        };
        let canonical_anchor = CanonicalFormObservationV1 {
            form_ref: row.source_form_ref,
            slot_id: row.source_slot_id,
            slot: row.transition.source_slot,
            applicability,
            normalized_surface: row.source_surface.clone(),
            support: 1,
            provenance_id: row.source_form_ref,
            variant_id: 1,
        };
        let bucket = (paradigm_id, row.target_slot_id, row.source_slot_id);
        let cached_transition = transitions_by_bucket
            .get(&bucket)
            .and_then(|transitions| {
                transitions.iter().find(|transition| {
                    recovery_template(
                        transition,
                        row.target_slot_id,
                        row.source_slot_id,
                        row.transition.target_slot,
                        row.transition.source_slot,
                    )
                    .reconstruct(&row.target_surface, None)
                    .is_ok_and(|surface| surface == row.source_surface)
                })
            })
            .cloned();
        let transition = match cached_transition {
            Some(transition) => transition,
            None => derive_edit_template(&exposed, &canonical_anchor)
                .map_err(str::to_string)?
                .transition_key(),
        };
        if recovery_template(
            &transition,
            row.target_slot_id,
            row.source_slot_id,
            row.transition.target_slot,
            row.transition.source_slot,
        )
        .reconstruct(&row.target_surface, None)
        .map_err(str::to_string)?
            != row.source_surface
        {
            return Err("anchor recovery reverse program failed exact train replay".to_string());
        }
        let bucket_transitions = transitions_by_bucket.entry(bucket).or_default();
        if !bucket_transitions.contains(&transition) {
            accounted_bytes = accounted_bytes
                .checked_add(80_usize.saturating_add(transition.canonical_bytes().len()))
                .ok_or_else(|| "anchor recovery replay cache accounting overflow".to_string())?;
            if accounted_bytes > config.maximum_buffer_bytes {
                return Err(
                    "anchor recovery replay cache exceeded its bounded memory budget".to_string(),
                );
            }
            bucket_transitions.push(transition.clone());
        }
        let shared_key = SharedRecoveryAggregateKeyV1 {
            pos_domain,
            source_slot_id: row.target_slot_id,
            canonical_anchor_slot_id: row.source_slot_id,
            transition: transition.clone(),
        };
        let key = RecoveryAggregateKeyV1 {
            paradigm_id,
            source_slot_id: row.target_slot_id,
            canonical_anchor_slot_id: row.source_slot_id,
            transition,
        };
        if !aggregates.contains_key(&key) {
            let entry_bytes = ENTRY_ACCOUNTING_BYTES
                .checked_add(key.transition.canonical_bytes().len())
                .ok_or_else(|| "anchor recovery entry accounting overflow".to_string())?;
            accounted_bytes = accounted_bytes
                .checked_add(entry_bytes)
                .ok_or_else(|| "anchor recovery total accounting overflow".to_string())?;
            if accounted_bytes > config.maximum_buffer_bytes {
                return Err("anchor recovery reduce exceeded its bounded memory budget".to_string());
            }
        }
        observe_lemma_support(aggregates.entry(key).or_default(), row.lemma_id)?;
        if collect_shared_support {
            if !shared_aggregates.contains_key(&shared_key) {
                let entry_bytes = ENTRY_ACCOUNTING_BYTES
                    .checked_add(shared_key.transition.canonical_bytes().len())
                    .ok_or_else(|| {
                        "shared anchor recovery entry accounting overflow".to_string()
                    })?;
                accounted_bytes = accounted_bytes.checked_add(entry_bytes).ok_or_else(|| {
                    "shared anchor recovery total accounting overflow".to_string()
                })?;
                if accounted_bytes > config.maximum_buffer_bytes {
                    return Err(
                        "shared anchor recovery audit exceeded its bounded memory budget"
                            .to_string(),
                    );
                }
            }
            observe_lemma_support(
                shared_aggregates.entry(shared_key).or_default(),
                row.lemma_id,
            )?;
        }
    }
    if assignment.is_some() {
        return Err("anchor recovery left an unmatched paradigm assignment".to_string());
    }

    let mut definitions = Vec::new();
    let mut maximum_program_operations = 0_u16;
    for (key, aggregate) in &aggregates {
        let shared_support_certified = aggregate.train_lemma_support == 1
            && admit_shared_support
            && shared_aggregates
                .get(&SharedRecoveryAggregateKeyV1 {
                    pos_domain: key.transition.source_slot.pos_domain(),
                    source_slot_id: key.source_slot_id,
                    canonical_anchor_slot_id: key.canonical_anchor_slot_id,
                    transition: key.transition.clone(),
                })
                .is_some_and(|shared| shared.train_lemma_support >= MINIMUM_TRANSFER_LEMMA_SUPPORT);
        if aggregate.train_lemma_support < MINIMUM_TRANSFER_LEMMA_SUPPORT
            && !shared_support_certified
        {
            continue;
        }
        let denominator = paradigm_support
            .get(&key.paradigm_id)
            .copied()
            .ok_or_else(|| {
                "anchor recovery definition references an unknown paradigm".to_string()
            })?;
        if denominator < aggregate.train_lemma_support || denominator == 0 {
            return Err("anchor recovery support exceeds paradigm evidence".to_string());
        }
        let stability = u16::try_from(
            u64::from(aggregate.train_lemma_support).saturating_mul(u64::from(u16::MAX))
                / u64::from(denominator),
        )
        .map_err(|_| "anchor recovery stability exceeds u16".to_string())?;
        let mut provenance = key.paradigm_id.to_le_bytes().to_vec();
        provenance.extend_from_slice(&key.source_slot_id.to_le_bytes());
        provenance.extend_from_slice(&key.canonical_anchor_slot_id.to_le_bytes());
        provenance.extend_from_slice(&key.transition.canonical_bytes());
        let flags = if shared_support_certified {
            ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED
        } else {
            0
        };
        if flags != 0 {
            provenance.extend_from_slice(&flags.to_le_bytes());
        }
        let digest = Sha256::digest(&provenance);
        let operation_count = key
            .transition
            .operations
            .len()
            .checked_add(1)
            .ok_or_else(|| "anchor recovery operation count overflow".to_string())?;
        maximum_program_operations = maximum_program_operations.max(
            u16::try_from(operation_count)
                .map_err(|_| "anchor recovery program exceeds u16 operations".to_string())?,
        );
        definitions.push(AnchorRecoveryDefinitionV1 {
            paradigm_id: key.paradigm_id,
            pos_domain: key.transition.source_slot.pos_domain(),
            source_slot_id: key.source_slot_id,
            canonical_anchor_slot_id: key.canonical_anchor_slot_id,
            transition: key.transition.clone(),
            train_lemma_support: aggregate.train_lemma_support,
            stability,
            flags,
            provenance_hash_low: u32::from_le_bytes(digest[0..4].try_into().expect("digest")),
        });
    }
    definitions.sort_by(|left, right| {
        (
            left.pos_domain,
            left.source_slot_id,
            left.paradigm_id,
            left.canonical_anchor_slot_id,
            &left.transition,
        )
            .cmp(&(
                right.pos_domain,
                right.source_slot_id,
                right.paradigm_id,
                right.canonical_anchor_slot_id,
                &right.transition,
            ))
    });
    definitions.dedup();

    let path = config.root.join("anchor-recovery-definitions.p2r");
    write_definitions(
        &path,
        axis_schema_sha256,
        maximum_program_operations,
        &definitions,
    )?;
    let manifest = reopen_anchor_recovery_field(&path)?;
    let support_audit = collect_shared_support.then(|| {
        anchor_recovery_support_audit(&aggregates, &shared_aggregates, &definitions, &path)
    });
    Ok((manifest, support_audit))
}

pub(super) fn audit_existing_anchor_recovery_field(
    induction_root: &Path,
    axis_schema: &MorphologyAxisSchemaV1,
    scratch_config: &TransitionReduceConfigV1,
) -> Result<serde_json::Value, String> {
    let existing_path = induction_root.join("anchor-recovery-definitions.p2r");
    let existing = reopen_anchor_recovery_field(&existing_path)?;
    fs::create_dir_all(&scratch_config.root).map_err(|error| error.to_string())?;
    let induction = TransitionInductionManifestV1 {
        classified_transitions_path: induction_root.join("classified-by-lemma.p2c"),
        paradigms_path: induction_root.join("paradigms.p2p"),
        lemma_bindings_path: induction_root.join("lemma-bindings.p2b"),
        compatibility_index_path: induction_root.join("compatibility-index.p2x"),
        paradigm_postings_path: induction_root.join("paradigm-postings.p2o"),
        axis_schema_sha256: existing.axis_schema_sha256,
        transition_observations: 0,
        transferable_observations: 0,
        exact_allomorph_observations: 0,
        paradigm_count: 0,
        bound_lemma_count: 0,
        compatibility_index_count: 0,
        compatibility_posting_count: 0,
        maximum_program_operations: existing.maximum_program_operations,
        anchor_recovery: Some(existing.clone()),
    };
    let (rebuilt, support) =
        induce_anchor_recovery_field_inner(&induction, axis_schema, scratch_config, true, false)?;
    let existing_bytes = fs::read(&existing.path).map_err(|error| error.to_string())?;
    let rebuilt_bytes = fs::read(&rebuilt.path).map_err(|error| error.to_string())?;
    let byte_identical = existing_bytes == rebuilt_bytes;
    let evidence_identical = existing.evidence_sha256 == rebuilt.evidence_sha256;
    if !byte_identical
        || !evidence_identical
        || existing.definition_count != rebuilt.definition_count
        || existing.maximum_program_operations != rebuilt.maximum_program_operations
    {
        return Err("anchor recovery support audit failed frozen V66 parity".to_string());
    }
    Ok(serde_json::json!({
        "kind": "l2_productive_anchor_recovery_support_audit_v1",
        "verdict": "PASS_measurement_only",
        "runtime_authority_changed": false,
        "measured": {
            "fine_and_shared_support": true,
            "definition_fanout": true,
            "definition_spool_bytes": true,
            "compiled_sidecar_bytes": false,
            "runtime_latency": false,
            "fixed_proof_quality": false,
        },
        "frozen_v66_parity": {
            "byte_identical": byte_identical,
            "evidence_identical": evidence_identical,
            "definition_count": existing.definition_count,
            "maximum_program_operations": existing.maximum_program_operations,
            "bytes": existing_bytes.len(),
        },
        "support": support.expect("shared support audit requested"),
    }))
}

fn anchor_recovery_support_audit(
    fine: &BTreeMap<RecoveryAggregateKeyV1, RecoveryAggregateV1>,
    shared: &BTreeMap<SharedRecoveryAggregateKeyV1, RecoveryAggregateV1>,
    current_definitions: &[AnchorRecoveryDefinitionV1],
    current_path: &Path,
) -> serde_json::Value {
    let fine_support_histogram =
        support_histogram(fine.values().map(|aggregate| aggregate.train_lemma_support));
    let shared_support_histogram = support_histogram(
        shared
            .values()
            .map(|aggregate| aggregate.train_lemma_support),
    );
    let mut current_fanout = BTreeMap::<(u8, u32), usize>::new();
    for definition in current_definitions {
        *current_fanout
            .entry((definition.pos_domain, definition.source_slot_id))
            .or_default() += 1;
    }
    let mut proposed_fanout = current_fanout.clone();
    let mut lifted_fine_definitions = 0_usize;
    let mut lifted_definition_spool_bytes = 0_usize;
    for (key, aggregate) in fine {
        if aggregate.train_lemma_support >= MINIMUM_TRANSFER_LEMMA_SUPPORT {
            continue;
        }
        let shared_key = SharedRecoveryAggregateKeyV1 {
            pos_domain: key.transition.source_slot.pos_domain(),
            source_slot_id: key.source_slot_id,
            canonical_anchor_slot_id: key.canonical_anchor_slot_id,
            transition: key.transition.clone(),
        };
        if shared
            .get(&shared_key)
            .is_none_or(|aggregate| aggregate.train_lemma_support < MINIMUM_TRANSFER_LEMMA_SUPPORT)
        {
            continue;
        }
        lifted_fine_definitions += 1;
        lifted_definition_spool_bytes += 4 + 32 + key.transition.canonical_bytes().len();
        *proposed_fanout
            .entry((shared_key.pos_domain, key.source_slot_id))
            .or_default() += 1;
    }
    let current_spool_bytes = fs::metadata(current_path)
        .ok()
        .and_then(|metadata| usize::try_from(metadata.len()).ok())
        .unwrap_or_default();
    serde_json::json!({
        "minimum_transfer_lemma_support": MINIMUM_TRANSFER_LEMMA_SUPPORT,
        "fine": {
            "aggregate_count": fine.len(),
            "admitted_definition_count": current_definitions.len(),
            "filtered_definition_count": fine.len().saturating_sub(current_definitions.len()),
            "support_histogram": fine_support_histogram,
        },
        "shared": {
            "aggregate_count": shared.len(),
            "admitted_aggregate_count": shared.values().filter(|aggregate| aggregate.train_lemma_support >= MINIMUM_TRANSFER_LEMMA_SUPPORT).count(),
            "support_histogram": shared_support_histogram,
        },
        "shared_support_lift": {
            "fine_definition_count": lifted_fine_definitions,
            "current_definition_count": current_definitions.len(),
            "proposed_definition_count": current_definitions.len().saturating_add(lifted_fine_definitions),
            "current_definition_spool_bytes": current_spool_bytes,
            "estimated_proposed_definition_spool_bytes": current_spool_bytes.saturating_add(lifted_definition_spool_bytes),
            "estimate_scope": "definition spool only; compiled sidecar not built",
        },
        "fanout_per_pos_source_slot": {
            "current": fanout_summary(&current_fanout),
            "with_shared_support_lift": fanout_summary(&proposed_fanout),
        },
    })
}

fn support_histogram(supports: impl Iterator<Item = u32>) -> serde_json::Value {
    let mut histogram = BTreeMap::<u32, u64>::new();
    for support in supports {
        *histogram.entry(support).or_default() += 1;
    }
    serde_json::to_value(histogram).expect("support histogram is serializable")
}

fn fanout_summary(fanout: &BTreeMap<(u8, u32), usize>) -> serde_json::Value {
    let mut values = fanout.values().copied().collect::<Vec<_>>();
    values.sort_unstable();
    let percentile = |percent: usize| {
        values
            .get(values.len().saturating_sub(1).saturating_mul(percent) / 100)
            .copied()
            .unwrap_or_default()
    };
    serde_json::json!({
        "lookup_buckets": values.len(),
        "postings": values.iter().sum::<usize>(),
        "min": values.first().copied().unwrap_or_default(),
        "p50": percentile(50),
        "p75": percentile(75),
        "p90": percentile(90),
        "p95": percentile(95),
        "p99": percentile(99),
        "max": values.last().copied().unwrap_or_default(),
    })
}

pub(super) fn reopen_anchor_recovery_field(
    path: &Path,
) -> Result<AnchorRecoveryManifestV1, String> {
    let (definitions, header) = decode_file(path)?;
    if definitions.len() != header.definition_count as usize {
        return Err("anchor recovery definition denominator mismatch".to_string());
    }
    Ok(AnchorRecoveryManifestV1 {
        path: path.to_path_buf(),
        definition_count: header.definition_count,
        maximum_program_operations: header.maximum_program_operations,
        axis_schema_sha256: header.axis_schema_sha256,
        evidence_sha256: header.evidence_sha256,
    })
}

pub(super) fn read_anchor_recovery_definitions(
    manifest: &AnchorRecoveryManifestV1,
) -> Result<Vec<AnchorRecoveryDefinitionV1>, String> {
    let (definitions, header) = decode_file(&manifest.path)?;
    if header.definition_count != manifest.definition_count
        || header.maximum_program_operations != manifest.maximum_program_operations
        || header.axis_schema_sha256 != manifest.axis_schema_sha256
        || header.evidence_sha256 != manifest.evidence_sha256
    {
        return Err("anchor recovery manifest disagrees with its definition spool".to_string());
    }
    Ok(definitions)
}

#[derive(Clone, Copy)]
struct DefinitionHeaderV1 {
    definition_count: u32,
    maximum_program_operations: u16,
    axis_schema_sha256: [u8; 32],
    evidence_sha256: [u8; 32],
}

fn write_definitions(
    path: &Path,
    axis_schema_sha256: [u8; 32],
    maximum_program_operations: u16,
    definitions: &[AnchorRecoveryDefinitionV1],
) -> Result<(), String> {
    let mut payload = Vec::new();
    for definition in definitions {
        let record = encode_definition(definition)?;
        payload.extend_from_slice(
            &u32::try_from(record.len())
                .map_err(|_| "anchor recovery definition exceeds u32 bytes".to_string())?
                .to_le_bytes(),
        );
        payload.extend_from_slice(&record);
    }
    let evidence_sha256: [u8; 32] = Sha256::digest(&payload).into();
    let mut bytes = vec![0_u8; HEADER_BYTES];
    bytes[0..8].copy_from_slice(&MAGIC);
    bytes[8..10].copy_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
    bytes[10..12].copy_from_slice(&(HEADER_BYTES as u16).to_le_bytes());
    bytes[12..16].copy_from_slice(
        &u32::try_from(definitions.len())
            .map_err(|_| "anchor recovery definition count exceeds u32".to_string())?
            .to_le_bytes(),
    );
    bytes[16..18].copy_from_slice(&maximum_program_operations.to_le_bytes());
    bytes[20..52].copy_from_slice(&axis_schema_sha256);
    bytes[52..84].copy_from_slice(&evidence_sha256);
    bytes[84..92].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&payload);
    let temporary = path.with_extension("p2r.tmp");
    fs::write(&temporary, &bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

fn decode_file(
    path: &Path,
) -> Result<(Vec<AnchorRecoveryDefinitionV1>, DefinitionHeaderV1), String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() < HEADER_BYTES
        || bytes[0..8] != MAGIC
        || u16::from_le_bytes(bytes[8..10].try_into().expect("version"))
            != PRODUCTIVE_V1_SCHEMA_VERSION
        || u16::from_le_bytes(bytes[10..12].try_into().expect("header")) as usize != HEADER_BYTES
        || bytes[18..20] != [0; 2]
        || bytes[92..HEADER_BYTES].iter().any(|byte| *byte != 0)
    {
        return Err("anchor recovery definition header is invalid".to_string());
    }
    let definition_count = u32::from_le_bytes(bytes[12..16].try_into().expect("count"));
    let maximum_program_operations =
        u16::from_le_bytes(bytes[16..18].try_into().expect("operations"));
    let axis_schema_sha256 = bytes[20..52].try_into().expect("axis hash");
    let evidence_sha256 = bytes[52..84].try_into().expect("evidence hash");
    let payload_bytes = u64::from_le_bytes(bytes[84..92].try_into().expect("payload bytes"));
    if payload_bytes as usize != bytes.len() - HEADER_BYTES
        || <[u8; 32]>::from(Sha256::digest(&bytes[HEADER_BYTES..])) != evidence_sha256
    {
        return Err("anchor recovery definition payload identity is invalid".to_string());
    }
    let mut cursor = HEADER_BYTES;
    let mut definitions = Vec::with_capacity(definition_count as usize);
    while cursor < bytes.len() {
        let length_end = cursor
            .checked_add(4)
            .ok_or_else(|| "anchor recovery record length overflow".to_string())?;
        if length_end > bytes.len() {
            return Err("anchor recovery record length is truncated".to_string());
        }
        let record_bytes =
            u32::from_le_bytes(bytes[cursor..length_end].try_into().expect("record length"))
                as usize;
        cursor = length_end;
        let end = cursor
            .checked_add(record_bytes)
            .ok_or_else(|| "anchor recovery record end overflow".to_string())?;
        if record_bytes == 0 || end > bytes.len() {
            return Err("anchor recovery record is truncated".to_string());
        }
        definitions.push(decode_definition(&bytes[cursor..end])?);
        cursor = end;
    }
    if definitions.len() != definition_count as usize
        || !definitions
            .windows(2)
            .all(|pair| definition_order(&pair[0], &pair[1]).is_lt())
    {
        return Err("anchor recovery definitions are not strictly canonical".to_string());
    }
    Ok((
        definitions,
        DefinitionHeaderV1 {
            definition_count,
            maximum_program_operations,
            axis_schema_sha256,
            evidence_sha256,
        },
    ))
}

fn encode_definition(definition: &AnchorRecoveryDefinitionV1) -> Result<Vec<u8>, String> {
    validate_definition(definition)?;
    let transition = definition.transition.canonical_bytes();
    let mut bytes = Vec::with_capacity(32 + transition.len());
    bytes.extend_from_slice(&definition.paradigm_id.to_le_bytes());
    bytes.push(definition.pos_domain);
    bytes.extend_from_slice(&[0; 3]);
    bytes.extend_from_slice(&definition.source_slot_id.to_le_bytes());
    bytes.extend_from_slice(&definition.canonical_anchor_slot_id.to_le_bytes());
    bytes.extend_from_slice(&definition.train_lemma_support.to_le_bytes());
    bytes.extend_from_slice(&definition.stability.to_le_bytes());
    bytes.extend_from_slice(&definition.flags.to_le_bytes());
    bytes.extend_from_slice(&definition.provenance_hash_low.to_le_bytes());
    bytes.extend_from_slice(
        &u32::try_from(transition.len())
            .map_err(|_| "anchor recovery transition exceeds u32 bytes".to_string())?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&transition);
    Ok(bytes)
}

fn decode_definition(bytes: &[u8]) -> Result<AnchorRecoveryDefinitionV1, String> {
    if bytes.len() < 32 || bytes[5..8] != [0; 3] {
        return Err("anchor recovery definition fixed fields are invalid".to_string());
    }
    let transition_bytes =
        u32::from_le_bytes(bytes[28..32].try_into().expect("transition bytes")) as usize;
    if transition_bytes != bytes.len() - 32 {
        return Err("anchor recovery transition length is invalid".to_string());
    }
    let definition = AnchorRecoveryDefinitionV1 {
        paradigm_id: u32::from_le_bytes(bytes[0..4].try_into().expect("paradigm")),
        pos_domain: bytes[4],
        source_slot_id: u32::from_le_bytes(bytes[8..12].try_into().expect("source slot")),
        canonical_anchor_slot_id: u32::from_le_bytes(
            bytes[12..16].try_into().expect("anchor slot"),
        ),
        train_lemma_support: u32::from_le_bytes(bytes[16..20].try_into().expect("support")),
        stability: u16::from_le_bytes(bytes[20..22].try_into().expect("stability")),
        flags: u16::from_le_bytes(bytes[22..24].try_into().expect("flags")),
        provenance_hash_low: u32::from_le_bytes(bytes[24..28].try_into().expect("provenance")),
        transition: decode_transition_key(&bytes[32..])?,
    };
    validate_definition(&definition)?;
    Ok(definition)
}

fn validate_definition(definition: &AnchorRecoveryDefinitionV1) -> Result<(), String> {
    if definition.paradigm_id == 0
        || definition.pos_domain < 2
        || definition.source_slot_id == 0
        || definition.canonical_anchor_slot_id == 0
        || definition.source_slot_id == definition.canonical_anchor_slot_id
        || (definition.train_lemma_support < MINIMUM_TRANSFER_LEMMA_SUPPORT
            && !(definition.train_lemma_support == 1
                && definition.flags == ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED))
        || (definition.train_lemma_support >= MINIMUM_TRANSFER_LEMMA_SUPPORT
            && definition.flags != 0)
        || definition.flags & !ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED != 0
        || definition.provenance_hash_low == 0
        || definition.transition.source_slot.pos_domain() != definition.pos_domain
        || definition.transition.target_slot.pos_domain() != definition.pos_domain
        || definition.transition.operations.is_empty()
    {
        return Err("anchor recovery definition identity or evidence is invalid".to_string());
    }
    Ok(())
}

fn definition_order(
    left: &AnchorRecoveryDefinitionV1,
    right: &AnchorRecoveryDefinitionV1,
) -> std::cmp::Ordering {
    (
        left.pos_domain,
        left.source_slot_id,
        left.paradigm_id,
        left.canonical_anchor_slot_id,
        &left.transition,
    )
        .cmp(&(
            right.pos_domain,
            right.source_slot_id,
            right.paradigm_id,
            right.canonical_anchor_slot_id,
            &right.transition,
        ))
}
