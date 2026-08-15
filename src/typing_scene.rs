//! Typed language-scene identities shared by typing memory and L4.
//!
//! Language, keyboard layout, Unicode script, and physical keyboard geometry
//! are deliberately separate. The compact identifiers are hashes of canonical
//! package labels; packages retain the labels and reject identity collisions.

const LANGUAGE_DOMAIN: u64 = 0x4c41_4e47_5541_4745;
const LAYOUT_DOMAIN: u64 = 0x4c41_594f_5554_5f49;
const GEOMETRY_DOMAIN: u64 = 0x4b45_5947_454f_4d45;

macro_rules! scene_id {
    ($name:ident, $domain:ident, $normalizer:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) struct $name(u64);

        impl $name {
            pub(crate) const UNKNOWN: Self = Self(0);

            pub(crate) fn from_label(label: &str) -> Option<Self> {
                let canonical = $normalizer(label)?;
                Some(Self(symbol_id($domain, &canonical)))
            }

            pub(crate) const fn from_canonical_static(label: &str) -> Self {
                Self(symbol_id_const($domain, label.as_bytes()))
            }

            pub(crate) const fn code(self) -> u64 {
                self.0
            }

            pub(crate) const fn from_code(code: u64) -> Self {
                Self(code)
            }

            pub(crate) const fn is_unknown(self) -> bool {
                self.0 == 0
            }
        }
    };
}

scene_id!(LanguageId, LANGUAGE_DOMAIN, canonical_language_label);
scene_id!(LayoutId, LAYOUT_DOMAIN, canonical_layout_label);
scene_id!(
    KeyboardGeometryId,
    GEOMETRY_DOMAIN,
    canonical_geometry_label
);

impl LanguageId {
    pub(crate) const ENGLISH: Self = Self::from_canonical_static("en");
    pub(crate) const RUSSIAN: Self = Self::from_canonical_static("ru");

    pub(crate) const fn known_label(self) -> Option<&'static str> {
        if self.0 == Self::ENGLISH.0 {
            Some("en")
        } else if self.0 == Self::RUSSIAN.0 {
            Some("ru")
        } else {
            None
        }
    }
}

impl LayoutId {
    pub(crate) const XKB_US: Self = Self::from_canonical_static("xkb:us");
    pub(crate) const XKB_RU: Self = Self::from_canonical_static("xkb:ru");

    pub(crate) const fn known_label(self) -> Option<&'static str> {
        if self.0 == Self::XKB_US.0 {
            Some("xkb:us")
        } else if self.0 == Self::XKB_RU.0 {
            Some("xkb:ru")
        } else {
            None
        }
    }
}

impl KeyboardGeometryId {
    pub(crate) const PC105: Self = Self::from_canonical_static("pc105");

