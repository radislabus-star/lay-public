use std::collections::{BTreeMap, BTreeSet};

use super::format::{self, Cursor};
use super::model::{
    CompetitionEdge, FormCenterRef, L2FieldPackage, LemmaCenter, LocalContextMode, MorphBinding,
    NeighborCoupling, SlotPhaseCenter, TieCalibration,
};
use super::{compositional::LemmaWaveIndex, compositional_format};

const MAGIC: &[u8; 8] = b"LAYL2C01";
const LEGACY_VERSION: u32 = 1;
const VERSION: u32 = 2;
const HEADER_BYTES: usize = 128;
pub(super) const DECODER_BLOCK_FORMS: usize = 32;
const COMPACT_FORM_REF_BYTES: usize = 8;
const COMPACT_BINDING_BYTES: usize = 9;
const LEMMA_CENTER_BYTES: usize = 32;
const CONTEXT_MODE_BYTES: usize = 16;
const SLOT_CENTER_BYTES: usize = 76;
const NEIGHBOR_COUPLING_BYTES: usize = 24;
const COMPETITION_EDGE_BYTES: usize = 24;
const CALIBRATION_BYTES: usize = 24;

pub(super) fn is_compact_package(bytes: &[u8]) -> bool {
    bytes.get(..MAGIC.len()) == Some(MAGIC)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CompactFormatStats {
    pub(super) version: u32,
    pub(super) total_bytes: usize,
    pub(super) header_bytes: usize,
    pub(super) form_ref_bytes: usize,
    pub(super) decoder_offset_bytes: usize,
    pub(super) decoder_payload_bytes: usize,
    pub(super) feature_dictionary_bytes: usize,
    pub(super) lemma_center_bytes: usize,
    pub(super) morph_binding_bytes: usize,
    pub(super) context_mode_bytes: usize,
    pub(super) slot_center_bytes: usize,
    pub(super) neighbor_coupling_bytes: usize,
    pub(super) competition_edge_bytes: usize,
    pub(super) calibration_bytes: usize,
    pub(super) lemma_wave_range_bytes: usize,
    pub(super) surface_wave_code_bytes: usize,
    pub(super) wave_band_offset_bytes: usize,
    pub(super) wave_band_posting_bytes: usize,
    pub(super) atom_key_bytes: usize,
    pub(super) atom_offset_bytes: usize,
    pub(super) atom_posting_bytes: usize,
    pub(super) decoder_blocks: usize,
    pub(super) feature_dictionary_entries: usize,
    pub(super) lemma_wave_ranges: usize,
    pub(super) surface_wave_codes: usize,
    pub(super) wave_band_offsets: usize,
    pub(super) wave_band_postings: usize,
    pub(super) atom_keys: usize,
    pub(super) atom_offsets: usize,
    pub(super) atom_postings: usize,
}

pub(super) fn encode_package(
    package: &L2FieldPackage,
) -> Result<(Vec<u8>, CompactFormatStats), String> {
    encode_package_version(package, VERSION)
}

fn encode_package_version(
    package: &L2FieldPackage,
    version: u32,
) -> Result<(Vec<u8>, CompactFormatStats), String> {
    if !matches!(version, LEGACY_VERSION | VERSION) {
        return Err(format!("unsupported compact L2 package version {version}"));
    }
    validate_compact_source(package)?;

    let feature_masks = package
        .morph_bindings
        .iter()
        .map(|binding| binding.feature_mask)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if feature_masks.len() > usize::from(u8::MAX) + 1 {
        return Err(format!(
            "compact L2 feature dictionary has {} entries; maximum is 256",
            feature_masks.len()
        ));
    }
    let feature_ids = feature_masks
        .iter()
        .enumerate()
        .map(|(index, mask)| (*mask, index as u8))
        .collect::<BTreeMap<_, _>>();
    let (decoder_offsets, decoder_payload) = encode_decoder(package)?;
    let lemma_wave_index = (version >= VERSION)
        .then(|| LemmaWaveIndex::build_reference(package))
        .transpose()?;

    let mut body = Vec::new();
    for form in &package.form_refs {
        format::put_u32(&mut body, form.l1_terminal_id);
        format::put_u16(&mut body, form.script_flags);
        body.push(form.length_bucket);
        body.push(form.flags);
    }
    for offset in &decoder_offsets {
        format::put_u32(&mut body, *offset);
    }
    body.extend_from_slice(&decoder_payload);
    for feature_mask in &feature_masks {
        format::put_u32(&mut body, *feature_mask);
    }
    for center in &package.lemma_centers {
        format::put_lemma_center(&mut body, *center);
    }
    for binding in &package.morph_bindings {
        format::put_u32(&mut body, binding.form_center_ref);
        body.push(feature_ids[&binding.feature_mask]);
        format::put_u16(&mut body, binding.support);
        body.push(binding.phase as u8);
        body.push(binding.flags);
    }
    for mode in &package.context_modes {
        format::put_context_mode(&mut body, *mode);
    }
    for center in &package.slot_centers {
        format::put_slot_center(&mut body, *center);
    }
    for coupling in &package.neighbor_couplings {
        format::put_neighbor_coupling(&mut body, *coupling);
    }
    for edge in &package.competition_edges {
        format::put_competition_edge(&mut body, *edge);
    }
    format::put_calibration(&mut body, package.calibration);
    if let Some(index) = &lemma_wave_index {
        compositional_format::encode(index, &mut body);
    }

    let total_bytes = HEADER_BYTES
        .checked_add(body.len())
        .ok_or_else(|| "compact L2 package size overflow".to_string())?;
    let mut bytes = Vec::with_capacity(total_bytes);
    bytes.extend_from_slice(MAGIC);
    format::put_u32(&mut bytes, version);
    format::put_u32(&mut bytes, HEADER_BYTES as u32);
    format::put_u64(&mut bytes, total_bytes as u64);
    format::put_u64(&mut bytes, format::checksum64(&body));
    format::put_u64(&mut bytes, package.l1_package_fingerprint);
    let base_counts = [
        DECODER_BLOCK_FORMS,
        package.form_refs.len(),
        decoder_offsets.len(),
        decoder_payload.len(),
        feature_masks.len(),
        package.lemma_centers.len(),
        package.morph_bindings.len(),
        package.context_modes.len(),
        package.slot_centers.len(),
        package.neighbor_couplings.len(),
        package.competition_edges.len(),
    ];
    for value in base_counts {
        format::put_u32(
            &mut bytes,
            u32::try_from(value).map_err(|_| "compact L2 section count exceeds u32".to_string())?,
        );
    }
    if let Some(index) = &lemma_wave_index {
        for value in [
            index.ranges().len(),
            index.centers().len(),
            index.band_offsets().len(),
            index.band_postings().len(),
            index.atom_keys().len(),
            index.atom_offsets().len(),
            index.atom_postings().len(),
        ] {
            format::put_u32(
                &mut bytes,
                u32::try_from(value)
                    .map_err(|_| "compact L2 lemma wave count exceeds u32".to_string())?,
            );
        }
    }
    bytes.resize(HEADER_BYTES, 0);
    bytes.extend_from_slice(&body);

    let stats = CompactFormatStats {
        version,
        total_bytes,
        header_bytes: HEADER_BYTES,
        form_ref_bytes: package.form_refs.len() * COMPACT_FORM_REF_BYTES,
        decoder_offset_bytes: decoder_offsets.len() * std::mem::size_of::<u32>(),
        decoder_payload_bytes: decoder_payload.len(),
        feature_dictionary_bytes: feature_masks.len() * std::mem::size_of::<u32>(),
        lemma_center_bytes: package.lemma_centers.len() * LEMMA_CENTER_BYTES,
        morph_binding_bytes: package.morph_bindings.len() * COMPACT_BINDING_BYTES,
        context_mode_bytes: package.context_modes.len() * CONTEXT_MODE_BYTES,
        slot_center_bytes: package.slot_centers.len() * SLOT_CENTER_BYTES,
        neighbor_coupling_bytes: package.neighbor_couplings.len() * NEIGHBOR_COUPLING_BYTES,
        competition_edge_bytes: package.competition_edges.len() * COMPETITION_EDGE_BYTES,
        calibration_bytes: CALIBRATION_BYTES,
        lemma_wave_range_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.ranges().len() * compositional_format::LEMMA_WAVE_RANGE_BYTES
        }),
        surface_wave_code_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.centers().len() * compositional_format::SURFACE_WAVE_CODE_BYTES
        }),
        wave_band_offset_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.band_offsets().len() * compositional_format::WAVE_BAND_OFFSET_BYTES
        }),
        wave_band_posting_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.band_postings().len() * compositional_format::WAVE_BAND_POSTING_BYTES
        }),
        atom_key_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.atom_keys().len() * compositional_format::ATOM_KEY_BYTES
        }),
        atom_offset_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.atom_offsets().len() * compositional_format::ATOM_OFFSET_BYTES
        }),
        atom_posting_bytes: lemma_wave_index.as_ref().map_or(0, |index| {
            index.atom_postings().len() * compositional_format::ATOM_POSTING_BYTES
        }),
        decoder_blocks: decoder_offsets.len(),
        feature_dictionary_entries: feature_masks.len(),
        lemma_wave_ranges: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.ranges().len()),
        surface_wave_codes: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.centers().len()),
        wave_band_offsets: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.band_offsets().len()),
        wave_band_postings: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.band_postings().len()),
        atom_keys: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.atom_keys().len()),
        atom_offsets: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.atom_offsets().len()),
        atom_postings: lemma_wave_index
            .as_ref()
            .map_or(0, |index| index.atom_postings().len()),
    };
    debug_assert_eq!(stats.total_bytes, bytes.len());
    Ok((bytes, stats))
}

