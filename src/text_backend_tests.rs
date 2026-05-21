use super::*;

#[test]
fn parses_text_backend_preference() {
    assert_eq!(
        TextBackendPreference::parse("uinput"),
        TextBackendPreference::Uinput
    );
    assert_eq!(
        TextBackendPreference::parse("ime"),
        TextBackendPreference::Ime
    );
    assert_eq!(
        TextBackendPreference::parse("IBUS"),
        TextBackendPreference::Ime
    );
    assert_eq!(
        TextBackendPreference::parse("auto"),
        TextBackendPreference::Auto
    );
    assert_eq!(
        TextBackendPreference::parse("unknown"),
        TextBackendPreference::Uinput
    );
}

#[test]
fn ime_request_counts_unicode_tail_chars() {
    let request = ImeReplaceRequest::committed_tail("привет ", "hello ");
    assert_eq!(request.backspaces, 7);
    assert_eq!(request.text, "hello ");
    assert!(!request.is_noop());
}

#[test]
fn exposes_backend_capabilities_for_decoder_policy() {
    assert!(!TextBackendCapabilities::uinput().can_atomic_replace());
    assert!(TextBackendCapabilities::uinput().can_switch_layout);
    assert!(TextBackendCapabilities::ime().can_atomic_replace());
    assert!(!TextBackendCapabilities::ime().can_switch_layout);
}
