use super::model::{
    CompetitionEdge, FormCenterRef, L2FieldPackage, LemmaCenter, LocalContextMode, MorphBinding,
    NeighborCoupling, SlotPhaseCenter, TieCalibration, L2_PHASE_CELLS,
};

const MAGIC: &[u8; 8] = b"LAYL2F01";
const VERSION: u32 = 2;
const HEADER_BYTES: usize = 128;

pub(crate) fn encode_package(package: &L2FieldPackage) -> Result<Vec<u8>, String> {
    validate_package(package)?;
    let mut body = Vec::new();
    for value in &package.form_refs {
        put_form_ref(&mut body, *value);
    }
    body.extend_from_slice(&package.decoder_bytes);
    for value in &package.lemma_centers {
        put_lemma_center(&mut body, *value);
    }
    for value in &package.morph_bindings {
        put_morph_binding(&mut body, *value);
    }
    for value in &package.context_modes {
        put_context_mode(&mut body, *value);
    }
    for value in &package.slot_centers {
        put_slot_center(&mut body, *value);
    }
    for value in &package.neighbor_couplings {
        put_neighbor_coupling(&mut body, *value);
    }
    for value in &package.competition_edges {
        put_competition_edge(&mut body, *value);
    }
    put_calibration(&mut body, package.calibration);

    let total_bytes = HEADER_BYTES
        .checked_add(body.len())
        .ok_or_else(|| "L2 package size overflow".to_string())?;
    let mut bytes = Vec::with_capacity(total_bytes);
    bytes.extend_from_slice(MAGIC);
    put_u32(&mut bytes, VERSION);
    put_u32(&mut bytes, HEADER_BYTES as u32);
    put_u64(&mut bytes, total_bytes as u64);
    put_u64(&mut bytes, checksum64(&body));
    put_u64(&mut bytes, package.l1_package_fingerprint);
    for count in [
        package.form_refs.len(),
        package.decoder_bytes.len(),
        package.lemma_centers.len(),
        package.morph_bindings.len(),
        package.context_modes.len(),
        package.slot_centers.len(),
        package.neighbor_couplings.len(),
        package.competition_edges.len(),
    ] {
        put_u32(
            &mut bytes,
            u32::try_from(count).map_err(|_| "L2 section count exceeds u32".to_string())?,
        );
    }
    bytes.resize(HEADER_BYTES, 0);
    bytes.extend_from_slice(&body);
    Ok(bytes)
}

pub(crate) fn decode_package(bytes: &[u8]) -> Result<L2FieldPackage, String> {
    if bytes.len() < HEADER_BYTES || bytes.get(..8) != Some(MAGIC) {
        return Err("invalid L2 package magic or truncated header".to_string());
    }
    let mut header = Cursor::new(&bytes[8..HEADER_BYTES]);
    let version = header.u32()?;
    if version != VERSION {
        return Err(format!("unsupported L2 package version {version}"));
    }
    let header_bytes = header.u32()? as usize;
    if header_bytes != HEADER_BYTES {
        return Err(format!("invalid L2 header size {header_bytes}"));
    }
    let total_bytes = usize::try_from(header.u64()?)
        .map_err(|_| "L2 package size does not fit usize".to_string())?;
    if total_bytes != bytes.len() {
        return Err(format!(
            "L2 package size mismatch: header={total_bytes} actual={}",
            bytes.len()
        ));
    }
    let expected_checksum = header.u64()?;
    let l1_package_fingerprint = header.u64()?;
    let counts = [
        header.u32()? as usize,
        header.u32()? as usize,
        header.u32()? as usize,
        header.u32()? as usize,
        header.u32()? as usize,
        header.u32()? as usize,
        header.u32()? as usize,
        header.u32()? as usize,
    ];
    let body = &bytes[HEADER_BYTES..];
    if checksum64(body) != expected_checksum {
        return Err("L2 package checksum mismatch".to_string());
    }
    let mut cursor = Cursor::new(body);
    let package = L2FieldPackage {
        l1_package_fingerprint,
        form_refs: read_many(counts[0], || read_form_ref(&mut cursor))?,
        decoder_bytes: cursor.bytes(counts[1])?.to_vec(),
        lemma_centers: read_many(counts[2], || read_lemma_center(&mut cursor))?,
        morph_bindings: read_many(counts[3], || read_morph_binding(&mut cursor))?,
        context_modes: read_many(counts[4], || read_context_mode(&mut cursor))?,
        slot_centers: read_many(counts[5], || read_slot_center(&mut cursor))?,
        neighbor_couplings: read_many(counts[6], || read_neighbor_coupling(&mut cursor))?,
        competition_edges: read_many(counts[7], || read_competition_edge(&mut cursor))?,
        calibration: read_calibration(&mut cursor)?,
    };
    if cursor.remaining() != 0 {
        return Err(format!(
            "L2 package has {} trailing bytes",
            cursor.remaining()
        ));
    }
    validate_package(&package)?;
    Ok(package)
}

