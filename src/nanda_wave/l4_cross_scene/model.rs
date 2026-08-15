use super::super::phase_field::{PhaseCell, PhaseCenter};
use crate::transition_relation::TransitionOperatorKind;
use crate::typing_memory::{LayoutProjectionDirection, LayoutProjectionScope, TypingMemoryOutcome};
use crate::typing_scene::{
    LanguageId, LanguageSceneIdentity, SceneSymbol, SentenceLanguageEvidence,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum L4CrossSceneContextSignal {
    #[default]
    Unknown = 0,
    Neutral = 1,
    Support = 2,
    Suppress = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i8)]
pub(crate) enum L4CrossSceneL2Signal {
    Repel = -1,
    #[default]
    Unknown = 0,
    Support = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct L4CrossSceneProfileKey {
    pub(crate) operator: TransitionOperatorKind,
    pub(crate) direction: Option<LayoutProjectionDirection>,
    pub(crate) scope: Option<LayoutProjectionScope>,
    pub(crate) scene: LanguageSceneIdentity,
    pub(crate) sentence_language: LanguageId,
    pub(crate) sentence_evidence_bucket: u8,
}

impl L4CrossSceneProfileKey {
    pub(crate) fn new(
        operator: TransitionOperatorKind,
        direction: Option<LayoutProjectionDirection>,
        scope: Option<LayoutProjectionScope>,
    ) -> Self {
        Self {
            operator,
            direction,
            scope,
            scene: LanguageSceneIdentity::default(),
            sentence_language: LanguageId::UNKNOWN,
            sentence_evidence_bucket: 0,
        }
    }

    pub(crate) const fn with_scene(
        mut self,
        scene: LanguageSceneIdentity,
        sentence_language: SentenceLanguageEvidence,
    ) -> Self {
        self.scene = scene;
        self.sentence_language = sentence_language.language;
        self.sentence_evidence_bucket = sentence_language.profile_bucket();
        self
    }

