use clap::Parser;

// CLI, component XML and tray share the package release version.
pub(crate) const DISPLAY_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "lay-ibus-engine",
    version = DISPLAY_VERSION,
    about = "Rust IBus text-edit bridge for lay"
)]
pub(crate) struct Args {
    /// Run when started by ibus-daemon. Kept for component XML compatibility.
    #[arg(long)]
    pub(crate) ibus: bool,
    /// Consume key events and run lay typing-assist directly inside IBus.
    #[arg(long)]
    pub(crate) managed: bool,
    /// Print IBus component XML.
    #[arg(long)]
    pub(crate) xml: bool,
}
