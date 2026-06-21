use zbus::fdo;
use zbus::{interface, Connection};

use super::engine::{LayIbusEngine, ManualToggleAuthority};
use super::protocol::Shared;
use crate::bridge_policy::should_suppress_next_autocorrect;

pub(crate) struct LayImeBridge {
    pub(crate) ibus_connection: Connection,
    pub(crate) shared: Shared,
}

#[interface(name = "io.github.radislabus_star.LayIme")]
impl LayImeBridge {
    #[zbus(name = "Ping")]
    fn ping(&self) -> String {
        let state = self.shared.lock().expect("lay ime state poisoned");
        match state.active_path.as_deref() {
            Some(path) => format!("lay-ibus-engine-rs focused {path}"),
            None => "lay-ibus-engine-rs no-focus".to_string(),
        }
    }

    #[zbus(name = "Focused")]
    fn focused(&self) -> bool {
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .is_some()
    }

    #[zbus(name = "OwnsActiveText")]
    async fn owns_active_text(&self) -> fdo::Result<bool> {
        let Some(path) = self.active_path() else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let engine = iface_ref.get().await;
        Ok(engine.manual_toggle_authority() == ManualToggleAuthority::ImeActiveComposition)
    }

    #[zbus(name = "InputState")]
    async fn input_state(&self) -> fdo::Result<String> {
        self.input_state_inner().await
    }

    #[zbus(name = "SuppressNextAutocorrect")]
    async fn suppress_next_autocorrect(&self) -> fdo::Result<bool> {
        let Some(path) = self.active_path() else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let mut engine = iface_ref.get_mut().await;
        engine.suppress_next_committed_tail_autocorrect = true;
        engine.publish_autocorrect_suppression_handoff();
        super::trace::record(r#"{"kind":"ibus_suppress_next_autocorrect","source":"daemon"}"#);
        Ok(true)
    }

    #[zbus(name = "ManualToggle")]
    async fn manual_toggle(&self) -> fdo::Result<bool> {
        self.manual_toggle_inner().await
    }

    #[zbus(name = "ManualToggleV2")]
    async fn manual_toggle_v2(&self) -> fdo::Result<(bool, bool)> {
        self.manual_toggle_v2_inner().await
    }

    #[zbus(name = "ReplaceTail")]
    async fn replace_tail(&self, backspaces: u32, text: String) -> fdo::Result<bool> {
        self.replace_tail_inner(backspaces, text, false).await
    }

    #[zbus(name = "ReplaceTailV2")]
    async fn replace_tail_v2(
        &self,
        backspaces: u32,
        text: String,
        kind: String,
    ) -> fdo::Result<bool> {
        self.replace_tail_inner(backspaces, text, should_suppress_next_autocorrect(&kind))
            .await
    }
}