    pub(crate) const fn known_label(self) -> Option<&'static str> {
        if self.0 == Self::PC105.0 {
            Some("pc105")
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum ScriptFamily {
    #[default]
    Unknown = 0,
    Latin = 1,
    Cyrillic = 2,
    Greek = 3,
    Armenian = 4,
    Georgian = 5,
    Hebrew = 6,
    Arabic = 7,
    Han = 8,
    Kana = 9,
    Hangul = 10,
    Mixed = 11,
    OtherAlphabetic = 12,
}

impl ScriptFamily {
    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0 => Self::Unknown,
            1 => Self::Latin,
            2 => Self::Cyrillic,
            3 => Self::Greek,
            4 => Self::Armenian,
            5 => Self::Georgian,
            6 => Self::Hebrew,
            7 => Self::Arabic,
            8 => Self::Han,
            9 => Self::Kana,
            10 => Self::Hangul,
            11 => Self::Mixed,
            12 => Self::OtherAlphabetic,
            _ => return None,
        })
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Latin => "latin",
            Self::Cyrillic => "cyrillic",
            Self::Greek => "greek",
            Self::Armenian => "armenian",
            Self::Georgian => "georgian",
            Self::Hebrew => "hebrew",
            Self::Arabic => "arabic",
            Self::Han => "han",
            Self::Kana => "kana",
            Self::Hangul => "hangul",
            Self::Mixed => "mixed",
            Self::OtherAlphabetic => "other_alphabetic",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "unknown" => Self::Unknown,
            "latin" => Self::Latin,
            "cyrillic" => Self::Cyrillic,
            "greek" => Self::Greek,
            "armenian" => Self::Armenian,
            "georgian" => Self::Georgian,
            "hebrew" => Self::Hebrew,
            "arabic" => Self::Arabic,
            "han" => Self::Han,
            "kana" => Self::Kana,
            "hangul" => Self::Hangul,
            "mixed" => Self::Mixed,
            "other_alphabetic" => Self::OtherAlphabetic,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum SceneIdentityEvidence {
    #[default]
    Unknown = 0,
    ScriptOnly = 1,
    LegacyRuEnAdapter = 2,
    Package = 3,
}

impl SceneIdentityEvidence {
    pub(crate) const fn code(self) -> u8 {
        self as u8
    }

    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0 => Self::Unknown,
            1 => Self::ScriptOnly,
            2 => Self::LegacyRuEnAdapter,
            3 => Self::Package,
            _ => return None,
        })
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::ScriptOnly => "script_only",
            Self::LegacyRuEnAdapter => "legacy_ru_en_adapter",
            Self::Package => "package",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "unknown" => Self::Unknown,
            "script_only" => Self::ScriptOnly,
            "legacy_ru_en_adapter" => Self::LegacyRuEnAdapter,
            "package" => Self::Package,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LanguageSceneIdentity {
    pub(crate) source_language: LanguageId,
    pub(crate) target_language: LanguageId,
    pub(crate) source_layout: LayoutId,
    pub(crate) target_layout: LayoutId,
    pub(crate) source_script: ScriptFamily,
    pub(crate) target_script: ScriptFamily,
    pub(crate) keyboard_geometry: KeyboardGeometryId,
    pub(crate) evidence: SceneIdentityEvidence,
}

impl LanguageSceneIdentity {
    pub(crate) fn observed(from: &str, to: &str) -> Self {
        let source_script = script_family(from);
        let target_script = script_family(to);
        Self {
            source_script,
            target_script,
            evidence: if source_script == ScriptFamily::Unknown
                && target_script == ScriptFamily::Unknown
            {
                SceneIdentityEvidence::Unknown
            } else {
                SceneIdentityEvidence::ScriptOnly
            },
            ..Self::default()
        }
    }

    pub(crate) fn with_legacy_ru_en_layout(
        mut self,
        source_layout: LayoutId,
        target_layout: LayoutId,
        target_language: LanguageId,
    ) -> Self {
        self.source_layout = source_layout;
        self.target_layout = target_layout;
        self.target_language = target_language;
        self.keyboard_geometry = KeyboardGeometryId::PC105;
        self.evidence = SceneIdentityEvidence::LegacyRuEnAdapter;
        self
    }