    pub(crate) const fn legacy_v1(self) -> Self {
        Self {
            operator: self.operator,
            direction: self.direction,
            scope: self.scope,
            scene: LanguageSceneIdentity {
                source_language: LanguageId::UNKNOWN,
                target_language: LanguageId::UNKNOWN,
                source_layout: crate::typing_scene::LayoutId::UNKNOWN,
                target_layout: crate::typing_scene::LayoutId::UNKNOWN,
                source_script: crate::typing_scene::ScriptFamily::Unknown,
                target_script: crate::typing_scene::ScriptFamily::Unknown,
                keyboard_geometry: crate::typing_scene::KeyboardGeometryId::UNKNOWN,
                evidence: crate::typing_scene::SceneIdentityEvidence::Unknown,
            },
            sentence_language: LanguageId::UNKNOWN,
            sentence_evidence_bucket: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct L4CrossSceneInput<'a> {
    pub(crate) profile: L4CrossSceneProfileKey,
    pub(crate) context: &'a [String],
    pub(crate) from_text: &'a str,
    pub(crate) to_text: &'a str,
    pub(crate) relation_atoms: &'a [String],
    pub(crate) candidate_relation_id: u64,
    pub(crate) keep_relation_id: u64,
    pub(crate) l3_relation_class: u64,
    pub(crate) context_signal: L4CrossSceneContextSignal,
    pub(crate) l2_signal: L4CrossSceneL2Signal,
    pub(crate) sentence_language: SentenceLanguageEvidence,
}

#[derive(Clone, Debug)]
pub(crate) struct EncodedL4Scene {
    pub(crate) vector: Vec<PhaseCell>,
    pub(crate) fingerprint: u64,
    pub(crate) candidate_relation_id: u64,
    pub(crate) keep_relation_id: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct L4CrossSceneObservation {
    pub(crate) receipt_id: u64,
    pub(crate) complete_chain: bool,
    pub(crate) profile: L4CrossSceneProfileKey,
    pub(crate) context: Vec<String>,
    pub(crate) from_text: String,
    pub(crate) to_text: String,
    pub(crate) relation_atoms: Vec<String>,
    pub(crate) candidate_relation_id: u64,
    pub(crate) keep_relation_id: u64,
    pub(crate) l3_relation_class: u64,
    pub(crate) context_signal: L4CrossSceneContextSignal,
    pub(crate) l2_signal: L4CrossSceneL2Signal,
    pub(crate) sentence_language: SentenceLanguageEvidence,
    pub(crate) scene_symbols: Vec<SceneSymbol>,
    pub(crate) outcome: TypingMemoryOutcome,
}

impl L4CrossSceneObservation {
    pub(crate) fn input(&self) -> L4CrossSceneInput<'_> {
        L4CrossSceneInput {
            profile: self.profile,
            context: &self.context,
            from_text: &self.from_text,
            to_text: &self.to_text,
            relation_atoms: &self.relation_atoms,
            candidate_relation_id: self.candidate_relation_id,
            keep_relation_id: self.keep_relation_id,
            l3_relation_class: self.l3_relation_class,
            context_signal: self.context_signal,
            l2_signal: self.l2_signal,
            sentence_language: self.sentence_language,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct L4CrossSceneProfile {
    pub(crate) key: L4CrossSceneProfileKey,
    pub(crate) threshold_micro: i32,
    pub(crate) positive: Vec<PhaseCenter>,
    pub(crate) negative: Vec<PhaseCenter>,
    pub(crate) hard_negative: Vec<PhaseCenter>,
    pub(crate) ambiguity: Vec<PhaseCenter>,
    pub(crate) positive_examples: u32,
    pub(crate) negative_examples: u32,
    pub(crate) reverted_examples: u32,
    pub(crate) ambiguity_examples: u32,
    pub(crate) censored_examples: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct L4CrossScenePairProfile {
    pub(crate) key: L4CrossSceneProfileKey,
    pub(crate) low_relation: u64,
    pub(crate) high_relation: u64,
    pub(crate) threshold_micro: i32,
    pub(crate) low_wins: Vec<PhaseCenter>,
    pub(crate) high_wins: Vec<PhaseCenter>,
    pub(crate) hard_low_wins: Vec<PhaseCenter>,
    pub(crate) hard_high_wins: Vec<PhaseCenter>,
    pub(crate) ambiguity: Vec<PhaseCenter>,
    pub(crate) observations: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct L4CrossScenePackage {
    pub(crate) encoder_version: u32,
    pub(crate) encoder_hash: u64,
    pub(crate) applied_segment: u64,
    pub(crate) symbols: Vec<SceneSymbol>,
    pub(crate) profiles: Vec<L4CrossSceneProfile>,
    pub(crate) pair_profiles: Vec<L4CrossScenePairProfile>,
    pub(crate) source_observations: u32,
    pub(crate) joined_observations: u32,
    pub(crate) positive_observations: u32,
    pub(crate) negative_observations: u32,
    pub(crate) reverted_observations: u32,
    pub(crate) ambiguity_observations: u32,
    pub(crate) censored_observations: u32,
}

impl Default for L4CrossScenePackage {
    fn default() -> Self {
        Self {
            encoder_version: super::ENCODER_VERSION,
            encoder_hash: super::ENCODER_HASH,
            applied_segment: 0,
            symbols: Vec::new(),
            profiles: Vec::new(),
            pair_profiles: Vec::new(),
            source_observations: 0,
            joined_observations: 0,
            positive_observations: 0,
            negative_observations: 0,
            reverted_observations: 0,
            ambiguity_observations: 0,
            censored_observations: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum L4CrossSceneDisposition {
    Supported,
    Repelled,
    Ambiguous,
    #[default]
    Unknown,
}

impl L4CrossSceneDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Repelled => "repelled",
            Self::Ambiguous => "ambiguous",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum L4CrossSceneRecommendation {
    SuggestOnly,
    #[default]
    Keep,
}

impl L4CrossSceneRecommendation {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::SuggestOnly => "suggest_only",
            Self::Keep => "keep",
        }
    }

    pub(crate) const fn automatic_apply(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct L4CrossSceneReadout {
    pub(crate) package_loaded: bool,
    pub(crate) profile_present: bool,
    pub(crate) disposition: L4CrossSceneDisposition,
    pub(crate) recommendation: L4CrossSceneRecommendation,
    pub(crate) margin_milli: i16,
    pub(crate) threshold_milli: i16,
    pub(crate) positive_milli: i16,
    pub(crate) negative_milli: i16,
    pub(crate) hard_negative_milli: i16,
    pub(crate) ambiguity_milli: i16,
    pub(crate) pair_margin_milli: i16,
    pub(crate) positive_centers: u8,
    pub(crate) negative_centers: u8,
    pub(crate) hard_negative_centers: u8,
    pub(crate) ambiguity_centers: u8,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub(crate) struct CrossSceneCompileReport {
    pub(crate) source_observations: u32,
    pub(crate) live_source_observations: u32,
    pub(crate) backfilled_revert_receipts: u32,
    pub(crate) backfilled_revert_observations: u32,
    pub(crate) joined_observations: u32,
    pub(crate) orphan_observations: u32,
    pub(crate) ignored_observations: u32,
    pub(crate) invalid_observations: u32,
    pub(crate) positive_observations: u32,
    pub(crate) negative_observations: u32,
    pub(crate) reverted_observations: u32,
    pub(crate) ambiguity_observations: u32,
    pub(crate) censored_observations: u32,
    pub(crate) consolidated_scenes: u32,
    pub(crate) conflict_scenes: u32,
    pub(crate) profiles: u32,
    pub(crate) pair_profiles: u32,
    pub(crate) symbols: u32,
    pub(crate) logical_center_bytes: u64,
    pub(crate) raw_text_stored: bool,
    pub(crate) runtime_authority_changed: bool,
}