pub(super) fn decode_package(bytes: &[u8]) -> Result<L2FieldPackage, String> {
    if bytes.len() < HEADER_BYTES || bytes.get(..MAGIC.len()) != Some(MAGIC) {
        return Err("invalid compact L2 package magic or truncated header".to_string());
    }
    let mut header = Cursor::new(&bytes[MAGIC.len()..HEADER_BYTES]);
    let version = header.u32()?;
    if !matches!(version, LEGACY_VERSION | VERSION) {
        return Err(format!("unsupported compact L2 package version {version}"));
    }
    let header_bytes = header.u32()? as usize;
    if header_bytes != HEADER_BYTES {
        return Err(format!("invalid compact L2 header size {header_bytes}"));
    }
    let total_bytes = usize::try_from(header.u64()?)
        .map_err(|_| "compact L2 package size does not fit usize".to_string())?;
    if total_bytes != bytes.len() {
        return Err(format!(
            "compact L2 package size mismatch: header={total_bytes} actual={}",
            bytes.len()
        ));
    }
    let expected_checksum = header.u64()?;
    let l1_package_fingerprint = header.u64()?;
    let counts = read_header_counts(&mut header, version)?;
    validate_header_counts(version, counts)?;

    let body = &bytes[HEADER_BYTES..];
    if format::checksum64(body) != expected_checksum {
        return Err("compact L2 package checksum mismatch".to_string());
    }
    let mut cursor = Cursor::new(body);
    let compact_forms = read_many(counts.form_refs, || {
        Ok(CompactFormRef {
            l1_terminal_id: cursor.u32()?,
            script_flags: cursor.u16()?,
            length_bucket: cursor.u8()?,
            flags: cursor.u8()?,
        })
    })?;
    let decoder_offsets = read_many(counts.decoder_blocks, || cursor.u32())?;
    let decoder_payload = cursor.bytes(counts.decoder_payload_bytes)?;
    let feature_masks = read_many(counts.feature_masks, || cursor.u32())?;
    if feature_masks.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("compact L2 feature dictionary is not strictly ordered".to_string());
    }
    let lemma_centers = read_many(counts.lemma_centers, || {
        format::read_lemma_center(&mut cursor)
    })?;
    validate_lemma_binding_ranges(&lemma_centers, counts.morph_bindings, None)?;

    let mut lemma_id = 0_usize;
    let morph_bindings = read_many_indexed(counts.morph_bindings, |binding_index| {
        while lemma_id < lemma_centers.len()
            && binding_index
                >= lemma_centers[lemma_id].form_start as usize
                    + lemma_centers[lemma_id].form_count as usize
        {
            lemma_id += 1;
        }
        let center = lemma_centers
            .get(lemma_id)
            .ok_or_else(|| "compact L2 binding has no owning lemma range".to_string())?;
        if binding_index < center.form_start as usize {
            return Err("compact L2 lemma binding ranges contain a gap".to_string());
        }
        let form_center_ref = cursor.u32()?;
        let feature_id = cursor.u8()? as usize;
        let feature_mask = feature_masks
            .get(feature_id)
            .copied()
            .ok_or_else(|| format!("compact L2 binding uses missing feature ID {feature_id}"))?;
        Ok(MorphBinding {
            form_center_ref,
            lemma_center_id: lemma_id as u32,
            feature_mask,
            support: cursor.u16()?,
            phase: cursor.i8()?,
            flags: cursor.u8()?,
        })
    })?;
    let context_modes = read_many(counts.context_modes, || {
        format::read_context_mode(&mut cursor)
    })?;
    let slot_centers = read_many(counts.slot_centers, || {
        format::read_slot_center(&mut cursor)
    })?;
    let neighbor_couplings = read_many(counts.neighbor_couplings, || {
        format::read_neighbor_coupling(&mut cursor)
    })?;
    let competition_edges = read_many(counts.competition_edges, || {
        format::read_competition_edge(&mut cursor)
    })?;
    let calibration = format::read_calibration(&mut cursor)?;
    let wave_range_bytes = cursor.bytes(
        counts
            .lemma_wave_ranges
            .saturating_mul(compositional_format::LEMMA_WAVE_RANGE_BYTES),
    )?;
    let wave_center_bytes = cursor.bytes(
        counts
            .surface_wave_codes
            .saturating_mul(compositional_format::SURFACE_WAVE_CODE_BYTES),
    )?;
    let wave_band_offset_bytes = cursor.bytes(
        counts
            .wave_band_offsets
            .saturating_mul(compositional_format::WAVE_BAND_OFFSET_BYTES),
    )?;
    let wave_band_posting_bytes = cursor.bytes(
        counts
            .wave_band_postings
            .saturating_mul(compositional_format::WAVE_BAND_POSTING_BYTES),
    )?;
    let atom_key_bytes = cursor.bytes(
        counts
            .atom_keys
            .saturating_mul(compositional_format::ATOM_KEY_BYTES),
    )?;
    let atom_offset_bytes = cursor.bytes(
        counts
            .atom_offsets
            .saturating_mul(compositional_format::ATOM_OFFSET_BYTES),
    )?;
    let atom_posting_bytes = cursor.bytes(
        counts
            .atom_postings
            .saturating_mul(compositional_format::ATOM_POSTING_BYTES),
    )?;
    let lemma_wave_index = (!wave_range_bytes.is_empty() || !wave_center_bytes.is_empty())
        .then(|| {
            compositional_format::decode(
                wave_range_bytes,
                counts.lemma_wave_ranges,
                wave_center_bytes,
                counts.surface_wave_codes,
                wave_band_offset_bytes,
                counts.wave_band_offsets,
                wave_band_posting_bytes,
                counts.wave_band_postings,
                atom_key_bytes,
                counts.atom_keys,
                atom_offset_bytes,
                counts.atom_offsets,
                atom_posting_bytes,
                counts.atom_postings,
            )
        })
        .transpose()?;
    validate_embedded_wave_index(version, counts.lemma_centers, lemma_wave_index.as_ref())?;
    if cursor.remaining() != 0 {
        return Err(format!(
            "compact L2 package has {} trailing bytes",
            cursor.remaining()
        ));
    }
    let (form_refs, decoder_bytes) =
        decode_decoder(&compact_forms, &decoder_offsets, decoder_payload)?;
    let package = L2FieldPackage {
        l1_package_fingerprint,
        form_refs,
        decoder_bytes,
        lemma_centers,
        morph_bindings,
        context_modes,
        slot_centers,
        neighbor_couplings,
        competition_edges,
        calibration,
    };
    format::validate_package(&package)?;
    Ok(package)
}

