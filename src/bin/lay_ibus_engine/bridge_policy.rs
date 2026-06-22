pub(crate) fn should_suppress_next_autocorrect(kind: &str) -> bool {
    matches!(
        kind,
        "layout-replay"
            | "smart-text"
            | "auto-replace"
            | "ime-replay"
            | "auto-undo"
            | "typing-assist"
            | "enter-autocorrect"
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn manual_replace_kinds_suppress_immediate_ime_autocorrect() {
        for kind in [
            "layout-replay",
            "smart-text",
            "auto-replace",
            "ime-replay",
            "auto-undo",
        ] {
            assert!(super::should_suppress_next_autocorrect(kind), "{kind}");
        }
    }

    #[test]
    fn daemon_boundary_autocorrect_kinds_suppress_inner_ime_repeat() {
        for kind in ["typing-assist", "enter-autocorrect"] {
            assert!(super::should_suppress_next_autocorrect(kind), "{kind}");
        }
    }
}
