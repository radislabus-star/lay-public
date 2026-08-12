use super::phase::FittedPhaseCenterV1;
use super::score::PRODUCTIVE_FEATURE_COUNT;
use super::types::{LemmaParadigmBindingV1, MorphologySlotKeyV1, MORPHOLOGY_SLOT_BYTES};
use super::L2_SCENE_PHASE_CELLS;

pub(super) trait FixedRecordV1: Sized {
    const BYTES: usize;

    fn encode_record(&self, output: &mut Vec<u8>);
    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str>;
}

pub(super) fn encode_records<T: FixedRecordV1>(records: &[T]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(records.len().saturating_mul(T::BYTES));
    for record in records {
        record.encode_record(&mut bytes);
    }
    bytes
}

pub(super) fn decode_records<T: FixedRecordV1>(bytes: &[u8]) -> Result<Vec<T>, &'static str> {
    if bytes.len() % T::BYTES != 0 {
        return Err("productive fixed-record section has a partial record");
    }
    bytes.chunks_exact(T::BYTES).map(T::decode_record).collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ParadigmCenterRecordV1 {
    pub(super) pos_domain: u16,
    pub(super) flags: u16,
    pub(super) root_node: u32,
    pub(super) transition_start: u32,
    pub(super) transition_count: u32,
    pub(super) slot_profile_start: u32,
    pub(super) slot_profile_count: u32,
    pub(super) program_start: u32,
    pub(super) program_count: u32,
    pub(super) support: u32,
    pub(super) stability: u16,
    pub(super) calibration_class: u16,
    pub(super) provenance_ref: u32,
    pub(super) signature_hash_low: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ParadigmCompatibilityIndexRecordV1 {
    pub(super) pos_domain: u16,
    pub(super) flags: u16,
    pub(super) source_slot_id: u32,
    pub(super) posting_start: u32,
    pub(super) posting_count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ParadigmPostingRecordV1 {
    pub(super) paradigm_id: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MorphProgramHeaderRecordV1 {
    pub(super) source_slot_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) op_start: u32,
    pub(super) op_count: u16,
    pub(super) flags: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum MorphOpcodeV1 {
    CopySourceRange = 1,
    DropSourcePrefix = 2,
    DropSourceSuffix = 3,
    EmitSegment = 4,
    ReplaceSourceRange = 5,
    EmitExactAllomorph = 6,
    Terminate = 7,
}

impl MorphOpcodeV1 {
    fn decode(value: u8) -> Result<Self, &'static str> {
        match value {
            1 => Ok(Self::CopySourceRange),
            2 => Ok(Self::DropSourcePrefix),
            3 => Ok(Self::DropSourceSuffix),
            4 => Ok(Self::EmitSegment),
            5 => Ok(Self::ReplaceSourceRange),
            6 => Ok(Self::EmitExactAllomorph),
            7 => Ok(Self::Terminate),
            _ => Err("productive morph operation has an unknown opcode"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MorphOpRecordV1 {
    pub(super) opcode: u8,
    pub(super) anchor: u8,
    pub(super) flags: u16,
    pub(super) arg0: i32,
    pub(super) arg1: u32,
    pub(super) arg2: u32,
}

impl MorphOpRecordV1 {
    pub(super) fn decoded_opcode(self) -> Result<MorphOpcodeV1, &'static str> {
        MorphOpcodeV1::decode(self.opcode)
    }

    fn validate(self) -> Result<(), &'static str> {
        if self.flags != 0 {
            return Err("productive morph operation has unknown flags");
        }
        let signed_i16 = || {
            i16::try_from(self.arg0)
                .map(|_| ())
                .map_err(|_| "productive morph operation offset exceeds i16")
        };
        match self.decoded_opcode()? {
            MorphOpcodeV1::CopySourceRange => {
                if !matches!(self.anchor, 1 | 2)
                    || self.arg2 != 0
                    || self.arg1 > u32::from(u16::MAX)
                {
                    return Err("productive COPY_SOURCE_RANGE arguments are invalid");
                }
                signed_i16()?;
            }
            MorphOpcodeV1::DropSourcePrefix | MorphOpcodeV1::DropSourceSuffix => {
                if self.anchor != 0
                    || self.arg0 != 0
                    || self.arg1 == 0
                    || self.arg1 >= u32::from(u16::MAX)
                    || self.arg2 != 0
                {
                    return Err("productive DROP_SOURCE edge arguments are invalid");
                }
            }
            MorphOpcodeV1::EmitSegment => {
                if self.anchor != 0 || self.arg0 != 0 || self.arg1 == 0 || self.arg2 != 0 {
                    return Err("productive EMIT_SEGMENT arguments are invalid");
                }
            }
            MorphOpcodeV1::ReplaceSourceRange => {
                if self.anchor != 2
                    || self.arg1 >= u32::from(u16::MAX)
                    || (self.arg1 == 0 && self.arg2 == 0)
                {
                    return Err("productive REPLACE_SOURCE_RANGE arguments are invalid");
                }
                signed_i16()?;
            }
            MorphOpcodeV1::EmitExactAllomorph => {
                if self.anchor != 0 || self.arg0 != 0 || self.arg1 == 0 || self.arg2 != 0 {
                    return Err("productive EMIT_EXACT_ALLOMORPH arguments are invalid");
                }
            }
            MorphOpcodeV1::Terminate => {
                if self.anchor != 0
                    || self.arg0 != 0
                    || self.arg1 == 0
                    || self.arg2 == 0
                    || self.arg2 > u32::from(u16::MAX)
                {
                    return Err("productive TERMINATE arguments are invalid");
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveTrieNodeRecordV1 {
    pub(super) arc_start: u32,
    pub(super) arc_count: u16,
    pub(super) terminal_count: u16,
    pub(super) terminal_start: u32,
    pub(super) flags: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveTrieArcRecordV1 {
    pub(super) child_node: u32,
    pub(super) stable_order: u32,
    pub(super) opcode: u8,
    pub(super) anchor: u8,
    pub(super) flags: u16,
    pub(super) arg0: i32,
    pub(super) arg1: u32,
    pub(super) arg2: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum ProductiveTrieArcOpcodeV1 {
    CopySourceRange = 1,
    CopyToRetainedEdge = 2,
    DropSourcePrefix = 3,
    DropSourceSuffix = 4,
    EmitSegment = 5,
    ReplaceSourceStart = 6,
    EmitExactAllomorph = 7,
}

impl ProductiveTrieArcRecordV1 {
    pub(super) fn decoded_opcode(self) -> Result<ProductiveTrieArcOpcodeV1, &'static str> {
        match self.opcode {
            1 => Ok(ProductiveTrieArcOpcodeV1::CopySourceRange),
            2 => Ok(ProductiveTrieArcOpcodeV1::CopyToRetainedEdge),
            3 => Ok(ProductiveTrieArcOpcodeV1::DropSourcePrefix),
            4 => Ok(ProductiveTrieArcOpcodeV1::DropSourceSuffix),
            5 => Ok(ProductiveTrieArcOpcodeV1::EmitSegment),
            6 => Ok(ProductiveTrieArcOpcodeV1::ReplaceSourceStart),
            7 => Ok(ProductiveTrieArcOpcodeV1::EmitExactAllomorph),
            _ => Err("productive trie arc has an unknown action opcode"),
        }
    }

    fn validate(self) -> Result<(), &'static str> {
        if self.flags != 0 {
            return Err("productive trie arc has unknown flags");
        }
        let signed_i16 = || {
            i16::try_from(self.arg0)
                .map(|_| ())
                .map_err(|_| "productive trie arc signed offset exceeds i16")
        };
        match self.decoded_opcode()? {
            ProductiveTrieArcOpcodeV1::CopySourceRange => {
                if !matches!(self.anchor, 1 | 2)
                    || self.arg1 == 0
                    || self.arg1 >= u32::from(u16::MAX)
                    || self.arg2 != 0
                {
                    return Err("productive trie COPY_SOURCE_RANGE arguments are invalid");
                }
                signed_i16()?;
            }
            ProductiveTrieArcOpcodeV1::CopyToRetainedEdge => {
                if !matches!(self.anchor, 1 | 2)
                    || self.arg1 > u32::from(u16::MAX)
                    || self.arg2 != 0
                {
                    return Err("productive trie COPY_TO_RETAINED_EDGE arguments are invalid");
                }
                signed_i16()?;
            }
            ProductiveTrieArcOpcodeV1::DropSourcePrefix
            | ProductiveTrieArcOpcodeV1::DropSourceSuffix => {
                if self.anchor != 0
                    || self.arg0 != 0
                    || self.arg1 == 0
                    || self.arg1 >= u32::from(u16::MAX)
                    || self.arg2 != 0
                {
                    return Err("productive trie DROP_SOURCE arguments are invalid");
                }
            }
            ProductiveTrieArcOpcodeV1::EmitSegment
            | ProductiveTrieArcOpcodeV1::EmitExactAllomorph => {
                if self.anchor != 0 || self.arg0 != 0 || self.arg1 == 0 || self.arg2 != 0 {
                    return Err("productive trie emitted reference arguments are invalid");
                }
            }
            ProductiveTrieArcOpcodeV1::ReplaceSourceStart => {
                if self.anchor != 2 || self.arg1 >= u32::from(u16::MAX) || self.arg2 != 0 {
                    return Err("productive trie replacement arguments are invalid");
                }
                signed_i16()?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveTerminalRecordV1 {
    pub(super) program_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) variant_id: u16,
    pub(super) flags: u16,
    pub(super) decoder_ref: u32,
    pub(super) evidence_ref: u32,
    pub(super) calibration_class: u16,
    pub(super) provenance_ref: u32,
    pub(super) stable_identity_hash: u32,
}

pub(super) const PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE: u16 = 1;
const PRODUCTIVE_TERMINAL_KNOWN_FLAGS: u16 = PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SlotPhaseProfileRecordV1 {
    pub(super) slot_id: u32,
    pub(super) feature_schema_id: u32,
    pub(super) positive_start: u32,
    pub(super) anti_start: u32,
    pub(super) hard_negative_start: u32,
    pub(super) ambiguity_start: u32,
    pub(super) positive_count: u16,
    pub(super) anti_count: u16,
    pub(super) hard_negative_count: u16,
    pub(super) ambiguity_count: u16,
    pub(super) calibration_class: u16,
    pub(super) flags: u16,
    pub(super) support: u32,
    pub(super) explicit_anti_support: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PhaseCenterRecordV1 {
    pub(super) cells: [i8; L2_SCENE_PHASE_CELLS],
    pub(super) feature_mask: u32,
    pub(super) context_mode_id: u32,
    pub(super) support: u16,
    pub(super) mass: u16,
    pub(super) polarity: i8,
    pub(super) flags: u8,
}

impl Default for PhaseCenterRecordV1 {
    fn default() -> Self {
        Self {
            cells: [0; L2_SCENE_PHASE_CELLS],
            feature_mask: 0,
            context_mode_id: 0,
            support: 0,
            mass: 0,
            polarity: 0,
            flags: 0,
        }
    }
}

impl From<FittedPhaseCenterV1> for PhaseCenterRecordV1 {
    fn from(center: FittedPhaseCenterV1) -> Self {
        Self {
            cells: center.cells,
            feature_mask: center.feature_mask,
            context_mode_id: center.context_mode_id,
            support: center.support,
            mass: center.mass,
            polarity: center.polarity,
            flags: center.flags,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DirectionalResidualRecordV1 {
    pub(super) source_scene_key: u32,
    pub(super) from_slot_id: u32,
    pub(super) to_slot_id: u32,
    pub(super) positive_support: u32,
    pub(super) explicit_anti_support: u32,
    pub(super) flags: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ModelCoefficientRecordV1 {
    pub(super) feature_id: u16,
    pub(super) flags: u16,
    pub(super) coefficient_q16: i32,
    pub(super) train_support: u32,
    pub(super) feature_schema_hash_low: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct EvidencePriorRecordV1 {
    pub(super) channel_id: u16,
    pub(super) flags: u16,
    pub(super) positive_prior_twice: u64,
    pub(super) contradiction_prior_twice: u64,
    pub(super) reserved: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CalibrationCellRecordV1 {
    pub(super) stratum_key_id: u32,
    pub(super) winner_margin_q16: i32,
    pub(super) tie_radius_q16: i32,
    pub(super) support: u32,
    pub(super) correct_winner_count: u32,
    pub(super) false_winner_count: u32,
    pub(super) tied_count: u32,
    pub(super) flags: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProvenanceRecordV1 {
    pub(super) source_kind: u16,
    pub(super) flags: u16,
    pub(super) source_id: u64,
    pub(super) event_start: u64,
    pub(super) event_count: u32,
    pub(super) source_hash_prefix: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeltaManifestRecordV1 {
    pub(super) base_package_sha256: [u8; 32],
    pub(super) previous_generation_sha256: [u8; 32],
    pub(super) generation: u64,
    pub(super) event_start: u64,
    pub(super) event_end: u64,
    pub(super) section_count_ref: u64,
    pub(super) coefficient_generation: u64,
    pub(super) calibration_generation: u64,
    pub(super) proof_receipt_sha256: [u8; 32],
    pub(super) requested_authority_scope: u32,
    pub(super) flags: u32,
    pub(super) payload_sha256: [u8; 32],
}

impl Default for DeltaManifestRecordV1 {
    fn default() -> Self {
        Self {
            base_package_sha256: [0; 32],
            previous_generation_sha256: [0; 32],
            generation: 0,
            event_start: 0,
            event_end: 0,
            section_count_ref: 0,
            coefficient_generation: 0,
            calibration_generation: 0,
            proof_receipt_sha256: [0; 32],
            requested_authority_scope: 0,
            flags: 0,
            payload_sha256: [0; 32],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DeltaRecordHeaderRecordV1 {
    pub(super) kind: u16,
    pub(super) flags: u16,
    pub(super) generation: u64,
    pub(super) typed_key_hash: u64,
    pub(super) payload_offset: u32,
    pub(super) payload_bytes: u32,
    pub(super) crc32: u32,
}

impl FixedRecordV1 for MorphologySlotKeyV1 {
    const BYTES: usize = MORPHOLOGY_SLOT_BYTES;

    fn encode_record(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.to_bytes());
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        expect_width(bytes, Self::BYTES)?;
        Self::from_bytes(bytes.try_into().expect("fixed morphology slot"))
    }
}

impl FixedRecordV1 for LemmaParadigmBindingV1 {
    const BYTES: usize = 40;

    fn encode_record(&self, output: &mut Vec<u8>) {
        put_u32(output, self.lemma_id);
        put_u32(output, self.paradigm_id);
        put_u32(output, self.canonical_source_form_ref);
        put_u32(output, self.observed_slot_set_ref);
        put_u32(output, self.positive_support);
        put_u32(output, self.explicit_anti_support);
        put_u16(output, self.stability);
        put_u16(output, self.flags);
        put_u32(output, self.program_start);
        put_u16(output, self.program_count);
        put_u16(output, 0);
        put_u32(output, self.provenance_ref);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let record = Self {
            lemma_id: input.u32()?,
            paradigm_id: input.u32()?,
            canonical_source_form_ref: input.u32()?,
            observed_slot_set_ref: input.u32()?,
            positive_support: input.u32()?,
            explicit_anti_support: input.u32()?,
            stability: input.u16()?,
            flags: input.u16()?,
            program_start: input.u32()?,
            program_count: input.u16()?,
            provenance_ref: {
                if input.u16()? != 0 {
                    return Err("productive lemma binding reserved value is not zero");
                }
                input.u32()?
            },
        };
        if record.paradigm_id == 0 || record.observed_slot_set_ref == 0 || record.flags != 0 {
            return Err("productive lemma binding identity or flags are invalid");
        }
        Ok(record)
    }
}

macro_rules! fixed_record {
    ($type:ty, $bytes:expr, {$($field:ident : $read:ident),+ $(,)?}, $validate:expr) => {
        impl FixedRecordV1 for $type {
            const BYTES: usize = $bytes;

            fn encode_record(&self, output: &mut Vec<u8>) {
                $(paste_put(output, stringify!($read), self.$field as i128);)+
            }

            fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
                let mut input = RecordInput::new(bytes, Self::BYTES)?;
                let record = Self { $($field: input.$read()?,)+ };
                ($validate)(&record)?;
                Ok(record)
            }
        }
    };
}

fixed_record!(ParadigmCenterRecordV1, 48, {
    pos_domain:u16, flags:u16, root_node:u32, transition_start:u32,
    transition_count:u32, slot_profile_start:u32, slot_profile_count:u32,
    program_start:u32, program_count:u32, support:u32, stability:u16,
    calibration_class:u16, provenance_ref:u32, signature_hash_low:u32
}, |record: &ParadigmCenterRecordV1| {
    if record.pos_domain == 0 || record.flags != 0 || record.calibration_class == 0 {
        Err("productive paradigm center identity or flags are invalid")
    } else { Ok(()) }
});

fixed_record!(ParadigmCompatibilityIndexRecordV1, 16, {
    pos_domain:u16, flags:u16, source_slot_id:u32, posting_start:u32, posting_count:u32
}, |record: &ParadigmCompatibilityIndexRecordV1| {
    if record.pos_domain == 0 || record.flags != 0 || record.source_slot_id == 0 || record.posting_count == 0 {
        Err("productive compatibility index identity or flags are invalid")
    } else { Ok(()) }
});

fixed_record!(ParadigmPostingRecordV1, 4, { paradigm_id:u32 }, |record: &ParadigmPostingRecordV1| {
    if record.paradigm_id == 0 { Err("productive paradigm posting has a zero identity") } else { Ok(()) }
});

fixed_record!(MorphProgramHeaderRecordV1, 16, {
    source_slot_id:u32, target_slot_id:u32, op_start:u32, op_count:u16, flags:u16
}, |record: &MorphProgramHeaderRecordV1| {
    if record.source_slot_id == 0 || record.target_slot_id == 0 || record.op_count == 0 || record.flags != 0 {
        Err("productive morph program identity or flags are invalid")
    } else { Ok(()) }
});

impl FixedRecordV1 for MorphOpRecordV1 {
    const BYTES: usize = 16;

    fn encode_record(&self, output: &mut Vec<u8>) {
        output.push(self.opcode);
        output.push(self.anchor);
        put_u16(output, self.flags);
        put_i32(output, self.arg0);
        put_u32(output, self.arg1);
        put_u32(output, self.arg2);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let record = Self {
            opcode: input.u8()?,
            anchor: input.u8()?,
            flags: input.u16()?,
            arg0: input.i32()?,
            arg1: input.u32()?,
            arg2: input.u32()?,
        };
        record.validate()?;
        Ok(record)
    }
}

fixed_record!(ProductiveTrieNodeRecordV1, 16, {
    arc_start:u32, arc_count:u16, terminal_count:u16, terminal_start:u32, flags:u32
}, |record: &ProductiveTrieNodeRecordV1| {
    if record.flags != 0 { Err("productive trie node has unknown flags") } else { Ok(()) }
});

impl FixedRecordV1 for ProductiveTrieArcRecordV1 {
    const BYTES: usize = 24;

    fn encode_record(&self, output: &mut Vec<u8>) {
        put_u32(output, self.child_node);
        put_u32(output, self.stable_order);
        output.push(self.opcode);
        output.push(self.anchor);
        put_u16(output, self.flags);
        put_i32(output, self.arg0);
        put_u32(output, self.arg1);
        put_u32(output, self.arg2);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let record = Self {
            child_node: input.u32()?,
            stable_order: input.u32()?,
            opcode: input.u8()?,
            anchor: input.u8()?,
            flags: input.u16()?,
            arg0: input.i32()?,
            arg1: input.u32()?,
            arg2: input.u32()?,
        };
        record.validate()?;
        Ok(record)
    }
}

impl FixedRecordV1 for ProductiveTerminalRecordV1 {
    const BYTES: usize = 32;

    fn encode_record(&self, output: &mut Vec<u8>) {
        put_u32(output, self.program_id);
        put_u32(output, self.target_slot_id);
        put_u16(output, self.variant_id);
        put_u16(output, self.flags);
        put_u32(output, self.decoder_ref);
        put_u32(output, self.evidence_ref);
        put_u16(output, self.calibration_class);
        put_u16(output, 0);
        put_u32(output, self.provenance_ref);
        put_u32(output, self.stable_identity_hash);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let record = Self {
            program_id: input.u32()?,
            target_slot_id: input.u32()?,
            variant_id: input.u16()?,
            flags: input.u16()?,
            decoder_ref: input.u32()?,
            evidence_ref: input.u32()?,
            calibration_class: input.u16()?,
            provenance_ref: {
                if input.u16()? != 0 {
                    return Err("productive terminal reserved value is not zero");
                }
                input.u32()?
            },
            stable_identity_hash: input.u32()?,
        };
        let surface_from_trie = record.flags & PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE != 0;
        if record.program_id == 0
            || record.target_slot_id == 0
            || record.variant_id == 0
            || record.flags & !PRODUCTIVE_TERMINAL_KNOWN_FLAGS != 0
            || surface_from_trie == (record.decoder_ref != 0)
            || record.evidence_ref == 0
            || record.calibration_class == 0
        {
            return Err("productive terminal identity, reference, or flags are invalid");
        }
        Ok(record)
    }
}

fixed_record!(SlotPhaseProfileRecordV1, 44, {
    slot_id:u32, feature_schema_id:u32, positive_start:u32, anti_start:u32,
    hard_negative_start:u32, ambiguity_start:u32, positive_count:u16,
    anti_count:u16, hard_negative_count:u16, ambiguity_count:u16,
    calibration_class:u16, flags:u16, support:u32, explicit_anti_support:u32
}, |record: &SlotPhaseProfileRecordV1| {
    if record.slot_id == 0
        || record.feature_schema_id == 0
        || record.calibration_class == 0
        || record.flags != 0
        || usize::from(record.positive_count) > super::MAX_POSITIVE_SUBCENTERS
        || usize::from(record.anti_count) > super::MAX_ANTI_SUBCENTERS
        || usize::from(record.hard_negative_count) > super::MAX_HARD_NEGATIVE_SUBCENTERS
        || usize::from(record.ambiguity_count) > super::MAX_AMBIGUITY_SUBCENTERS
    {
        Err("productive slot phase profile identity, count, or flags are invalid")
    } else { Ok(()) }
});

impl FixedRecordV1 for PhaseCenterRecordV1 {
    const BYTES: usize = 76;

    fn encode_record(&self, output: &mut Vec<u8>) {
        output.extend(self.cells.map(|value| value as u8));
        put_u32(output, self.feature_mask);
        put_u32(output, self.context_mode_id);
        put_u16(output, self.support);
        put_u16(output, self.mass);
        output.push(self.polarity as u8);
        output.push(self.flags);
        put_u16(output, 0);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let mut cells = [0_i8; L2_SCENE_PHASE_CELLS];
        for cell in &mut cells {
            *cell = input.u8()? as i8;
        }
        let record = Self {
            cells,
            feature_mask: input.u32()?,
            context_mode_id: input.u32()?,
            support: input.u16()?,
            mass: input.u16()?,
            polarity: input.u8()? as i8,
            flags: input.u8()?,
        };
        if input.u16()? != 0
            || record.flags & !1 != 0
            || !matches!(record.polarity, -2 | -1 | 0 | 1)
            || record.support == 0
        {
            return Err(
                "productive phase center polarity, support, flags, or reserved value is invalid",
            );
        }
        Ok(record)
    }
}

fixed_record!(DirectionalResidualRecordV1, 24, {
    source_scene_key:u32, from_slot_id:u32, to_slot_id:u32,
    positive_support:u32, explicit_anti_support:u32, flags:u32
}, |record: &DirectionalResidualRecordV1| {
    if record.source_scene_key == 0 || record.from_slot_id == 0 || record.to_slot_id == 0 || record.flags != 0 {
        Err("productive directional residual identity or flags are invalid")
    } else { Ok(()) }
});

fixed_record!(ModelCoefficientRecordV1, 16, {
    feature_id:u16, flags:u16, coefficient_q16:i32, train_support:u32,
    feature_schema_hash_low:u32
}, |record: &ModelCoefficientRecordV1| {
    if record.feature_id == 0
        || usize::from(record.feature_id) > PRODUCTIVE_FEATURE_COUNT
        || record.flags != 0
        || record.coefficient_q16 < 0
    {
        Err("productive model coefficient feature, polarity, or flags are invalid")
    } else { Ok(()) }
});

fixed_record!(EvidencePriorRecordV1, 24, {
    channel_id:u16, flags:u16, positive_prior_twice:u64,
    contradiction_prior_twice:u64, reserved:u32
}, |record: &EvidencePriorRecordV1| {
    if !(1..=4).contains(&record.channel_id)
        || record.flags != 0
        || record.positive_prior_twice == 0
        || record.positive_prior_twice % 2 == 0
        || record.contradiction_prior_twice == 0
        || record.contradiction_prior_twice % 2 == 0
        || record.reserved != 0
    {
        Err("productive evidence prior identity, smoothing, or reserved value is invalid")
    } else { Ok(()) }
});

fixed_record!(CalibrationCellRecordV1, 32, {
    stratum_key_id:u32, winner_margin_q16:i32, tie_radius_q16:i32,
    support:u32, correct_winner_count:u32, false_winner_count:u32,
    tied_count:u32, flags:u32
}, |record: &CalibrationCellRecordV1| {
    if record.stratum_key_id == 0
        || (record.winner_margin_q16 != i32::MIN && record.winner_margin_q16 < 0)
        || record.tie_radius_q16 < 0
        || record.flags != 0
    {
        Err("productive calibration cell key, radius, or flags are invalid")
    } else { Ok(()) }
});

impl FixedRecordV1 for ProvenanceRecordV1 {
    const BYTES: usize = 32;

    fn encode_record(&self, output: &mut Vec<u8>) {
        put_u16(output, self.source_kind);
        put_u16(output, self.flags);
        put_u64(output, self.source_id);
        put_u64(output, self.event_start);
        put_u32(output, self.event_count);
        put_u64(output, self.source_hash_prefix);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let record = Self {
            source_kind: input.u16()?,
            flags: input.u16()?,
            source_id: input.u64()?,
            event_start: input.u64()?,
            event_count: input.u32()?,
            source_hash_prefix: input.u64()?,
        };
        if record.source_kind == 0 || record.flags != 0 || record.source_id == 0 {
            return Err("productive provenance identity or flags are invalid");
        }
        Ok(record)
    }
}

impl FixedRecordV1 for DeltaManifestRecordV1 {
    const BYTES: usize = 192;

    fn encode_record(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.base_package_sha256);
        output.extend_from_slice(&self.previous_generation_sha256);
        put_u64(output, self.generation);
        put_u64(output, self.event_start);
        put_u64(output, self.event_end);
        put_u64(output, self.section_count_ref);
        put_u64(output, self.coefficient_generation);
        put_u64(output, self.calibration_generation);
        output.extend_from_slice(&self.proof_receipt_sha256);
        put_u32(output, self.requested_authority_scope);
        put_u32(output, self.flags);
        output.extend_from_slice(&self.payload_sha256);
        output.extend_from_slice(&[0; 8]);
    }

    fn decode_record(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = RecordInput::new(bytes, Self::BYTES)?;
        let record = Self {
            base_package_sha256: input.array()?,
            previous_generation_sha256: input.array()?,
            generation: input.u64()?,
            event_start: input.u64()?,
            event_end: input.u64()?,
            section_count_ref: input.u64()?,
            coefficient_generation: input.u64()?,
            calibration_generation: input.u64()?,
            proof_receipt_sha256: input.array()?,
            requested_authority_scope: input.u32()?,
            flags: input.u32()?,
            payload_sha256: input.array()?,
        };
        if input.array::<8>()? != [0; 8]
            || record.flags != 0
            || record.event_end < record.event_start
            || record.requested_authority_scope > 3
        {
            return Err(
                "productive delta manifest range, scope, flags, or reserved bytes are invalid",
            );
        }
        Ok(record)
    }
}

fixed_record!(DeltaRecordHeaderRecordV1, 32, {
    kind:u16, flags:u16, generation:u64, typed_key_hash:u64,
    payload_offset:u32, payload_bytes:u32, crc32:u32
}, |record: &DeltaRecordHeaderRecordV1| {
    if !(1..=9).contains(&record.kind) || record.flags != 0 || record.generation == 0 || record.payload_bytes == 0 {
        Err("productive delta record header kind, generation, payload, or flags are invalid")
    } else { Ok(()) }
});

fn expect_width(bytes: &[u8], expected: usize) -> Result<(), &'static str> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err("productive fixed record width mismatch")
    }
}

struct RecordInput<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> RecordInput<'a> {
    fn new(bytes: &'a [u8], expected: usize) -> Result<Self, &'static str> {
        expect_width(bytes, expected)?;
        Ok(Self { bytes, offset: 0 })
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], &'static str> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or("productive fixed record read overflow")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("productive fixed record is truncated")?
            .try_into()
            .expect("fixed slice");
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, &'static str> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, &'static str> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, &'static str> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32, &'static str> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, &'static str> {
        Ok(u64::from_le_bytes(self.array()?))
    }
}

fn put_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn paste_put(output: &mut Vec<u8>, kind: &str, value: i128) {
    match kind {
        "u16" => put_u16(output, value as u16),
        "u32" => put_u32(output, value as u32),
        "i32" => put_i32(output, value as i32),
        "u64" => put_u64(output, value as u64),
        _ => unreachable!("fixed record macro contains an unsupported field type"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<T>(record: T)
    where
        T: FixedRecordV1 + Clone + std::fmt::Debug + PartialEq,
    {
        let bytes = encode_records(std::slice::from_ref(&record));
        assert_eq!(bytes.len(), T::BYTES);
        assert_eq!(decode_records::<T>(&bytes).expect("decode"), vec![record]);
    }

    #[test]
    fn fixed_record_codecs_preserve_normative_widths_and_fields() {
        roundtrip(MorphologySlotKeyV1::new(
            2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ));
        roundtrip(LemmaParadigmBindingV1 {
            lemma_id: 1,
            paradigm_id: 1,
            canonical_source_form_ref: 1,
            observed_slot_set_ref: 8,
            positive_support: 4,
            explicit_anti_support: 0,
            stability: 9,
            flags: 0,
            program_start: 0,
            program_count: 1,
            provenance_ref: 1,
        });
        roundtrip(ParadigmCenterRecordV1 {
            pos_domain: 2,
            root_node: 0,
            calibration_class: 1,
            ..ParadigmCenterRecordV1::default()
        });
        roundtrip(ParadigmCompatibilityIndexRecordV1 {
            pos_domain: 2,
            source_slot_id: 1,
            posting_count: 1,
            ..ParadigmCompatibilityIndexRecordV1::default()
        });
        roundtrip(ParadigmPostingRecordV1 { paradigm_id: 1 });
        roundtrip(MorphProgramHeaderRecordV1 {
            source_slot_id: 1,
            target_slot_id: 2,
            op_count: 1,
            ..MorphProgramHeaderRecordV1::default()
        });
        roundtrip(MorphOpRecordV1 {
            opcode: MorphOpcodeV1::Terminate as u8,
            arg1: 2,
            arg2: 1,
            ..MorphOpRecordV1::default()
        });
        roundtrip(MorphOpRecordV1 {
            opcode: MorphOpcodeV1::ReplaceSourceRange as u8,
            anchor: 2,
            arg0: -2,
            arg1: 1,
            arg2: 0,
            ..MorphOpRecordV1::default()
        });
        roundtrip(MorphOpRecordV1 {
            opcode: MorphOpcodeV1::ReplaceSourceRange as u8,
            anchor: 2,
            arg0: -1,
            arg1: 0,
            arg2: 8,
            ..MorphOpRecordV1::default()
        });
        roundtrip(ProductiveTrieNodeRecordV1::default());
        roundtrip(ProductiveTrieArcRecordV1 {
            opcode: ProductiveTrieArcOpcodeV1::EmitSegment as u8,
            arg1: 8,
            ..ProductiveTrieArcRecordV1::default()
        });
        roundtrip(ProductiveTrieArcRecordV1 {
            opcode: ProductiveTrieArcOpcodeV1::ReplaceSourceStart as u8,
            anchor: 2,
            arg0: -1,
            arg1: 0,
            ..ProductiveTrieArcRecordV1::default()
        });
        roundtrip(ProductiveTerminalRecordV1 {
            program_id: 1,
            target_slot_id: 2,
            variant_id: 1,
            decoder_ref: 8,
            evidence_ref: 1,
            calibration_class: 1,
            provenance_ref: 1,
            stable_identity_hash: 7,
            ..ProductiveTerminalRecordV1::default()
        });
        roundtrip(SlotPhaseProfileRecordV1 {
            slot_id: 2,
            feature_schema_id: 1,
            calibration_class: 1,
            ..SlotPhaseProfileRecordV1::default()
        });
        roundtrip(PhaseCenterRecordV1 {
            cells: [1; L2_SCENE_PHASE_CELLS],
            feature_mask: 1,
            context_mode_id: 1,
            support: 1,
            mass: u16::MAX,
            polarity: 1,
            flags: 0,
        });
        roundtrip(DirectionalResidualRecordV1 {
            source_scene_key: 1,
            from_slot_id: 1,
            to_slot_id: 2,
            ..DirectionalResidualRecordV1::default()
        });
        roundtrip(ModelCoefficientRecordV1 {
            feature_id: 1,
            coefficient_q16: 1,
            train_support: 1,
            feature_schema_hash_low: 1,
            ..ModelCoefficientRecordV1::default()
        });
        roundtrip(EvidencePriorRecordV1 {
            channel_id: 1,
            positive_prior_twice: 201,
            contradiction_prior_twice: 21,
            ..EvidencePriorRecordV1::default()
        });
        roundtrip(CalibrationCellRecordV1 {
            stratum_key_id: 1,
            tie_radius_q16: 1,
            support: 200,
            ..CalibrationCellRecordV1::default()
        });
        roundtrip(ProvenanceRecordV1 {
            source_kind: 1,
            source_id: 1,
            event_count: 1,
            source_hash_prefix: 1,
            ..ProvenanceRecordV1::default()
        });
        roundtrip(DeltaManifestRecordV1::default());
        roundtrip(DeltaRecordHeaderRecordV1 {
            kind: 1,
            generation: 1,
            payload_bytes: 1,
            ..DeltaRecordHeaderRecordV1::default()
        });
    }

    #[test]
    fn codecs_reject_reserved_unknown_and_partial_records() {
        let mut phase = encode_records(&[PhaseCenterRecordV1 {
            cells: [1; L2_SCENE_PHASE_CELLS],
            support: 1,
            polarity: 1,
            ..PhaseCenterRecordV1::default()
        }]);
        phase[75] = 1;
        assert!(decode_records::<PhaseCenterRecordV1>(&phase).is_err());
        assert!(decode_records::<ParadigmPostingRecordV1>(&[0; 3]).is_err());
        assert!(MorphOpRecordV1 {
            opcode: 255,
            ..MorphOpRecordV1::default()
        }
        .validate()
        .is_err());
        assert!(MorphOpRecordV1 {
            opcode: MorphOpcodeV1::ReplaceSourceRange as u8,
            anchor: 2,
            ..MorphOpRecordV1::default()
        }
        .validate()
        .is_err());
    }
}
