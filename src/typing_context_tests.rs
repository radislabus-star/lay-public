use crate::config::{
    default_typing_assist_pipeline, typing_assist_pipeline_for_policy, CorrectionSafety,
};
use crate::typing_context::{should_enable_ascii_to_ru_layout, typing_assist_pipeline_for_context};

#[test]
fn russian_context_enables_ascii_to_ru_layout_rule() {
    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
        "я ghbdtn ",
    );

    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));
    assert!(pipeline
        .iter()
        .find(|rule| rule.id == "contextual_layout_en_to_ru")
        .is_some_and(|rule| rule.enabled));
    assert!(should_enable_ascii_to_ru_layout("пишу ckjdf "));
    assert!(should_enable_ascii_to_ru_layout("проверяю Lfdfq "));
    assert!(should_enable_ascii_to_ru_layout("'nj "));
    assert!(should_enable_ascii_to_ru_layout("пишу 'nj "));
    assert!(should_enable_ascii_to_ru_layout("worked 'nj "));
    assert!(should_enable_ascii_to_ru_layout(
        "можно открыть Windows на NTFS и написать Lfdfq "
    ));
}

#[test]
fn no_context_or_english_context_keeps_ascii_to_ru_disabled() {
    let base = typing_assist_pipeline_for_policy(
        true,
        CorrectionSafety::Normal,
        &default_typing_assist_pipeline(),
    );
    assert!(base
        .iter()
        .find(|rule| rule.id == "layout_en_to_ru")
        .is_some_and(|rule| !rule.enabled));

    for context in [
        "ghbdtn ",
        "good ghbdtn ",
        "status; 'nj ",
        "git 'nj ",
        "wi-fi 'nj ",
        "njkmrj? vjue& yt$ hf,jnftn 100% 'nj ",
        "я good ",
        "я WPS ",
        "я wi-fi ",
    ] {
        assert!(
            !should_enable_ascii_to_ru_layout(context),
            "context={context:?}"
        );
        let pipeline = typing_assist_pipeline_for_context(
            true,
            CorrectionSafety::Normal,
            &default_typing_assist_pipeline(),
            context,
        );
        assert!(
            pipeline
                .iter()
                .all(|rule| rule.id != "contextual_layout_en_to_ru"),
            "context={context:?}"
        );
    }
}

#[test]
fn explicit_user_disabled_rule_stays_disabled() {
    let mut configured = default_typing_assist_pipeline();
    configured
        .iter_mut()
        .find(|rule| rule.id == "layout_en_to_ru")
        .expect("layout_en_to_ru rule")
        .enabled = false;

    let pipeline = typing_assist_pipeline_for_context(
        true,
        CorrectionSafety::Normal,
        &configured,
        "я ghbdtn ",
    );

    assert!(pipeline
        .iter()
        .all(|rule| rule.id != "contextual_layout_en_to_ru"));
}
