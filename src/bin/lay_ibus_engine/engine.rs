use lay::config::LayConfig;
use std::time::{Duration, Instant};

use super::preedit::PreeditFastState;
use super::protocol::Shared;

pub(super) const DOUBLE_SHIFT_WINDOW: Duration = Duration::from_millis(650);

pub(crate) struct LayIbusEngine {
    pub(super) path: String,
    pub(super) shared: Shared,
    pub(super) buffer: String,
    pub(super) composition_cursor: usize,
    pub(super) tail_buffer: String,
    pub(super) preedit_suffix: String,
    pub(super) preedit_candidates: Vec<String>,
    pub(super) preedit_candidate_index: usize,
    pub(super) preedit_fast: PreeditFastState,
    pub(super) preedit_dirty: bool,
    pub(super) cursor_cell_width: i32,
    pub(super) surrounding_text_supported: bool,
    pub(super) layout_is_ru: bool,
    pub(super) shift_active: bool,
    pub(super) shift_used_as_modifier: bool,
    pub(super) last_shift_release_at: Option<Instant>,
    pub(super) last_commit_at: Option<Instant>,
    pub(super) managed_input: bool,
    pub(super) config: LayConfig,
}