fn validate_compact_source(package: &L2FieldPackage) -> Result<(), String> {
    format::validate_package(package)?;
    validate_lemma_binding_ranges(
        &package.lemma_centers,
        package.morph_bindings.len(),
        Some(&package.morph_bindings),
    )?;
    let mut expected_decoder_ref = 0_usize;
    for (index, form) in package.form_refs.iter().enumerate() {
        if form.reserved != 0 {
            return Err(format!(
                "compact L2 form {index} has non-zero reserved field"
            ));
        }
        if form.decoder_ref as usize != expected_decoder_ref {
            return Err(format!(
                "compact L2 source decoder is not canonical at form {index}"
            ));
        }
        let surface = format::decoder_surface(&package.decoder_bytes, form.decoder_ref)?;
        expected_decoder_ref = expected_decoder_ref
            .checked_add(surface.len() + 1)
            .ok_or_else(|| "compact L2 decoder offset overflow".to_string())?;
    }
    if expected_decoder_ref != package.decoder_bytes.len() {
        return Err("compact L2 source decoder contains unreferenced bytes".to_string());
    }
    Ok(())
}

fn validate_lemma_binding_ranges(
    centers: &[super::model::LemmaCenter],
    binding_count: usize,
    bindings: Option<&[MorphBinding]>,
) -> Result<(), String> {
    let mut expected_start = 0_usize;
    for (lemma_id, center) in centers.iter().enumerate() {
        if center.form_start as usize != expected_start {
            return Err(format!(
                "compact L2 lemma {lemma_id} binding range starts at {}, expected {expected_start}",
                center.form_start
            ));
        }
        let end = expected_start
            .checked_add(center.form_count as usize)
            .ok_or_else(|| "compact L2 lemma binding range overflow".to_string())?;
        if end > binding_count {
            return Err(format!(
                "compact L2 lemma {lemma_id} binding range exceeds section"
            ));
        }
        if let Some(bindings) = bindings {
            if bindings[expected_start..end]
                .iter()
                .any(|binding| binding.lemma_center_id as usize != lemma_id)
            {
                return Err(format!(
                    "compact L2 lemma {lemma_id} range contains a foreign binding"
                ));
            }
        }
        expected_start = end;
    }
    if expected_start != binding_count {
        return Err(format!(
            "compact L2 lemma ranges cover {expected_start} of {binding_count} bindings"
        ));
    }
    Ok(())
}

