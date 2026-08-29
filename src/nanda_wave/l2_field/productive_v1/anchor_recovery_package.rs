use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::super::package_bytes::PackageBytes;
use super::anchor_recovery_reduce::{
    read_anchor_recovery_definitions, AnchorRecoveryDefinitionV1, AnchorRecoveryManifestV1,
    ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED,
};
use super::induce::{
    EditOperationV1, ParadigmTransitionKeyV1, SourceAnchorV1, COPY_TO_RETAINED_EDGE,
};
use super::records::{
    encode_records, FixedRecordV1, MorphOpRecordV1, MorphOpcodeV1, MorphProgramHeaderRecordV1,
    ParadigmCompatibilityIndexRecordV1,
};
use super::runtime::resolve_source_offset;
use super::PRODUCTIVE_V1_SCHEMA_VERSION;

const MAGIC: [u8; 8] = *b"LAYARV1\0";
const HEADER_BYTES: usize = 256;
const MICRO_SIDECAR_BYTE_CEILING: u64 = 35 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AnchorRecoveryPostingRecordV1 {
    pub(super) paradigm_id: u32,
    pub(super) program_id: u32,
    pub(super) train_lemma_support: u32,
    pub(super) stability: u16,
    pub(super) flags: u16,
    pub(super) provenance_hash_low: u32,
}

impl FixedRecordV1 for AnchorRecoveryPostingRecordV1 {
    const BYTES: usize = 20;

