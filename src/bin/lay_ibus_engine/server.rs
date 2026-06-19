use std::sync::{Arc, Mutex};

use zbus::connection;

use super::args::Args;
use super::bridge::LayImeBridge;
use super::factory::LayIbusFactory;
use super::protocol::{SharedState, BUS_NAME, BUS_PATH, IBUS_ENGINE_NAME, IBUS_FACTORY_PATH};

pub(crate) async fn run(args: &Args) -> zbus::Result<()> {
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let managed_input = managed_input_enabled(args);
    let ibus_connection = connection::Builder::ibus()?.build().await?;
    ibus_connection
        .object_server()
        .at(
            IBUS_FACTORY_PATH,
            LayIbusFactory {
                ibus_connection: ibus_connection.clone(),
                shared: Arc::clone(&shared),
                managed_input,
            },
        )
        .await?;
    ibus_connection.request_name(IBUS_ENGINE_NAME).await?;

    let _session_connection = connection::Builder::session()?
        .serve_at(
            BUS_PATH,
            LayImeBridge {
                ibus_connection,
                shared,
            },
        )?
        .name(BUS_NAME)?
        .allow_name_replacements(true)
        .replace_existing_names(true)
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}

fn managed_input_enabled(args: &Args) -> bool {
    if args.managed {
        return true;
    }
    std::env::var("LAY_IME_MANAGED")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}
