//! Optional letter trigger. Ownership and all writing remain in existing actors.

use super::engine::{InputFrameIdentity, LayIbusEngine};
use super::output::EngineOutput;
use super::protocol::{has_command_modifier, is_key_press};
use super::shift::WordLayoutOrigin;
use super::window_interaction::{ExecutionReceipt, WindowInteraction};

#[path = "fragment_layout/reply.rs"]
pub(crate) mod reply;

/// Callback-local evidence of a paired letter, never authority to edit a word.
pub(super) struct FragmentLetterRelease {
    keycode: u32,
    focus_serial: u64,
    letter: char,
    native: bool,
}

impl LayIbusEngine {
    pub(super) fn capture_fragment_letter_release(
        &self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> Option<FragmentLetterRelease> {
        if !self.config.fragment_layout_autoflip
            || !self.live_composition_enabled()
            || is_key_press(state)
            || has_command_modifier(state)
        {
            return None;
        }
        let letter = self
            .physical_char(keyval, keycode)
            .filter(|ch| ch.is_alphabetic())?;
        let typed = if self.composition.buffer.is_empty() {
            &self.committed_tail.buffer
        } else {
            &self.composition.buffer
        };
        if !typed.ends_with(letter) {
            return None;
        }
        let managed = self
            .layout_gesture
            .handled_press_keycodes
            .contains(&keycode);
        let native = self
            .layout_gesture
            .native_letter_release_focus
            .get(&keycode)
            == Some(&self.client_context.focus_serial)
            && self.uses_native_terminal_input();
        (managed || native).then_some(FragmentLetterRelease {
            keycode,
            focus_serial: self.client_context.focus_serial,
            letter,
            native: !managed && native,
        })
    }

    fn fragment_letter_release_is_current(
        &self,
        release: &FragmentLetterRelease,
        handled: bool,
    ) -> bool {
        release.focus_serial == self.client_context.focus_serial
            && self.context_callback_entered.is_none()
            && !self
                .layout_gesture
                .handled_press_keycodes
                .contains(&release.keycode)
            && !self
                .layout_gesture
                .native_letter_release_focus
                .contains_key(&release.keycode)
            && if release.native {
                self.uses_native_terminal_input()
                    && self.capture_space_autocorrect_frame_identity().is_some()
            } else {
                handled
            }
            && (!self.context_admission_required || self.live_context_token().is_some())
    }

    pub(super) async fn request_fragment_after_letter_release(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        release: FragmentLetterRelease,
        handled: bool,
    ) {
        if !self.fragment_letter_release_is_current(&release, handled) {
            return;
        }
        // The canonical key path has retired the paired release unchanged.
        // The automatic request uses its verified current InputFrame. The
        // quiet-input bridge fence cannot be borrowed while letters continue.
        self.consider_fragment_layout_letter(output, release.letter)
            .await;
    }

    /// An unfinished mode may differ from the selected factory after a
    /// fragment flip. Its first explicit return completes that same mode.
    pub(super) fn fragment_word_mode_return_needed(&self) -> bool {
        self.config.fragment_layout_autoflip
            && self.config.auto_switch_layout
            && self.composition.buffer.is_empty()
            && !self.committed_tail.buffer.is_empty()
            && !self
                .committed_tail
                .buffer
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace)
            && self.layout_gesture.layout_is_ru
                != self
                    .client_context
                    .factory_engine_profile
                    .initial_layout_is_ru()
    }

    pub(super) fn fragment_manual_mode_handoff_matches(
        &self,
        expected_suffix: &str,
        expected_epoch: u64,
        expected_path: &str,
        expected_is_ru: bool,
        target_is_ru: bool,
    ) -> bool {
        self.config.auto_switch_layout
            && self.config.active_layout_backend() == lay::desktop::LayoutBackend::Gnome
            && self.layout_gesture.fragment_word_mode_generation
                == Some(self.layout_gesture.layout_generation)
            && self.content_allows_text_assistance()
            && self.composition.buffer.is_empty()
            && !self.composition.legacy_word_preedit_active
            && !expected_suffix
                .trim_end_matches(char::is_whitespace)
                .is_empty()
            && self.last_tail_token_text() == expected_suffix.trim_end_matches(char::is_whitespace)
            && self.committed_tail.buffer.ends_with(expected_suffix)
            && self.path == expected_path
            && self.committed_tail.epoch == expected_epoch
            && self.layout_gesture.layout_is_ru == expected_is_ru
            && expected_is_ru != target_is_ru
            && self
                .shared
                .lock()
                .is_ok_and(|state| state.active_path.as_deref() == Some(self.path.as_str()))
            && (self.context_word_is_known()
                || self.context_exact_manual_handoff_bounds_unknown_suffix()
                || self.exact_snapshot_manual_source_is_current())
            && self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_none_or(|snapshot| !snapshot.has_selection())
            && (self.current_external_snapshot_agrees_with_owned_tail()
                || self.inherited_exact_manual_snapshot_agrees_with_owned_tail()
                || self.context_stock_owned_manual_source_is_current())
            && self.exact_manual_toggle_handoff_is_live()
    }

    pub(super) fn fragment_manual_return_has_mode_origin(&self) -> bool {
        self.config.auto_switch_layout
            && self.config.active_layout_backend() == lay::desktop::LayoutBackend::Gnome
            && self.layout_gesture.fragment_word_mode_generation
                == Some(self.layout_gesture.layout_generation)
            && self.composition.buffer.is_empty()
            && !self.last_tail_token_text().is_empty()
            && self.layout_gesture.layout_is_ru
                != self
                    .client_context
                    .factory_engine_profile
                    .initial_layout_is_ru()
    }

