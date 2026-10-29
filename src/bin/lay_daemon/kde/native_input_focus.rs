//! KDE's existing client must keep focus while the original keyboard is frozen.
//! This directs only the already-owned replay master; it grants no edit rights.
use std::io;

use lay::desktop::LayoutBackend;
use x11rb::protocol::xinput::ConnectionExt as _;

use super::{err, replay_master::ReplayMaster};

pub(super) fn required(backend: LayoutBackend) -> bool {
    backend == LayoutBackend::Kde
}

pub(super) fn retain(replay: &mut ReplayMaster, focus: u32) -> io::Result<()> {
    if focus <= 1 || replay.keyboard == 0 || !replay.held.is_empty() {
        return Err(err(
            "KDE native acquisition requires an idle owned replay master and client focus",
        ));
    }
    // Reuse the same-client focus operation already used by Input::ungrab.
    // Never set the original master's focus or activate a window.
    replay.with_owned_devices(|| {
        replay
            .conn
            .xinput_xi_set_focus(focus, x11rb::CURRENT_TIME, replay.keyboard)
            .map_err(err)?
            .check()
            .map_err(err)
    })
}
