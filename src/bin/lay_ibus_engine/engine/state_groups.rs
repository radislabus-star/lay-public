use std::collections::BTreeSet;
use std::time::Instant;

use super::super::preedit::PreeditFastState;
use super::super::protocol::ExactManualToggleSuppression;
use super::types::{DeferredLayoutAction, DeferredLearningAction, SurroundingTextSnapshot};
use super::types::{
    InputFrameIdentity, PendingImeCompletionLearning, PendingVisiblePostcondition,
    RecentCommittedTailReplace, WordInputMode,
};

#[derive(Clone)]
pub(crate) struct CommittedTailState {
    pub(crate) buffer: String,
    pub(crate) epoch: u64,
    pub(crate) last_commit_at: Option<Instant>,
    pub(crate) last_input_at: Option<Instant>,
    pub(crate) recent_replace: Option<RecentCommittedTailReplace>,
    pub(crate) pending_visible_postcondition: Option<PendingVisiblePostcondition>,
    pub(crate) pending_completion_learning: Option<PendingImeCompletionLearning>,
    pub(crate) suppress_next_autocorrect: bool,
    pub(crate) exact_manual_toggle_suppression: Option<ExactManualToggleSuppression>,
}

impl CommittedTailState {
    pub(crate) fn new(buffer: String, epoch: u64) -> Self {
        Self {
            buffer,
            epoch,
            last_commit_at: None,
            last_input_at: None,
            recent_replace: None,
            pending_visible_postcondition: None,
            pending_completion_learning: None,
            suppress_next_autocorrect: false,
            exact_manual_toggle_suppression: None,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct CompositionState {
    pub(crate) buffer: String,
    pub(crate) cursor: usize,
    pub(crate) preedit_visible: bool,
    pub(crate) preedit_suffix: String,
    pub(crate) preedit_candidates: Vec<String>,
    pub(crate) preedit_replacement_targets: Vec<Option<String>>,
    pub(crate) preedit_candidate_index: usize,
    pub(crate) preedit_display_only_pending: bool,
    pub(crate) preedit_fast: PreeditFastState,
    pub(crate) preedit_dirty: bool,
    pub(crate) pending_display_frame: Option<InputFrameIdentity>,
    pub(crate) pending_passthrough_preedit_clear: bool,
    pub(crate) word_input_mode: Option<WordInputMode>,
}

#[derive(Clone)]
pub(crate) struct ClientContextState {
    pub(crate) focus_receipt: Option<String>,
    pub(crate) focus_serial: u64,
    pub(crate) runtime_owner_lease_identity: u64,
    pub(crate) cursor_cell_width: i32,
    pub(crate) content_purpose: u32,
    pub(crate) content_hints: u32,
    pub(crate) surrounding_text_supported: bool,
    pub(crate) surrounding_text_snapshot: Option<SurroundingTextSnapshot>,
    pub(crate) surrounding_observation_revision: u64,
    pub(crate) factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
    pub(crate) managed_input: bool,
}

impl ClientContextState {
    pub(crate) fn new(
        focus_receipt: Option<String>,
        factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
        managed_input: bool,
    ) -> Self {
        Self {
            focus_receipt,
            focus_serial: super::next_input_identity(),
            runtime_owner_lease_identity: super::next_input_identity(),
            cursor_cell_width: 0,
            content_purpose: 0,
            content_hints: 0,
            surrounding_text_supported: false,
            surrounding_text_snapshot: None,
            surrounding_observation_revision: 0,
            factory_engine_profile,
            managed_input,
        }
    }
}

#[derive(Clone)]
pub(crate) struct LayoutGestureState {
    pub(crate) layout_is_ru: bool,
    pub(crate) layout_generation: u64,
    pub(crate) shift_active: bool,
    pub(crate) shift_used_as_modifier: bool,
    pub(crate) shift_pressed_at: Option<Instant>,
    pub(crate) alt_completion_active: bool,
    pub(crate) alt_used_as_modifier: bool,
    pub(crate) handled_press_keycodes: BTreeSet<u32>,
    pub(crate) last_shift_release_at: Option<Instant>,
    pub(crate) pending_manual_toggle: bool,
}

impl LayoutGestureState {
    pub(crate) fn new(layout_is_ru: bool, layout_generation: u64) -> Self {
        Self {
            layout_is_ru,
            layout_generation,
            shift_active: false,
            shift_used_as_modifier: false,
            shift_pressed_at: None,
            alt_completion_active: false,
            alt_used_as_modifier: false,
            handled_press_keycodes: BTreeSet::new(),
            last_shift_release_at: None,
            pending_manual_toggle: false,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct AtomicRouteState {
    pub(crate) active: bool,
    pub(crate) speculation: bool,
    pub(crate) deferred_layout_actions: Vec<DeferredLayoutAction>,
    pub(crate) deferred_learning_actions: Vec<DeferredLearningAction>,
}
