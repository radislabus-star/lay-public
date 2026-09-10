use zbus::fdo;

use super::bridge::LayImeBridge;
use super::context_admission::AdmissionToken;
use super::engine::{LayIbusEngine, ManualToggleAuthority};
use super::output::EngineOutput;
use super::state::CommittedTailReplaceRequest;
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};

impl LayImeBridge {
    async fn bridge_admission_token(&self) -> fdo::Result<Option<AdmissionToken>> {
        if !self.context_admission_required {
            return Ok(None);
        }
        let admission = self
            .admission
            .as_ref()
            .ok_or_else(|| fdo::Error::Failed("context admission unavailable".to_string()))?;
        let fence = admission
            .begin_bridge_fence()
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        admission
            .complete_bridge_fence(fence)
            .await
            .map(Some)
            .map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    pub(crate) fn bridge_token_is_live(
        &self,
        engine: &LayIbusEngine,
        token: Option<&AdmissionToken>,
    ) -> bool {
        if !self.context_admission_required {
            return true;
        }
        let (Some(admission), Some(token)) = (self.admission.as_ref(), token) else {
            return false;
        };
        admission.revalidate_bridge(token)
            && engine
                .live_context_token()
                .as_ref()
                .is_some_and(|live| live == token)
    }

    pub(super) fn active_path(&self) -> Option<String> {
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .clone()
    }

    pub(super) async fn input_state_inner(&self) -> fdo::Result<String> {
        let token = self.bridge_admission_token().await?;
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
        if !self.bridge_token_is_live(&engine, token.as_ref()) {
            return Ok("passive:unknown-context".to_string());
        }
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
        let (state, text, layout, epoch, path, _) = self.visible_tail_v3_inner().await?;
        Ok((state, text, layout, epoch, path))
    }

    pub(super) async fn visible_tail_v3_inner(
        &self,
    ) -> fdo::Result<(String, String, bool, u64, String, String)> {
        let token = self.bridge_admission_token().await?;
        let Some(path) = self.active_path() else {
            return Ok((
                "passive:no-focus".to_string(),
                String::new(),
                false,
                0,
                String::new(),
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
        if !self.bridge_token_is_live(&engine, token.as_ref()) || !engine.context_word_is_known() {
            return Ok((
                "passive:unknown-context".to_string(),
                String::new(),
                engine.layout_gesture.layout_is_ru,
                engine.committed_tail.epoch,
                path,
                String::new(),
            ));
        }
        engine.refresh_empty_tail_from_handoff();
        let source = tail_source_for_authority(engine.manual_toggle_authority());
        let text = visible_text_for_source(
            source,
            &engine.composition.buffer,
            &engine.committed_tail.buffer,
        );
        let focus_receipt = engine
            .client_context
            .focus_receipt
            .clone()
            .unwrap_or_default();
        Ok((
            source.bridge_state().to_string(),
            text,
            engine.layout_gesture.layout_is_ru,
            engine.committed_tail.epoch,
            path,
            focus_receipt,
        ))
    }

    pub(super) async fn owns_active_text_inner(&self) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
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
        if !self.bridge_token_is_live(&engine, token.as_ref()) {
            return Ok(false);
        }
        Ok(engine.manual_toggle_authority() == ManualToggleAuthority::ImeActiveComposition)
    }

    pub(super) async fn can_replace_committed_tail_inner(
        &self,
        backspaces: u32,
    ) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
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
        if !self.bridge_token_is_live(&engine, token.as_ref()) || !engine.context_word_is_known() {
            return Ok(false);
        }
        Ok(engine.can_replace_committed_tail(backspaces))
    }

    pub(super) async fn suppress_next_autocorrect_inner(&self) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
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
        if !self.bridge_token_is_live(&engine, token.as_ref()) || !engine.context_word_is_known() {
            return Ok(false);
        }
        engine.consume_exact_manual_toggle_handoff();
        if !engine.arm_legacy_replay_autocorrect_suppression() {
            return Ok(false);
        }
        super::trace::record(r#"{"kind":"ibus_suppress_next_autocorrect","source":"daemon"}"#);
        Ok(true)
    }

    pub(super) async fn suppress_next_autocorrect_v2_inner(
        &self,
        expected_suffix: String,
        expected_epoch: u64,
        expected_path: String,
        expected_layout_is_ru: bool,
    ) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
        if expected_path.is_empty() || self.active_path().as_deref() != Some(expected_path.as_str())
        {
            return Ok(false);
        }
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(expected_path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let mut engine = iface_ref.get_mut().await;
        if !self.bridge_token_is_live(&engine, token.as_ref()) || !engine.context_word_is_known() {
            return Ok(false);
        }
        let accepted = engine.arm_exact_manual_toggle_autocorrect_suppression(
            &expected_suffix,
            expected_epoch,
            &expected_path,
            expected_layout_is_ru,
        );
        super::trace::record(format!(
            r#"{{"kind":"ibus_suppress_next_autocorrect","source":"daemon_exact","status":"{}","epoch":{},"path":{:?}}}"#,
            if accepted { "accepted" } else { "rejected" },
            expected_epoch,
            expected_path,
        ));
        Ok(accepted)
    }

    pub(super) async fn cancel_exact_manual_toggle_handoff_v2_inner(
        &self,
        expected_epoch: u64,
        expected_path: String,
    ) -> bool {
        let Ok(token) = self.bridge_admission_token().await else {
            return false;
        };
        if self.context_admission_required {
            let Some(path) = self.active_path() else {
                return false;
            };
            let Ok(iface_ref) = self
                .ibus_connection
                .object_server()
                .interface::<_, LayIbusEngine>(path.as_str())
                .await
            else {
                return false;
            };
            let engine = iface_ref.get().await;
            if !self.bridge_token_is_live(&engine, token.as_ref()) {
                return false;
            }
        }
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        if !cancel_exact_manual_toggle_handoff_state(&mut state, expected_epoch, &expected_path) {
            return false;
        }
        super::trace::record(format!(
            r#"{{"kind":"ibus_exact_manual_toggle_handoff","status":"cancelled_exact","epoch":{},"path":{:?}}}"#,
            expected_epoch, expected_path,
        ));
        true
    }

    pub(super) async fn cancel_exact_manual_toggle_suppression_v2_inner(
        &self,
        expected_epoch: u64,
        expected_path: String,
    ) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
        if expected_path.is_empty() {
            return Ok(false);
        }
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(expected_path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let mut engine = iface_ref.get_mut().await;
        if !self.bridge_token_is_live(&engine, token.as_ref()) {
            return Ok(false);
        }
        let cancelled = engine
            .revoke_exact_manual_toggle_autocorrect_suppression(expected_epoch, &expected_path);
        super::trace::record(format!(
            r#"{{"kind":"ibus_suppress_next_autocorrect","source":"daemon_exact","status":"{}","epoch":{},"path":{:?}}}"#,
            if cancelled {
                "cancelled"
            } else {
                "cancel_rejected"
            },
            expected_epoch,
            expected_path,
        ));
        Ok(cancelled)
    }

    pub(super) async fn replace_tail_inner(
        &self,
        backspaces: u32,
        text: String,
        suppress_next_autocorrect: bool,
        expected_original_tail: Option<String>,
        expected_revision: Option<(u64, String)>,
    ) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
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
        if !self.bridge_token_is_live(&engine, token.as_ref()) || !engine.context_word_is_known() {
            return Ok(false);
        }
        if engine.atomic.active {
            return Ok(false);
        }
        let expected_tail = expected_original_tail.map(|expected| {
            let (epoch, focus) = expected_revision
                .clone()
                .unwrap_or_else(|| (engine.committed_tail.epoch, path.clone()));
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
        let mut engine = engine.begin_context_bridge_output(token.as_ref());
        let mut output = EngineOutput::legacy(emitter);
        let result = engine.replace_committed_tail(&mut output, request).await?;
        engine.complete();
        Ok(result)
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
        let token = self.bridge_admission_token().await?;
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
        if !self.bridge_token_is_live(&engine, token.as_ref())
            || !engine.context_allows_manual_toggle()
        {
            return Ok(ImeManualToggleOutcome::NotHandled);
        }
        let atomic_route_active = engine.atomic.active;
        if atomic_route_active {
            return Ok(manual_toggle_outcome_for_authority(
                atomic_route_active,
                engine.manual_toggle_authority(),
                None,
            ));
        }
        let mut engine = engine.begin_context_bridge_output(token.as_ref());
        engine.refresh_empty_tail_from_handoff();
        let authority = engine.manual_toggle_authority();
        let complete_word = engine.context_word_is_known();
        let mut output = EngineOutput::legacy(emitter);
        let target_layout_is_ru = engine.manual_toggle_active_text_target(&mut output).await?;
        engine.complete();
        if !complete_word && target_layout_is_ru.is_none() {
            return Ok(ImeManualToggleOutcome::NotHandled);
        }
        Ok(manual_toggle_outcome_for_authority(
            atomic_route_active,
            authority,
            target_layout_is_ru,
        ))
    }
}

pub(crate) fn cancel_exact_manual_toggle_handoff_state(
    state: &mut super::protocol::SharedState,
    expected_epoch: u64,
    expected_path: &str,
) -> bool {
    if state.exact_manual_toggle_handoff_epoch != Some(expected_epoch)
        || state.exact_manual_toggle_handoff_path.as_deref() != Some(expected_path)
    {
        return false;
    }
    state.preserve_active_path_until = None;
    state.exact_manual_toggle_handoff_epoch = None;
    state.exact_manual_toggle_handoff_path = None;
    state.handoff_tail_buffer.clear();
    state.handoff_focus_receipt = None;
    state.suppression_revision = state.suppression_revision.wrapping_add(1);
    true
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
        None if authority == ManualToggleAuthority::ImeCommittedTail => {
            ImeManualToggleOutcome::DelegateExactImeTail
        }
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
    use std::time::{Duration, Instant};

    #[test]
    fn manual_toggle_delegates_each_passive_authority_to_its_typed_daemon_route() {
        assert_eq!(
            manual_toggle_outcome_for_authority(
                false,
                ManualToggleAuthority::DaemonWordBuffer,
                None,
            ),
            ImeManualToggleOutcome::DelegateDaemon
        );
        assert_eq!(
            manual_toggle_outcome_for_authority(
                false,
                ManualToggleAuthority::ImeCommittedTail,
                None,
            ),
            ImeManualToggleOutcome::DelegateExactImeTail
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
    fn exact_handoff_cancellation_requires_matching_path_and_epoch() {
        let mut state = super::super::protocol::SharedState {
            handoff_tail_buffer: "ghbdtn".to_string(),
            handoff_tail_epoch: 17,
            handoff_focus_receipt: Some("focus".to_string()),
            preserve_active_path_until: Some(Instant::now() + Duration::from_secs(1)),
            exact_manual_toggle_handoff_epoch: Some(17),
            exact_manual_toggle_handoff_path: Some("/engine/us".to_string()),
            ..Default::default()
        };

        assert!(!cancel_exact_manual_toggle_handoff_state(
            &mut state,
            18,
            "/engine/us"
        ));
        assert!(!cancel_exact_manual_toggle_handoff_state(
            &mut state,
            17,
            "/engine/ru"
        ));
        assert_eq!(state.handoff_tail_buffer, "ghbdtn");
        assert!(cancel_exact_manual_toggle_handoff_state(
            &mut state,
            17,
            "/engine/us"
        ));
        assert!(state.handoff_tail_buffer.is_empty());
        assert!(state.exact_manual_toggle_handoff_epoch.is_none());
        assert!(state.exact_manual_toggle_handoff_path.is_none());
    }
}
