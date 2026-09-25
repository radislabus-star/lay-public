use super::execution::ExecutionReceipt;
use super::{TextTargetAuthority, WindowInteraction, WindowRejectReason};
/// Shared output-backend capability selector for committed-text targets.
///
/// GUI targets must expose SurroundingText and an exact current snapshot before
/// deletion. Terminal targets use their separately proven erase executor. All
/// other targets remain read-only for edits that delete committed text.
/// Request-specific selection, sensitive-content, snapshot, path, owner and
/// epoch authority stays with the existing route executors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextTargetEditRoute {
    CommitOnly,
    ExactSurroundingText,
    TerminalErase,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextTargetDecisionReason {
    AppendNeedsNoDelete,
    SurroundingTextDelete,
    ProvenTerminalErase,
    MissingProvenDelete,
}

impl TextTargetDecisionReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::AppendNeedsNoDelete => "append_needs_no_delete_capability",
            Self::SurroundingTextDelete => "surrounding_text_delete_capability",
            Self::ProvenTerminalErase => "proven_terminal_erase_capability",
            Self::MissingProvenDelete => "missing_proven_delete_capability",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextTargetCapabilityFacts {
    pub(crate) backspaces: u32,
    pub(crate) surrounding_text_supported: bool,
    pub(crate) terminal_erase_supported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextTargetDecision {
    pub(crate) route: TextTargetEditRoute,
    pub(crate) reason: TextTargetDecisionReason,
}

impl TextTargetDecision {
    pub(crate) fn from_facts(facts: TextTargetCapabilityFacts) -> Self {
        if facts.backspaces == 0 {
            return Self {
                route: TextTargetEditRoute::CommitOnly,
                reason: TextTargetDecisionReason::AppendNeedsNoDelete,
            };
        }
        if facts.surrounding_text_supported {
            return Self {
                route: TextTargetEditRoute::ExactSurroundingText,
                reason: TextTargetDecisionReason::SurroundingTextDelete,
            };
        }
        if facts.terminal_erase_supported {
            return Self {
                route: TextTargetEditRoute::TerminalErase,
                reason: TextTargetDecisionReason::ProvenTerminalErase,
            };
        }
        Self {
            route: TextTargetEditRoute::Unsupported,
            reason: TextTargetDecisionReason::MissingProvenDelete,
        }
    }
}

impl TextTargetEditRoute {
    #[cfg(test)]
    pub(crate) fn select(
        backspaces: u32,
        surrounding_text_supported: bool,
        terminal_erase_supported: bool,
    ) -> Self {
        TextTargetDecision::from_facts(TextTargetCapabilityFacts {
            backspaces,
            surrounding_text_supported,
            terminal_erase_supported,
        })
        .route
    }

    pub(crate) fn output_route(self) -> &'static str {
        match self {
            Self::CommitOnly => "commit",
            Self::ExactSurroundingText => "surrounding_text_delete_commit",
            Self::TerminalErase => "terminal_erase_commit",
            Self::Unsupported => "no_proven_delete_backend",
        }
    }

    pub(crate) fn uses_terminal_erase(self) -> bool {
        matches!(self, Self::TerminalErase)
    }

    pub(crate) fn can_execute(self) -> bool {
        !matches!(self, Self::Unsupported)
    }
}

use zbus::fdo;

use crate::bridge::LayImeBridge;
use crate::context_admission::{AdapterError, AdmissionToken};
use crate::engine::{LayIbusEngine, ManualToggleAuthority};
use crate::output::EngineOutput;
use crate::state::CommittedTailReplaceRequest;
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};

enum BridgeAdmissionError {
    Unavailable,
    Adapter(AdapterError),
}

impl BridgeAdmissionError {
    fn into_fdo(self) -> fdo::Error {
        match self {
            Self::Unavailable => fdo::Error::Failed("context admission unavailable".to_string()),
            Self::Adapter(error) => fdo::Error::Failed(error.to_string()),
        }
    }
}

impl LayImeBridge {
    async fn bridge_admission_token_result(
        &self,
    ) -> Result<Option<AdmissionToken>, BridgeAdmissionError> {
        if !self.context_admission_required {
            return Ok(None);
        }
        let admission = self
            .admission
            .as_ref()
            .ok_or(BridgeAdmissionError::Unavailable)?;
        let fence = admission
            .begin_bridge_fence()
            .await
            .map_err(BridgeAdmissionError::Adapter)?;
        admission
            .complete_bridge_fence(fence)
            .await
            .map(Some)
            .map_err(BridgeAdmissionError::Adapter)
    }

