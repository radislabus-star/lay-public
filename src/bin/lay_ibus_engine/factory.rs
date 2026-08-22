use std::sync::Arc;

use lay::config::LayConfig;
use zbus::fdo;
use zbus::zvariant::ObjectPath;
use zbus::{interface, Connection};

use super::engine::LayIbusEngine;
use super::protocol::Shared;

pub(crate) struct LayIbusFactory {
    pub(crate) ibus_connection: Connection,
    pub(crate) shared: Shared,
    pub(crate) managed_input: bool,
}

#[interface(name = "org.freedesktop.IBus.Factory")]
impl LayIbusFactory {
    #[zbus(name = "CreateEngine")]
    async fn create_engine(&mut self, name: String) -> fdo::Result<ObjectPath<'_>> {
        let id = {
            let mut state = self.shared.lock().expect("lay ime state poisoned");
            let id = state.next_engine_id;
            state.next_engine_id = state.next_engine_id.saturating_add(1);
            id
        };
        let path = format!(
            "/io/github/radislabus_star/LayIme/engine/{}/{}",
            safe_engine_name(&name),
            id
        );
        self.ibus_connection
            .object_server()
            .at(
                path.as_str(),
                LayIbusEngine::new_from_component(
                    path.clone(),
                    Arc::clone(&self.shared),
                    &name,
                    self.managed_input,
                    LayConfig::load(),
                ),
            )
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        ObjectPath::try_from(path).map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

fn safe_engine_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
