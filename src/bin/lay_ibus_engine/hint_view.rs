//! Read-only presentation projection. None of these coordinates acknowledge a
//! field, authorize a key, or participate in an edit precondition.
use serde::Serialize;
use zbus::{fdo, Connection};

use super::context_admission::{ActivationOutcome, AdmissionToken, PendingHintActivation};
use super::engine::LayIbusEngine;
use super::output::EngineOutput;
use super::protocol::{BUS_NAME, BUS_PATH};

/// Optional rendering data, never a document or input authority.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct SuffixTextStyle {
    pub(crate) font_family: Option<String>,
}

#[derive(Clone)]
pub(crate) struct HintCursorGeometry {
    rect: [i32; 4],
    focus_serial: u64,
    activation: Option<AdmissionToken>,
    pending: Option<PendingHintActivation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct HintViewFrame {
    version: u8,
    stream: String,
    revision: String,
    owner: String,
    text: String,
    rect: Option<[i32; 4]>,
    monospace: bool,
    outlined: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    suffix_style: Option<SuffixTextStyle>,
}

#[derive(Debug, Clone)]
pub(crate) struct HintViewBus {
    pub(crate) connection: Connection,
    frame: HintViewFrame,
    revision: u64,
    publication_failed: bool,
}

impl HintViewBus {
    pub(crate) fn new(connection: Connection) -> Self {
        let stream = connection
            .unique_name()
            .map(ToString::to_string)
            .unwrap_or_default();
        Self {
            connection,
            frame: HintViewFrame {
                version: 1,
                stream,
                revision: "0".into(),
                ..Default::default()
            },
            revision: 0,
            publication_failed: false,
        }
    }

    pub(crate) fn snapshot(&self) -> String {
        serde_json::to_string(&self.frame).expect("hint view contains serializable values")
    }
}

impl HintViewFrame {
    #[cfg(test)]
    fn replace(
        &mut self,
        active: Option<&str>,
        owner: &str,
        text: String,
        rect: Option<[i32; 4]>,
        monospace: bool,
        outlined: bool,
    ) -> bool {
        self.replace_with_style(active, owner, text, rect, monospace, outlined, None)
    }

    fn replace_with_style(
        &mut self,
        active: Option<&str>,
        owner: &str,
        text: String,
        rect: Option<[i32; 4]>,
        monospace: bool,
        outlined: bool,
        suffix_style: Option<SuffixTextStyle>,
    ) -> bool {
        // A late old-engine clear cannot remove the new engine's projection.
        if (text.is_empty() && self.owner != owner) || (!text.is_empty() && active != Some(owner)) {
            return false;
        }
        let text = if text.chars().count() <= 512 && !text.chars().any(char::is_control) {
            text
        } else {
            String::new()
        };
        let rect = if text.is_empty() { None } else { rect };
        let outlined = outlined && rect.is_some();
        let suffix_style = suffix_style.filter(|_| rect.is_some());
        if self.owner == owner
            && self.text == text
            && self.rect == rect
            && self.monospace == monospace
            && self.outlined == outlined
            && self.suffix_style == suffix_style
        {
            return false;
        }
        self.owner = owner.to_owned();
        self.text = text;
        self.rect = rect;
        self.monospace = monospace;
        self.outlined = outlined;
        self.suffix_style = suffix_style;
        true
    }
}

impl LayIbusEngine {
    pub(crate) async fn prepare_inline_terminal_hint(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        hint: &str,
    ) -> fdo::Result<()> {
        if !hint.is_empty()
            && self.uses_native_terminal_input()
            && self.composition.buffer.is_empty()
            && !self.composition.preedit_visible
        {
            // Native PTY rendering can follow the last key-time cursor
            // report. Existing terminal lifecycle wake remains unchanged.
            self.client_context.hint_cursor_geometry = None;
            let result = output.hide_preedit_text().await;
            self.finish_inline_hint_publication(result)?;
        }
        Ok(())
    }

    pub(crate) fn remember_hint_geometry(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.client_context.hint_cursor_geometry = (height > 0
            && height <= 512
            && (0..=4096).contains(&width)
            && x.abs_diff(0) <= 100_000
            && y.abs_diff(0) <= 100_000)
            .then(|| {
                std::sync::Arc::new(HintCursorGeometry {
                    rect: [x, y, width, height],
                    focus_serial: self.client_context.focus_serial,
                    activation: self.live_context_token(),
                    pending: self
                        .context_admission
                        .as_ref()
                        .and_then(|admission| admission.pending_hint_activation(&self.path)),
                })
            });
    }

