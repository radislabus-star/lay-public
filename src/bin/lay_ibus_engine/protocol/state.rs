use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub(crate) struct PendingImeAutoUndo {
    pub(crate) original: String,
    pub(crate) replacement: String,
    pub(crate) recorded_at: Instant,
}

#[derive(Debug, Default)]
pub(crate) struct SharedState {
    pub(crate) active_path: Option<String>,
    pub(crate) handoff_tail_buffer: String,
    pub(crate) handoff_tail_epoch: u64,
    pub(crate) handoff_focus_receipt: Option<String>,
    pub(crate) suppress_next_committed_tail_autocorrect: bool,
    pub(crate) preserve_active_path_until: Option<Instant>,
    pub(crate) pending_auto_undo: Option<PendingImeAutoUndo>,
    pub(crate) next_engine_id: u32,
}

pub(crate) type Shared = Arc<Mutex<SharedState>>;
