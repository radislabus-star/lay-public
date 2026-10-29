//! KDE Firefox result retention after complete native delivery, before actual
//! client confirmation. No layout operation, output or word authority here.
use lay::desktop::LayoutBackend;
use std::path::Path;

fn scoped_client(backend: LayoutBackend, expected: &str, observed: &str, exe: &Path) -> bool {
    backend == LayoutBackend::Kde
        && !expected.is_empty()
        && expected == observed
        && exe
            .file_name()
            .is_some_and(|name| name == "firefox" || name == "firefox-bin")
}

pub(super) fn current_client_is_scoped(expected_focus: &str) -> bool {
    let backend = crate::active_layout_backend();
    if backend != LayoutBackend::Kde {
        return false;
    }
    let Ok(observation) = crate::kde_focus::read() else {
        return false;
    };
    let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&observation.info) else {
        return false;
    };
    let Some(pid) = metadata
        .get("pid")
        .and_then(serde_json::Value::as_u64)
        .filter(|pid| *pid > 0 && *pid <= u32::MAX as u64)
    else {
        return false;
    };
    let Ok(exe) = std::fs::read_link(format!("/proc/{pid}/exe")) else {
        return false;
    };
    let Ok(observed) = crate::focus_guard::exact_kde_window_identity(Ok(observation)) else {
        return false;
    };
    scoped_client(backend, expected_focus, &observed, &exe)
}

/// Read-only policy for this operation, never a field/focus or edit receipt.
pub(super) struct ProcessingReadiness<'a> {
    scoped: bool,
    expected_focus: &'a str,
    epoch: u64,
    target_path: &'a str,
    target_layout_is_ru: bool,
}

fn processing_scope(verified_before_output: bool, classify_now: impl FnOnce() -> bool) -> bool {
    verified_before_output || classify_now()
}

/// The common settlement loop keeps its deadline and exact client readback.
pub(super) fn processing_readiness<'a>(
    expected_focus: &'a str,
    verified_before_output: bool,
    epoch: u64,
    target_path: &'a str,
    target_layout_is_ru: bool,
) -> ProcessingReadiness<'a> {
    ProcessingReadiness {
        scoped: processing_scope(verified_before_output, || {
            current_client_is_scoped(expected_focus)
        }),
        expected_focus,
        epoch,
        target_path,
        target_layout_is_ru,
    }
}

impl ProcessingReadiness<'_> {
    pub(super) fn tail_matches(&self, observed: &str, expected: &str) -> bool {
        readback_policy_for_client(self.scoped)(observed, expected)
    }

    pub(super) fn input_processed(&self) -> Result<bool, String> {
        if self.scoped {
            crate::layout_controller::native_ime_replay_processed(
                self.epoch,
                self.target_path,
                self.target_layout_is_ru,
            )
        } else {
            Ok(true)
        }
    }

    pub(super) fn retain_processed_result(&self) -> bool {
        retain_processed_result_on(
            self.scoped,
            || self.input_processed(),
            || crate::verify_exact_focused_window_identity(self.expected_focus),
        )
    }
}

pub(crate) fn readback_policy(expected_focus: &str) -> impl Fn(&str, &str) -> bool {
    readback_policy_for_client(current_client_is_scoped(expected_focus))
}

fn readback_policy_for_client(scoped: bool) -> impl Fn(&str, &str) -> bool {
    move |observed, expected| {
        if scoped {
            acknowledged_space_tail_matches(observed, expected)
        } else {
            observed.ends_with(expected)
        }
    }
}

fn retain_processed_result_on(
    scoped: bool,
    processed: impl FnOnce() -> Result<bool, String>,
    current_focus: impl FnOnce() -> Result<(), String>,
) -> bool {
    scoped && processed() == Ok(true) && current_focus().is_ok()
}