    async fn bridge_admission_token(&self) -> fdo::Result<Option<AdmissionToken>> {
        self.bridge_admission_token_result()
            .await
            .map_err(BridgeAdmissionError::into_fdo)
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

    pub(crate) fn active_path(&self) -> Option<String> {
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .clone()
    }

    fn bridge_engine_path(&self, token: Option<&AdmissionToken>) -> Option<String> {
        if self.context_admission_required {
            return token.map(|token| token.owner_path().to_string());
        }
        self.active_path()
    }

    fn shared_active_path_matches(&self, expected_path: &str) -> bool {
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .as_deref()
            == Some(expected_path)
    }

    pub(crate) async fn input_state_inner(&self) -> fdo::Result<String> {
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

    pub(crate) async fn visible_tail_v1_inner(&self) -> fdo::Result<(String, String, bool)> {
        let (state, text, layout, _, _) = self.visible_tail_v2_inner().await?;
        Ok((state, text, layout))
    }

    pub(crate) async fn visible_tail_v2_inner(
        &self,
    ) -> fdo::Result<(String, String, bool, u64, String)> {
        let (state, text, layout, epoch, path, _) = self.visible_tail_v3_inner().await?;
        Ok((state, text, layout, epoch, path))
    }

    pub(crate) async fn visible_tail_v3_inner(
        &self,
    ) -> fdo::Result<(String, String, bool, u64, String, String)> {
        let token = match self.bridge_admission_token_result().await {
            Ok(token) => token,
            Err(BridgeAdmissionError::Adapter(AdapterError::Denied)) => {
                return Ok((
                    "passive:unknown-context".to_string(),
                    String::new(),
                    false,
                    0,
                    String::new(),
                    String::new(),
                ));
            }
            Err(error) => return Err(error.into_fdo()),
        };
        let Some(path) = self.bridge_engine_path(token.as_ref()) else {
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
        if self.context_admission_required {
            engine.try_install_pending_context_activation();
        }
        let word_is_known = engine.context_word_is_known();
        let bounded_unknown_suffix = engine.context_exact_manual_handoff_bounds_unknown_suffix();
        let observed_exact_suffix = engine.context_observed_suffix_exact_manual_handoff_allowed();
        let reset_rereceipt = engine.context_reset_rereceipt_exact_manual_handoff_allowed();
        if !self.shared_active_path_matches(&path)
            || !self.bridge_token_is_live(&engine, token.as_ref())
            || !(word_is_known
                || bounded_unknown_suffix
                || observed_exact_suffix
                || reset_rereceipt)
        {
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
        if source == VisibleTailSource::ImeCommittedTail
            && engine.exact_manual_toggle_handoff_is_bound_to_current_owner()
            && (!engine.exact_manual_toggle_handoff_is_live()
                || !(engine.current_external_snapshot_agrees_with_owned_tail()
                    || engine.inherited_exact_manual_snapshot_agrees_with_owned_tail()))
        {
            engine.clear_identity_bound_exact_manual_toggle_authority();
            return Ok((
                "passive:unknown-context".to_string(),
                String::new(),
                engine.layout_gesture.layout_is_ru,
                engine.committed_tail.epoch,
                path,
                String::new(),
            ));
        }
        let text = visible_text_for_source(
            source,
            &engine.composition.buffer,
            &engine.committed_tail.buffer,
        );
        let focus_receipt = if self.context_admission_required {
            token
                .as_ref()
                .map(AdmissionToken::exact_field_receipt)
                .unwrap_or_default()
        } else {
            engine
                .client_context
                .focus_receipt
                .clone()
                .unwrap_or_default()
        };
        Ok((
            source.bridge_state().to_string(),
            text,
            engine.layout_gesture.layout_is_ru,
            engine.committed_tail.epoch,
            path,
            focus_receipt,
        ))
    }

    pub(crate) async fn owns_active_text_inner(&self) -> fdo::Result<bool> {
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

    pub(crate) async fn can_replace_committed_tail_inner(
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

    pub(crate) async fn suppress_next_autocorrect_inner(&self) -> fdo::Result<bool> {
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
        crate::trace::record(r#"{"kind":"ibus_suppress_next_autocorrect","source":"daemon"}"#);
        Ok(true)
    }

    pub(crate) async fn suppress_next_autocorrect_v2_inner(
        &self,
        expected_suffix: String,
        expected_epoch: u64,
        expected_path: String,
        expected_layout_is_ru: bool,
    ) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
        let Some(path) = self.bridge_engine_path(token.as_ref()) else {
            return Ok(false);
        };
        if expected_path.is_empty() || path != expected_path {
            return Ok(false);
        }
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let mut engine = iface_ref.get_mut().await;
        if self.context_admission_required {
            engine.try_install_pending_context_activation();
        }
        let word_is_known = engine.context_word_is_known();
        let bounded_unknown_suffix = engine.context_exact_manual_handoff_bounds_unknown_suffix();
        if !self.shared_active_path_matches(&path)
            || !self.bridge_token_is_live(&engine, token.as_ref())
            || !(word_is_known || bounded_unknown_suffix)
        {
            return Ok(false);
        }
        let accepted = engine.arm_exact_manual_toggle_autocorrect_suppression(
            &expected_suffix,
            expected_epoch,
            &expected_path,
            expected_layout_is_ru,
        );
        crate::trace::record(format!(
            r#"{{"kind":"ibus_suppress_next_autocorrect","source":"daemon_exact","status":"{}","epoch":{},"path":{:?}}}"#,
            if accepted { "accepted" } else { "rejected" },
            expected_epoch,
            expected_path,
        ));
        Ok(accepted)
    }

    pub(crate) async fn cancel_exact_manual_toggle_handoff_v2_inner(
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
        crate::trace::record(format!(
            r#"{{"kind":"ibus_exact_manual_toggle_handoff","status":"cancelled_exact","epoch":{},"path":{:?}}}"#,
            expected_epoch, expected_path,
        ));
        true
    }

    pub(crate) async fn cancel_exact_manual_toggle_suppression_v2_inner(
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
        crate::trace::record(format!(
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

    pub(crate) async fn replace_tail_inner(
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
        let authority = WindowInteraction::admit_replace(&engine, backspaces);
        let mut engine = engine.begin_context_bridge_output(token.as_ref());
        let mut output = EngineOutput::legacy(emitter);
        let result =
            match WindowInteraction::execute_local(&mut engine, authority, request, &mut output)
                .await
            {
                Ok(ExecutionReceipt::LocalComplete) => true,
                Ok(ExecutionReceipt::Rejected) => false,
                Ok(_) => false,
                Err(failure) => {
                    let receipt = failure.receipt();
                    crate::trace::record(format!(
                        r#"{{"kind":"window_interaction_execution","receipt":"{receipt:?}"}}"#
                    ));
                    debug_assert!(matches!(
                        receipt,
                        ExecutionReceipt::LocalIndeterminatePartial
                            | ExecutionReceipt::LocalErrorBeforeMutation
                    ));
                    return Err(failure.into());
                }
            };
        engine.complete();
        Ok(result)
    }

    pub(crate) async fn manual_toggle_inner(&self) -> fdo::Result<bool> {
        Ok(self.manual_toggle_v2_inner().await?.0)
    }

    pub(crate) async fn manual_toggle_v2_inner(&self) -> fdo::Result<(bool, bool)> {
        Ok(self.manual_toggle_outcome_inner().await?.as_legacy_v2())
    }

    pub(crate) async fn manual_toggle_v3_inner(&self) -> fdo::Result<(u8, bool)> {
        Ok(self.manual_toggle_outcome_inner().await?.as_v3())
    }

    async fn manual_toggle_outcome_inner(&self) -> fdo::Result<ImeManualToggleOutcome> {
        let token = match self.bridge_admission_token().await {
            Ok(token) => token,
            Err(error) => {
                crate::trace::record(
                    r#"{"kind":"ibus_manual_toggle_rpc","stage":"rejected","reason":"bridge_admission"}"#,
                );
                return Err(error);
            }
        };
        let Some(path) = self.active_path() else {
            crate::trace::record(
                r#"{"kind":"ibus_manual_toggle_rpc","stage":"not_handled","reason":"no_active_path"}"#,
            );
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
        let reset_snapshot_pending = engine.context_reset_rereceipt_computation_allowed()
            && engine.client_context.surrounding_text_supported
            && engine.client_context.surrounding_text_snapshot.is_none();
        let manual_toggle_allowed = engine.context_allows_manual_toggle()
            || engine.context_observed_suffix_exact_manual_handoff_allowed()
            || engine.context_reset_rereceipt_exact_manual_handoff_allowed()
            || reset_snapshot_pending;
        let bridge_token_live = self.bridge_token_is_live(&engine, token.as_ref());
        if !bridge_token_live || !manual_toggle_allowed {
            crate::trace::record(format!(
                r#"{{"kind":"ibus_manual_toggle_rpc","stage":"not_handled","reason":"context_authority","bridge_token_live":{bridge_token_live},"manual_toggle_allowed":{manual_toggle_allowed}}}"#,
            ));
            return Ok(ImeManualToggleOutcome::NotHandled);
        }
        let atomic_route_active = engine.atomic.active;
        if atomic_route_active {
            crate::trace::record(
                r#"{"kind":"ibus_manual_toggle_rpc","stage":"not_handled","reason":"atomic_route"}"#,
            );
            return Ok(ImeManualToggleOutcome::NotHandled);
        }
        let mut engine = engine.begin_context_bridge_output(token.as_ref());
        engine.refresh_empty_tail_from_handoff();
        let target_authority = WindowInteraction::admit_manual_toggle(&engine);
        if matches!(
            target_authority,
            TextTargetAuthority::AtomicOwned | TextTargetAuthority::Reject(_)
        ) {
            crate::trace::record(
                r#"{"kind":"ibus_manual_toggle_rpc","stage":"not_handled","reason":"target_authority"}"#,
            );
            return Ok(ImeManualToggleOutcome::NotHandled);
        }
        let mut output = EngineOutput::legacy(emitter);
        let (target_layout_is_ru, execution) =
            WindowInteraction::execute_manual_toggle(&mut engine, target_authority, &mut output)
                .await
                .map_err(fdo::Error::from)?;
        let outcome = manual_toggle_outcome_from_execution(target_layout_is_ru, execution);
        let (status, target_layout_is_ru) = outcome.as_v3();
        crate::trace::record(format!(
            r#"{{"kind":"ibus_manual_toggle_rpc","stage":"complete","status":{status},"target_layout_is_ru":{target_layout_is_ru}}}"#,
        ));
        engine.complete();
        Ok(outcome)
    }
}

pub(crate) fn cancel_exact_manual_toggle_handoff_state(
    state: &mut crate::protocol::SharedState,
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

fn manual_toggle_outcome_from_execution(
    target_layout_is_ru: Option<bool>,
    execution: ExecutionReceipt,
) -> ImeManualToggleOutcome {
    match execution {
        ExecutionReceipt::LocalComplete
        | ExecutionReceipt::LocalPending
        | ExecutionReceipt::LocalCancelled => target_layout_is_ru
            .map(ImeManualToggleOutcome::handled)
            .unwrap_or(ImeManualToggleOutcome::NotHandled),
        ExecutionReceipt::DelegatedExactImeTail => ImeManualToggleOutcome::DelegateExactImeTail,
        ExecutionReceipt::DelegatedDaemonBuffer => ImeManualToggleOutcome::DelegateDaemon,
        ExecutionReceipt::LocalIndeterminatePartial
        | ExecutionReceipt::LocalErrorBeforeMutation
        | ExecutionReceipt::Rejected => ImeManualToggleOutcome::NotHandled,
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

impl LayIbusEngine {
    pub(crate) fn manual_toggle_authority(&self) -> ManualToggleAuthority {
        if !self.composition.buffer.is_empty() {
            return ManualToggleAuthority::ImeActiveComposition;
        }
        let committed_tail_chars = self.last_tail_token_text().chars().count() as u32;
        // Generic cursor geometry is not proof that CommitText control
        // characters can delete client text. An explicit terminal purpose plus
        // an executable terminal-erase profile is such proof for terminals
        // that do not expose SurroundingText (notably Kitty).
        let terminal_erase_supported = self.terminal_committed_tail_executor_available();
        if committed_tail_chars > 0
            && (self.client_context.surrounding_text_supported || terminal_erase_supported)
        {
            return ManualToggleAuthority::ImeCommittedTail;
        }
        ManualToggleAuthority::DaemonWordBuffer
    }
    pub(crate) fn text_target_edit_route(&self, backspaces: u32) -> TextTargetEditRoute {
        self.text_target_decision(backspaces).route
    }
    pub(crate) fn text_target_decision(&self, backspaces: u32) -> TextTargetDecision {
        TextTargetDecision::from_facts(TextTargetCapabilityFacts {
            backspaces,
            surrounding_text_supported: self.client_context.surrounding_text_supported,
            terminal_erase_supported: self.has_proven_terminal_input()
                && self.client_context.cursor_cell_width > 0,
        })
    }
    pub(crate) fn terminal_committed_tail_executor_available(&self) -> bool {
        let committed_tail_chars = self.last_tail_token_text().chars().count() as u32;
        committed_tail_chars > 0
            && self.text_target_edit_route(committed_tail_chars)
                == TextTargetEditRoute::TerminalErase
    }
}

impl WindowInteraction {
    pub(crate) fn admit_manual_toggle(engine: &LayIbusEngine) -> TextTargetAuthority {
        if engine.atomic.active {
            return TextTargetAuthority::AtomicOwned;
        }
        if engine.content_is_sensitive() {
            return TextTargetAuthority::Reject(WindowRejectReason::SensitiveContent);
        }
        match engine.manual_toggle_authority() {
            ManualToggleAuthority::ImeActiveComposition => {
                TextTargetAuthority::ActiveCompositionOwned
            }
            ManualToggleAuthority::ImeCommittedTail
                if engine.terminal_committed_tail_executor_available() =>
            {
                TextTargetAuthority::LocalTerminalErase
            }
            ManualToggleAuthority::ImeCommittedTail => TextTargetAuthority::DelegateExactImeTail,
            ManualToggleAuthority::DaemonWordBuffer => TextTargetAuthority::DelegateDaemonBuffer,
        }
    }

    pub(crate) fn admit_replace(engine: &LayIbusEngine, backspaces: u32) -> TextTargetAuthority {
        if engine.atomic.active {
            return TextTargetAuthority::AtomicOwned;
        }
        if engine.content_is_sensitive() {
            return TextTargetAuthority::Reject(WindowRejectReason::SensitiveContent);
        }
        if engine
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.has_selection())
        {
            return TextTargetAuthority::Reject(WindowRejectReason::SelectionPresent);
        }
        match engine.text_target_edit_route(backspaces) {
            TextTargetEditRoute::ExactSurroundingText | TextTargetEditRoute::CommitOnly => {
                TextTargetAuthority::LocalSurroundingDeleteCommit
            }
            TextTargetEditRoute::TerminalErase => TextTargetAuthority::LocalTerminalErase,
            TextTargetEditRoute::Unsupported => {
                TextTargetAuthority::Reject(WindowRejectReason::MissingProvenDeleteCapability)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn manual_toggle_projects_only_the_actual_execution_disposition() {
        assert_eq!(
            manual_toggle_outcome_from_execution(None, ExecutionReceipt::DelegatedDaemonBuffer),
            ImeManualToggleOutcome::DelegateDaemon
        );
        assert_eq!(
            manual_toggle_outcome_from_execution(None, ExecutionReceipt::DelegatedExactImeTail),
            ImeManualToggleOutcome::DelegateExactImeTail
        );
        for execution in [
            ExecutionReceipt::Rejected,
            ExecutionReceipt::LocalErrorBeforeMutation,
            ExecutionReceipt::LocalIndeterminatePartial,
        ] {
            assert_eq!(
                manual_toggle_outcome_from_execution(None, execution),
                ImeManualToggleOutcome::NotHandled
            );
        }
        assert_eq!(
            manual_toggle_outcome_from_execution(Some(true), ExecutionReceipt::LocalPending,),
            ImeManualToggleOutcome::handled(true)
        );
        assert_eq!(
            manual_toggle_outcome_from_execution(Some(false), ExecutionReceipt::LocalCancelled,),
            ImeManualToggleOutcome::handled(false)
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
        let mut state = crate::protocol::SharedState {
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
