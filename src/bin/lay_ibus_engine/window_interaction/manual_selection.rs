use super::desktop::{self, FocusedWindow};
use super::native_document::{bounded, verify_manual_selection};
use super::{ExecutionReceipt, SurroundingTextSnapshot, WindowInteraction};
use crate::bridge::LayImeBridge;
use crate::context_admission::AdmissionToken;
use crate::engine::LayIbusEngine;
use crate::output::EngineOutput;
use lay::desktop::LayoutBackend;
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::text_edit::{
    authorize_backend_edit, plan_manual_selection_layout_edit, AuthorizedEdit, TextEditBackend,
};
use std::time::{Duration, Instant};
use zbus::fdo;

const SELECTION_OBSERVATION_BUDGET: Duration = Duration::from_millis(80);

pub(super) fn has_document_snapshot(engine: &LayIbusEngine) -> bool {
    matches!(
        engine.config.active_layout_backend(),
        LayoutBackend::Gnome | LayoutBackend::X11
    ) && engine.client_context.native_document_binding.is_none()
        && engine.client_context.surrounding_text_snapshot.is_some()
}

pub(super) struct VerifiedManualSelection {
    selected: String,
}

impl VerifiedManualSelection {
    pub(super) fn authorize(self) -> Option<AuthorizedEdit> {
        let action = plan_manual_selection_layout_edit(&self.selected)?;
        authorize_backend_edit(TextEditBackend::Ime, action).into_authorized()
    }
}

fn selected_text(snapshot: &SurroundingTextSnapshot, (start, end): (u32, u32)) -> Option<String> {
    let (start, end) = (start as usize, end as usize);
    if start >= end || end > snapshot.text.chars().count() {
        return None;
    }
    Some(
        snapshot
            .text
            .chars()
            .skip(start)
            .take(end - start)
            .collect(),
    )
}

impl LayImeBridge {
    pub(super) async fn manual_selection_outcome(
        &self,
        engine: &mut LayIbusEngine,
        token: Option<&AdmissionToken>,
        path: &str,
        output: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<Option<ImeManualToggleOutcome>> {
        let Some(snapshot) = engine.client_context.surrounding_text_snapshot.as_ref() else {
            return Ok(None);
        };
        let rejected = Some(ImeManualToggleOutcome::NotHandled);
        let unverified = snapshot
            .has_selection()
            .then_some(ImeManualToggleOutcome::NotHandled);
        if !selection_admitted(engine) || !self.bridge_token_is_live(engine, token) {
            return Ok(unverified);
        }
        let backend = engine.config.active_layout_backend();
        let deadline = Instant::now() + SELECTION_OBSERVATION_BUDGET;
        let verification = bounded(deadline, async {
            let before = focused_window(backend, deadline)
                .await
                .ok_or("manual_selection_missing_window")?;
            let Some(range) = verify_manual_selection(before.clone(), snapshot).await? else {
                return Ok(None);
            };
            if focused_window(backend, deadline).await.as_ref() != Some(&before) {
                return Err("manual_selection_window_changed");
            }
            Ok(selected_text(snapshot, range))
        })
        .await;
        let selected = match verification {
            Ok(Some(selected)) => selected,
            Ok(None) => return Ok(unverified),
            Err(reason) => {
                crate::trace::record(format!(
                    r#"{{"kind":"ibus_manual_selection","stage":"rejected","reason":"{reason}"}}"#
                ));
                return Ok(unverified);
            }
        };
        if self.active_path().as_deref() != Some(path) || !self.bridge_token_is_live(engine, token)
        {
            return Ok(rejected);
        }

        let layout_is_ru = engine.layout_gesture.layout_is_ru;
        let mut engine = engine.begin_context_bridge_output(token);
        let result = WindowInteraction::execute_manual_selection(
            &mut engine,
            VerifiedManualSelection { selected },
            output,
        )
        .await
        .map_err(fdo::Error::from)?;
        engine.complete();
        if result != ExecutionReceipt::LocalComplete {
            return Ok(rejected);
        }
        crate::trace::record(r#"{"kind":"ibus_manual_selection","stage":"commit_dispatched"}"#);
        Ok(Some(ImeManualToggleOutcome::handled(layout_is_ru)))
    }
}

fn selection_admitted(engine: &LayIbusEngine) -> bool {
    engine.client_context.surrounding_text_callback_observed
        && !engine.content_is_sensitive()
        && engine.client_context.content_purpose != super::IBUS_INPUT_PURPOSE_TERMINAL
        && !engine.atomic.active
        && !engine.atomic.speculation
        && !engine.has_live_composition_state()
}

async fn focused_window(backend: LayoutBackend, deadline: Instant) -> Option<FocusedWindow> {
    match backend {
        LayoutBackend::Gnome => desktop::focused_window_until(backend, deadline).await,
        LayoutBackend::X11 => blocking::unblock(x11_focused_window).await,
        _ => None,
    }
}

fn x11_focused_window() -> Option<FocusedWindow> {
    use x11rb::protocol::res::{ClientIdMask, ClientIdSpec, ConnectionExt as _};
    use x11rb::protocol::xproto::ConnectionExt as _;

    let (connection, _) = x11rb::connect(None).ok()?;
    let window = connection.get_input_focus().ok()?.reply().ok()?.focus;
    if window <= 1 {
        return None;
    }
    let ids = connection
        .res_query_client_ids(&[ClientIdSpec {
            client: window,
            mask: ClientIdMask::LOCAL_CLIENT_PID,
        }])
        .ok()?
        .reply()
        .ok()?;
    let [id] = ids.ids.as_slice() else {
        return None;
    };
    let [pid] = id.value.as_slice() else {
        return None;
    };
    (id.spec.mask == ClientIdMask::LOCAL_CLIENT_PID && *pid != 0).then(|| FocusedWindow {
        identity: window.to_string(),
        pid: *pid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_double_shift_owner_selection_projects_only_the_exact_unicode_range() {
        for (from, to) in [
            ("[jxtim ddtcnb", "хочешь ввести"),
            ("хочешь ввести", "[jxtim ddtcnb"),
        ] {
            for (cursor, anchor) in [(16, 3), (3, 16)] {
                let snapshot =
                    SurroundingTextSnapshot::new(format!("до {from} текст"), cursor, anchor);
                let selected = selected_text(&snapshot, (3, 16)).expect("editable selection");
                let authorized = VerifiedManualSelection { selected }
                    .authorize()
                    .expect("verified physical projection");
                let plan = authorized.action().plan().expect("exact edit plan");
                assert_eq!(plan.insert, to);
                assert_eq!(plan.backspaces, 13);
                assert_eq!((plan.move_left, plan.move_right), (0, 0));
            }
        }
    }

    #[test]
    fn physical_double_shift_owner_selection_requires_a_real_nonempty_range_and_change() {
        let snapshot = SurroundingTextSnapshot::new("до ".into(), 3, 3);
        for range in [(3, 3), (3, 300), (1, 4)] {
            assert!(selected_text(&snapshot, range).is_none());
        }
        assert!(VerifiedManualSelection {
            selected: "👋".into()
        }
        .authorize()
        .is_none());
    }
}
