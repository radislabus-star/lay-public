//! A layout-only descriptor for one existing exact-tail replay.
//! Normal layout completion and every text output operation remain unchanged.
use super::{ime_bridge, ImeDelegatedTailLease};
use std::cell::Cell;

pub(crate) struct Handoff {
    lease: ImeDelegatedTailLease,
    target_is_ru: bool,
    window: String,
    restored: Cell<bool>,
}

pub(super) fn activate(
    lease: &ImeDelegatedTailLease,
    target_is_ru: bool,
    window: &str,
) -> Result<Handoff, String> {
    if crate::active_layout_backend() != lay::desktop::LayoutBackend::Gnome
        || !crate::active_text_backend().should_try_ime()
        || lease.initial_layout_is_ru() == target_is_ru
    {
        return Err("fragment word-mode replay requires its GNOME IME boundary".into());
    }
    crate::verify_exact_focused_window_identity(window)?;
    let handoff = Handoff {
        lease: lease.clone(),
        target_is_ru,
        window: window.into(),
        restored: Cell::new(false),
    };
    let result: Result<(), String> = (|| {
        if !ime_bridge::complete_fragment_word_mode(
            lease,
            lease.initial_layout_is_ru(),
            target_is_ru,
        )? {
            return Err("fragment word-mode completion refused its current handoff".into());
        }
        crate::verify_exact_focused_window_identity(window)?;
        if ime_bridge::current_input_mode_is_ru()? != target_is_ru {
            return Err("fragment word-mode decoder readback mismatch".into());
        }
        Ok(())
    })();
    if let Err(error) = result {
        let restore = handoff
            .restore_once()
            .err()
            .map(|error| format!("; initial mode unresolved: {error}"))
            .unwrap_or_default();
        return Err(format!("{error}{restore}"));
    }
    Ok(handoff)
}

impl Handoff {
    pub(super) fn restore_once(&self) -> Result<(), String> {
        if self.restored.replace(true) {
            return Ok(());
        }
        crate::verify_exact_focused_window_identity(&self.window)?;
        let initial = self.lease.initial_layout_is_ru();
        if ime_bridge::current_input_mode_is_ru()? == initial {
            return Ok(());
        }
        if !ime_bridge::complete_fragment_word_mode(&self.lease, self.target_is_ru, initial)? {
            return Err("initial fragment mode no longer has the same handoff".into());
        }
        Ok(())
    }
}
