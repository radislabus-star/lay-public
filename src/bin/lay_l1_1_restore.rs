use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "lay-l1.1-restore",
    about = "Shadow L1.1 damaged-surface signal restorer"
)]
struct Args {
    #[arg(long, value_name = "PACKAGE")]
    memory: PathBuf,

    #[arg(long, default_value_t = 64)]
    limit: usize,

    #[arg(required = true, num_args = 1..)]
    surface: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let surface = args.surface.join(" ");
    let report = lay::nanda_wave::restore_l1_surface(&args.memory, &surface, args.limit)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