    pub(super) async fn complete_fragment_word_mode(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        target_is_ru: bool,
    ) -> zbus::fdo::Result<()> {
        if !self.config.auto_switch_layout || !self.content_allows_text_assistance() {
            return Err(zbus::fdo::Error::Failed(
                "fragment mode completion is disabled".into(),
            ));
        }
        if self.config.active_layout_backend() == lay::desktop::LayoutBackend::Gnome
            && (!self.composition.buffer.is_empty() || self.composition.legacy_word_preedit_active)
        {
            return Err(zbus::fdo::Error::Failed(
                "fragment stack completion requires an already committed word".into(),
            ));
        }
        self.invalidate_input_frame_background_work();
        super::layout_sync::supersede_active_ime_engine_switch();
        let native = self
            .capture_fragment_native_mode_continuation(target_is_ru)
            .map_err(|reason| zbus::fdo::Error::Failed(reason.into()))?;
        let token = self.live_layout_intent_token();
        if self.context_admission_required && token.is_none() {
            return Err(zbus::fdo::Error::Failed(
                "fragment mode has no current layout intent".into(),
            ));
        }
        output.register_input_mode(target_is_ru).await?;
        self.sync_layout_for_fragment_word_mode(target_is_ru, token)
            .map_err(zbus::fdo::Error::Failed)?;
        if let Some(ticket) = native {
            self.finish_fragment_native_mode_continuation(ticket)
                .map_err(|reason| zbus::fdo::Error::Failed(reason.into()))?;
            self.arm_fragment_native_mode_observer(output).await;
        } else {
            self.set_layout_is_ru(target_is_ru);
        }
        // The existing writer already published and settled its tail. Mode
        // completion changes no word epoch and cannot settle it a second time.
        Ok(())
    }

    /// This origin may use an independently observed complete native word.
    /// Readout alone cannot admit a write or promote UnknownStart to KnownStart.
    pub(super) fn capture_fragment_layout_frame(&self) -> Option<InputFrameIdentity> {
        if let Some(frame) = self.capture_space_autocorrect_frame_identity() {
            return Some(frame);
        }
        if !self.native_document_is_current()
            || !self.current_external_snapshot_agrees_with_owned_tail()
        {
            return None;
        }
        self.capture_observed_suffix_display_frame()
    }

    fn trace_fragment_refusal(&self, status: &str) {
        if super::trace::enabled() && self.content_allows_text_assistance() {
            super::trace::record(format!(
                r#"{{"kind":"ibus_letter_fragment_admission","status":"{status}","auto_switch":{},"suppressed":{},"manual_pending":{},"known_word":{},"tail_chars":{},"composition_chars":{},"composition_cursor":{},"ru":{}}}"#,
                self.config.auto_switch_layout,
                self.committed_tail.autocorrect_suppression.is_some(),
                self.layout_gesture.pending_manual_toggle,
                self.context_word_is_known(),
                self.committed_tail.buffer.chars().count(),
                self.composition.buffer.chars().count(),
                self.composition.cursor,
                self.layout_gesture.layout_is_ru,
            ));
        }
    }

    pub(super) async fn consider_fragment_layout_letter(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        letter: char,
    ) {
        let started = std::time::Instant::now();
        let Some(request) = self.fragment_layout_letter_request(letter) else {
            return;
        };
        self.apply_fragment_layout_request(output, request, started)
            .await;
    }

    fn fragment_layout_letter_is_allowed(&self, letter: char) -> bool {
        self.config.fragment_layout_autoflip
            && self.config.auto_switch_layout
            && letter.is_alphabetic()
            && self.content_allows_text_assistance()
            && self.committed_tail.autocorrect_suppression.is_none()
            && !self.layout_gesture.pending_manual_toggle
            && self.committed_tail.pending_visible_postcondition.is_none()
            && (self.composition.buffer.is_empty()
                || self.composition.cursor == self.composition.buffer.chars().count())
    }

    fn fragment_layout_letter_request(
        &self,
        letter: char,
    ) -> Option<lay::layout_fragments::FragmentLayoutRequest> {
        if !self.fragment_layout_letter_is_allowed(letter) {
            self.trace_fragment_refusal("guard_refused");
            return None;
        }
        let token = if self.composition.buffer.is_empty() {
            self.committed_tail
                .buffer
                .split_whitespace()
                .next_back()
                .unwrap_or("")
        } else {
            &self.composition.buffer
        };
        let Some(request) =
            lay::layout_fragments::on_letter(token, letter, self.layout_gesture.layout_is_ru)
        else {
            self.trace_fragment_refusal("no_proposal");
            return None;
        };
        Some(request)
    }

    fn fragment_layout_source_is_current(&self) -> bool {
        // Read readiness only. A native client can normalize an older separator
        // while this known word remains exact. The complete frame and existing
        // verifier below still decide whether any replacement is permitted.
        self.current_external_snapshot_agrees_with_owned_tail()
            || (self.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde
                && self.context_word_is_known()
                && self.native_managed_readout_is_current())
    }

    fn fragment_native_observation_is_pending(&self, delete_chars: u32) -> bool {
        self.composition.buffer.is_empty()
            && (self.capture_fragment_layout_frame().is_none()
                || !self.fragment_layout_source_is_current()
                || !self.text_target_edit_route(delete_chars).can_execute())
    }

