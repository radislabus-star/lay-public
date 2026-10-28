//! KDE-specific foreground policy for the common daemon dispatcher.
//! This preserves the canonical KWin provider and grants no text-edit rights.
use super::{focused_window_json_is_ignored, window_identity_with_tab_title, FocusedWindowState};
use crate::{kde_focus, log};
use std::sync::atomic::{AtomicBool, Ordering};

static KDE_FOCUS_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

pub(super) fn provider_unavailable() -> bool {
    KDE_FOCUS_UNAVAILABLE.load(Ordering::Relaxed)
}

pub(crate) fn kde_focus_poll_state(
    observation: Result<kde_focus::Observation, kde_focus::ReadError>,
) -> Option<FocusedWindowState> {
    match observation {
        Ok(observation) => {
            KDE_FOCUS_UNAVAILABLE.store(false, Ordering::Relaxed);
            Some(kde_window_state(observation))
        }
        // Another keyboard's fresh request is not evidence of lost focus.
        Err(kde_focus::ReadError::Busy) => None,
        Err(kde_focus::ReadError::Unavailable(error)) => {
            if !KDE_FOCUS_UNAVAILABLE.swap(true, Ordering::Relaxed) {
                log(&format!(
                    "⚠ KWin foreground unavailable; host input paused: {error}"
                ));
            }
            Some(unavailable_kde_focus())
        }
    }
}

pub(super) fn unavailable_kde_focus() -> FocusedWindowState {
    FocusedWindowState {
        ignored: true,
        identity: None,
    }
}

pub(super) fn kde_window_state(observation: kde_focus::Observation) -> FocusedWindowState {
    let identity = serde_json::from_str::<serde_json::Value>(&observation.info)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .map(|object| window_identity_with_tab_title(observation.identity, &object));
    FocusedWindowState {
        ignored: identity.is_none() || focused_window_json_is_ignored(&observation.info),
        identity,
    }
}

pub(crate) fn exact_kde_window_identity(
    observation: Result<kde_focus::Observation, kde_focus::ReadError>,
) -> Result<String, String> {
    let observation =
        observation.map_err(|error| format!("exact KWin foreground read failed: {error}"))?;
    let state = kde_window_state(observation);
    if state.ignored {
        return Err("exact KWin foreground is ignored or unavailable".into());
    }
    state
        .identity
        .ok_or_else(|| "exact KWin foreground identity unavailable".into())
}
