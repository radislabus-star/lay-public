use zbus::fdo;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Str, Structure, Value};
use zbus::Connection;

use super::engine::LayIbusEngine;
use super::ibus_interface::ibus_text_value_to_string;

pub(crate) const PROPOSAL_NATIVE_UNHANDLED: u8 = 0;
pub(crate) const PROPOSAL_FRAME_READY: u8 = 1;
pub(crate) const PROPOSAL_CONSUMED_NO_EFFECT: u8 = 2;

const EFFECT_COMMIT_TEXT: u8 = 1;
const EFFECT_DELETE_SURROUNDING: u8 = 2;
const EFFECT_UPDATE_PREEDIT: u8 = 3;
const EFFECT_HIDE_PREEDIT: u8 = 4;
const MAX_EFFECTS: usize = 3;
const MAX_TEXT_BYTES: usize = 4096;
const MAX_DELETE_CHARS: u32 = 4096;

pub(crate) type AtomicProposal = (u8, Vec<(u8, OwnedValue)>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum PendingPreedit {
    Update {
        text: String,
        cursor: u32,
        mode: u32,
    },
    Hide,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AtomicEffectBuilder {
    delete: Option<(i32, u32)>,
    commit: String,
    preedit: Option<PendingPreedit>,
    unsupported: bool,
    allowed_mask: u32,
    maximum_effects: usize,
    delete_allowed: bool,
}

impl Default for AtomicEffectBuilder {
    fn default() -> Self {
        Self::new(0x0f, MAX_EFFECTS as u32, true)
    }
}

impl AtomicEffectBuilder {
    pub(crate) fn new(allowed_mask: u32, maximum_effects: u32, delete_allowed: bool) -> Self {
        Self {
            delete: None,
            commit: String::new(),
            preedit: None,
            unsupported: false,
            allowed_mask,
            maximum_effects: (maximum_effects as usize).min(MAX_EFFECTS),
            delete_allowed,
        }
    }

    fn push_commit(&mut self, text: String) {
        if text.is_empty() || self.commit.len().saturating_add(text.len()) > MAX_TEXT_BYTES {
            self.unsupported = true;
            return;
        }
        self.commit.push_str(&text);
    }

    fn push_delete(&mut self, offset: i32, nchars: u32) {
        if !self.delete_allowed
            || nchars == 0
            || nchars > MAX_DELETE_CHARS
            || offset != -(nchars as i32)
        {
            self.unsupported = true;
            return;
        }
        match self.delete {
            None => self.delete = Some((offset, nchars)),
            Some((_, previous)) if previous.saturating_add(nchars) <= MAX_DELETE_CHARS => {
                let combined = previous + nchars;
                self.delete = Some((-(combined as i32), combined));
            }
            Some(_) => self.unsupported = true,
        }
    }

    fn push_preedit(&mut self, text: String, cursor: u32, visible: bool, mode: u32) {
        let chars = text.chars().count() as u32;
        if !visible || text.is_empty() {
            self.preedit = Some(PendingPreedit::Hide);
        } else if text.len() <= MAX_TEXT_BYTES && cursor <= chars && mode <= 1 {
            self.preedit = Some(PendingPreedit::Update { text, cursor, mode });
        } else {
            self.unsupported = true;
        }
    }

    fn hide_preedit(&mut self) {
        self.preedit = Some(PendingPreedit::Hide);
    }

    fn show_preedit(&mut self) {
        if !matches!(self.preedit, Some(PendingPreedit::Update { .. })) {
            self.unsupported = true;
        }
    }

    pub(crate) fn finish(self, handled: bool) -> AtomicProposal {
        if self.unsupported || self.delete.is_some() && self.commit.is_empty() {
            return (PROPOSAL_NATIVE_UNHANDLED, Vec::new());
        }

        let mut effects = Vec::with_capacity(MAX_EFFECTS);
        if let Some((offset, nchars)) = self.delete {
            effects.push((EFFECT_DELETE_SURROUNDING, owned_structure((offset, nchars))));
        }
        if !self.commit.is_empty() {
            effects.push((EFFECT_COMMIT_TEXT, OwnedValue::from(Str::from(self.commit))));
        }
        if let Some(preedit) = self.preedit {
            match preedit {
                PendingPreedit::Update { text, cursor, mode } => effects.push((
                    EFFECT_UPDATE_PREEDIT,
                    owned_structure((Str::from(text), cursor, cursor, mode)),
                )),
                PendingPreedit::Hide => {
                    effects.push((EFFECT_HIDE_PREEDIT, OwnedValue::from(false)))
                }
            }
        }

        if effects.len() > self.maximum_effects
            || effects
                .iter()
                .any(|(tag, _)| self.allowed_mask & (1 << (u32::from(*tag) - 1)) == 0)
        {
            return (PROPOSAL_NATIVE_UNHANDLED, Vec::new());
        }
        if effects.is_empty() {
            if handled {
                (PROPOSAL_CONSUMED_NO_EFFECT, effects)
            } else {
                (PROPOSAL_NATIVE_UNHANDLED, effects)
            }
        } else {
            (PROPOSAL_FRAME_READY, effects)
        }
    }
}

fn owned_structure<T>(value: T) -> OwnedValue
where
    T: Into<Structure<'static>>,
{
    OwnedValue::try_from(value.into()).expect("static atomic payload structure")
}

pub(crate) enum EngineOutput<'a, 'e> {
    Legacy(&'a SignalEmitter<'e>),
    Atomic(&'a mut AtomicEffectBuilder),
}

impl<'a, 'e> EngineOutput<'a, 'e> {
    pub(crate) fn legacy(emitter: &'a SignalEmitter<'e>) -> Self {
        Self::Legacy(emitter)
    }

    pub(crate) fn atomic(builder: &'a mut AtomicEffectBuilder) -> Self {
        Self::Atomic(builder)
    }

    pub(crate) fn connection(&self) -> Option<&Connection> {
        match self {
            Self::Legacy(emitter) => Some(emitter.connection()),
            Self::Atomic(_) => None,
        }
    }

    pub(crate) async fn commit_text(&mut self, text: Value<'_>) -> fdo::Result<()> {
        match self {
            Self::Legacy(emitter) => LayIbusEngine::commit_text(emitter, text)
                .await
                .map_err(|error| fdo::Error::Failed(error.to_string())),
            Self::Atomic(builder) => {
                let Some(text) = ibus_text_value_to_string(&text) else {
                    return Err(fdo::Error::InvalidArgs("invalid IBusText commit".into()));
                };
                builder.push_commit(text);
                Ok(())
            }
        }
    }

    pub(crate) async fn delete_surrounding_text(
        &mut self,
        offset: i32,
        nchars: u32,
    ) -> fdo::Result<()> {
        match self {
            Self::Legacy(emitter) => {
                LayIbusEngine::delete_surrounding_text(emitter, offset, nchars)
                    .await
                    .map_err(|error| fdo::Error::Failed(error.to_string()))
            }
            Self::Atomic(builder) => {
                builder.push_delete(offset, nchars);
                Ok(())
            }
        }
    }

    pub(crate) async fn update_preedit_text(
        &mut self,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> fdo::Result<()> {
        match self {
            Self::Legacy(emitter) => {
                LayIbusEngine::update_preedit_text(emitter, text, cursor_pos, visible, mode)
                    .await
                    .map_err(|error| fdo::Error::Failed(error.to_string()))
            }
            Self::Atomic(builder) => {
                let Some(text) = ibus_text_value_to_string(&text) else {
                    return Err(fdo::Error::InvalidArgs("invalid IBusText preedit".into()));
                };
                builder.push_preedit(text, cursor_pos, visible, mode);
                Ok(())
            }
        }
    }

    pub(crate) async fn show_preedit_text(&mut self) -> fdo::Result<()> {
        match self {
            Self::Legacy(emitter) => LayIbusEngine::show_preedit_text(emitter)
                .await
                .map_err(|error| fdo::Error::Failed(error.to_string())),
            Self::Atomic(builder) => {
                builder.show_preedit();
                Ok(())
            }
        }
    }

    pub(crate) async fn hide_preedit_text(&mut self) -> fdo::Result<()> {
        match self {
            Self::Legacy(emitter) => LayIbusEngine::hide_preedit_text(emitter)
                .await
                .map_err(|error| fdo::Error::Failed(error.to_string())),
            Self::Atomic(builder) => {
                builder.hide_preedit();
                Ok(())
            }
        }
    }

    pub(crate) async fn forward_key_event(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<()> {
        match self {
            Self::Legacy(emitter) => {
                LayIbusEngine::forward_key_event(emitter, keyval, keycode, state)
                    .await
                    .map_err(|error| fdo::Error::Failed(error.to_string()))
            }
            Self::Atomic(builder) => {
                builder.unsupported = true;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_delete_commit_and_hide() {
        let mut builder = AtomicEffectBuilder::default();
        builder.push_delete(-3, 3);
        builder.push_commit("тест".to_string());
        builder.hide_preedit();

        let proposal = builder.finish(true);
        assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
        assert_eq!(
            proposal.1.iter().map(|effect| effect.0).collect::<Vec<_>>(),
            [2, 1, 4]
        );
        assert!(!bool::try_from(proposal.1[2].1.clone()).expect("hide marker"));
    }

    #[test]
    fn refuses_forward_or_delete_without_commit() {
        let mut forward = AtomicEffectBuilder::default();
        forward.unsupported = true;
        assert_eq!(forward.finish(true).0, PROPOSAL_NATIVE_UNHANDLED);

        let mut delete = AtomicEffectBuilder::default();
        delete.push_delete(-1, 1);
        assert_eq!(delete.finish(true).0, PROPOSAL_NATIVE_UNHANDLED);
    }
}
