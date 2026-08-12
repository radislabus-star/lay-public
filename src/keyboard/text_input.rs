mod ru_emit;
mod runs;
mod script;
mod us_emit;

pub(crate) use runs::text_to_key_events_into;
pub use runs::{text_to_key_events, text_to_uinput_runs};
pub use script::{is_cyrillic_letter, preferred_layout_for_text};