fn encode_decoder(package: &L2FieldPackage) -> Result<(Vec<u32>, Vec<u8>), String> {
    let mut offsets = Vec::with_capacity(package.form_refs.len().div_ceil(DECODER_BLOCK_FORMS));
    let mut payload = Vec::new();
    let mut previous = Vec::<u8>::new();
    for (index, form) in package.form_refs.iter().enumerate() {
        if index % DECODER_BLOCK_FORMS == 0 {
            offsets.push(
                u32::try_from(payload.len())
                    .map_err(|_| "compact L2 decoder payload exceeds u32".to_string())?,
            );
            previous.clear();
        }
        let surface = format::decoder_surface(&package.decoder_bytes, form.decoder_ref)?.as_bytes();
        let prefix = common_prefix_len(&previous, surface);
        let suffix = &surface[prefix..];
        put_var_u32(&mut payload, prefix)?;
        put_var_u32(&mut payload, suffix.len())?;
        payload.extend_from_slice(suffix);
        previous.clear();
        previous.extend_from_slice(surface);
    }
    Ok((offsets, payload))
}

fn decode_decoder(
    compact_forms: &[CompactFormRef],
    block_offsets: &[u32],
    payload: &[u8],
) -> Result<(Vec<FormCenterRef>, Vec<u8>), String> {
    let mut form_refs = Vec::with_capacity(compact_forms.len());
    let mut decoder_bytes = Vec::new();
    let mut previous = Vec::<u8>::new();
    let mut offset = 0_usize;
    for (index, compact) in compact_forms.iter().enumerate() {
        if index % DECODER_BLOCK_FORMS == 0 {
            let block = index / DECODER_BLOCK_FORMS;
            let recorded = block_offsets
                .get(block)
                .copied()
                .ok_or_else(|| "compact L2 decoder block offset is missing".to_string())?
                as usize;
            if recorded != offset {
                return Err(format!(
                    "compact L2 decoder block {block} starts at {recorded}, expected {offset}"
                ));
            }
            previous.clear();
        }
        let prefix = read_var_u32(payload, &mut offset)? as usize;
        let suffix_len = read_var_u32(payload, &mut offset)? as usize;
        if prefix > previous.len() {
            return Err(format!(
                "compact L2 decoder form {index} prefix exceeds previous surface"
            ));
        }
        if index % DECODER_BLOCK_FORMS == 0 && prefix != 0 {
            return Err(format!(
                "compact L2 decoder block {} does not start from a full surface",
                index / DECODER_BLOCK_FORMS
            ));
        }
        let end = offset
            .checked_add(suffix_len)
            .ok_or_else(|| "compact L2 decoder suffix offset overflow".to_string())?;
        let suffix = payload
            .get(offset..end)
            .ok_or_else(|| "compact L2 decoder suffix is truncated".to_string())?;
        offset = end;
        let mut surface = Vec::with_capacity(prefix + suffix_len);
        surface.extend_from_slice(&previous[..prefix]);
        surface.extend_from_slice(suffix);
        if surface.is_empty() {
            return Err(format!("compact L2 decoder form {index} is empty"));
        }
        std::str::from_utf8(&surface)
            .map_err(|error| format!("compact L2 decoder form {index} is not UTF-8: {error}"))?;
        let decoder_ref = u32::try_from(decoder_bytes.len())
            .map_err(|_| "decoded L2 decoder exceeds u32 address space".to_string())?;
        decoder_bytes.extend_from_slice(&surface);
        decoder_bytes.push(0);
        form_refs.push(FormCenterRef {
            l1_terminal_id: compact.l1_terminal_id,
            decoder_ref,
            script_flags: compact.script_flags,
            length_bucket: compact.length_bucket,
            flags: compact.flags,
            reserved: 0,
        });
        previous = surface;
    }
    if offset != payload.len() {
        return Err(format!(
            "compact L2 decoder has {} trailing payload bytes",
            payload.len() - offset
        ));
    }
    Ok((form_refs, decoder_bytes))
}

fn common_prefix_len(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right)
        .position(|(left, right)| left != right)
        .unwrap_or(left.len().min(right.len()))
}

fn put_var_u32(out: &mut Vec<u8>, value: usize) -> Result<(), String> {
    let mut value =
        u32::try_from(value).map_err(|_| "compact L2 decoder length exceeds u32".to_string())?;
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            return Ok(());
        }
    }
}

fn read_var_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let mut value = 0_u32;
    for shift in (0..35).step_by(7) {
        let byte = *bytes
            .get(*offset)
            .ok_or_else(|| "compact L2 decoder varint is truncated".to_string())?;
        *offset += 1;
        if shift == 28 && byte > 0x0f {
            return Err("compact L2 decoder varint overflows u32".to_string());
        }
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("compact L2 decoder varint is too long".to_string())
}

fn read_many<T>(
    count: usize,
    mut read: impl FnMut() -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read()?);
    }
    Ok(values)
}

fn read_many_indexed<T>(
    count: usize,
    mut read: impl FnMut(usize) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let mut values = Vec::with_capacity(count);
    for index in 0..count {
        values.push(read(index)?);
    }
    Ok(values)
}

fn read_header_counts(header: &mut Cursor<'_>, version: u32) -> Result<HeaderCounts, String> {
    let mut counts = HeaderCounts {
        decoder_block_forms: header.u32()? as usize,
        form_refs: header.u32()? as usize,
        decoder_blocks: header.u32()? as usize,
        decoder_payload_bytes: header.u32()? as usize,
        feature_masks: header.u32()? as usize,
        lemma_centers: header.u32()? as usize,
        morph_bindings: header.u32()? as usize,
        context_modes: header.u32()? as usize,
        slot_centers: header.u32()? as usize,
        neighbor_couplings: header.u32()? as usize,
        competition_edges: header.u32()? as usize,
        lemma_wave_ranges: 0,
        surface_wave_codes: 0,
        wave_band_offsets: 0,
        wave_band_postings: 0,
        atom_keys: 0,
        atom_offsets: 0,
        atom_postings: 0,
    };
    if version >= VERSION {
        counts.lemma_wave_ranges = header.u32()? as usize;
        counts.surface_wave_codes = header.u32()? as usize;
        counts.wave_band_offsets = header.u32()? as usize;
        counts.wave_band_postings = header.u32()? as usize;
        counts.atom_keys = header.u32()? as usize;
        counts.atom_offsets = header.u32()? as usize;
        counts.atom_postings = header.u32()? as usize;
    }
    Ok(counts)
}

#[derive(Clone, Copy)]
struct CompactFormRef {
    l1_terminal_id: u32,
    script_flags: u16,
    length_bucket: u8,
    flags: u8,
}

