//! Firefox's IBus rectangle describes the composition start, not the suffix.
//! This reads display coordinates only; it never installs a document binding.
use super::{
    bounded, bus, LayIbusEngine, NativeResult, SurroundingTextSnapshot, OBSERVATION_BUDGET,
};
use crate::hint_view::SuffixTextStyle;
use std::future::Future;
use std::path::Path;
use std::time::{Duration, Instant};

async fn current_suffix_style(
    connection: &zbus::Connection,
    identity: &super::NativeFieldIdentity,
    snapshot: &SurroundingTextSnapshot,
) -> NativeResult<SuffixTextStyle> {
    let offset =
        i32::try_from(snapshot.cursor_pos.saturating_sub(1)).map_err(|_| "native_document_size")?;
    let attributes = bus::call(
        connection,
        &identity.owner,
        identity.field.as_str(),
        "org.a11y.atspi.Text",
        "GetAttributeRun",
        &(offset, true),
    )
    .await
    .ok()
    .and_then(|reply| {
        reply
            .body()
            .deserialize::<(std::collections::HashMap<String, String>, i32, i32)>()
            .ok()
    });
    let font_family = attributes.and_then(|(mut values, start, end)| {
        if start > offset || end <= offset || values.len() > 64 {
            return None;
        }
        values.remove("family-name").filter(|family| {
            !family.trim().is_empty()
                && family.chars().count() <= 128
                && !family.chars().any(char::is_control)
        })
    });
    // Font metadata is optional. It must neither block completion merely for
    // being absent nor survive a field, privacy, caret or text change.
    if bus::read_bound(connection, identity, false).await?.snapshot != *snapshot {
        return Err("native_document_changed");
    }
    Ok(SuffixTextStyle { font_family })
}

pub(super) fn is_firefox_process(exe: &Path) -> bool {
    exe.file_name()
        .is_some_and(|name| name == "firefox" || name == "firefox-bin")
}

fn snapshot_has_owned_prefix(snapshot: &SurroundingTextSnapshot, owned: &str) -> bool {
    !owned.is_empty()
        && !snapshot.has_selection()
        && snapshot
            .suffix_before_cursor(owned.chars().count())
            .as_deref()
            == Some(owned)
}

// Follow KDE/Chrome's read_with_owned_append_resampling: every actual read and
// cooperative yield retains the original deadline. No snapshot is installed.
async fn read_ready_geometry<T, R, F>(deadline: Instant, mut read: R) -> NativeResult<T>
where
    R: FnMut() -> F,
    F: Future<Output = NativeResult<T>>,
{
    loop {
        if Instant::now() >= deadline {
            return Err("native_deadline");
        }
        let result = read().await;
        if Instant::now() >= deadline {
            return Err("native_deadline");
        }
        match result {
            Err(
                "native_document_shape"
                | "native_document_changed"
                | "firefox_hint_prefix_pending"
                | "firefox_hint_no_character_bounds",
            ) => {
                async_io::Timer::at((Instant::now() + Duration::from_millis(1)).min(deadline))
                    .await;
            }
            other => return other,
        }
    }
}

impl LayIbusEngine {
    fn kde_firefox_typed_hint_prefix(&self) -> String {
        // Same two representations as visible_case_completion_suffix:
        // an owned preedit buffer or the already committed current word.
        if !self.composition.buffer.is_empty() {
            return self.composition.buffer.clone();
        }
        if self.committed_tail.buffer.ends_with(char::is_whitespace) {
            return String::new();
        }
        self.last_tail_token_text()
    }

