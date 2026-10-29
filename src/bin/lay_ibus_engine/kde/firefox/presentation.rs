//! Firefox rich editors reflow the separator preceding an owned preedit.
//! The existing postcondition and tail owner validate and retain actual text.
use super::{firefox_hint_geometry::is_firefox_process, LayIbusEngine, IO_BUDGET};
use crate::engine::PendingVisiblePostcondition;
use lay::{desktop::LayoutBackend, text_edit::VisibleTailSource};
use std::time::Instant;

impl LayIbusEngine {
    pub(crate) fn acknowledge_kde_firefox_preedit_space_postcondition(
        &mut self,
        pending: &mut PendingVisiblePostcondition,
    ) -> bool {
        if self.config.active_layout_backend() != LayoutBackend::Kde
            || !self.client_context.exact_surrounding_refresh_available
            || self.content_is_sensitive()
            || self.atomic.active
            || self.context_handoff_sealed
            || pending.snapshot.source != VisibleTailSource::ImeActiveComposition
            || !pending
                .expected_external_snapshot
                .as_ref()
                .is_some_and(|expected| {
                    expected.text.ends_with(' ') && expected.text.contains('\u{a0}')
                })
            || !self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|actual| actual.text.ends_with('\u{a0}'))
        {
            return false;
        }
        let Some(token) = self.live_context_token() else {
            return false;
        };
        // Use the canonical desktop provider, as the adjacent Firefox geometry
        // reader does. This read selects presentation only, never edit rights.
        let deadline = Instant::now() + IO_BUDGET;
        let Ok(focused) = super::super::desktop::kde::observe_focused_window_until(deadline) else {
            return false;
        };
        let Ok(exe) = std::fs::read_link(format!("/proc/{}/exe", focused.pid)) else {
            return false;
        };
        if Instant::now() >= deadline
            || !is_firefox_process(&exe)
            || self.live_context_token().as_ref() != Some(&token)
            || !self.acknowledge_owned_preedit_space_reflow(pending)
        {
            return false;
        }
        crate::trace::record(
            r#"{"kind":"ibus_owned_space_boundary","stage":"client_acknowledged","source":"kde_firefox_preedit_reflow","preceding_sent":160,"preceding_observed":32,"closing_sent":32,"closing_observed":160}"#,
        );
        true
    }
}
