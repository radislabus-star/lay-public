use clap::Parser;

#[path = "lay_ibus_engine/args.rs"]
mod args;
#[path = "lay_ibus_engine/bridge.rs"]
mod bridge;
#[path = "lay_ibus_engine/bridge_actions.rs"]
mod bridge_actions;
#[path = "lay_ibus_engine/bridge_policy.rs"]
mod bridge_policy;
#[path = "lay_ibus_engine/committed_tail.rs"]
mod committed_tail;
#[path = "lay_ibus_engine/composition_commit.rs"]
mod composition_commit;
#[path = "lay_ibus_engine/composition_edit.rs"]
mod composition_edit;
#[path = "lay_ibus_engine/engine.rs"]
mod engine;
#[path = "lay_ibus_engine/factory.rs"]
mod factory;
#[path = "lay_ibus_engine/ibus_interface.rs"]
mod ibus_interface;
#[path = "lay_ibus_engine/key_decode.rs"]
mod key_decode;
#[path = "lay_ibus_engine/key_trace.rs"]
mod key_trace;
#[path = "lay_ibus_engine/layout_sync.rs"]
mod layout_sync;
#[path = "lay_ibus_engine/managed.rs"]
mod managed;
#[path = "lay_ibus_engine/preedit.rs"]
mod preedit;
#[path = "lay_ibus_engine/protocol.rs"]
mod protocol;
#[path = "lay_ibus_engine/server.rs"]
mod server;
#[path = "lay_ibus_engine/shift.rs"]
mod shift;
#[path = "lay_ibus_engine/state.rs"]
mod state;
#[path = "lay_ibus_engine/tail_memory.rs"]
mod tail_memory;
#[path = "lay_ibus_engine/text.rs"]
mod text;
#[path = "lay_ibus_engine/trace.rs"]
mod trace;
#[path = "lay_ibus_engine/xml.rs"]
mod xml;

use args::Args;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.xml {
        println!("{}", xml::component_xml(&xml::component_exec_path()));
        return Ok(());
    }
    let _ = args.ibus;
    // Start compact L2/lexicon readout before an input context is created. The
    // first typed word must not be the warmup trigger for IME candidates.
    lay::nanda_wave::ensure_l2_ime_warmup_started();
    zbus::block_on(server::run(&args))?;
    Ok(())
}
