use std::time::{Duration, Instant};

use zbus::fdo;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

use super::atomic::{AtomicCapability, AtomicEnvelope, AtomicPriorReceipt};
use super::engine::{LayIbusEngine, SurroundingTextSnapshot};
use super::output::{AtomicProposal, EngineOutput};
use super::protocol::{is_accept_completion_with_space_key, is_key_press, is_shift_key};
use super::trace;

#[interface(name = "org.freedesktop.IBus.Engine")]
impl LayIbusEngine {
    #[zbus(name = "ProcessKeyEvent")]
    async fn process_key_event(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if !self.legacy_key_route_allowed() {
            trace::record(r#"{"kind":"ibus_legacy_key_blocked","owner":"atomic"}"#);
            return Ok(false);
        }
        self.consume_shift_gesture_handoff();
        let mut output = EngineOutput::legacy(&emitter);
        self.process_key_event_with_output(&mut output, keyval, keycode, state)
            .await
    }

    #[zbus(name = "ProcessKeyEventAtomicV1")]
    async fn process_key_event_atomic_v1(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<AtomicProposal> {
        self.process_atomic_key_event(keyval, keycode, state, envelope, capability, prior_receipt)
            .await
    }

    #[zbus(name = "FocusIn")]
    fn focus_in(&mut self) {
        self.discard_atomic_pending();
        self.atomic_route_active = false;
        self.invalidate_input_frame_background_work();
        let changed = self.bind_focus_path();
        trace::record(if changed {
            r#"{"kind":"ibus_focus","stage":"focus_in","receipt":"new_path"}"#
        } else {
            r#"{"kind":"ibus_focus","stage":"focus_in","receipt":"same_path"}"#
        });
        self.config = lay::config::LayConfig::load();
        self.surrounding_text_snapshot = None;
        if !changed {
            self.refresh_empty_tail_from_handoff();
        }
    }

    #[zbus(name = "FocusInId")]
    fn focus_in_id(&mut self, object_path: String, client: String) {
        let changed = self.bind_focus_receipt(object_path, client);
        trace::record(if changed {
            r#"{"kind":"ibus_focus","stage":"focus_in_id","receipt":"new"}"#
        } else {
            r#"{"kind":"ibus_focus","stage":"focus_in_id","receipt":"same"}"#
        });
        self.focus_in();
    }

    #[zbus(name = "FocusOut")]
    fn focus_out(&mut self) {
        self.discard_atomic_pending();
        self.atomic_route_active = false;
        trace::record(r#"{"kind":"ibus_focus","stage":"focus_out"}"#);
        let preserve_active_path =
            self.should_preserve_focus_handoff() || self.shared_active_path_preserved();
        self.reset_for_ibus_focus_change();
        if preserve_active_path {
            return;
        }
        let mut state = self.shared.lock().expect("lay ime state poisoned");
        if state.active_path.as_deref() == Some(self.path.as_str()) {
            state.active_path = None;
        }
    }

    #[zbus(name = "FocusOutId")]
    fn focus_out_id(&mut self, _object_path: String) {
        self.focus_out();
    }

    #[zbus(name = "SetCursorLocation")]
    async fn set_cursor_location(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> fdo::Result<()> {
        self.cursor_cell_width = w;
        trace::record_cursor_location(x, y, w, h);
        if self.atomic_route_active {
            return Ok(());
        }
        let mut output = EngineOutput::legacy(&emitter);
        self.flush_dirty_preedit(&mut output).await
    }

    #[zbus(name = "ProcessHandWritingEvent")]
    fn process_hand_writing_event(&mut self, _coordinates: Vec<f64>) {}

    #[zbus(name = "CancelHandWriting")]
    fn cancel_hand_writing(&mut self, _n_strokes: u32) {}

    #[zbus(name = "SetCapabilities")]
    fn set_capabilities(&mut self, caps: u32) {
        self.set_client_capabilities(caps);
        trace::record_capabilities(caps, self.surrounding_text_supported);
    }

    #[zbus(name = "PropertyActivate")]
    fn property_activate(&mut self, _name: String, _state: u32) {}

    #[zbus(name = "PropertyShow")]
    fn property_show(&mut self, _name: String) {}

    #[zbus(name = "PropertyHide")]
    fn property_hide(&mut self, _name: String) {}

    #[zbus(name = "CandidateClicked")]
    fn candidate_clicked(&mut self, _index: u32, _button: u32, _state: u32) {}

    #[zbus(name = "Reset")]
    fn reset(&mut self) {
        self.discard_atomic_pending();
        trace::record(r#"{"kind":"ibus_focus","stage":"reset"}"#);
        self.reset_for_ibus_soft_reset();
    }

    #[zbus(name = "Enable")]
    async fn enable(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        trace::record(r#"{"kind":"ibus_focus","stage":"enable"}"#);
        Self::require_surrounding_text(&emitter)
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    #[zbus(name = "Disable")]
    fn disable(&mut self) {
        self.discard_atomic_pending();
        self.atomic_route_active = false;
        trace::record(r#"{"kind":"ibus_focus","stage":"disable"}"#);
        self.reset_for_ibus_soft_reset();
    }

    #[zbus(name = "PageUp")]
    fn page_up(&mut self) {}

    #[zbus(name = "PageDown")]
    fn page_down(&mut self) {}

    #[zbus(name = "CursorUp")]
    fn cursor_up(&mut self) {}

    #[zbus(name = "CursorDown")]
    fn cursor_down(&mut self) {}

    #[zbus(name = "SetSurroundingText")]
    async fn set_surrounding_text(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> fdo::Result<()> {
        let snapshot = ibus_text_value_to_string(&text)
            .map(|text| SurroundingTextSnapshot::new(text, cursor_pos, anchor_pos));
        self.observe_external_surrounding_text(snapshot);
        let retry_status = self.pending_ime_auto_undo_retry_status();
        trace::record_surrounding_text_snapshot(
            self.surrounding_text_snapshot
                .as_ref()
                .map_or(0, |snapshot| snapshot.text.chars().count()),
            cursor_pos,
            anchor_pos,
            retry_status,
        );
        if self.atomic_route_active {
            self.observe_visible_postcondition();
            return Ok(());
        }
        let mut output = EngineOutput::legacy(&emitter);
        if should_apply_auto_undo_before_postcondition(retry_status) {
            let status = if self.undo_last_ime_autocorrect(&mut output).await?.is_some() {
                "applied_after_causal_precondition_snapshot"
            } else {
                "causal_precondition_apply_failed"
            };
            trace::record_auto_undo_retry(status);
        }
        self.observe_visible_postcondition();
        if matches!(retry_status, "ready" | "ready_boundary_elided") {
            let status = if self.undo_last_ime_autocorrect(&mut output).await?.is_some() {
                if retry_status == "ready_boundary_elided" {
                    "applied_after_boundary_elided_snapshot"
                } else {
                    "applied_after_exact_snapshot"
                }
            } else {
                "snapshot_apply_failed"
            };
            trace::record_auto_undo_retry(status);
        }
        Ok(())
    }

    #[zbus(name = "PanelExtensionReceived")]
    fn panel_extension_received(&mut self, _event: Value<'_>) {}

    #[zbus(name = "PanelExtensionRegisterKeys")]
    fn panel_extension_register_keys(&mut self, _data: Value<'_>) {}

    #[zbus(signal, name = "CommitText")]
    pub(crate) async fn commit_text(
        emitter: &SignalEmitter<'_>,
        text: Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "ForwardKeyEvent")]
    pub(crate) async fn forward_key_event(
        emitter: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "DeleteSurroundingText")]
    pub(crate) async fn delete_surrounding_text(
        emitter: &SignalEmitter<'_>,
        offset: i32,
        nchars: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "RequireSurroundingText")]
    pub(crate) async fn require_surrounding_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal, name = "UpdatePreeditText")]
    pub(crate) async fn update_preedit_text(
        emitter: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "ShowPreeditText")]
    pub(crate) async fn show_preedit_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal, name = "HidePreeditText")]
    pub(crate) async fn hide_preedit_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property, name = "ContentType")]
    fn content_type(&self) -> (u32, u32) {
        (0, 0)
    }

    #[zbus(property, name = "ContentType")]
    fn set_content_type(&mut self, _value: (u32, u32)) {}

    #[zbus(property, name = "FocusId")]
    fn focus_id(&self) -> bool {
        false
    }

    #[zbus(property, name = "ActiveSurroundingText")]
    fn active_surrounding_text(&self) -> bool {
        true
    }
}

impl LayIbusEngine {
    pub(crate) async fn process_key_event_with_output(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if !self.managed_input {
            return Ok(false);
        }
        if !is_key_press(state) && self.consume_handled_release(keycode) {
            trace::record_key(
                "managed_release",
                keyval,
                keycode,
                true,
                None,
                self.tail_buffer.chars().count(),
                self.preedit_suffix.chars().count(),
            );
            return Ok(true);
        }
        if !self.live_composition_enabled() {
            if self.has_live_composition_state() {
                self.reset_for_ibus_focus_change();
                self.clear_preedit(output).await?;
            }
            trace::record_key("composition_disabled", keyval, keycode, false, None, 0, 0);
            return Ok(false);
        }
        if is_shift_key(keyval) {
            let pressed = is_key_press(state);
            self.shift_active = pressed;
            if pressed {
                self.shift_pressed_at = Some(Instant::now());
                self.shift_used_as_modifier = false;
                if self.alt_completion_active {
                    self.alt_used_as_modifier = true;
                    self.shift_used_as_modifier = true;
                    return Ok(self.toggle_layout_from_modifier_hotkey());
                }
            } else {
                let now = Instant::now();
                let tapped = self.shift_pressed_at.take().is_some_and(|pressed_at| {
                    now.duration_since(pressed_at) <= Duration::from_millis(self.config.tap_max_ms)
                }) && !self.shift_used_as_modifier;
                let double_tapped = tapped
                    && self.last_shift_release_at.is_some_and(|released_at| {
                        now.duration_since(released_at)
                            <= Duration::from_millis(self.config.shift_window_ms)
                    });
                self.shift_used_as_modifier = false;
                self.last_shift_release_at = tapped.then_some(now);
                if double_tapped {
                    self.last_shift_release_at = None;
                    if self
                        .manual_toggle_active_text_target(output)
                        .await?
                        .is_some()
                    {
                        return Ok(true);
                    }
                }
            }
            return Ok(false);
        }
        if is_accept_completion_with_space_key(keyval) {
            let pressed = is_key_press(state);
            if pressed {
                self.alt_completion_active = true;
                self.alt_used_as_modifier = self.shift_active;
                if self.shift_active {
                    self.shift_used_as_modifier = true;
                    return Ok(self.toggle_layout_from_modifier_hotkey());
                }
                return Ok(false);
            }
            if self.alt_completion_active && !self.alt_used_as_modifier {
                self.alt_completion_active = false;
                return self.accept_completion_with_space(output).await;
            }
            self.alt_completion_active = false;
            self.alt_used_as_modifier = false;
            return Ok(false);
        }
        if !is_key_press(state) {
            return Ok(false);
        }
        if self.shift_active {
            self.shift_used_as_modifier = true;
        }
        if self.alt_completion_active {
            self.alt_used_as_modifier = true;
        }
        let handled = self
            .process_pressed_key(output, keyval, keycode, state)
            .await?;
        self.remember_handled_press(keycode, handled);
        Ok(handled)
    }
}

fn should_apply_auto_undo_before_postcondition(retry_status: &str) -> bool {
    retry_status == "ready_causal_precondition"
}

#[cfg(test)]
mod causal_precondition_tests {
    use super::should_apply_auto_undo_before_postcondition;