fn validate_package(package: &L2FieldPackage) -> Result<(), String> {
    let mut previous_surface = None;
    for (index, form) in package.form_refs.iter().enumerate() {
        let surface = decoder_surface(&package.decoder_bytes, form.decoder_ref)
            .map_err(|error| format!("form {index} {error}"))?;
        if previous_surface.is_some_and(|previous| previous >= surface) {
            return Err(format!(
                "form {index} decoder surfaces are not strictly ordered"
            ));
        }
        previous_surface = Some(surface);
    }
    for (index, binding) in package.morph_bindings.iter().enumerate() {
        if binding.form_center_ref as usize >= package.form_refs.len() {
            return Err(format!("binding {index} references missing form"));
        }
        if binding.lemma_center_id as usize >= package.lemma_centers.len() {
            return Err(format!("binding {index} references missing lemma"));
        }
    }
    for (index, edge) in package.competition_edges.iter().enumerate() {
        if edge.left_form_ref as usize >= package.form_refs.len()
            || edge.right_form_ref as usize >= package.form_refs.len()
        {
            return Err(format!("competition edge {index} references missing form"));
        }
        if edge.context_mode_id != u32::MAX
            && edge.context_mode_id as usize >= package.context_modes.len()
        {
            return Err(format!(
                "competition edge {index} references missing context mode"
            ));
        }
    }
    Ok(())
}

fn decoder_surface(decoder: &[u8], decoder_ref: u32) -> Result<&str, String> {
    let start = decoder_ref as usize;
    let tail = decoder
        .get(start..)
        .ok_or_else(|| "decoder reference is out of range".to_string())?;
    let length = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| "decoder surface is not terminated".to_string())?;
    if length == 0 {
        return Err("decoder surface is empty".to_string());
    }
    std::str::from_utf8(&tail[..length])
        .map_err(|error| format!("decoder surface is not UTF-8: {error}"))
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

fn put_form_ref(out: &mut Vec<u8>, value: FormCenterRef) {
    put_u32(out, value.l1_terminal_id);
    put_u32(out, value.decoder_ref);
    put_u16(out, value.script_flags);
    out.push(value.length_bucket);
    out.push(value.flags);
    put_u32(out, value.reserved);
}

fn read_form_ref(cursor: &mut Cursor<'_>) -> Result<FormCenterRef, String> {
    Ok(FormCenterRef {
        l1_terminal_id: cursor.u32()?,
        decoder_ref: cursor.u32()?,
        script_flags: cursor.u16()?,
        length_bucket: cursor.u8()?,
        flags: cursor.u8()?,
        reserved: cursor.u32()?,
    })
}

fn put_lemma_center(out: &mut Vec<u8>, value: LemmaCenter) {
    put_u16(out, value.primary_pos);
    put_u16(out, value.flags);
    put_u32(out, value.form_start);
    put_u32(out, value.form_count);
    put_u32(out, value.local_context_start);
    put_u32(out, value.local_context_count);
    put_u32(out, value.competition_start);
    put_u32(out, value.competition_count);
    put_u32(out, value.reserved);
}

fn read_lemma_center(cursor: &mut Cursor<'_>) -> Result<LemmaCenter, String> {
    Ok(LemmaCenter {
        primary_pos: cursor.u16()?,
        flags: cursor.u16()?,
        form_start: cursor.u32()?,
        form_count: cursor.u32()?,
        local_context_start: cursor.u32()?,
        local_context_count: cursor.u32()?,
        competition_start: cursor.u32()?,
        competition_count: cursor.u32()?,
        reserved: cursor.u32()?,
    })
}

