pub(crate) const BUS_NAME: &str = "io.github.radislabus_star.LayIme";
pub(crate) const BUS_PATH: &str = "/io/github/radislabus_star/LayIme";
pub(crate) const IBUS_ENGINE_NAME: &str = "org.freedesktop.IBus.Lay";
pub(crate) const IBUS_FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";

#[path = "protocol/keys.rs"]
mod keys;
#[path = "protocol/modifiers.rs"]
mod modifiers;
#[path = "protocol/state.rs"]
mod state;

pub(crate) use keys::*;
pub(crate) use modifiers::has_command_modifier;
pub(crate) use state::{PendingImeAutoUndo, PendingImeAutoUndoRetry, Shared, SharedState};
