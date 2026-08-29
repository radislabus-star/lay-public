#[cfg(test)]
use std::io::Cursor;
use std::io::{self, Read};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::super::phase_field::hash_text;
use super::ContextPhasePackage;
use super::SurfaceMutationField;
use std::sync::Arc;

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct ContextPhaseCompileInput<'a> {
    pub(crate) corpus_text: &'a str,
    pub(crate) max_fragments: usize,
    pub(crate) min_profile_support: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseCompileReport {
    pub(crate) kind: &'static str,
    pub(crate) architecture: &'static str,
    pub(crate) signature_schema: u32,
    pub(crate) corpus_passes: u8,
    pub(crate) raw_words_stored: bool,
    pub(crate) corpus_fragments: usize,
    pub(crate) transitions: u64,
    pub(crate) semantic_states: usize,
    pub(crate) candidate_profiles: usize,
    pub(crate) pair_profiles: usize,
    pub(crate) exact_pair_profiles: usize,
    pub(crate) generalized_pair_profiles: usize,
    pub(crate) pair_centers: usize,
    pub(crate) positive_centers: usize,
    pub(crate) anti_centers: usize,
    pub(crate) positive_examples: u64,
    pub(crate) negative_examples: u64,
    pub(crate) l2_lattice_negative_examples: u64,
    pub(crate) self_mined_hard_negatives: u64,
    pub(crate) dropped_pair_profiles: u64,
    pub(crate) evicted_provisional_pair_profiles: u64,
    pub(crate) l2_lattice_probes: u64,
    pub(crate) l2_probe_workers: usize,
    pub(crate) l2_probe_batch_fragments: usize,
    pub(crate) competition_calibration_cases: usize,
    pub(crate) positive_reinforcements: u64,
    pub(crate) positive_subcenter_splits: u64,
    pub(crate) anti_reinforcements: u64,
    pub(crate) anti_subcenter_splits: u64,
    pub(crate) dropped_semantic_states: u64,
    pub(crate) dropped_profiles: u64,
    pub(crate) evicted_provisional_semantic_states: u64,
    pub(crate) evicted_provisional_profiles: u64,
    pub(crate) pending_negative_profiles: usize,
    pub(crate) pending_negative_centers: usize,
    pub(crate) resident_positive_phase_centers: usize,
    pub(crate) resident_negative_phase_centers: usize,
    pub(crate) resident_hard_negative_phase_centers: usize,
    pub(crate) max_positive_phase_centers: usize,
    pub(crate) max_negative_phase_centers: usize,
    pub(crate) max_hard_negative_phase_centers: usize,
    pub(crate) dropped_pending_negative_profiles: u64,
    pub(crate) evicted_pending_negative_profiles: u64,
    pub(crate) rejected_incompatible_modes: u64,
    pub(crate) rejected_token_count_fragments: usize,
    pub(crate) oversized_fragments: usize,
    pub(crate) invalid_utf8_fragments: usize,
    pub(crate) peak_fragment_bytes: usize,
    pub(crate) estimated_learner_bytes: u64,
    pub(crate) rss_bytes: u64,
    pub(crate) elapsed_millis: u64,
    pub(crate) fragments_per_second: u64,
    pub(crate) global_threshold_micro: i32,
    pub(crate) competition_threshold_micro: i32,
    pub(crate) min_profile_support: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseProgressReport {
    pub(crate) kind: &'static str,
    pub(crate) fragments: usize,
    pub(crate) fragment_limit: usize,
    pub(crate) transitions: u64,
    pub(crate) fragments_per_second: u64,
    pub(crate) eta_seconds: Option<u64>,
    pub(crate) profiles: usize,
    pub(crate) semantic_states: usize,
    pub(crate) phase_centers: usize,
    pub(crate) positive_phase_centers: usize,
    pub(crate) negative_phase_centers: usize,
    pub(crate) hard_negative_phase_centers: usize,
    pub(crate) pair_profiles: usize,
    pub(crate) pair_centers: usize,
    pub(crate) pending_negative_profiles: usize,
    pub(crate) pending_negative_centers: usize,
    pub(crate) competition_calibration_cases: usize,
    pub(crate) estimated_learner_bytes: u64,
    pub(crate) rss_bytes: u64,
}

/// Receipt for classifying personal IME feedback against a phase packet.
///
/// A single live surface never mutates a generalizing L3 packet. The JSONL
/// event source is never copied into it; accepted evidence is exported as a
/// separate corpus and rejection without a final target is censored.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseFeedbackOverlayReport {
    pub(crate) kind: &'static str,
    pub(crate) raw_words_stored: bool,
    pub(crate) source_events: usize,
    pub(crate) positive_source_events: usize,
    pub(crate) negative_source_events: usize,
    pub(crate) positive_admitted: usize,
    pub(crate) negative_admitted: usize,
    /// Accepted live text is a personal corpus source, not an independent
    /// surface for a generalizing L3 phase packet.
    pub(crate) positive_censored_pending_surface_support: usize,
    /// A dismissed suggestion has no known replacement target. It is an
    /// observation for outcome telemetry, but not evidence for an anti-wave.
    pub(crate) negative_censored_no_observed_target: usize,
    pub(crate) skipped_unattested_context: usize,
    pub(crate) skipped_unattested_positive: usize,
    pub(crate) skipped_missing_profile: usize,
    pub(crate) candidate_profiles: usize,
    pub(crate) positive_centers: usize,
    pub(crate) anti_centers: usize,
}

