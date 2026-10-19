//! Transaction-local KDE capability for the existing exact replay handoff.
//! Group, selected engine and decoder mode are independent observations.
use std::cell::Cell;
use std::process::{Command, Stdio};

use super::{gnome_dbus::dbus_connection, ime_bridge, ime_target_engine, target_layout};

type Layouts = Vec<(String, String, String)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    owner: String,
    layouts: Layouts,
    index: u32,
}

impl Snapshot {
    fn index_for(&self, is_ru: bool) -> Result<u32, String> {
        let id = target_layout(is_ru).0;
        let mut indices = self
            .layouts
            .iter()
            .enumerate()
            .filter(|(_, item)| item.0 == id);
        let index = indices
            .next()
            .ok_or_else(|| format!("KDE layout {id} is absent"))?
            .0;
        if indices.next().is_some() {
            return Err(format!("KDE layout {id} is ambiguous"));
        }
        Ok(index as u32)
    }

    fn same_mapping(&self, other: &Self) -> Result<(), String> {
        if self.owner != other.owner || self.layouts != other.layouts {
            return Err("KDE layout service or mapping changed".to_string());
        }
        Ok(())
    }
}

// The production adapter and ordered tests use the same transaction below.
pub(crate) trait Io {
    fn window(&self, expected: &str) -> Result<(), String>;
    fn snapshot(&self) -> Result<Snapshot, String>;
    fn engine(&self) -> Result<String, String>;
    fn mode(&self) -> Result<bool, String>;
    fn set_layout(&self, owner: &str, index: u32) -> Result<bool, String>;
    fn set_engine(&self, engine: &str) -> Result<(), String>;
}

pub(crate) struct Session {
    connection: zbus::blocking::Connection,
}

pub(crate) struct Handoff<I = Session> {
    io: I,
    initial: Snapshot,
    target_index: u32,
    initial_is_ru: bool,
    focused_window: String,
    restore_attempted: Cell<bool>,
}

pub(super) fn activate(
    initial_is_ru: bool,
    target_is_ru: bool,
    window: &str,
) -> Result<Handoff, String> {
    activate_on(
        Session {
            connection: dbus_connection()?,
        },
        initial_is_ru,
        target_is_ru,
        window,
    )
}

fn activate_on<I: Io>(
    io: I,
    initial_is_ru: bool,
    target_is_ru: bool,
    window: &str,
) -> Result<Handoff<I>, String> {
    if initial_is_ru == target_is_ru {
        return Err("KDE exact replay must invert the observed mode".to_string());
    }
    let initial = io.snapshot()?;
    let initial_index = initial.index_for(initial_is_ru)?;
    let target_index = initial.index_for(target_is_ru)?;
    if initial.index != initial_index {
        return Err("initial KDE group differs from the leased IME mode".to_string());
    }
    let handoff = Handoff {
        io,
        initial,
        target_index,
        initial_is_ru,
        focused_window: window.to_string(),
        restore_attempted: Cell::new(false),
    };
    // A fixed-source decoder/engine divergence cannot be restored by one alias
    // request. Refuse it before mutation; do not infer mode from the engine name.
    handoff.verify_stack(initial_index, initial_is_ru)?;
    let result = (|| {
        handoff.set_group(target_index)?;
        handoff.verify_group(target_index)?;
        handoff.set_engine_if_compatible(target_index, target_is_ru)?;
        handoff.verify_stack(target_index, target_is_ru)
    })();
    if let Err(error) = result {
        let restoration = handoff
            .restore_once()
            .err()
            .map(|error| format!("; initial KDE layout unresolved: {error}"))
            .unwrap_or_default();
        return Err(format!("{error}{restoration}"));
    }
    Ok(handoff)
}

impl<I: Io> Handoff<I> {
    fn current(&self) -> Result<Snapshot, String> {
        let current = self.io.snapshot()?;
        self.initial.same_mapping(&current)?;
        self.io.window(&self.focused_window)?;
        Ok(current)
    }

    fn verify_group(&self, index: u32) -> Result<(), String> {
        if self.current()?.index != index {
            return Err("KDE group readback mismatch".to_string());
        }
        Ok(())
    }

    fn verify_stack(&self, index: u32, is_ru: bool) -> Result<(), String> {
        if self.io.engine()? != ime_target_engine(is_ru) {
            return Err("selected IBus engine differs from the exact replay target".to_string());
        }
        if self.io.mode()? != is_ru {
            return Err("actual Lay input mode differs from the exact replay target".to_string());
        }
        self.verify_group(index)
    }

