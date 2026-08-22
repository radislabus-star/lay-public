use std::time::Instant;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use lay::config::{CorrectionSafety, LayConfig};
use lay::text_edit::{EditAction, SnapshotIdentity, TransitionOperator, VisibleTailSource};

#[derive(Debug, Clone)]
pub(crate) enum DeferredLayoutAction {
    BackgroundSwitch {
        previous_is_ru: bool,
        target_is_ru: bool,
        engine: &'static str,
    },
    BlockingSwitch {
        previous_is_ru: bool,
        target_is_ru: bool,
        engine: &'static str,
        activate_gnome: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum DeferredLearningAction {
    AcceptedLayoutProjection {
        original: String,
        replacement: String,
    },
    RevertedSystemApply {
        original: String,
        rejected: String,
        transition: lay::typing_cpu::ObservedSystemTransition,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypingAssistRuleIdentity {
    id: String,
    enabled: bool,
    priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct InputConfigIdentity {
    auto_replace: bool,
    typing_assist: bool,
    auto_switch_layout: bool,
    nanda_autocorrect: bool,
    correction_safety: CorrectionSafety,
    typing_assist_pipeline: Vec<TypingAssistRuleIdentity>,
    nanda_l2_weight_percent: u8,
    nanda_l3_weight_percent: u8,
    llmwave_shadow: bool,
    llmwave_apply: bool,
    nanda_l2_phase_shadow: bool,
    nanda_l2_phase_apply: bool,
    nanda_l3_phase_shadow: bool,
    nanda_precognition: bool,
    ime_bracket_candidates: bool,
    text_backend: String,
}

impl InputConfigIdentity {
    fn from_config(config: &LayConfig) -> Self {
        Self {
            auto_replace: config.auto_replace,
            typing_assist: config.typing_assist,
            auto_switch_layout: config.auto_switch_layout,
            nanda_autocorrect: config.nanda_autocorrect,
            correction_safety: config.active_correction_safety(),
            typing_assist_pipeline: config
                .typing_assist_pipeline
                .iter()
                .map(|rule| TypingAssistRuleIdentity {
                    id: rule.id.clone(),
                    enabled: rule.enabled,
                    priority: rule.priority,
                })
                .collect(),
            nanda_l2_weight_percent: config.nanda_l2_weight_percent.min(200),
            nanda_l3_weight_percent: config.nanda_l3_weight_percent.min(200),
            llmwave_shadow: config.llmwave_shadow,
            llmwave_apply: config.llmwave_shadow && config.llmwave_apply,
            nanda_l2_phase_shadow: config.nanda_l2_phase_shadow,
            nanda_l2_phase_apply: config.nanda_l2_phase_shadow && config.nanda_l2_phase_apply,
            nanda_l3_phase_shadow: config.nanda_l3_phase_shadow,
            nanda_precognition: config.nanda_precognition,
            ime_bracket_candidates: config.ime_bracket_candidates,
            text_backend: config.text_backend.trim().to_ascii_lowercase(),
        }
    }
}

/// Exact identity of one printable-input frame shared by display readout and
/// the prepared Space correction. Exact text remains present because a hash is
/// not sufficient authority for a later committed-tail edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputFrameIdentity {
    pub(crate) path: String,
    pub(crate) focus_receipt: Option<String>,
    pub(crate) tail_epoch: u64,
    pub(crate) committed_tail: String,
    pub(crate) context_prefix: String,
    pub(crate) observed_token: String,
    pub(crate) active_composition: bool,
    pub(crate) active_layout_is_ru: bool,
    pub(crate) factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
    pub(crate) exact_authority_snapshot:
        Option<lay::exact_layout_authority::ExactAuthoritySnapshot>,
    pub(crate) output_capability_fingerprint: u64,
    pub(crate) frame_fingerprint: u64,
    pub(crate) config: InputConfigIdentity,
}

impl InputFrameIdentity {
    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    pub(crate) fn new(
        path: String,
        focus_receipt: Option<String>,
        tail_epoch: u64,
        committed_tail: String,
        context_prefix: String,
        observed_token: String,
        active_composition: bool,
        active_layout_is_ru: bool,
        config: &LayConfig,
    ) -> Self {
        let factory_engine_profile = if active_layout_is_ru {
            lay::exact_layout_authority::FactoryEngineProfile::Ru
        } else {
            lay::exact_layout_authority::FactoryEngineProfile::UsQwerty
        };
        Self::new_authoritative(
            path,
            focus_receipt,
            tail_epoch,
            committed_tail,
            context_prefix,
            observed_token,
            active_composition,
            active_layout_is_ru,
            factory_engine_profile,
            0,
            config,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_authoritative(
        path: String,
        focus_receipt: Option<String>,
        tail_epoch: u64,
        committed_tail: String,
        context_prefix: String,
        observed_token: String,
        active_composition: bool,
        active_layout_is_ru: bool,
        factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
        output_capability_fingerprint: u64,
        config: &LayConfig,
    ) -> Self {
        let config = InputConfigIdentity::from_config(config);
        let active_decoder_layout =
            lay::exact_layout_authority::ActiveDecoderLayout::from_layout_is_ru(
                active_layout_is_ru,
            );
        let exact_authority_snapshot =
            lay::exact_layout_authority::exact_authority_snapshot_if_warm(
                factory_engine_profile,
                active_decoder_layout,
            );
        let frame_fingerprint = frame_fingerprint(FrameFingerprintInput {
            path: &path,
            focus_receipt: focus_receipt.as_deref(),
            tail_epoch,
            committed_tail: &committed_tail,
            context_prefix: &context_prefix,
            observed_token: &observed_token,
            active_composition,
            active_layout_is_ru,
            factory_engine_profile,
            exact_authority_snapshot,
            output_capability_fingerprint,
            config: &config,
        });
        Self {
            path,
            focus_receipt,
            tail_epoch,
            committed_tail,
            context_prefix,
            observed_token,
            active_composition,
            active_layout_is_ru,
            factory_engine_profile,
            exact_authority_snapshot,
            output_capability_fingerprint,
            frame_fingerprint,
            config,
        }
    }

    pub(crate) fn config_matches(&self, config: &LayConfig) -> bool {
        self.config == InputConfigIdentity::from_config(config)
    }

    pub(crate) fn boundary_text(&self) -> Option<String> {
        let trimmed = self.committed_tail.trim_end_matches(char::is_whitespace);
        if trimmed != format!("{}{}", self.context_prefix, self.observed_token)
            || self.observed_token.is_empty()
        {
            return None;
        }
        Some(format!("{trimmed} "))
    }

    pub(crate) fn exact_layout_frame(&self) -> lay::exact_layout_authority::ExactLayoutFrame {
        lay::exact_layout_authority::ExactLayoutFrame {
            frame_revision: self.tail_epoch,
            frame_fingerprint: self.frame_fingerprint,
            observed_token: self.observed_token.clone(),
            active_composition: self.active_composition,
            factory_engine_profile: self.factory_engine_profile,
            active_decoder_layout:
                lay::exact_layout_authority::ActiveDecoderLayout::from_layout_is_ru(
                    self.active_layout_is_ru,
                ),
            authority_snapshot: self.exact_authority_snapshot,
        }
    }

    pub(crate) fn certificate_matches(
        &self,
        certificate: &lay::exact_layout_authority::ExactLayoutContourCertificate,
    ) -> bool {
        certificate.matches_frame(self.tail_epoch, self.frame_fingerprint)
    }
}

struct FrameFingerprintInput<'a> {
    path: &'a str,
    focus_receipt: Option<&'a str>,
    tail_epoch: u64,
    committed_tail: &'a str,
    context_prefix: &'a str,
    observed_token: &'a str,
    active_composition: bool,
    active_layout_is_ru: bool,
    factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
    exact_authority_snapshot: Option<lay::exact_layout_authority::ExactAuthoritySnapshot>,
    output_capability_fingerprint: u64,
    config: &'a InputConfigIdentity,
}

fn frame_fingerprint(input: FrameFingerprintInput<'_>) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.path.hash(&mut hasher);
    input.focus_receipt.hash(&mut hasher);
    input.tail_epoch.hash(&mut hasher);
    input.committed_tail.hash(&mut hasher);
    input.context_prefix.hash(&mut hasher);
    input.observed_token.hash(&mut hasher);
    input.active_composition.hash(&mut hasher);
    input.active_layout_is_ru.hash(&mut hasher);
    input.factory_engine_profile.hash(&mut hasher);
    input
        .exact_authority_snapshot
        .map(lay::exact_layout_authority::ExactAuthoritySnapshot::fingerprint)
        .hash(&mut hasher);
    input.output_capability_fingerprint.hash(&mut hasher);
    input.config.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod input_frame_identity_tests {
    use lay::config::LayConfig;

    use super::InputFrameIdentity;

    fn identity(config: &LayConfig) -> InputFrameIdentity {
        InputFrameIdentity::new(
            "/engine/a".to_string(),
            Some("focus-a".to_string()),
            17,
            "контекст слово".to_string(),
            "контекст ".to_string(),
            "слово".to_string(),
            true,
            true,
            config,
        )
    }

    #[test]
    fn correction_affecting_config_is_part_of_frame_identity() {
        let baseline_config = LayConfig::default();
        let baseline = identity(&baseline_config);

        let mut changed_config = baseline_config.clone();
        changed_config.auto_replace = !changed_config.auto_replace;
        assert_ne!(baseline, identity(&changed_config));

        let mut changed_config = baseline_config.clone();
        changed_config.nanda_l2_weight_percent =
            changed_config.nanda_l2_weight_percent.saturating_add(1);
        assert_ne!(baseline, identity(&changed_config));

        let mut changed_config = baseline_config;
        changed_config.ime_bracket_candidates = !changed_config.ime_bracket_candidates;
        assert_ne!(baseline, identity(&changed_config));
    }

    #[test]
    fn exact_text_and_focus_dimensions_are_not_hash_only() {
        let expected = identity(&LayConfig::default());

        let mut changed = expected.clone();
        changed.committed_tail.push('x');
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.context_prefix.push('x');
        assert_ne!(expected, changed);

        let mut changed = expected.clone();
        changed.focus_receipt = Some("focus-b".to_string());
        assert_ne!(expected, changed);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WordInputMode {
    ManagedCommit,
    TerminalPassthrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManualToggleAuthority {
    ImeActiveComposition,
    ImeCommittedTail,
    DaemonWordBuffer,
}

#[derive(Debug, Clone)]
pub(crate) struct RecentCommittedTailReplace {
    pub(crate) backspaces: u32,
    pub(crate) text: String,
    pub(crate) at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingVisiblePostcondition {
    pub(crate) expected_suffix: String,
    /// Immutable identity of the field state that dispatched this transition.
    /// A later observation can confirm this receipt, but cannot teach L4 if it
    /// belongs to a different focus, epoch, or visible tail.
    pub(crate) snapshot: SnapshotIdentity,
    pub(crate) dispatched_epoch: u64,
    pub(crate) dispatched_at: Instant,
    pub(crate) feedback: Option<PendingSystemOutcomeFeedback>,
    /// IME layout ownership follows the externally confirmed committed text.
    /// Keeping it on the receipt prevents an engine switch from destroying the
    /// observation path before the client publishes the new surrounding tail.
    pub(crate) layout_sync_text: Option<String>,
}

/// Payload retained until IBus exposes the post-dispatch visible state.
/// Dispatch alone is censored evidence, never positive learning evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingSystemOutcomeFeedback {
    pub(crate) original: String,
    pub(crate) replacement: String,
    pub(crate) source: VisibleTailSource,
    pub(crate) kind: SystemOutcomeKind,
}

impl PendingSystemOutcomeFeedback {
    pub(crate) fn from_winner(source: VisibleTailSource, action: &EditAction) -> Self {
        let kind = if action.transition().operator() == Some(TransitionOperator::LayoutProjection) {
            SystemOutcomeKind::LayoutProjection
        } else {
            SystemOutcomeKind::Correction
        };
        Self {
            original: action.from_text().to_string(),
            replacement: action.to_text().to_string(),
            source,
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SystemOutcomeKind {
    LayoutProjection,
    Correction,
}

/// A Tab completion is provisional until the user starts the next word.
/// Deleting the accepted tail first means the candidate was not actually useful.
#[derive(Debug, Clone)]
pub(crate) struct PendingImeCompletionLearning {
    pub(crate) context_tail: String,
    pub(crate) typed_prefix: String,
    pub(crate) accepted_word: String,
    pub(crate) editing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurroundingTextSnapshot {
    pub(crate) text: String,
    pub(crate) cursor_pos: u32,
    pub(crate) anchor_pos: u32,
}

impl SurroundingTextSnapshot {
    pub(crate) fn new(text: String, cursor_pos: u32, anchor_pos: u32) -> Self {
        Self {
            text,
            cursor_pos,
            anchor_pos,
        }
    }

    pub(crate) fn suffix_before_cursor(&self, chars: usize) -> Option<String> {
        if chars == 0 {
            return Some(String::new());
        }
        let cursor = self.cursor_pos as usize;
        if cursor < chars || self.text.chars().count() < cursor {
            return None;
        }
        Some(
            self.text
                .chars()
                .take(cursor)
                .skip(cursor - chars)
                .collect(),
        )
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.cursor_pos != self.anchor_pos
    }
}
