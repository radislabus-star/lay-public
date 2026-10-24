use super::*;
use crate::text::preedit_attribute_geometry;

#[test]
fn desktop_completion_surface_is_explicit_and_separate() {
    assert_eq!(
        completion_presentation(LayoutBackend::Gnome),
        CompletionPresentation::Inline,
    );
    for backend in [LayoutBackend::Kde, LayoutBackend::Niri, LayoutBackend::X11] {
        assert_eq!(
            completion_presentation(backend),
            CompletionPresentation::Auxiliary,
        );
    }
}

#[test]
fn gnome_keeps_accepted_inline_attributes_across_a_kde_round_trip() {
    let expected = vec![(1, 0, 0, 2), (2, 0x888888, 2, 4), (1, 1, 2, 4)];
    for backend in [
        LayoutBackend::Gnome,
        LayoutBackend::Kde,
        LayoutBackend::Gnome,
    ] {
        let payload = make_preedit_ibus_text(backend, "а𐐀бв".into(), 2);
        let actual = preedit_attribute_geometry(&payload);
        if backend == LayoutBackend::Kde {
            assert_eq!(
                actual,
                [
                    (1, 0, 0, 2),
                    (2, 0, 2, 4),
                    (3, 0xffffff, 2, 4),
                    (1, 0, 2, 4)
                ],
            );
        } else {
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn other_backends_keep_pre_refactor_white_suffix_style() {
    for backend in [LayoutBackend::Niri, LayoutBackend::X11] {
        let payload = make_preedit_ibus_text(backend, "а𐐀бв".into(), 2);
        assert_eq!(
            preedit_attribute_geometry(&payload),
            [
                (1, 0, 0, 2),
                (2, 0, 2, 4),
                (3, 0xffffff, 2, 4),
                (1, 0, 2, 4),
            ],
            "{backend:?}: preserve the saved pre-refactor serializer contract",
        );
        let hint = make_hint_ibus_text(backend, "а𐐀б".into());
        assert_eq!(
            preedit_attribute_geometry(&hint),
            [(2, 0, 0, 3), (3, 0xffffff, 0, 3), (1, 0, 0, 3)],
            "{backend:?}: preserve the saved pre-refactor auxiliary style",
        );
    }
}

#[test]
fn desktop_styles_never_color_typed_text_or_extend_scalar_ranges() {
    for backend in [
        LayoutBackend::Gnome,
        LayoutBackend::Kde,
        LayoutBackend::Niri,
        LayoutBackend::X11,
    ] {
        for (text, start, expected) in [
            ("а𐐀б", u32::MAX, vec![(1, 0, 0, 3)]),
            ("а𐐀б", 3, vec![(1, 0, 0, 3)]),
            ("", 0, vec![]),
        ] {
            let payload = make_preedit_ibus_text(backend, text.into(), start);
            assert_eq!(preedit_attribute_geometry(&payload), expected);
        }
    }
}

#[test]
fn auxiliary_style_is_desktop_specific_and_empty_hints_are_unstyled() {
    assert_eq!(
        preedit_attribute_geometry(&make_hint_ibus_text(LayoutBackend::Gnome, "а𐐀б".into())),
        [(2, 0x888888, 0, 3)],
    );
    assert_eq!(
        preedit_attribute_geometry(&make_hint_ibus_text(LayoutBackend::Kde, "а𐐀б".into())),
        [(2, 0, 0, 3), (3, 0xffffff, 0, 3), (1, 0, 0, 3)],
    );
    for backend in [LayoutBackend::Gnome, LayoutBackend::Kde] {
        assert!(
            preedit_attribute_geometry(&make_hint_ibus_text(backend, String::new())).is_empty()
        );
    }
}
