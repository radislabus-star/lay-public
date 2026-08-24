use zbus::fdo;

use super::bridge::LayImeBridge;
use super::engine::{LayIbusEngine, ManualToggleAuthority};
use super::output::EngineOutput;
use super::state::CommittedTailReplaceRequest;
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};
use std::time::{Duration, Instant};

const DAEMON_DELEGATED_LAYOUT_HANDOFF_WINDOW: Duration = Duration::from_millis(700);

impl LayImeBridge {
    pub(super) fn active_path(&self) -> Option<String> {
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .clone()
    }

    pub(super) async fn input_state_inner(&self) -> fdo::Result<String> {
        let Some(path) = self.active_path() else {
            return Ok("passive:no-focus".to_string());
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let engine = iface_ref.get().await;
        Ok(tail_source_for_authority(engine.manual_toggle_authority())
            .bridge_state()
            .to_string())
    }

    pub(super) async fn visible_tail_v1_inner(&self) -> fdo::Result<(String, String, bool)> {
        let (state, text, layout, _, _) = self.visible_tail_v2_inner().await?;
        Ok((state, text, layout))
    }

    pub(super) async fn visible_tail_v2_inner(
        &self,
    ) -> fdo::Result<(String, String, bool, u64, String)> {
        let Some(path) = self.active_path() else {
            return Ok((
                "passive:no-focus".to_string(),
                String::new(),
                false,
                0,
                String::new(),
            ));
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let mut engine = iface_ref.get_mut().await;
        engine.refresh_empty_tail_from_handoff();
        let source = tail_source_for_authority(engine.manual_toggle_authority());
        let text = visible_text_for_source(source, &engine.buffer, &engine.tail_buffer);
        Ok((
            source.bridge_state().to_string(),
            text,
            engine.layout_is_ru,
            engine.tail_epoch,
            path,
        ))
    }

    pub(super) async fn owns_active_text_inner(&self) -> fdo::Result<bool> {
        let Some(path) = self.active_path() else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let engine = iface_ref.get().await;
        Ok(engine.manual_toggle_authority() == ManualToggleAuthority::ImeActiveComposition)
    }

    pub(super) async fn can_replace_committed_tail_inner(
        &self,
        backspaces: u32,
    ) -> fdo::Result<bool> {
        let Some(path) = self.active_path() else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let engine = iface_ref.get().await;
        Ok(engine.can_replace_committed_tail(backspaces))
    }

    pub(super) async fn suppress_next_autocorrect_inner(&self) -> fdo::Result<bool> {
        let Some(path) = self.active_path() else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let mut engine = iface_ref.get_mut().await;
        engine.suppress_next_committed_tail_autocorrect = true;
        engine.publish_autocorrect_suppression_handoff();
        super::trace::record(r#"{"kind":"ibus_suppress_next_autocorrect","source":"daemon"}"#);
        Ok(true)
    }

    pub(super) async fn replace_tail_inner(
        &self,
        backspaces: u32,
        text: String,
        suppress_next_autocorrect: bool,
        expected_original_tail: Option<String>,
        expected_revision: Option<(u64, String)>,
    ) -> fdo::Result<bool> {
        if backspaces == 0 && text.is_empty() {
            return Ok(false);
        }
        let Some(path) = self.active_path() else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let emitter = iface_ref.signal_emitter();
        let mut engine = iface_ref.get_mut().await;
        if engine.atomic_route_active {
            return Ok(false);
        }
        let expected_tail = expected_original_tail.map(|expected| {
            let (epoch, focus) = expected_revision
                .clone()
                .unwrap_or_else(|| (engine.tail_epoch, path.clone()));
            VisibleTailSnapshot::new(
                VisibleTailSource::DaemonWordBuffer,
                expected,
                Some(focus),
                epoch,
            )
        });
        let mut request =
            CommittedTailReplaceRequest::daemon_bridge(backspaces, text, suppress_next_autocorrect);
        if let Some(expected_tail) = expected_tail {
            request = request.with_expected_tail(expected_tail);
        }
        let mut output = EngineOutput::legacy(emitter);
        engine.replace_committed_tail(&mut output, request).await
    }

    pub(super) async fn manual_toggle_inner(&self) -> fdo::Result<bool> {
        Ok(self.manual_toggle_v2_inner().await?.0)
    }

    pub(super) async fn manual_toggle_v2_inner(&self) -> fdo::Result<(bool, bool)> {
        Ok(self.manual_toggle_outcome_inner().await?.as_legacy_v2())
    }

    pub(super) async fn manual_toggle_v3_inner(&self) -> fdo::Result<(u8, bool)> {
        Ok(self.manual_toggle_outcome_inner().await?.as_v3())
    }

    async fn manual_toggle_outcome_inner(&self) -> fdo::Result<ImeManualToggleOutcome> {
        let Some(path) = self.active_path() else {
            return Ok(ImeManualToggleOutcome::NotHandled);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let emitter = iface_ref.signal_emitter();
        let mut engine = iface_ref.get_mut().await;
        let atomic_route_active = engine.atomic_route_active;
        if atomic_route_active {
            return Ok(manual_toggle_outcome_for_authority(
                atomic_route_active,
                engine.manual_toggle_authority(),
                None,
            ));
        }
        engine.refresh_empty_tail_from_handoff();
        let authority = engine.manual_toggle_authority();
        let mut output = EngineOutput::legacy(emitter);
        let target_layout_is_ru = engine.manual_toggle_active_text_target(&mut output).await?;
        arm_daemon_delegated_layout_handoff(&engine, authority, target_layout_is_ru);
        Ok(manual_toggle_outcome_for_authority(
            atomic_route_active,
            authority,
            target_layout_is_ru,
        ))
    }
}

fn arm_daemon_delegated_layout_handoff(
    engine: &LayIbusEngine,
    authority: ManualToggleAuthority,
    target_layout_is_ru: Option<bool>,
) {
    if target_layout_is_ru.is_some()
        || authority != ManualToggleAuthority::DaemonWordBuffer
        || engine.tail_buffer.is_empty()
    {
        return;
    }

    if !engine.arm_daemon_delegated_layout_handoff(
        Instant::now() + DAEMON_DELEGATED_LAYOUT_HANDOFF_WINDOW,
    ) {
        return;
    }
    super::trace::record(format!(
        r#"{{"kind":"ibus_manual_toggle_handoff","route":"daemon_delegated_layout","preserve_ms":{},"tail_chars":{},"tail_epoch":{}}}"#,
        DAEMON_DELEGATED_LAYOUT_HANDOFF_WINDOW.as_millis(),
        engine.tail_buffer.chars().count(),
        engine.tail_epoch,
    ));
}

fn manual_toggle_outcome_for_authority(
    atomic_route_active: bool,
    authority: ManualToggleAuthority,
    target_layout_is_ru: Option<bool>,
) -> ImeManualToggleOutcome {
    if atomic_route_active {
        return ImeManualToggleOutcome::NotHandled;
    }
    match target_layout_is_ru {
        Some(target_layout_is_ru) => ImeManualToggleOutcome::handled(target_layout_is_ru),
        None if authority == ManualToggleAuthority::DaemonWordBuffer => {
            ImeManualToggleOutcome::DelegateDaemon
        }
        None => ImeManualToggleOutcome::NotHandled,
    }
}

fn tail_source_for_authority(authority: ManualToggleAuthority) -> VisibleTailSource {
    match authority {
        ManualToggleAuthority::ImeActiveComposition => VisibleTailSource::ImeActiveComposition,
        ManualToggleAuthority::ImeCommittedTail => VisibleTailSource::ImeCommittedTail,
        ManualToggleAuthority::DaemonWordBuffer => VisibleTailSource::DaemonWordBuffer,
    }
}

fn visible_text_for_source(
    source: VisibleTailSource,
    active_composition: &str,
    committed_tail: &str,
) -> String {
    match source {
        VisibleTailSource::ImeActiveComposition => active_composition.to_string(),
        VisibleTailSource::ImeCommittedTail | VisibleTailSource::DaemonWordBuffer => {
            committed_tail.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    #[test]
    fn manual_toggle_delegates_only_non_atomic_daemon_authority() {
        assert_eq!(
            manual_toggle_outcome_for_authority(
                false,
                ManualToggleAuthority::DaemonWordBuffer,
                None,
            ),
            ImeManualToggleOutcome::DelegateDaemon
        );
        for authority in [
            ManualToggleAuthority::DaemonWordBuffer,
            ManualToggleAuthority::ImeActiveComposition,
            ManualToggleAuthority::ImeCommittedTail,
        ] {
            assert_eq!(
                manual_toggle_outcome_for_authority(true, authority, None),
                ImeManualToggleOutcome::NotHandled
            );
        }
        assert_eq!(
            manual_toggle_outcome_for_authority(
                false,
                ManualToggleAuthority::ImeActiveComposition,
                None,
            ),
            ImeManualToggleOutcome::NotHandled
        );
    }

    #[test]
    fn daemon_authority_exposes_its_typed_ime_observation_without_claiming_ime_ownership() {
        assert_eq!(
            visible_text_for_source(
                VisibleTailSource::DaemonWordBuffer,
                "ignored-composition",
                "prefix ytn",
            ),
            "prefix ytn"
        );
        assert_eq!(
            VisibleTailSource::DaemonWordBuffer.bridge_state(),
            "passive:daemon-word-buffer"
        );
    }

    #[test]
    fn daemon_rpc_delegation_preserves_exact_tail_across_one_layout_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut source = LayIbusEngine::new(
            "/engine/us".to_string(),
            Arc::clone(&shared),
            false,
            true,
            LayConfig::default(),
        );
        assert!(source.bind_focus_path());
        source.tail_buffer = "file ghjdthrf".to_string();
        source.publish_tail_handoff();
        let expected_epoch = source.tail_epoch;
        let expected_focus = source.focus_receipt.clone();

        arm_daemon_delegated_layout_handoff(&source, ManualToggleAuthority::DaemonWordBuffer, None);

        let mut target = LayIbusEngine::new(
            "/engine/ru".to_string(),
            shared,
            true,
            true,
            LayConfig::default(),
        );
        assert!(target.bind_focus_path());
        assert_eq!(target.tail_buffer, "file ghjdthrf");
        assert_eq!(target.tail_epoch, expected_epoch);
        assert_eq!(target.focus_receipt, expected_focus);
        assert_eq!(
            target.manual_toggle_authority(),
            ManualToggleAuthority::DaemonWordBuffer
        );
        let state = target.shared.lock().expect("shared state");
        let handoff = state
            .daemon_delegated_layout_handoff
            .as_ref()
            .expect("typed delegated handoff");
        assert_eq!(handoff.source_path, "/engine/us");
        assert_eq!(handoff.target_path.as_deref(), Some("/engine/ru"));
        assert!(handoff.target_layout_is_ru);
        assert_eq!(handoff.tail_epoch, expected_epoch);
    }

    #[test]
    fn handled_or_empty_daemon_reply_does_not_arm_layout_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/engine/us".to_string(),
            Arc::clone(&shared),
            false,
            true,
            LayConfig::default(),
        );
        assert!(engine.bind_focus_path());

        arm_daemon_delegated_layout_handoff(&engine, ManualToggleAuthority::DaemonWordBuffer, None);
        assert!(shared
            .lock()
            .expect("shared state")
            .preserve_active_path_until
            .is_none());
        assert!(shared
            .lock()
            .expect("shared state")
            .daemon_delegated_layout_handoff
            .is_none());

        engine.tail_buffer = "ghjdthrf".to_string();
        engine.publish_tail_handoff();
        arm_daemon_delegated_layout_handoff(
            &engine,
            ManualToggleAuthority::DaemonWordBuffer,
            Some(true),
        );
        assert!(shared
            .lock()
            .expect("shared state")
            .preserve_active_path_until
            .is_none());
    }

    #[test]
    fn local_shift_delegation_cannot_arm_cross_path_tail_inheritance() {
        let production = include_str!("shift.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(!production.contains("publish_active_path_preserve_handoff"));
    }
}