    #[test]
    fn causal_precondition_undo_precedes_stale_postcondition_quarantine() {
        assert!(should_apply_auto_undo_before_postcondition(
            "ready_causal_precondition"
        ));
        assert!(!should_apply_auto_undo_before_postcondition("ready"));
        assert!(!should_apply_auto_undo_before_postcondition(
            "ready_boundary_elided"
        ));
        assert!(!should_apply_auto_undo_before_postcondition(
            "waiting_exact_snapshot"
        ));
    }
}

pub(crate) fn ibus_text_value_to_string(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Str(text) => Some(text.as_str().to_string()),
        Value::Structure(structure) => {
            let fields = structure.fields();
            match fields.get(2) {
                Some(Value::Str(text)) => Some(text.as_str().to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ibus_text_value_to_string;
    use crate::text::make_ibus_text;
    use zbus::zvariant::Value;

    #[test]
    fn parses_plain_string_surrounding_text() {
        assert_eq!(
            ibus_text_value_to_string(&Value::new("привет")),
            Some("привет".to_string())
        );
    }

    #[test]
    fn parses_ibus_text_surrounding_text() {
        let value = make_ibus_text("abc ghbdtn".to_string());

        assert_eq!(
            ibus_text_value_to_string(&value),
            Some("abc ghbdtn".to_string())
        );
    }
}
