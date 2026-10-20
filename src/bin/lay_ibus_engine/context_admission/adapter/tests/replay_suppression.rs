//! Replay suppression follows the reducer's authenticated same-field transfer.
use super::*;

#[test]
fn legacy_replay_suppression_survives_authenticated_factory_transfer_once() {
    zbus::block_on(bounded(async {
        for capabilities in [9, 41] {
            let mut harness = cycle09_harness().await;
            let mut source = cycle09_source(&mut harness).await;
            cycle09_add_trailing_boundary(&mut harness, &mut source).await;
            source.set_client_capabilities(capabilities);
            let (mut source, disposition) = cycle09_manual_toggle(&mut harness, source).await;
            assert_eq!(
                disposition,
                Ok((if capabilities == 41 { 3 } else { 2 }, false))
            );
            assert_eq!(cycle09_exact_handoff_is_live(&source), capabilities == 41);
            assert!(source.arm_legacy_replay_autocorrect_suppression());
            // Exercise factory observation, source FocusOut/Disable, target
            // capabilities/FocusIn, marker and install_context_activation.
            // bind_focus_path alone does not model this production transfer.
            let target_path = format!("{TARGET_PATH}_legacy_replay_{capabilities}");
            let mut target = cycle09_factory_handoff_with_capabilities(
                &mut harness,
                source,
                99_000,
                &target_path,
                None,
                "lay-ime-ru",
                (capabilities, false, None),
            )
            .await;
            assert!(target.live_context_token().is_some());
            assert_eq!(target.committed_tail.buffer, " abc ");
            assert!(
                target.take_manual_toggle_autocorrect_suppression(),
                "capabilities={capabilities}"
            );
            assert!(!target.take_manual_toggle_autocorrect_suppression());
            assert!(target
                .shared
                .lock()
                .unwrap()
                .autocorrect_suppression
                .is_none());
            cycle09_assert_no_local_text_effect(&mut harness).await;
        }
    }));
}
