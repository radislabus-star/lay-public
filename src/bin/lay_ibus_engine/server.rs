use std::sync::{Arc, Mutex};

use zbus::connection;

use super::args::Args;
use super::bridge::LayImeBridge;
use super::context_admission::{
    AdapterConfig, ConnectionGeneration, EngineProfile, PendingContextAdapter,
};
use super::factory::LayIbusFactory;
use super::protocol::{SharedState, BUS_NAME, BUS_PATH, IBUS_ENGINE_NAME, IBUS_FACTORY_PATH};

pub(crate) async fn run(args: &Args) -> zbus::Result<()> {
    let shared = Arc::new(Mutex::new(SharedState::default()));
    let managed_input = managed_input_enabled(args);
    let ibus_connection = connection::Builder::ibus()?.build().await?;
    let admission_config = AdapterConfig::new(
        ConnectionGeneration(1),
        ["lay-ime-us", "lay-ime-ru"]
            .into_iter()
            .filter_map(EngineProfile::new)
            .collect(),
    )
    .map_err(|error| zbus::Error::Failure(error.to_string()))?;
    let admission = match PendingContextAdapter::subscribe(
        ibus_connection.clone(),
        admission_config,
    )
    .await
    {
        Ok(pending) => match pending.bootstrap().await {
            Ok((admission, observer, _bootstrap_identity)) => {
                ibus_connection
                    .executor()
                    .spawn(
                        async move {
                            if let Err(error) = observer.run().await {
                                super::trace::record(format!(
                                    r#"{{"kind":"ibus_context_admission_observer","status":"stopped","error":{:?}}}"#,
                                    error.to_string()
                                ));
                            }
                        },
                        "lay-context-admission-observer",
                    )
                    .detach();
                Some(admission)
            }
            Err(error) => {
                super::trace::record(format!(
                    r#"{{"kind":"ibus_context_admission","status":"bootstrap_unavailable","error":{:?}}}"#,
                    error.to_string()
                ));
                None
            }
        },
        Err(error) => {
            super::trace::record(format!(
                r#"{{"kind":"ibus_context_admission","status":"subscribe_unavailable","error":{:?}}}"#,
                error.to_string()
            ));
            None
        }
    };
    ibus_connection
        .object_server()
        .at(
            IBUS_FACTORY_PATH,
            LayIbusFactory {
                ibus_connection: ibus_connection.clone(),
                shared: Arc::clone(&shared),
                admission: admission.clone(),
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
                context_admission_required: true,
                admission,
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
            let started = std::time::Instant::now();
            super::trace::record(r#"{"kind":"ibus_exact_authority_warmup","stage":"started"}"#);
            let receipt = lay::exact_layout_authority::warm_up_exact_layout_authority_for_ibus();
            if super::trace::enabled() {
                super::trace::record(format!(
                    r#"{{"kind":"ibus_exact_authority_warmup","stage":"completed","available":{},"elapsed_us":{}}}"#,
                    receipt.is_some(),
                    started.elapsed().as_micros(),
                ));
            }
        })
        .map_err(|error| zbus::Error::Failure(error.to_string()))?;
    let startup_config = lay::config::LayConfig::load();
    lay::typing_cpu::TypingCpu::ensure_ime_runtime_warmup_started(startup_config.nanda_autocorrect);

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