/// Receipt for the private, locally generated L3 corpus. The corpus is
/// intentionally built only from outcomes the user accepted or completed, not
/// from arbitrary typed text or automatic corrections.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContextPhaseFeedbackCorpusReport {
    pub(crate) kind: &'static str,
    pub(crate) raw_words_stored_in_packet: bool,
    pub(crate) source_events: usize,
    pub(crate) accepted_source_events: usize,
    pub(crate) rejected_source_events: usize,
    pub(crate) skipped_unattested: usize,
    pub(crate) skipped_duplicate_cap: usize,
    pub(crate) corpus_lines: usize,
    pub(crate) unique_phrases: usize,
}

#[derive(Debug, Deserialize)]
struct FeedbackEvent {
    kind: String,
    word: String,
    #[serde(default)]
    context: Vec<String>,
}

/// Produces a private clean-text training surface from explicit IME outcomes.
///
/// The output is a corpus source, never a runtime packet. Each emitted phrase
/// is lexically attested and bounded by `max_repeat_per_phrase`, so one user's
/// repeated completion cannot dominate the shared clean corpus during a later
/// cold compile. Rejections are counted for observability, but do not become
/// anti-wave evidence until a linked final target or undo receipt exists.
pub(crate) fn build_feedback_corpus(
    events_text: &str,
    max_repeat_per_phrase: usize,
) -> io::Result<(String, ContextPhaseFeedbackCorpusReport)> {
    use std::collections::BTreeMap;

    let max_repeat_per_phrase = max_repeat_per_phrase.max(1);
    let mut report = ContextPhaseFeedbackCorpusReport {
        kind: "l3_context_phase_feedback_corpus",
        raw_words_stored_in_packet: false,
        source_events: 0,
        accepted_source_events: 0,
        rejected_source_events: 0,
        skipped_unattested: 0,
        skipped_duplicate_cap: 0,
        corpus_lines: 0,
        unique_phrases: 0,
    };
    let mut phrase_counts = BTreeMap::<String, usize>::new();
    let mut lines = Vec::new();

    for (line_number, line) in events_text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: FeedbackEvent = serde_json::from_str(line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid typing feedback JSONL at line {}: {error}",
                    line_number + 1
                ),
            )
        })?;
        report.source_events += 1;
        match event.kind.as_str() {
            "accepted_ime" | "edited_ime" | "confirmed_ime_prediction" => {
                report.accepted_source_events += 1;
            }
            "rejected_ime" | "rejected_candidate" => {
                report.rejected_source_events += 1;
                continue;
            }
            _ => continue,
        }

        let mut phrase = event
            .context
            .iter()
            .map(|word| crate::typing_memory::normalize_memory_word(word))
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        let candidate = crate::typing_memory::normalize_memory_word(&event.word);
        if candidate.is_empty()
            || (matches!(
                event.kind.as_str(),
                "edited_ime" | "confirmed_ime_prediction"
            ) && !crate::typing_memory::learning_target_is_attested(&candidate))
        {
            report.skipped_unattested += 1;
            continue;
        }
        phrase.push(candidate);
        let phrase = phrase.join(" ");
        if !crate::typing_memory::phrase_is_attested_for_learning(&phrase) {
            report.skipped_unattested += 1;
            continue;
        }
        let count = phrase_counts.entry(phrase.clone()).or_default();
        if *count >= max_repeat_per_phrase {
            report.skipped_duplicate_cap += 1;
            continue;
        }
        *count += 1;
        lines.push(phrase);
    }

    report.corpus_lines = lines.len();
    report.unique_phrases = phrase_counts.len();
    let mut corpus = lines.join("\n");
    if !corpus.is_empty() {
        corpus.push('\n');
    }
    Ok((corpus, report))
}

