use std::io;
use std::path::PathBuf;

use lay::nanda_wave::{build_lay_self_teacher_l3_report, LaySelfTeacherL3Config};

pub(crate) fn print_json(args: &[String]) -> io::Result<()> {
    let mut config = LaySelfTeacherL3Config::default();
    if let Some(path) = super::arg_value(args, "--out-dir") {
        config.output_dir = PathBuf::from(path);
    }
    if let Some(path) = super::arg_value(args, "--clean-corpus") {
        config.clean_corpus = Some(PathBuf::from(path));
    }
    if let Some(path) = super::arg_value(args, "--usage-events") {
        config.usage_events = Some(PathBuf::from(path));
        config.include_default_live_feedback = true;
    }
    if args.iter().any(|arg| arg == "--no-live-feedback") {
        config.include_default_live_feedback = false;
    }
    if let Some(value) = parse_usize(args, "--max-phrases") {
        config.max_phrases = value.max(1);
    }
    if let Some(value) = parse_usize(args, "--max-pairs") {
        config.max_pairs = value.max(1);
    }
    if let Some(value) = parse_usize(args, "--max-fragments") {
        config.max_fragments = value;
    }
    if let Some(value) = parse_u32(args, "--min-profile-support") {
        config.min_profile_support = value.max(1);
    }
    if let Some(value) = parse_u32(args, "--min-surface-support") {
        config.min_surface_support = value.max(1);
    }
    let report = build_lay_self_teacher_l3_report(config)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn parse_usize(args: &[String], name: &str) -> Option<usize> {
    super::arg_value(args, name).and_then(|value| value.parse().ok())
}

fn parse_u32(args: &[String], name: &str) -> Option<u32> {
    super::arg_value(args, name).and_then(|value| value.parse().ok())
}
