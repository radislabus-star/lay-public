use evdev::uinput::VirtualDevice;
use lay::word_buffer::WordBuffer;

use super::super::physical_input_grab::PhysicalInputGrab;
use super::super::DaemonTextObservation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManualCorrectionOutputRoute {
    ConfiguredBackend,
    DaemonUinput,
}

impl ManualCorrectionOutputRoute {
    pub(crate) fn allows_native_stage(self) -> bool {
        !matches!(self, Self::DaemonUinput)
    }

    pub(crate) fn allows_ime_stage(self) -> bool {
        matches!(self, Self::ConfiguredBackend)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManualCorrectionInputIsolation {
    Shared,
    PhysicalGrab,
}

impl ManualCorrectionInputIsolation {
    pub(crate) const fn requires_physical_grab(self) -> bool {
        matches!(self, Self::PhysicalGrab)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ManualCorrectionDispatchPlan {
    pub(crate) output_route: ManualCorrectionOutputRoute,
    pub(crate) input_isolation: ManualCorrectionInputIsolation,
}

pub(crate) struct ManualCorrectionRequest<'a, 'grab> {
    pub(crate) buf: &'a mut WordBuffer,
    pub(crate) replace_words: usize,
    pub(crate) virtual_kbd: Option<&'a mut VirtualDevice>,
    pub(crate) executing: &'a mut bool,
    pub(crate) input_isolated: bool,
    pub(crate) text_observation: DaemonTextObservation<'a>,
    pub(crate) physical_grab: Option<&'a mut PhysicalInputGrab<'grab>>,
    pub(crate) output_route: ManualCorrectionOutputRoute,
}

pub(crate) struct ScopedManualCorrectionRequest<'a, 'grab> {
    pub(crate) manual: ManualCorrectionRequest<'a, 'grab>,
    pub(crate) events_since_word_start: u32,
    pub(crate) label: &'a str,
}

#[cfg(test)]
mod tests {
    use super::{ManualCorrectionInputIsolation, ManualCorrectionOutputRoute};

    #[test]
    fn output_route_and_input_isolation_are_independent_contracts() {
        assert!(ManualCorrectionOutputRoute::ConfiguredBackend.allows_native_stage());
        assert!(ManualCorrectionOutputRoute::ConfiguredBackend.allows_ime_stage());
        assert!(!ManualCorrectionOutputRoute::DaemonUinput.allows_native_stage());
        assert!(!ManualCorrectionOutputRoute::DaemonUinput.allows_ime_stage());

        assert!(!ManualCorrectionInputIsolation::Shared.requires_physical_grab());
        assert!(ManualCorrectionInputIsolation::PhysicalGrab.requires_physical_grab());
    }
}
