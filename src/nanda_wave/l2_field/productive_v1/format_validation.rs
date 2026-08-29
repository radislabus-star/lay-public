use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::marker::PhantomData;

use super::format::{
    ProductiveAlgorithmModeV1, ProductivePackageHeaderV1, ProductiveSectionKindV1,
    SectionDirectoryEntryV1, REQUIRED_SECTION_COUNT,
};
use super::records::{
    CalibrationCellRecordV1, DeltaManifestRecordV1, DirectionalResidualRecordV1,
    EvidencePriorRecordV1, FixedRecordV1, ModelCoefficientRecordV1, MorphOpRecordV1, MorphOpcodeV1,
    MorphProgramHeaderRecordV1, ParadigmCenterRecordV1, ParadigmCompatibilityIndexRecordV1,
    ParadigmPostingRecordV1, PhaseCenterRecordV1, ProductiveTerminalRecordV1,
    ProductiveTrieArcOpcodeV1, ProductiveTrieArcRecordV1, ProductiveTrieNodeRecordV1,
    ProvenanceRecordV1, SlotPhaseProfileRecordV1, PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE,
};
use super::score::PRODUCTIVE_FEATURE_COUNT;
use super::types::{LemmaParadigmBindingV1, MorphologySlotKeyV1, MORPHOLOGY_AXIS_COUNT};

const SEGMENT_POOL_MAGIC: &[u8; 4] = b"SPV1";
const AXIS_DICTIONARY_MAGIC: &[u8; 4] = b"ADV1";
const AXIS_LABEL_KIND: u8 = 1;
const OBSERVED_SLOT_SET_KIND: u8 = 2;

#[derive(Clone, Copy)]
struct FixedRecordViewV1<'a, T> {
    bytes: &'a [u8],
    count: usize,
    marker: PhantomData<T>,
}

impl<'a, T: FixedRecordV1> FixedRecordViewV1<'a, T> {
    fn new(bytes: &'a [u8], count: u32) -> Result<Self, String> {
        let count = count as usize;
        if count
            .checked_mul(T::BYTES)
            .is_none_or(|expected| expected != bytes.len())
        {
            return Err("productive fixed-record view count/bytes mismatch".to_string());
        }
        Ok(Self {
            bytes,
            count,
            marker: PhantomData,
        })
    }

    const fn len(&self) -> usize {
        self.count
    }

    const fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn get(&self, index: usize) -> Result<T, String> {
        if index >= self.count {
            return Err("productive fixed-record index lies outside section".to_string());
        }
        let start = index
            .checked_mul(T::BYTES)
            .ok_or_else(|| "productive fixed-record offset overflow".to_string())?;
        T::decode_record(&self.bytes[start..start + T::BYTES]).map_err(str::to_string)
    }
}

#[derive(Clone, Copy)]
struct PoolEntryV1<'a> {
    offset: u32,
    scalar_length: u16,
    payload: &'a [u8],
}

struct CheckedPoolV1<'a> {
    entries: Vec<PoolEntryV1<'a>>,
}

impl<'a> CheckedPoolV1<'a> {
    fn parse(bytes: &'a [u8], magic: &[u8; 4], directory_count: u32) -> Result<Self, String> {
        if bytes.len() < 8 || &bytes[0..4] != magic {
            return Err("productive variable pool magic or header is invalid".to_string());
        }
        let entry_count = read_u32(bytes, 4)?;
        if entry_count != directory_count {
            return Err("productive variable pool count disagrees with directory".to_string());
        }
        let mut entries = Vec::with_capacity(entry_count as usize);
        let mut offset = 8_usize;
        for _ in 0..entry_count {
            if !offset.is_multiple_of(8) || offset > u32::MAX as usize {
                return Err(
                    "productive variable pool entry alignment or offset is invalid".to_string(),
                );
            }
            let byte_length = read_u32(bytes, offset)? as usize;
            let scalar_length = read_u16(bytes, offset + 4)?;
            let flags = read_u16(bytes, offset + 6)?;
            if flags != 0 {
                return Err("productive variable pool entry has unknown flags".to_string());
            }
            let payload_start = offset
                .checked_add(8)
                .ok_or_else(|| "productive variable pool payload offset overflow".to_string())?;
            let payload_end = payload_start
                .checked_add(byte_length)
                .ok_or_else(|| "productive variable pool payload range overflow".to_string())?;
            let aligned_end = align8(payload_end)?;
            if aligned_end > bytes.len() {
                return Err("productive variable pool entry lies outside section".to_string());
            }
            if bytes[payload_end..aligned_end]
                .iter()
                .any(|byte| *byte != 0)
            {
                return Err("productive variable pool alignment padding is not zero".to_string());
            }
            entries.push(PoolEntryV1 {
                offset: offset as u32,
                scalar_length,
                payload: &bytes[payload_start..payload_end],
            });
            offset = aligned_end;
        }
        if offset != bytes.len() {
            return Err("productive variable pool has unowned trailing bytes".to_string());
        }
        Ok(Self { entries })
    }

    fn entry(&self, reference: u32) -> Option<PoolEntryV1<'a>> {
        self.entries
            .binary_search_by_key(&reference, |entry| entry.offset)
            .ok()
            .map(|index| self.entries[index])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AxisEntryKindV1 {
    Label,
    ObservedSlotSet,
}

struct CheckedAxisPoolV1<'a> {
    pool: CheckedPoolV1<'a>,
    kinds: Vec<AxisEntryKindV1>,
    labels: BTreeSet<(u8, u8)>,
}

