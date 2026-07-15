//! Hot runtime field contract.
//!
//! The hot input path must read compact field state and tiny renderer metadata.
//! Full dictionaries, generated forms and full NANDA traces are cold/reference
//! authority. They may build or verify the field, but they must not become the
//! live daemon's default memory object.

use crate::text_backend::TextBackendPreference;
use std::sync::atomic::{AtomicU8, Ordering};

const ROUTE_DAEMON: u8 = 0;
const ROUTE_IME: u8 = 1;
const AUTHORITY_FIELD_SNAPSHOT_ONLY: u8 = 0;
const AUTHORITY_FULL_REFERENCE_ALLOWED: u8 = 1;

static PROCESS_ROUTE: AtomicU8 = AtomicU8::new(ROUTE_DAEMON);
static PROCESS_AUTHORITY: AtomicU8 = AtomicU8::new(AUTHORITY_FULL_REFERENCE_ALLOWED);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotRuntimeRoute {
    Daemon,
    Ime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotAuthority {
    FieldSnapshotOnly,
    FullReferenceAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotFieldPolicy {
    route: HotRuntimeRoute,
    authority: HotAuthority,
}

impl HotFieldPolicy {
    pub fn daemon_for_text_backend(backend: TextBackendPreference) -> Self {
        let authority = if backend.should_try_ime() {
            HotAuthority::FieldSnapshotOnly
        } else {
            HotAuthority::FullReferenceAllowed
        };
        Self {
            route: HotRuntimeRoute::Daemon,
            authority,
        }
    }

    pub const fn ime() -> Self {
        Self {
            route: HotRuntimeRoute::Ime,
            authority: HotAuthority::FieldSnapshotOnly,
        }
    }

    pub const fn route(self) -> HotRuntimeRoute {
        self.route
    }

    pub const fn authority(self) -> HotAuthority {
        self.authority
    }

    pub const fn allows_full_reference_authority(self) -> bool {
        matches!(self.authority, HotAuthority::FullReferenceAllowed)
    }

    pub const fn allows_full_nanda_authority(self) -> bool {
        self.allows_full_reference_authority()
    }
}

pub fn set_process_policy(policy: HotFieldPolicy) {
    PROCESS_ROUTE.store(encode_route(policy.route), Ordering::Relaxed);
    PROCESS_AUTHORITY.store(encode_authority(policy.authority), Ordering::Relaxed);
}

pub fn process_policy() -> HotFieldPolicy {
    HotFieldPolicy {
        route: decode_route(PROCESS_ROUTE.load(Ordering::Relaxed)),
        authority: decode_authority(PROCESS_AUTHORITY.load(Ordering::Relaxed)),
    }
}

pub fn process_allows_full_reference_authority() -> bool {
    process_policy().allows_full_reference_authority()
}

const fn encode_route(route: HotRuntimeRoute) -> u8 {
    match route {
        HotRuntimeRoute::Daemon => ROUTE_DAEMON,
        HotRuntimeRoute::Ime => ROUTE_IME,
    }
}

const fn decode_route(value: u8) -> HotRuntimeRoute {
    match value {
        ROUTE_IME => HotRuntimeRoute::Ime,
        _ => HotRuntimeRoute::Daemon,
    }
}

const fn encode_authority(authority: HotAuthority) -> u8 {
    match authority {
        HotAuthority::FieldSnapshotOnly => AUTHORITY_FIELD_SNAPSHOT_ONLY,
        HotAuthority::FullReferenceAllowed => AUTHORITY_FULL_REFERENCE_ALLOWED,
    }
}

const fn decode_authority(value: u8) -> HotAuthority {
    match value {
        AUTHORITY_FULL_REFERENCE_ALLOWED => HotAuthority::FullReferenceAllowed,
        _ => HotAuthority::FieldSnapshotOnly,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotWordAuthority {
    Unknown,
    CommonSurface,
    L2SurfaceCenter,
    L2FormCenter,
    UserUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotWordReadout {
    pub authority: HotWordAuthority,
}

impl HotWordReadout {
    pub const fn is_known(self) -> bool {
        !matches!(self.authority, HotWordAuthority::Unknown)
    }

    pub(crate) const fn has_structural_center(self) -> bool {
        matches!(
            self.authority,
            HotWordAuthority::CommonSurface
                | HotWordAuthority::L2SurfaceCenter
                | HotWordAuthority::L2FormCenter
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HotSurfacePhaseReadout {
    pub(crate) exact_center: bool,
    pub(crate) l1_refs: usize,
    pub(crate) motif_refs: usize,
    pub(crate) covered_l1_refs: usize,
    pub(crate) residual_l1_refs: usize,
    pub(crate) coherence_milli: u32,
}

impl HotSurfacePhaseReadout {
    const MIN_FORM_L1_REFS: usize = 8;
    const MIN_FORM_MOTIFS: usize = 2;
    const MIN_FORM_COHERENCE_MILLI: u32 = 620;

    pub(crate) const fn settles_as_form(self) -> bool {
        self.exact_center
            || (self.l1_refs >= Self::MIN_FORM_L1_REFS
                && self.motif_refs >= Self::MIN_FORM_MOTIFS
                && self.coherence_milli >= Self::MIN_FORM_COHERENCE_MILLI)
    }

    pub(crate) const fn transition_mass(self) -> u32 {
        self.coherence_milli
            .saturating_add((self.motif_refs as u32).saturating_mul(40))
            .saturating_add(if self.exact_center { 240 } else { 0 })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HotBoundaryShiftReadout {
    pub(crate) original_left: HotSurfacePhaseReadout,
    pub(crate) original_right: HotSurfacePhaseReadout,
    pub(crate) candidate_left: HotSurfacePhaseReadout,
    pub(crate) candidate_right: HotSurfacePhaseReadout,
    pub(crate) candidate_left_form: HotWordReadout,
    pub(crate) candidate_right_form: HotWordReadout,
}

impl HotBoundaryShiftReadout {
    const MIN_DIRECT_MASS_GAIN: u32 = 150;

    pub(crate) const fn candidate_settles(self) -> bool {
        (self.candidate_left_form.has_structural_center() || self.candidate_left.settles_as_form())
            && (self.candidate_right_form.has_structural_center()
                || self.candidate_right.settles_as_form())
    }

    pub(crate) const fn mass_gain(self) -> u32 {
        self.candidate_left
            .transition_mass()
            .saturating_add(self.candidate_right.transition_mass())
            .saturating_sub(
                self.original_left
                    .transition_mass()
                    .saturating_add(self.original_right.transition_mass()),
            )
    }

    pub(crate) const fn has_direct_apply_mass(self) -> bool {
        self.candidate_settles()
            && !self.original_right.exact_center
            && self.mass_gain() >= Self::MIN_DIRECT_MASS_GAIN
    }
}

#[derive(Debug, Default)]
pub struct HotFieldSnapshot;

impl HotFieldSnapshot {
    pub const fn current() -> Self {
        Self
    }

    pub fn word_readout(&self, word: &str) -> HotWordReadout {
        let lower = word.trim().to_lowercase();
        let authority = if lower.is_empty() {
            HotWordAuthority::Unknown
        } else if crate::lexicon::is_common_ru_word(&lower) {
            HotWordAuthority::CommonSurface
        } else if crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower) {
            HotWordAuthority::L2SurfaceCenter
        } else if accepted_usage_has_authority(&lower) {
            HotWordAuthority::UserUsage
        } else {
            HotWordAuthority::Unknown
        };
        HotWordReadout { authority }
    }

    /// Reads a surface that may be reconstructed from a compact morphology
    /// center even when that exact inflected form is not a stored hot surface.
    pub(crate) fn form_readout(&self, word: &str) -> HotWordReadout {
        let surface = self.word_readout(word);
        if surface.has_structural_center() {
            return surface;
        }
        let lower = word.trim().to_lowercase();
        let phase = self.surface_phase_readout(word);
        let authority = if phase.exact_center {
            HotWordAuthority::L2SurfaceCenter
        } else if crate::nanda_wave::l2::l2_decoder_contains_surface(&lower)
            || crate::russian_lexicon::is_center_backed_russian_form(&lower)
            || crate::russian_lexicon::is_reference_backed_russian_form(&lower)
            || phase.settles_as_form()
        {
            HotWordAuthority::L2FormCenter
        } else {
            surface.authority
        };
        HotWordReadout { authority }
    }

    /// Stable input authority is stricter than candidate-form settlement. A
    /// nearby phase basin may propose a form, but only an exact surface or a
    /// morphology transition backed by a lexical center may protect the input
    /// from correction.
    pub(crate) fn stable_form_readout(&self, word: &str) -> HotWordReadout {
        let surface = self.word_readout(word);
        if surface.has_structural_center() {
            return surface;
        }
        let lower = word.trim().to_lowercase();
        let phase = self.surface_phase_readout(word);
        let authority = if phase.exact_center {
            HotWordAuthority::L2SurfaceCenter
        } else if crate::nanda_wave::l2::l2_decoder_contains_surface(&lower)
            || crate::russian_lexicon::is_center_backed_russian_form(&lower)
            || crate::russian_lexicon::is_reference_backed_russian_form(&lower)
        {
            HotWordAuthority::L2FormCenter
        } else {
            surface.authority
        };
        HotWordReadout { authority }
    }

    pub(crate) fn surface_phase_readout(&self, word: &str) -> HotSurfacePhaseReadout {
        let readout = crate::nanda_wave::l2::l2_surface_phase_readout(word);
        HotSurfacePhaseReadout {
            exact_center: readout.exact_center,
            l1_refs: readout.l1_refs,
            motif_refs: readout.motif_refs,
            covered_l1_refs: readout.covered_l1_refs,
            residual_l1_refs: readout.residual_l1_refs,
            coherence_milli: readout.coherence_milli(),
        }
    }

    pub(crate) fn boundary_shift_readout(
        &self,
        original_left: &str,
        original_right: &str,
        candidate_left: &str,
        candidate_right: &str,
    ) -> HotBoundaryShiftReadout {
        HotBoundaryShiftReadout {
            original_left: self.surface_phase_readout(original_left),
            original_right: self.surface_phase_readout(original_right),
            candidate_left: self.surface_phase_readout(candidate_left),
            candidate_right: self.surface_phase_readout(candidate_right),
            candidate_left_form: self.form_readout(candidate_left),
            candidate_right_form: self.form_readout(candidate_right),
        }
    }
}

fn accepted_usage_has_authority(word: &str) -> bool {
    let usage = crate::nanda_wave::cached_usage_prior_snapshot();
    let readout = usage.hot_readout(&[], "*", "*", "*", word);
    readout.accepted_count >= 2 && readout.accepted_count > readout.rejected_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_auto_backend_uses_field_snapshot_only() {
        let policy = HotFieldPolicy::daemon_for_text_backend(TextBackendPreference::Auto);
        assert_eq!(policy.route(), HotRuntimeRoute::Daemon);
        assert_eq!(policy.authority(), HotAuthority::FieldSnapshotOnly);
        assert!(!policy.allows_full_nanda_authority());
    }

    #[test]
    fn daemon_uinput_backend_can_use_full_reference_authority() {
        let policy = HotFieldPolicy::daemon_for_text_backend(TextBackendPreference::Uinput);
        assert_eq!(policy.authority(), HotAuthority::FullReferenceAllowed);
        assert!(policy.allows_full_nanda_authority());
    }

    #[test]
    fn process_policy_tracks_hot_authority() {
        let original = process_policy();
        set_process_policy(HotFieldPolicy::ime());

        assert_eq!(process_policy().route(), HotRuntimeRoute::Ime);
        assert_eq!(
            process_policy().authority(),
            HotAuthority::FieldSnapshotOnly
        );
        assert!(!process_allows_full_reference_authority());

        set_process_policy(original);
    }

    #[test]
    fn hot_word_readout_does_not_need_full_dictionary_for_common_words() {
        let snapshot = HotFieldSnapshot::current();
        assert!(snapshot.word_readout("это").is_known());
    }

    #[test]
    fn reference_surface_is_not_terminal_known_word() {
        let snapshot = HotFieldSnapshot::current();
        let readout = snapshot.word_readout("пров");
        assert_eq!(readout.authority, HotWordAuthority::Unknown);
        assert!(!readout.is_known());
    }

    #[test]
    fn exact_surface_and_reconstructed_form_are_distinct_readouts() {
        let snapshot = HotFieldSnapshot::current();
        assert_eq!(
            snapshot.word_readout("набирать").authority,
            HotWordAuthority::L2SurfaceCenter
        );
        assert_eq!(
            snapshot.form_readout("допустим").authority,
            HotWordAuthority::L2SurfaceCenter
        );
        assert_eq!(
            snapshot.form_readout("набираю").authority,
            HotWordAuthority::L2FormCenter
        );
        assert!(!snapshot.word_readout("мнабираю").is_known());
    }
}