#[derive(Clone, Copy, Debug)]
struct HeaderCounts {
    decoder_block_forms: usize,
    form_refs: usize,
    decoder_blocks: usize,
    decoder_payload_bytes: usize,
    feature_masks: usize,
    lemma_centers: usize,
    morph_bindings: usize,
    context_modes: usize,
    slot_centers: usize,
    neighbor_couplings: usize,
    competition_edges: usize,
    lemma_wave_ranges: usize,
    surface_wave_codes: usize,
    wave_band_offsets: usize,
    wave_band_postings: usize,
    atom_keys: usize,
    atom_offsets: usize,
    atom_postings: usize,
}

#[derive(Clone, Debug)]
pub(super) struct CompactPackageView {
    bytes: Vec<u8>,
    version: u32,
    l1_package_fingerprint: u64,
    counts: HeaderCounts,
    form_start: usize,
    decoder_offset_start: usize,
    decoder_payload_start: usize,
    feature_masks: Vec<u32>,
    binding_start: usize,
    lemma_centers: Vec<LemmaCenter>,
    context_modes: Vec<LocalContextMode>,
    slot_centers: Vec<SlotPhaseCenter>,
    neighbor_couplings: Vec<NeighborCoupling>,
    competition_edges: Vec<CompetitionEdge>,
    calibration: TieCalibration,
    lemma_wave_index: Option<LemmaWaveIndex>,
    raw_decoder_bytes: usize,
}

impl CompactPackageView {
    pub(super) fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < HEADER_BYTES || !is_compact_package(&bytes) {
            return Err("invalid compact L2 package magic or truncated header".to_string());
        }
        let mut header = Cursor::new(&bytes[MAGIC.len()..HEADER_BYTES]);
        let version = header.u32()?;
        if !matches!(version, LEGACY_VERSION | VERSION) {
            return Err(format!("unsupported compact L2 package version {version}"));
        }
        let header_bytes = header.u32()? as usize;
        if header_bytes != HEADER_BYTES {
            return Err(format!("invalid compact L2 header size {header_bytes}"));
        }
        let total_bytes = usize::try_from(header.u64()?)
            .map_err(|_| "compact L2 package size does not fit usize".to_string())?;
        if total_bytes != bytes.len() {
            return Err(format!(
                "compact L2 package size mismatch: header={total_bytes} actual={}",
                bytes.len()
            ));
        }
        let expected_checksum = header.u64()?;
        let l1_package_fingerprint = header.u64()?;
        let counts = read_header_counts(&mut header, version)?;
        validate_header_counts(version, counts)?;
        if format::checksum64(&bytes[HEADER_BYTES..]) != expected_checksum {
            return Err("compact L2 package checksum mismatch".to_string());
        }

        let mut next = HEADER_BYTES;
        let form_start = take_section(
            &mut next,
            counts.form_refs,
            COMPACT_FORM_REF_BYTES,
            bytes.len(),
            "form refs",
        )?
        .start;
        let decoder_offset_start = take_section(
            &mut next,
            counts.decoder_blocks,
            std::mem::size_of::<u32>(),
            bytes.len(),
            "decoder offsets",
        )?
        .start;
        let decoder_payload_start = take_section(
            &mut next,
            counts.decoder_payload_bytes,
            1,
            bytes.len(),
            "decoder payload",
        )?
        .start;
        let feature_range = take_section(
            &mut next,
            counts.feature_masks,
            std::mem::size_of::<u32>(),
            bytes.len(),
            "feature dictionary",
        )?;
        let lemma_range = take_section(
            &mut next,
            counts.lemma_centers,
            LEMMA_CENTER_BYTES,
            bytes.len(),
            "lemma centers",
        )?;
        let binding_start = take_section(
            &mut next,
            counts.morph_bindings,
            COMPACT_BINDING_BYTES,
            bytes.len(),
            "morph bindings",
        )?
        .start;
        let context_range = take_section(
            &mut next,
            counts.context_modes,
            CONTEXT_MODE_BYTES,
            bytes.len(),
            "context modes",
        )?;
        let slot_range = take_section(
            &mut next,
            counts.slot_centers,
            SLOT_CENTER_BYTES,
            bytes.len(),
            "slot centers",
        )?;
        let neighbor_range = take_section(
            &mut next,
            counts.neighbor_couplings,
            NEIGHBOR_COUPLING_BYTES,
            bytes.len(),
            "neighbor couplings",
        )?;
        let competition_range = take_section(
            &mut next,
            counts.competition_edges,
            COMPETITION_EDGE_BYTES,
            bytes.len(),
            "competition edges",
        )?;
        let calibration_range =
            take_section(&mut next, 1, CALIBRATION_BYTES, bytes.len(), "calibration")?;
        let lemma_wave_range = take_section(
            &mut next,
            counts.lemma_wave_ranges,
            compositional_format::LEMMA_WAVE_RANGE_BYTES,
            bytes.len(),
            "lemma wave ranges",
        )?;
        let surface_wave_range = take_section(
            &mut next,
            counts.surface_wave_codes,
            compositional_format::SURFACE_WAVE_CODE_BYTES,
            bytes.len(),
            "surface wave codes",
        )?;
        let wave_band_offset_range = take_section(
            &mut next,
            counts.wave_band_offsets,
            compositional_format::WAVE_BAND_OFFSET_BYTES,
            bytes.len(),
            "wave band offsets",
        )?;
        let wave_band_posting_range = take_section(
            &mut next,
            counts.wave_band_postings,
            compositional_format::WAVE_BAND_POSTING_BYTES,
            bytes.len(),
            "wave band postings",
        )?;
        let atom_key_range = take_section(
            &mut next,
            counts.atom_keys,
            compositional_format::ATOM_KEY_BYTES,
            bytes.len(),
            "typed atom keys",
        )?;
        let atom_offset_range = take_section(
            &mut next,
            counts.atom_offsets,
            compositional_format::ATOM_OFFSET_BYTES,
            bytes.len(),
            "typed atom offsets",
        )?;
        let atom_posting_range = take_section(
            &mut next,
            counts.atom_postings,
            compositional_format::ATOM_POSTING_BYTES,
            bytes.len(),
            "typed atom postings",
        )?;
        if next != bytes.len() {
            return Err(format!(
                "compact L2 package has {} trailing bytes",
                bytes.len().saturating_sub(next)
            ));
        }

