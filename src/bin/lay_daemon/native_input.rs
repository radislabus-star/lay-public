//! Session-local resources for the existing daemon loop and key-frame executor.
//! These adapters do not recognize gestures or choose text mutations.
use evdev::{AttributeSet, InputEvent, KeyCode};
use std::io;
use std::os::fd::{AsRawFd, RawFd};

#[path = "native_input/x11.rs"]
pub(super) mod x11;

pub(crate) enum Device {
    Evdev(Box<evdev::Device>),
    X11(Box<x11::Input>),
}

impl Device {
    pub(crate) fn requires_isolated_replay(&self) -> bool {
        matches!(self, Self::X11(_))
    }

    pub(crate) fn set_batch_pending(&mut self, pending: bool) {
        if let Self::X11(device) = self {
            device.set_batch_pending(pending);
        }
    }

    pub(crate) fn name(&self) -> Option<&str> {
        match self {
            Self::Evdev(device) => device.name(),
            Self::X11(_) => Some("X11 session keyboard"),
        }
    }

    pub(crate) fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        match self {
            Self::Evdev(device) => device.set_nonblocking(nonblocking),
            Self::X11(_) if nonblocking => Ok(()),
            Self::X11(_) => Err(io::Error::other("X11 input is nonblocking")),
        }
    }

    pub(crate) fn fetch_events(&mut self) -> io::Result<std::vec::IntoIter<InputEvent>> {
        match self {
            Self::Evdev(device) => device
                .fetch_events()
                .map(|events| events.collect::<Vec<_>>().into_iter()),
            Self::X11(device) => device.fetch_events().map(Vec::into_iter),
        }
    }

    pub(crate) fn get_key_state(&self) -> io::Result<AttributeSet<KeyCode>> {
        match self {
            Self::Evdev(device) => device.get_key_state(),
            Self::X11(device) => device.key_state(),
        }
    }

    pub(crate) fn grab(&mut self) -> io::Result<()> {
        match self {
            Self::Evdev(device) => device.grab(),
            Self::X11(device) => device.grab(),
        }
    }

    pub(crate) fn ungrab(&mut self) -> io::Result<()> {
        match self {
            Self::Evdev(device) => device.ungrab(),
            Self::X11(device) => device.ungrab(),
        }
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            Self::Evdev(device) => device.as_raw_fd(),
            Self::X11(device) => device.as_raw_fd(),
        }
    }
}

// Retain existing executor signatures while making the transport explicit here.
pub(crate) enum VirtualDevice {
    Evdev(evdev::uinput::VirtualDevice),
    X11(Box<x11::Output>),
}

impl VirtualDevice {
    pub(crate) fn requires_client_settlement(&self) -> bool {
        matches!(self, Self::X11(_))
    }

    pub(crate) fn emit(&mut self, events: &[InputEvent]) -> io::Result<()> {
        match self {
            Self::Evdev(device) => device.emit(events),
            Self::X11(device) => device.emit(events),
        }
    }
}