    fn encode_record(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.paradigm_id.to_le_bytes());
        output.extend_from_slice(&self.program_id.to_le_bytes());
        output.extend_from_slice(&self.train_lemma_support.to_le_bytes());
        output.extend_from_slice(&self.stability.to_le_bytes());
        output.extend_from_slice(&self.flags.to_le_bytes());
        output.extend_from_slice(&self.provenance_hash_low.to_le_bytes());
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != Self::BYTES {
            return Err("anchor recovery posting width is invalid");
        }
        let record = Self {
            paradigm_id: u32::from_le_bytes(bytes[0..4].try_into().expect("paradigm")),
            program_id: u32::from_le_bytes(bytes[4..8].try_into().expect("program")),
            train_lemma_support: u32::from_le_bytes(bytes[8..12].try_into().expect("support")),
            stability: u16::from_le_bytes(bytes[12..14].try_into().expect("stability")),
            flags: u16::from_le_bytes(bytes[14..16].try_into().expect("flags")),
            provenance_hash_low: u32::from_le_bytes(bytes[16..20].try_into().expect("provenance")),
        };
        if record.paradigm_id == 0
            || record.program_id == 0
            || (record.train_lemma_support < 2
                && !(record.train_lemma_support == 1
                    && record.flags == ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED))
            || (record.train_lemma_support >= 2 && record.flags != 0)
            || record.flags & !ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED != 0
        {
            return Err("anchor recovery posting identity or evidence is invalid");
        }
        Ok(record)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledAnchorRecoveryPackageV1 {
    pub(super) path: PathBuf,
    pub(super) package_sha256: [u8; 32],
    pub(super) package_bytes: u64,
    pub(super) index_count: u32,
    pub(super) posting_count: u32,
    pub(super) shared_support_certified_posting_count: u32,
    pub(super) program_count: u32,
    pub(super) operation_count: u32,
}

#[derive(Clone, Copy, Debug)]
struct AnchorRecoveryHeaderV1 {
    total_bytes: u64,
    index_offset: u64,
    posting_offset: u64,
    program_offset: u64,
    operation_offset: u64,
    segment_offset: u64,
    index_count: u32,
    posting_count: u32,
    program_count: u32,
    operation_count: u32,
    segment_bytes: u32,
    maximum_observed_scalars: u16,
    maximum_generated_scalars: u16,
    maximum_program_operations: u16,
    split_seed: u64,
    base_package_sha256: [u8; 32],
    axis_schema_sha256: [u8; 32],
    evidence_sha256: [u8; 32],
    payload_sha256: [u8; 32],
}

#[derive(Clone, Debug)]
pub(super) struct AnchorRecoveryPackageViewV1 {
    bytes: PackageBytes,
    header: AnchorRecoveryHeaderV1,
    indexes: Box<[ParadigmCompatibilityIndexRecordV1]>,
    postings: Box<[AnchorRecoveryPostingRecordV1]>,
    programs: Box<[PreparedAnchorRecoveryProgramV1]>,
    operations: Box<[MorphOpRecordV1]>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PreparedAnchorRecoveryProgramV1 {
    record: MorphProgramHeaderRecordV1,
    suffix_drop: u16,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AnchorRecoveryPathV1 {
    pub(super) posting: AnchorRecoveryPostingRecordV1,
    pub(super) program: MorphProgramHeaderRecordV1,
    suffix_drop: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AnchorRecoveryProgramKeyV1 {
    pos_domain: u8,
    source_slot_id: u32,
    canonical_anchor_slot_id: u32,
    transition: ParadigmTransitionKeyV1,
}

impl AnchorRecoveryProgramKeyV1 {
    fn from_definition(definition: &AnchorRecoveryDefinitionV1) -> Self {
        Self {
            pos_domain: definition.pos_domain,
            source_slot_id: definition.source_slot_id,
            canonical_anchor_slot_id: definition.canonical_anchor_slot_id,
            transition: definition.transition.clone(),
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
pub(super) fn compile_anchor_recovery_package(
    manifest: &AnchorRecoveryManifestV1,
    output_path: &Path,
    base_package_sha256: [u8; 32],
    split_seed: u64,
    maximum_observed_scalars: u16,
    maximum_generated_scalars: u16,
    total_package_byte_budget: u64,
    base_package_bytes: u64,
) -> Result<CompiledAnchorRecoveryPackageV1, String> {
    let definitions = read_anchor_recovery_definitions(manifest)?;
    let mut segments = BTreeSet::new();
    for definition in &definitions {
        collect_segments(&definition.transition.operations, &mut segments)?;
    }
    let (segment_pool, segment_refs) = build_segment_pool(&segments)?;
    let mut indexes = Vec::<ParadigmCompatibilityIndexRecordV1>::new();
    let mut postings = Vec::<AnchorRecoveryPostingRecordV1>::new();
    let mut programs = Vec::<MorphProgramHeaderRecordV1>::new();
    let mut operations = Vec::<MorphOpRecordV1>::new();
    let mut program_definitions = BTreeMap::<AnchorRecoveryProgramKeyV1, usize>::new();
    for (definition_index, definition) in definitions.iter().enumerate() {
        program_definitions
            .entry(AnchorRecoveryProgramKeyV1::from_definition(definition))
            .or_insert(definition_index);
    }
    let mut program_ids = BTreeMap::<AnchorRecoveryProgramKeyV1, u32>::new();
    for (key, definition_index) in &program_definitions {
        let program_id = checked_u32(programs.len() + 1, "recovery program identity")?;
        append_program(
            &definitions[*definition_index],
            &segment_refs,
            &mut programs,
            &mut operations,
        )?;
        program_ids.insert(key.clone(), program_id);
    }
    let mut current_key = None;
    let mut posting_start = 0_u32;
    for definition in &definitions {
        let key = (u16::from(definition.pos_domain), definition.source_slot_id);
        if current_key.is_some_and(|current| current != key) {
            let (pos_domain, source_slot_id) = current_key.expect("recovery index key");
            let posting_end = checked_u32(postings.len(), "recovery posting end")?;
            sort_recovery_postings(&mut postings[posting_start as usize..posting_end as usize]);
            indexes.push(ParadigmCompatibilityIndexRecordV1 {
                pos_domain,
                flags: 0,
                source_slot_id,
                posting_start,
                posting_count: posting_end
                    .checked_sub(posting_start)
                    .ok_or_else(|| "anchor recovery posting range underflow".to_string())?,
            });
            posting_start = posting_end;
        }
        current_key = Some(key);
        let program_id = program_ids
            .get(&AnchorRecoveryProgramKeyV1::from_definition(definition))
            .copied()
            .ok_or_else(|| "anchor recovery definition has no shared program".to_string())?;
        postings.push(AnchorRecoveryPostingRecordV1 {
            paradigm_id: definition.paradigm_id,
            program_id,
            train_lemma_support: definition.train_lemma_support,
            stability: definition.stability,
            flags: definition.flags,
            provenance_hash_low: definition.provenance_hash_low,
        });
    }
    if let Some((pos_domain, source_slot_id)) = current_key {
        let posting_end = checked_u32(postings.len(), "recovery posting end")?;
        sort_recovery_postings(&mut postings[posting_start as usize..posting_end as usize]);
        indexes.push(ParadigmCompatibilityIndexRecordV1 {
            pos_domain,
            flags: 0,
            source_slot_id,
            posting_start,
            posting_count: posting_end
                .checked_sub(posting_start)
                .ok_or_else(|| "anchor recovery posting range underflow".to_string())?,
        });
    }

    let bytes = encode_package(
        &indexes,
        &postings,
        &programs,
        &operations,
        &segment_pool,
        AnchorRecoveryHeaderV1 {
            total_bytes: 0,
            index_offset: 0,
            posting_offset: 0,
            program_offset: 0,
            operation_offset: 0,
            segment_offset: 0,
            index_count: checked_u32(indexes.len(), "recovery index count")?,
            posting_count: checked_u32(postings.len(), "recovery posting count")?,
            program_count: checked_u32(programs.len(), "recovery program count")?,
            operation_count: checked_u32(operations.len(), "recovery operation count")?,
            segment_bytes: checked_u32(segment_pool.len(), "recovery segment bytes")?,
            maximum_observed_scalars,
            maximum_generated_scalars,
            maximum_program_operations: manifest.maximum_program_operations,
            split_seed,
            base_package_sha256,
            axis_schema_sha256: manifest.axis_schema_sha256,
            evidence_sha256: manifest.evidence_sha256,
            payload_sha256: [0; 32],
        },
    )?;
    let package_bytes = bytes.len() as u64;
    if package_bytes > MICRO_SIDECAR_BYTE_CEILING
        || base_package_bytes
            .checked_add(package_bytes)
            .is_none_or(|total| total > total_package_byte_budget)
    {
        return Err("anchor recovery sidecar exceeds its frozen package budget".to_string());
    }
    let package_sha256: [u8; 32] = Sha256::digest(&bytes).into();
    write_atomic(output_path, &bytes)?;
    let view = AnchorRecoveryPackageViewV1::load(output_path, base_package_sha256)?;
    if !view.mmap_backed() || view.backing_bytes() != bytes.len() {
        return Err("anchor recovery sidecar failed mmap publication parity".to_string());
    }
    Ok(CompiledAnchorRecoveryPackageV1 {
        path: output_path.to_path_buf(),
        package_sha256,
        package_bytes,
        index_count: indexes.len() as u32,
        posting_count: postings.len() as u32,
        shared_support_certified_posting_count: postings
            .iter()
            .filter(|posting| posting.flags == ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED)
            .count() as u32,
        program_count: programs.len() as u32,
        operation_count: operations.len() as u32,
    })
}

pub(super) fn recovery_sidecar_path(base_package_path: &Path) -> PathBuf {
    base_package_path.with_extension("p2r")
}

impl AnchorRecoveryPackageViewV1 {
    pub(super) fn load(path: &Path, expected_base_sha256: [u8; 32]) -> Result<Self, String> {
        let bytes = PackageBytes::load(path)?;
        let header = decode_header(bytes.as_slice())?;
        if header.base_package_sha256 != expected_base_sha256 {
            return Err("anchor recovery sidecar does not belong to the base package".to_string());
        }
        validate_contents(bytes.as_slice(), header)?;
        let indexes = fixed_records::<ParadigmCompatibilityIndexRecordV1>(
            bytes.as_slice(),
            header.index_offset,
            header.index_count,
        )?;
        let postings = fixed_records::<AnchorRecoveryPostingRecordV1>(
            bytes.as_slice(),
            header.posting_offset,
            header.posting_count,
        )?;
        let operations = fixed_records::<MorphOpRecordV1>(
            bytes.as_slice(),
            header.operation_offset,
            header.operation_count,
        )?;
        let program_records = fixed_records::<MorphProgramHeaderRecordV1>(
            bytes.as_slice(),
            header.program_offset,
            header.program_count,
        )?;
        let mut programs = Vec::with_capacity(program_records.len());
        for record in program_records.iter().copied() {
            let start = record.op_start as usize;
            let end = start
                .checked_add(record.op_count as usize)
                .filter(|end| *end <= operations.len())
                .ok_or_else(|| "anchor recovery prepared operation range is invalid".to_string())?;
            let suffix_drop = operations[start..end]
                .iter()
                .filter_map(|operation| {
                    (operation.decoded_opcode().ok()? == MorphOpcodeV1::DropSourceSuffix)
                        .then_some(operation.arg1)
                })
                .try_fold(0_u32, |total, count| total.checked_add(count))
                .ok_or_else(|| "anchor recovery prepared suffix drop overflows".to_string())?;
            programs.push(PreparedAnchorRecoveryProgramV1 {
                record,
                suffix_drop: u16::try_from(suffix_drop)
                    .map_err(|_| "anchor recovery prepared suffix drop exceeds u16".to_string())?,
            });
        }
        Ok(Self {
            bytes,
            header,
            indexes,
            postings,
            programs: programs.into_boxed_slice(),
            operations,
        })
    }

    pub(super) fn mmap_backed(&self) -> bool {
        self.bytes.is_mapped()
    }

    pub(super) fn backing_bytes(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn path_count(&self) -> usize {
        self.header.posting_count as usize
    }

    pub(super) fn resident_cache_bytes(&self) -> usize {
        self.indexes.len() * std::mem::size_of::<ParadigmCompatibilityIndexRecordV1>()
            + self.postings.len() * std::mem::size_of::<AnchorRecoveryPostingRecordV1>()
            + self.programs.len() * std::mem::size_of::<PreparedAnchorRecoveryProgramV1>()
            + self.operations.len() * std::mem::size_of::<MorphOpRecordV1>()
    }

    pub(super) fn recovery_paths(
        &self,
        pos_domain: u16,
        source_slot_id: u32,
    ) -> Result<Vec<AnchorRecoveryPathV1>, String> {
        let mut low = 0_usize;
        let mut high = self.header.index_count as usize;
        while low < high {
            let middle = low + (high - low) / 2;
            let index = self.index(middle)?;
            if (index.pos_domain, index.source_slot_id) < (pos_domain, source_slot_id) {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == self.header.index_count as usize {
            return Ok(Vec::new());
        }
        let index = self.index(low)?;
        if (index.pos_domain, index.source_slot_id) != (pos_domain, source_slot_id) {
            return Ok(Vec::new());
        }
        (0..index.posting_count as usize)
            .map(|offset| {
                let posting = self.posting(index.posting_start as usize + offset)?;
                let program = self.program(posting.program_id as usize - 1)?;
                if program.record.source_slot_id != source_slot_id {
                    return Err("anchor recovery posting crosses source slots".to_string());
                }
                Ok(AnchorRecoveryPathV1 {
                    posting,
                    program: program.record,
                    suffix_drop: program.suffix_drop,
                })
            })
            .collect()
    }

    pub(super) fn recover_anchor(
        &self,
        program: MorphProgramHeaderRecordV1,
        normalized_source: &str,
    ) -> Result<String, String> {
        let source = normalized_source.chars().collect::<Vec<_>>();
        let mut output = String::new();
        self.recover_anchor_into(program, &source, &mut output)?;
        Ok(output)
    }

    pub(super) fn recover_anchor_into(
        &self,
        program: MorphProgramHeaderRecordV1,
        source: &[char],
        output: &mut String,
    ) -> Result<(), String> {
        let operations = self.program_operations(program)?;
        let suffix_drop = operations
            .iter()
            .filter_map(|operation| {
                (operation.decoded_opcode().ok()? == MorphOpcodeV1::DropSourceSuffix)
                    .then_some(operation.arg1 as usize)
            })
            .try_fold(0_usize, |total, count| total.checked_add(count))
            .ok_or_else(|| "anchor recovery suffix drop overflow".to_string())?;
        self.recover_anchor_with_operations(program, suffix_drop, operations, source, output)
    }

    pub(super) fn recover_path_into(
        &self,
        path: AnchorRecoveryPathV1,
        source: &[char],
        output: &mut String,
    ) -> Result<(), String> {
        self.recover_anchor_with_operations(
            path.program,
            usize::from(path.suffix_drop),
            self.program_operations(path.program)?,
            source,
            output,
        )
    }

    fn recover_anchor_with_operations(
        &self,
        program: MorphProgramHeaderRecordV1,
        suffix_drop: usize,
        ops: &[MorphOpRecordV1],
        source: &[char],
        output: &mut String,
    ) -> Result<(), String> {
        if source.len() > usize::from(self.header.maximum_observed_scalars) {
            return Err("anchor recovery source exceeds the sidecar scalar bound".to_string());
        }
        let retained_end = source
            .len()
            .checked_sub(suffix_drop)
            .ok_or_else(|| "anchor recovery suffix drops exceed source".to_string())?;
        output.clear();
        let mut cursor = 0_usize;
        let mut terminated = false;
        for (operation_index, operation) in ops.iter().copied().enumerate() {
            if terminated {
                return Err("anchor recovery operation follows terminate".to_string());
            }
            match operation.decoded_opcode().map_err(str::to_string)? {
                MorphOpcodeV1::CopySourceRange => {
                    let anchor = decode_anchor(operation.anchor)?;
                    let start = resolve_source_offset(
                        source.len(),
                        anchor,
                        i16::try_from(operation.arg0)
                            .map_err(|_| "anchor recovery copy offset exceeds i16".to_string())?,
                    )
                    .ok_or_else(|| "anchor recovery copy offset is outside source".to_string())?;
                    let end = if operation.arg1 == u32::from(COPY_TO_RETAINED_EDGE) {
                        match ops.get(operation_index + 1).copied() {
                            Some(next)
                                if next.decoded_opcode().map_err(str::to_string)?
                                    == MorphOpcodeV1::ReplaceSourceRange =>
                            {
                                resolve_source_offset(
                                    source.len(),
                                    SourceAnchorV1::End,
                                    i16::try_from(next.arg0).map_err(|_| {
                                        "anchor recovery replacement offset exceeds i16".to_string()
                                    })?,
                                )
                                .ok_or_else(|| {
                                    "anchor recovery replacement offset is outside source"
                                        .to_string()
                                })?
                            }
                            _ => retained_end,
                        }
                    } else {
                        start
                            .checked_add(operation.arg1 as usize)
                            .ok_or_else(|| "anchor recovery copy range overflow".to_string())?
                    };
                    if start != cursor || end <= start || end > retained_end {
                        return Err("anchor recovery copy range is invalid".to_string());
                    }
                    output.extend(source[start..end].iter());
                    cursor = end;
                }
                MorphOpcodeV1::DropSourcePrefix => {
                    let count = operation.arg1 as usize;
                    if cursor != 0 || count == 0 || count > source.len() {
                        return Err("anchor recovery prefix drop is invalid".to_string());
                    }
                    cursor = count;
                }
                MorphOpcodeV1::DropSourceSuffix => {
                    let count = operation.arg1 as usize;
                    if count == 0 || cursor.checked_add(count) != Some(source.len()) {
                        return Err("anchor recovery suffix drop is invalid".to_string());
                    }
                    cursor = source.len();
                }
                MorphOpcodeV1::EmitSegment => output.push_str(self.segment(operation.arg1)?),
                MorphOpcodeV1::ReplaceSourceRange => {
                    let start = source
                        .len()
                        .checked_add_signed(operation.arg0 as isize)
                        .ok_or_else(|| {
                            "anchor recovery replacement offset is invalid".to_string()
                        })?;
                    let end = start
                        .checked_add(operation.arg1 as usize)
                        .ok_or_else(|| "anchor recovery replacement range overflow".to_string())?;
                    if start != cursor || end > source.len() {
                        return Err("anchor recovery replacement range is invalid".to_string());
                    }
                    if operation.arg2 != 0 {
                        output.push_str(self.segment(operation.arg2)?);
                    }
                    cursor = end;
                }
                MorphOpcodeV1::EmitExactAllomorph => {
                    return Err(
                        "lemma-local exact allomorph leaked into anchor recovery".to_string()
                    )
                }
                MorphOpcodeV1::Terminate => {
                    if operation.arg1 != program.target_slot_id
                        || operation.arg2 != 1
                        || cursor != source.len()
                    {
                        return Err("anchor recovery terminate identity is invalid".to_string());
                    }
                    terminated = true;
                }
            }
        }
        if !terminated
            || output.is_empty()
            || output.chars().count() > usize::from(self.header.maximum_generated_scalars)
        {
            return Err("anchor recovery output violates its exact bounds".to_string());
        }
        Ok(())
    }

    fn index(&self, index: usize) -> Result<ParadigmCompatibilityIndexRecordV1, String> {
        self.indexes
            .get(index)
            .copied()
            .ok_or_else(|| "anchor recovery prepared index is outside its section".to_string())
    }

    fn posting(&self, index: usize) -> Result<AnchorRecoveryPostingRecordV1, String> {
        self.postings
            .get(index)
            .copied()
            .ok_or_else(|| "anchor recovery prepared posting is outside its section".to_string())
    }

    fn program(&self, index: usize) -> Result<PreparedAnchorRecoveryProgramV1, String> {
        self.programs
            .get(index)
            .copied()
            .ok_or_else(|| "anchor recovery prepared program is outside its section".to_string())
    }

    fn program_operations(
        &self,
        program: MorphProgramHeaderRecordV1,
    ) -> Result<&[MorphOpRecordV1], String> {
        let start = program.op_start as usize;
        let end = start
            .checked_add(program.op_count as usize)
            .ok_or_else(|| "anchor recovery prepared operation range overflows".to_string())?;
        self.operations.get(start..end).ok_or_else(|| {
            "anchor recovery prepared operation range is outside its section".to_string()
        })
    }

    fn segment(&self, reference: u32) -> Result<&str, String> {
        if reference == 0 {
            return Err("anchor recovery segment reference is zero".to_string());
        }
        let pool_start = self.header.segment_offset as usize;
        let pool_end = pool_start + self.header.segment_bytes as usize;
        let start = pool_start
            .checked_add(reference as usize)
            .ok_or_else(|| "anchor recovery segment reference overflow".to_string())?;
        if start < pool_start + 8 || start + 8 > pool_end {
            return Err("anchor recovery segment reference is outside the pool".to_string());
        }
        let byte_count = u32::from_le_bytes(
            self.bytes.as_slice()[start..start + 4]
                .try_into()
                .expect("segment bytes"),
        ) as usize;
        if self.bytes.as_slice()[start + 6..start + 8] != [0; 2] {
            return Err("anchor recovery segment reserved field is not zero".to_string());
        }
        let end = start
            .checked_add(8)
            .and_then(|value| value.checked_add(byte_count))
            .ok_or_else(|| "anchor recovery segment end overflow".to_string())?;
        if end > pool_end {
            return Err("anchor recovery segment is truncated".to_string());
        }
        std::str::from_utf8(&self.bytes.as_slice()[start + 8..end])
            .map_err(|_| "anchor recovery segment is not UTF-8".to_string())
    }
}

fn collect_segments(
    operations: &[EditOperationV1],
    segments: &mut BTreeSet<String>,
) -> Result<(), String> {
    for operation in operations {
        match operation {
            EditOperationV1::EmitSegment { segment }
            | EditOperationV1::ReplaceSourceRange { segment, .. }
                if !segment.is_empty() =>
            {
                segments.insert(segment.clone());
            }
            EditOperationV1::EmitExactAllomorph { .. } => {
                return Err("anchor recovery contains a lemma-local exact allomorph".to_string())
            }
            _ => {}
        }
    }
    Ok(())
}

fn posting_order_key(posting: &AnchorRecoveryPostingRecordV1) -> (u32, u32, u32, u16, u16, u32) {
    (
        posting.program_id,
        posting.paradigm_id,
        posting.train_lemma_support,
        posting.stability,
        posting.flags,
        posting.provenance_hash_low,
    )
}

fn sort_recovery_postings(postings: &mut [AnchorRecoveryPostingRecordV1]) {
    postings.sort_by_key(posting_order_key);
}

fn build_segment_pool(
    segments: &BTreeSet<String>,
) -> Result<(Vec<u8>, BTreeMap<String, u32>), String> {
    let mut bytes = b"SPV1".to_vec();
    bytes.extend_from_slice(&checked_u32(segments.len(), "recovery segment count")?.to_le_bytes());
    let mut refs = BTreeMap::new();
    for segment in segments {
        let reference = checked_u32(bytes.len(), "recovery segment reference")?;
        bytes.extend_from_slice(
            &checked_u32(segment.len(), "recovery segment bytes")?.to_le_bytes(),
        );
        bytes.extend_from_slice(
            &u16::try_from(segment.chars().count())
                .map_err(|_| "anchor recovery segment scalar count exceeds u16".to_string())?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(segment.as_bytes());
        bytes.resize(align8(bytes.len()), 0);
        refs.insert(segment.clone(), reference);
    }
    Ok((bytes, refs))
}

fn append_program(
    definition: &AnchorRecoveryDefinitionV1,
    segment_refs: &BTreeMap<String, u32>,
    programs: &mut Vec<MorphProgramHeaderRecordV1>,
    operations: &mut Vec<MorphOpRecordV1>,
) -> Result<(), String> {
    let op_start = checked_u32(operations.len(), "recovery operation start")?;
    for operation in
        definition
            .transition
            .operations
            .iter()
            .cloned()
            .chain([EditOperationV1::Terminate {
                slot_id: definition.canonical_anchor_slot_id,
                variant_id: 1,
            }])
    {
        operations.push(match operation {
            EditOperationV1::CopySourceRange {
                start_anchor,
                start_delta,
                scalar_count,
            } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::CopySourceRange as u8,
                anchor: start_anchor as u8,
                flags: 0,
                arg0: i32::from(start_delta),
                arg1: u32::from(scalar_count),
                arg2: 0,
            },
            EditOperationV1::DropSourcePrefix { scalar_count } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::DropSourcePrefix as u8,
                arg1: u32::from(scalar_count),
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::DropSourceSuffix { scalar_count } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::DropSourceSuffix as u8,
                arg1: u32::from(scalar_count),
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::EmitSegment { segment } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::EmitSegment as u8,
                arg1: required_segment_ref(segment_refs, &segment)?,
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::ReplaceSourceRange {
                end_relative_offset,
                delete_count,
                segment,
            } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::ReplaceSourceRange as u8,
                anchor: SourceAnchorV1::End as u8,
                arg0: i32::from(end_relative_offset),
                arg1: u32::from(delete_count),
                arg2: if segment.is_empty() {
                    0
                } else {
                    required_segment_ref(segment_refs, &segment)?
                },
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::EmitExactAllomorph { .. } => {
                return Err("anchor recovery exact allomorph reached package compile".to_string())
            }
            EditOperationV1::Terminate {
                slot_id,
                variant_id,
            } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::Terminate as u8,
                arg1: slot_id,
                arg2: u32::from(variant_id),
                ..MorphOpRecordV1::default()
            },
        });
    }
    programs.push(MorphProgramHeaderRecordV1 {
        source_slot_id: definition.source_slot_id,
        target_slot_id: definition.canonical_anchor_slot_id,
        op_start,
        op_count: u16::try_from(operations.len() - op_start as usize)
            .map_err(|_| "anchor recovery program exceeds u16 operations".to_string())?,
        flags: 0,
    });
    Ok(())
}

fn encode_package(
    indexes: &[ParadigmCompatibilityIndexRecordV1],
    postings: &[AnchorRecoveryPostingRecordV1],
    programs: &[MorphProgramHeaderRecordV1],
    operations: &[MorphOpRecordV1],
    segment_pool: &[u8],
    mut header: AnchorRecoveryHeaderV1,
) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0_u8; HEADER_BYTES];
    header.index_offset = append_aligned(&mut bytes, &encode_records(indexes))?;
    header.posting_offset = append_aligned(&mut bytes, &encode_records(postings))?;
    header.program_offset = append_aligned(&mut bytes, &encode_records(programs))?;
    header.operation_offset = append_aligned(&mut bytes, &encode_records(operations))?;
    header.segment_offset = append_aligned(&mut bytes, segment_pool)?;
    header.total_bytes = bytes.len() as u64;
    header.payload_sha256 = Sha256::digest(&bytes[HEADER_BYTES..]).into();
    encode_header(&mut bytes[..HEADER_BYTES], header)?;
    validate_contents(&bytes, decode_header(&bytes)?)?;
    Ok(bytes)
}

fn encode_header(output: &mut [u8], header: AnchorRecoveryHeaderV1) -> Result<(), String> {
    if output.len() != HEADER_BYTES {
        return Err("anchor recovery header width is invalid".to_string());
    }
    output[0..8].copy_from_slice(&MAGIC);
    output[8..10].copy_from_slice(&PRODUCTIVE_V1_SCHEMA_VERSION.to_le_bytes());
    output[10..12].copy_from_slice(&(HEADER_BYTES as u16).to_le_bytes());
    output[12..16].copy_from_slice(&0_u32.to_le_bytes());
    output[16..24].copy_from_slice(&header.total_bytes.to_le_bytes());
    output[24..32].copy_from_slice(&header.index_offset.to_le_bytes());
    output[32..40].copy_from_slice(&header.posting_offset.to_le_bytes());
    output[40..48].copy_from_slice(&header.program_offset.to_le_bytes());
    output[48..56].copy_from_slice(&header.operation_offset.to_le_bytes());
    output[56..64].copy_from_slice(&header.segment_offset.to_le_bytes());
    output[64..68].copy_from_slice(&header.index_count.to_le_bytes());
    output[68..72].copy_from_slice(&header.posting_count.to_le_bytes());
    output[72..76].copy_from_slice(&header.program_count.to_le_bytes());
    output[76..80].copy_from_slice(&header.operation_count.to_le_bytes());
    output[80..84].copy_from_slice(&header.segment_bytes.to_le_bytes());
    output[84..86].copy_from_slice(&header.maximum_observed_scalars.to_le_bytes());
    output[86..88].copy_from_slice(&header.maximum_generated_scalars.to_le_bytes());
    output[88..90].copy_from_slice(&header.maximum_program_operations.to_le_bytes());
    output[90..92].copy_from_slice(&0_u16.to_le_bytes());
    output[92..100].copy_from_slice(&header.split_seed.to_le_bytes());
    output[100..132].copy_from_slice(&header.base_package_sha256);
    output[132..164].copy_from_slice(&header.axis_schema_sha256);
    output[164..196].copy_from_slice(&header.evidence_sha256);
    output[196..228].copy_from_slice(&header.payload_sha256);
    Ok(())
}

fn decode_header(bytes: &[u8]) -> Result<AnchorRecoveryHeaderV1, String> {
    if bytes.len() < HEADER_BYTES
        || bytes[0..8] != MAGIC
        || u16::from_le_bytes(bytes[8..10].try_into().expect("version"))
            != PRODUCTIVE_V1_SCHEMA_VERSION
        || u16::from_le_bytes(bytes[10..12].try_into().expect("header")) as usize != HEADER_BYTES
        || bytes[12..16] != [0; 4]
        || bytes[90..92] != [0; 2]
        || bytes[228..HEADER_BYTES].iter().any(|byte| *byte != 0)
    {
        return Err("anchor recovery sidecar header is invalid".to_string());
    }
    Ok(AnchorRecoveryHeaderV1 {
        total_bytes: u64::from_le_bytes(bytes[16..24].try_into().expect("total")),
        index_offset: u64::from_le_bytes(bytes[24..32].try_into().expect("index offset")),
        posting_offset: u64::from_le_bytes(bytes[32..40].try_into().expect("posting offset")),
        program_offset: u64::from_le_bytes(bytes[40..48].try_into().expect("program offset")),
        operation_offset: u64::from_le_bytes(bytes[48..56].try_into().expect("operation offset")),
        segment_offset: u64::from_le_bytes(bytes[56..64].try_into().expect("segment offset")),
        index_count: u32::from_le_bytes(bytes[64..68].try_into().expect("index count")),
        posting_count: u32::from_le_bytes(bytes[68..72].try_into().expect("posting count")),
        program_count: u32::from_le_bytes(bytes[72..76].try_into().expect("program count")),
        operation_count: u32::from_le_bytes(bytes[76..80].try_into().expect("operation count")),
        segment_bytes: u32::from_le_bytes(bytes[80..84].try_into().expect("segment bytes")),
        maximum_observed_scalars: u16::from_le_bytes(
            bytes[84..86].try_into().expect("observed scalars"),
        ),
        maximum_generated_scalars: u16::from_le_bytes(
            bytes[86..88].try_into().expect("generated scalars"),
        ),
        maximum_program_operations: u16::from_le_bytes(
            bytes[88..90].try_into().expect("program operations"),
        ),
        split_seed: u64::from_le_bytes(bytes[92..100].try_into().expect("split seed")),
        base_package_sha256: bytes[100..132].try_into().expect("base hash"),
        axis_schema_sha256: bytes[132..164].try_into().expect("axis hash"),
        evidence_sha256: bytes[164..196].try_into().expect("evidence hash"),
        payload_sha256: bytes[196..228].try_into().expect("payload hash"),
    })
}

fn validate_contents(bytes: &[u8], header: AnchorRecoveryHeaderV1) -> Result<(), String> {
    if header.total_bytes as usize != bytes.len()
        || header.maximum_observed_scalars == 0
        || header.maximum_generated_scalars == 0
        || header.maximum_program_operations == 0
        || header.base_package_sha256 == [0; 32]
        || header.axis_schema_sha256 == [0; 32]
        || header.evidence_sha256 == [0; 32]
    {
        return Err("anchor recovery sidecar identity or bounds are invalid".to_string());
    }
    let expected_posting_offset =
        section_end::<ParadigmCompatibilityIndexRecordV1>(header.index_offset, header.index_count)?;
    let expected_program_offset =
        section_end::<AnchorRecoveryPostingRecordV1>(header.posting_offset, header.posting_count)?;
    let expected_operation_offset =
        section_end::<MorphProgramHeaderRecordV1>(header.program_offset, header.program_count)?;
    let expected_segment_offset =
        section_end::<MorphOpRecordV1>(header.operation_offset, header.operation_count)?;
    if header.index_offset != HEADER_BYTES as u64
        || header.posting_offset != expected_posting_offset
        || header.program_offset != expected_program_offset
        || header.operation_offset != expected_operation_offset
        || header.segment_offset != expected_segment_offset
        || header.segment_offset + u64::from(header.segment_bytes) != header.total_bytes
        || <[u8; 32]>::from(Sha256::digest(&bytes[HEADER_BYTES..])) != header.payload_sha256
    {
        return Err("anchor recovery sidecar section layout or digest is invalid".to_string());
    }
    if header.segment_bytes < 8
        || bytes[header.segment_offset as usize..header.segment_offset as usize + 4] != *b"SPV1"
    {
        return Err("anchor recovery segment pool header is invalid".to_string());
    }
    let mut previous_key = None;
    let mut expected_posting_start = 0_u32;
    let mut program_pos_owners = vec![None; header.program_count as usize];
    for index_id in 0..header.index_count as usize {
        let index: ParadigmCompatibilityIndexRecordV1 =
            fixed_record(bytes, header.index_offset, header.index_count, index_id)?;
        let key = (index.pos_domain, index.source_slot_id);
        if previous_key.is_some_and(|previous| previous >= key)
            || index.posting_start != expected_posting_start
            || index.posting_start as u64 + u64::from(index.posting_count)
                > u64::from(header.posting_count)
        {
            return Err("anchor recovery index order or range is invalid".to_string());
        }
        previous_key = Some(key);
        expected_posting_start = index
            .posting_start
            .checked_add(index.posting_count)
            .ok_or_else(|| "anchor recovery posting range overflows".to_string())?;
        let mut previous_posting_key = None;
        for offset in 0..index.posting_count as usize {
            let posting: AnchorRecoveryPostingRecordV1 = fixed_record(
                bytes,
                header.posting_offset,
                header.posting_count,
                index.posting_start as usize + offset,
            )?;
            let program: MorphProgramHeaderRecordV1 = fixed_record(
                bytes,
                header.program_offset,
                header.program_count,
                posting.program_id as usize - 1,
            )?;
            let posting_key = posting_order_key(&posting);
            if previous_posting_key.is_some_and(|previous| previous >= posting_key)
                || program.source_slot_id != index.source_slot_id
                || program.op_count > header.maximum_program_operations
                || program.op_start as u64 + u64::from(program.op_count)
                    > u64::from(header.operation_count)
            {
                return Err("anchor recovery program ownership or range is invalid".to_string());
            }
            previous_posting_key = Some(posting_key);
            let owner = program_pos_owners
                .get_mut(posting.program_id as usize - 1)
                .ok_or_else(|| {
                    "anchor recovery program owner is outside its section".to_string()
                })?;
            if owner.is_some_and(|pos_domain| pos_domain != index.pos_domain) {
                return Err("anchor recovery shared program crosses POS domains".to_string());
            }
            *owner = Some(index.pos_domain);
        }
    }
    if expected_posting_start != header.posting_count
        || program_pos_owners.iter().any(Option::is_none)
    {
        return Err("anchor recovery posting or program coverage is incomplete".to_string());
    }
    Ok(())
}

fn fixed_record<T: FixedRecordV1>(
    bytes: &[u8],
    section_offset: u64,
    section_count: u32,
    index: usize,
) -> Result<T, String> {
    if index >= section_count as usize {
        return Err("anchor recovery fixed record index is outside its section".to_string());
    }
    let start = section_offset as usize + index * T::BYTES;
    let end = start + T::BYTES;
    T::decode_record(&bytes[start..end]).map_err(str::to_string)
}

fn fixed_records<T: FixedRecordV1>(
    bytes: &[u8],
    section_offset: u64,
    section_count: u32,
) -> Result<Box<[T]>, String> {
    (0..section_count as usize)
        .map(|index| fixed_record(bytes, section_offset, section_count, index))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn section_end<T: FixedRecordV1>(offset: u64, count: u32) -> Result<u64, String> {
    let raw = offset
        .checked_add(
            u64::from(count)
                .checked_mul(T::BYTES as u64)
                .ok_or_else(|| "anchor recovery section size overflow".to_string())?,
        )
        .ok_or_else(|| "anchor recovery section end overflow".to_string())?;
    Ok(align8(raw as usize) as u64)
}

fn append_aligned(output: &mut Vec<u8>, section: &[u8]) -> Result<u64, String> {
    output.resize(align8(output.len()), 0);
    let offset = output.len() as u64;
    output.extend_from_slice(section);
    output.resize(align8(output.len()), 0);
    Ok(offset)
}

const fn align8(value: usize) -> usize {
    (value + 7) & !7
}

fn decode_anchor(value: u8) -> Result<SourceAnchorV1, String> {
    match value {
        1 => Ok(SourceAnchorV1::Start),
        2 => Ok(SourceAnchorV1::End),
        _ => Err("anchor recovery source anchor is invalid".to_string()),
    }
}

fn required_segment_ref(refs: &BTreeMap<String, u32>, segment: &str) -> Result<u32, String> {
    refs.get(segment)
        .copied()
        .ok_or_else(|| "anchor recovery program segment is absent from its pool".to_string())
}

fn checked_u32(value: usize, owner: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("anchor recovery {owner} exceeds u32"))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension("p2r.tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn posting(support: u32, flags: u16) -> AnchorRecoveryPostingRecordV1 {
        AnchorRecoveryPostingRecordV1 {
            paradigm_id: 1,
            program_id: 1,
            train_lemma_support: support,
            stability: 1,
            flags,
            provenance_hash_low: 1,
        }
    }

    fn roundtrip(record: AnchorRecoveryPostingRecordV1) -> Result<(), &'static str> {
        let mut bytes = Vec::new();
        record.encode_record(&mut bytes);
        AnchorRecoveryPostingRecordV1::decode_record(&bytes).map(|decoded| {
            assert_eq!(decoded, record);
        })
    }

    #[test]
    fn support_one_requires_exact_shared_certificate_flag() {
        assert!(roundtrip(posting(1, ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED)).is_ok());
        assert!(roundtrip(posting(1, 0)).is_err());
        assert!(roundtrip(posting(2, ANCHOR_RECOVERY_SHARED_SUPPORT_CERTIFIED)).is_err());
        assert!(roundtrip(posting(1, u16::MAX)).is_err());
    }

    #[test]
    fn fine_owner_postings_group_by_shared_program_deterministically() {
        let mut postings = vec![
            AnchorRecoveryPostingRecordV1 {
                paradigm_id: 3,
                program_id: 2,
                ..posting(2, 0)
            },
            AnchorRecoveryPostingRecordV1 {
                paradigm_id: 2,
                program_id: 1,
                ..posting(2, 0)
            },
            AnchorRecoveryPostingRecordV1 {
                paradigm_id: 1,
                program_id: 2,
                ..posting(2, 0)
            },
        ];

        sort_recovery_postings(&mut postings);

        assert_eq!(
            postings
                .iter()
                .map(|posting| (posting.program_id, posting.paradigm_id))
                .collect::<Vec<_>>(),
            vec![(1, 2), (2, 1), (2, 3)]
        );
    }
}