#[cfg(test)]
pub(crate) fn compile_context_phase(
    input: ContextPhaseCompileInput<'_>,
) -> (ContextPhasePackage, ContextPhaseCompileReport) {
    // Tests exercise the same learned-surface route as production compilation.
    // The fixture supplies geometry only; its literal words never enter a
    // package or participate in candidate ranking.
    let surface_field = SurfaceMutationField::from_corrections_jsonl(
        concat!(
            r#"{"from":"дожь","to":"дождь"}"#,
            "\n",
            r#"{"from":"све","to":"свет"}"#,
            "\n",
            r#"{"from":"гор","to":"горит"}"#,
        ),
        1,
    )
    .expect("valid learned surface fixture");
    compile_context_phase_reader_with_surface_field(
        Cursor::new(input.corpus_text.as_bytes()),
        input.max_fragments,
        input.min_profile_support,
        0,
        std::sync::Arc::new(surface_field),
        |_, _| Ok(()),
    )
    .expect("in-memory L3 corpus reader cannot fail")
}

pub(crate) fn compile_context_phase_reader<R, F>(
    reader: R,
    max_fragments: usize,
    min_profile_support: u32,
    snapshot_every_fragments: usize,
    snapshot: F,
) -> io::Result<(ContextPhasePackage, ContextPhaseCompileReport)>
where
    R: Read,
    F: FnMut(&ContextPhasePackage, &ContextPhaseProgressReport) -> io::Result<()>,
{
    compile_context_phase_reader_with_surface_field(
        reader,
        max_fragments,
        min_profile_support,
        snapshot_every_fragments,
        Arc::new(SurfaceMutationField::default()),
        snapshot,
    )
}

pub(crate) fn surface_field_from_corrections_path(
    path: &std::path::Path,
    min_support: u32,
) -> io::Result<SurfaceMutationField> {
    let text = std::fs::read_to_string(path)?;
    SurfaceMutationField::from_corrections_jsonl(&text, min_support)
}

pub(crate) fn compile_context_phase_reader_with_surface_field<R, F>(
    reader: R,
    max_fragments: usize,
    min_profile_support: u32,
    snapshot_every_fragments: usize,
    surface_field: Arc<SurfaceMutationField>,
    snapshot: F,
) -> io::Result<(ContextPhasePackage, ContextPhaseCompileReport)>
where
    R: Read,
    F: FnMut(&ContextPhasePackage, &ContextPhaseProgressReport) -> io::Result<()>,
{
    compile_context_phase_reader_with_surface_field_and_schema(
        reader,
        max_fragments,
        min_profile_support,
        snapshot_every_fragments,
        super::SIGNATURE_SCHEMA_RELATION_ROLES,
        surface_field,
        snapshot,
    )
}