        let feature_masks = read_section(&bytes[feature_range], counts.feature_masks, |cursor| {
            cursor.u32()
        })?;
        if feature_masks.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("compact L2 feature dictionary is not strictly ordered".to_string());
        }
        let lemma_centers = read_section(&bytes[lemma_range], counts.lemma_centers, |cursor| {
            format::read_lemma_center(cursor)
        })?;
        validate_lemma_binding_ranges(&lemma_centers, counts.morph_bindings, None)?;
        let context_modes = read_section(&bytes[context_range], counts.context_modes, |cursor| {
            format::read_context_mode(cursor)
        })?;
        let slot_centers = read_section(&bytes[slot_range], counts.slot_centers, |cursor| {
            format::read_slot_center(cursor)
        })?;
        let neighbor_couplings = read_section(
            &bytes[neighbor_range],
            counts.neighbor_couplings,
            |cursor| format::read_neighbor_coupling(cursor),
        )?;
        let competition_edges = read_section(
            &bytes[competition_range],
            counts.competition_edges,
            |cursor| format::read_competition_edge(cursor),
        )?;
        let mut calibration_cursor = Cursor::new(&bytes[calibration_range]);
        let calibration = format::read_calibration(&mut calibration_cursor)?;
        let lemma_wave_index = (counts.lemma_wave_ranges != 0 || counts.surface_wave_codes != 0)
            .then(|| {
                compositional_format::decode(
                    &bytes[lemma_wave_range],
                    counts.lemma_wave_ranges,
                    &bytes[surface_wave_range],
                    counts.surface_wave_codes,
                    &bytes[wave_band_offset_range],
                    counts.wave_band_offsets,
                    &bytes[wave_band_posting_range],
                    counts.wave_band_postings,
                    &bytes[atom_key_range],
                    counts.atom_keys,
                    &bytes[atom_offset_range],
                    counts.atom_offsets,
                    &bytes[atom_posting_range],
                    counts.atom_postings,
                )
            })
            .transpose()?;
        validate_embedded_wave_index(version, counts.lemma_centers, lemma_wave_index.as_ref())?;

        let mut view = Self {
            bytes,
            version,
            l1_package_fingerprint,
            counts,
            form_start,
            decoder_offset_start,
            decoder_payload_start,
            feature_masks,
            binding_start,
            lemma_centers,
            context_modes,
            slot_centers,
            neighbor_couplings,
            competition_edges,
            calibration,
            lemma_wave_index,
            raw_decoder_bytes: 0,
        };
        view.raw_decoder_bytes = view.validate_decoder()?;
        view.validate_bindings_and_edges()?;
        Ok(view)
    }

    pub(super) fn backing_bytes(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn storage_kind(&self) -> &'static str {
        match self.version {
            LEGACY_VERSION => "compact_v1_direct",
            VERSION => "compact_v2_compositional",
            _ => "compact_unknown",
        }
    }

    pub(super) fn take_lemma_wave_index(&mut self) -> Option<LemmaWaveIndex> {
        self.lemma_wave_index.take()
    }

    pub(super) fn l1_package_fingerprint(&self) -> u64 {
        self.l1_package_fingerprint
    }

    pub(super) fn form_count(&self) -> usize {
        self.counts.form_refs
    }

    pub(super) fn binding_count(&self) -> usize {
        self.counts.morph_bindings
    }

    pub(super) fn raw_decoder_bytes(&self) -> usize {
        self.raw_decoder_bytes
    }

    pub(super) fn form(&self, index: usize) -> Option<FormCenterRef> {
        let start = self
            .form_start
            .checked_add(index.checked_mul(COMPACT_FORM_REF_BYTES)?)?;
        let mut cursor = Cursor::new(self.bytes.get(start..start + COMPACT_FORM_REF_BYTES)?);
        Some(FormCenterRef {
            l1_terminal_id: cursor.u32().ok()?,
            decoder_ref: 0,
            script_flags: cursor.u16().ok()?,
            length_bucket: cursor.u8().ok()?,
            flags: cursor.u8().ok()?,
            reserved: 0,
        })
    }

    pub(super) fn surface(&self, form_ref: usize) -> Option<String> {
        if form_ref >= self.counts.form_refs {
            return None;
        }
        let block = form_ref / DECODER_BLOCK_FORMS;
        let block_first = block * DECODER_BLOCK_FORMS;
        let payload = self.decoder_payload();
        let mut offset = self.decoder_block_offset(block)? as usize;
        let mut previous = Vec::<u8>::new();
        for index in block_first..=form_ref {
            let prefix = read_var_u32(payload, &mut offset).ok()? as usize;
            let suffix_len = read_var_u32(payload, &mut offset).ok()? as usize;
            if prefix > previous.len() || (index == block_first && prefix != 0) {
                return None;
            }
            let end = offset.checked_add(suffix_len)?;
            let suffix = payload.get(offset..end)?;
            offset = end;
            previous.truncate(prefix);
            previous.extend_from_slice(suffix);
        }
        String::from_utf8(previous).ok()
    }

    pub(super) fn binding(&self, index: usize) -> Option<MorphBinding> {
        if index >= self.counts.morph_bindings {
            return None;
        }
        let lemma_id = self.lemma_centers.partition_point(|center| {
            center.form_start as usize + center.form_count as usize <= index
        });
        let center = self.lemma_centers.get(lemma_id)?;
        if index < center.form_start as usize {
            return None;
        }
        let start = self
            .binding_start
            .checked_add(index.checked_mul(COMPACT_BINDING_BYTES)?)?;
        let mut cursor = Cursor::new(self.bytes.get(start..start + COMPACT_BINDING_BYTES)?);
        let form_center_ref = cursor.u32().ok()?;
        let feature_id = cursor.u8().ok()? as usize;
        Some(MorphBinding {
            form_center_ref,
            lemma_center_id: lemma_id as u32,
            feature_mask: *self.feature_masks.get(feature_id)?,
            support: cursor.u16().ok()?,
            phase: cursor.i8().ok()?,
            flags: cursor.u8().ok()?,
        })
    }

    pub(super) fn lemma_centers(&self) -> &[LemmaCenter] {
        &self.lemma_centers
    }

    pub(super) fn context_modes(&self) -> &[LocalContextMode] {
        &self.context_modes
    }

    pub(super) fn slot_centers(&self) -> &[SlotPhaseCenter] {
        &self.slot_centers
    }

    pub(super) fn neighbor_couplings(&self) -> &[NeighborCoupling] {
        &self.neighbor_couplings
    }

    pub(super) fn competition_edges(&self) -> &[CompetitionEdge] {
        &self.competition_edges
    }

    pub(super) fn calibration(&self) -> TieCalibration {
        self.calibration
    }

    fn decoder_block_offset(&self, block: usize) -> Option<u32> {
        let start = self
            .decoder_offset_start
            .checked_add(block.checked_mul(std::mem::size_of::<u32>())?)?;
        Some(u32::from_le_bytes(
            self.bytes.get(start..start + 4)?.try_into().ok()?,
        ))
    }

    fn decoder_payload(&self) -> &[u8] {
        &self.bytes[self.decoder_payload_start
            ..self.decoder_payload_start + self.counts.decoder_payload_bytes]
    }

    fn validate_decoder(&self) -> Result<usize, String> {
        let payload = self.decoder_payload();
        let mut offset = 0_usize;
        let mut previous_in_block = Vec::<u8>::new();
        let mut previous_surface = Vec::<u8>::new();
        let mut raw_bytes = 0_usize;
        for index in 0..self.counts.form_refs {
            if index % DECODER_BLOCK_FORMS == 0 {
                let block = index / DECODER_BLOCK_FORMS;
                let recorded = self
                    .decoder_block_offset(block)
                    .ok_or_else(|| "compact L2 decoder block offset is missing".to_string())?
                    as usize;
                if recorded != offset {
                    return Err(format!(
                        "compact L2 decoder block {block} starts at {recorded}, expected {offset}"
                    ));
                }
                previous_in_block.clear();
            }
            let prefix = read_var_u32(payload, &mut offset)? as usize;
            let suffix_len = read_var_u32(payload, &mut offset)? as usize;
            if prefix > previous_in_block.len() {
                return Err(format!(
                    "compact L2 decoder form {index} prefix exceeds previous surface"
                ));
            }
            if index % DECODER_BLOCK_FORMS == 0 && prefix != 0 {
                return Err(format!(
                    "compact L2 decoder block {} does not start from a full surface",
                    index / DECODER_BLOCK_FORMS
                ));
            }
            let end = offset
                .checked_add(suffix_len)
                .ok_or_else(|| "compact L2 decoder suffix offset overflow".to_string())?;
            let suffix = payload
                .get(offset..end)
                .ok_or_else(|| "compact L2 decoder suffix is truncated".to_string())?;
            offset = end;
            previous_in_block.truncate(prefix);
            previous_in_block.extend_from_slice(suffix);
            if previous_in_block.is_empty() {
                return Err(format!("compact L2 decoder form {index} is empty"));
            }
            std::str::from_utf8(&previous_in_block).map_err(|error| {
                format!("compact L2 decoder form {index} is not UTF-8: {error}")
            })?;
            if !previous_surface.is_empty() && previous_surface >= previous_in_block {
                return Err(format!(
                    "compact L2 decoder surfaces are not strictly ordered at form {index}"
                ));
            }
            raw_bytes = raw_bytes
                .checked_add(previous_in_block.len() + 1)
                .ok_or_else(|| "compact L2 raw decoder size overflow".to_string())?;
            previous_surface.clear();
            previous_surface.extend_from_slice(&previous_in_block);
        }
        if offset != payload.len() {
            return Err(format!(
                "compact L2 decoder has {} trailing payload bytes",
                payload.len() - offset
            ));
        }
        Ok(raw_bytes)
    }

    fn validate_bindings_and_edges(&self) -> Result<(), String> {
        for index in 0..self.counts.morph_bindings {
            let binding = self
                .binding(index)
                .ok_or_else(|| format!("compact L2 binding {index} is invalid"))?;
            if binding.form_center_ref as usize >= self.counts.form_refs {
                return Err(format!(
                    "compact L2 binding {index} references missing form"
                ));
            }
        }
        for (index, edge) in self.competition_edges.iter().enumerate() {
            if edge.left_form_ref as usize >= self.counts.form_refs
                || edge.right_form_ref as usize >= self.counts.form_refs
            {
                return Err(format!(
                    "compact L2 competition edge {index} references missing form"
                ));
            }
            if edge.context_mode_id != u32::MAX
                && edge.context_mode_id as usize >= self.context_modes.len()
            {
                return Err(format!(
                    "compact L2 competition edge {index} references missing context mode"
                ));
            }
        }
        Ok(())
    }
}