fn put_morph_binding(out: &mut Vec<u8>, value: MorphBinding) {
    put_u32(out, value.form_center_ref);
    put_u32(out, value.lemma_center_id);
    put_u32(out, value.feature_mask);
    put_u16(out, value.support);
    out.push(value.phase as u8);
    out.push(value.flags);
}

fn read_morph_binding(cursor: &mut Cursor<'_>) -> Result<MorphBinding, String> {
    Ok(MorphBinding {
        form_center_ref: cursor.u32()?,
        lemma_center_id: cursor.u32()?,
        feature_mask: cursor.u32()?,
        support: cursor.u16()?,
        phase: cursor.i8()?,
        flags: cursor.u8()?,
    })
}

fn put_context_mode(out: &mut Vec<u8>, value: LocalContextMode) {
    put_u16(out, value.left_class);
    put_u16(out, value.right_class);
    out.push(value.punctuation_class);
    out.push(value.adjacency_mode as u8);
    out.push(value.position_mode);
    out.push(value.flags);
    put_u32(out, value.lexical_anchor);
    put_u32(out, value.stable_key);
}

fn read_context_mode(cursor: &mut Cursor<'_>) -> Result<LocalContextMode, String> {
    Ok(LocalContextMode {
        left_class: cursor.u16()?,
        right_class: cursor.u16()?,
        punctuation_class: cursor.u8()?,
        adjacency_mode: cursor.i8()?,
        position_mode: cursor.u8()?,
        flags: cursor.u8()?,
        lexical_anchor: cursor.u32()?,
        stable_key: cursor.u32()?,
    })
}

fn put_slot_center(out: &mut Vec<u8>, value: SlotPhaseCenter) {
    out.extend(value.cells.iter().map(|value| *value as u8));
    put_u32(out, value.feature_mask);
    put_u32(out, value.context_mode_id);
    put_u16(out, value.support);
    put_u16(out, value.mass);
    out.push(value.polarity as u8);
    out.push(value.flags);
    put_u16(out, value.reserved);
}

fn read_slot_center(cursor: &mut Cursor<'_>) -> Result<SlotPhaseCenter, String> {
    let mut cells = [0_i8; L2_PHASE_CELLS];
    for cell in &mut cells {
        *cell = cursor.i8()?;
    }
    Ok(SlotPhaseCenter {
        cells,
        feature_mask: cursor.u32()?,
        context_mode_id: cursor.u32()?,
        support: cursor.u16()?,
        mass: cursor.u16()?,
        polarity: cursor.i8()?,
        flags: cursor.u8()?,
        reserved: cursor.u16()?,
    })
}

fn put_neighbor_coupling(out: &mut Vec<u8>, value: NeighborCoupling) {
    put_u32(out, value.context_mode_id);
    put_u32(out, value.target_lemma_id);
    put_u32(out, value.target_feature_mask);
    put_i16(out, value.support);
    put_i16(out, value.repel);
    put_u32(out, value.source_anchor);
    put_u16(out, value.flags);
    put_u16(out, value.reserved);
}

fn read_neighbor_coupling(cursor: &mut Cursor<'_>) -> Result<NeighborCoupling, String> {
    Ok(NeighborCoupling {
        context_mode_id: cursor.u32()?,
        target_lemma_id: cursor.u32()?,
        target_feature_mask: cursor.u32()?,
        support: cursor.i16()?,
        repel: cursor.i16()?,
        source_anchor: cursor.u32()?,
        flags: cursor.u16()?,
        reserved: cursor.u16()?,
    })
}

fn put_competition_edge(out: &mut Vec<u8>, value: CompetitionEdge) {
    put_u32(out, value.left_form_ref);
    put_u32(out, value.right_form_ref);
    put_u32(out, value.context_mode_id);
    put_i16(out, value.support_delta);
    put_i16(out, value.anti_delta);
    put_u32(out, value.evidence);
    put_u16(out, value.flags);
    put_u16(out, value.reserved);
}

fn read_competition_edge(cursor: &mut Cursor<'_>) -> Result<CompetitionEdge, String> {
    Ok(CompetitionEdge {
        left_form_ref: cursor.u32()?,
        right_form_ref: cursor.u32()?,
        context_mode_id: cursor.u32()?,
        support_delta: cursor.i16()?,
        anti_delta: cursor.i16()?,
        evidence: cursor.u32()?,
        flags: cursor.u16()?,
        reserved: cursor.u16()?,
    })
}

