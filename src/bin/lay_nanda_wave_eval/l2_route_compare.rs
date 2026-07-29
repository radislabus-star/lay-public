use lay::config::LayConfig;
use lay::correction_core::{
    resolve_text_correction, CandidateGateAction, CandidateReadoutRoute, CorrectionMode,
    CorrectionRequest, CorrectionResolution, UnifiedCorrectionCandidate,
};
use rayon::prelude::*;
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_CORRECTIONS_PATH: &str = ".local/share/lay/corrections.jsonl";
const DEFAULT_LIMIT: usize = 200;
const DEFAULT_EXAMPLES: usize = 8;

struct RouteComparisonRecord {
    input: String,
    target: Option<String>,
    reference: CorrectionResolution,
    shadow: CorrectionResolution,
}

pub(crate) fn print_json(args: &[String]) -> io::Result<()> {
    let path = input_path(args)?;
    let limit = arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_LIMIT);
    let examples = arg_value(args, "--examples")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_EXAMPLES);
    let jobs = arg_value(args, "--jobs")
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|jobs| *jobs > 0)
        .unwrap_or_else(default_jobs);
    let report = report_json(&path, limit, examples, jobs)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn report_json(path: &Path, limit: usize, max_examples: usize, jobs: usize) -> io::Result<Value> {
    let text = fs::read_to_string(path)?;
    let cfg = LayConfig::load();
    let lines = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let start = lines.len().saturating_sub(limit);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .map_err(io::Error::other)?;
    let records = pool.install(|| {
        lines[start..]
            .par_iter()
            .filter_map(|line| {
                let value = serde_json::from_str::<Value>(line).ok()?;
                let input = value.get("lay_from").and_then(Value::as_str)?;
                if input.trim().is_empty() {
                    return None;
                }
                Some(RouteComparisonRecord {
                    input: input.to_string(),
                    target: full_user_target(&value),
                    reference: resolve_with_route(input, &cfg, CandidateReadoutRoute::FullWave),
                    shadow: resolve_with_route(input, &cfg, CandidateReadoutRoute::L2FieldShadow),
                })
            })
            .collect::<Vec<_>>()
    });

    let mut records_used = 0usize;
    let mut surface_diverged = 0usize;
    let mut gate_diverged = 0usize;
    let mut provenance_diverged = 0usize;
    let mut reference_apply = 0usize;
    let mut shadow_apply = 0usize;
    let mut reference_matches_target = 0usize;
    let mut shadow_matches_target = 0usize;
    let mut both_match_target = 0usize;
    let mut reference_apply_matches_target = 0usize;
    let mut shadow_apply_matches_target = 0usize;
    let mut reference_false_authority = 0usize;
    let mut shadow_false_authority = 0usize;
    let mut examples = Vec::new();

    for record in records {
        let RouteComparisonRecord {
            input,
            target,
            reference,
            shadow,
        } = record;
        let surface_changed =
            selected_surface_diverged(reference.selected.as_ref(), shadow.selected.as_ref());
        let gate_changed =
            selected_gate_diverged(reference.selected.as_ref(), shadow.selected.as_ref());
        let provenance_changed =
            selected_provenance_diverged(reference.selected.as_ref(), shadow.selected.as_ref());

        surface_diverged += usize::from(surface_changed);
        gate_diverged += usize::from(gate_changed);
        provenance_diverged += usize::from(provenance_changed);
        let reference_applies = selected_apply(&reference);
        let shadow_applies = selected_apply(&shadow);
        reference_apply += usize::from(reference_applies);
        shadow_apply += usize::from(shadow_applies);

        let reference_target_match = target
            .as_deref()
            .is_some_and(|target| selected_matches_target(reference.selected.as_ref(), target));
        let shadow_target_match = target
            .as_deref()
            .is_some_and(|target| selected_matches_target(shadow.selected.as_ref(), target));
        let has_user_target = target.is_some();
        let reference_apply_conflicts =
            has_user_target && reference_applies && !reference_target_match;
        let shadow_apply_conflicts = has_user_target && shadow_applies && !shadow_target_match;
        reference_matches_target += usize::from(reference_target_match);
        shadow_matches_target += usize::from(shadow_target_match);
        both_match_target += usize::from(reference_target_match && shadow_target_match);
        reference_apply_matches_target += usize::from(reference_applies && reference_target_match);
        shadow_apply_matches_target += usize::from(shadow_applies && shadow_target_match);
        reference_false_authority += usize::from(reference_apply_conflicts);
        shadow_false_authority += usize::from(shadow_apply_conflicts);
        records_used += 1;

        if examples.len() < max_examples
            && (surface_changed
                || gate_changed
                || provenance_changed
                || reference_apply_conflicts
                || shadow_apply_conflicts)
        {
            examples.push(json!({
                "input": input,
                "user_target": target,
                "reference_apply_conflicts_user_target": reference_apply_conflicts,
                "shadow_apply_conflicts_user_target": shadow_apply_conflicts,
                "selected_surface_diverged": surface_changed,
                "selected_gate_diverged": gate_changed,
                "selected_provenance_diverged": provenance_changed,
                "reference": resolution_summary_json(CandidateReadoutRoute::FullWave, &reference),
                "shadow": resolution_summary_json(CandidateReadoutRoute::L2FieldShadow, &shadow),
            }));
        }
    }

    Ok(json!({
        "kind": "l2_route_compare_report",
        "status": "ok",
        "source": path.display().to_string(),
        "window": {
            "records_seen": lines.len(),
            "records_used": records_used,
            "limit": limit,
            "jobs": jobs,
        },
        "selected": {
            "surface_diverged": surface_diverged,
            "surface_identical": records_used.saturating_sub(surface_diverged),
            "gate_diverged": gate_diverged,
            "provenance_diverged": provenance_diverged,
            "reference_apply": reference_apply,
            "shadow_apply": shadow_apply,
        },
        "user_target_match": {
            "reference": reference_matches_target,
            "shadow": shadow_matches_target,
            "both": both_match_target,
        },
        "authority_against_user_target": {
            "reference_apply_matches": reference_apply_matches_target,
            "shadow_apply_matches": shadow_apply_matches_target,
            "reference_false_authority": reference_false_authority,
            "shadow_false_authority": shadow_false_authority,
        },
        "examples": examples,
        "read_as": "compare full-wave and l2-field-shadow on real lay_from inputs from corrections.jsonl after lexical-route removal; surface and gate parity matter more than provenance",
    }))
}

fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(20)
}

fn resolve_with_route(
    text: &str,
    cfg: &LayConfig,
    route: CandidateReadoutRoute,
) -> CorrectionResolution {
    resolve_text_correction(CorrectionRequest {
        text,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: cfg.active_correction_safety(),
        typing_assist_pipeline: &cfg.typing_assist_pipeline,
        nanda_autocorrect: true,
        nanda_candidate_route: route,
        nanda_wave_options: cfg.active_nanda_wave_options(),
        mode: CorrectionMode::DeterministicThenNanda,
    })
}

fn selected_apply(resolution: &CorrectionResolution) -> bool {
    resolution
        .selected
        .as_ref()
        .is_some_and(|candidate| candidate.gate.action == CandidateGateAction::Eligible)
}

fn selected_matches_target(candidate: Option<&UnifiedCorrectionCandidate>, target: &str) -> bool {
    candidate
        .is_some_and(|candidate| normalized_text(&candidate.replacement) == normalized_text(target))
}

fn selected_surface_diverged(
    left: Option<&UnifiedCorrectionCandidate>,
    right: Option<&UnifiedCorrectionCandidate>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.replacement != right.replacement,
        (None, None) => false,
        _ => true,
    }
}

fn selected_gate_diverged(
    left: Option<&UnifiedCorrectionCandidate>,
    right: Option<&UnifiedCorrectionCandidate>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.gate.action != right.gate.action
                || left.gate.reason != right.gate.reason
                || left.error_class != right.error_class
        }
        (None, None) => false,
        _ => true,
    }
}

fn selected_provenance_diverged(
    left: Option<&UnifiedCorrectionCandidate>,
    right: Option<&UnifiedCorrectionCandidate>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.source != right.source || left.source_id != right.source_id
        }
        (None, None) => false,
        _ => true,
    }
}

fn resolution_summary_json(
    route: CandidateReadoutRoute,
    resolution: &CorrectionResolution,
) -> Value {
    json!({
        "route": route_name(route),
        "candidate_count": resolution.candidates.len(),
        "selected": resolution.selected.as_ref().map(candidate_summary_json),
        "scoreboard": {
            "total_candidates": resolution.scoreboard.total_candidates,
            "apply_candidates": resolution.scoreboard.apply_candidates,
            "suggest_only_candidates": resolution.scoreboard.suggest_only_candidates,
            "keep_original_candidates": resolution.scoreboard.keep_original_candidates,
            "veto_candidates": resolution.scoreboard.veto_candidates,
        },
    })
}

fn candidate_summary_json(candidate: &UnifiedCorrectionCandidate) -> Value {
    json!({
        "replacement": candidate.replacement,
        "source": format!("{:?}", candidate.source),
        "source_id": candidate.source_id,
        "error_class": candidate.error_class.as_str(),
        "gate_action": format!("{:?}", candidate.gate.action),
        "gate_reason": candidate.gate.reason,
    })
}

fn route_name(route: CandidateReadoutRoute) -> &'static str {
    match route {
        CandidateReadoutRoute::L2FieldShadow => "l2-field-shadow",
        CandidateReadoutRoute::FullWave => "full-wave",
    }
}

fn input_path(args: &[String]) -> io::Result<PathBuf> {
    if let Some(path) = arg_value(args, "--input").or_else(|| arg_value(args, "--learning-log")) {
        return Ok(PathBuf::from(path));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(DEFAULT_CORRECTIONS_PATH))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "HOME is not set and no input path was provided",
            )
        })
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == name {
            return iter.next().map(String::as_str);
        }
        if let Some(value) = arg.strip_prefix(&(name.to_string() + "=")) {
            return Some(value);
        }
    }
    None
}

fn full_user_target(value: &Value) -> Option<String> {
    if let Some(target) = value.get("user_target").and_then(Value::as_str) {
        return Some(target.to_string());
    }
    lay::word_buffer::reconstruct_user_correction_target(
        value.get("lay_to")?.as_str()?,
        value.get("from")?.as_str()?,
        value.get("to")?.as_str()?,
    )
}

fn normalized_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