fn validate_header_counts(version: u32, counts: HeaderCounts) -> Result<(), String> {
    if counts.decoder_block_forms != DECODER_BLOCK_FORMS {
        return Err(format!(
            "unsupported compact L2 decoder block size {}",
            counts.decoder_block_forms
        ));
    }
    if counts.feature_masks > usize::from(u8::MAX) + 1 {
        return Err("compact L2 feature dictionary exceeds 256 entries".to_string());
    }
    let expected_blocks = counts.form_refs.div_ceil(DECODER_BLOCK_FORMS);
    if counts.decoder_blocks != expected_blocks {
        return Err(format!(
            "compact L2 decoder block count mismatch: header={} expected={expected_blocks}",
            counts.decoder_blocks
        ));
    }
    if version == LEGACY_VERSION
        && (counts.lemma_wave_ranges != 0
            || counts.surface_wave_codes != 0
            || counts.wave_band_offsets != 0
            || counts.wave_band_postings != 0
            || counts.atom_keys != 0
            || counts.atom_offsets != 0
            || counts.atom_postings != 0)
    {
        return Err("compact L2 V1 cannot contain lemma wave sections".to_string());
    }
    Ok(())
}

fn validate_embedded_wave_index(
    version: u32,
    lemma_count: usize,
    index: Option<&LemmaWaveIndex>,
) -> Result<(), String> {
    match (version, index) {
        (LEGACY_VERSION, None) => Ok(()),
        (VERSION, Some(index)) if index.ranges().len() == lemma_count => Ok(()),
        (VERSION, Some(index)) => Err(format!(
            "compact L2 lemma wave range count {} does not match lemma count {lemma_count}",
            index.ranges().len()
        )),
        (VERSION, None) => Err(format!(
            "compact L2 V{version} is missing lemma wave sections"
        )),
        (LEGACY_VERSION, Some(_)) => {
            Err("compact L2 V1 unexpectedly contains lemma wave sections".to_string())
        }
        _ => Err(format!("unsupported compact L2 package version {version}")),
    }
}