fn put_calibration(out: &mut Vec<u8>, value: TieCalibration) {
    put_i32(out, value.minimum_positive);
    put_i32(out, value.minimum_margin);
    put_i32(out, value.tie_window);
    put_i32(out, value.abstain_window);
    put_u16(out, value.false_authority_ceiling_milli);
    put_u16(out, value.flags);
    put_u32(out, value.evidence_count);
}

fn read_calibration(cursor: &mut Cursor<'_>) -> Result<TieCalibration, String> {
    Ok(TieCalibration {
        minimum_positive: cursor.i32()?,
        minimum_margin: cursor.i32()?,
        tie_window: cursor.i32()?,
        abstain_window: cursor.i32()?,
        false_authority_ceiling_milli: cursor.u16()?,
        flags: cursor.u16()?,
        evidence_count: cursor.u32()?,
    })
}

fn checksum64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |state, byte| {
        state.wrapping_mul(0x100000001b3) ^ u64::from(*byte)
    })
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i16(out: &mut Vec<u8>, value: i16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or_else(|| "L2 package offset overflow".to_string())?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "truncated L2 package section".to_string())?;
        self.offset = end;
        slice
            .try_into()
            .map_err(|_| "invalid L2 package field width".to_string())
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take::<1>()?[0])
    }

    fn i8(&mut self) -> Result<i8, String> {
        Ok(self.u8()? as i8)
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take()?))
    }

    fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.take()?))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take()?))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take()?))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take()?))
    }

    fn bytes(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| "L2 package offset overflow".to_string())?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| "truncated L2 package section".to_string())?;
        self.offset = end;
        Ok(bytes)
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> L2FieldPackage {
        L2FieldPackage {
            l1_package_fingerprint: 0x1234,
            form_refs: vec![
                FormCenterRef {
                    l1_terminal_id: 7,
                    decoder_ref: 0,
                    length_bucket: 4,
                    ..FormCenterRef::default()
                },
                FormCenterRef {
                    l1_terminal_id: 9,
                    decoder_ref: 11,
                    length_bucket: 5,
                    ..FormCenterRef::default()
                },
            ],
            decoder_bytes: "альфа\0бета\0".as_bytes().to_vec(),
            lemma_centers: vec![LemmaCenter {
                form_count: 2,
                ..LemmaCenter::default()
            }],
            morph_bindings: vec![
                MorphBinding {
                    form_center_ref: 0,
                    lemma_center_id: 0,
                    feature_mask: 1,
                    ..MorphBinding::default()
                },
                MorphBinding {
                    form_center_ref: 1,
                    lemma_center_id: 0,
                    feature_mask: 2,
                    ..MorphBinding::default()
                },
            ],
            context_modes: vec![LocalContextMode {
                stable_key: 42,
                ..LocalContextMode::default()
            }],
            slot_centers: vec![SlotPhaseCenter {
                cells: [1; L2_PHASE_CELLS],
                feature_mask: 2,
                context_mode_id: 0,
                support: 3,
                mass: 60,
                polarity: 1,
                flags: 0,
                reserved: 0,
            }],
            neighbor_couplings: vec![NeighborCoupling {
                context_mode_id: 0,
                target_feature_mask: 2,
                support: 12,
                ..NeighborCoupling::default()
            }],
            competition_edges: vec![CompetitionEdge {
                left_form_ref: 0,
                right_form_ref: 1,
                context_mode_id: 0,
                support_delta: -4,
                anti_delta: 8,
                evidence: 3,
                ..CompetitionEdge::default()
            }],
            calibration: TieCalibration {
                minimum_positive: 10,
                minimum_margin: 4,
                tie_window: 3,
                abstain_window: 2,
                false_authority_ceiling_milli: 1,
                evidence_count: 12,
                flags: 0,
            },
        }
    }

    #[test]
    fn standalone_l2_format_roundtrips_deterministically() {
        let package = fixture();
        let first = encode_package(&package).expect("encode");
        let second = encode_package(&package).expect("encode");
        assert_eq!(first, second);
        assert_eq!(decode_package(&first), Ok(package));
    }

    #[test]
    fn standalone_l2_format_rejects_corruption() {
        let mut bytes = encode_package(&fixture()).expect("encode");
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert_eq!(
            decode_package(&bytes),
            Err("L2 package checksum mismatch".to_string())
        );
    }
}
