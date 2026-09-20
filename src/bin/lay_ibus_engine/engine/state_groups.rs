use std::collections::BTreeSet;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
#[cfg(test)]
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::preedit::PreeditFastState;
use super::super::protocol::AutocorrectSuppression;
use super::types::{DeferredLayoutAction, DeferredLearningAction};
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
    pub(crate) autocorrect_suppression: Option<AutocorrectSuppression>,
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
            autocorrect_suppression: None,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct CompositionState {
    pub(crate) buffer: String,
    pub(crate) cursor: usize,
    /// The unfinished word is owned as legacy IBus preedit, so its boundary
    /// may commit a verified replacement without editing application text.
    pub(crate) legacy_word_preedit_active: bool,
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
    #[cfg(test)]
    pub(crate) precognition_schedule_count: Arc<AtomicUsize>,
    #[cfg(test)]
    pub(crate) precognition_apply_count: Arc<AtomicUsize>,
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
    #[cfg(test)]
    pub(crate) before_capture: Option<Arc<dyn Fn() + Send + Sync>>,
    #[cfg(test)]
    pub(crate) settlement_feedback_events: Arc<Mutex<Vec<&'static str>>>,
}
