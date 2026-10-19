//! A matching actual observation refreshes a current manual lease, never creates one.
use super::*;

async fn observed_manual_target(
    harness: &mut Harness,
    text: &str,
    hints: u32,
    early: bool,
) -> (LayIbusEngine, Instant) {
    let mut source = td159_fresh_focus_source(harness).await;
    source.set_content_type_state(0, hints);
    let cursor = text.chars().count() as u32;
    cycle09_surrounding_receipt(harness, &mut source, text, cursor, cursor).await;
    let (source, result, _) = cycle09_manual_toggle_with_effects(harness, source).await;
    assert_eq!(result, Ok((3, false)));
    let expires_at = harness
        .adapter
        .shared
        .reducer
        .lock()
        .unwrap()
        .exact_manual_snapshot
        .as_ref()
        .unwrap()
        .expires_at;
    let path = format!("{TARGET_PATH}_observation_refresh");
    let snapshot = early.then(|| Some(SurroundingTextSnapshot::new(text.into(), cursor, cursor)));
    let target = cycle09_factory_handoff_with_capabilities(
        harness,
        source,
        8980,
        &path,
        snapshot,
        "lay-ime-ru",
        (169, true, None),
    )
    .await;
    assert!(target.exact_snapshot_manual_source_is_current());
    (target, expires_at)
}

#[test]
fn td159_exact_manual_observation_order_preserves_current_lease() {
    zbus::block_on(bounded(async {
        for (text, hints) in [("abcdef", 0), ("πrefix abcdef", 0), ("prefix abcdef", 2048)] {
            for early in [false, true] {
                let mut harness = cycle09_harness().await;
                let (mut target, expires_at) =
                    observed_manual_target(&mut harness, text, hints, early).await;
                let cursor = text.chars().count() as u32;
                for _ in 0..2 {
                    cycle09_surrounding_receipt(&mut harness, &mut target, text, cursor, cursor)
                        .await;
                    assert!(target.exact_snapshot_manual_source_is_current(),
                        "matching target observation must preserve the current exact manual lease: early={early}, hints={hints}, text={text:?}");
                    assert_eq!(
                        harness
                            .adapter
                            .shared
                            .reducer
                            .lock()
                            .unwrap()
                            .exact_manual_snapshot
                            .as_ref()
                            .unwrap()
                            .expires_at,
                        expires_at,
                        "matching observation must not extend the original lease"
                    );
                    assert!(!target.context_word_is_known());
                    assert_eq!(
                        target
                            .context_word_scope
                            .as_ref()
                            .unwrap()
                            .lineage()
                            .observed_suffix_chars,
                        0
                    );
                }
                let tail = target.committed_tail.buffer.clone();
                let epoch = target.committed_tail.epoch;
                let path = target.path.clone();
                let (target, visible) = private_visible_tail_v4(&mut harness, target).await;
                let visible = visible.unwrap();
                assert_eq!(
                    (visible.1.as_str(), visible.3, visible.4.as_str()),
                    (tail.as_str(), epoch, path.as_str())
                );
                assert_eq!(visible.6, hints == 0);
                let (target, suppression) = cycle09_suppress_exact_replay(
                    &mut harness,
                    target,
                    &tail,
                    epoch,
                    path.as_str(),
                    true,
                )
                .await;
                assert_eq!(suppression, Ok(true));
                assert!(cycle09_exact_suppression_is_armed(&target));
                assert!(!target.context_word_is_known());
            }
        }
    }));
}

#[test]
fn td159_exact_manual_observation_changed_or_revoked_lease_cannot_revive() {
    zbus::block_on(bounded(async {
        for refusal in [
            "prefix",
            "tail",
            "selection",
            "caret",
            "missing",
            "expired",
            "consumed",
            "revoked",
            "sensitive",
        ] {
            let mut harness = cycle09_harness().await;
            let (mut target, _) =
                observed_manual_target(&mut harness, "prefix abcdef", 0, true).await;
            match refusal {
                "prefix" => {
                    cycle09_surrounding_receipt(&mut harness, &mut target, "otherx abcdef", 13, 13)
                        .await
                }
                "tail" => {
                    cycle09_surrounding_receipt(&mut harness, &mut target, "prefix abcdeg", 13, 13)
                        .await
                }
                "selection" => {
                    cycle09_surrounding_receipt(&mut harness, &mut target, "prefix abcdef", 13, 7)
                        .await
                }
                "caret" => {
                    cycle09_surrounding_receipt(&mut harness, &mut target, "prefix abcdef", 12, 12)
                        .await
                }
                "missing" => target.observe_external_surrounding_text(None),
                "expired" => {
                    target.shared.lock().unwrap().preserve_active_path_until =
                        Some(Instant::now() - Duration::from_secs(1))
                }
                "consumed" => harness
                    .adapter
                    .invalidate_exact_manual_snapshot(target.context_owner.as_ref().unwrap()),
                "revoked" => assert!(harness
                    .adapter
                    .revoke_current_owner(target.context_owner.as_ref().unwrap())),
                "sensitive" => target.set_content_type_state(8, 0),
                _ => unreachable!(),
            }
            cycle09_surrounding_receipt(&mut harness, &mut target, "prefix abcdef", 13, 13).await;
            assert!(
                !target.exact_snapshot_manual_source_is_current(),
                "{refusal}"
            );
            assert!(!target.context_word_is_known(), "{refusal}");
            let (_target, visible) = private_visible_tail_v4(&mut harness, target).await;
            assert_eq!(visible.unwrap().0, "passive:unknown-context", "{refusal}");
        }
    }));
}

#[test]
fn td159_exact_manual_observation_alone_cannot_create_manual_lease() {
    zbus::block_on(bounded(async {
        let mut harness = cycle09_harness().await;
        let mut engine = td159_fresh_focus_source(&mut harness).await;
        for _ in 0..2 {
            cycle09_surrounding_receipt(&mut harness, &mut engine, "prefix abcdef", 13, 13).await;
            assert!(!engine.exact_snapshot_manual_source_is_current());
            assert!(!engine.context_word_is_known());
            assert!(harness
                .adapter
                .shared
                .reducer
                .lock()
                .unwrap()
                .exact_manual_snapshot
                .is_none());
        }
    }));
}
