use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub(crate) struct SharedState {
    pub(crate) active_path: Option<String>,
    pub(crate) handoff_tail_buffer: String,
    pub(crate) next_engine_id: u32,
}

pub(crate) type Shared = Arc<Mutex<SharedState>>;
