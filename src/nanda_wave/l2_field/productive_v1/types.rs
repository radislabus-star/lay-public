use std::fmt;

pub(super) const MORPHOLOGY_AXIS_COUNT: usize = 13;
pub(super) const MORPHOLOGY_SLOT_BYTES: usize = 16;
pub(super) const AXIS_INAPPLICABLE: u8 = 0;
pub(super) const AXIS_UNKNOWN_OR_UNANNOTATED: u8 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(super) struct ImportedCanonicalL2LemmaRefV1(pub(super) u32);

impl ImportedCanonicalL2LemmaRefV1 {
    pub(super) fn validate_for_count(self, count: usize) -> Result<(), &'static str> {
        if self.0 as usize >= count {
            return Err("imported canonical L2 lemma reference is out of range");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(super) struct ImportedCanonicalL2FormRefV1(pub(super) u32);

impl ImportedCanonicalL2FormRefV1 {
    pub(super) fn validate_for_count(self, count: usize) -> Result<(), &'static str> {
        if self.0 as usize >= count {
            return Err("imported canonical L2 form reference is out of range");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct CanonicalL2BindingIdentityV1 {
    pub(super) lemma_ref: ImportedCanonicalL2LemmaRefV1,
    pub(super) form_ref: ImportedCanonicalL2FormRefV1,
    pub(super) legacy_feature_mask: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub(super) struct MorphologySlotKeyV1 {
    pub(super) part_of_speech: u8,
    pub(super) number: u8,
    pub(super) case: u8,
    pub(super) gender: u8,
    pub(super) person: u8,
    pub(super) tense: u8,
    pub(super) mood: u8,
    pub(super) aspect: u8,
    pub(super) voice: u8,
    pub(super) form_kind: u8,
    pub(super) degree: u8,
    pub(super) animacy: u8,
    pub(super) variant_kind: u8,
    reserved: [u8; 3],
}

const _: [(); MORPHOLOGY_SLOT_BYTES] = [(); std::mem::size_of::<MorphologySlotKeyV1>()];

impl MorphologySlotKeyV1 {
    #[expect(
        clippy::too_many_arguments,
        reason = "existing explicit boundary contract"
    )]
    pub(super) const fn new(
        part_of_speech: u8,
        number: u8,
        case: u8,
        gender: u8,
        person: u8,
        tense: u8,
        mood: u8,
        aspect: u8,
        voice: u8,
        form_kind: u8,
        degree: u8,
        animacy: u8,
        variant_kind: u8,
    ) -> Self {
        Self {
            part_of_speech,
            number,
            case,
            gender,
            person,
            tense,
            mood,
            aspect,
            voice,
            form_kind,
            degree,
            animacy,
            variant_kind,
            reserved: [0; 3],
        }
    }

    pub(super) fn from_bytes(bytes: [u8; MORPHOLOGY_SLOT_BYTES]) -> Result<Self, &'static str> {
        if bytes[13..16] != [0; 3] {
            return Err("productive morphology slot reserved bytes are not zero");
        }
        Ok(Self::new(
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12],
        ))
    }

    pub(super) const fn axes(self) -> [u8; MORPHOLOGY_AXIS_COUNT] {
        [
            self.part_of_speech,
            self.number,
            self.case,
            self.gender,
            self.person,
            self.tense,
            self.mood,
            self.aspect,
            self.voice,
            self.form_kind,
            self.degree,
            self.animacy,
            self.variant_kind,
        ]
    }

    pub(super) const fn to_bytes(self) -> [u8; MORPHOLOGY_SLOT_BYTES] {
        let axes = self.axes();
        [
            axes[0], axes[1], axes[2], axes[3], axes[4], axes[5], axes[6], axes[7], axes[8],
            axes[9], axes[10], axes[11], axes[12], 0, 0, 0,
        ]
    }

    pub(super) const fn pos_domain(self) -> u8 {
        self.part_of_speech
    }

    pub(super) fn annotation_complete(self, applicability: MorphologyApplicabilityMaskV1) -> bool {
        self.validate(applicability).is_ok()
            && self.axes().into_iter().enumerate().all(|(axis, value)| {
                !applicability.contains(axis) || value != AXIS_UNKNOWN_OR_UNANNOTATED
            })
    }

    pub(super) fn validate(
        self,
        applicability: MorphologyApplicabilityMaskV1,
    ) -> Result<(), SlotValidationErrorV1> {
        if self.reserved != [0; 3] {
            return Err(SlotValidationErrorV1::ReservedBytesNotZero);
        }
        for (axis, value) in self.axes().into_iter().enumerate() {
            let expected_applicable = applicability.contains(axis);
            let encoded_applicable = value != AXIS_INAPPLICABLE;
            if expected_applicable != encoded_applicable {
                return Err(SlotValidationErrorV1::ApplicabilityMismatch {
                    axis: axis as u8,
                    value,
                    expected_applicable,
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MorphologyApplicabilityMaskV1(u16);

impl MorphologyApplicabilityMaskV1 {
    const VALID_BITS: u16 = (1_u16 << MORPHOLOGY_AXIS_COUNT) - 1;

    pub(super) fn new(bits: u16) -> Result<Self, SlotValidationErrorV1> {
        if bits & !Self::VALID_BITS != 0 {
            return Err(SlotValidationErrorV1::UnknownApplicabilityBits(bits));
        }
        Ok(Self(bits))
    }

    pub(super) const fn bits(self) -> u16 {
        self.0
    }

    pub(super) const fn contains(self, axis: usize) -> bool {
        axis < MORPHOLOGY_AXIS_COUNT && self.0 & (1_u16 << axis) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SlotValidationErrorV1 {
    UnknownApplicabilityBits(u16),
    ReservedBytesNotZero,
    ApplicabilityMismatch {
        axis: u8,
        value: u8,
        expected_applicable: bool,
    },
}

impl fmt::Display for SlotValidationErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownApplicabilityBits(bits) => {
                write!(formatter, "unknown morphology applicability bits: {bits:#06x}")
            }
            Self::ReservedBytesNotZero => formatter.write_str("slot reserved bytes are not zero"),
            Self::ApplicabilityMismatch {
                axis,
                value,
                expected_applicable,
            } => write!(
                formatter,
                "morphology axis {axis} value {value} disagrees with applicability={expected_applicable}"
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct FormIdentityV1 {
    pub(super) lemma_id: u32,
    pub(super) morphology_slot_id: u32,
    pub(super) normalized_surface_id: u32,
    pub(super) variant_id: u32,
}

impl FormIdentityV1 {
    pub(super) fn validate(self) -> Result<(), &'static str> {
        if self.morphology_slot_id == 0 || self.normalized_surface_id == 0 || self.variant_id == 0 {
            return Err("sidecar-owned form identity contains zero");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct LemmaParadigmBindingV1 {
    pub(super) lemma_id: u32,
    pub(super) paradigm_id: u32,
    pub(super) canonical_source_form_ref: u32,
    pub(super) observed_slot_set_ref: u32,
    pub(super) positive_support: u32,
    pub(super) explicit_anti_support: u32,
    pub(super) stability: u16,
    pub(super) flags: u16,
    pub(super) program_start: u32,
    pub(super) program_count: u16,
    pub(super) provenance_ref: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ProductiveCandidateIdentityV1 {
    pub(super) lemma_id: u32,
    pub(super) paradigm_id: u32,
    pub(super) program_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) normalized_surface_id: u32,
    pub(super) variant_id: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum IndependentEvidenceSourceKindV1 {
    ExplicitFeedback = 1,
    StructurallyImpossibleAgreement = 2,
    LocalContextAntiCenter = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ContradictionCertificateV1 {
    pub(super) grounded_candidate: ProductiveCandidateIdentityV1,
    pub(super) competing_candidate: Option<ProductiveCandidateIdentityV1>,
    pub(super) impossible_slot_id: Option<u32>,
    pub(super) independent_source: IndependentEvidenceSourceKindV1,
    pub(super) calibration_stratum_id: u32,
    pub(super) support_count: u32,
    pub(super) calibrated_margin_q16: i32,
    pub(super) false_contradiction_count: u32,
    pub(super) package_generation: u64,
    pub(super) delta_generation: u64,
    pub(super) provenance_start: u32,
    pub(super) provenance_count: u32,
}

impl ContradictionCertificateV1 {
    pub(super) fn validate(self) -> Result<(), &'static str> {
        if self.false_contradiction_count != 0 {
            return Err("contradiction certificate has nonzero false contradiction count");
        }
        if self.competing_candidate.is_some() == self.impossible_slot_id.is_some() {
            return Err("certificate must identify exactly one competitor or impossible slot");
        }
        if self.support_count == 0 || self.calibration_stratum_id == 0 {
            return Err("certificate lacks independently calibrated support");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morphology_slot_key_has_normative_width() {
        assert_eq!(std::mem::size_of::<MorphologySlotKeyV1>(), 16);
    }

    #[test]
    fn applicability_distinguishes_inapplicable_from_unknown() {
        let slot = MorphologySlotKeyV1::new(2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        let mask = MorphologyApplicabilityMaskV1::new(0b11).expect("valid mask");
        assert!(slot.validate(mask).is_ok());

        let wrong = MorphologyApplicabilityMaskV1::new(0b1).expect("valid mask");
        assert!(matches!(
            slot.validate(wrong),
            Err(SlotValidationErrorV1::ApplicabilityMismatch { axis: 1, .. })
        ));
    }
}
