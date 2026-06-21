use zbus::fdo;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

use super::engine::LayIbusEngine;
use super::protocol::{is_accept_completion_with_space_key, is_key_press, is_shift_key, KEY_SPACE};
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
        if !self.managed_input {
            return Ok(false);
        }
        if !self.live_composition_enabled() {
            if self.has_live_composition_state() {
                self.reset_for_ibus_focus_change();
                self.clear_preedit(&emitter).await?;
            }
            trace::record_key("composition_disabled", keyval, keycode, false, None, 0, 0);
            return Ok(false);
        }
        if is_shift_key(keyval) {
            let pressed = is_key_press(state);
            self.shift_active = pressed;
            if pressed {
                self.shift_used_as_modifier = false;
            } else {
                self.shift_used_as_modifier = false;
                self.last_shift_release_at = None;
            }
            return Ok(false);
        }
        if is_accept_completion_with_space_key(keyval) {
            let pressed = is_key_press(state);
            if pressed {
                self.alt_completion_active = true;
                self.alt_used_as_modifier = false;
                return Ok(false);
            }
            if self.alt_completion_active && !self.alt_used_as_modifier {
                self.alt_completion_active = false;
                return self.accept_completion_with_space(&emitter).await;
            }
            self.alt_completion_active = false;
            self.alt_used_as_modifier = false;
            return Ok(false);
        }
        if keyval == KEY_SPACE && !is_key_press(state) {
            return self
                .apply_pending_committed_tail_space_autocorrect(&emitter)
                .await;
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
        self.process_pressed_key(&emitter, keyval, keycode, state)
            .await
    }

    #[zbus(name = "FocusIn")]
    fn focus_in(&mut self) {
        trace::record(r#"{"kind":"ibus_focus","stage":"focus_in"}"#);
        self.config = lay::config::LayConfig::load();
        self.surrounding_text_supported = false;
        self.refresh_empty_tail_from_handoff();
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path = Some(self.path.clone());
    }

    #[zbus(name = "FocusInId")]
    fn focus_in_id(&mut self, _object_path: String, _client: String) {
        self.focus_in();
    }

    #[zbus(name = "FocusOut")]
    fn focus_out(&mut self) {
        trace::record(r#"{"kind":"ibus_focus","stage":"focus_out"}"#);
        let preserve_active_path =
            self.should_preserve_focus_handoff() || self.shared_active_path_preserved();
        self.reset_for_ibus_focus_change();
        self.surrounding_text_supported = false;
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
        self.flush_dirty_preedit(&emitter).await
    }

    #[zbus(name = "ProcessHandWritingEvent")]
    fn process_hand_writing_event(&mut self, _coordinates: Vec<f64>) {}

    #[zbus(name = "CancelHandWriting")]
    fn cancel_hand_writing(&mut self, _n_strokes: u32) {}

    #[zbus(name = "SetCapabilities")]
    fn set_capabilities(&mut self, _caps: u32) {}

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
        trace::record(r#"{"kind":"ibus_focus","stage":"reset"}"#);
        self.reset_for_ibus_soft_reset();
    }

    #[zbus(name = "Enable")]
    fn enable(&mut self) {}

    #[zbus(name = "Disable")]
    fn disable(&mut self) {
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
    fn set_surrounding_text(&mut self, _text: Value<'_>, _cursor_pos: u32, _anchor_pos: u32) {
        self.surrounding_text_supported = true;
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
