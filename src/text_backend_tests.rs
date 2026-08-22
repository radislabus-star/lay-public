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
fn auto_backend_is_allowed_to_try_ime() {
    assert!(TextBackendPreference::Ime.should_try_ime());
    assert!(TextBackendPreference::Auto.should_try_ime());
    assert!(!TextBackendPreference::Uinput.should_try_ime());
}

#[test]
fn only_explicit_uinput_grants_daemon_text_mutation_authority() {
    assert!(TextBackendPreference::Uinput.daemon_owns_text_mutation());
    assert!(!TextBackendPreference::Ime.daemon_owns_text_mutation());
    assert!(!TextBackendPreference::Auto.daemon_owns_text_mutation());
}

#[test]
fn ime_request_counts_unicode_tail_chars() {
    let request = ImeReplaceRequest::committed_tail("привет ", "hello ");
    assert_eq!(request.backspaces, 7);
    assert_eq!(request.text, "hello ");
    assert!(!request.is_noop());
}

#[test]
fn ime_request_preserves_stable_context_prefix() {
    let request = ImeReplaceRequest::committed_tail("ри автозамене ", "ри автозаменае ");
    assert_eq!(request.backspaces, 2);
    assert_eq!(request.text, "ае ");

    let request = ImeReplaceRequest::committed_tail("Короче существет ", "Короче существует ");
    assert_eq!(request.backspaces, 3);
    assert_eq!(request.text, "ует ");
}

#[test]
fn exposes_backend_capabilities_for_decoder_policy() {
    assert!(!TextBackendCapabilities::uinput().can_atomic_replace());
    assert!(TextBackendCapabilities::uinput().can_switch_layout);
    assert!(TextBackendCapabilities::ime().can_atomic_replace());
    assert!(!TextBackendCapabilities::ime().can_switch_layout);
}
