use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Train and prove the shadow L2 Russian morphology field")]
struct Args {
    #[arg(long)]
    corpus: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let report = match args.corpus {
        Some(path) => lay::nanda_wave::run_russian_morphology_proof_path(&path)?,
        None => lay::nanda_wave::run_embedded_russian_morphology_proof()?,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
