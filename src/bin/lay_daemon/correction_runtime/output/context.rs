use evdev::uinput::VirtualDevice;
use lay::engine::ManualCorrectionDecision;
use lay::keyboard::KeyEvent;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

pub(crate) struct ManualCorrectionOutputContext<'a> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) mapped_orig: &'a str,
    pub(crate) mapped_target: &'a str,
    pub(crate) target_is_ru: bool,
    pub(crate) n_backspaces: u32,
    pub(crate) replace_words: usize,
    pub(crate) words_orig: usize,
    pub(crate) force_replay_toggle: bool,
    pub(crate) started_at: Instant,
    pub(crate) decision: &'a ManualCorrectionDecision,
    pub(crate) virtual_kbd: Option<&'a mut VirtualDevice>,
}

pub(crate) struct ManualOutputCommon<'a> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) events: &'a [KeyEvent],
    pub(crate) mapped_orig: &'a str,
    pub(crate) mapped_target: &'a str,
    pub(crate) target_is_ru: bool,
    pub(crate) n_backspaces: u32,
    pub(crate) replace_words: usize,
    pub(crate) words_orig: usize,
    pub(crate) force_replay_toggle: bool,
    pub(crate) started_at: Instant,
    pub(crate) decision: &'a ManualCorrectionDecision,
}

pub(crate) enum OutputFlow {
    ContinueReplay,
    Return(Option<bool>),
}
