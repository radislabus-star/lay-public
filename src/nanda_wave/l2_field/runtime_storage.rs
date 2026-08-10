use std::borrow::Cow;
use std::path::Path;

use super::compact_format::{self, CompactPackageView};
use super::compositional::RuntimeLemmaWaveIndex;
use super::format;
use super::model::{
    CompetitionEdge, FormCenterRef, L2FieldPackage, LemmaCenter, LocalContextMode, MorphBinding,
    NeighborCoupling, SlotPhaseCenter, TieCalibration,
};
use super::package_bytes::PackageBytes;

#[derive(Clone, Debug)]
pub(super) enum RuntimeL2Package {
    Reference(L2FieldPackage),
    Compact(CompactPackageView),
}

impl RuntimeL2Package {
    pub(super) fn load(path: &Path) -> Result<Self, String> {
        let bytes = PackageBytes::load(path)?;
        if compact_format::is_compact_package(bytes.as_slice()) {
            CompactPackageView::from_backing(bytes).map(Self::Compact)
        } else {
            format::decode_package(bytes.as_slice()).map(Self::Reference)
        }
    }

    pub(super) fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if compact_format::is_compact_package(&bytes) {
            CompactPackageView::from_bytes(bytes).map(Self::Compact)
        } else {
            format::decode_package(&bytes).map(Self::Reference)
        }
    }

    pub(super) fn from_reference(package: L2FieldPackage) -> Self {
        Self::Reference(package)
    }

    pub(super) fn storage_kind(&self) -> &'static str {
        match self {
            Self::Reference(_) => "reference_v2_owned",
            Self::Compact(package) => package.storage_kind(),
        }
    }

    pub(super) fn take_lemma_wave_index(&mut self) -> Option<RuntimeLemmaWaveIndex> {
        match self {
            Self::Reference(_) => None,
            Self::Compact(package) => package.take_lemma_wave_index(),
        }
    }

    pub(super) fn mmap_backed(&self) -> bool {
        match self {
            Self::Reference(_) => false,
            Self::Compact(package) => package.mmap_backed(),
        }
    }

    pub(super) fn backing_bytes(&self) -> usize {
        match self {
            Self::Reference(package) => {
                128 + package.form_refs.len() * 16
                    + package.decoder_bytes.len()
                    + package.lemma_centers.len() * 32
                    + package.morph_bindings.len() * 16
                    + package.context_modes.len() * 16
                    + package.slot_centers.len() * 76
                    + package.neighbor_couplings.len() * 24
                    + package.competition_edges.len() * 24
                    + 24
            }
            Self::Compact(package) => package.backing_bytes(),
        }
    }

    pub(super) fn l1_package_fingerprint(&self) -> u64 {
        match self {
            Self::Reference(package) => package.l1_package_fingerprint,
            Self::Compact(package) => package.l1_package_fingerprint(),
        }
    }

    pub(super) fn form_count(&self) -> usize {
        match self {
            Self::Reference(package) => package.form_refs.len(),
            Self::Compact(package) => package.form_count(),
        }
    }

    pub(super) fn binding_count(&self) -> usize {
        match self {
            Self::Reference(package) => package.morph_bindings.len(),
            Self::Compact(package) => package.binding_count(),
        }
    }

    pub(super) fn raw_decoder_bytes(&self) -> usize {
        match self {
            Self::Reference(package) => package.decoder_bytes.len(),
            Self::Compact(package) => package.raw_decoder_bytes(),
        }
    }

    pub(super) fn form(&self, index: usize) -> Option<FormCenterRef> {
        match self {
            Self::Reference(package) => package.form_refs.get(index).copied(),
            Self::Compact(package) => package.form(index),
        }
    }

    pub(super) fn surface(&self, form_ref: usize) -> Option<Cow<'_, str>> {
        match self {
            Self::Reference(package) => {
                let form = package.form_refs.get(form_ref)?;
                let tail = package.decoder_bytes.get(form.decoder_ref as usize..)?;
                let length = tail.iter().position(|byte| *byte == 0)?;
                std::str::from_utf8(&tail[..length]).ok().map(Cow::Borrowed)
            }
            Self::Compact(package) => package.surface(form_ref).map(Cow::Owned),
        }
    }

    pub(super) fn binding(&self, index: usize) -> Option<MorphBinding> {
        match self {
            Self::Reference(package) => package.morph_bindings.get(index).copied(),
            Self::Compact(package) => package.binding(index),
        }
    }

    pub(super) fn lemma_centers(&self) -> &[LemmaCenter] {
        match self {
            Self::Reference(package) => &package.lemma_centers,
            Self::Compact(package) => package.lemma_centers(),
        }
    }

    pub(super) fn context_modes(&self) -> &[LocalContextMode] {
        match self {
            Self::Reference(package) => &package.context_modes,
            Self::Compact(package) => package.context_modes(),
        }
    }

    pub(super) fn slot_centers(&self) -> &[SlotPhaseCenter] {
        match self {
            Self::Reference(package) => &package.slot_centers,
            Self::Compact(package) => package.slot_centers(),
        }
    }

    pub(super) fn neighbor_couplings(&self) -> &[NeighborCoupling] {
        match self {
            Self::Reference(package) => &package.neighbor_couplings,
            Self::Compact(package) => package.neighbor_couplings(),
        }
    }

    pub(super) fn competition_edges(&self) -> &[CompetitionEdge] {
        match self {
            Self::Reference(package) => &package.competition_edges,
            Self::Compact(package) => package.competition_edges(),
        }
    }

    pub(super) fn calibration(&self) -> TieCalibration {
        match self {
            Self::Reference(package) => package.calibration,
            Self::Compact(package) => package.calibration(),
        }
    }
}
