mod ru_emit;
mod runs;
mod script;
mod us_emit;

pub use runs::{text_to_key_events, text_to_uinput_runs};
pub use script::{is_cyrillic_letter, preferred_layout_for_text};
