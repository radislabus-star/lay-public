use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::super::package_bytes::PackageBytes;
use super::format_validation::validate_package_contents;
use super::records::{encode_records, FixedRecordV1};

const MAGIC: [u8; 8] = *b"LAYP2V1\0";
const FORMAT_VERSION: u16 = 1;
const HEADER_BYTES: usize = 256;
const DIRECTORY_ENTRY_BYTES: usize = 32;
const BYTE_ORDER_MARKER: u32 = 0x0102_0304;
const HEADER_FLAG_V39_SPEED_PARITY: u32 = 1;
const HEADER_FLAG_PRODUCTIVE_V1_MODEL: u32 = 2;
const HEADER_KNOWN_FLAGS: u32 = HEADER_FLAG_V39_SPEED_PARITY | HEADER_FLAG_PRODUCTIVE_V1_MODEL;
pub(super) const REQUIRED_SECTION_COUNT: usize = 23;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProductiveAlgorithmModeV1 {
    V39SpeedParity,
    ProductiveV1Model,
}

impl ProductiveAlgorithmModeV1 {
    const fn flags(self) -> u32 {
        match self {
            Self::V39SpeedParity => HEADER_FLAG_V39_SPEED_PARITY,
            Self::ProductiveV1Model => HEADER_FLAG_PRODUCTIVE_V1_MODEL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub(super) enum ProductiveSectionKindV1 {
    AxisDictionaries = 1,
    SlotKeys = 2,
    ParadigmCenters = 3,
    LemmaBindings = 4,
    ParadigmCompatibilityIndex = 5,
    ParadigmPostings = 6,
    MorphProgramHeaders = 7,
    MorphOperations = 8,
    SegmentPool = 9,
    TrieNodes = 10,
    TrieArcs = 11,
    Terminals = 12,
    SlotPhaseProfiles = 13,
    PositivePhaseCenters = 14,
    AntiPhaseCenters = 15,
    HardNegativePhaseCenters = 16,
    AmbiguityPhaseCenters = 17,
    DirectionalResiduals = 18,
    ModelCoefficients = 19,
    CalibrationCells = 20,
    Provenance = 21,
    DeltaManifest = 22,
    EvidencePriors = 23,
}

impl ProductiveSectionKindV1 {
    const ALL: [Self; REQUIRED_SECTION_COUNT] = [
        Self::AxisDictionaries,
        Self::SlotKeys,
        Self::ParadigmCenters,
        Self::LemmaBindings,
        Self::ParadigmCompatibilityIndex,
        Self::ParadigmPostings,
        Self::MorphProgramHeaders,
        Self::MorphOperations,
        Self::SegmentPool,
        Self::TrieNodes,
        Self::TrieArcs,
        Self::Terminals,
        Self::SlotPhaseProfiles,
        Self::PositivePhaseCenters,
        Self::AntiPhaseCenters,
        Self::HardNegativePhaseCenters,
        Self::AmbiguityPhaseCenters,
        Self::DirectionalResiduals,
        Self::ModelCoefficients,
        Self::CalibrationCells,
        Self::Provenance,
        Self::DeltaManifest,
        Self::EvidencePriors,
    ];

    fn decode(value: u16) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|kind| *kind as u16 == value)
            .ok_or_else(|| format!("unknown productive V1 section kind {value}"))
    }

    const fn fixed_record_size(self) -> Option<u32> {
        match self {
            Self::AxisDictionaries | Self::SegmentPool => None,
            Self::SlotKeys => Some(16),
            Self::ParadigmCenters => Some(48),
            Self::LemmaBindings => Some(40),
            Self::ParadigmCompatibilityIndex => Some(16),
            Self::ParadigmPostings => Some(4),
            Self::MorphProgramHeaders => Some(16),
            Self::MorphOperations => Some(16),
            Self::TrieNodes => Some(16),
            Self::TrieArcs => Some(24),
            Self::Terminals => Some(32),
            Self::SlotPhaseProfiles => Some(44),
            Self::PositivePhaseCenters
            | Self::AntiPhaseCenters
            | Self::HardNegativePhaseCenters
            | Self::AmbiguityPhaseCenters => Some(76),
            Self::DirectionalResiduals => Some(24),
            Self::ModelCoefficients => Some(16),
            Self::CalibrationCells => Some(32),
            Self::Provenance => Some(32),
            Self::DeltaManifest => Some(192),
            Self::EvidencePriors => Some(24),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveSectionBuildV1 {
    pub(super) kind: ProductiveSectionKindV1,
    pub(super) flags: u16,
    pub(super) record_size: u32,
    pub(super) count: u32,
    pub(super) bytes: Vec<u8>,
}

impl ProductiveSectionBuildV1 {
    pub(super) fn empty(kind: ProductiveSectionKindV1) -> Self {
        Self {
            kind,
            flags: 0,
            record_size: kind.fixed_record_size().unwrap_or(0),
            count: 0,
            bytes: Vec::new(),
        }
    }

    pub(super) fn fixed_records<T: FixedRecordV1>(
        kind: ProductiveSectionKindV1,
        records: &[T],
    ) -> Result<Self, String> {
        let expected = kind.fixed_record_size().ok_or_else(|| {
            "productive V1 variable section cannot contain fixed records".to_string()
        })?;
        if T::BYTES != expected as usize {
            return Err(format!(
                "productive V1 {:?} codec width does not match its directory contract",
                kind
            ));
        }
        Ok(Self {
            kind,
            flags: 0,
            record_size: expected,
            count: u32::try_from(records.len())
                .map_err(|_| "productive V1 fixed section exceeds u32 records".to_string())?,
            bytes: encode_records(records),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductivePackageBuildV1 {
    pub(super) mode: ProductiveAlgorithmModeV1,
    pub(super) l11_package_sha256: [u8; 32],
    pub(super) canonical_l2_package_sha256: [u8; 32],
    pub(super) training_manifest_sha256: [u8; 32],
    pub(super) maximum_observed_scalars: u16,
    pub(super) maximum_generated_scalars: u16,
    pub(super) maximum_program_operations: u16,
    pub(super) split_seed: u64,
    pub(super) normalization_version: u32,
    pub(super) compiler_version: u32,
    pub(super) productive_package_byte_budget: u64,
    pub(super) steady_rss_kib_budget: u32,
    pub(super) peak_rss_kib_budget: u32,
    pub(super) cold_publish_budget_us: u32,
    pub(super) hot_p99_budget_us: u32,
    pub(super) sections: Vec<ProductiveSectionBuildV1>,
}

impl ProductivePackageBuildV1 {
    pub(super) fn with_empty_required_sections(mode: ProductiveAlgorithmModeV1) -> Self {
        Self {
            mode,
            l11_package_sha256: [0; 32],
            canonical_l2_package_sha256: [0; 32],
            training_manifest_sha256: [0; 32],
            maximum_observed_scalars: 0,
            maximum_generated_scalars: 0,
            maximum_program_operations: 0,
            split_seed: 0,
            normalization_version: 0,
            compiler_version: 0,
            productive_package_byte_budget: u64::MAX,
            steady_rss_kib_budget: u32::MAX,
            peak_rss_kib_budget: u32::MAX,
            cold_publish_budget_us: u32::MAX,
            hot_p99_budget_us: u32::MAX,
            sections: ProductiveSectionKindV1::ALL
                .into_iter()
                .map(ProductiveSectionBuildV1::empty)
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SectionDirectoryEntryV1 {
    pub(super) kind: ProductiveSectionKindV1,
    pub(super) flags: u16,
    pub(super) record_size: u32,
    pub(super) offset: u64,
    pub(super) bytes: u64,
    pub(super) count: u32,
    pub(super) crc32: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductivePackageHeaderV1 {
    pub(super) mode: ProductiveAlgorithmModeV1,
    pub(super) l11_package_sha256: [u8; 32],
    pub(super) canonical_l2_package_sha256: [u8; 32],
    pub(super) training_manifest_sha256: [u8; 32],
    pub(super) payload_sections_sha256: [u8; 32],
    pub(super) maximum_observed_scalars: u16,
    pub(super) maximum_generated_scalars: u16,
    pub(super) maximum_program_operations: u16,
    pub(super) split_seed: u64,
    pub(super) normalization_version: u32,
    pub(super) compiler_version: u32,
    pub(super) productive_package_byte_budget: u64,
    pub(super) steady_rss_kib_budget: u32,
    pub(super) peak_rss_kib_budget: u32,
    pub(super) cold_publish_budget_us: u32,
    pub(super) hot_p99_budget_us: u32,
}

#[derive(Clone, Debug)]
pub(super) struct ProductivePackageViewV1 {
    backing: PackageBytes,
    package_sha256: [u8; 32],
    pub(super) header: ProductivePackageHeaderV1,
    directory: BTreeMap<ProductiveSectionKindV1, SectionDirectoryEntryV1>,
}

impl ProductivePackageViewV1 {
    pub(super) fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        Self::from_backing(PackageBytes::from_vec(bytes))
    }

    pub(super) fn load(path: &Path) -> Result<Self, String> {
        Self::from_backing(PackageBytes::load(path)?)
    }

    fn from_backing(backing: PackageBytes) -> Result<Self, String> {
        let (header, directory) = decode_and_validate(backing.as_slice())?;
        let package_sha256 = Sha256::digest(backing.as_slice()).into();
        Ok(Self {
            backing,
            package_sha256,
            header,
            directory,
        })
    }

    pub(super) fn section(&self, kind: ProductiveSectionKindV1) -> &[u8] {
        let entry = self
            .directory
            .get(&kind)
            .expect("validated required section");
        &self.backing.as_slice()[entry.offset as usize..(entry.offset + entry.bytes) as usize]
    }

    pub(super) fn record_count(&self, kind: ProductiveSectionKindV1) -> usize {
        self.directory
            .get(&kind)
            .expect("validated required section")
            .count as usize
    }

    pub(super) fn record<T: FixedRecordV1>(
        &self,
        kind: ProductiveSectionKindV1,
        index: usize,
    ) -> Result<T, String> {
        let entry = self
            .directory
            .get(&kind)
            .copied()
            .ok_or_else(|| "productive mmap record section is missing".to_string())?;
        if entry.record_size as usize != T::BYTES || index >= entry.count as usize {
            return Err("productive mmap record type or index is invalid".to_string());
        }
        let start = index
            .checked_mul(T::BYTES)
            .ok_or_else(|| "productive mmap record offset overflow".to_string())?;
        T::decode_record(
            self.section(kind)
                .get(start..start + T::BYTES)
                .ok_or_else(|| "productive mmap record lies outside its section".to_string())?,
        )
        .map_err(str::to_string)
    }

    pub(super) fn segment(&self, reference: u32) -> Result<&str, String> {
        let payload =
            self.pool_payload(ProductiveSectionKindV1::SegmentPool, reference, b"SPV1")?;
        std::str::from_utf8(payload).map_err(|_| "productive mmap segment is not UTF-8".to_string())
    }

    pub(super) fn observed_slot_ids(&self, reference: u32) -> Result<Vec<u32>, String> {
        let payload = self.pool_payload(
            ProductiveSectionKindV1::AxisDictionaries,
            reference,
            b"ADV1",
        )?;
        if payload.len() < 8 || payload[0] != 2 || payload[1..4] != [0; 3] {
            return Err("productive mmap observed-slot reference has the wrong kind".to_string());
        }
        let count = get_u32(payload, 4)? as usize;
        if payload.len() != 8_usize.saturating_add(count.saturating_mul(4)) {
            return Err("productive mmap observed-slot payload length is invalid".to_string());
        }
        (0..count)
            .map(|index| get_u32(payload, 8 + index * 4))
            .collect()
    }

    fn pool_payload(
        &self,
        kind: ProductiveSectionKindV1,
        reference: u32,
        magic: &[u8; 4],
    ) -> Result<&[u8], String> {
        let bytes = self.section(kind);
        if bytes.get(0..4) != Some(magic) || reference < 8 {
            return Err("productive mmap pool magic or reference is invalid".to_string());
        }
        let start = reference as usize;
        let byte_length = get_u32(bytes, start)? as usize;
        let flags = get_u16(bytes, start + 6)?;
        let payload_start = start
            .checked_add(8)
            .ok_or_else(|| "productive mmap pool payload offset overflow".to_string())?;
        let payload_end = payload_start
            .checked_add(byte_length)
            .ok_or_else(|| "productive mmap pool payload length overflow".to_string())?;
        if flags != 0 {
            return Err("productive mmap pool entry has unknown flags".to_string());
        }
        bytes
            .get(payload_start..payload_end)
            .ok_or_else(|| "productive mmap pool payload lies outside its section".to_string())
    }

    pub(super) fn backing_bytes(&self) -> usize {
        self.backing.len()
    }

    pub(super) fn package_sha256(&self) -> [u8; 32] {
        self.package_sha256
    }

    pub(super) fn mmap_backed(&self) -> bool {
        self.backing.is_mapped()
    }
}

pub(super) fn encode_package(build: &ProductivePackageBuildV1) -> Result<Vec<u8>, String> {
    validate_build(build)?;
    let mut sections = build.sections.clone();
    sections.sort_by_key(|section| section.kind);
    let directory_offset = HEADER_BYTES;
    let directory_bytes = REQUIRED_SECTION_COUNT
        .checked_mul(DIRECTORY_ENTRY_BYTES)
        .ok_or_else(|| "productive V1 directory overflow".to_string())?;
    let mut next_offset = align8(
        directory_offset
            .checked_add(directory_bytes)
            .ok_or_else(|| "productive V1 directory end overflow".to_string())?,
    )?;
    let mut entries = Vec::with_capacity(REQUIRED_SECTION_COUNT);
    for section in &sections {
        let offset = next_offset;
        next_offset = align8(
            offset
                .checked_add(section.bytes.len())
                .ok_or_else(|| "productive V1 section end overflow".to_string())?,
        )?;
        entries.push(SectionDirectoryEntryV1 {
            kind: section.kind,
            flags: section.flags,
            record_size: section.record_size,
            offset: offset as u64,
            bytes: section.bytes.len() as u64,
            count: section.count,
            crc32: crc32(&section.bytes),
        });
    }
    let mut payload_hasher = Sha256::new();
    for section in &sections {
        payload_hasher.update(&section.bytes);
    }
    let payload_sha256: [u8; 32] = payload_hasher.finalize().into();
    if next_offset as u64 > build.productive_package_byte_budget {
        return Err("productive V1 package exceeds configured byte budget".to_string());
    }
    let mut bytes = vec![0_u8; next_offset];
    for (section, entry) in sections.iter().zip(&entries) {
        let start = entry.offset as usize;
        bytes[start..start + section.bytes.len()].copy_from_slice(&section.bytes);
    }
    for (index, entry) in entries.iter().enumerate() {
        encode_directory_entry(
            &mut bytes[directory_offset + index * DIRECTORY_ENTRY_BYTES
                ..directory_offset + (index + 1) * DIRECTORY_ENTRY_BYTES],
            *entry,
        );
    }
    encode_header(
        &mut bytes[..HEADER_BYTES],
        build,
        payload_sha256,
        directory_offset,
    )?;
    Ok(bytes)
}

fn validate_build(build: &ProductivePackageBuildV1) -> Result<(), String> {
    if build.maximum_observed_scalars == u16::MAX
        || build.maximum_generated_scalars == u16::MAX
        || build.maximum_program_operations >= u16::MAX - 1
    {
        return Err(
            "productive V1 checked length or operation bound reaches wire ceiling".to_string(),
        );
    }
    let kinds = build
        .sections
        .iter()
        .map(|section| section.kind)
        .collect::<BTreeSet<_>>();
    if kinds.len() != REQUIRED_SECTION_COUNT
        || ProductiveSectionKindV1::ALL
            .into_iter()
            .any(|kind| !kinds.contains(&kind))
    {
        return Err(
            "productive V1 package must contain each required section exactly once".to_string(),
        );
    }
    for section in &build.sections {
        if section.flags != 0 {
            return Err("productive V1 section has unknown mandatory flags".to_string());
        }
        if let Some(expected) = section.kind.fixed_record_size() {
            if section.record_size != expected {
                return Err(format!(
                    "productive V1 {:?} record size mismatch",
                    section.kind
                ));
            }
            let expected_bytes = u64::from(section.count)
                .checked_mul(u64::from(expected))
                .ok_or_else(|| "productive V1 section byte count overflow".to_string())?;
            if expected_bytes != section.bytes.len() as u64 {
                return Err(format!(
                    "productive V1 {:?} count/bytes mismatch",
                    section.kind
                ));
            }
        } else if section.record_size != 0 {
            return Err("variable productive V1 section must use record_size zero".to_string());
        }
    }
    Ok(())
}

fn encode_header(
    header: &mut [u8],
    build: &ProductivePackageBuildV1,
    payload_sha256: [u8; 32],
    directory_offset: usize,
) -> Result<(), String> {
    header.fill(0);
    header[0..8].copy_from_slice(&MAGIC);
    put_u16(header, 8, FORMAT_VERSION)?;
    put_u16(header, 10, HEADER_BYTES as u16)?;
    put_u32(header, 12, build.mode.flags())?;
    header[16..48].copy_from_slice(&build.l11_package_sha256);
    header[48..80].copy_from_slice(&build.canonical_l2_package_sha256);
    header[80..112].copy_from_slice(&build.training_manifest_sha256);
    header[112..144].copy_from_slice(&payload_sha256);
    put_u64(header, 144, directory_offset as u64)?;
    put_u32(header, 152, REQUIRED_SECTION_COUNT as u32)?;
    put_u32(header, 156, BYTE_ORDER_MARKER)?;
    put_u16(header, 160, build.maximum_observed_scalars)?;
    put_u16(header, 162, build.maximum_generated_scalars)?;
    put_u16(header, 164, build.maximum_program_operations)?;
    let counts = |kind| {
        build
            .sections
            .iter()
            .find(|section| section.kind == kind)
            .map(|section| section.count)
            .expect("validated required section")
    };
    put_u32(header, 168, counts(ProductiveSectionKindV1::SlotKeys))?;
    put_u32(
        header,
        172,
        counts(ProductiveSectionKindV1::ParadigmCenters),
    )?;
    put_u32(header, 176, counts(ProductiveSectionKindV1::LemmaBindings))?;
    put_u32(
        header,
        180,
        counts(ProductiveSectionKindV1::MorphProgramHeaders),
    )?;
    put_u32(
        header,
        184,
        counts(ProductiveSectionKindV1::MorphOperations),
    )?;
    put_u32(header, 188, counts(ProductiveSectionKindV1::TrieNodes))?;
    put_u32(header, 192, counts(ProductiveSectionKindV1::TrieArcs))?;
    put_u32(header, 196, counts(ProductiveSectionKindV1::Terminals))?;
    put_u32(
        header,
        200,
        counts(ProductiveSectionKindV1::SlotPhaseProfiles),
    )?;
    put_u32(
        header,
        204,
        counts(ProductiveSectionKindV1::CalibrationCells),
    )?;
    put_u64(header, 208, build.split_seed)?;
    put_u32(header, 216, build.normalization_version)?;
    put_u32(header, 220, build.compiler_version)?;
    put_u64(header, 224, build.productive_package_byte_budget)?;
    put_u32(header, 232, build.steady_rss_kib_budget)?;
    put_u32(header, 236, build.peak_rss_kib_budget)?;
    put_u32(header, 240, build.cold_publish_budget_us)?;
    put_u32(header, 244, build.hot_p99_budget_us)?;
    put_u32(header, 248, 0)?;
    let crc = crc32(header);
    put_u32(header, 248, crc)?;
    Ok(())
}

fn encode_directory_entry(output: &mut [u8], entry: SectionDirectoryEntryV1) {
    output[0..2].copy_from_slice(&(entry.kind as u16).to_le_bytes());
    output[2..4].copy_from_slice(&entry.flags.to_le_bytes());
    output[4..8].copy_from_slice(&entry.record_size.to_le_bytes());
    output[8..16].copy_from_slice(&entry.offset.to_le_bytes());
    output[16..24].copy_from_slice(&entry.bytes.to_le_bytes());
    output[24..28].copy_from_slice(&entry.count.to_le_bytes());
    output[28..32].copy_from_slice(&entry.crc32.to_le_bytes());
}

fn decode_and_validate(
    bytes: &[u8],
) -> Result<
    (
        ProductivePackageHeaderV1,
        BTreeMap<ProductiveSectionKindV1, SectionDirectoryEntryV1>,
    ),
    String,
> {
    if bytes.len() < HEADER_BYTES || bytes[0..8] != MAGIC {
        return Err("productive V1 header magic or length mismatch".to_string());
    }
    if get_u16(bytes, 8)? != FORMAT_VERSION || get_u16(bytes, 10)? as usize != HEADER_BYTES {
        return Err("productive V1 header version mismatch".to_string());
    }
    let flags = get_u32(bytes, 12)?;
    if flags & !HEADER_KNOWN_FLAGS != 0 || flags.count_ones() != 1 {
        return Err("productive V1 header algorithm flags are invalid".to_string());
    }
    if get_u32(bytes, 156)? != BYTE_ORDER_MARKER {
        return Err("productive V1 byte order marker mismatch".to_string());
    }
    if bytes[166..168] != [0, 0] || bytes[252..256] != [0, 0, 0, 0] {
        return Err("productive V1 header reserved bytes are not zero".to_string());
    }
    let mut header_for_crc = bytes[..HEADER_BYTES].to_vec();
    let expected_header_crc = get_u32(bytes, 248)?;
    header_for_crc[248..252].fill(0);
    if crc32(&header_for_crc) != expected_header_crc {
        return Err("productive V1 header CRC mismatch".to_string());
    }
    let directory_offset = usize::try_from(get_u64(bytes, 144)?)
        .map_err(|_| "productive V1 directory offset exceeds address space".to_string())?;
    let section_count = get_u32(bytes, 152)? as usize;
    if directory_offset % 8 != 0 || section_count != REQUIRED_SECTION_COUNT {
        return Err("productive V1 directory alignment or count mismatch".to_string());
    }
    let directory_end = directory_offset
        .checked_add(section_count * DIRECTORY_ENTRY_BYTES)
        .ok_or_else(|| "productive V1 directory end overflow".to_string())?;
    if directory_offset < HEADER_BYTES || directory_end > bytes.len() {
        return Err("productive V1 directory lies outside file".to_string());
    }
    let section_floor = align8(directory_end)? as u64;
    let mut directory = BTreeMap::new();
    let mut ranges = Vec::new();
    for index in 0..section_count {
        let record = &bytes[directory_offset + index * DIRECTORY_ENTRY_BYTES
            ..directory_offset + (index + 1) * DIRECTORY_ENTRY_BYTES];
        let kind = ProductiveSectionKindV1::decode(get_u16(record, 0)?)?;
        let entry = SectionDirectoryEntryV1 {
            kind,
            flags: get_u16(record, 2)?,
            record_size: get_u32(record, 4)?,
            offset: get_u64(record, 8)?,
            bytes: get_u64(record, 16)?,
            count: get_u32(record, 24)?,
            crc32: get_u32(record, 28)?,
        };
        validate_directory_entry(bytes, entry, section_floor)?;
        if directory.insert(kind, entry).is_some() {
            return Err("productive V1 directory repeats a section kind".to_string());
        }
        ranges.push((entry.offset, entry.offset + entry.bytes));
    }
    if ProductiveSectionKindV1::ALL
        .into_iter()
        .any(|kind| !directory.contains_key(&kind))
    {
        return Err("productive V1 directory lacks a required section".to_string());
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err("productive V1 sections overlap".to_string());
    }
    let mut payload_hasher = Sha256::new();
    for kind in ProductiveSectionKindV1::ALL {
        let entry = directory[&kind];
        payload_hasher.update(&bytes[entry.offset as usize..(entry.offset + entry.bytes) as usize]);
    }
    let payload_sections_sha256: [u8; 32] = payload_hasher.finalize().into();
    if bytes[112..144] != payload_sections_sha256 {
        return Err("productive V1 payload SHA-256 mismatch".to_string());
    }
    let package_budget = get_u64(bytes, 224)?;
    if bytes.len() as u64 > package_budget {
        return Err("productive V1 package exceeds header byte budget".to_string());
    }
    let mode = if flags == HEADER_FLAG_V39_SPEED_PARITY {
        ProductiveAlgorithmModeV1::V39SpeedParity
    } else {
        ProductiveAlgorithmModeV1::ProductiveV1Model
    };
    let header = ProductivePackageHeaderV1 {
        mode,
        l11_package_sha256: bytes[16..48].try_into().expect("fixed slice"),
        canonical_l2_package_sha256: bytes[48..80].try_into().expect("fixed slice"),
        training_manifest_sha256: bytes[80..112].try_into().expect("fixed slice"),
        payload_sections_sha256,
        maximum_observed_scalars: get_u16(bytes, 160)?,
        maximum_generated_scalars: get_u16(bytes, 162)?,
        maximum_program_operations: get_u16(bytes, 164)?,
        split_seed: get_u64(bytes, 208)?,
        normalization_version: get_u32(bytes, 216)?,
        compiler_version: get_u32(bytes, 220)?,
        productive_package_byte_budget: package_budget,
        steady_rss_kib_budget: get_u32(bytes, 232)?,
        peak_rss_kib_budget: get_u32(bytes, 236)?,
        cold_publish_budget_us: get_u32(bytes, 240)?,
        hot_p99_budget_us: get_u32(bytes, 244)?,
    };
    validate_header_counts(bytes, &directory)?;
    validate_package_contents(bytes, &header, &directory)?;
    Ok((header, directory))
}

fn validate_directory_entry(
    bytes: &[u8],
    entry: SectionDirectoryEntryV1,
    section_floor: u64,
) -> Result<(), String> {
    if entry.flags != 0 || !entry.offset.is_multiple_of(8) {
        return Err("productive V1 section flags or alignment invalid".to_string());
    }
    let end = entry
        .offset
        .checked_add(entry.bytes)
        .ok_or_else(|| "productive V1 section range overflow".to_string())?;
    if entry.offset < section_floor || end > bytes.len() as u64 {
        return Err("productive V1 section lies outside file".to_string());
    }
    if let Some(expected) = entry.kind.fixed_record_size() {
        if entry.record_size != expected
            || u64::from(entry.count) * u64::from(expected) != entry.bytes
        {
            return Err("productive V1 fixed section record contract mismatch".to_string());
        }
    } else if entry.record_size != 0 {
        return Err("productive V1 variable section record size is not zero".to_string());
    }
    let section = &bytes[entry.offset as usize..end as usize];
    if crc32(section) != entry.crc32 {
        return Err("productive V1 section CRC mismatch".to_string());
    }
    Ok(())
}

fn validate_header_counts(
    bytes: &[u8],
    directory: &BTreeMap<ProductiveSectionKindV1, SectionDirectoryEntryV1>,
) -> Result<(), String> {
    let expected = [
        (168, ProductiveSectionKindV1::SlotKeys),
        (172, ProductiveSectionKindV1::ParadigmCenters),
        (176, ProductiveSectionKindV1::LemmaBindings),
        (180, ProductiveSectionKindV1::MorphProgramHeaders),
        (184, ProductiveSectionKindV1::MorphOperations),
        (188, ProductiveSectionKindV1::TrieNodes),
        (192, ProductiveSectionKindV1::TrieArcs),
        (196, ProductiveSectionKindV1::Terminals),
        (200, ProductiveSectionKindV1::SlotPhaseProfiles),
        (204, ProductiveSectionKindV1::CalibrationCells),
    ];
    for (offset, kind) in expected {
        if get_u32(bytes, offset)? != directory[&kind].count {
            return Err("productive V1 header count disagrees with directory".to_string());
        }
    }
    Ok(())
}

fn align8(value: usize) -> Result<usize, String> {
    value
        .checked_add(7)
        .map(|value| value & !7)
        .ok_or_else(|| "productive V1 alignment overflow".to_string())
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

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<(), String> {
    let target = bytes
        .get_mut(offset..offset + 2)
        .ok_or_else(|| "productive V1 u16 write outside buffer".to_string())?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    let target = bytes
        .get_mut(offset..offset + 4)
        .ok_or_else(|| "productive V1 u32 write outside buffer".to_string())?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) -> Result<(), String> {
    let target = bytes
        .get_mut(offset..offset + 8)
        .ok_or_else(|| "productive V1 u64 write outside buffer".to_string())?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn get_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .map(|value| u16::from_le_bytes(value.try_into().expect("fixed slice")))
        .ok_or_else(|| "productive V1 u16 read outside buffer".to_string())
}

fn get_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .map(|value| u32::from_le_bytes(value.try_into().expect("fixed slice")))
        .ok_or_else(|| "productive V1 u32 read outside buffer".to_string())
}

fn get_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    bytes
        .get(offset..offset + 8)
        .map(|value| u64::from_le_bytes(value.try_into().expect("fixed slice")))
        .ok_or_else(|| "productive V1 u64 read outside buffer".to_string())
}

#[cfg(test)]
mod tests {
    use super::super::records::*;
    use super::super::types::{LemmaParadigmBindingV1, MorphologySlotKeyV1};
    use super::*;

    fn package() -> ProductivePackageBuildV1 {
        let mut build = ProductivePackageBuildV1::with_empty_required_sections(
            ProductiveAlgorithmModeV1::V39SpeedParity,
        );
        build.l11_package_sha256 = [1; 32];
        build.canonical_l2_package_sha256 = [2; 32];
        build.training_manifest_sha256 = [3; 32];
        build.maximum_observed_scalars = 64;
        build.maximum_generated_scalars = 96;
        build.maximum_program_operations = 12;
        build.split_seed = 17;
        build.normalization_version = 1;
        build.compiler_version = 1;
        build.productive_package_byte_budget = 1 << 20;
        for (kind, magic) in [
            (ProductiveSectionKindV1::AxisDictionaries, b"ADV1"),
            (ProductiveSectionKindV1::SegmentPool, b"SPV1"),
        ] {
            let section = build
                .sections
                .iter_mut()
                .find(|section| section.kind == kind)
                .expect("variable pool section");
            section.bytes.extend_from_slice(magic);
            section.bytes.extend_from_slice(&0_u32.to_le_bytes());
        }
        build
    }

    fn replace_fixed<T: FixedRecordV1>(
        build: &mut ProductivePackageBuildV1,
        kind: ProductiveSectionKindV1,
        records: &[T],
    ) {
        let replacement =
            ProductiveSectionBuildV1::fixed_records(kind, records).expect("fixed section");
        *build
            .sections
            .iter_mut()
            .find(|section| section.kind == kind)
            .expect("required section") = replacement;
    }

    fn replace_pool(
        build: &mut ProductivePackageBuildV1,
        kind: ProductiveSectionKindV1,
        bytes: Vec<u8>,
        count: usize,
    ) {
        let section = build
            .sections
            .iter_mut()
            .find(|section| section.kind == kind)
            .expect("required pool");
        section.count = count as u32;
        section.bytes = bytes;
    }

    fn aligned_pool(magic: &[u8; 4], entries: &[(u16, Vec<u8>)]) -> (Vec<u8>, Vec<u32>) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(magic);
        bytes.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        let mut references = Vec::new();
        for (scalar_length, payload) in entries {
            references.push(bytes.len() as u32);
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&scalar_length.to_le_bytes());
            bytes.extend_from_slice(&0_u16.to_le_bytes());
            bytes.extend_from_slice(payload);
            bytes.resize((bytes.len() + 7) & !7, 0);
        }
        (bytes, references)
    }

    fn axis_label(axis: u8, value: u8, label: &str) -> (u16, Vec<u8>) {
        let mut payload = vec![1, axis, value, 0];
        payload.extend_from_slice(label.as_bytes());
        (label.chars().count() as u16, payload)
    }

    fn observed_slots(slots: &[u32]) -> (u16, Vec<u8>) {
        let mut payload = vec![2, 0, 0, 0];
        payload.extend_from_slice(&(slots.len() as u32).to_le_bytes());
        for slot in slots {
            payload.extend_from_slice(&slot.to_le_bytes());
        }
        (0, payload)
    }

    fn semantic_package() -> ProductivePackageBuildV1 {
        let mut build = ProductivePackageBuildV1::with_empty_required_sections(
            ProductiveAlgorithmModeV1::ProductiveV1Model,
        );
        build.l11_package_sha256 = [1; 32];
        build.canonical_l2_package_sha256 = [2; 32];
        build.training_manifest_sha256 = [3; 32];
        build.maximum_observed_scalars = 64;
        build.maximum_generated_scalars = 96;
        build.maximum_program_operations = 3;
        build.split_seed = 17;
        build.normalization_version = 1;
        build.compiler_version = 1;
        build.productive_package_byte_budget = 1 << 20;

        let (axis_pool, axis_refs) = aligned_pool(
            b"ADV1",
            &[
                axis_label(0, 2, "noun"),
                axis_label(1, 2, "singular"),
                axis_label(1, 3, "plural"),
                observed_slots(&[1]),
            ],
        );
        replace_pool(
            &mut build,
            ProductiveSectionKindV1::AxisDictionaries,
            axis_pool,
            axis_refs.len(),
        );
        let (segment_pool, segment_refs) =
            aligned_pool(b"SPV1", &[(4, b"cats".to_vec()), (1, b"s".to_vec())]);
        replace_pool(
            &mut build,
            ProductiveSectionKindV1::SegmentPool,
            segment_pool,
            segment_refs.len(),
        );

        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::SlotKeys,
            &[
                MorphologySlotKeyV1::new(2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
                MorphologySlotKeyV1::new(2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::ParadigmCenters,
            &[ParadigmCenterRecordV1 {
                pos_domain: 2,
                root_node: 0,
                transition_start: 0,
                transition_count: 1,
                slot_profile_start: 0,
                slot_profile_count: 1,
                program_start: 0,
                program_count: 1,
                support: 2,
                stability: 100,
                calibration_class: 1,
                provenance_ref: 1,
                signature_hash_low: 7,
                flags: 0,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::LemmaBindings,
            &[LemmaParadigmBindingV1 {
                lemma_id: 1,
                paradigm_id: 1,
                canonical_source_form_ref: 1,
                observed_slot_set_ref: axis_refs[3],
                positive_support: 2,
                explicit_anti_support: 0,
                stability: 100,
                flags: 0,
                program_start: 1,
                program_count: 0,
                provenance_ref: 1,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::ParadigmCompatibilityIndex,
            &[ParadigmCompatibilityIndexRecordV1 {
                pos_domain: 2,
                flags: 0,
                source_slot_id: 1,
                posting_start: 0,
                posting_count: 1,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::ParadigmPostings,
            &[ParadigmPostingRecordV1 { paradigm_id: 1 }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::MorphProgramHeaders,
            &[MorphProgramHeaderRecordV1 {
                source_slot_id: 1,
                target_slot_id: 2,
                op_start: 0,
                op_count: 3,
                flags: 0,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::MorphOperations,
            &[
                MorphOpRecordV1 {
                    opcode: MorphOpcodeV1::CopySourceRange as u8,
                    anchor: 1,
                    arg0: 0,
                    arg1: u32::from(u16::MAX),
                    ..MorphOpRecordV1::default()
                },
                MorphOpRecordV1 {
                    opcode: MorphOpcodeV1::EmitSegment as u8,
                    arg1: segment_refs[1],
                    ..MorphOpRecordV1::default()
                },
                MorphOpRecordV1 {
                    opcode: MorphOpcodeV1::Terminate as u8,
                    arg1: 2,
                    arg2: 1,
                    ..MorphOpRecordV1::default()
                },
            ],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::TrieNodes,
            &[
                ProductiveTrieNodeRecordV1 {
                    arc_start: 0,
                    arc_count: 1,
                    ..ProductiveTrieNodeRecordV1::default()
                },
                ProductiveTrieNodeRecordV1 {
                    arc_start: 1,
                    arc_count: 1,
                    ..ProductiveTrieNodeRecordV1::default()
                },
                ProductiveTrieNodeRecordV1 {
                    arc_start: 2,
                    terminal_start: 0,
                    terminal_count: 1,
                    ..ProductiveTrieNodeRecordV1::default()
                },
            ],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::TrieArcs,
            &[
                ProductiveTrieArcRecordV1 {
                    child_node: 1,
                    stable_order: 1,
                    opcode: ProductiveTrieArcOpcodeV1::CopyToRetainedEdge as u8,
                    anchor: 1,
                    flags: 0,
                    arg0: 0,
                    arg1: 0,
                    arg2: 0,
                },
                ProductiveTrieArcRecordV1 {
                    child_node: 2,
                    stable_order: 1,
                    opcode: ProductiveTrieArcOpcodeV1::EmitSegment as u8,
                    anchor: 0,
                    flags: 0,
                    arg0: 0,
                    arg1: segment_refs[1],
                    arg2: 0,
                },
            ],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::Terminals,
            &[ProductiveTerminalRecordV1 {
                program_id: 1,
                target_slot_id: 2,
                variant_id: 1,
                flags: super::super::records::PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE,
                decoder_ref: 0,
                evidence_ref: 1,
                calibration_class: 1,
                provenance_ref: 1,
                stable_identity_hash: 11,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::SlotPhaseProfiles,
            &[SlotPhaseProfileRecordV1 {
                slot_id: 2,
                feature_schema_id: 1,
                positive_start: 0,
                positive_count: 1,
                calibration_class: 1,
                support: 2,
                ..SlotPhaseProfileRecordV1::default()
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::PositivePhaseCenters,
            &[PhaseCenterRecordV1 {
                cells: [1; super::super::L2_SCENE_PHASE_CELLS],
                feature_mask: 1,
                context_mode_id: 1,
                support: 2,
                mass: u16::MAX,
                polarity: 1,
                flags: 0,
            }],
        );
        replace_fixed::<PhaseCenterRecordV1>(
            &mut build,
            ProductiveSectionKindV1::AntiPhaseCenters,
            &[],
        );
        replace_fixed::<PhaseCenterRecordV1>(
            &mut build,
            ProductiveSectionKindV1::HardNegativePhaseCenters,
            &[],
        );
        replace_fixed::<PhaseCenterRecordV1>(
            &mut build,
            ProductiveSectionKindV1::AmbiguityPhaseCenters,
            &[],
        );
        replace_fixed::<DirectionalResidualRecordV1>(
            &mut build,
            ProductiveSectionKindV1::DirectionalResiduals,
            &[],
        );
        let coefficients = (1..=super::super::score::PRODUCTIVE_FEATURE_COUNT)
            .map(|feature_id| ModelCoefficientRecordV1 {
                feature_id: feature_id as u16,
                flags: 0,
                coefficient_q16: 1,
                train_support: 2,
                feature_schema_hash_low: 17,
            })
            .collect::<Vec<_>>();
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::ModelCoefficients,
            &coefficients,
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::EvidencePriors,
            &[
                EvidencePriorRecordV1 {
                    channel_id: 1,
                    positive_prior_twice: 5,
                    contradiction_prior_twice: 1,
                    ..EvidencePriorRecordV1::default()
                },
                EvidencePriorRecordV1 {
                    channel_id: 2,
                    positive_prior_twice: 5,
                    contradiction_prior_twice: 1,
                    ..EvidencePriorRecordV1::default()
                },
                EvidencePriorRecordV1 {
                    channel_id: 3,
                    positive_prior_twice: 5,
                    contradiction_prior_twice: 1,
                    ..EvidencePriorRecordV1::default()
                },
                EvidencePriorRecordV1 {
                    channel_id: 4,
                    positive_prior_twice: 5,
                    contradiction_prior_twice: 1,
                    ..EvidencePriorRecordV1::default()
                },
            ],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::CalibrationCells,
            &[CalibrationCellRecordV1 {
                stratum_key_id: 1,
                winner_margin_q16: i32::MIN,
                tie_radius_q16: 0,
                support: 200,
                correct_winner_count: 200,
                false_winner_count: 0,
                tied_count: 0,
                flags: 0,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::Provenance,
            &[ProvenanceRecordV1 {
                source_kind: 1,
                flags: 0,
                source_id: 1,
                event_start: 0,
                event_count: 1,
                source_hash_prefix: 1,
            }],
        );
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::DeltaManifest,
            &[DeltaManifestRecordV1 {
                section_count_ref: REQUIRED_SECTION_COUNT as u64,
                requested_authority_scope: 1,
                ..DeltaManifestRecordV1::default()
            }],
        );
        build
    }

    fn replace_one_record<T: FixedRecordV1>(
        build: &mut ProductivePackageBuildV1,
        kind: ProductiveSectionKindV1,
        record: T,
    ) {
        replace_fixed(build, kind, &[record]);
    }

    #[test]
    fn checked_package_roundtrip_preserves_header_and_sections() {
        let bytes = encode_package(&package()).expect("encode");
        let view = ProductivePackageViewV1::from_bytes(bytes.clone()).expect("view");
        assert_eq!(view.header.mode, ProductiveAlgorithmModeV1::V39SpeedParity);
        assert_eq!(view.header.split_seed, 17);
        assert!(view.section(ProductiveSectionKindV1::SlotKeys).is_empty());
        assert_eq!(view.backing_bytes(), bytes.len());
        let expected_sha256: [u8; 32] = Sha256::digest(&bytes).into();
        assert_eq!(view.package_sha256(), expected_sha256);
        assert!(!view.mmap_backed());
    }

    #[test]
    fn package_reader_rejects_section_corruption() {
        let mut bytes = encode_package(&package()).expect("encode");
        let view = ProductivePackageViewV1::from_bytes(bytes.clone()).expect("view");
        let pool_offset = view.directory[&ProductiveSectionKindV1::SegmentPool].offset as usize;
        bytes[pool_offset] ^= 1;
        assert!(ProductivePackageViewV1::from_bytes(bytes).is_err());
    }

    #[test]
    fn deep_reader_accepts_a_fully_attributed_semantic_package() {
        let bytes = encode_package(&semantic_package()).expect("encode");
        let view = ProductivePackageViewV1::from_bytes(bytes).expect("deep view");
        assert_eq!(
            view.header.mode,
            ProductiveAlgorithmModeV1::ProductiveV1Model
        );
    }

    #[test]
    fn deep_reader_rejects_a_trie_cycle_with_valid_section_checksums() {
        let mut build = semantic_package();
        replace_fixed(
            &mut build,
            ProductiveSectionKindV1::TrieArcs,
            &[
                ProductiveTrieArcRecordV1 {
                    child_node: 1,
                    stable_order: 1,
                    opcode: ProductiveTrieArcOpcodeV1::CopyToRetainedEdge as u8,
                    anchor: 1,
                    flags: 0,
                    arg0: 0,
                    arg1: 0,
                    arg2: 0,
                },
                ProductiveTrieArcRecordV1 {
                    child_node: 0,
                    stable_order: 1,
                    opcode: ProductiveTrieArcOpcodeV1::EmitSegment as u8,
                    anchor: 0,
                    flags: 0,
                    arg0: 0,
                    arg1: 24,
                    arg2: 0,
                },
            ],
        );
        let bytes = encode_package(&build).expect("structurally encoded");
        assert!(ProductivePackageViewV1::from_bytes(bytes).is_err());
    }

    #[test]
    fn deep_reader_rejects_phase_section_polarity_disagreement() {
        let mut build = semantic_package();
        replace_one_record(
            &mut build,
            ProductiveSectionKindV1::PositivePhaseCenters,
            PhaseCenterRecordV1 {
                cells: [1; super::super::L2_SCENE_PHASE_CELLS],
                feature_mask: 1,
                context_mode_id: 1,
                support: 2,
                mass: u16::MAX,
                polarity: -1,
                flags: 0,
            },
        );
        let bytes = encode_package(&build).expect("structurally encoded");
        assert!(ProductivePackageViewV1::from_bytes(bytes).is_err());
    }

    #[test]
    fn deep_reader_rejects_a_terminal_decoder_reference_to_no_pool_entry() {
        let mut build = semantic_package();
        replace_one_record(
            &mut build,
            ProductiveSectionKindV1::Terminals,
            ProductiveTerminalRecordV1 {
                program_id: 1,
                target_slot_id: 2,
                variant_id: 1,
                flags: 0,
                decoder_ref: 7,
                evidence_ref: 1,
                calibration_class: 1,
                provenance_ref: 1,
                stable_identity_hash: 11,
            },
        );
        let bytes = encode_package(&build).expect("structurally encoded");
        assert!(ProductivePackageViewV1::from_bytes(bytes).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn package_load_is_checked_and_mmap_backed() {
        let path = std::env::temp_dir().join(format!(
            "lay-productive-v1-package-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&path, encode_package(&package()).expect("encode")).expect("write");
        let view = ProductivePackageViewV1::load(&path).expect("mmap view");
        assert!(view.mmap_backed());
        std::fs::remove_file(path).expect("cleanup");
    }
}