    async fn apply_fragment_layout_request(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        request: lay::layout_fragments::FragmentLayoutRequest,
        started: std::time::Instant,
    ) {
        let delete_chars = request.action().from_text().chars().count() as u32;
        if self.fragment_native_observation_is_pending(delete_chars) {
            // A complete input frame can coexist with a native snapshot one
            // local append behind. Reuse the existing bounded observation;
            // neither this proposal nor a cached frame grants edit authority.
            for read in 0..2 {
                if !matches!(
                    WindowInteraction::refresh_native_document(self, output).await,
                    Ok(true)
                ) {
                    self.trace_fragment_refusal("native_observation_failed");
                    return;
                }
                if self.client_context.native_document_binding.is_none()
                    || self.fragment_layout_source_is_current()
                {
                    break;
                }
                if read == 1 {
                    self.trace_fragment_refusal("native_observation_pending");
                    return;
                }
                // CommitText can precede the client's actual document update.
                // One more existing bounded read may observe that update. No
                // proposal or cached frame substitutes for current proof.
                async_io::Timer::after(std::time::Duration::from_millis(1)).await;
            }
        }
        let Some(frame) = self.capture_fragment_layout_frame() else {
            self.trace_fragment_refusal("complete_frame_unavailable");
            return;
        };
        if request.action().from_text() != frame.observed_token
            || request.target_layout_is_ru() == frame.active_layout_is_ru
        {
            self.trace_fragment_refusal("proposal_source_changed");
            return;
        }
        if self.composition.legacy_word_preedit_active
            && self.composition.preedit_display_only_pending
            && self.exact_owned_legacy_preedit_is_current()
            && Box::pin(self.retire_pending_precognition(output))
                .await
                .is_err()
        {
            self.trace_fragment_refusal("preedit_preparation_failed");
            return;
        }
        if !self.input_frame_authority_matches(&frame)
            || self.capture_fragment_layout_frame().as_ref() != Some(&frame)
        {
            self.trace_fragment_refusal("frame_changed");
            return;
        }
        // Keep this optional trigger out of the ordinary key future's stack
        // footprint. Allocate the existing executor future only for an admitted
        // positive proposal, never while the option is OFF or merely scoring.
        let outcome =
            Box::pin(self.request_word_layout_flip(output, WordLayoutOrigin::Fragment(&request)))
                .await;
        let status = match outcome {
            Ok((Some(_), ExecutionReceipt::LocalComplete)) => {
                self.layout_gesture.fragment_word_mode_generation =
                    Some(self.layout_gesture.layout_generation);
                "applied"
            }
            Ok(_) => "not_applied",
            Err(error) => {
                if error.receipt() == ExecutionReceipt::LocalIndeterminatePartial {
                    self.close_committed_tail_field();
                    self.client_context.surrounding_text_snapshot = None;
                    self.revoke_context_word();
                }
                "execution_failed"
            }
        };
        super::trace::record(format!(
            r#"{{"kind":"ibus_letter_fragment_layout","status":"{status}","elapsed_us":{}}}"#,
            started.elapsed().as_micros()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{EngineOutput, TestEngineOutput};
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn composing(backend: &str, enabled: bool, text: &str, is_ru: bool) -> LayIbusEngine {
        let mut engine = LayIbusEngine::new(
            "/test/fragment-letter".into(),
            Arc::new(Mutex::new(Default::default())),
            is_ru,
            true,
            LayConfig {
                text_backend: "ime".into(),
                layout_backend: backend.into(),
                fragment_layout_autoflip: enabled,
                auto_switch_layout: true,
                ..LayConfig::default()
            },
        );
        engine.composition.buffer = text.into();
        engine.composition.cursor = text.chars().count();
        engine.composition.preedit_visible = true;
        engine.committed_tail.buffer = text.into();
        engine.committed_tail.last_input_at = Some(std::time::Instant::now());
        engine.shared.lock().unwrap().active_path = Some(engine.path.clone());
        engine
    }

    #[test]
    fn fragment_request_reuses_verified_commit_in_both_desktops_and_directions() {
        for backend in ["gnome", "kde"] {
            for (input, expected, is_ru) in [("ghbdtn", "привет", false), ("руддщ", "hello", true)]
            {
                let mut engine = composing(backend, true, input, is_ru);
                let mut observed = TestEngineOutput::default();
                assert!(engine.capture_space_autocorrect_frame_identity().is_some());
                zbus::block_on(engine.consider_fragment_layout_letter(
                    &mut EngineOutput::test(&mut observed),
                    input.chars().last().unwrap(),
                ));
                assert_eq!(
                    observed.committed_texts,
                    [expected.to_owned()],
                    "{backend} {input}"
                );
                assert_eq!(engine.committed_tail.buffer, expected);
                assert!(engine.composition.buffer.is_empty());
                assert_eq!(engine.layout_gesture.layout_is_ru, !is_ru);
                assert!(engine.committed_tail.autocorrect_suppression.is_none());
                assert!(!engine.layout_gesture.pending_manual_toggle);
                assert!(
                    observed.surrounding_deletes.is_empty(),
                    "uncommitted word needs no deletion"
                );
            }
        }
    }

    #[test]
    fn fragment_committed_word_verifies_only_its_exact_suffix_and_preserves_context() {
        for backend in ["gnome", "kde"] {
            for prefix in ["", "prior "] {
                for (input, expected, is_ru) in
                    [("ghbdtn", "привет", false), ("руддщ", "hello", true)]
                {
                    crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
                        *slot.borrow_mut() = Some(crate::layout_sync::LayoutStackProbe {
                            active_engine: Some(
                                if is_ru { "lay-ime-ru" } else { "lay-ime-us" }.into(),
                            ),
                            ..Default::default()
                        });
                    });
                    let mut engine = composing(backend, true, input, is_ru);
                    engine.composition.buffer.clear();
                    engine.composition.cursor = 0;
                    engine.composition.preedit_visible = false;
                    engine.committed_tail.buffer = format!("{prefix}{input}");
                    let full_text = engine.committed_tail.buffer.clone();
                    engine.set_client_capabilities(1 << 5);
                    engine.client_context.surrounding_text_snapshot =
                        Some(crate::engine::SurroundingTextSnapshot::new(
                            full_text.clone(),
                            full_text.chars().count() as u32,
                            full_text.chars().count() as u32,
                        ));
                    let mut observed = TestEngineOutput {
                        legacy_transport: true,
                        ..TestEngineOutput::default()
                    };
                    zbus::block_on(engine.consider_fragment_layout_letter(
                        &mut EngineOutput::test(&mut observed),
                        input.chars().last().unwrap(),
                    ));
                    let layout = crate::layout_sync::LAYOUT_STACK_PROBE
                        .with(|slot| slot.borrow_mut().take().unwrap());
                    assert!(
                        layout.ibus_switches.is_empty(),
                        "an unfinished fragment must preserve the selected IBus engine: {backend}"
                    );
                    assert_eq!(observed.input_mode_updates, [!is_ru]);
                    assert_eq!(
                        layout.kde_activations,
                        if backend == "kde" {
                            vec![!is_ru]
                        } else {
                            vec![]
                        }
                    );
                    assert!(layout.activations.is_empty());
                    assert_eq!(
                        engine.committed_tail.buffer,
                        format!("{prefix}{expected}"),
                        "{backend} {prefix:?} {input}"
                    );
                    assert_eq!(observed.committed_texts, [expected.to_owned()]);
                    assert_eq!(
                        observed.surrounding_deletes,
                        [(
                            -(input.chars().count() as i32),
                            input.chars().count() as u32
                        )]
                    );
                    assert_eq!(engine.layout_gesture.layout_is_ru, !is_ru);
                    assert!(engine.committed_tail.autocorrect_suppression.is_none());
                    assert!(!engine.layout_gesture.pending_manual_toggle);
                }
            }
        }
    }

    #[test]
    fn fragment_return_to_selected_source_reconciles_only_a_changed_kde_group() {
        for backend in ["gnome", "kde"] {
            for source_is_ru in [false, true] {
                crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
                    *slot.borrow_mut() = Some(crate::layout_sync::LayoutStackProbe {
                        active_engine: Some(
                            if source_is_ru {
                                "lay-ime-ru"
                            } else {
                                "lay-ime-us"
                            }
                            .into(),
                        ),
                        ..Default::default()
                    });
                });
                let (text, expected) = if source_is_ru {
                    ("hello", "руддщ")
                } else {
                    ("привет", "ghbdtn")
                };
                let mut engine = composing(backend, true, text, source_is_ru);
                engine.composition.buffer.clear();
                engine.composition.cursor = 0;
                engine.composition.preedit_visible = false;
                engine.set_client_capabilities(1 << 5);
                engine.client_context.surrounding_text_snapshot =
                    Some(crate::engine::SurroundingTextSnapshot::new(
                        text.into(),
                        text.chars().count() as u32,
                        text.chars().count() as u32,
                    ));
                engine.set_layout_is_ru(!source_is_ru);
                let mut observed = TestEngineOutput {
                    legacy_transport: true,
                    ..TestEngineOutput::default()
                };
                // This isolates completion after the existing native/terminal
                // dispatch. The physical dispatch is checked by the actor.
                let result = zbus::block_on(engine.toggle_committed_tail_target_from(
                    &mut EngineOutput::test(&mut observed),
                    WordLayoutOrigin::Manual,
                ))
                .unwrap();
                let layout = crate::layout_sync::LAYOUT_STACK_PROBE
                    .with(|slot| slot.borrow_mut().take().unwrap());
                assert_eq!(
                    result,
                    (Some(source_is_ru), ExecutionReceipt::LocalComplete)
                );
                assert_eq!(engine.committed_tail.buffer, expected);
                assert_eq!(observed.input_mode_updates, [source_is_ru]);
                assert!(engine.committed_tail.autocorrect_suppression.is_some());
                assert_eq!(engine.layout_gesture.layout_is_ru, source_is_ru);
                assert!(layout.ibus_switches.is_empty());
                assert!(layout.activations.is_empty());
                assert_eq!(
                    layout.kde_activations,
                    if backend == "kde" {
                        vec![source_is_ru]
                    } else {
                        vec![]
                    }
                );
            }
        }
    }

    #[test]
    fn fragment_mode_completion_preserves_the_existing_dispatched_result_epoch() {
        for backend in ["gnome", "kde"] {
            crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
                *slot.borrow_mut() = Some(crate::layout_sync::LayoutStackProbe {
                    active_engine: Some("lay-ime-us".into()),
                    ..Default::default()
                });
            });
            let mut engine = composing(backend, true, "привет", false);
            engine.composition.buffer.clear();
            engine.composition.cursor = 0;
            assert!(engine.publish_tail_handoff());
            let expected = crate::engine::SurroundingTextSnapshot::new("привет".into(), 6, 6);
            engine.arm_exact_visible_postcondition_from_surrounding_dispatch(
                std::time::Instant::now(),
                None,
                None,
                expected.clone(),
            );
            let dispatched_epoch = engine.committed_tail.epoch;
            let mut observed = TestEngineOutput::default();
            zbus::block_on(
                engine.complete_fragment_word_mode(&mut EngineOutput::test(&mut observed), true),
            )
            .unwrap();
            assert_eq!(engine.committed_tail.epoch, dispatched_epoch, "{backend}");
            let pending = engine
                .committed_tail
                .pending_visible_postcondition
                .as_ref()
                .unwrap();
            assert_eq!(pending.dispatched_epoch, dispatched_epoch);
            assert_eq!(pending.expected_external_snapshot.as_ref(), Some(&expected));
            assert!(engine.layout_gesture.layout_is_ru);
            assert!(observed.committed_texts.is_empty());
            assert!(observed.surrounding_deletes.is_empty());
            crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| *slot.borrow_mut() = None);
        }
    }

    #[test]
    fn fragment_committed_mode_entry_never_switches_source_for_owned_preedit() {
        crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
            *slot.borrow_mut() = Some(crate::layout_sync::LayoutStackProbe {
                active_engine: Some("lay-ime-us".into()),
                ..Default::default()
            });
        });
        let mut engine = composing("gnome", true, "hello", false);
        let epoch = engine.committed_tail.epoch;
        let mut observed = TestEngineOutput::default();
        assert!(zbus::block_on(
            engine.complete_fragment_word_mode(&mut EngineOutput::test(&mut observed), true)
        )
        .is_err());
        assert_eq!(engine.composition.buffer, "hello");
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert!(!engine.layout_gesture.layout_is_ru);
        assert!(observed.effects.is_empty());
        let layout =
            crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| slot.borrow_mut().take().unwrap());
        assert!(layout.activations.is_empty());
        assert!(layout.ibus_switches.is_empty());
        assert!(layout.kde_activations.is_empty());
    }

    fn prepared_fragment_manual_mode() -> LayIbusEngine {
        let mut engine = composing("gnome", true, "", false);
        engine.composition.preedit_visible = false;
        engine.committed_tail.buffer = "привет".into();
        engine.set_layout_is_ru(true);
        engine.layout_gesture.fragment_word_mode_generation =
            Some(engine.layout_gesture.layout_generation);
        engine.set_client_capabilities(1 << 5);
        engine.client_context.surrounding_text_snapshot = Some(
            crate::engine::SurroundingTextSnapshot::new("привет".into(), 6, 6),
        );
        engine.prepare_exact_manual_toggle_layout_handoff();
        assert!(engine.exact_manual_toggle_handoff_is_live());
        engine
    }

    #[test]
    fn fragment_manual_mode_requires_current_origin_and_exact_existing_handoff() {
        for changed in [
            "none",
            "off_after_origin",
            "no_origin",
            "old_generation",
            "backend",
            "auto_switch",
            "owned",
            "closed",
            "tail",
            "selection",
            "snapshot",
            "owner",
            "handoff",
            "admission",
        ] {
            let mut engine = prepared_fragment_manual_mode();
            let epoch = engine.committed_tail.epoch;
            let path = engine.path.clone();
            match changed {
                "none" => {}
                "off_after_origin" => engine.config.fragment_layout_autoflip = false,
                "no_origin" => engine.layout_gesture.fragment_word_mode_generation = None,
                "old_generation" => {
                    engine.layout_gesture.fragment_word_mode_generation =
                        Some(engine.layout_gesture.layout_generation.wrapping_add(1))
                }
                "backend" => engine.config.layout_backend = "kde".into(),
                "auto_switch" => engine.config.auto_switch_layout = false,
                "owned" => engine.composition.buffer = "привет".into(),
                "closed" => engine.committed_tail.buffer.push(' '),
                "tail" => engine.committed_tail.buffer = "другое".into(),
                "selection" => {
                    engine.client_context.surrounding_text_snapshot = Some(
                        crate::engine::SurroundingTextSnapshot::new("привет".into(), 6, 0),
                    )
                }
                "snapshot" => {
                    engine.client_context.surrounding_text_snapshot = Some(
                        crate::engine::SurroundingTextSnapshot::new("другое".into(), 6, 6),
                    )
                }
                "owner" => engine.shared.lock().unwrap().active_path = None,
                "handoff" => engine.consume_exact_manual_toggle_handoff(),
                "admission" => engine.context_admission_required = true,
                _ => unreachable!(),
            }
            assert_eq!(
                engine.fragment_manual_mode_handoff_matches("привет", epoch, &path, true, false),
                matches!(changed, "none" | "off_after_origin"),
                "{changed}"
            );
        }
        for (suffix, delta, path, initial, target) in [
            ("", 0, "/test/fragment-letter", true, false),
            ("other", 0, "/test/fragment-letter", true, false),
            ("привет", 1, "/test/fragment-letter", true, false),
            ("привет", 0, "/foreign", true, false),
            ("привет", 0, "/test/fragment-letter", false, true),
            ("привет", 0, "/test/fragment-letter", true, true),
        ] {
            let engine = prepared_fragment_manual_mode();
            assert!(!engine.fragment_manual_mode_handoff_matches(
                suffix,
                engine.committed_tail.epoch + delta,
                path,
                initial,
                target
            ));
        }
    }

    #[test]
    fn fragment_manual_mode_preserves_exact_closed_suffix() {
        for (prefix, separator) in [("", " "), ("первое ", "  "), ("первое ", "\t")] {
            let mut engine = prepared_fragment_manual_mode();
            let suffix = format!("привет{separator}");
            let tail = format!("{prefix}{suffix}");
            engine.committed_tail.buffer = tail.clone();
            let cursor = tail.chars().count() as u32;
            engine.client_context.surrounding_text_snapshot = Some(
                crate::engine::SurroundingTextSnapshot::new(tail.clone(), cursor, cursor),
            );
            engine.prepare_exact_manual_toggle_layout_handoff();
            let epoch = engine.committed_tail.epoch;
            let path = engine.path.clone();
            assert!(
                engine.fragment_manual_return_has_mode_origin(),
                "{suffix:?}"
            );
            let mut observed = TestEngineOutput::default();
            for (initial, target) in [(true, false), (false, true)] {
                assert!(
                    engine.fragment_manual_mode_handoff_matches(
                        &suffix, epoch, &path, initial, target
                    ),
                    "{suffix:?}"
                );
                zbus::block_on(
                    engine.complete_fragment_word_mode(
                        &mut EngineOutput::test(&mut observed),
                        target,
                    ),
                )
                .unwrap();
                engine.layout_gesture.fragment_word_mode_generation =
                    Some(engine.layout_gesture.layout_generation);
                assert_eq!(engine.committed_tail.buffer, tail);
                assert_eq!(engine.committed_tail.epoch, epoch);
                assert!(engine.exact_manual_toggle_handoff_is_live());
            }
            assert_eq!(observed.input_mode_updates, [false, true]);
            assert!(observed.committed_texts.is_empty());
            assert!(observed.surrounding_deletes.is_empty());
        }
    }

    #[test]
    fn fragment_manual_mode_rejects_mismatched_closed_suffix() {
        let mut engine = prepared_fragment_manual_mode();
        engine.committed_tail.buffer = "первое привет  ".into();
        engine.client_context.surrounding_text_snapshot = Some(
            crate::engine::SurroundingTextSnapshot::new("первое привет  ".into(), 15, 15),
        );
        engine.prepare_exact_manual_toggle_layout_handoff();
        let epoch = engine.committed_tail.epoch;
        let path = engine.path.clone();
        for suffix in [
            "",
            " ",
            "привет",
            "привет ",
            "привет   ",
            "вет  ",
            "первое привет  ",
        ] {
            assert!(
                !engine.fragment_manual_mode_handoff_matches(suffix, epoch, &path, true, false),
                "{suffix:?}"
            );
        }
        assert!(!engine.fragment_manual_mode_handoff_matches(
            "привет  ",
            epoch.wrapping_add(1),
            &path,
            true,
            false
        ));
        engine.consume_exact_manual_toggle_handoff();
        assert!(!engine.fragment_manual_mode_handoff_matches(
            "привет  ",
            epoch,
            &path,
            true,
            false
        ));
    }

    #[test]
    fn fragment_manual_mode_roundtrip_changes_no_word_epoch_or_writer() {
        let mut engine = prepared_fragment_manual_mode();
        let epoch = engine.committed_tail.epoch;
        let path = engine.path.clone();
        let mut observed = TestEngineOutput::default();
        for (initial, target) in [(true, false), (false, true)] {
            assert!(engine.fragment_manual_mode_handoff_matches(
                "привет",
                epoch,
                &path,
                initial,
                target
            ));
            zbus::block_on(
                engine.complete_fragment_word_mode(&mut EngineOutput::test(&mut observed), target),
            )
            .unwrap();
            engine.layout_gesture.fragment_word_mode_generation =
                Some(engine.layout_gesture.layout_generation);
            assert_eq!(engine.committed_tail.buffer, "привет");
            assert_eq!(engine.committed_tail.epoch, epoch);
            assert_eq!(engine.layout_gesture.layout_is_ru, target);
            assert!(engine.exact_manual_toggle_handoff_is_live());
        }
        assert_eq!(observed.input_mode_updates, [false, true]);
        assert!(observed.committed_texts.is_empty());
        assert!(observed.surrounding_deletes.is_empty());
    }

    #[test]
    fn fragment_mode_entry_rejects_stale_and_revoked_intent_without_effects() {
        use crate::context_admission::{ContextAdmissionAdapter, WordCompleteness};
        for backend in ["gnome", "kde"] {
            let path = "/org/freedesktop/IBus/Engine/Lay/fragment_mode";
            let (admission, grant, _peer) = ContextAdmissionAdapter::test_established(
                path,
                "/org/freedesktop/IBus/InputContext/fragment_mode",
                WordCompleteness::KnownStart,
                4,
            );
            let token = admission
                .current_layout_intent_token_for(&grant.target_owner)
                .unwrap();
            let mut engine = composing(backend, true, "", false);
            engine.context_admission_required = true;
            engine.context_admission = Some(admission.clone());
            engine.context_owner = Some(grant.target_owner);
            assert!(admission.revoke_layout_intent(&token));
            crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
                *slot.borrow_mut() = Some(Default::default());
            });
            assert!(engine
                .sync_layout_for_fragment_word_mode(true, Some(token))
                .is_err());
            let layout = crate::layout_sync::LAYOUT_STACK_PROBE
                .with(|slot| slot.borrow_mut().take().unwrap());
            assert!(layout.kde_activations.is_empty());
            assert!(layout.ibus_switches.is_empty());
            assert!(layout.activations.is_empty());
            assert!(!engine.layout_gesture.layout_is_ru);
        }
    }

    #[test]
    fn fragment_mode_publication_or_kde_failure_never_promotes_decoder_or_retries() {
        for failure in ["property", "group"] {
            crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
                *slot.borrow_mut() = Some(crate::layout_sync::LayoutStackProbe {
                    fail_kde_activation: failure == "group",
                    ..Default::default()
                });
            });
            let mut engine = composing("kde", true, "", false);
            let mut observed = TestEngineOutput {
                fail_input_mode_publication: failure == "property",
                ..Default::default()
            };
            assert!(zbus::block_on(
                engine.complete_fragment_word_mode(&mut EngineOutput::test(&mut observed), true,)
            )
            .is_err());
            let layout = crate::layout_sync::LAYOUT_STACK_PROBE
                .with(|slot| slot.borrow_mut().take().unwrap());
            assert!(!engine.layout_gesture.layout_is_ru);
            assert!(observed.committed_texts.is_empty());
            assert!(observed.surrounding_deletes.is_empty());
            assert!(layout.ibus_switches.is_empty());
            assert!(layout.activations.is_empty());
            assert_eq!(
                layout.kde_activations.len(),
                usize::from(failure == "group")
            );
            assert_eq!(
                observed
                    .effects
                    .iter()
                    .filter(|effect| **effect == "register-input-mode")
                    .count(),
                1
            );
        }
    }

    #[test]
    fn fragment_mode_return_does_not_intercept_off_or_closed_words() {
        for enabled in [false, true] {
            for text in ["привет", "привет "] {
                let mut engine = composing("kde", enabled, text, false);
                engine.composition.buffer.clear();
                engine.set_layout_is_ru(true);
                assert_eq!(
                    engine.fragment_word_mode_return_needed(),
                    enabled && !text.ends_with(' ')
                );
            }
        }
    }

    #[test]
    fn fragment_disabled_boundaries_sensitive_and_stale_requests_have_no_effect() {
        for control in [
            "off",
            "space",
            "enter",
            "punctuation",
            "password",
            "cursor",
            "stale-owner",
            "pending-postcondition",
        ] {
            let mut engine = composing("kde", control != "off", "ghbdtn", false);
            let letter = match control {
                "space" => ' ',
                "enter" => '\n',
                "punctuation" => '!',
                _ => 'n',
            };
            if control == "password" {
                engine.set_content_type_state(8, 0);
            }
            if control == "cursor" {
                engine.composition.cursor = 1;
            }
            if control == "stale-owner" {
                engine.context_admission_required = true;
            }
            if control == "pending-postcondition" {
                engine.set_client_capabilities(1 << 5);
                engine.arm_visible_postcondition(std::time::Instant::now());
                assert!(engine
                    .committed_tail
                    .pending_visible_postcondition
                    .is_some());
            }
            let expected_text = engine.composition.buffer.clone();
            let mut observed = TestEngineOutput::default();
            zbus::block_on(
                engine.consider_fragment_layout_letter(
                    &mut EngineOutput::test(&mut observed),
                    letter,
                ),
            );
            assert_eq!(engine.composition.buffer, expected_text, "{control}");
            assert!(!engine.layout_gesture.layout_is_ru, "{control}");
            assert!(observed.committed_texts.is_empty(), "{control}");
            assert!(observed.surrounding_deletes.is_empty(), "{control}");
            assert!(engine.committed_tail.autocorrect_suppression.is_none());
            assert!(!engine.layout_gesture.pending_manual_toggle);
        }
    }

    #[test]
    fn fragment_mode_failure_after_committed_text_reports_partial_without_retry() {
        crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| {
            *slot.borrow_mut() = Some(crate::layout_sync::LayoutStackProbe {
                active_engine: Some("lay-ime-us".into()),
                ..Default::default()
            });
        });
        let mut engine = composing("kde", true, "", false);
        // Controlled contract violation: bypass the outer option admission.
        // The text operation can complete, but mode completion must refuse.
        engine.config.auto_switch_layout = false;
        engine.committed_tail.buffer = "ghbdtn".into();
        engine.set_client_capabilities(1 << 5);
        engine.client_context.surrounding_text_snapshot = Some(
            crate::engine::SurroundingTextSnapshot::new("ghbdtn".into(), 6, 6),
        );
        let mut observed = TestEngineOutput {
            legacy_transport: true,
            ..Default::default()
        };
        let request = lay::layout_fragments::on_letter("ghbdtn", 'n', false).unwrap();
        let failure = zbus::block_on(engine.toggle_committed_tail_target_from(
            &mut EngineOutput::test(&mut observed),
            WordLayoutOrigin::Fragment(&request),
        ))
        .unwrap_err();
        let layout =
            crate::layout_sync::LAYOUT_STACK_PROBE.with(|slot| slot.borrow_mut().take().unwrap());
        assert_eq!(observed.committed_texts, ["привет".to_owned()]);
        assert_eq!(observed.surrounding_deletes, [(-6, 6)]);
        assert_eq!(
            failure.receipt(),
            ExecutionReceipt::LocalIndeterminatePartial
        );
        assert_eq!(
            failure.progress(),
            crate::window_interaction::LocalEffectProgress::CommitDispatched
        );
        assert!(layout.kde_activations.is_empty());
        assert!(layout.ibus_switches.is_empty());
        assert!(layout.activations.is_empty());
        assert!(observed.forwarded_keys.is_empty());
        assert!(!engine.layout_gesture.layout_is_ru);
        assert_eq!(engine.committed_tail.buffer, "привет");
        assert!(engine.committed_tail.autocorrect_suppression.is_none());
        assert!(!engine.layout_gesture.pending_manual_toggle);
    }

    #[test]
    fn fragment_unknown_literal_without_current_native_word_stays_read_only() {
        let mut engine = composing("kde", true, "", false);
        engine.committed_tail.buffer = "ghbdtn".into();
        engine.context_admission_required = true;
        let epoch = engine.committed_tail.epoch;
        assert!(lay::layout_fragments::on_letter("ghbdtn", 'n', false).is_some());
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_fragment_layout_frame().is_none());
        let mut observed = TestEngineOutput::default();
        zbus::block_on(
            engine.consider_fragment_layout_letter(&mut EngineOutput::test(&mut observed), 'n'),
        );
        assert_eq!(engine.committed_tail.buffer, "ghbdtn");
        assert_eq!(engine.committed_tail.epoch, epoch);
        assert!(!engine.context_word_is_known());
        assert!(engine.context_token.is_none());
        assert!(engine.context_word_scope.is_none());
        assert!(engine.composition.buffer.is_empty());
        assert!(!engine.layout_gesture.layout_is_ru);
        assert!(observed.committed_texts.is_empty());
        assert!(observed.surrounding_deletes.is_empty());
    }

    #[test]
    fn fragment_detector_future_keeps_a_bounded_callback_stack_frame() {
        fn bytes<F>(_: impl FnOnce() -> F) -> usize {
            std::mem::size_of::<F>()
        }
        let mut engine = composing("gnome", false, "ghbdtn", false);
        let mut observed = TestEngineOutput::default();
        let mut output = EngineOutput::test(&mut observed);
        let fragment_bytes = {
            let (engine, output) = (&mut engine, &mut output);
            bytes(move || engine.consider_fragment_layout_letter(output, 'n'))
        };
        let key_bytes = {
            let (engine, output) = (&mut engine, &mut output);
            bytes(move || engine.process_key_event_with_output(output, u32::from('n'), 49, 0))
        };
        let operation_bytes = {
            let (engine, output) = (&mut engine, &mut output);
            bytes(move || engine.request_word_layout_flip(output, WordLayoutOrigin::Manual))
        };
        eprintln!("fragment_future_bytes={fragment_bytes} key_future_bytes={key_bytes} word_flip_future_bytes={operation_bytes}");
        assert!(
            fragment_bytes <= 16 * 1024,
            "an optional letter trigger must not embed a large executor future: {fragment_bytes}"
        );
    }

    #[test]
    fn fragment_real_key_release_requires_a_paired_letter_and_ignores_space() {
        for backend in ["gnome", "kde"] {
            let mut closed = composing(backend, true, "ghbdtn ", false);
            closed.composition.buffer.clear();
            closed.composition.cursor = 0;
            closed.remember_handled_press(49, true);
            assert!(
                closed
                    .capture_fragment_letter_release(
                        u32::from('n'),
                        49,
                        crate::protocol::RELEASE_MASK,
                    )
                    .is_none(),
                "a delayed letter release after Space cannot trigger a closed word: {backend}"
            );
            for (keyval, keycode, paired, expected) in [
                (u32::from('n'), 49, true, "привет"),
                (u32::from('n'), 49, false, "ghbdtn"),
                (crate::protocol::KEY_SPACE, 57, true, "ghbdtn"),
                (crate::protocol::KEY_ENTER, 28, true, "ghbdtn"),
            ] {
                let mut engine = composing(backend, true, "ghbdtn", false);
                if paired {
                    engine.remember_handled_press(keycode, true);
                }
                let release = engine.capture_fragment_letter_release(
                    keyval,
                    keycode,
                    crate::protocol::RELEASE_MASK,
                );
                let mut observed = TestEngineOutput::default();
                let handled = zbus::block_on(engine.process_key_event_with_output(
                    &mut EngineOutput::test(&mut observed),
                    keyval,
                    keycode,
                    crate::protocol::RELEASE_MASK,
                ))
                .unwrap();
                assert_eq!(handled, paired);
                if let Some(release) = release {
                    zbus::block_on(engine.request_fragment_after_letter_release(
                        &mut EngineOutput::test(&mut observed),
                        release,
                        handled,
                    ));
                }
                assert_eq!(
                    engine.committed_tail.buffer, expected,
                    "{backend} key={keyval} paired={paired}"
                );
                if expected == "привет" {
                    assert_eq!(observed.committed_texts, [expected.to_owned()]);
                } else {
                    assert!(observed.committed_texts.is_empty());
                }
                assert!(observed.surrounding_deletes.is_empty());
                assert!(engine.committed_tail.autocorrect_suppression.is_none());
            }
        }
    }

    #[test]
    fn fragment_release_request_survives_early_retirement_but_not_focus_or_pair_loss() {
        for backend in ["gnome", "kde"] {
            for control in [
                "early-retired",
                "focus-changed",
                "not-retired",
                "not-handled",
            ] {
                let mut engine = composing(backend, true, "ghbdtn", false);
                engine.remember_handled_press(49, true);
                let release = engine
                    .capture_fragment_letter_release(
                        u32::from('n'),
                        49,
                        crate::protocol::RELEASE_MASK,
                    )
                    .unwrap();
                // The canonical early path already retires this press without
                // entering process_key_event_with_output; it stays unchanged.
                if control != "not-retired" {
                    assert!(engine.consume_handled_release(49));
                }
                if control == "focus-changed" {
                    engine.client_context.focus_serial =
                        engine.client_context.focus_serial.wrapping_add(1);
                }
                let mut observed = TestEngineOutput::default();
                zbus::block_on(engine.request_fragment_after_letter_release(
                    &mut EngineOutput::test(&mut observed),
                    release,
                    control != "not-handled",
                ));
                let applied = control == "early-retired";
                assert_eq!(
                    engine.committed_tail.buffer,
                    if applied { "привет" } else { "ghbdtn" },
                    "{backend} {control}"
                );
                assert_eq!(
                    observed.committed_texts,
                    if applied {
                        vec!["привет".to_owned()]
                    } else {
                        Vec::new()
                    }
                );
                assert!(observed.surrounding_deletes.is_empty());
                assert_eq!(engine.layout_gesture.layout_is_ru, applied);
                assert!(engine.context_bridge_token.is_none());
            }
        }
    }

    #[test]
    fn fragment_origin_cannot_apply_an_old_proposal_to_another_word() {
        let request = lay::layout_fragments::on_letter("ghbdtn", 'n', false).unwrap();
        let mut engine = composing("gnome", true, "other", false);
        let mut observed = TestEngineOutput::default();
        let result = zbus::block_on(engine.request_word_layout_flip(
            &mut EngineOutput::test(&mut observed),
            WordLayoutOrigin::Fragment(&request),
        ))
        .unwrap();
        assert_eq!(result, (None, ExecutionReceipt::Rejected));
        assert_eq!(engine.composition.buffer, "other");
        assert!(observed.committed_texts.is_empty());
        assert!(observed.surrounding_deletes.is_empty());
    }
}
