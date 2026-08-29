use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::events::{
    decode_verified_spool_record, deterministic_productive_split, ProductiveEventKindV1,
    ProductiveSplitV1, TypedProductiveEventV1, VerifiedSpoolShardReaderV1,
};
use super::spool_sort::SortedTypedEventSpoolManifestV1;
use super::types::MorphologySlotKeyV1;
use super::PRODUCTIVE_V1_SCHEMA_VERSION;

const REDUCED_LEMMA_MAGIC: [u8; 4] = *b"P2L1";
const REDUCED_LEMMA_HEADER_BYTES: usize = 48;

#[derive(Clone, Debug)]
pub(super) struct TrainMorphologyReduceConfigV1 {
    pub(super) output_path: PathBuf,
    pub(super) write_buffer_bytes: usize,
    pub(super) maximum_lemma_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReducedMorphologyFormV1 {
    pub(super) form_ref: u32,
    pub(super) slot: MorphologySlotKeyV1,
    pub(super) variant_id: u16,
    pub(super) normalized_surface: String,
    pub(super) support: u32,
    pub(super) event_identities: Vec<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReducedLemmaMorphologyV1 {
    pub(super) lemma_id: u32,
    pub(super) language: String,
    pub(super) normalized_lemma: String,
    pub(super) forms: Vec<ReducedMorphologyFormV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReducedMorphologyManifestV1 {
    pub(super) path: PathBuf,
    pub(super) split_seed: u64,
    pub(super) compiler_version: u32,
    pub(super) normalization_version: u32,
    pub(super) lemma_count: u32,
    pub(super) form_count: u32,
    pub(super) train_event_count: u64,
    pub(super) morphology_slots: Vec<MorphologySlotKeyV1>,
    pub(super) maximum_observed_scalars: u16,
    pub(super) payload_sha256: [u8; 32],
    pub(super) imported_identity_verified: bool,
    pub(super) imported_lemma_refs: BTreeMap<(String, String), u32>,
}

pub(super) fn reopen_imported_reduced_morphology(
    path: &Path,
    sorted_morphology_path: &Path,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    split_seed: u64,
    compiler_version: u32,
    normalization_version: u32,
) -> Result<ReducedMorphologyManifestV1, String> {
    let mut reader = ReducedLemmaReaderV1::open(path)?;
    let mut lemma_count = 0_u32;
    let mut form_count = 0_u32;
    let mut train_event_count = 0_u64;
    let mut morphology_slots = BTreeSet::new();
    let mut maximum_observed_scalars = 0_u16;
    let mut train_lemma_refs = BTreeMap::new();
    while let Some(lemma) = reader.next_lemma()? {
        if train_lemma_refs
            .insert(
                (lemma.language.clone(), lemma.normalized_lemma.clone()),
                lemma.lemma_id,
            )
            .is_some()
        {
            return Err("productive reopened lemma identity repeats".to_string());
        }
        lemma_count = lemma_count
            .checked_add(1)
            .ok_or_else(|| "productive reopened lemma count overflow".to_string())?;
        form_count = form_count
            .checked_add(
                u32::try_from(lemma.forms.len())
                    .map_err(|_| "productive reopened form count exceeds u32".to_string())?,
            )
            .ok_or_else(|| "productive reopened form count overflow".to_string())?;
        for form in lemma.forms {
            train_event_count = train_event_count
                .checked_add(u64::from(form.support))
                .ok_or_else(|| "productive reopened event count overflow".to_string())?;
            morphology_slots.insert(form.slot);
            maximum_observed_scalars = maximum_observed_scalars.max(
                u16::try_from(form.normalized_surface.chars().count())
                    .map_err(|_| "productive reopened surface exceeds u16".to_string())?,
            );
        }
    }
    let payload_sha256 = reader.payload_sha256();
    if train_lemma_refs.is_empty() {
        return Err("productive reopened morphology is empty".to_string());
    }
    let imported_lemma_refs =
        reopen_full_imported_lemma_refs(sorted_morphology_path, canonical_l2, split_seed)?;
    if train_lemma_refs
        .iter()
        .any(|(key, lemma_ref)| imported_lemma_refs.get(key) != Some(lemma_ref))
    {
        return Err(
            "productive reopened TRAIN identities disagree with imported ownership".to_string(),
        );
    }
    Ok(ReducedMorphologyManifestV1 {
        path: path.to_path_buf(),
        split_seed,
        compiler_version,
        normalization_version,
        lemma_count,
        form_count,
        train_event_count,
        morphology_slots: morphology_slots.into_iter().collect(),
        maximum_observed_scalars,
        payload_sha256,
        imported_identity_verified: true,
        imported_lemma_refs,
    })
}

fn reopen_full_imported_lemma_refs(
    sorted_morphology_path: &Path,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    split_seed: u64,
) -> Result<BTreeMap<(String, String), u32>, String> {
    let mut reader = VerifiedSpoolShardReaderV1::open(sorted_morphology_path)?;
    let mut imported_lemma_refs = BTreeMap::new();
    let mut current_key = None::<(String, String)>;
    let mut current_bindings = BTreeSet::<(u32, u32)>::new();
    let mut canonical_lemma_ref = 0_u32;

    while let Some(record) = reader.next_record()? {
        if record.kind != ProductiveEventKindV1::Morphology {
            return Err("productive morphology reopen contains another event kind".to_string());
        }
        let event = decode_verified_spool_record(&record, split_seed)?;
        let TypedProductiveEventV1::Morphology(event) = event else {
            return Err("productive morphology reopen decoded another event kind".to_string());
        };
        if record.split != deterministic_productive_split(&event.lemma, split_seed) {
            return Err("productive morphology reopen split identity disagrees".to_string());
        }
        let key = (event.lemma.language, event.lemma.normalized_lemma);
        if current_key.as_ref().is_some_and(|current| current != &key) {
            finish_reopened_imported_lemma(
                current_key.take().expect("current imported lemma"),
                &mut current_bindings,
                canonical_lemma_ref,
                canonical_l2,
                &mut imported_lemma_refs,
            )?;
            canonical_lemma_ref = canonical_lemma_ref
                .checked_add(1)
                .ok_or_else(|| "reopened canonical L2 lemma reference overflow".to_string())?;
        }
        current_key.get_or_insert(key);
        current_bindings.insert((event.canonical_form_ref.0, event.canonical_feature_mask));
    }
    if let Some(key) = current_key {
        finish_reopened_imported_lemma(
            key,
            &mut current_bindings,
            canonical_lemma_ref,
            canonical_l2,
            &mut imported_lemma_refs,
        )?;
        canonical_lemma_ref = canonical_lemma_ref
            .checked_add(1)
            .ok_or_else(|| "reopened canonical L2 lemma count overflow".to_string())?;
    }
    if canonical_lemma_ref as usize != canonical_l2.lemma_count() {
        return Err(format!(
            "productive reopened lemma count {} disagrees with canonical L2 {}",
            canonical_lemma_ref,
            canonical_l2.lemma_count()
        ));
    }
    Ok(imported_lemma_refs)
}

fn finish_reopened_imported_lemma(
    key: (String, String),
    bindings: &mut BTreeSet<(u32, u32)>,
    canonical_lemma_ref: u32,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    imported_lemma_refs: &mut BTreeMap<(String, String), u32>,
) -> Result<(), String> {
    let actual = canonical_l2
        .imported_binding_pairs_for_lemma(canonical_lemma_ref)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if *bindings != actual {
        return Err(format!(
            "productive reopened ownership mismatch lemma={:?} ref={} source_bindings={} package_bindings={}",
            key.1,
            canonical_lemma_ref,
            bindings.len(),
            actual.len(),
        ));
    }
    if imported_lemma_refs
        .insert(key, canonical_lemma_ref)
        .is_some()
    {
        return Err("productive reopened imported lemma key repeats".to_string());
    }
    bindings.clear();
    Ok(())
}

#[derive(Clone, Debug)]
struct PendingFormV1 {
    support: u32,
    event_identities: Vec<[u8; 32]>,
}

#[derive(Clone, Debug)]
struct ImportedPendingFormV1 {
    canonical_form_ref: u32,
    canonical_feature_mask: u32,
    support: u32,
    event_identities: Vec<[u8; 32]>,
}

pub(super) fn reduce_train_morphology(
    sorted: &SortedTypedEventSpoolManifestV1,
    config: &TrainMorphologyReduceConfigV1,
) -> Result<ReducedMorphologyManifestV1, String> {
    if sorted.shards.len() != 1 {
        return Err("productive train reduce requires one globally sorted spool".to_string());
    }
    if config.write_buffer_bytes < REDUCED_LEMMA_HEADER_BYTES || config.maximum_lemma_bytes == 0 {
        return Err("productive train reduce has an invalid memory or write budget".to_string());
    }
    let mut reader = VerifiedSpoolShardReaderV1::open(&sorted.shards[0].path)?;
    let file = File::create(&config.output_path).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::with_capacity(config.write_buffer_bytes, file);
    let mut output_hasher = Sha256::new();
    let mut current_key: Option<(String, String)> = None;
    let mut current_forms = BTreeMap::<(MorphologySlotKeyV1, String), PendingFormV1>::new();
    let mut current_bytes = 0_usize;
    let mut slots = BTreeSet::new();
    let mut lemma_count = 0_u32;
    let mut form_count = 0_u32;
    let mut train_event_count = 0_u64;
    let mut maximum_observed_scalars = 0_u16;
    while let Some(record) = reader.next_record()? {
        if record.split != ProductiveSplitV1::Train
            || record.kind != ProductiveEventKindV1::Morphology
        {
            continue;
        }
        let event = decode_verified_spool_record(&record, sorted.split_seed)?;
        let TypedProductiveEventV1::Morphology(event) = event else {
            return Err("productive morphology reduce decoded another event kind".to_string());
        };
        let key = (event.lemma.language, event.lemma.normalized_lemma);
        if current_key.as_ref().is_some_and(|current| current != &key) {
            flush_lemma(
                current_key.take().expect("current lemma"),
                &mut current_forms,
                &mut lemma_count,
                &mut form_count,
                &mut writer,
                &mut output_hasher,
            )?;
            current_bytes = 0;
        }
        if current_key.is_none() {
            current_key = Some(key);
        }
        let scalar_count = event.normalized_surface.chars().count();
        if scalar_count >= u16::MAX as usize {
            return Err("productive train surface reaches the scalar wire ceiling".to_string());
        }
        maximum_observed_scalars = maximum_observed_scalars.max(scalar_count as u16);
        slots.insert(event.slot);
        current_bytes = current_bytes
            .checked_add(event.normalized_surface.len())
            .and_then(|bytes| bytes.checked_add(64))
            .ok_or_else(|| "productive lemma reduction accounting overflow".to_string())?;
        if current_bytes > config.maximum_lemma_bytes {
            return Err(
                "productive lemma exceeds the configured bounded reduce budget".to_string(),
            );
        }
        let pending = current_forms
            .entry((event.slot, event.normalized_surface))
            .or_insert_with(|| PendingFormV1 {
                support: 0,
                event_identities: Vec::new(),
            });
        pending.support = pending
            .support
            .checked_add(event.support)
            .ok_or_else(|| "productive form support exceeds u32".to_string())?;
        pending.event_identities.push(record.event_sha256);
        train_event_count = train_event_count
            .checked_add(1)
            .ok_or_else(|| "productive train event count overflow".to_string())?;
    }
    if let Some(key) = current_key.take() {
        flush_lemma(
            key,
            &mut current_forms,
            &mut lemma_count,
            &mut form_count,
            &mut writer,
            &mut output_hasher,
        )?;
    }
    writer.flush().map_err(|error| error.to_string())?;
    Ok(ReducedMorphologyManifestV1 {
        path: config.output_path.clone(),
        split_seed: sorted.split_seed,
        compiler_version: sorted.compiler_version,
        normalization_version: sorted.normalization_version,
        lemma_count,
        form_count,
        train_event_count,
        morphology_slots: slots.into_iter().collect(),
        maximum_observed_scalars,
        payload_sha256: output_hasher.finalize().into(),
        imported_identity_verified: false,
        imported_lemma_refs: BTreeMap::new(),
    })
}

pub(super) fn reduce_train_morphology_with_imported_ownership(
    sorted: &SortedTypedEventSpoolManifestV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    config: &TrainMorphologyReduceConfigV1,
) -> Result<ReducedMorphologyManifestV1, String> {
    if sorted.shards.len() != 1 {
        return Err("productive imported reduce requires one globally sorted spool".to_string());
    }
    if config.write_buffer_bytes < REDUCED_LEMMA_HEADER_BYTES || config.maximum_lemma_bytes == 0 {
        return Err("productive imported reduce has an invalid memory or write budget".to_string());
    }
    let mut reader = VerifiedSpoolShardReaderV1::open(&sorted.shards[0].path)?;
    let file = File::create(&config.output_path).map_err(|error| error.to_string())?;
    let mut writer = BufWriter::with_capacity(config.write_buffer_bytes, file);
    let mut output_hasher = Sha256::new();
    let mut current_key: Option<(String, String)> = None;
    let mut current_split = None;
    let mut current_forms = BTreeMap::<(MorphologySlotKeyV1, String), ImportedPendingFormV1>::new();
    let mut current_bytes = 0_usize;
    let mut slots = BTreeSet::new();
    let mut imported_lemma_ref = 0_u32;
    let mut train_lemma_count = 0_u32;
    let mut train_form_count = 0_u32;
    let mut train_event_count = 0_u64;
    let mut maximum_observed_scalars = 0_u16;
    let mut imported_lemma_refs = BTreeMap::new();
    while let Some(record) = reader.next_record()? {
        if record.kind != ProductiveEventKindV1::Morphology {
            continue;
        }
        let event = decode_verified_spool_record(&record, sorted.split_seed)?;
        let TypedProductiveEventV1::Morphology(event) = event else {
            return Err("productive imported reduce decoded another event kind".to_string());
        };
        let key = (event.lemma.language, event.lemma.normalized_lemma);
        if current_key.as_ref().is_some_and(|current| current != &key) {
            let completed_key = current_key.take().expect("current imported lemma");
            if imported_lemma_refs
                .insert(completed_key.clone(), imported_lemma_ref)
                .is_some()
            {
                return Err("productive imported lemma key repeats".to_string());
            }
            flush_imported_lemma(
                completed_key,
                current_split.take().expect("current imported split"),
                &mut current_forms,
                imported_lemma_ref,
                canonical_l2,
                &mut train_lemma_count,
                &mut train_form_count,
                &mut writer,
                &mut output_hasher,
            )?;
            imported_lemma_ref = imported_lemma_ref
                .checked_add(1)
                .ok_or_else(|| "imported canonical L2 lemma reference overflow".to_string())?;
            current_bytes = 0;
        }
        if current_key.is_none() {
            current_key = Some(key);
            current_split = Some(record.split);
        } else if current_split != Some(record.split) {
            return Err("one productive lemma crosses deterministic splits".to_string());
        }
        let scalar_count = event.normalized_surface.chars().count();
        if scalar_count >= u16::MAX as usize {
            return Err("productive imported surface reaches the scalar wire ceiling".to_string());
        }
        current_bytes = current_bytes
            .checked_add(event.normalized_surface.len())
            .and_then(|bytes| bytes.checked_add(72))
            .ok_or_else(|| "productive imported lemma accounting overflow".to_string())?;
        if current_bytes > config.maximum_lemma_bytes {
            return Err("productive imported lemma exceeds the bounded reduce budget".to_string());
        }
        let form_key = (event.slot, event.normalized_surface);
        let pending = current_forms
            .entry(form_key)
            .or_insert_with(|| ImportedPendingFormV1 {
                canonical_form_ref: event.canonical_form_ref.0,
                canonical_feature_mask: event.canonical_feature_mask,
                support: 0,
                event_identities: Vec::new(),
            });
        if pending.canonical_form_ref != event.canonical_form_ref.0
            || pending.canonical_feature_mask != event.canonical_feature_mask
        {
            return Err("productive imported form has conflicting canonical ownership".to_string());
        }
        pending.support = pending
            .support
            .checked_add(event.support)
            .ok_or_else(|| "productive imported form support exceeds u32".to_string())?;
        pending.event_identities.push(record.event_sha256);
        if record.split == ProductiveSplitV1::Train {
            slots.insert(event.slot);
            maximum_observed_scalars = maximum_observed_scalars.max(scalar_count as u16);
            train_event_count = train_event_count
                .checked_add(1)
                .ok_or_else(|| "productive imported TRAIN event count overflow".to_string())?;
        }
    }
    if let Some(key) = current_key.take() {
        if imported_lemma_refs
            .insert(key.clone(), imported_lemma_ref)
            .is_some()
        {
            return Err("productive final imported lemma key repeats".to_string());
        }
        flush_imported_lemma(
            key,
            current_split.expect("final imported split"),
            &mut current_forms,
            imported_lemma_ref,
            canonical_l2,
            &mut train_lemma_count,
            &mut train_form_count,
            &mut writer,
            &mut output_hasher,
        )?;
        imported_lemma_ref = imported_lemma_ref
            .checked_add(1)
            .ok_or_else(|| "imported canonical L2 lemma count overflow".to_string())?;
    }
    if imported_lemma_ref as usize != canonical_l2.lemma_count() {
        return Err(format!(
            "productive imported lemma count {} disagrees with canonical L2 {}",
            imported_lemma_ref,
            canonical_l2.lemma_count()
        ));
    }
    writer.flush().map_err(|error| error.to_string())?;
    Ok(ReducedMorphologyManifestV1 {
        path: config.output_path.clone(),
        split_seed: sorted.split_seed,
        compiler_version: sorted.compiler_version,
        normalization_version: sorted.normalization_version,
        lemma_count: train_lemma_count,
        form_count: train_form_count,
        train_event_count,
        morphology_slots: slots.into_iter().collect(),
        maximum_observed_scalars,
        payload_sha256: output_hasher.finalize().into(),
        imported_identity_verified: true,
        imported_lemma_refs,
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn flush_imported_lemma(
    (language, normalized_lemma): (String, String),
    split: ProductiveSplitV1,
    forms: &mut BTreeMap<(MorphologySlotKeyV1, String), ImportedPendingFormV1>,
    canonical_lemma_ref: u32,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    train_lemma_count: &mut u32,
    train_form_count: &mut u32,
    writer: &mut BufWriter<File>,
    output_hasher: &mut Sha256,
) -> Result<(), String> {
    let expected = forms
        .values()
        .map(|form| (form.canonical_form_ref, form.canonical_feature_mask))
        .collect::<BTreeSet<_>>();
    let actual = canonical_l2
        .imported_binding_pairs_for_lemma(canonical_lemma_ref)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if expected != actual {
        return Err(format!(
            "productive imported ownership mismatch lemma={normalized_lemma:?} ref={canonical_lemma_ref} source_bindings={} package_bindings={}",
            expected.len(),
            actual.len(),
        ));
    }
    if split != ProductiveSplitV1::Train {
        forms.clear();
        return Ok(());
    }
    *train_lemma_count = train_lemma_count
        .checked_add(1)
        .ok_or_else(|| "productive TRAIN lemma count exceeds u32".to_string())?;
    let mut output_forms = Vec::with_capacity(forms.len());
    let mut previous_slot = None;
    let mut variant_id = 0_u16;
    for ((slot, surface), mut pending) in std::mem::take(forms) {
        if previous_slot != Some(slot) {
            previous_slot = Some(slot);
            variant_id = 1;
        } else {
            variant_id = variant_id
                .checked_add(1)
                .ok_or_else(|| "productive imported slot variant count exceeds u16".to_string())?;
            if variant_id == u16::MAX {
                return Err("productive imported slot variant reaches the wire ceiling".to_string());
            }
        }
        *train_form_count = train_form_count
            .checked_add(1)
            .ok_or_else(|| "productive TRAIN form count exceeds u32".to_string())?;
        pending.event_identities.sort_unstable();
        pending.event_identities.dedup();
        output_forms.push(ReducedMorphologyFormV1 {
            form_ref: pending.canonical_form_ref,
            slot,
            variant_id,
            normalized_surface: surface,
            support: pending.support,
            event_identities: pending.event_identities,
        });
    }
    write_reduced_lemma(
        &ReducedLemmaMorphologyV1 {
            lemma_id: canonical_lemma_ref,
            language,
            normalized_lemma,
            forms: output_forms,
        },
        writer,
        output_hasher,
    )
}

fn flush_lemma(
    (language, normalized_lemma): (String, String),
    forms: &mut BTreeMap<(MorphologySlotKeyV1, String), PendingFormV1>,
    lemma_count: &mut u32,
    form_count: &mut u32,
    writer: &mut BufWriter<File>,
    output_hasher: &mut Sha256,
) -> Result<(), String> {
    *lemma_count = lemma_count
        .checked_add(1)
        .ok_or_else(|| "productive lemma count exceeds u32".to_string())?;
    let lemma_id = *lemma_count;
    let mut output_forms = Vec::with_capacity(forms.len());
    let mut previous_slot = None;
    let mut variant_id = 0_u16;
    for ((slot, surface), mut pending) in std::mem::take(forms) {
        if previous_slot != Some(slot) {
            previous_slot = Some(slot);
            variant_id = 1;
        } else {
            variant_id = variant_id
                .checked_add(1)
                .ok_or_else(|| "productive slot variant count exceeds u16".to_string())?;
            if variant_id == u16::MAX {
                return Err("productive slot variant reaches the wire ceiling".to_string());
            }
        }
        *form_count = form_count
            .checked_add(1)
            .ok_or_else(|| "productive form count exceeds u32".to_string())?;
        pending.event_identities.sort_unstable();
        pending.event_identities.dedup();
        output_forms.push(ReducedMorphologyFormV1 {
            form_ref: *form_count,
            slot,
            variant_id,
            normalized_surface: surface,
            support: pending.support,
            event_identities: pending.event_identities,
        });
    }
    let lemma = ReducedLemmaMorphologyV1 {
        lemma_id,
        language,
        normalized_lemma,
        forms: output_forms,
    };
    write_reduced_lemma(&lemma, writer, output_hasher)
}

fn write_reduced_lemma(
    lemma: &ReducedLemmaMorphologyV1,
    writer: &mut BufWriter<File>,
    output_hasher: &mut Sha256,
) -> Result<(), String> {
    let payload = encode_lemma(lemma)?;
    let payload_sha256: [u8; 32] = Sha256::digest(&payload).into();
    let mut header = [0_u8; REDUCED_LEMMA_HEADER_BYTES];
    header[0..4].copy_from_slice(&REDUCED_LEMMA_MAGIC);
    header[4..6].copy_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
    header[8..12].copy_from_slice(&lemma.lemma_id.to_le_bytes());
    header[12..16].copy_from_slice(
        &u32::try_from(payload.len())
            .map_err(|_| "productive reduced lemma payload exceeds u32".to_string())?
            .to_le_bytes(),
    );
    header[16..48].copy_from_slice(&payload_sha256);
    writer
        .write_all(&header)
        .and_then(|_| writer.write_all(&payload))
        .map_err(|error| error.to_string())?;
    output_hasher.update(header);
    output_hasher.update(payload);
    Ok(())
}

fn encode_lemma(lemma: &ReducedLemmaMorphologyV1) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    push_string(&mut bytes, &lemma.language)?;
    push_string(&mut bytes, &lemma.normalized_lemma)?;
    push_u32(&mut bytes, lemma.forms.len())?;
    for form in &lemma.forms {
        bytes.extend_from_slice(&form.form_ref.to_le_bytes());
        bytes.extend_from_slice(&form.slot.to_bytes());
        bytes.extend_from_slice(&form.variant_id.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&form.support.to_le_bytes());
        push_string(&mut bytes, &form.normalized_surface)?;
        push_u32(&mut bytes, form.event_identities.len())?;
        for identity in &form.event_identities {
            bytes.extend_from_slice(identity);
        }
    }
    Ok(bytes)
}

pub(super) struct ReducedLemmaReaderV1 {
    reader: BufReader<File>,
    previous_lemma_id: Option<u32>,
    hasher: Sha256,
}

impl ReducedLemmaReaderV1 {
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        Ok(Self {
            reader: BufReader::new(File::open(path).map_err(|error| error.to_string())?),
            previous_lemma_id: None,
            hasher: Sha256::new(),
        })
    }

    pub(super) fn next_lemma(&mut self) -> Result<Option<ReducedLemmaMorphologyV1>, String> {
        let mut header = [0_u8; REDUCED_LEMMA_HEADER_BYTES];
        match self.reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(1) => self
                .reader
                .read_exact(&mut header[1..])
                .map_err(|_| "productive reduced lemma header is truncated".to_string())?,
            Ok(_) => unreachable!("one-byte reduced lemma read returned more than one byte"),
            Err(error) => return Err(error.to_string()),
        }
        if header[0..4] != REDUCED_LEMMA_MAGIC
            || u16::from_le_bytes(header[4..6].try_into().expect("fixed slice"))
                != PRODUCTIVE_V1_SCHEMA_VERSION
            || header[6..8] != [0; 2]
        {
            return Err("productive reduced lemma header magic or version is invalid".to_string());
        }
        let lemma_id = u32::from_le_bytes(header[8..12].try_into().expect("fixed slice"));
        if self
            .previous_lemma_id
            .is_some_and(|previous| lemma_id <= previous)
        {
            return Err("productive reduced lemma identities are not strictly ordered".to_string());
        }
        self.previous_lemma_id = Some(lemma_id);
        let payload_bytes =
            u32::from_le_bytes(header[12..16].try_into().expect("fixed slice")) as usize;
        let expected_sha: [u8; 32] = header[16..48].try_into().expect("fixed slice");
        let mut payload = vec![0_u8; payload_bytes];
        self.reader
            .read_exact(&mut payload)
            .map_err(|error| error.to_string())?;
        if <[u8; 32]>::from(Sha256::digest(&payload)) != expected_sha {
            return Err("productive reduced lemma payload SHA-256 mismatch".to_string());
        }
        self.hasher.update(header);
        self.hasher.update(&payload);
        let lemma = decode_lemma(lemma_id, &payload)?;
        Ok(Some(lemma))
    }

    pub(super) fn payload_sha256(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }
}

fn decode_lemma(lemma_id: u32, bytes: &[u8]) -> Result<ReducedLemmaMorphologyV1, String> {
    let mut input = ReducedInputV1::new(bytes);
    let language = input.string()?;
    let normalized_lemma = input.string()?;
    let form_count = input.u32()? as usize;
    let mut forms = Vec::with_capacity(form_count);
    for _ in 0..form_count {
        let form_ref = input.u32()?;
        let slot = MorphologySlotKeyV1::from_bytes(input.array()?).map_err(str::to_string)?;
        let variant_id = input.u16()?;
        if input.u16()? != 0 {
            return Err("productive reduced form reserved value is not zero".to_string());
        }
        let support = input.u32()?;
        let normalized_surface = input.string()?;
        let identity_count = input.u32()? as usize;
        let mut event_identities = Vec::with_capacity(identity_count);
        for _ in 0..identity_count {
            event_identities.push(input.array()?);
        }
        if variant_id == 0
            || support == 0
            || normalized_surface.is_empty()
            || event_identities.is_empty()
            || event_identities.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(
                "productive reduced form identity, support, or provenance is invalid".to_string(),
            );
        }
        forms.push(ReducedMorphologyFormV1 {
            form_ref,
            slot,
            variant_id,
            normalized_surface,
            support,
            event_identities,
        });
    }
    if !input.is_empty() || forms.is_empty() {
        return Err("productive reduced lemma is empty or has an unowned suffix".to_string());
    }
    Ok(ReducedLemmaMorphologyV1 {
        lemma_id,
        language,
        normalized_lemma,
        forms,
    })
}

fn push_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), String> {
    push_u32(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_u32(bytes: &mut Vec<u8>, value: usize) -> Result<(), String> {
    bytes.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| "productive reduced sequence exceeds u32".to_string())?
            .to_le_bytes(),
    );
    Ok(())
}

struct ReducedInputV1<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ReducedInputV1<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| "productive reduced lemma read overflow".to_string())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "productive reduced lemma is truncated".to_string())?
            .try_into()
            .expect("fixed reduced lemma field");
        self.offset = end;
        Ok(value)
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| "productive reduced string range overflow".to_string())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "productive reduced string is truncated".to_string())?;
        self.offset = end;
        Ok(value)
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn string(&mut self) -> Result<String, String> {
        let count = self.u32()? as usize;
        std::str::from_utf8(self.bytes(count)?)
            .map(str::to_owned)
            .map_err(|_| "productive reduced string is not UTF-8".to_string())
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::super::events::{
        deterministic_productive_split, LemmaSplitKeyV1, MorphologyEventV1,
        TypedEventSpoolConfigV1, TypedEventSpoolWriterV1,
    };
    use super::super::spool_sort::{external_sort_verified_spool, ExternalSpoolSortConfigV1};
    use super::*;

    fn train_event(lemma: &str, surface: &str, provenance: &[u8]) -> TypedProductiveEventV1 {
        TypedProductiveEventV1::Morphology(MorphologyEventV1 {
            lemma: LemmaSplitKeyV1 {
                language: "ru".to_string(),
                normalized_lemma: lemma.to_string(),
            },
            normalized_surface: surface.to_string(),
            canonical_form_ref: super::super::types::ImportedCanonicalL2FormRefV1(1),
            canonical_feature_mask: 1,
            slot: MorphologySlotKeyV1::new(2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            support: 2,
            provenance: provenance.to_vec(),
        })
    }

    #[test]
    fn train_reduce_keeps_one_lemma_in_memory_and_assigns_canonical_ids() {
        let root = std::env::temp_dir().join(format!(
            "lay-productive-reduce-{}-{}",
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
            write_buffer_bytes: 4096,
        })
        .expect("writer");
        let mut expected_lemmas = BTreeSet::new();
        for index in 0..200 {
            let lemma = format!("lemma-{index:03}");
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
            expected_lemmas.insert(lemma.clone());
            writer
                .append(&train_event(&lemma, &format!("surface-{index:03}"), b"a"))
                .expect("first source");
            writer
                .append(&train_event(&lemma, &format!("surface-{index:03}"), b"b"))
                .expect("second source");
        }
        let raw = writer.finish().expect("raw manifest");
        let sorted = external_sort_verified_spool(
            &raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("sorted"),
                maximum_buffer_bytes: 1024,
                maximum_open_runs: 4,
                write_buffer_bytes: 1024,
            },
        )
        .expect("sort");
        let manifest = reduce_train_morphology(
            &sorted,
            &TrainMorphologyReduceConfigV1 {
                output_path: root.join("lemmas.p2l"),
                write_buffer_bytes: 1024,
                maximum_lemma_bytes: 4096,
            },
        )
        .expect("reduce");
        assert_eq!(manifest.lemma_count as usize, expected_lemmas.len());
        assert_eq!(manifest.form_count, manifest.lemma_count);
        assert_eq!(
            manifest.train_event_count,
            u64::from(manifest.lemma_count) * 2
        );
        let mut reader = ReducedLemmaReaderV1::open(&manifest.path).expect("reader");
        let mut lemmas = Vec::new();
        while let Some(lemma) = reader.next_lemma().expect("lemma") {
            assert_eq!(lemma.forms.len(), 1);
            assert_eq!(lemma.forms[0].support, 4);
            assert_eq!(lemma.forms[0].event_identities.len(), 2);
            lemmas.push(lemma);
        }
        assert_eq!(reader.payload_sha256(), manifest.payload_sha256);
        assert!(lemmas.windows(2).all(|pair| {
            pair[0].lemma_id + 1 == pair[1].lemma_id
                && pair[0].normalized_lemma < pair[1].normalized_lemma
                && pair[0].forms[0].form_ref + 1 == pair[1].forms[0].form_ref
        }));
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn imported_reduce_and_reopen_preserve_sparse_canonical_refs() {
        let corpus = super::super::super::teacher::L2TeacherCorpus::parse_tsv(
            "F\tlemma-a\tmiddle\tnoun:nom:sg\n\
             F\tlemma-b\talpha\tnoun:nom:sg\n\
             F\tlemma-b\tomega\tnoun:gen:sg\n\
             T\tlemma-b\talpha\tnoun:nom:sg\t_ middle\n\
             H\tlemma-b\tomega\tnoun:gen:sg\tmiddle _\n",
        )
        .expect("canonical corpus");
        let terminals = BTreeMap::from([("alpha", 7), ("middle", 11), ("omega", 13)]);
        let (package, _) =
            super::super::super::compiler::compile_l2_package(&corpus, 99, |surface| {
                terminals.get(surface).copied()
            })
            .expect("canonical package");
        let field = super::super::super::runtime::StandaloneL2Field::from_package(package)
            .expect("canonical field");
        assert_eq!(field.form_ref_for_surface("alpha"), Some(0));
        assert_eq!(field.form_ref_for_surface("middle"), Some(1));
        assert_eq!(field.form_ref_for_surface("omega"), Some(2));

        let keys = ["lemma-a", "lemma-b"].map(|lemma| LemmaSplitKeyV1 {
            language: "ru".to_string(),
            normalized_lemma: lemma.to_string(),
        });
        let split_seed = (0_u64..10_000)
            .find(|seed| {
                deterministic_productive_split(&keys[0], *seed) != ProductiveSplitV1::Train
                    && deterministic_productive_split(&keys[1], *seed) == ProductiveSplitV1::Train
            })
            .expect("sparse TRAIN seed");
        let root = std::env::temp_dir().join(format!(
            "lay-productive-imported-reduce-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.join("raw"),
            shard_count: 2,
            split_seed,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("writer");
        for (lemma, surface, features, slot) in [
            (
                "lemma-a",
                "middle",
                "noun:nom:sg",
                MorphologySlotKeyV1::new(2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0),
            ),
            (
                "lemma-b",
                "alpha",
                "noun:nom:sg",
                MorphologySlotKeyV1::new(2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0),
            ),
            (
                "lemma-b",
                "omega",
                "noun:gen:sg",
                MorphologySlotKeyV1::new(2, 2, 3, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0),
            ),
        ] {
            let form_ref = field.form_ref_for_surface(surface).expect("form ref");
            writer
                .append(&TypedProductiveEventV1::Morphology(MorphologyEventV1 {
                    lemma: LemmaSplitKeyV1 {
                        language: "ru".to_string(),
                        normalized_lemma: lemma.to_string(),
                    },
                    normalized_surface: surface.to_string(),
                    canonical_form_ref: super::super::types::ImportedCanonicalL2FormRefV1(form_ref),
                    canonical_feature_mask: crate::nanda_wave::morphology_phase::parse_features(
                        features,
                    )
                    .expect("feature mask"),
                    slot,
                    support: 1,
                    provenance: surface.as_bytes().to_vec(),
                }))
                .expect("morphology event");
        }
        let raw = writer.finish().expect("raw manifest");
        let sorted = external_sort_verified_spool(
            &raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("sorted"),
                maximum_buffer_bytes: 1024,
                maximum_open_runs: 4,
                write_buffer_bytes: 1024,
            },
        )
        .expect("sort");
        let manifest = reduce_train_morphology_with_imported_ownership(
            &sorted,
            &field,
            &TrainMorphologyReduceConfigV1 {
                output_path: root.join("lemmas.p2l"),
                write_buffer_bytes: 1024,
                maximum_lemma_bytes: 4096,
            },
        )
        .expect("imported reduce");

        assert!(manifest.imported_identity_verified);
        assert_eq!(
            manifest
                .imported_lemma_refs
                .get(&("ru".to_string(), "lemma-a".to_string())),
            Some(&0)
        );
        assert_eq!(manifest.lemma_count, 1);
        let mut reader = ReducedLemmaReaderV1::open(&manifest.path).expect("reduced reader");
        let first = reader
            .next_lemma()
            .expect("first read")
            .expect("first lemma");
        assert_eq!(first.lemma_id, 1);
        assert_eq!(
            first
                .forms
                .iter()
                .map(|form| form.form_ref)
                .collect::<Vec<_>>(),
            vec![0, 2]
        );
        assert!(reader.next_lemma().expect("eof").is_none());
        let reopened = reopen_imported_reduced_morphology(
            &manifest.path,
            &sorted.shards[0].path,
            &field,
            split_seed,
            sorted.compiler_version,
            sorted.normalization_version,
        )
        .expect("reopen sparse canonical refs");
        assert_eq!(reopened.lemma_count, 1);
        assert_eq!(reopened.form_count, 2);
        assert_eq!(reopened.imported_lemma_refs, manifest.imported_lemma_refs);
        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