    pub(crate) fn current_hint_rect(&self) -> Option<[i32; 4]> {
        if let Some(rect) = self.native_document_hint_rect() {
            return rect;
        }
        let geometry = self.client_context.hint_cursor_geometry.as_ref()?;
        if geometry.focus_serial != self.client_context.focus_serial {
            return None;
        }
        match (&geometry.activation, self.live_context_token().as_ref()) {
            (Some(old), Some(current)) if old.shares_input_activation(current) => {
                Some(geometry.rect)
            }
            (None, None) if !self.context_admission_required => Some(geometry.rect),
            _ => None,
        }
    }

    pub(crate) fn install_pending_hint_geometry(
        &mut self,
        before_serial: u64,
        outcome: &ActivationOutcome,
    ) {
        let preserves = self
            .client_context
            .hint_cursor_geometry
            .as_ref()
            .is_some_and(|geometry| {
                geometry.focus_serial == before_serial
                    && geometry.pending.as_ref().is_some_and(|pending| {
                        self.context_admission.as_ref().is_some_and(|admission| {
                            admission.activation_preserves_hint_geometry(pending, outcome)
                        })
                    })
            });
        if preserves {
            let activation = self.live_context_token();
            let geometry = std::sync::Arc::make_mut(
                self.client_context.hint_cursor_geometry.as_mut().unwrap(),
            );
            geometry.focus_serial = self.client_context.focus_serial;
            geometry.activation = activation;
            geometry.pending = None;
        }
    }