impl<'a> CheckedAxisPoolV1<'a> {
    fn parse(bytes: &'a [u8], count: u32, slot_count: usize) -> Result<Self, String> {
        let pool = CheckedPoolV1::parse(bytes, AXIS_DICTIONARY_MAGIC, count)?;
        let mut kinds = Vec::with_capacity(pool.entries.len());
        let mut labels = BTreeSet::new();
        let mut previous_label: Option<(u8, u8, Vec<u8>)> = None;
        for entry in &pool.entries {
            let kind = *entry
                .payload
                .first()
                .ok_or_else(|| "productive axis dictionary entry is empty".to_string())?;
            match kind {
                AXIS_LABEL_KIND => {
                    if entry.payload.len() <= 4 || entry.payload[3] != 0 {
                        return Err("productive axis label prefix or label is invalid".to_string());
                    }
                    let axis = entry.payload[1];
                    let value = entry.payload[2];
                    if usize::from(axis) >= MORPHOLOGY_AXIS_COUNT || value < 2 {
                        return Err(
                            "productive axis label identity is outside the typed axes".to_string()
                        );
                    }
                    let label = std::str::from_utf8(&entry.payload[4..])
                        .map_err(|_| "productive axis label is not UTF-8".to_string())?;
                    if label.chars().count() != usize::from(entry.scalar_length) {
                        return Err("productive axis label scalar length mismatch".to_string());
                    }
                    let order = (axis, value, label.as_bytes().to_vec());
                    if previous_label
                        .as_ref()
                        .is_some_and(|previous| previous >= &order)
                    {
                        return Err("productive axis labels are not strictly canonical".to_string());
                    }
                    if !labels.insert((axis, value)) {
                        return Err("productive axis dictionary repeats an axis value".to_string());
                    }
                    previous_label = Some(order);
                    kinds.push(AxisEntryKindV1::Label);
                }
                OBSERVED_SLOT_SET_KIND => {
                    if entry.scalar_length != 0
                        || entry.payload.len() < 8
                        || entry.payload[1..4] != [0; 3]
                    {
                        return Err("productive observed slot set envelope is invalid".to_string());
                    }
                    let slot_ids = observed_slot_ids(entry.payload)?;
                    if slot_ids.is_empty()
                        || slot_ids.windows(2).any(|pair| pair[0] >= pair[1])
                        || slot_ids
                            .iter()
                            .any(|slot_id| *slot_id == 0 || *slot_id as usize > slot_count)
                    {
                        return Err(
                            "productive observed slot set identities are invalid".to_string()
                        );
                    }
                    kinds.push(AxisEntryKindV1::ObservedSlotSet);
                }
                _ => return Err("productive axis dictionary has an unknown entry kind".to_string()),
            }
        }
        Ok(Self {
            pool,
            kinds,
            labels,
        })
    }

    fn require_observed_slot_set(&self, reference: u32) -> Result<Vec<u32>, String> {
        let index = self
            .pool
            .entries
            .binary_search_by_key(&reference, |entry| entry.offset)
            .map_err(|_| "productive observed slot set reference is invalid".to_string())?;
        if self.kinds[index] != AxisEntryKindV1::ObservedSlotSet {
            return Err(
                "productive observed slot set reference points to an axis label".to_string(),
            );
        }
        observed_slot_ids(self.pool.entries[index].payload)
    }
}