    /// None preserves the existing presentation of other desktops/clients.
    /// Firefox needs actual bounds for both a committed typed prefix and an
    /// owned preedit; their raw IBus rectangles have the same ambiguous anchor.
    /// An applicable read failure must hide the suffix, never reuse its old
    /// composition-start rectangle. A returned rectangle grants no edit proof.
    pub(crate) async fn kde_firefox_owned_hint_presentation(
        &self,
    ) -> Option<Result<([i32; 4], SuffixTextStyle), &'static str>> {
        if self.config.active_layout_backend() != lay::desktop::LayoutBackend::Kde
            || !self.client_context.exact_surrounding_refresh_available
            || self.atomic.active
            || self.content_is_sensitive()
        {
            return None;
        }
        let prefix = self.kde_firefox_typed_hint_prefix();
        if prefix.is_empty() {
            return None;
        }
        let Some(token) = self.live_context_token() else {
            return Some(Err("firefox_hint_no_live_context"));
        };
        let started = Instant::now();
        let deadline = started + OBSERVATION_BUDGET;
        let mut samples = 0;
        let result = bounded(deadline, async {
            let desktop = super::super::desktop::kde::observe_focused_window_until(deadline)?;
            let exe = std::fs::read_link(format!("/proc/{}/exe", desktop.pid))
                .map_err(|_| "firefox_hint_process")?;
            if !is_firefox_process(&exe) {
                return Ok(None);
            }
            let rect = Box::pin(read_ready_geometry(deadline, || {
                samples += 1;
                let desktop = desktop.clone();
                let prefix = &prefix;
                async move {
                    let (connection, identity, actual) =
                        Box::pin(bus::read_focused(desktop, true)).await?;
                    if !snapshot_has_owned_prefix(&actual.snapshot, prefix) {
                        return Err("firefox_hint_prefix_pending");
                    }
                    let rect = actual.hint_rect.ok_or("firefox_hint_no_character_bounds")?;
                    let style =
                        current_suffix_style(&connection, &identity, &actual.snapshot).await?;
                    Ok((rect, style))
                }
            }))
            .await?;
            let current = super::super::desktop::kde::observe_focused_window_until(deadline)?;
            if !super::super::desktop::kde::native_window_continues(&desktop, &current)
                || self.live_context_token() != Some(token)
            {
                return Err("firefox_hint_context_changed");
            }
            Ok(Some(rect))
        })
        .await;
        if crate::trace::enabled() {
            crate::trace::record(format!(
                r#"{{"kind":"ibus_firefox_hint_geometry","elapsed_us":{},"samples":{},"available":{},"error":{:?}}}"#,
                started.elapsed().as_micros(),
                samples,
                matches!(&result, Ok(Some(_))),
                result.as_ref().err().copied().unwrap_or("")
            ));
        }
        match result {
            Ok(None) => None,
            Ok(Some(rect)) => Some(Ok(rect)),
            Err(reason) => Some(Err(reason)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_waits_for_actual_text_without_resampling_privacy_or_transport_errors() {
        use std::cell::Cell;
        use std::collections::VecDeque;
        let calls = Cell::new(0);
        let mut samples = VecDeque::from([
            Err("native_document_shape"),
            Err("native_document_changed"),
            Err("firefox_hint_prefix_pending"),
            Err("firefox_hint_no_character_bounds"),
            Ok([10, 20, 0, 22]),
        ]);
        let actual = zbus::block_on(read_ready_geometry(
            Instant::now() + OBSERVATION_BUDGET,
            || {
                calls.set(calls.get() + 1);
                std::future::ready(samples.pop_front().expect("unexpected extra read"))
            },
        ));
        assert_eq!(actual, Ok([10, 20, 0, 22]));
        assert_eq!(calls.get(), 5);
        for reason in [
            "native_rpc",
            "native_field_state",
            "native_selection_present",
            "native_private_or_terminal",
            "firefox_hint_context_changed",
        ] {
            calls.set(0);
            let result = zbus::block_on(read_ready_geometry::<[i32; 4], _, _>(
                Instant::now() + OBSERVATION_BUDGET,
                || {
                    calls.set(calls.get() + 1);
                    std::future::ready(Err(reason))
                },
            ));
            assert_eq!(result, Err(reason));
            assert_eq!(calls.get(), 1);
        }
    }

    #[test]
    fn expired_geometry_budget_performs_no_native_read() {
        let calls = std::cell::Cell::new(0);
        let result = zbus::block_on(read_ready_geometry::<[i32; 4], _, _>(
            Instant::now(),
            || {
                calls.set(calls.get() + 1);
                std::future::ready(Ok([0, 0, 0, 1]))
            },
        ));
        assert_eq!(result, Err("native_deadline"));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn typed_prefix_covers_committed_and_owned_text_without_changing_case() {
        use lay::config::LayConfig;
        use std::sync::{Arc, Mutex};
        let mut engine = LayIbusEngine::new(
            "/test".into(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                layout_backend: "kde".into(),
                ..LayConfig::default()
            },
        );
        for (tail, owned, expected) in [
            ("до П", "", "П"),
            ("до при", "", "при"),
            ("до 𐐀", "", "𐐀"),
            ("до ", "При", "При"),
            ("", "", ""),
            ("до ", "", ""),
            ("до\n", "", ""),
        ] {
            engine.committed_tail.buffer = tail.into();
            engine.composition.buffer = owned.into();
            assert_eq!(engine.kde_firefox_typed_hint_prefix(), expected);
        }
    }

    #[test]
    fn owned_prefix_must_be_observed_at_the_unselected_caret() {
        for (text, cursor, anchor, owned, expected) in [
            ("п", 1, 1, "п", true),
            ("до п", 4, 4, "п", true),
            ("а𐐀б", 3, 3, "𐐀б", true),
            ("п после", 1, 1, "п", true),
            ("п", 0, 0, "п", false),
            ("п", 1, 0, "п", false),
            ("п", 1, 1, "р", false),
            ("п", 2, 2, "п", false),
            ("п", 1, 1, "", false),
        ] {
            let snapshot = SurroundingTextSnapshot::new(text.into(), cursor, anchor);
            assert_eq!(snapshot_has_owned_prefix(&snapshot, owned), expected);
        }
    }

    #[test]
    fn process_classification_excludes_chrome_and_terminals() {
        for exe in ["/usr/bin/firefox", "/opt/firefox/firefox-bin"] {
            assert!(is_firefox_process(Path::new(exe)));
        }
        for exe in [
            "/opt/google/chrome/chrome",
            "/usr/bin/kitty",
            "/usr/bin/konsole",
            "/",
        ] {
            assert!(!is_firefox_process(Path::new(exe)));
        }
    }

    #[test]
    fn gnome_and_unadapted_kde_never_start_a_native_geometry_read() {
        use lay::config::LayConfig;
        use std::sync::{Arc, Mutex};
        for (desktop, exact_refresh) in [("gnome", true), ("kde", false)] {
            let mut engine = LayIbusEngine::new(
                "/test".into(),
                Arc::new(Mutex::new(Default::default())),
                true,
                true,
                LayConfig {
                    layout_backend: desktop.into(),
                    ..LayConfig::default()
                },
            );
            engine.client_context.exact_surrounding_refresh_available = exact_refresh;
            engine.composition.legacy_word_preedit_active = true;
            engine.composition.buffer = "п".into();
            engine.committed_tail.buffer = "п".into();
            assert!(zbus::block_on(engine.kde_firefox_owned_hint_presentation()).is_none());
        }
    }
}
