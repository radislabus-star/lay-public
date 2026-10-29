//! Select existing key pacing for an already authorized native replay.
//! This policy supplies no output, focus proof or mutation authority.

pub(super) fn use_fast_replay(
    input_isolated: bool,
    requires_client_settlement: bool,
    original_token: &str,
    leased_suffix: &str,
    expected_focus: &str,
) -> bool {
    select_fast_profile(
        input_isolated,
        requires_client_settlement,
        original_token,
        leased_suffix,
        || super::firefox_receipt::current_client_is_scoped(expected_focus),
    )
}

fn select_fast_profile(
    input_isolated: bool,
    requires_client_settlement: bool,
    original_token: &str,
    leased_suffix: &str,
    current_client_is_scoped: impl FnOnce() -> bool,
) -> bool {
    let word_is_open = leased_suffix == original_token;
    input_isolated && requires_client_settlement && word_is_open && current_client_is_scoped()
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "other",
        ] {
            assert!(
                !select_fast_profile(true, true, "source", suffix, || {
                    panic!("closed or mismatched suffix must not query the current client")
                }),
                "ineligible suffix {suffix:?}"
            );
        }
        for (isolated, native) in [(false, false), (false, true), (true, false)] {
            assert!(!select_fast_profile(
                isolated,
                native,
                "source",
                "source",
                || { panic!("ineligible replay must not query the current client") }
            ));
        }
    }
}