pub(super) fn validate_package_contents(
    bytes: &[u8],
    header: &ProductivePackageHeaderV1,
    directory: &BTreeMap<ProductiveSectionKindV1, SectionDirectoryEntryV1>,
) -> Result<(), String> {
    let slots =
        fixed_section::<MorphologySlotKeyV1>(bytes, directory, ProductiveSectionKindV1::SlotKeys)?;
    let paradigms = fixed_section::<ParadigmCenterRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::ParadigmCenters,
    )?;
    let bindings = fixed_section::<LemmaParadigmBindingV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::LemmaBindings,
    )?;
    let compatibility = fixed_section::<ParadigmCompatibilityIndexRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::ParadigmCompatibilityIndex,
    )?;
    let postings = fixed_section::<ParadigmPostingRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::ParadigmPostings,
    )?;
    let programs = fixed_section::<MorphProgramHeaderRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::MorphProgramHeaders,
    )?;
    let operations = fixed_section::<MorphOpRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::MorphOperations,
    )?;
    let nodes = fixed_section::<ProductiveTrieNodeRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::TrieNodes,
    )?;
    let arcs = fixed_section::<ProductiveTrieArcRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::TrieArcs,
    )?;
    let terminals = fixed_section::<ProductiveTerminalRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::Terminals,
    )?;
    let profiles = fixed_section::<SlotPhaseProfileRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::SlotPhaseProfiles,
    )?;
    let positive = fixed_section::<PhaseCenterRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::PositivePhaseCenters,
    )?;
    let anti = fixed_section::<PhaseCenterRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::AntiPhaseCenters,
    )?;
    let hard_negative = fixed_section::<PhaseCenterRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::HardNegativePhaseCenters,
    )?;
    let ambiguity = fixed_section::<PhaseCenterRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::AmbiguityPhaseCenters,
    )?;
    let residuals = fixed_section::<DirectionalResidualRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::DirectionalResiduals,
    )?;
    let coefficients = fixed_section::<ModelCoefficientRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::ModelCoefficients,
    )?;
    let evidence_priors = fixed_section::<EvidencePriorRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::EvidencePriors,
    )?;
    let calibration = fixed_section::<CalibrationCellRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::CalibrationCells,
    )?;
    let provenance =
        fixed_section::<ProvenanceRecordV1>(bytes, directory, ProductiveSectionKindV1::Provenance)?;
    let delta_manifest = fixed_section::<DeltaManifestRecordV1>(
        bytes,
        directory,
        ProductiveSectionKindV1::DeltaManifest,
    )?;

    let segment_entry = directory_entry(directory, ProductiveSectionKindV1::SegmentPool)?;
    let segment_pool = CheckedPoolV1::parse(
        section_bytes(bytes, segment_entry),
        SEGMENT_POOL_MAGIC,
        segment_entry.count,
    )?;
    validate_segment_pool(&segment_pool, header.maximum_generated_scalars)?;
    let axis_entry = directory_entry(directory, ProductiveSectionKindV1::AxisDictionaries)?;
    let axis_pool = CheckedAxisPoolV1::parse(
        section_bytes(bytes, axis_entry),
        axis_entry.count,
        slots.len(),
    )?;

    validate_header_semantics(header, paradigms.len(), programs.len())?;
    validate_slots(slots, &axis_pool)?;
    validate_calibration(calibration)?;
    validate_coefficients(header.mode, coefficients)?;
    validate_evidence_priors(header.mode, evidence_priors)?;
    validate_provenance(provenance)?;
    validate_delta_manifest(
        header.mode,
        delta_manifest,
        coefficients.len(),
        calibration.len(),
    )?;

    let mut program_owner_paradigm = vec![0_u32; programs.len()];
    let mut program_owner_kind = vec![0_u8; programs.len()];
    let mut profile_owner_paradigm = vec![0_u32; profiles.len()];
    let roots = validate_paradigms(
        paradigms,
        programs.len(),
        profiles.len(),
        nodes.len(),
        calibration.len(),
        provenance.len(),
        &mut program_owner_paradigm,
        &mut program_owner_kind,
        &mut profile_owner_paradigm,
    )?;
    validate_bindings(
        bindings,
        paradigms.len(),
        programs.len(),
        provenance.len(),
        &axis_pool,
        &mut program_owner_paradigm,
        &mut program_owner_kind,
    )?;
    require_complete_ownership(&program_owner_kind, "morph program")?;
    require_complete_ownership_u32(&profile_owner_paradigm, "slot phase profile")?;

    validate_compatibility(compatibility, postings, slots, paradigms)?;
    let program_terminators = validate_programs(
        programs,
        operations,
        slots,
        paradigms,
        header.maximum_program_operations,
        &segment_pool,
        &program_owner_paradigm,
        &program_owner_kind,
    )?;
    validate_phase_profiles(
        profiles,
        positive,
        anti,
        hard_negative,
        ambiguity,
        slots,
        paradigms,
        calibration.len(),
        &profile_owner_paradigm,
    )?;
    validate_residuals(residuals, slots.len())?;
    validate_trie(
        nodes,
        arcs,
        terminals,
        programs,
        &roots,
        &program_owner_paradigm,
        &program_terminators,
        calibration.len(),
        provenance.len(),
        &segment_pool,
        header.mode,
    )?;
    Ok(())
}

fn fixed_section<'a, T: FixedRecordV1>(
    bytes: &'a [u8],
    directory: &BTreeMap<ProductiveSectionKindV1, SectionDirectoryEntryV1>,
    kind: ProductiveSectionKindV1,
) -> Result<FixedRecordViewV1<'a, T>, String> {
    let entry = directory_entry(directory, kind)?;
    FixedRecordViewV1::new(section_bytes(bytes, entry), entry.count)
}

fn directory_entry(
    directory: &BTreeMap<ProductiveSectionKindV1, SectionDirectoryEntryV1>,
    kind: ProductiveSectionKindV1,
) -> Result<SectionDirectoryEntryV1, String> {
    directory
        .get(&kind)
        .copied()
        .ok_or_else(|| "productive package lacks a validated required section".to_string())
}

fn section_bytes(bytes: &[u8], entry: SectionDirectoryEntryV1) -> &[u8] {
    &bytes[entry.offset as usize..(entry.offset + entry.bytes) as usize]
}

fn validate_header_semantics(
    header: &ProductivePackageHeaderV1,
    paradigm_count: usize,
    program_count: usize,
) -> Result<(), String> {
    let has_model_rows = paradigm_count != 0 || program_count != 0;
    if has_model_rows
        && (header.l11_package_sha256 == [0; 32]
            || header.canonical_l2_package_sha256 == [0; 32]
            || header.training_manifest_sha256 == [0; 32]
            || header.maximum_observed_scalars == 0
            || header.maximum_generated_scalars == 0
            || header.maximum_program_operations == 0
            || header.normalization_version == 0
            || header.compiler_version == 0)
    {
        return Err("productive model header lacks a bound, fingerprint, or version".to_string());
    }
    Ok(())
}

fn validate_segment_pool(
    pool: &CheckedPoolV1<'_>,
    maximum_generated_scalars: u16,
) -> Result<(), String> {
    let mut previous: Option<&[u8]> = None;
    for entry in &pool.entries {
        let value = std::str::from_utf8(entry.payload)
            .map_err(|_| "productive segment pool entry is not UTF-8".to_string())?;
        let scalar_length = value.chars().count();
        if value.is_empty()
            || scalar_length != usize::from(entry.scalar_length)
            || (maximum_generated_scalars != 0
                && scalar_length > usize::from(maximum_generated_scalars))
        {
            return Err("productive segment pool scalar length is invalid".to_string());
        }
        if previous.is_some_and(|prior| prior >= entry.payload) {
            return Err(
                "productive segment pool is not strictly sorted and deduplicated".to_string(),
            );
        }
        previous = Some(entry.payload);
    }
    Ok(())
}

