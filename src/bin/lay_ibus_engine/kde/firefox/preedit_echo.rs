//! Select Firefox's delayed preedit echo; the existing word witness owns proof.
use super::{
    firefox_hint_geometry::is_firefox_process, LayIbusEngine, SurroundingTextSnapshot, IO_BUDGET,
};
use lay::desktop::LayoutBackend;
use std::time::Instant;

impl LayIbusEngine {
    /// Only an inert candidate for the existing current-Firefox observation.
    /// It never confirms a client result or creates an edit/word receipt.
    pub(crate) fn kde_firefox_native_erased_boundary_echo(
        &self,
        snapshot: &SurroundingTextSnapshot,
    ) -> bool {
        if self.config.active_layout_backend() != LayoutBackend::Kde
            || !self.client_context.exact_surrounding_refresh_available
            || self.atomic.active
            || snapshot.has_selection()
            || snapshot.cursor_pos as usize != snapshot.text.chars().count()
        {
            return false;
        }
        let (Some(local), Some(shared)) = self.exact_replay_scope_pair() else {
            return false;
        };
        let deletes = local.original_suffix.chars().count();
        let distance = self.committed_tail.epoch.wrapping_sub(local.epoch) as usize;
        let complete = deletes.saturating_add(local.replacement.chars().count());
        if local != shared
            || local.held_boundary.is_some()
            || deletes == 0
            || distance < deletes
            || distance > complete
            || local.native_press_count != distance
            || (distance < complete && Instant::now() > local.expires_at)
            || !self.exact_replay_scope_is_current_except_expiry(&local)
        {
            return false;
        }
        let Some(external) = local.observed_external_prefix.as_deref() else {
            return false;
        };
        // Firefox reflows the now-trailing separator after erasure. Compare the
        // whole field and only that final run; never normalize letters, internal
        // separators, selection or caret position into a matching receipt.
        let erased = format!("{external}{}", local.unchanged_prefix);
        let boundary = |ch| matches!(ch, ' ' | '\u{a0}');
        let expected_body = erased.trim_end_matches(boundary);
        let actual_body = snapshot.text.trim_end_matches(boundary);
        let closing = erased.chars().count() - expected_body.chars().count();
        closing > 0
            && !expected_body.is_empty()
            && actual_body == expected_body
            && snapshot.text.chars().count() - actual_body.chars().count() == closing
    }

    pub(crate) fn kde_firefox_owned_append_caret_echo_matches(
        previous: &SurroundingTextSnapshot,
        current: &SurroundingTextSnapshot,
        token: &str,
    ) -> bool {
        let prefix = previous
            .text
            .rsplit(crate::preedit::is_observed_word_boundary)
            .next()
            .unwrap_or("");
        let Some(appended) = token
            .strip_prefix(prefix)
            .and_then(|rest| rest.chars().next())
        else {
            return false;
        };
        !prefix.is_empty()
            && !crate::preedit::is_observed_word_boundary(appended)
            && !previous.has_selection()
            && !current.has_selection()
            && previous.cursor_pos as usize == previous.text.chars().count()
            && current.cursor_pos == previous.cursor_pos
            && current.text == format!("{}{appended}", previous.text)
    }

    pub(crate) fn kde_firefox_owned_append_caret_echo_settles(
        previous: &SurroundingTextSnapshot,
        current: &SurroundingTextSnapshot,
        token: &str,
    ) -> bool {
        let prefix = previous
            .text
            .rsplit(crate::preedit::is_observed_word_boundary)
            .next()
            .unwrap_or("");
        let external = &previous.text[..previous.text.len() - prefix.len()];
        !prefix.is_empty()
            && token.starts_with(prefix)
            && !previous.has_selection()
            && !current.has_selection()
            && (previous.cursor_pos as usize <= previous.text.chars().count())
            && previous.text.chars().count() - previous.cursor_pos as usize <= 1
            && current.cursor_pos > previous.cursor_pos
            && current.cursor_pos as usize == current.text.chars().count()
            && current.text.starts_with(&previous.text)
            && format!("{external}{token}").starts_with(&current.text)
    }

    /// An inert partial readback preserves its preceding whole actual field,
    /// including external context. The ordinary exact receipt still owns edits.
    pub(crate) fn kde_firefox_owned_prefix_progress(
        anchor: &SurroundingTextSnapshot,
        previous: &SurroundingTextSnapshot,
        current: &SurroundingTextSnapshot,
        token: &str,
    ) -> bool {
        let anchor_word = anchor
            .text
            .rsplit(crate::preedit::is_observed_word_boundary)
            .next()
            .unwrap_or("");
        let external = &anchor.text[..anchor.text.len() - anchor_word.len()];
        let expected = format!("{external}{token}");
        !anchor_word.is_empty()
            && !anchor.has_selection()
            && anchor.cursor_pos as usize == anchor.text.chars().count()
            && token.starts_with(anchor_word)
            && previous.text.starts_with(&anchor.text)
            && current.text != expected
            && expected.starts_with(&current.text)
            && Self::kde_firefox_owned_append_caret_echo_settles(previous, current, token)
    }

    pub(crate) fn kde_firefox_erased_preedit_snapshot_matches(
        snapshot: &SurroundingTextSnapshot,
        prefix: &str,
        suffix: &str,
        cursor: u32,
        erased: char,
    ) -> bool {
        !snapshot.has_selection()
            && snapshot.cursor_pos == cursor
            && snapshot
                .text
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.strip_prefix(erased) == Some(suffix))
    }

    pub(crate) fn kde_firefox_preedit_echo_client_is_current(&self) -> bool {
        if self.config.active_layout_backend() != LayoutBackend::Kde {
            return false;
        }
        let Some(token) = self.live_context_token() else {
            return false;
        };
        let deadline = Instant::now() + IO_BUDGET;
        let Ok(window) = super::super::desktop::kde::observe_focused_window_until(deadline) else {
            return false;
        };
        let Ok(exe) = std::fs::read_link(format!("/proc/{}/exe", window.pid)) else {
            return false;
        };
        Instant::now() < deadline
            && is_firefox_process(&exe)
            && self.live_context_token() == Some(token)
    }
}
