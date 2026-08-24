use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub(crate) struct PendingImeAutoUndo {
    pub(crate) original: String,
    pub(crate) replacement: String,
    pub(crate) visible_tail: String,
    pub(crate) transition: lay::typing_cpu::ObservedSystemTransition,
    pub(crate) recorded_at: Instant,
    pub(crate) atomic_submission_proven: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PendingImeAutoUndoRetry {
    pub(crate) undo_recorded_at: Instant,
    pub(crate) requested_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShiftGestureHandoffAuthority {
    PendingAutoUndo,
    CyclicLayout,
}

#[derive(Debug, Clone)]
pub(crate) struct ShiftGestureHandoff {
    pub(crate) authority: ShiftGestureHandoffAuthority,
    pub(crate) source_path: String,
    pub(crate) exact_tail: String,
    pub(crate) tail_epoch: u64,
    pub(crate) target_layout_is_ru: Option<bool>,
    pub(crate) expires_at: Instant,
    pub(crate) shift_active: bool,
    pub(crate) shift_pressed_at: Option<Instant>,
    pub(crate) shift_used_as_modifier: bool,
    pub(crate) last_shift_release_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub(crate) struct CyclicLayoutHandoff {
    pub(crate) source_path: String,
    pub(crate) target_path: Option<String>,
    pub(crate) exact_tail: String,
    pub(crate) tail_epoch: u64,
    pub(crate) source_layout_is_ru: bool,
    pub(crate) target_layout_is_ru: bool,
    pub(crate) expires_at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct DaemonDelegatedLayoutHandoff {
    pub(crate) source_path: String,
    pub(crate) target_path: Option<String>,
    pub(crate) target_layout_is_ru: bool,
    pub(crate) tail_epoch: u64,
    pub(crate) expires_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SharedState {
    pub(crate) active_path: Option<String>,
    pub(crate) handoff_tail_buffer: String,
    pub(crate) handoff_tail_epoch: u64,
    pub(crate) handoff_focus_receipt: Option<String>,
    pub(crate) suppress_next_committed_tail_autocorrect: bool,
    pub(crate) preserve_active_path_until: Option<Instant>,
    pub(crate) daemon_delegated_layout_handoff: Option<DaemonDelegatedLayoutHandoff>,
    pub(crate) pending_auto_undo: Option<PendingImeAutoUndo>,
    pub(crate) pending_auto_undo_retry: Option<PendingImeAutoUndoRetry>,
    pub(crate) cyclic_layout_handoff: Option<CyclicLayoutHandoff>,
    pub(crate) shift_gesture_handoff: Option<ShiftGestureHandoff>,
    pub(crate) next_engine_id: u32,
}

pub(crate) type Shared = Arc<Mutex<SharedState>>;