fn validate_slots(
    slots: FixedRecordViewV1<'_, MorphologySlotKeyV1>,
    axis_pool: &CheckedAxisPoolV1<'_>,
) -> Result<(), String> {
    let mut previous: Option<[u8; 16]> = None;
    for index in 0..slots.len() {
        let slot = slots.get(index)?;
        let bytes = slot.to_bytes();
        if slot.pos_domain() == 0 || previous.is_some_and(|prior| prior >= bytes) {
            return Err("productive morphology slots are not canonical or have no POS".to_string());
        }
        for (axis, value) in slot.axes().into_iter().enumerate() {
            if value >= 2 && !axis_pool.labels.contains(&(axis as u8, value)) {
                return Err("productive morphology slot uses an undefined axis value".to_string());
            }
        }
        previous = Some(bytes);
    }
    Ok(())
}

fn validate_calibration(
    calibration: FixedRecordViewV1<'_, CalibrationCellRecordV1>,
) -> Result<(), String> {
    if calibration.len() > u16::MAX as usize {
        return Err(
            "productive calibration section exceeds terminal row identity width".to_string(),
        );
    }
    let mut previous_key = 0_u32;
    for index in 0..calibration.len() {
        let cell = calibration.get(index)?;
        if cell.stratum_key_id == 0
            || cell.stratum_key_id <= previous_key
            || cell.support == 0
            || cell.correct_winner_count > cell.support
            || cell.false_winner_count > cell.support
            || cell.tied_count > cell.support
        {
            return Err("productive calibration identity or denominator is invalid".to_string());
        }
        previous_key = cell.stratum_key_id;
    }
    Ok(())
}

fn validate_coefficients(
    mode: ProductiveAlgorithmModeV1,
    coefficients: FixedRecordViewV1<'_, ModelCoefficientRecordV1>,
) -> Result<(), String> {
    match mode {
        ProductiveAlgorithmModeV1::V39SpeedParity if !coefficients.is_empty() => {
            return Err(
                "V39 speed-parity package contains productive model coefficients".to_string(),
            );
        }
        ProductiveAlgorithmModeV1::ProductiveV1Model
            if coefficients.len() != PRODUCTIVE_FEATURE_COUNT =>
        {
            return Err(
                "productive model package lacks the complete coefficient vector".to_string(),
            );
        }
        _ => {}
    }
    let mut schema_hash = None;
    for index in 0..coefficients.len() {
        let coefficient = coefficients.get(index)?;
        if coefficient.feature_id as usize != index + 1
            || coefficient.train_support == 0
            || coefficient.feature_schema_hash_low == 0
            || schema_hash.is_some_and(|hash| hash != coefficient.feature_schema_hash_low)
        {
            return Err(
                "productive coefficient identity, support, or schema is invalid".to_string(),
            );
        }
        schema_hash = Some(coefficient.feature_schema_hash_low);
    }
    Ok(())
}

fn validate_evidence_priors(
    mode: ProductiveAlgorithmModeV1,
    priors: FixedRecordViewV1<'_, EvidencePriorRecordV1>,
) -> Result<(), String> {
    match mode {
        ProductiveAlgorithmModeV1::V39SpeedParity if !priors.is_empty() => {
            return Err("V39 speed-parity package contains productive evidence priors".to_string());
        }
        ProductiveAlgorithmModeV1::ProductiveV1Model if priors.len() != 4 => {
            return Err("productive model package lacks four evidence prior channels".to_string());
        }
        _ => {}
    }
    for index in 0..priors.len() {
        let prior = priors.get(index)?;
        if prior.channel_id as usize != index + 1 {
            return Err("productive evidence prior channels are not canonical".to_string());
        }
    }
    Ok(())
}

fn validate_provenance(
    provenance: FixedRecordViewV1<'_, ProvenanceRecordV1>,
) -> Result<(), String> {
    let mut previous = None;
    for index in 0..provenance.len() {
        let record = provenance.get(index)?;
        let key = (record.source_kind, record.source_id, record.event_start);
        if record.event_count == 0
            || record
                .event_start
                .checked_add(u64::from(record.event_count))
                .is_none()
            || previous.is_some_and(|prior| prior >= key)
        {
            return Err("productive provenance order or event range is invalid".to_string());
        }
        previous = Some(key);
    }
    Ok(())
}