    pub(crate) fn known_symbols(self) -> Vec<SceneSymbol> {
        let mut symbols = Vec::with_capacity(5);
        for language in [self.source_language, self.target_language] {
            if let Some(label) = language.known_label() {
                symbols.push(SceneSymbol::language(label).expect("known language label"));
            }
        }
        for layout in [self.source_layout, self.target_layout] {
            if let Some(label) = layout.known_label() {
                symbols.push(SceneSymbol::layout(label).expect("known layout label"));
            }
        }
        if let Some(label) = self.keyboard_geometry.known_label() {
            symbols.push(
                SceneSymbol::keyboard_geometry(label).expect("known keyboard geometry label"),
            );
        }
        symbols.sort();
        symbols.dedup();
        symbols
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SentenceLanguageEvidence {
    pub(crate) language: LanguageId,
    pub(crate) support_milli: u16,
    pub(crate) alternative_milli: u16,
    pub(crate) observed_tokens: u8,
}

impl SentenceLanguageEvidence {
    pub(crate) fn script_only(context: &[String], target: &str) -> Self {
        let target_script = script_family(target);
        if target_script == ScriptFamily::Unknown || target_script == ScriptFamily::Mixed {
            return Self::default();
        }
        let mut matching = 0_u16;
        let mut alternative = 0_u16;
        for token in context.iter().rev().take(8) {
            let script = script_family(token);
            if script == ScriptFamily::Unknown || script == ScriptFamily::Mixed {
                continue;
            }
            if script == target_script {
                matching = matching.saturating_add(1);
            } else {
                alternative = alternative.saturating_add(1);
            }
        }
        let observed = matching.saturating_add(alternative);
        if observed == 0 {
            return Self::default();
        }
        Self {
            language: LanguageId::UNKNOWN,
            support_milli: matching.saturating_mul(1_000) / observed,
            alternative_milli: alternative.saturating_mul(1_000) / observed,
            observed_tokens: observed.min(u8::MAX as u16) as u8,
        }
    }

    pub(crate) const fn profile_bucket(self) -> u8 {
        if self.observed_tokens == 0 {
            0
        } else if self.support_milli >= 750 && self.support_milli > self.alternative_milli {
            1
        } else if self.support_milli > self.alternative_milli {
            2
        } else if self.alternative_milli >= 750 && self.alternative_milli > self.support_milli {
            5
        } else if self.alternative_milli > self.support_milli {
            4
        } else {
            3
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum SceneSymbolKind {
    Language = 1,
    Layout = 2,
    KeyboardGeometry = 3,
}

impl SceneSymbolKind {
    pub(crate) const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::Language,
            2 => Self::Layout,
            3 => Self::KeyboardGeometry,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SceneSymbol {
    pub(crate) kind: SceneSymbolKind,
    pub(crate) id: u64,
    pub(crate) label: String,
}

impl SceneSymbol {
    pub(crate) fn language(label: &str) -> Option<Self> {
        let label = canonical_language_label(label)?;
        Some(Self {
            kind: SceneSymbolKind::Language,
            id: LanguageId::from_label(&label)?.code(),
            label,
        })
    }

    pub(crate) fn layout(label: &str) -> Option<Self> {
        let label = canonical_layout_label(label)?;
        Some(Self {
            kind: SceneSymbolKind::Layout,
            id: LayoutId::from_label(&label)?.code(),
            label,
        })
    }

    pub(crate) fn keyboard_geometry(label: &str) -> Option<Self> {
        let label = canonical_geometry_label(label)?;
        Some(Self {
            kind: SceneSymbolKind::KeyboardGeometry,
            id: KeyboardGeometryId::from_label(&label)?.code(),
            label,
        })
    }

    pub(crate) fn validate(&self) -> bool {
        let expected = match self.kind {
            SceneSymbolKind::Language => Self::language(&self.label),
            SceneSymbolKind::Layout => Self::layout(&self.label),
            SceneSymbolKind::KeyboardGeometry => Self::keyboard_geometry(&self.label),
        };
        expected.is_some_and(|expected| expected.id == self.id)
    }
}

pub(crate) fn script_family(text: &str) -> ScriptFamily {
    let mut family = ScriptFamily::Unknown;
    for ch in text.chars() {
        let current = char_script_family(ch);
        if current == ScriptFamily::Unknown {
            continue;
        }
        if family == ScriptFamily::Unknown {
            family = current;
        } else if family != current {
            return ScriptFamily::Mixed;
        }
    }
    family
}

fn char_script_family(ch: char) -> ScriptFamily {
    let code = ch as u32;
    match code {
        0x0041..=0x005a | 0x0061..=0x007a | 0x00c0..=0x02af | 0x1d00..=0x1d7f | 0x1e00..=0x1eff => {
            ScriptFamily::Latin
        }
        0x0400..=0x052f | 0x2de0..=0x2dff | 0xa640..=0xa69f => ScriptFamily::Cyrillic,
        0x0370..=0x03ff | 0x1f00..=0x1fff => ScriptFamily::Greek,
        0x0530..=0x058f => ScriptFamily::Armenian,
        0x10a0..=0x10ff | 0x2d00..=0x2d2f => ScriptFamily::Georgian,
        0x0590..=0x05ff | 0xfb1d..=0xfb4f => ScriptFamily::Hebrew,
        0x0600..=0x06ff | 0x0750..=0x077f | 0x08a0..=0x08ff => ScriptFamily::Arabic,
        0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff => ScriptFamily::Han,
        0x3040..=0x30ff | 0x31f0..=0x31ff => ScriptFamily::Kana,
        0x1100..=0x11ff | 0x3130..=0x318f | 0xac00..=0xd7af => ScriptFamily::Hangul,
        _ if ch.is_alphabetic() => ScriptFamily::OtherAlphabetic,
        _ => ScriptFamily::Unknown,
    }
}

pub(crate) fn canonical_language_label(value: &str) -> Option<String> {
    canonical_label(value, 35, true, false)
}

pub(crate) fn canonical_layout_label(value: &str) -> Option<String> {
    canonical_label(value, 63, false, true)
}

pub(crate) fn canonical_geometry_label(value: &str) -> Option<String> {
    canonical_label(value, 31, false, false)
}

fn canonical_label(
    value: &str,
    maximum_bytes: usize,
    underscore_to_hyphen: bool,
    allow_colon: bool,
) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > maximum_bytes || !trimmed.is_ascii() {
        return None;
    }
    let mut canonical = String::with_capacity(trimmed.len());
    for byte in trimmed.bytes() {
        let byte = byte.to_ascii_lowercase();
        let byte = if underscore_to_hyphen && byte == b'_' {
            b'-'
        } else {
            byte
        };
        let valid = byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'+')
            || (allow_colon && byte == b':');
        if !valid {
            return None;
        }
        canonical.push(byte as char);
    }
    Some(canonical)
}

fn symbol_id(domain: u64, canonical: &str) -> u64 {
    symbol_id_const(domain, canonical.as_bytes())
}

const fn symbol_id_const(domain: u64, bytes: &[u8]) -> u64 {
    let mut state = 0xcbf2_9ce4_8422_2325_u64 ^ domain;
    let mut index = 0;
    while index < bytes.len() {
        state = (state ^ bytes[index] as u64).wrapping_mul(0x0000_0100_0000_01b3);
        index += 1;
    }
    if state == 0 {
        1
    } else {
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_domains_are_stable_and_separate() {
        assert_eq!(LanguageId::from_label("RU"), Some(LanguageId::RUSSIAN));
        assert_eq!(LayoutId::from_label(" XKB:RU "), Some(LayoutId::XKB_RU));
        assert_eq!(
            KeyboardGeometryId::from_label("PC105"),
            Some(KeyboardGeometryId::PC105)
        );
        assert_ne!(LanguageId::RUSSIAN.code(), LayoutId::XKB_RU.code());
        assert!(LanguageId::from_label("ru / invalid").is_none());
    }

    #[test]
    fn script_family_does_not_collapse_latin_into_english() {
        assert_eq!(script_family("francais"), ScriptFamily::Latin);
        assert_eq!(script_family("Deutsch"), ScriptFamily::Latin);
        assert_eq!(script_family("русский"), ScriptFamily::Cyrillic);
        assert_eq!(script_family("abcрус"), ScriptFamily::Mixed);
        assert_eq!(script_family("123!?"), ScriptFamily::Unknown);
    }

    #[test]
    fn symbol_registry_entry_validates_domain_and_label() {
        let symbol = SceneSymbol::language("de-DE").unwrap();
        assert!(symbol.validate());
        assert_ne!(symbol.id, SceneSymbol::layout("de-de").unwrap().id);
    }

    #[test]
    fn script_only_sentence_evidence_never_claims_a_language() {
        let context = vec!["one".to_string(), "two".to_string()];
        let evidence = SentenceLanguageEvidence::script_only(&context, "three");
        assert_eq!(evidence.language, LanguageId::UNKNOWN);
        assert_eq!(evidence.support_milli, 1_000);
        assert_eq!(evidence.alternative_milli, 0);
        assert_eq!(evidence.observed_tokens, 2);
    }
}
