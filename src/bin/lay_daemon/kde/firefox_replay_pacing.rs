//! Select existing key pacing for an already authorized native replay.
//! This policy supplies no output, focus proof or mutation authority.

pub(super) fn use_fast_replay(
    input_isolated: bool,
    requires_client_settlement: bool,
    original_token: &str,
    leased_suffix: &str,
    expected_focus: &str,
) -> bool {
    let mut scoped = None;
    let fast = select_fast_profile(
        input_isolated,
        requires_client_settlement,
        original_token,
        leased_suffix,
        || {
            let current = super::firefox_receipt::current_client_is_scoped(expected_focus);
            scoped = Some(current);
            current
        },
    );
    if lay::config::runtime_debug_action_log() {
        lay::debug_log::append_private_line(
            lay::debug_log::ibus_runtime_trace_path()
                .with_file_name("kde_exact_replay_debug.jsonl"),
            serde_json::json!({
                "kind": "kde_firefox_replay_pacing",
                "ts": lay::time::unix_timestamp(),
                "pid": std::process::id(),
                "input_isolated": input_isolated,
                "requires_client_settlement": requires_client_settlement,
                "original_chars": original_token.chars().count(),
                "suffix_chars": leased_suffix.chars().count(),
                "current_scoped_client": scoped,
                "fast_profile": fast,
            })
            .to_string(),
        );
    }
    fast
}

fn select_fast_profile(
    input_isolated: bool,
    requires_client_settlement: bool,
    original_token: &str,
    leased_suffix: &str,
    current_client_is_scoped: impl FnOnce() -> bool,
) -> bool {
    let exact_word_boundary = leased_suffix == original_token
        || leased_suffix
            .strip_prefix(original_token)
            .is_some_and(|closing| {
                !closing.is_empty() && closing.chars().all(|ch| matches!(ch, ' ' | '\u{a0}'))
            });
    input_isolated
        && requires_client_settlement
        && exact_word_boundary
        && current_client_is_scoped()
}

#[cfg(test)]
mod tests {
    use super::*;

    // The canonical runner gives this process its own config and HOME. It must
    // stay process-isolated because published runtime flags are process-global.
    #[test]
    fn debug_probe_preserves_published_config_until_explicit_reload() {
        let path = std::path::PathBuf::from(
            std::env::var_os(lay::config::CONFIG_PATH_ENV)
                .expect("run this test through the canonical hermetic runner"),
        );
        let private_config = std::path::PathBuf::from(
            std::env::var_os("XDG_CONFIG_HOME").expect("private hermetic config directory"),
        );
        assert_eq!(path, private_config.join("lay/config.toml"));
        assert!(!path.exists(), "the isolated config must start empty");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        for enabled in [false, true] {
            let published = serde_json::json!({
                "debug_action_log": enabled,
                "fragment_layout_autoflip": enabled,
            });
            std::fs::write(&path, published.to_string()).unwrap();
            let _ = lay::config::LayConfig::load();
            let pending = serde_json::json!({
                "debug_action_log": !enabled,
                "fragment_layout_autoflip": !enabled,
            });
            std::fs::write(&path, pending.to_string()).unwrap();

            assert!(!use_fast_replay(false, true, "source", "source ", ""));
            assert_eq!(
                lay::config::runtime_debug_action_log(),
                enabled,
                "a debug probe must not publish pending disk configuration"
            );
            assert_eq!(lay::config::runtime_fragment_layout_enabled(), enabled);

            let _ = lay::config::LayConfig::load();
            assert_eq!(lay::config::runtime_debug_action_log(), !enabled);
            assert_eq!(lay::config::runtime_fragment_layout_enabled(), !enabled);
        }
    }

    #[test]
    fn fast_pacing_requires_isolated_native_replay_and_current_scoped_client() {
        for token in ["source", "Слово", "e\u{301}"] {
            assert!(select_fast_profile(true, true, token, token, || true));
            assert!(!select_fast_profile(true, true, token, token, || false));
        }
        for suffix in [
            "source ",
            "source\u{a0}",
            "source  ",
            "source \u{a0}",
            "source\u{a0} ",
        ] {
            assert!(
                select_fast_profile(true, true, "source", suffix, || true),
                "acknowledged closed-word replay must use the bounded fast profile: {suffix:?}"
            );
            assert!(!select_fast_profile(true, true, "source", suffix, || false));
        }
        for suffix in [
            "other",
            "sourceword ",
            "source x",
            "source\t",
            "source\n",
            "source\u{2003}",
            " source ",
        ] {
            assert!(
                !select_fast_profile(true, true, "source", suffix, || {
                    panic!("mismatched suffix must not query the current client")
                }),
                "ineligible suffix {suffix:?}"
            );
        }
        for (isolated, native) in [(false, false), (false, true), (true, false)] {
            for suffix in ["source", "source ", "source\u{a0}"] {
                assert!(!select_fast_profile(
                    isolated,
                    native,
                    "source",
                    suffix,
                    || { panic!("ineligible replay must not query the current client") }
                ));
            }
        }
    }
}