fn take_section(
    next: &mut usize,
    count: usize,
    width: usize,
    total: usize,
    name: &str,
) -> Result<std::ops::Range<usize>, String> {
    let bytes = count
        .checked_mul(width)
        .ok_or_else(|| format!("compact L2 {name} size overflow"))?;
    let end = next
        .checked_add(bytes)
        .ok_or_else(|| format!("compact L2 {name} offset overflow"))?;
    if end > total {
        return Err(format!("compact L2 {name} section is truncated"));
    }
    let range = *next..end;
    *next = end;
    Ok(range)
}

fn read_section<T>(
    bytes: &[u8],
    count: usize,
    mut read: impl FnMut(&mut Cursor<'_>) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let mut cursor = Cursor::new(bytes);
    let values = read_many(count, || read(&mut cursor))?;
    if cursor.remaining() != 0 {
        return Err(format!(
            "compact L2 fixed section has {} trailing bytes",
            cursor.remaining()
        ));
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::super::model::{
        CompetitionEdge, LemmaCenter, LocalContextMode, NeighborCoupling, SlotPhaseCenter,
        TieCalibration, L2_PHASE_CELLS,
    };
    use super::*;

    fn fixture() -> L2FieldPackage {
        let surfaces = ["дом", "дома", "замки", "замок"];
        let mut decoder_bytes = Vec::new();
        let form_refs = surfaces
            .iter()
            .enumerate()
            .map(|(index, surface)| {
                let decoder_ref = decoder_bytes.len() as u32;
                decoder_bytes.extend_from_slice(surface.as_bytes());
                decoder_bytes.push(0);
                FormCenterRef {
                    l1_terminal_id: index as u32 + 7,
                    decoder_ref,
                    script_flags: 1,
                    length_bucket: surface.chars().count() as u8,
                    flags: index as u8,
                    reserved: 0,
                }
            })
            .collect();
        L2FieldPackage {
            l1_package_fingerprint: 0x1234,
            form_refs,
            decoder_bytes,
            lemma_centers: vec![
                LemmaCenter {
                    form_start: 0,
                    form_count: 2,
                    ..LemmaCenter::default()
                },
                LemmaCenter {
                    form_start: 2,
                    form_count: 2,
                    ..LemmaCenter::default()
                },
                LemmaCenter {
                    form_start: 4,
                    form_count: 1,
                    ..LemmaCenter::default()
                },
            ],
            morph_bindings: vec![
                MorphBinding {
                    form_center_ref: 0,
                    lemma_center_id: 0,
                    feature_mask: 10,
                    support: 3,
                    phase: 1,
                    flags: 2,
                },
                MorphBinding {
                    form_center_ref: 1,
                    lemma_center_id: 0,
                    feature_mask: 10,
                    support: 4,
                    phase: -1,
                    flags: 3,
                },
                MorphBinding {
                    form_center_ref: 2,
                    lemma_center_id: 1,
                    feature_mask: 20,
                    support: 5,
                    phase: 1,
                    flags: 4,
                },
                MorphBinding {
                    form_center_ref: 3,
                    lemma_center_id: 1,
                    feature_mask: 21,
                    support: 6,
                    phase: 0,
                    flags: 5,
                },
                MorphBinding {
                    form_center_ref: 2,
                    lemma_center_id: 2,
                    feature_mask: 30,
                    support: u16::MAX,
                    phase: i8::MIN,
                    flags: u8::MAX,
                },
            ],
            context_modes: vec![LocalContextMode {
                stable_key: 42,
                ..LocalContextMode::default()
            }],
            slot_centers: vec![SlotPhaseCenter {
                cells: [1; L2_PHASE_CELLS],
                feature_mask: 20,
                context_mode_id: 0,
                support: 3,
                mass: 60,
                polarity: 1,
                flags: 0,
                reserved: 0,
            }],
            neighbor_couplings: vec![NeighborCoupling {
                target_lemma_id: 1,
                ..NeighborCoupling::default()
            }],
            competition_edges: vec![CompetitionEdge {
                left_form_ref: 2,
                right_form_ref: 3,
                context_mode_id: 0,
                ..CompetitionEdge::default()
            }],
            calibration: TieCalibration::default(),
        }
    }

    #[test]
    fn compact_format_roundtrips_variant_slots_multi_lemma_and_utf8_exactly() {
        let package = fixture();
        let (first, stats) = encode_package(&package).expect("compact encode");
        let (second, _) = encode_package(&package).expect("deterministic compact encode");

        assert_eq!(first, second);
        assert_eq!(decode_package(&first), Ok(package));
        assert_eq!(stats.version, VERSION);
        assert_eq!(stats.form_ref_bytes, 4 * COMPACT_FORM_REF_BYTES);
        assert_eq!(stats.morph_binding_bytes, 5 * COMPACT_BINDING_BYTES);
        assert_eq!(stats.feature_dictionary_entries, 4);
        assert_eq!(stats.lemma_wave_ranges, 3);
        assert!(stats.surface_wave_codes >= stats.lemma_wave_ranges);
        assert!(stats.atom_keys > 0);
        assert_eq!(stats.atom_offsets, stats.atom_keys + 1);
        assert!(stats.atom_postings > 0);

        let mut direct = CompactPackageView::from_bytes(first).expect("compact direct view");
        assert_eq!(direct.storage_kind(), "compact_v2_compositional");
        assert_eq!(
            direct
                .take_lemma_wave_index()
                .expect("embedded lemma wave index")
                .atom_keys()
                .len(),
            stats.atom_keys
        );
    }

    #[test]
    fn compact_runtime_remains_backward_compatible_with_v1() {
        let package = fixture();
        let (bytes, stats) =
            encode_package_version(&package, LEGACY_VERSION).expect("legacy compact encode");

        assert_eq!(stats.version, LEGACY_VERSION);
        assert_eq!(stats.lemma_wave_ranges, 0);
        assert_eq!(stats.atom_keys, 0);
        assert_eq!(stats.atom_postings, 0);
        assert_eq!(decode_package(&bytes), Ok(package));
        let mut direct = CompactPackageView::from_bytes(bytes).expect("legacy direct view");
        assert_eq!(direct.storage_kind(), "compact_v1_direct");
        assert_eq!(direct.take_lemma_wave_index(), None);
    }

    #[test]
    fn compact_format_rejects_corruption() {
        let (mut bytes, _) = encode_package(&fixture()).expect("compact encode");
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert_eq!(
            decode_package(&bytes),
            Err("compact L2 package checksum mismatch".to_string())
        );
    }
}
