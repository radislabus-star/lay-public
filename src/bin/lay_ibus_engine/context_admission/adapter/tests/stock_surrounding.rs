//! Stock clients attach to an already-enabled engine after its first Enable.
//! The current Ready publication must request their actual surrounding text;
//! the request itself must never manufacture a word boundary or snapshot.
use super::*;

async fn ready_surrounding_case(caps: u32, sensitive: bool) {
    let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
    let mut engine = new_engine(&harness);
    start_source_free_unknown(&mut harness, &mut engine).await;
    engine.set_client_capabilities(caps);
    if sensitive {
        engine.set_content_type_state(crate::window_interaction::IBUS_INPUT_PURPOSE_PASSWORD, 0);
    }
    engine.try_install_pending_context_activation();
    assert!(engine.live_context_token().is_some());
    assert!(engine.client_context.input_mode_property_refresh_pending);
    let before = (
        engine.context_owner.clone(),
        engine.context_token.clone(),
        engine.context_word_scope,
        engine.committed_tail.buffer.clone(),
        engine.committed_tail.epoch,
        engine.client_context.surrounding_observation_revision,
    );
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert!(!engine.context_word_is_known());
    let emitter = zbus::object_server::SignalEmitter::new(&harness.connection, TARGET_PATH)
        .expect("legacy output emitter");
    engine
        .publish_pending_input_mode_property(&mut EngineOutput::legacy(&emitter))
        .await;
    let effects = terminal_delivery::legacy_effects(&mut harness).await;
    let members: Vec<_> = effects
        .iter()
        .map(|message| message.header().member().unwrap().as_str().to_string())
        .collect();
    let expected = if caps & 32 != 0 && !sensitive {
        vec!["UpdateProperty", "RequireSurroundingText"]
    } else {
        vec!["UpdateProperty"]
    };
    assert_eq!(
        members, expected,
        "Ready must request actual surrounding text from the current capable client"
    );
    assert!(!engine.client_context.input_mode_property_refresh_pending);
    assert_eq!(
        (
            engine.context_owner.clone(),
            engine.context_token.clone(),
            engine.context_word_scope,
            engine.committed_tail.buffer.clone(),
            engine.committed_tail.epoch,
            engine.client_context.surrounding_observation_revision,
        ),
        before,
        "requesting a snapshot cannot confer edit authority"
    );
    assert!(engine.client_context.surrounding_text_snapshot.is_none());
    assert!(!engine.context_word_is_known());
    engine
        .publish_pending_input_mode_property(&mut EngineOutput::legacy(&emitter))
        .await;
    assert!(terminal_delivery::legacy_effects(&mut harness)
        .await
        .is_empty());
}

#[test]
fn c09_stock_ready_surrounding_requires_request_without_edit_authority() {
    zbus::block_on(bounded(ready_surrounding_case(41, false)));
}

#[test]
fn c09_stock_ready_surrounding_skips_unsupported_client() {
    zbus::block_on(bounded(ready_surrounding_case(9, false)));
}

#[test]
fn c09_stock_ready_surrounding_skips_sensitive_client() {
    zbus::block_on(bounded(ready_surrounding_case(41, true)));
}

#[test]
fn c09_stock_ready_surrounding_skips_owner_revoked_during_publication() {
    zbus::block_on(bounded(async {
        let mut harness = bootstrap_harness_with_budget(CALLBACK_BUDGET).await;
        let mut engine = new_engine(&harness);
        start_source_free_unknown(&mut harness, &mut engine).await;
        engine.set_client_capabilities(41);
        engine.try_install_pending_context_activation();
        let owner = engine.context_owner.clone().expect("installed Ready owner");
        assert!(engine.live_context_token().is_some());
        let mut effects = crate::output::TestEngineOutput {
            legacy_transport: true,
            pause_after_input_mode: true,
            ..Default::default()
        };
        {
            let mut output = EngineOutput::test(&mut effects);
            let mut pending = Box::pin(engine.publish_pending_input_mode_property(&mut output));
            assert!(future::poll_once(&mut pending).await.is_none());
            assert!(harness.adapter.revoke_current_owner(&owner));
            pending.await;
        }
        assert_eq!(effects.input_mode_updates.len(), 1);
        assert_eq!(
            effects.effects,
            ["register-input-mode"],
            "no request after owner revocation"
        );
        assert!(engine.live_context_token().is_none());
        assert!(!engine.client_context.input_mode_property_refresh_pending);
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert!(!engine.context_word_is_known());
    }));
}
