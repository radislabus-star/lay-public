use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordedBoundaryPrecondition {
    pub(crate) frame: crate::engine::InputFrameIdentity,
    pub(crate) snapshot: crate::engine::SurroundingTextSnapshot,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingImeAutoUndo {
    pub(crate) original: String,
    pub(crate) replacement: String,
    pub(crate) visible_tail: String,
    pub(crate) transition: lay::typing_cpu::ObservedSystemTransition,
    pub(crate) recorded_at: Instant,
    pub(crate) atomic_submission_proven: bool,
    pub(crate) boundary_forward_action: Option<lay::text_edit::EditAction>,
    pub(crate) recorded_boundary_precondition: Option<RecordedBoundaryPrecondition>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingImeAutoUndoRetry {
    pub(crate) undo_recorded_at: Instant,
    pub(crate) requested_at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct ShiftGestureHandoff {
    pub(crate) source_path: String,
    pub(crate) shift_active: bool,
    pub(crate) shift_pressed_at: Option<Instant>,
    pub(crate) shift_used_as_modifier: bool,
    pub(crate) last_shift_release_at: Option<Instant>,
    pub(crate) space_pressed: bool,
    pub(crate) space_cancelled: bool,
    pub(crate) last_space_release_at: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactReplayBoundaryHold {
    pub(crate) token: crate::context_admission::AdmissionToken,
    pub(crate) focus_serial: u64,
    pub(crate) layout_generation: u64,
    pub(crate) tail_epoch: u64,
    pub(crate) observation_revision: u64,
    /// The existing final key pair was dispatched, never a client result ACK.
    pub(crate) dispatched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactManualToggleSuppression {
    /// Inert admitted source. It never becomes live again after Reset.
    pub(crate) source_token: Option<Box<crate::context_admission::AdmissionToken>>,
    /// Origin of that same validated source, never a replacement delivery ACK.
    pub(crate) source_provenance: Option<crate::context_admission::ExactManualSourceProvenance>,
    pub(crate) path: String,
    pub(crate) epoch: u64,
    pub(crate) expires_at: Instant,
    pub(crate) owner_lease_identity: u64,
    /// An explicit Ready may rebind delivery without changing the armed owner.
    pub(crate) delivery_lease_identity: Option<u64>,
    pub(crate) target_layout_is_ru: bool,
    /// Delivery progress survives lexical revocation; it never proves a word.
    pub(crate) native_press_count: usize,
    /// One planned Space; source admission still needs the actual whole word.
    pub(crate) held_boundary: Option<Box<ExactReplayBoundaryHold>>,
    pub(crate) original_tail: String,
    pub(crate) original_suffix: String,
    pub(crate) unchanged_prefix: String,
    pub(crate) observed_external_prefix: Option<String>,
    pub(crate) replacement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurrentWordSuppression {
    pub(crate) incarnation: u64,
    pub(crate) owner_lease_identity: u64,
    pub(crate) open_token_chars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AutocorrectSuppression {
    CurrentWord(CurrentWordSuppression),
    LegacyReplayV1,
    ExactReplay(ExactManualToggleSuppression),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SharedState {
    pub(crate) active_path: Option<String>,
    /// TD-121 admission owner generation for `active_path`. Legacy engines
    /// leave this unset and retain their existing path-only behavior.
    pub(crate) context_owner_generation: Option<u64>,
    pub(crate) handoff_tail_buffer: String,
    pub(crate) handoff_tail_epoch: u64,
    pub(crate) handoff_focus_receipt: Option<String>,
    pub(crate) autocorrect_suppression: Option<AutocorrectSuppression>,
    pub(crate) suppression_revision: u64,
    pub(crate) preserve_active_path_until: Option<Instant>,
    pub(crate) exact_manual_toggle_handoff_epoch: Option<u64>,
    pub(crate) exact_manual_toggle_handoff_path: Option<String>,
    pub(crate) pending_auto_undo: Option<PendingImeAutoUndo>,
    pub(crate) pending_auto_undo_retry: Option<PendingImeAutoUndoRetry>,
    pub(crate) shift_gesture_handoff: Option<ShiftGestureHandoff>,
    pub(crate) next_engine_id: u32,
    pub(crate) hint_view: Option<crate::hint_view::HintViewBus>,
}

pub(crate) type Shared = Arc<Mutex<SharedState>>;