    /// None means the isolated output transport has no session view publisher.
    /// Runtime never retries this publication through document composition.
    pub(crate) async fn publish_inline_hint(&self, mut text: String) -> Option<fdo::Result<()>> {
        // Test/isolated transports have no session renderer and must not
        // perform any native display read.
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .hint_view
            .as_ref()?;
        let mut geometry_failure = None;
        let (rect, suffix_style) = if text.is_empty() {
            (None, None)
        } else {
            match Box::pin(self.kde_firefox_owned_hint_presentation()).await {
                None => (self.current_hint_rect(), None),
                // Only the authenticated KDE Firefox adapter supplies this
                // optional typography. Other clients retain their style.
                Some(Ok((rect, style))) => (Some(rect), Some(style)),
                Some(Err(reason)) => {
                    // Withdraw the old frame before retiring this hint's
                    // selectability through finish_inline_hint_publication.
                    text.clear();
                    geometry_failure = Some(reason);
                    (None, None)
                }
            }
        };
        let monospace = self.has_proven_terminal_input();
        let (connection, payload, revision) = {
            let mut state = self.shared.lock().expect("lay ime state poisoned");
            let active = state.active_path.clone();
            let view = state.hint_view.as_mut()?;
            let changed = view.frame.replace_with_style(
                active.as_deref(),
                &self.path,
                text,
                rect,
                monospace,
                false,
                suffix_style,
            );
            if !changed && !view.publication_failed {
                return Some(Ok(()));
            }
            view.revision = view.revision.saturating_add(1);
            view.frame.revision = view.revision.to_string();
            (view.connection.clone(), view.snapshot(), view.revision)
        };
        let result = connection
            .emit_signal(
                None::<&str>,
                BUS_PATH,
                BUS_NAME,
                "InlineHintV1",
                &(payload,),
            )
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))
            .and_then(|()| match geometry_failure {
                Some(reason) => Err(fdo::Error::Failed(reason.into())),
                None => Ok(()),
            });
        if let Some(view) = self
            .shared
            .lock()
            .expect("lay ime state poisoned")
            .hint_view
            .as_mut()
        {
            if view.revision == revision {
                view.publication_failed = result.is_err();
            }
        }
        Some(result)
    }

    pub(crate) async fn refresh_inline_hint_geometry(&mut self) -> fdo::Result<()> {
        let text = {
            let state = self.shared.lock().expect("lay ime state poisoned");
            state
                .hint_view
                .as_ref()
                .filter(|view| view.frame.owner == self.path)
                .map(|view| view.frame.text.clone())
        };
        if let Some(text) = text {
            if let Some(result) = Box::pin(self.publish_inline_hint(text)).await {
                self.finish_inline_hint_publication(result)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_style_tracks_current_owner_and_same_text_font_changes() {
        let mut frame = HintViewFrame::default();
        let rect = Some([100, 100, 0, 22]);
        let style = Some(SuffixTextStyle {
            font_family: Some("Observed Font".into()),
        });
        assert!(frame.replace_with_style(
            Some("a"),
            "a",
            "вет".into(),
            rect,
            false,
            false,
            style.clone()
        ));
        assert_eq!(
            frame.suffix_style, style,
            "Firefox suffix style must follow current hint frame"
        );
        assert!(!frame.outlined);
        assert!(!frame.replace_with_style(
            Some("a"),
            "old",
            String::new(),
            None,
            false,
            false,
            None
        ));
        assert_eq!(frame.suffix_style, style);
        let changed = Some(SuffixTextStyle {
            font_family: Some("Changed Font".into()),
        });
        assert!(frame.replace_with_style(
            Some("a"),
            "a",
            "вет".into(),
            rect,
            false,
            false,
            changed.clone()
        ));
        assert_eq!(frame.suffix_style, changed);
        assert!(!frame.replace_with_style(
            Some("a"),
            "a",
            "вет".into(),
            rect,
            false,
            false,
            changed
        ));
        assert!(frame.replace_with_style(Some("b"), "b", "вет".into(), rect, true, false, None));
        assert!(frame.suffix_style.is_none());
        assert!(serde_json::to_value(&frame)
            .unwrap()
            .get("suffix_style")
            .is_none());
        assert!(frame.replace_with_style(Some("b"), "b", String::new(), None, true, false, style));
        assert!(frame.suffix_style.is_none());
    }

    #[test]
    fn late_owner_clear_cannot_hide_new_hint() {
        let mut frame = HintViewFrame::default();
        assert!(frame.replace(
            Some("a"),
            "a",
            "етка".into(),
            Some([1, 2, 0, 20]),
            false,
            false
        ));
        assert!(frame.replace(
            Some("b"),
            "b",
            "верка".into(),
            Some([3, 4, 1, 20]),
            false,
            false
        ));
        assert!(!frame.replace(Some("b"), "a", String::new(), None, false, false));
        assert_eq!(frame.text, "верка");
        assert!(!frame.replace(Some("b"), "a", "старое".into(), None, false, false));
        assert!(frame.replace(Some("b"), "b", String::new(), None, false, false));
        assert!(frame.text.is_empty());
        assert_eq!(frame.rect, None);
    }

    #[test]
    fn projection_is_bounded_and_geometry_is_not_fabricated() {
        let mut frame = HintViewFrame::default();
        assert!(frame.replace(Some("a"), "a", "роверка".into(), None, false, false));
        assert_eq!(frame.rect, None);
        assert!(frame.replace(
            Some("a"),
            "a",
            "ы".repeat(513),
            Some([1, 2, 0, 20]),
            false,
            false
        ));
        assert!(frame.text.is_empty());
        assert_eq!(frame.rect, None);
        assert!(!frame.replace(Some("a"), "a", "bad\ntext".into(), None, false, false));
    }

    #[test]
    fn suffix_outline_is_presentation_only_and_cannot_leak_between_owners() {
        let mut frame = HintViewFrame::default();
        let rect = Some([100, 100, 0, 22]);
        assert!(frame.replace(Some("a"), "a", "вет".into(), rect, false, false));
        assert!(!frame.outlined);
        assert!(frame.replace(Some("a"), "a", "вет".into(), rect, false, true));
        assert!(frame.outlined);
        assert_eq!(serde_json::to_value(&frame).unwrap()["outlined"], true);
        assert!(!frame.replace(Some("a"), "old", String::new(), None, false, false));
        assert!(frame.outlined);
        assert_eq!(frame.text, "вет");
        assert!(frame.replace(Some("b"), "b", "вет".into(), rect, true, false));
        assert!(!frame.outlined);
        assert_eq!(frame.text, "вет");
        assert!(frame.replace(Some("b"), "b", String::new(), None, true, true));
        assert!(!frame.outlined);
        assert_eq!(frame.rect, None);
    }

    #[test]
    fn cursor_geometry_expires_on_focus_change_and_rejects_invalid_rectangles() {
        let mut engine = LayIbusEngine::new(
            "/test".into(),
            Default::default(),
            true,
            true,
            lay::config::LayConfig::default(),
        );
        engine.context_admission_required = false;
        engine.remember_hint_geometry(-30, 12, 0, 20);
        assert_eq!(engine.current_hint_rect(), Some([-30, 12, 0, 20]));
        engine.client_context.focus_serial += 1;
        assert_eq!(engine.current_hint_rect(), None);
        for rect in [[0, 0, 1, 0], [i32::MIN, 0, 1, 20], [0, 0, -1, 20]] {
            engine.remember_hint_geometry(rect[0], rect[1], rect[2], rect[3]);
            assert_eq!(engine.current_hint_rect(), None);
        }
    }
}