    fn set_group(&self, index: u32) -> Result<(), String> {
        // Revalidate immediately before each backend mutation. Never use the
        // ordinary controller's cached indices or redirect to a new bus owner.
        let current = self.current()?;
        if current.index != self.initial.index && current.index != self.target_index {
            return Err("KDE group changed independently".to_string());
        }
        if !self.io.set_layout(&self.initial.owner, index)? {
            return Err("KDE setLayout returned false".to_string());
        }
        Ok(())
    }

    fn set_engine_if_compatible(&self, index: u32, is_ru: bool) -> Result<(), String> {
        let engine = self.io.engine()?;
        let expected = ime_target_engine(is_ru);
        if engine != expected && engine != ime_target_engine(!is_ru) {
            return Err("IBus engine changed independently; request skipped".to_string());
        }
        self.verify_group(index)?;
        if engine != expected {
            self.io.set_engine(expected)?;
        }
        Ok(())
    }

    pub(super) fn restore_once(&self) -> Result<(), String> {
        if self.restore_attempted.replace(true) {
            return Err("KDE layout restoration already attempted".to_string());
        }
        let engine = self.io.engine()?;
        let initial_engine = ime_target_engine(self.initial_is_ru);
        if engine != initial_engine && engine != ime_target_engine(!self.initial_is_ru) {
            return Err("IBus engine changed independently; restoration skipped".to_string());
        }
        let current = self.current()?;
        if current.index != self.initial.index {
            self.set_group(self.initial.index)?;
        }
        self.verify_group(self.initial.index)?;
        self.set_engine_if_compatible(self.initial.index, self.initial_is_ru)?;
        self.verify_stack(self.initial.index, self.initial_is_ru)
    }
}

impl Session {
    fn owner(&self, name: &str) -> Result<String, String> {
        let owner: String = self
            .connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "GetNameOwner",
                &(name,),
            )
            .map_err(|e| e.to_string())?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())?;
        zbus::names::UniqueName::try_from(owner.as_str()).map_err(|e| e.to_string())?;
        Ok(owner)
    }

    fn keyboard_owner(&self) -> Result<String, String> {
        let owner = self.owner("org.kde.keyboard")?;
        if owner != self.owner("org.kde.KWin")? {
            return Err("KDE keyboard and foreground service owners differ".to_string());
        }
        Ok(owner)
    }

    fn keyboard_call<B: serde::ser::Serialize + zbus::zvariant::DynamicType>(
        &self,
        owner: &str,
        method: &str,
        body: &B,
    ) -> Result<zbus::Message, String> {
        self.connection
            .call_method(
                Some(owner),
                "/Layouts",
                Some("org.kde.KeyboardLayouts"),
                method,
                body,
            )
            .map_err(|e| e.to_string())
    }
}

impl Io for Session {
    fn window(&self, expected: &str) -> Result<(), String> {
        super::super::focus_guard::verify_exact_focused_window_identity(expected)
    }

    fn snapshot(&self) -> Result<Snapshot, String> {
        let owner = self.keyboard_owner()?;
        let layouts: Layouts = self
            .keyboard_call(&owner, "getLayoutsList", &())?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())?;
        if layouts.is_empty()
            || layouts.len() > 32
            || layouts
                .iter()
                .map(|(a, b, c)| a.len() + b.len() + c.len())
                .sum::<usize>()
                > 8192
        {
            return Err("KDE layout mapping is outside the bounded capability".to_string());
        }
        let index: u32 = self
            .keyboard_call(&owner, "getLayout", &())?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())?;
        if index as usize >= layouts.len() || owner != self.keyboard_owner()? {
            return Err("KDE layout index or service lifetime changed".to_string());
        }
        Ok(Snapshot {
            owner,
            layouts,
            index,
        })
    }

    fn engine(&self) -> Result<String, String> {
        ime_bridge::observed_ibus_engine()
    }
    fn mode(&self) -> Result<bool, String> {
        ime_bridge::current_input_mode_is_ru()
    }

    fn set_layout(&self, owner: &str, index: u32) -> Result<bool, String> {
        self.keyboard_call(owner, "setLayout", &(index,))?
            .body()
            .deserialize()
            .map_err(|e| e.to_string())
    }

    fn set_engine(&self, engine: &str) -> Result<(), String> {
        // Use the existing IBus CLI without its unbounded wait/fallback. Both
        // TERM and forced cleanup fit the 250ms request budget; readbacks, not
        // the process exit status, establish the resulting engine and decoder.
        let status = Command::new("timeout")
            .args(["--kill-after=0.05s", "0.20s", "ibus", "engine", engine])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("bounded IBus engine request failed".to_string())
        }
    }
}

#[cfg(test)]
#[path = "kde_exact/tests.rs"]
mod tests;