fn validate_delta_manifest(
    mode: ProductiveAlgorithmModeV1,
    manifests: FixedRecordViewV1<'_, DeltaManifestRecordV1>,
    coefficient_count: usize,
    calibration_count: usize,
) -> Result<(), String> {
    if manifests.len() > 1
        || (mode == ProductiveAlgorithmModeV1::ProductiveV1Model && manifests.len() != 1)
    {
        return Err("productive package must contain at most one base delta manifest".to_string());
    }
    if manifests.is_empty() {
        return Ok(());
    }
    let manifest = manifests.get(0)?;
    if manifest.section_count_ref != REQUIRED_SECTION_COUNT as u64
        || manifest.coefficient_generation != manifest.calibration_generation
        || manifest.requested_authority_scope == 0
        || (manifest.requested_authority_scope == 3 && manifest.proof_receipt_sha256 == [0; 32])
        || (coefficient_count == 0 && manifest.coefficient_generation != 0)
        || (calibration_count == 0 && manifest.calibration_generation != 0)
    {
        return Err(
            "productive delta manifest generation or authority scope is invalid".to_string(),
        );
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn validate_paradigms(
    paradigms: FixedRecordViewV1<'_, ParadigmCenterRecordV1>,
    program_count: usize,
    profile_count: usize,
    node_count: usize,
    calibration_count: usize,
    provenance_count: usize,
    program_owner_paradigm: &mut [u32],
    program_owner_kind: &mut [u8],
    profile_owner_paradigm: &mut [u32],
) -> Result<Vec<u32>, String> {
    let mut roots = Vec::with_capacity(paradigms.len());
    let mut root_set = BTreeSet::new();
    for index in 0..paradigms.len() {
        let paradigm_id = (index + 1) as u32;
        let center = paradigms.get(index)?;
        if center.support == 0
            || center.root_node as usize >= node_count
            || center.calibration_class as usize > calibration_count
            || !valid_optional_ref(center.provenance_ref, provenance_count)
            || center.program_count == 0
            || center.transition_count == 0
        {
            return Err(
                "productive paradigm center has an invalid root, support, or reference".to_string(),
            );
        }
        let program_range = checked_range(
            center.program_start,
            center.program_count,
            program_count,
            "paradigm program",
        )?;
        let transition_range = checked_range(
            center.transition_start,
            center.transition_count,
            program_count,
            "paradigm transition",
        )?;
        if transition_range.start < program_range.start || transition_range.end > program_range.end
        {
            return Err(
                "productive paradigm transition range is outside its program range".to_string(),
            );
        }
        claim_typed_range(
            program_owner_paradigm,
            program_owner_kind,
            program_range,
            paradigm_id,
            1,
            "paradigm program",
        )?;
        let profile_range = checked_range(
            center.slot_profile_start,
            center.slot_profile_count,
            profile_count,
            "paradigm phase profile",
        )?;
        claim_u32_range(
            profile_owner_paradigm,
            profile_range,
            paradigm_id,
            "paradigm phase profile",
        )?;
        if !root_set.insert(center.root_node) {
            return Err("productive paradigms repeat a trie root".to_string());
        }
        roots.push(center.root_node);
    }
    Ok(roots)
}

fn validate_bindings(
    bindings: FixedRecordViewV1<'_, LemmaParadigmBindingV1>,
    paradigm_count: usize,
    program_count: usize,
    provenance_count: usize,
    axis_pool: &CheckedAxisPoolV1<'_>,
    program_owner_paradigm: &mut [u32],
    program_owner_kind: &mut [u8],
) -> Result<(), String> {
    let mut previous = None;
    for index in 0..bindings.len() {
        let binding = bindings.get(index)?;
        let key = (binding.lemma_id, binding.paradigm_id);
        if binding.paradigm_id as usize > paradigm_count
            || binding.positive_support == 0
            || !valid_optional_ref(binding.provenance_ref, provenance_count)
            || previous.is_some_and(|prior| prior >= key)
        {
            return Err(
                "productive lemma binding order, support, or reference is invalid".to_string(),
            );
        }
        axis_pool.require_observed_slot_set(binding.observed_slot_set_ref)?;
        let range = checked_range(
            binding.program_start,
            u32::from(binding.program_count),
            program_count,
            "lemma-local program",
        )?;
        claim_typed_range(
            program_owner_paradigm,
            program_owner_kind,
            range,
            binding.paradigm_id,
            2,
            "lemma-local program",
        )?;
        previous = Some(key);
    }
    Ok(())
}

fn validate_compatibility(
    indexes: FixedRecordViewV1<'_, ParadigmCompatibilityIndexRecordV1>,
    postings: FixedRecordViewV1<'_, ParadigmPostingRecordV1>,
    slots: FixedRecordViewV1<'_, MorphologySlotKeyV1>,
    paradigms: FixedRecordViewV1<'_, ParadigmCenterRecordV1>,
) -> Result<(), String> {
    let mut posting_owner = vec![0_u8; postings.len()];
    let mut previous = None;
    for index in 0..indexes.len() {
        let row = indexes.get(index)?;
        let key = (row.pos_domain, row.source_slot_id);
        let slot = one_based(slots, row.source_slot_id, "compatibility source slot")?;
        if u16::from(slot.pos_domain()) != row.pos_domain
            || previous.is_some_and(|prior| prior >= key)
        {
            return Err("productive compatibility index order or POS is invalid".to_string());
        }
        let range = checked_range(
            row.posting_start,
            row.posting_count,
            postings.len(),
            "compatibility posting",
        )?;
        claim_u8_range(&mut posting_owner, range.clone(), "compatibility posting")?;
        let mut prior_paradigm = 0_u32;
        for posting_index in range {
            let posting = postings.get(posting_index)?;
            let paradigm = one_based(paradigms, posting.paradigm_id, "compatibility paradigm")?;
            if posting.paradigm_id <= prior_paradigm || paradigm.pos_domain != row.pos_domain {
                return Err(
                    "productive compatibility postings are unsorted or cross POS".to_string(),
                );
            }
            prior_paradigm = posting.paradigm_id;
        }
        previous = Some(key);
    }
    require_complete_ownership(&posting_owner, "compatibility posting")
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn validate_programs(
    programs: FixedRecordViewV1<'_, MorphProgramHeaderRecordV1>,
    operations: FixedRecordViewV1<'_, MorphOpRecordV1>,
    slots: FixedRecordViewV1<'_, MorphologySlotKeyV1>,
    paradigms: FixedRecordViewV1<'_, ParadigmCenterRecordV1>,
    maximum_program_operations: u16,
    segment_pool: &CheckedPoolV1<'_>,
    program_owner_paradigm: &[u32],
    program_owner_kind: &[u8],
) -> Result<Vec<(u32, u16)>, String> {
    let mut operation_owner = vec![0_u8; operations.len()];
    let mut terminators = Vec::with_capacity(programs.len());
    for index in 0..programs.len() {
        let program = programs.get(index)?;
        let source = one_based(slots, program.source_slot_id, "program source slot")?;
        let target = one_based(slots, program.target_slot_id, "program target slot")?;
        let paradigm = one_based(
            paradigms,
            program_owner_paradigm[index],
            "program owner paradigm",
        )?;
        if source.pos_domain() != target.pos_domain()
            || u16::from(source.pos_domain()) != paradigm.pos_domain
            || program.op_count > maximum_program_operations
        {
            return Err("productive program POS or operation bound is invalid".to_string());
        }
        let range = checked_range(
            program.op_start,
            u32::from(program.op_count),
            operations.len(),
            "program operation",
        )?;
        claim_u8_range(&mut operation_owner, range.clone(), "program operation")?;
        let mut terminal = None;
        let mut exact_allomorph = false;
        for operation_index in range.clone() {
            let operation = operations.get(operation_index)?;
            let opcode = operation.decoded_opcode().map_err(str::to_string)?;
            match opcode {
                MorphOpcodeV1::EmitSegment => require_segment(segment_pool, operation.arg1)?,
                MorphOpcodeV1::ReplaceSourceRange if operation.arg2 != 0 => {
                    require_segment(segment_pool, operation.arg2)?
                }
                MorphOpcodeV1::EmitExactAllomorph => {
                    require_segment(segment_pool, operation.arg1)?;
                    exact_allomorph = true;
                }
                MorphOpcodeV1::Terminate => {
                    if operation_index + 1 != range.end || operation.arg1 != program.target_slot_id
                    {
                        return Err(
                            "productive program TERMINATE is not final or changes target slot"
                                .to_string(),
                        );
                    }
                    terminal = Some((operation.arg1, operation.arg2 as u16));
                }
                _ => {}
            }
            if operation_index + 1 != range.end && opcode == MorphOpcodeV1::Terminate {
                return Err("productive program contains an early TERMINATE".to_string());
            }
        }
        if exact_allomorph {
            if program_owner_kind[index] != 2 || range.len() != 2 {
                return Err(
                    "productive exact allomorph is not an isolated lemma-local program".to_string(),
                );
            }
            let first = operations.get(range.start)?;
            if first.decoded_opcode().map_err(str::to_string)? != MorphOpcodeV1::EmitExactAllomorph
            {
                return Err(
                    "productive exact allomorph has another emitting instruction".to_string(),
                );
            }
        }
        terminators.push(terminal.ok_or_else(|| "productive program lacks TERMINATE".to_string())?);
    }
    require_complete_ownership(&operation_owner, "morph operation")?;
    Ok(terminators)
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn validate_phase_profiles(
    profiles: FixedRecordViewV1<'_, SlotPhaseProfileRecordV1>,
    positive: FixedRecordViewV1<'_, PhaseCenterRecordV1>,
    anti: FixedRecordViewV1<'_, PhaseCenterRecordV1>,
    hard_negative: FixedRecordViewV1<'_, PhaseCenterRecordV1>,
    ambiguity: FixedRecordViewV1<'_, PhaseCenterRecordV1>,
    slots: FixedRecordViewV1<'_, MorphologySlotKeyV1>,
    paradigms: FixedRecordViewV1<'_, ParadigmCenterRecordV1>,
    calibration_count: usize,
    profile_owner_paradigm: &[u32],
) -> Result<(), String> {
    let mut positive_owner = vec![0_u8; positive.len()];
    let mut anti_owner = vec![0_u8; anti.len()];
    let mut hard_owner = vec![0_u8; hard_negative.len()];
    let mut ambiguity_owner = vec![0_u8; ambiguity.len()];
    for (index, paradigm_id) in profile_owner_paradigm
        .iter()
        .copied()
        .enumerate()
        .take(profiles.len())
    {
        let profile = profiles.get(index)?;
        let slot = one_based(slots, profile.slot_id, "phase profile slot")?;
        let paradigm = one_based(paradigms, paradigm_id, "phase profile paradigm")?;
        if profile.support == 0
            || profile.calibration_class as usize > calibration_count
            || u16::from(slot.pos_domain()) != paradigm.pos_domain
        {
            return Err(
                "productive slot phase profile support, calibration, or POS is invalid".to_string(),
            );
        }
        validate_phase_range(
            positive,
            &mut positive_owner,
            profile.positive_start,
            profile.positive_count,
            1,
            "positive phase center",
        )?;
        validate_phase_range(
            anti,
            &mut anti_owner,
            profile.anti_start,
            profile.anti_count,
            -1,
            "anti phase center",
        )?;
        validate_phase_range(
            hard_negative,
            &mut hard_owner,
            profile.hard_negative_start,
            profile.hard_negative_count,
            -2,
            "hard-negative phase center",
        )?;
        validate_phase_range(
            ambiguity,
            &mut ambiguity_owner,
            profile.ambiguity_start,
            profile.ambiguity_count,
            0,
            "ambiguity phase center",
        )?;
    }
    require_complete_ownership(&positive_owner, "positive phase center")?;
    require_complete_ownership(&anti_owner, "anti phase center")?;
    require_complete_ownership(&hard_owner, "hard-negative phase center")?;
    require_complete_ownership(&ambiguity_owner, "ambiguity phase center")
}

fn validate_phase_range(
    centers: FixedRecordViewV1<'_, PhaseCenterRecordV1>,
    owners: &mut [u8],
    start: u32,
    count: u16,
    polarity: i8,
    label: &str,
) -> Result<(), String> {
    let range = checked_range(start, u32::from(count), centers.len(), label)?;
    claim_u8_range(owners, range.clone(), label)?;
    for index in range {
        let center = centers.get(index)?;
        if center.polarity != polarity || center.context_mode_id == 0 {
            return Err(format!(
                "productive {label} section or context identity mismatch"
            ));
        }
    }
    Ok(())
}

fn validate_residuals(
    residuals: FixedRecordViewV1<'_, DirectionalResidualRecordV1>,
    slot_count: usize,
) -> Result<(), String> {
    let mut previous = None;
    for index in 0..residuals.len() {
        let residual = residuals.get(index)?;
        let key = (
            residual.source_scene_key,
            residual.from_slot_id,
            residual.to_slot_id,
        );
        if residual.from_slot_id as usize > slot_count
            || residual.to_slot_id as usize > slot_count
            || (residual.positive_support == 0 && residual.explicit_anti_support == 0)
            || previous.is_some_and(|prior| prior >= key)
        {
            return Err(
                "productive directional residual order, slot, or support is invalid".to_string(),
            );
        }
        previous = Some(key);
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn validate_trie(
    nodes: FixedRecordViewV1<'_, ProductiveTrieNodeRecordV1>,
    arcs: FixedRecordViewV1<'_, ProductiveTrieArcRecordV1>,
    terminals: FixedRecordViewV1<'_, ProductiveTerminalRecordV1>,
    programs: FixedRecordViewV1<'_, MorphProgramHeaderRecordV1>,
    roots: &[u32],
    program_owner_paradigm: &[u32],
    program_terminators: &[(u32, u16)],
    calibration_count: usize,
    provenance_count: usize,
    segment_pool: &CheckedPoolV1<'_>,
    mode: ProductiveAlgorithmModeV1,
) -> Result<(), String> {
    if nodes.is_empty() {
        if !roots.is_empty() || !arcs.is_empty() || !terminals.is_empty() {
            return Err("productive trie has edges, terminals, or roots without nodes".to_string());
        }
        return Ok(());
    }
    let mut arc_owner = vec![0_u8; arcs.len()];
    let mut terminal_owner = vec![0_u8; terminals.len()];
    let mut parent_count = vec![0_u8; nodes.len()];
    for node_index in 0..nodes.len() {
        let node = nodes.get(node_index)?;
        let arc_range = checked_range(
            node.arc_start,
            u32::from(node.arc_count),
            arcs.len(),
            "trie arc",
        )?;
        claim_u8_range(&mut arc_owner, arc_range.clone(), "trie arc")?;
        let mut prior_order = None;
        for arc_index in arc_range {
            let arc = arcs.get(arc_index)?;
            let opcode = arc.decoded_opcode().map_err(str::to_string)?;
            if arc.child_node as usize >= nodes.len()
                || prior_order.is_some_and(|prior| prior >= arc.stable_order)
            {
                return Err("productive trie arc child or order is invalid".to_string());
            }
            if opcode == ProductiveTrieArcOpcodeV1::EmitSegment {
                require_segment(segment_pool, arc.arg1)?
            }
            parent_count[arc.child_node as usize] = parent_count[arc.child_node as usize]
                .checked_add(1)
                .ok_or_else(|| "productive trie parent count overflow".to_string())?;
            if parent_count[arc.child_node as usize] > 1 {
                return Err("productive trie node has multiple parents".to_string());
            }
            prior_order = Some(arc.stable_order);
        }
        let terminal_range = checked_range(
            node.terminal_start,
            u32::from(node.terminal_count),
            terminals.len(),
            "trie terminal",
        )?;
        claim_u8_range(&mut terminal_owner, terminal_range, "trie terminal")?;
    }
    require_complete_ownership(&arc_owner, "trie arc")?;
    require_complete_ownership(&terminal_owner, "trie terminal")?;

    let mut root_owner = vec![0_u32; nodes.len()];
    let mut queue = VecDeque::new();
    for (index, root) in roots.iter().copied().enumerate() {
        if root as usize >= nodes.len()
            || parent_count[root as usize] != 0
            || root_owner[root as usize] != 0
        {
            return Err("productive trie root is invalid, repeated, or has a parent".to_string());
        }
        root_owner[root as usize] = (index + 1) as u32;
        queue.push_back(root);
    }
    while let Some(node_id) = queue.pop_front() {
        let owner = root_owner[node_id as usize];
        let node = nodes.get(node_id as usize)?;
        let range = checked_range(
            node.arc_start,
            u32::from(node.arc_count),
            arcs.len(),
            "trie traversal arc",
        )?;
        for arc_index in range {
            let child = arcs.get(arc_index)?.child_node;
            if root_owner[child as usize] != 0 {
                return Err("productive trie contains a cycle or cross-root child".to_string());
            }
            root_owner[child as usize] = owner;
            queue.push_back(child);
        }
    }
    let root_set = roots.iter().copied().collect::<BTreeSet<_>>();
    for index in 0..nodes.len() {
        let is_root = root_set.contains(&(index as u32));
        if root_owner[index] == 0 || (!is_root && parent_count[index] != 1) {
            return Err("productive trie node is unreachable or lacks one parent".to_string());
        }
    }

    let mut terminal_identities = BTreeSet::new();
    let mut terminal_hashes = BTreeSet::new();
    for (node_index, owner) in root_owner.iter().copied().enumerate().take(nodes.len()) {
        let node = nodes.get(node_index)?;
        let range = checked_range(
            node.terminal_start,
            u32::from(node.terminal_count),
            terminals.len(),
            "trie terminal attribution",
        )?;
        let mut previous = None;
        for terminal_index in range {
            let terminal = terminals.get(terminal_index)?;
            let program = one_based(programs, terminal.program_id, "terminal program")?;
            let terminator = program_terminators[terminal.program_id as usize - 1];
            let identity = (
                terminal.program_id,
                terminal.target_slot_id,
                terminal.variant_id,
                terminal.decoder_ref,
                terminal.evidence_ref,
                terminal.calibration_class,
                terminal.provenance_ref,
                terminal.stable_identity_hash,
            );
            if program_owner_paradigm[terminal.program_id as usize - 1] != owner
                || program.target_slot_id != terminal.target_slot_id
                || terminator != (terminal.target_slot_id, terminal.variant_id)
                || terminal.calibration_class as usize > calibration_count
                || !valid_optional_ref(terminal.provenance_ref, provenance_count)
                || terminal.stable_identity_hash == 0
                || (mode == ProductiveAlgorithmModeV1::ProductiveV1Model
                    && terminal.flags & PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE == 0)
                || previous.is_some_and(|prior| prior >= identity)
                || !terminal_identities.insert(identity)
                || !terminal_hashes.insert(terminal.stable_identity_hash)
            {
                return Err(
                    "productive terminal ownership, identity, or reference is invalid".to_string(),
                );
            }
            if terminal.flags & PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE == 0 {
                require_segment(segment_pool, terminal.decoder_ref)?;
            }
            previous = Some(identity);
        }
    }
    Ok(())
}

fn require_segment(pool: &CheckedPoolV1<'_>, reference: u32) -> Result<(), String> {
    pool.entry(reference)
        .map(|_| ())
        .ok_or_else(|| "productive segment reference is invalid".to_string())
}

fn one_based<T: FixedRecordV1>(
    view: FixedRecordViewV1<'_, T>,
    identity: u32,
    label: &str,
) -> Result<T, String> {
    if identity == 0 || identity as usize > view.len() {
        return Err(format!(
            "productive {label} identity is outside its section"
        ));
    }
    view.get(identity as usize - 1)
}

fn checked_range(
    start: u32,
    count: u32,
    limit: usize,
    label: &str,
) -> Result<std::ops::Range<usize>, String> {
    let start = start as usize;
    let end = start
        .checked_add(count as usize)
        .ok_or_else(|| format!("productive {label} range overflow"))?;
    if end > limit {
        return Err(format!("productive {label} range lies outside its section"));
    }
    Ok(start..end)
}

fn claim_typed_range(
    paradigm_owners: &mut [u32],
    kind_owners: &mut [u8],
    range: std::ops::Range<usize>,
    paradigm_id: u32,
    kind: u8,
    label: &str,
) -> Result<(), String> {
    for index in range {
        if kind_owners[index] != 0 {
            return Err(format!("productive {label} range overlaps another owner"));
        }
        paradigm_owners[index] = paradigm_id;
        kind_owners[index] = kind;
    }
    Ok(())
}

fn claim_u32_range(
    owners: &mut [u32],
    range: std::ops::Range<usize>,
    owner: u32,
    label: &str,
) -> Result<(), String> {
    for index in range {
        if owners[index] != 0 {
            return Err(format!("productive {label} range overlaps another owner"));
        }
        owners[index] = owner;
    }
    Ok(())
}

fn claim_u8_range(
    owners: &mut [u8],
    range: std::ops::Range<usize>,
    label: &str,
) -> Result<(), String> {
    for index in range {
        if owners[index] != 0 {
            return Err(format!("productive {label} range overlaps another owner"));
        }
        owners[index] = 1;
    }
    Ok(())
}

fn require_complete_ownership(owners: &[u8], label: &str) -> Result<(), String> {
    if owners.contains(&0) {
        Err(format!(
            "productive {label} section contains an unowned row"
        ))
    } else {
        Ok(())
    }
}

fn require_complete_ownership_u32(owners: &[u32], label: &str) -> Result<(), String> {
    if owners.contains(&0) {
        Err(format!(
            "productive {label} section contains an unowned row"
        ))
    } else {
        Ok(())
    }
}

fn valid_optional_ref(reference: u32, count: usize) -> bool {
    reference == 0 || reference as usize <= count
}

fn observed_slot_ids(payload: &[u8]) -> Result<Vec<u32>, String> {
    let count = read_u32(payload, 4)? as usize;
    if count.checked_mul(4).and_then(|bytes| bytes.checked_add(8)) != Some(payload.len()) {
        return Err("productive observed slot set count/bytes mismatch".to_string());
    }
    (0..count)
        .map(|index| read_u32(payload, 8 + index * 4))
        .collect()
}

fn align8(value: usize) -> Result<usize, String> {
    value
        .checked_add(7)
        .map(|value| value & !7)
        .ok_or_else(|| "productive variable pool alignment overflow".to_string())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .map(|value| u16::from_le_bytes(value.try_into().expect("fixed slice")))
        .ok_or_else(|| "productive variable pool u16 lies outside section".to_string())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .map(|value| u32::from_le_bytes(value.try_into().expect("fixed slice")))
        .ok_or_else(|| "productive variable pool u32 lies outside section".to_string())
}
