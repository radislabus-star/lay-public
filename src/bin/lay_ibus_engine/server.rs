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

    // Publish the IBus factory and bridge before touching lexical memory. GNOME
    // may select the configured engine immediately during login; registration
    // must remain available while compact L2 memory warms in the background.
    super::space_autocorrect_prefetch::initialize();
    std::thread::Builder::new()
        .name("lay-exact-layout-warmup".to_string())
        .spawn(|| {
            let _ = lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus();
        })
        .map_err(|error| zbus::Error::Failure(error.to_string()))?;
    lay::typing_cpu::TypingCpu::ensure_ime_warmup_started();

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
