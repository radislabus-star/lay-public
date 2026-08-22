//! Warm-only exact US-QWERTY to Russian layout authority.
//!
//! This module certifies a closed raw-layout contour. It never ranks, mutates
//! text, initializes data on the input path, or performs spelling repair.

use std::sync::OnceLock;

use crate::keyboard::is_cyrillic_letter;
use crate::word_reader::split_last_ws_token;

const COMPONENT_MAPPING: &[u8] = b"lay-ime-us\0us\0lay-ime-ru\0ru\0";
const US_QWERTY_PROFILE: &[u8] = b"lay-ime-us\0layout=us\0profile=us-qwerty\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FactoryEngineProfile {
    UsQwerty,
    Ru,
    Unknown,
}

impl FactoryEngineProfile {
    pub fn from_component_name(name: &str) -> Self {
        match name {
            "lay-ime-us" => Self::UsQwerty,
            "lay-ime-ru" => Self::Ru,
            _ => Self::Unknown,
        }
    }

    pub const fn initial_layout_is_ru(self) -> bool {
        matches!(self, Self::Ru)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActiveDecoderLayout {
    Us,
    Ru,
}

impl ActiveDecoderLayout {
    pub const fn from_layout_is_ru(layout_is_ru: bool) -> Self {
        if layout_is_ru {
            Self::Ru
        } else {
            Self::Us
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactAuthoritySnapshot {
    factory_engine_profile: FactoryEngineProfile,
    component_layout_mapping_fingerprint: u64,
    source_layout_profile_fingerprint: u64,
    keyboard_map_fingerprint: u64,
    russian_terminal_fingerprint: u64,
    english_guard_fingerprint: u64,
    protection_policy_fingerprint: u64,
}

impl ExactAuthoritySnapshot {
    pub fn fingerprint(self) -> u64 {
        fingerprint_u64s(&[
            self.component_layout_mapping_fingerprint,
            self.source_layout_profile_fingerprint,
            self.keyboard_map_fingerprint,
            self.russian_terminal_fingerprint,
            self.english_guard_fingerprint,
            self.protection_policy_fingerprint,
        ])
    }

    pub(crate) const fn russian_terminal_fingerprint(self) -> u64 {
        self.russian_terminal_fingerprint
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactLayoutFrame {
    pub frame_revision: u64,
    pub frame_fingerprint: u64,
    pub observed_token: String,
    pub active_composition: bool,
    pub factory_engine_profile: FactoryEngineProfile,
    pub active_decoder_layout: ActiveDecoderLayout,
    pub authority_snapshot: Option<ExactAuthoritySnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExactLayoutCaseShape {
    Lower,
    Title,
    Upper,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactLayoutContourCertificate {
    frame_revision: u64,
    frame_fingerprint: u64,
    original_token: String,
    projected_token: String,
    original_text: String,
    replacement_text: String,
    case_shape: ExactLayoutCaseShape,
    authority_snapshot: ExactAuthoritySnapshot,
}

impl ExactLayoutContourCertificate {
    pub fn frame_revision(&self) -> u64 {
        self.frame_revision
    }

    pub fn replacement_text(&self) -> &str {
        &self.replacement_text
    }

    pub fn original_token(&self) -> &str {
        &self.original_token
    }

    pub fn projected_token(&self) -> &str {
        &self.projected_token
    }

    pub(crate) fn matches_candidate(&self, original: &str, replacement: &str) -> bool {
        self.original_text == original && self.replacement_text == replacement
    }

    pub fn matches_frame(&self, frame_revision: u64, frame_fingerprint: u64) -> bool {
        self.frame_revision == frame_revision && self.frame_fingerprint == frame_fingerprint
    }

    pub(crate) fn snapshot(&self) -> ExactAuthoritySnapshot {
        self.authority_snapshot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactAuthorityWarmReceipt {
    pub english_entries: usize,
    pub protection_entries: usize,
    pub resident_bytes: usize,
    pub authority_fingerprint: u64,
}

#[derive(Clone, Copy)]
struct WarmExactAuthority {
    keyboard_map_fingerprint: u64,
    russian_terminal_fingerprint: u64,
    english_guard_fingerprint: u64,
    protection_policy_fingerprint: u64,
    receipt: ExactAuthorityWarmReceipt,
}

static WARM_EXACT_AUTHORITY: OnceLock<Option<WarmExactAuthority>> = OnceLock::new();

pub fn warm_up_exact_layout_authority_for_ibus() -> Option<ExactAuthorityWarmReceipt> {
    WARM_EXACT_AUTHORITY
        .get_or_init(|| {
            let keyboard_map_fingerprint = crate::dict::warm_up_us_to_ru();
            let russian_terminal_fingerprint =
                crate::nanda_wave::warm_up_exact_layout_terminal_authority()?;
            let guard = crate::word_recognizer::warm_up_exact_layout_guard();
            let authority_fingerprint = fingerprint_u64s(&[
                fingerprint_bytes(COMPONENT_MAPPING),
                fingerprint_bytes(US_QWERTY_PROFILE),
                keyboard_map_fingerprint,
                russian_terminal_fingerprint,
                guard.english_fingerprint,
                guard.protection_fingerprint,
            ]);
            Some(WarmExactAuthority {
                keyboard_map_fingerprint,
                russian_terminal_fingerprint,
                english_guard_fingerprint: guard.english_fingerprint,
                protection_policy_fingerprint: guard.protection_fingerprint,
                receipt: ExactAuthorityWarmReceipt {
                    english_entries: guard.english_entries,
                    protection_entries: guard.protection_entries,
                    resident_bytes: guard.resident_bytes,
                    authority_fingerprint,
                },
            })
        })
        .as_ref()
        .map(|authority| authority.receipt)
}

pub fn exact_authority_snapshot_if_warm(
    factory_engine_profile: FactoryEngineProfile,
    active_decoder_layout: ActiveDecoderLayout,
) -> Option<ExactAuthoritySnapshot> {
    if factory_engine_profile != FactoryEngineProfile::UsQwerty
        || active_decoder_layout != ActiveDecoderLayout::Us
    {
        return None;
    }
    let authority = WARM_EXACT_AUTHORITY.get()?.as_ref()?;
    Some(ExactAuthoritySnapshot {
        factory_engine_profile,
        component_layout_mapping_fingerprint: fingerprint_bytes(COMPONENT_MAPPING),
        source_layout_profile_fingerprint: fingerprint_bytes(US_QWERTY_PROFILE),
        keyboard_map_fingerprint: authority.keyboard_map_fingerprint,
        russian_terminal_fingerprint: authority.russian_terminal_fingerprint,
        english_guard_fingerprint: authority.english_guard_fingerprint,
        protection_policy_fingerprint: authority.protection_policy_fingerprint,
    })
}

pub(crate) fn certify_closed_exact_layout(
    decision_text: &str,
    frame: &ExactLayoutFrame,
    auto_replace: bool,
    auto_switch_layout: bool,
) -> Option<ExactLayoutContourCertificate> {
    if !frame.active_composition
        || !auto_replace
        || !auto_switch_layout
        || frame.factory_engine_profile != FactoryEngineProfile::UsQwerty
        || frame.active_decoder_layout != ActiveDecoderLayout::Us
    {
        return None;
    }
    let snapshot = frame.authority_snapshot?;
    if snapshot.factory_engine_profile != FactoryEngineProfile::UsQwerty
        || exact_authority_snapshot_if_warm(
            frame.factory_engine_profile,
            frame.active_decoder_layout,
        ) != Some(snapshot)
    {
        return None;
    }

    let token = frame.observed_token.as_str();
    if token.chars().count() < 2
        || !crate::layout_autoswitch::is_ascii_layout_letter_surface(token)
        || !token
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
        || !token
            .chars()
            .last()
            .is_some_and(|character| character.is_ascii_alphabetic())
    {
        return None;
    }
    let case_shape = closed_case_shape(token)?;
    let source_lower = token.to_ascii_lowercase();
    if crate::word_recognizer::exact_english_word_if_warm(&source_lower)?
        || crate::word_recognizer::exact_ascii_protected_if_warm(token)?
    {
        return None;
    }

    let projected_token = crate::dict::convert_us_to_ru_if_warm(token)?;
    if projected_token == token
        || !projected_token.chars().all(is_cyrillic_letter)
        || crate::nanda_wave::exact_layout_terminal_contains_if_warm(
            &projected_token.to_lowercase(),
            snapshot.russian_terminal_fingerprint(),
        )? != true
    {
        return None;
    }

    let trimmed = decision_text.trim_end_matches(char::is_whitespace);
    let trailing = &decision_text[trimmed.len()..];
    let (prefix, observed) = split_last_ws_token(trimmed)?;
    if observed != token || trailing.chars().count() != 1 || trailing != " " {
        return None;
    }
    let replacement_text = format!("{prefix}{projected_token}{trailing}");
    Some(ExactLayoutContourCertificate {
        frame_revision: frame.frame_revision,
        frame_fingerprint: frame.frame_fingerprint,
        original_token: token.to_string(),
        projected_token,
        original_text: decision_text.to_string(),
        replacement_text,
        case_shape,
        authority_snapshot: snapshot,
    })
}

fn closed_case_shape(token: &str) -> Option<ExactLayoutCaseShape> {
    let letters = token
        .chars()
        .filter(char::is_ascii_alphabetic)
        .collect::<Vec<_>>();
    if letters
        .iter()
        .all(|character| character.is_ascii_lowercase())
    {
        return Some(ExactLayoutCaseShape::Lower);
    }
    if letters
        .iter()
        .all(|character| character.is_ascii_uppercase())
    {
        return Some(ExactLayoutCaseShape::Upper);
    }
    if letters
        .first()
        .is_some_and(|character| character.is_ascii_uppercase())
        && letters
            .iter()
            .skip(1)
            .all(|character| character.is_ascii_lowercase())
    {
        return Some(ExactLayoutCaseShape::Title);
    }
    None
}

fn fingerprint_u64s(values: &[u64]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for value in values {
        for byte in value.to_le_bytes() {
            digest ^= u64::from(byte);
            digest = digest.wrapping_mul(0x100_0000_01b3);
        }
    }
    digest
}

fn fingerprint_bytes(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100_0000_01b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_profile_never_treats_unknown_as_us() {
        assert_eq!(
            FactoryEngineProfile::from_component_name("lay-ime-us"),
            FactoryEngineProfile::UsQwerty
        );
        assert_eq!(
            FactoryEngineProfile::from_component_name("lay-ime-ru"),
            FactoryEngineProfile::Ru
        );
        assert_eq!(
            FactoryEngineProfile::from_component_name("lay-ime-custom"),
            FactoryEngineProfile::Unknown
        );
    }

    #[test]
    fn authority_snapshot_fingerprint_fault_matrix_is_fail_closed() {
        warm_up_exact_layout_authority_for_ibus().expect("warm exact-layout authority");
        let snapshot = exact_authority_snapshot_if_warm(
            FactoryEngineProfile::UsQwerty,
            ActiveDecoderLayout::Us,
        )
        .expect("complete exact-layout snapshot");
        let frame = |authority_snapshot| ExactLayoutFrame {
            frame_revision: 17,
            frame_fingerprint: 19,
            observed_token: "ghbdtn".to_string(),
            active_composition: true,
            factory_engine_profile: FactoryEngineProfile::UsQwerty,
            active_decoder_layout: ActiveDecoderLayout::Us,
            authority_snapshot: Some(authority_snapshot),
        };
        assert!(certify_closed_exact_layout("ghbdtn ", &frame(snapshot), true, true).is_some());

        let mut faults = Vec::new();
        let mut changed = snapshot;
        changed.component_layout_mapping_fingerprint ^= 1;
        faults.push(changed);
        let mut changed = snapshot;
        changed.source_layout_profile_fingerprint ^= 1;
        faults.push(changed);
        let mut changed = snapshot;
        changed.keyboard_map_fingerprint ^= 1;
        faults.push(changed);
        let mut changed = snapshot;
        changed.russian_terminal_fingerprint ^= 1;
        faults.push(changed);
        let mut changed = snapshot;
        changed.english_guard_fingerprint ^= 1;
        faults.push(changed);
        let mut changed = snapshot;
        changed.protection_policy_fingerprint ^= 1;
        faults.push(changed);

        for changed in faults {
            assert!(
                certify_closed_exact_layout("ghbdtn ", &frame(changed), true, true).is_none(),
                "a changed authority fingerprint must invalidate the certificate"
            );
        }
    }

    #[test]
    fn exact_snapshot_readout_is_warm_only_and_cannot_initialize_data() {
        let source = include_str!("exact_layout_authority.rs");
        let readout = source
            .split("pub fn exact_authority_snapshot_if_warm")
            .nth(1)
            .expect("warm-only snapshot readout")
            .split("pub(crate) fn certify_closed_exact_layout")
            .next()
            .expect("bounded snapshot readout body");

        assert!(readout.contains("WARM_EXACT_AUTHORITY.get()?"));
        for forbidden in ["get_or_init", "warm_up", "std::fs", "File::open", "mmap"] {
            assert!(
                !readout.contains(forbidden),
                "snapshot readout must not initialize or load data: {forbidden}"
            );
        }
    }
}

#[cfg(test)]
#[path = "exact_layout_authority/proof.rs"]
mod proof;
