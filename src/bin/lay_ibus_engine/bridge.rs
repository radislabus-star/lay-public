use zbus::fdo;
use zbus::{interface, Connection};

use super::engine::LayIbusEngine;
use super::protocol::Shared;

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

    #[zbus(name = "ReplaceTail")]
    async fn replace_tail(&self, backspaces: u32, text: String) -> fdo::Result<bool> {
        if backspaces == 0 && text.is_empty() {
            return Ok(false);
        }
        let active_path = self
            .shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .clone();
        let Some(path) = active_path else {
            return Ok(false);
        };
        let iface_ref = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let emitter = iface_ref.signal_emitter();
        let mut engine = iface_ref.get_mut().await;
        engine
            .replace_committed_tail(emitter, backspaces, text)
            .await
    }
}
