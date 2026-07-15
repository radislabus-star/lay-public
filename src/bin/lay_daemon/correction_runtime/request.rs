use evdev::uinput::VirtualDevice;
use lay::config::CorrectionEngine;
use lay::word_buffer::WordBuffer;

use super::super::physical_input_grab::PhysicalInputGrab;
use super::super::DaemonTextObservation;

pub(crate) struct ManualCorrectionRequest<'a, 'grab> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) replace_words: usize,
    pub(crate) engine: CorrectionEngine,
    pub(crate) auto_replace: bool,
    pub(crate) virtual_kbd: Option<&'a mut VirtualDevice>,
    pub(crate) executing: &'a mut bool,
    pub(crate) input_isolated: bool,
    pub(crate) text_observation: DaemonTextObservation<'a>,
    pub(crate) physical_grab: Option<&'a mut PhysicalInputGrab<'grab>>,
}

pub(crate) struct ScopedManualCorrectionRequest<'a, 'grab> {
    pub(crate) manual: ManualCorrectionRequest<'a, 'grab>,
    pub(crate) events_since_word_start: u32,
    pub(crate) label: &'a str,
}
