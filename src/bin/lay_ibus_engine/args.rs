use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "lay-ibus-engine",
    version,
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
