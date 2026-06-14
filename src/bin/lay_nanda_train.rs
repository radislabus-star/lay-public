use lay::microbrain::{
    default_trained_layout_signal_path, evaluate_expert64_layout_signal,
    train_expert64_layout_signal, Expert64Cell,
};
use lay::nanda_training_data::read_training_rows;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dataset = arg_path(&args, "--dataset")
        .unwrap_or_else(|| PathBuf::from("data/neural_arbiter/dataset.tsv"));
    let holdout = arg_path(&args, "--holdout")
        .unwrap_or_else(|| PathBuf::from("data/neural_arbiter/holdout.tsv"));
    let out = arg_path(&args, "--out").unwrap_or_else(default_trained_layout_signal_path);
    let epochs = arg_value(&args, "--epochs")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(8);

    let train_rows = read_training_rows(&dataset)?;
    let holdout_rows = read_training_rows(&holdout)?;
    let (cell, train_report) = train_expert64_layout_signal(&train_rows, epochs);
    let holdout_report = evaluate_expert64_layout_signal(&cell, &holdout_rows, epochs);
    cell.write(&out)?;

    let bytes = fs::metadata(&out)?.len();
    let loaded = Expert64Cell::read(&out)?;
    println!("expert: {}", loaded.expert_id);
    println!("out: {}", out.display());
    println!("bytes: {bytes}");
    println!(
        "train: rows={} groups={} acc={:.3} group_acc={:.3} pos={:.3} neg={:.3}",
        train_report.rows,
        train_report.groups,
        train_report.accuracy,
        train_report.group_accuracy,
        train_report.positive_accuracy,
        train_report.negative_accuracy
    );
    println!(
        "holdout: rows={} groups={} acc={:.3} group_acc={:.3} pos={:.3} neg={:.3}",
        holdout_report.rows,
        holdout_report.groups,
        holdout_report.accuracy,
        holdout_report.group_accuracy,
        holdout_report.positive_accuracy,
        holdout_report.negative_accuracy
    );
    Ok(())
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].as_str())
}

fn arg_path(args: &[String], name: &str) -> Option<PathBuf> {
    arg_value(args, name).map(PathBuf::from)
}