fn acknowledged_space_tail_matches(observed: &str, expected: &str) -> bool {
    if observed.ends_with(expected) {
        return true;
    }
    // The prepared native plan emits ASCII Space for the acknowledged closing
    // run. Firefox may encode those same keys as NBSP, without changing its count.
    let expected_body = expected.trim_end_matches(' ');
    let closing_spaces = expected.len() - expected_body.len();
    if closing_spaces == 0 || expected_body.is_empty() {
        return false;
    }
    let observed_body = observed.trim_end_matches(|ch| matches!(ch, ' ' | '\u{a0}'));
    observed[observed_body.len()..].chars().count() == closing_spaces
        && observed_body.ends_with(expected_body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firefox_processed_result_policy_acknowledged_space_readback_preserves_other_bytes() {
        for prefix in ["", "文 🌍 ", "prior\u{a0}word "] {
            for count in 1..=4 {
                let expected = format!("{prefix}changed{}", " ".repeat(count));
                for mask in 0..(1 << count) {
                    let actual_spaces: String = (0..count)
                        .map(|slot| {
                            if mask & (1 << slot) == 0 {
                                ' '
                            } else {
                                '\u{a0}'
                            }
                        })
                        .collect();
                    let observed = format!("{prefix}changed{actual_spaces}");
                    assert!(
                        acknowledged_space_tail_matches(&observed, &expected),
                        "acknowledged Firefox NBSP must settle without the full deadline: {observed:?} {expected:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn firefox_processed_result_policy_space_readback_refuses_changed_bytes_and_other_clients() {
        let expected = "文 prior\u{a0}changed ";
        for observed in [
            "文 prior\u{a0}changed",
            "文 prior\u{a0}changed\u{a0}\u{a0}",
            "文 prior\u{a0}changed \u{a0}",
            "文 prior changed\u{a0}",
            "foreign prior\u{a0}changed\u{a0}",
            "文 prior\u{a0}changed!\u{a0}",
            "文 prior\u{a0}changed\t",
            "文 prior\u{a0}changed\n",
            "文 prior\u{a0}changed\u{200b}",
        ] {
            assert!(
                !acknowledged_space_tail_matches(observed, expected),
                "{observed:?}"
            );
        }
        assert!(!acknowledged_space_tail_matches("changed\u{a0}", "changed"));
        assert!(!acknowledged_space_tail_matches(
            "changed ",
            "changed\u{a0}"
        ));
        for scoped in [false, true] {
            let policy = ProcessingReadiness {
                scoped,
                expected_focus: "window-a",
                epoch: 7,
                target_path: "/engine/target",
                target_layout_is_ru: false,
            };
            let queued_readback = readback_policy_for_client(scoped);
            assert!(policy.tail_matches("changed ", "changed "));
            assert!(queued_readback("changed ", "changed "));
            assert_eq!(queued_readback("changed\u{a0}", "changed "), scoped);
            assert_eq!(
                queued_readback("changed\u{a0}", "changed "),
                policy.tail_matches("changed\u{a0}", "changed "),
            );
            if !scoped {
                assert!(!policy.tail_matches("changed\u{a0}", "changed "));
            }
        }
    }

    #[test]
    fn firefox_processed_result_policy_keeps_prior_verified_scope_during_output() {
        let calls = std::cell::Cell::new(0);
        assert!(
            processing_scope(true, || {
                calls.set(calls.get() + 1);
                false
            }),
            "an already verified KDE Firefox operation must not fall to generic policy after a later unavailable classification"
        );
        assert_eq!(calls.get(), 0);
        assert!(processing_scope(false, || true));
        assert!(!processing_scope(false, || false));
    }

    #[test]
    fn firefox_processed_result_policy_requires_native_ack_and_current_focus() {
        for scoped in [false, true] {
            for processed in [Ok(false), Ok(true), Err("bridge unavailable".into())] {
                for focus in [Ok(()), Err("focus changed".into())] {
                    let expected = scoped && processed == Ok(true) && focus.is_ok();
                    let processed_calls = std::cell::Cell::new(0);
                    let focus_calls = std::cell::Cell::new(0);
                    assert_eq!(
                        retain_processed_result_on(
                            scoped,
                            || {
                                processed_calls.set(processed_calls.get() + 1);
                                processed.clone()
                            },
                            || {
                                focus_calls.set(focus_calls.get() + 1);
                                focus.clone()
                            },
                        ),
                        expected
                    );
                    assert_eq!(processed_calls.get(), usize::from(scoped));
                    assert_eq!(
                        focus_calls.get(),
                        usize::from(scoped && processed == Ok(true))
                    );
                }
            }
        }
    }

    #[test]
    fn firefox_processed_result_policy_keeps_desktop_client_and_exact_window_bound() {
        assert!(scoped_client(
            LayoutBackend::Kde,
            "window-a",
            "window-a",
            Path::new("/opt/firefox-bin")
        ));
        assert!(!scoped_client(
            LayoutBackend::Gnome,
            "window-a",
            "window-a",
            Path::new("/opt/firefox-bin")
        ));
        assert!(!scoped_client(
            LayoutBackend::Kde,
            "window-a",
            "window-b",
            Path::new("/opt/firefox-bin")
        ));
        assert!(!scoped_client(
            LayoutBackend::Kde,
            "",
            "",
            Path::new("/opt/firefox-bin")
        ));
        for exe in ["/opt/google-chrome", "/usr/bin/kitty", "/usr/bin/konsole"] {
            assert!(!scoped_client(
                LayoutBackend::Kde,
                "window-a",
                "window-a",
                Path::new(exe)
            ));
        }
    }
}