pub(crate) fn compile_context_phase_reader_with_surface_field_and_schema<R, F>(
    reader: R,
    max_fragments: usize,
    min_profile_support: u32,
    snapshot_every_fragments: usize,
    signature_schema: u32,
    surface_field: Arc<SurfaceMutationField>,
    snapshot: F,
) -> io::Result<(ContextPhasePackage, ContextPhaseCompileReport)>
where
    R: Read,
    F: FnMut(&ContextPhasePackage, &ContextPhaseProgressReport) -> io::Result<()>,
{
    compile_context_phase_reader_with_projection_base(
        reader,
        max_fragments,
        min_profile_support,
        snapshot_every_fragments,
        signature_schema,
        surface_field,
        None,
        snapshot,
    )
}

pub(crate) fn compile_context_phase_delta_reader_with_projection_base<R, F>(
    reader: R,
    min_profile_support: u32,
    signature_schema: u32,
    surface_field: Arc<SurfaceMutationField>,
    base: &ContextPhasePackage,
    snapshot: F,
) -> io::Result<(ContextPhasePackage, ContextPhaseCompileReport)>
where
    R: Read,
    F: FnMut(&ContextPhasePackage, &ContextPhaseProgressReport) -> io::Result<()>,
{
    compile_context_phase_reader_with_projection_base(
        reader,
        0,
        min_profile_support,
        0,
        signature_schema,
        surface_field,
        Some(base),
        snapshot,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn compile_context_phase_reader_with_projection_base<R, F>(
    reader: R,
    max_fragments: usize,
    min_profile_support: u32,
    snapshot_every_fragments: usize,
    signature_schema: u32,
    surface_field: Arc<SurfaceMutationField>,
    projection_base: Option<&ContextPhasePackage>,
    mut snapshot: F,
) -> io::Result<(ContextPhasePackage, ContextPhaseCompileReport)>
where
    R: Read,
    F: FnMut(&ContextPhasePackage, &ContextPhaseProgressReport) -> io::Result<()>,
{
    let started = Instant::now();
    let config = super::online::OnlineContextPhaseConfig::production_with_signature_schema(
        min_profile_support,
        signature_schema,
    );
    let mut learner = match projection_base {
        Some(base) => super::online::OnlineContextPhaseLearner::new_with_projection_base(
            config,
            surface_field,
            base,
        ),
        None => {
            super::online::OnlineContextPhaseLearner::new_with_surface_field(config, surface_field)
        }
    };
    let l2_pool = super::online::L2ProbePool::new();
    let mut pending_l2 = Vec::new();
    let mut batch_fragments = 0_usize;
    let stream_stats =
        super::stream::visit_tokenized_fragments(reader, max_fragments, |ordinal, tokens| {
            pending_l2.extend(learner.ingest_fragment_positive(tokens));
            batch_fragments = batch_fragments.saturating_add(1);
            let fragments = ordinal.saturating_add(1);
            let snapshot_due =
                snapshot_every_fragments > 0 && fragments % snapshot_every_fragments == 0;
            if batch_fragments >= super::online::L2_PROBE_BATCH_FRAGMENTS || snapshot_due {
                learner.apply_l2_probe_batch(&l2_pool, &mut pending_l2)?;
                batch_fragments = 0;
            }
            if snapshot_due {
                let package = learner.snapshot();
                let progress =
                    progress_report(&learner, fragments, max_fragments, started.elapsed());
                snapshot(&package, &progress)?;
            }
            Ok(())
        })?;
    learner.apply_l2_probe_batch(&l2_pool, &mut pending_l2)?;
    let package = learner.snapshot();
    let (exact_pair_profiles, generalized_pair_profiles) = package.pair_profile_counts();
    let elapsed = started.elapsed();
    let stats = learner.stats();
    let report = ContextPhaseCompileReport {
        kind: "l3_context_phase_compile",
        architecture: "online_relation_phase_v4_role_scene_lattice",
        signature_schema: package.signature_schema,
        corpus_passes: 1,
        raw_words_stored: false,
        corpus_fragments: stream_stats.accepted_fragments,
        transitions: stats.transitions,
        semantic_states: package.semantic_states.len(),
        candidate_profiles: package.profiles.len(),
        pair_profiles: package.pair_profiles.len(),
        exact_pair_profiles,
        generalized_pair_profiles,
        pair_centers: package
            .pair_profiles
            .iter()
            .map(|profile| {
                profile.low_wins.len()
                    + profile.high_wins.len()
                    + profile.hard_low_wins.len()
                    + profile.hard_high_wins.len()
            })
            .sum(),
        positive_centers: package
            .profiles
            .iter()
            .map(|profile| profile.positive.len())
            .sum(),
        anti_centers: package
            .profiles
            .iter()
            .map(|profile| profile.negative.len() + profile.hard_negative.len())
            .sum(),
        positive_examples: package
            .profiles
            .iter()
            .map(|profile| u64::from(profile.positive_examples))
            .sum(),
        negative_examples: package
            .profiles
            .iter()
            .map(|profile| u64::from(profile.negative_examples))
            .sum(),
        l2_lattice_negative_examples: stats.l2_lattice_negative_examples,
        self_mined_hard_negatives: stats.hard_negative_false_winners,
        dropped_pair_profiles: stats.dropped_pair_profiles,
        evicted_provisional_pair_profiles: stats.evicted_provisional_pair_profiles,
        l2_lattice_probes: stats.l2_lattice_probes,
        l2_probe_workers: l2_pool.worker_count(),
        l2_probe_batch_fragments: super::online::L2_PROBE_BATCH_FRAGMENTS,
        competition_calibration_cases: learner.competition_calibration_cases(),
        positive_reinforcements: stats.positive_reinforcements,
        positive_subcenter_splits: stats.positive_splits,
        anti_reinforcements: stats.anti_reinforcements,
        anti_subcenter_splits: stats.anti_splits,
        dropped_semantic_states: stats.dropped_semantic_states,
        dropped_profiles: stats.dropped_profiles,
        evicted_provisional_semantic_states: stats.evicted_provisional_semantic_states,
        evicted_provisional_profiles: stats.evicted_provisional_profiles,
        pending_negative_profiles: learner.pending_negative_profile_count(),
        pending_negative_centers: learner.pending_negative_center_count(),
        resident_positive_phase_centers: learner.positive_phase_center_count(),
        resident_negative_phase_centers: learner.negative_phase_center_count(),
        resident_hard_negative_phase_centers: learner.hard_negative_phase_center_count(),
        max_positive_phase_centers: config.max_positive_phase_centers,
        max_negative_phase_centers: config.max_negative_phase_centers,
        max_hard_negative_phase_centers: config.max_hard_negative_phase_centers,
        dropped_pending_negative_profiles: stats.dropped_pending_negative_profiles,
        evicted_pending_negative_profiles: stats.evicted_pending_negative_profiles,
        rejected_incompatible_modes: stats.rejected_incompatible_modes,
        rejected_token_count_fragments: stream_stats.rejected_token_count,
        oversized_fragments: stream_stats.oversized_fragments,
        invalid_utf8_fragments: stream_stats.invalid_utf8_fragments,
        peak_fragment_bytes: stream_stats.peak_fragment_bytes,
        estimated_learner_bytes: learner.estimated_bytes(),
        rss_bytes: current_rss_bytes(),
        elapsed_millis: duration_millis(elapsed),
        fragments_per_second: rate_per_second(stream_stats.accepted_fragments, elapsed),
        global_threshold_micro: package.global_threshold_micro,
        competition_threshold_micro: package.competition_threshold_micro,
        min_profile_support: config.min_profile_support,
    };
    Ok((package, report))
}

fn progress_report(
    learner: &super::online::OnlineContextPhaseLearner,
    fragments: usize,
    fragment_limit: usize,
    elapsed: Duration,
) -> ContextPhaseProgressReport {
    let rate = rate_per_second(fragments, elapsed);
    let eta_seconds = (fragment_limit > fragments && rate > 0)
        .then(|| fragment_limit.saturating_sub(fragments) as u64 / rate.max(1));
    ContextPhaseProgressReport {
        kind: "l3_context_phase_online_progress",
        fragments,
        fragment_limit,
        transitions: learner.stats().transitions,
        fragments_per_second: rate,
        eta_seconds,
        profiles: learner.profile_count(),
        semantic_states: learner.semantic_state_count(),
        phase_centers: learner.phase_center_count(),
        positive_phase_centers: learner.positive_phase_center_count(),
        negative_phase_centers: learner.negative_phase_center_count(),
        hard_negative_phase_centers: learner.hard_negative_phase_center_count(),
        pair_profiles: learner.pair_profile_count(),
        pair_centers: learner.pair_center_count(),
        pending_negative_profiles: learner.pending_negative_profile_count(),
        pending_negative_centers: learner.pending_negative_center_count(),
        competition_calibration_cases: learner.competition_calibration_cases(),
        estimated_learner_bytes: learner.estimated_bytes(),
        rss_bytes: current_rss_bytes(),
    }
}

fn rate_per_second(items: usize, elapsed: Duration) -> u64 {
    if items == 0 {
        return 0;
    }
    ((items as f64 / elapsed.as_secs_f64().max(0.001)).round() as u64).max(1)
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn current_rss_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|text| text.split_whitespace().nth(1)?.parse::<u64>().ok())
        .map(|pages| pages.saturating_mul(4_096))
        .unwrap_or_default()
}

/// Classifies explicit user IME outcomes against a canonical context package.
///
/// This preserves the command contract while refusing to make a one-surface
/// live outcome general L3 authority. The feedback corpus and L4 retain the
/// local signal; promotion needs independent cold-surface support.
pub(crate) fn apply_feedback_overlay(
    package: &mut ContextPhasePackage,
    events_text: &str,
) -> io::Result<ContextPhaseFeedbackOverlayReport> {
    let mut report = ContextPhaseFeedbackOverlayReport {
        kind: "l3_context_phase_feedback_overlay",
        raw_words_stored: false,
        source_events: 0,
        positive_source_events: 0,
        negative_source_events: 0,
        positive_admitted: 0,
        negative_admitted: 0,
        positive_censored_pending_surface_support: 0,
        negative_censored_no_observed_target: 0,
        skipped_unattested_context: 0,
        skipped_unattested_positive: 0,
        skipped_missing_profile: 0,
        candidate_profiles: package.profiles.len(),
        positive_centers: 0,
        anti_centers: 0,
    };

    for (line_number, line) in events_text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: FeedbackEvent = serde_json::from_str(line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid typing feedback JSONL at line {}: {error}",
                    line_number + 1
                ),
            )
        })?;
        match event.kind.as_str() {
            "accepted_ime" | "edited_ime" | "confirmed_ime_prediction" => {
                report.source_events += 1;
                report.positive_source_events += 1;
                let context = event
                    .context
                    .iter()
                    .map(|word| crate::typing_memory::normalize_memory_word(word))
                    .filter(|word| !word.is_empty())
                    .collect::<Vec<_>>();
                let candidate = crate::typing_memory::normalize_memory_word(&event.word);
                if context.is_empty()
                    || !crate::typing_memory::phrase_is_attested_for_learning(&context.join(" "))
                {
                    report.skipped_unattested_context += 1;
                    continue;
                }
                if matches!(
                    event.kind.as_str(),
                    "edited_ime" | "confirmed_ime_prediction"
                ) && !crate::typing_memory::learning_target_is_attested(&candidate)
                {
                    report.skipped_unattested_positive += 1;
                    continue;
                }
                let mut phrase = context;
                phrase.push(candidate);
                if !crate::typing_memory::phrase_is_attested_for_learning(&phrase.join(" ")) {
                    report.skipped_unattested_positive += 1;
                    continue;
                }
                // One user's accepted completion is meaningful local evidence,
                // but one surface cannot safely create a general L3 center.
                // It is exported by build_feedback_corpus and routed to L4;
                // only a later independent cold merge may promote it.
                report.positive_censored_pending_surface_support += 1;
                continue;
            }
            "rejected_ime" | "rejected_candidate" => {
                report.source_events += 1;
                report.negative_source_events += 1;
                // Closing or replacing a suggestion does not tell us which
                // candidate won instead. Treat it as censored until the
                // runtime records a linked observed target or an explicit
                // undo receipt. Otherwise an unrelated correct completion
                // can be suppressed in a nearby phase scene.
                report.negative_censored_no_observed_target += 1;
                continue;
            }
            _ => continue,
        }
    }
    report.positive_centers = package
        .profiles
        .iter()
        .map(|profile| profile.positive.len())
        .sum();
    report.anti_centers = package
        .profiles
        .iter()
        .map(|profile| profile.negative.len() + profile.hard_negative.len())
        .sum();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_learns_context_centers_and_destructive_competitors() {
        let corpus = "на улице снова идет дождь. вечером на улице идет дождь. утром на улице идет дождь. в доме снова горит свет. вечером в доме горит свет. утром в доме горит свет.";
        let (package, report) = compile_context_phase(ContextPhaseCompileInput {
            corpus_text: corpus,
            max_fragments: 0,
            min_profile_support: 2,
        });

        assert!(report.semantic_states > 0);
        assert!(report.candidate_profiles > 0);
        assert!(report.positive_centers > 0);
        assert!(report.l2_lattice_negative_examples > 0);
        let readouts = package.score_candidates(
            &super::super::super::llmwave::tokenize("вечером на улице идет"),
            &["дождь", "домик"],
        );
        assert!(readouts[0].profile_present);
        assert!(readouts[0].margin_micro >= readouts[1].margin_micro);
    }

    #[test]
    fn online_compiler_emits_bounded_periodic_snapshots() {
        let corpus = concat!(
            "на улице идет дождь. ",
            "вечером на улице идет дождь. ",
            "утром на улице идет дождь. ",
            "сегодня на улице идет дождь."
        );
        let mut snapshots = Vec::new();
        let (_, report) =
            compile_context_phase_reader(corpus.as_bytes(), 0, 2, 2, |package, progress| {
                snapshots.push((progress.fragments, package.corpus_fragments));
                Ok(())
            })
            .unwrap();

        assert_eq!(snapshots, vec![(2, 2), (4, 4)]);
        assert_eq!(
            report.architecture,
            "online_relation_phase_v4_role_scene_lattice"
        );
        assert_eq!(report.corpus_passes, 1);
        assert!(!report.raw_words_stored);
    }

    #[test]
    fn delta_compiler_inherits_legacy_base_signature_schema() {
        let corpus = concat!(
            "обновлять модель по ходу. ",
            "изменять поле на ходу. ",
            "обновлять модель по ходу. ",
            "изменять поле на ходу."
        );
        let (package, report) = compile_context_phase_reader_with_surface_field_and_schema(
            corpus.as_bytes(),
            0,
            2,
            0,
            super::super::SIGNATURE_SCHEMA_LEGACY,
            Arc::new(SurfaceMutationField::default()),
            |_, _| Ok(()),
        )
        .unwrap();

        assert_eq!(
            package.signature_schema,
            super::super::SIGNATURE_SCHEMA_LEGACY
        );
        assert_eq!(
            report.signature_schema,
            super::super::SIGNATURE_SCHEMA_LEGACY
        );
        assert_eq!(report.corpus_passes, 1);
    }

    #[test]
    fn feedback_overlay_censors_single_surface_outcomes_without_storing_live_text() {
        let corpus = concat!(
            "на улице идет дождь. ",
            "вечером на улице идет дождь. ",
            "утром на улице идет дождь. ",
            "в комнате горит свет. ",
            "вечером в комнате горит свет. ",
            "утром в комнате горит свет."
        );
        let (mut package, _) = compile_context_phase(ContextPhaseCompileInput {
            corpus_text: corpus,
            max_fragments: 0,
            min_profile_support: 2,
        });
        let rain_hash = hash_text("дождь");
        let light_hash = hash_text("свет");
        let before_rain = package
            .profile(rain_hash)
            .expect("corpus profile")
            .positive_examples;
        let before_light = package
            .profile(light_hash)
            .expect("corpus profile")
            .negative_examples;
        let events = concat!(
            r#"{"kind":"confirmed_ime_prediction","word":"дождь","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"rejected_ime","word":"свет","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"typed","word":"мусор","context":["на","улице"]}"#,
        );

        let report = apply_feedback_overlay(&mut package, events).expect("valid feedback");

        assert!(!report.raw_words_stored);
        assert_eq!(report.positive_admitted, 0);
        assert_eq!(report.negative_admitted, 0);
        assert_eq!(report.positive_censored_pending_surface_support, 1);
        assert_eq!(report.negative_censored_no_observed_target, 1);
        assert_eq!(report.source_events, 2);
        assert_eq!(
            package.profile(rain_hash).unwrap().positive_examples,
            before_rain
        );
        assert_eq!(
            package.profile(light_hash).unwrap().negative_examples,
            before_light
        );
        let dir =
            std::env::temp_dir().join(format!("lay-l3-feedback-overlay-{}", std::process::id()));
        let path = dir.join("feedback.nwpc");
        std::fs::create_dir_all(&dir).unwrap();
        super::super::write_package(&path, &package).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(!bytes
            .windows("дождь".len())
            .any(|window| window == "дождь".as_bytes()));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn feedback_corpus_keeps_only_attested_explicit_accepts() {
        let events = concat!(
            r#"{"kind":"accepted_ime","word":"дождь","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"confirmed_ime_prediction","word":"дождь","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"rejected_ime","word":"свет","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"typed","word":"дожть","context":["на","улице","идёт"]}"#,
            "\n",
            r#"{"kind":"accepted_ime","word":"дожть","context":["на","улице","идёт"]}"#,
        );

        let (corpus, report) = build_feedback_corpus(events, 1).expect("feedback corpus");

        assert_eq!(corpus, "на улице идёт дождь\n");
        assert_eq!(report.corpus_lines, 1);
        assert_eq!(report.unique_phrases, 1);
        assert_eq!(report.rejected_source_events, 1);
        assert_eq!(report.skipped_duplicate_cap, 1);
        assert_eq!(report.skipped_unattested, 1);
        assert!(!report.raw_words_stored_in_packet);
    }

    #[test]
    fn feedback_corpus_keeps_final_word_from_partial_ime_edit() {
        let events = r#"{"kind":"edited_ime","word":"прекрасно","context":["это","было"],"from":"прекрасный","to":"прекрасно","source":"ime","operation":"completion_edit"}"#;

        let (corpus, report) = build_feedback_corpus(events, 2).expect("feedback corpus");

        assert_eq!(corpus, "это было прекрасно\n");
        assert_eq!(report.accepted_source_events, 1);
        assert_eq!(report.rejected_source_events, 0);
        assert_eq!(report.corpus_lines, 1);
    }

    #[test]
    fn feedback_corpus_rejects_unattested_prediction_and_edit_targets() {
        let events = r#"{"kind":"confirmed_ime_prediction","word":"режимем","context":["в","норм"],"source":"ime","operation":"prediction_match"}
{"kind":"edited_ime","word":"ивдешь","context":["косяков","не"],"from":"использовать","to":"ивдешь","source":"ime","operation":"completion_edit"}"#;

        let (corpus, report) = build_feedback_corpus(events, 2).expect("feedback corpus");

        assert!(corpus.is_empty());
        assert_eq!(report.accepted_source_events, 2);
        assert_eq!(report.skipped_unattested, 2);
    }
}
