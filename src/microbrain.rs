//! Experimental NANDA micro-expert network for typing correction candidates.
//!
//! Writer cells may propose correction candidates, judge cells score them, and
//! guard cells veto unsafe ones. The module never writes to the desktop directly:
//! accepted candidates still go through the normal lay replacement pipeline.

use crate::dict::{convert, detect_direction};
use crate::lexicon::{is_common_ru_word, is_ru_short_function_word, is_ru_short_pronoun};
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_metrics::{
    common_replacement_span, has_cyrillic, has_latin, is_cyrillic_char, normalized_edit_distance,
    without_whitespace,
};
use crate::typing_rule_graph::ids;
use crate::word_reader::split_word_punctuation;
use crate::word_recognizer::{
    is_plain_layout_autocorrect_risky, is_probably_completed_natural_word, recognize_token,
    WordKind,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const EXPERT64_STATE_BYTES: usize = 64 * 1024;
pub const EXPERT64_MAGIC: &str = "LAYEX64";
pub const MIN_EXPERT_CELL_BYTES: usize = 16 * 1024;
pub const MAX_EXPERT_CELL_BYTES: usize = 256 * 1024;
pub const TRAINED_LAYOUT_SIGNAL_EXPERT_ID: &str = "layout_signal_64k_trained";
const EXPERT64_HEADER_BYTES: usize = 256;
const EXPERT64_WEIGHT_BYTES: usize = EXPERT64_STATE_BYTES - EXPERT64_HEADER_BYTES;
const EXPERT64_VERSION: u32 = 1;
const EXPERT64_SCHEMA_HASH: u64 = 0x6c61_792e_6d66_7632;
const TRAINED_SCORE_SCALE: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpertDomain {
    Layout,
    Typo,
    Guard,
    User,
    App,
    Language,
    Mesh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpertWeightFormat {
    Heuristic,
    Int8,
    F16,
    F32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expert64Profile {
    pub magic: &'static str,
    pub expert_id: &'static str,
    pub version: u32,
    pub domain: ExpertDomain,
    pub input_schema_id: &'static str,
    pub weight_format: ExpertWeightFormat,
    pub state_budget_bytes: usize,
    pub calibration: Expert64Calibration,
    pub stats: Expert64Stats,
}

impl Expert64Profile {
    fn heuristic(expert_id: &'static str, domain: ExpertDomain) -> Self {
        Self {
            magic: EXPERT64_MAGIC,
            expert_id,
            version: 1,
            domain,
            input_schema_id: "lay.microfeatures.v2",
            weight_format: ExpertWeightFormat::Heuristic,
            state_budget_bytes: EXPERT64_STATE_BYTES,
            calibration: Expert64Calibration::default(),
            stats: Expert64Stats::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Expert64Calibration {
    pub accept_threshold: f32,
    pub veto_threshold: f32,
}

impl Default for Expert64Calibration {
    fn default() -> Self {
        Self {
            accept_threshold: 0.62,
            veto_threshold: 0.10,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Expert64Stats {
    pub seen_count: u64,
    pub accepted_count: u64,
    pub reverted_count: u64,
    pub false_positive_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expert64Cell {
    pub expert_id: String,
    pub version: u32,
    pub schema_hash: u64,
    pub weights: Vec<i8>,
}

impl Expert64Cell {
    pub fn neutral(expert_id: &str) -> Self {
        Self {
            expert_id: expert_id.to_string(),
            version: EXPERT64_VERSION,
            schema_hash: EXPERT64_SCHEMA_HASH,
            weights: vec![0; EXPERT64_WEIGHT_BYTES],
        }
    }

    pub fn read(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_bytes())
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != EXPERT64_STATE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expert64 packet must be exactly 64 KiB",
            ));
        }
        if &bytes[0..7] != EXPERT64_MAGIC.as_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Expert64 magic",
            ));
        }
        let version = u32::from_le_bytes(bytes[8..12].try_into().expect("version bytes"));
        let schema_hash = u64::from_le_bytes(bytes[12..20].try_into().expect("schema bytes"));
        let id_end = bytes[32..128]
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(96);
        let expert_id = String::from_utf8_lossy(&bytes[32..32 + id_end]).to_string();
        let weights = bytes[EXPERT64_HEADER_BYTES..]
            .iter()
            .map(|byte| *byte as i8)
            .collect();
        Ok(Self {
            expert_id,
            version,
            schema_hash,
            weights,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; EXPERT64_STATE_BYTES];
        bytes[0..7].copy_from_slice(EXPERT64_MAGIC.as_bytes());
        bytes[8..12].copy_from_slice(&self.version.to_le_bytes());
        bytes[12..20].copy_from_slice(&self.schema_hash.to_le_bytes());
        let id = self.expert_id.as_bytes();
        let id_len = id.len().min(96);
        bytes[32..32 + id_len].copy_from_slice(&id[..id_len]);
        for (idx, weight) in self.weights.iter().take(EXPERT64_WEIGHT_BYTES).enumerate() {
            bytes[EXPERT64_HEADER_BYTES + idx] = *weight as u8;
        }
        bytes
    }

    fn score(&self, features: &[String]) -> f32 {
        if self.weights.is_empty() {
            return 0.56;
        }
        let sum: i32 = features
            .iter()
            .map(|feature| {
                let idx = feature_index(feature, self.weights.len());
                self.weights[idx] as i32
            })
            .sum();
        sigmoid(sum as f32 / TRAINED_SCORE_SCALE)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expert64TrainingRow {
    pub group_id: String,
    pub original: String,
    pub candidate: String,
    pub operation: String,
    pub label: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expert64TrainingReport {
    pub rows: usize,
    pub groups: usize,
    pub epochs: usize,
    pub accuracy: f32,
    pub group_accuracy: f32,
    pub positive_accuracy: f32,
    pub negative_accuracy: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuCacheProfile {
    pub l1d_bytes_per_core: usize,
    pub l2_bytes_per_core: usize,
    pub l3_bytes_shared: usize,
    pub cache_line_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpertCellPlan {
    pub cell_bytes: usize,
    pub active_experts: usize,
    pub warm_pool_experts: usize,
}

pub fn plan_expert_cells(cache: CpuCacheProfile) -> ExpertCellPlan {
    let l2_quarter = cache.l2_bytes_per_core / 4;
    let cell_bytes = clamp_power_of_two(l2_quarter, MIN_EXPERT_CELL_BYTES, MAX_EXPERT_CELL_BYTES);
    let active_budget = if cache.l3_bytes_shared > 0 {
        cache.l3_bytes_shared / 8
    } else {
        cache.l2_bytes_per_core
    }
    .max(cell_bytes);
    let active_experts = (active_budget / cell_bytes).clamp(4, 32);
    let warm_budget = if cache.l3_bytes_shared > 0 {
        cache.l3_bytes_shared / 2
    } else {
        cache.l2_bytes_per_core
    };
    let warm_pool_experts = (warm_budget / cell_bytes).clamp(active_experts, 4096);
    ExpertCellPlan {
        cell_bytes,
        active_experts,
        warm_pool_experts,
    }
}

fn clamp_power_of_two(value: usize, min: usize, max: usize) -> usize {
    let value = value.clamp(min, max);
    let lower = value.next_power_of_two() >> usize::from(!value.is_power_of_two());
    let upper = value.next_power_of_two();
    let rounded = if value - lower <= upper.saturating_sub(value) {
        lower
    } else {
        upper
    };
    rounded.clamp(min, max)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptMix {
    Empty,
    Ascii,
    Cyrillic,
    Mixed,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicroFeatures {
    pub original_len: u16,
    pub candidate_len: u16,
    pub original_script: ScriptMix,
    pub candidate_script: ScriptMix,
    pub edit_distance: f32,
    pub whitespace_preserved: bool,
    pub source_is_layout: bool,
    pub source_is_personal: bool,
    pub cli_like: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroAction {
    Keep,
    LayoutRuToEn,
    LayoutEnToRu,
    TypoFix,
    SplitGlue,
    Protect,
}

#[derive(Debug, Clone)]
pub struct MicroContext {
    pub original_tail: String,
    pub app_class: Option<String>,
    pub previous_tokens: Vec<String>,
    pub current_engine_signals: Vec<String>,
}

impl MicroContext {
    pub fn new(original_tail: &str) -> Self {
        Self {
            original_tail: original_tail.to_string(),
            app_class: None,
            previous_tokens: previous_tokens(original_tail),
            current_engine_signals: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CorrectionCandidate {
    pub action: MicroAction,
    pub text: String,
    pub source: String,
    pub engine_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedCandidate {
    pub action: MicroAction,
    pub text: String,
    pub source: &'static str,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MicroScore {
    pub expert: &'static str,
    pub confidence: f32,
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshBoard {
    pub layout_energy: f32,
    pub morphology_energy: f32,
    pub space_energy: f32,
    pub technical_risk: f32,
    pub undo_risk: f32,
    pub coherence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshTickTrace {
    pub tick: u8,
    pub confidence: f32,
    pub board: MeshBoard,
    pub reason_code: &'static str,
}

pub trait MicroExpert: Sync {
    fn name(&self) -> &'static str;
    fn profile(&self) -> Expert64Profile;
    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore;
}

pub trait MicroWriter: Sync {
    fn name(&self) -> &'static str;
    fn profile(&self) -> Expert64Profile;
    fn write_candidates(&self, ctx: &MicroContext) -> Vec<GeneratedCandidate>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MicroCandidateTrace {
    pub candidate: String,
    pub source: String,
    pub action: MicroAction,
    pub engine_score: Option<f64>,
    pub confidence: f32,
    pub expert_scores: Vec<MicroScore>,
    pub mesh_ticks: Vec<MeshTickTrace>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MicroDecisionTrace {
    pub chosen: Option<String>,
    pub candidates: Vec<MicroCandidateTrace>,
    pub generated: Vec<GeneratedCandidate>,
    pub disabled_experts: Vec<String>,
    pub no_raw_secret_text: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MicrobrainOptions {
    disabled_experts: Vec<String>,
    trained_layout_signal: Option<Expert64Cell>,
}

impl MicrobrainOptions {
    pub fn with_disabled(disabled: &[String]) -> Self {
        Self {
            disabled_experts: disabled.iter().map(|id| id.to_ascii_lowercase()).collect(),
            trained_layout_signal: None,
        }
    }

    pub fn with_enabled_only(enabled: &[String]) -> Self {
        let enabled: Vec<String> = enabled.iter().map(|id| id.to_ascii_lowercase()).collect();
        let disabled_experts = default_expert_names()
            .into_iter()
            .chain(default_writer_names())
            .filter(|name| !enabled.iter().any(|enabled| enabled == name))
            .map(str::to_string)
            .collect();
        Self {
            disabled_experts,
            trained_layout_signal: None,
        }
    }

    pub fn with_trained_layout_signal(mut self, cell: Expert64Cell) -> Self {
        self.trained_layout_signal = Some(cell);
        self
    }

    fn expert_enabled(&self, name: &str) -> bool {
        !self
            .disabled_experts
            .iter()
            .any(|disabled| disabled == name)
    }

    pub fn mesh_enabled(&self) -> bool {
        self.expert_enabled("sentence_mesh_64k_stub")
    }

    fn trained_layout_signal(&self) -> Option<&Expert64Cell> {
        self.trained_layout_signal.as_ref()
    }
}

pub fn default_expert_names() -> Vec<&'static str> {
    default_experts()
        .iter()
        .map(|expert| expert.name())
        .collect()
}

pub fn default_expert_profiles() -> Vec<Expert64Profile> {
    default_experts()
        .iter()
        .map(|expert| expert.profile())
        .collect()
}

pub fn default_writer_names() -> Vec<&'static str> {
    default_writers()
        .iter()
        .map(|writer| writer.name())
        .collect()
}

pub fn nanda_status_text() -> String {
    let profiles = default_expert_profiles();
    let writers = default_writer_names();
    let trained_path = default_trained_layout_signal_path();
    let trained_loaded = trained_path.exists();
    let profile = crate::nanda_profile::NandaCpuProfile::detect();
    let mut lines = Vec::new();
    lines.push(format!(
        "NANDA: {} клеток в цепочке ({} оценка/защита/сетка + {} генератор)",
        profiles.len() + writers.len(),
        profiles.len(),
        writers.len()
    ));
    lines.push(profile.compact_text());
    lines.push(format!(
        "Обученная клетка: {} ({})",
        if trained_loaded {
            "загружена"
        } else {
            "нейтральная заглушка"
        },
        expert_display_name(TRAINED_LAYOUT_SIGNAL_EXPERT_ID)
    ));
    lines.push("Роли:".to_string());
    for writer in writers {
        lines.push(format!(
            "  генератор раскладки: {}",
            expert_display_name(writer)
        ));
    }
    for expert in profiles {
        lines.push(format!(
            "  {}: {} [{}]",
            expert_domain_label(expert.domain),
            expert_display_name(expert.expert_id),
            expert_weight_label(expert.weight_format)
        ));
    }
    lines.join("\n")
}

fn expert_domain_label(domain: ExpertDomain) -> &'static str {
    match domain {
        ExpertDomain::Layout => "оценка раскладки",
        ExpertDomain::Typo => "оценка опечаток",
        ExpertDomain::Guard => "защита",
        ExpertDomain::User => "память пользователя",
        ExpertDomain::App => "контекст приложения",
        ExpertDomain::Language => "контекст фразы",
        ExpertDomain::Mesh => "сетка согласования",
    }
}

fn expert_weight_label(format: ExpertWeightFormat) -> &'static str {
    match format {
        ExpertWeightFormat::Heuristic => "правила",
        ExpertWeightFormat::Int8 => "int8",
        ExpertWeightFormat::F16 => "f16",
        ExpertWeightFormat::F32 => "f32",
    }
}

fn expert_display_name(expert_id: &str) -> &'static str {
    match expert_id {
        "layout_writer_64k_stub" => "генератор вариантов раскладки",
        "layout_signal_16k_stub" => "сигнал раскладки",
        "protected_token_16k_stub" => "защита технических слов",
        "cli_guard_16k_stub" => "защита командной строки",
        "context_tail_32k_stub" => "контекст хвоста фразы",
        "user_memory_64k_stub" => "память пользователя",
        TRAINED_LAYOUT_SIGNAL_EXPERT_ID => "обученный сигнал раскладки",
        "sentence_mesh_64k_stub" => "сетка согласования фразы",
        _ => "неизвестная клетка",
    }
}

pub fn default_trained_layout_signal_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".cache")
        .join("lay")
        .join("nanda")
        .join("layout_signal_64k.ex64")
}

pub fn train_expert64_layout_signal(
    rows: &[Expert64TrainingRow],
    epochs: usize,
) -> (Expert64Cell, Expert64TrainingReport) {
    let mut weights = vec![0i16; EXPERT64_WEIGHT_BYTES];
    for _ in 0..epochs {
        let mut start = 0usize;
        while start < rows.len() {
            let group_id = &rows[start].group_id;
            let mut end = start + 1;
            while end < rows.len() && rows[end].group_id == *group_id {
                end += 1;
            }
            let group = &rows[start..end];
            if let Some((positive, negative)) = hardest_group_pair(&weights, group) {
                if trained_score_i16(&weights, &row_training_features(positive))
                    <= trained_score_i16(&weights, &row_training_features(negative)) + 0.04
                {
                    update_weights(&mut weights, &row_training_features(positive), 1);
                    update_weights(&mut weights, &row_training_features(negative), -1);
                }
            }
            start = end;
        }
    }
    let cell = Expert64Cell {
        expert_id: TRAINED_LAYOUT_SIGNAL_EXPERT_ID.to_string(),
        version: EXPERT64_VERSION,
        schema_hash: EXPERT64_SCHEMA_HASH,
        weights: weights.into_iter().map(|weight| weight as i8).collect(),
    };
    let report = evaluate_expert64_layout_signal(&cell, rows, epochs);
    (cell, report)
}

pub fn evaluate_expert64_layout_signal(
    cell: &Expert64Cell,
    rows: &[Expert64TrainingRow],
    epochs: usize,
) -> Expert64TrainingReport {
    let mut correct = 0usize;
    let mut positive_total = 0usize;
    let mut positive_correct = 0usize;
    let mut negative_total = 0usize;
    let mut negative_correct = 0usize;
    let mut groups = 0usize;
    let mut group_correct = 0usize;
    for row in rows {
        let score = cell.score(&row_training_features(row));
        let predicted = score >= 0.5;
        if predicted == row.label {
            correct += 1;
        }
        if row.label {
            positive_total += 1;
            positive_correct += usize::from(predicted);
        } else {
            negative_total += 1;
            negative_correct += usize::from(!predicted);
        }
    }
    let mut start = 0usize;
    while start < rows.len() {
        let group_id = &rows[start].group_id;
        let mut end = start + 1;
        while end < rows.len() && rows[end].group_id == *group_id {
            end += 1;
        }
        groups += 1;
        let group = &rows[start..end];
        let best = group.iter().max_by(|left, right| {
            cell.score(&row_training_features(left))
                .total_cmp(&cell.score(&row_training_features(right)))
        });
        if best.is_some_and(|row| row.label) {
            group_correct += 1;
        }
        start = end;
    }
    Expert64TrainingReport {
        rows: rows.len(),
        groups,
        epochs,
        accuracy: ratio(correct, rows.len()),
        group_accuracy: ratio(group_correct, groups),
        positive_accuracy: ratio(positive_correct, positive_total),
        negative_accuracy: ratio(negative_correct, negative_total),
    }
}

fn hardest_group_pair<'a>(
    weights: &[i16],
    group: &'a [Expert64TrainingRow],
) -> Option<(&'a Expert64TrainingRow, &'a Expert64TrainingRow)> {
    let positive = group.iter().filter(|row| row.label).max_by(|left, right| {
        trained_score_i16(weights, &row_training_features(left))
            .total_cmp(&trained_score_i16(weights, &row_training_features(right)))
    })?;
    let negative = group
        .iter()
        .filter(|row| !row.label)
        .max_by(|left, right| {
            trained_score_i16(weights, &row_training_features(left))
                .total_cmp(&trained_score_i16(weights, &row_training_features(right)))
        })?;
    Some((positive, negative))
}

fn update_weights(weights: &mut [i16], features: &[String], delta: i16) {
    for feature in features {
        let idx = feature_index(feature, weights.len());
        weights[idx] = (weights[idx] + delta).clamp(-127, 127);
    }
}

pub fn expert64_pool_bytes(expert_count: usize) -> usize {
    expert_count.saturating_mul(EXPERT64_STATE_BYTES)
}

pub fn expert_pool_bytes(expert_count: usize, cell_bytes: usize) -> usize {
    expert_count.saturating_mul(cell_bytes)
}

pub fn extract_features(ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroFeatures {
    MicroFeatures {
        original_len: bounded_len(&ctx.original_tail),
        candidate_len: bounded_len(&candidate.text),
        original_script: script_mix(&ctx.original_tail),
        candidate_script: script_mix(&candidate.text),
        edit_distance: normalized_edit_distance(&ctx.original_tail, &candidate.text) as f32,
        whitespace_preserved: ctx.original_tail.ends_with(char::is_whitespace)
            == candidate.text.ends_with(char::is_whitespace),
        source_is_layout: candidate.source.contains("layout"),
        source_is_personal: candidate.source.contains("personal")
            || candidate.source.contains("exact"),
        cli_like: contains_cli_flag(&ctx.original_tail)
            || ctx
                .previous_tokens
                .iter()
                .any(|token| matches!(token.as_str(), "git" | "cd" | "ssh" | "sudo")),
    }
}

pub fn decide(
    ctx: &MicroContext,
    candidates: &[CorrectionCandidate],
    options: &MicrobrainOptions,
) -> MicroDecisionTrace {
    let mut traces = Vec::new();

    for candidate in candidates {
        let expert_scores: Vec<MicroScore> = default_experts()
            .iter()
            .filter(|expert| options.expert_enabled(expert.name()))
            .map(|expert| score_expert_with_options(*expert, ctx, candidate, options))
            .collect();
        let base_confidence = consensus_confidence(&expert_scores);
        let (confidence, mesh_ticks) = if options.mesh_enabled() {
            run_mesh_relaxation(ctx, candidate, base_confidence)
        } else {
            (base_confidence, Vec::new())
        };
        traces.push(MicroCandidateTrace {
            candidate: candidate.text.clone(),
            source: candidate.source.clone(),
            action: candidate.action,
            engine_score: candidate.engine_score,
            confidence,
            expert_scores,
            mesh_ticks,
        });
    }

    let chosen = choose_trace(&traces).map(|trace| trace.candidate.clone());
    MicroDecisionTrace {
        chosen,
        candidates: traces,
        generated: Vec::new(),
        disabled_experts: options.disabled_experts.clone(),
        no_raw_secret_text: true,
    }
}

fn score_expert_with_options(
    expert: &dyn MicroExpert,
    ctx: &MicroContext,
    candidate: &CorrectionCandidate,
    options: &MicrobrainOptions,
) -> MicroScore {
    if expert.name() == TRAINED_LAYOUT_SIGNAL_EXPERT_ID {
        if let Some(cell) = options.trained_layout_signal() {
            return trained_layout_signal_score(ctx, candidate, cell);
        }
    }
    expert.score(ctx, candidate)
}

pub fn generate_candidates(
    ctx: &MicroContext,
    options: &MicrobrainOptions,
) -> Vec<GeneratedCandidate> {
    let mut generated = Vec::new();
    for writer in default_writers()
        .iter()
        .filter(|writer| options.expert_enabled(writer.name()))
    {
        generated.extend(writer.write_candidates(ctx));
    }
    dedup_generated(generated)
}

pub fn decide_with_generated(
    ctx: &MicroContext,
    candidates: &[CorrectionCandidate],
    generated: Vec<GeneratedCandidate>,
    options: &MicrobrainOptions,
) -> MicroDecisionTrace {
    let mut all_candidates = candidates.to_vec();
    all_candidates.extend(generated.iter().map(|candidate| CorrectionCandidate {
        action: candidate.action,
        text: candidate.text.clone(),
        source: candidate.source.to_string(),
        engine_score: None,
    }));
    let mut trace = decide(ctx, &all_candidates, options);
    trace.generated = generated;
    trace
}

pub fn accepts_candidate(
    original_tail: &str,
    candidate_text: &str,
    action: MicroAction,
    source: &str,
) -> bool {
    let candidate = CorrectionCandidate {
        action,
        text: candidate_text.to_string(),
        source: source.to_string(),
        engine_score: None,
    };
    let trace = decide(
        &MicroContext::new(original_tail),
        &[candidate],
        &MicrobrainOptions::default(),
    );
    trace.chosen.as_deref() == Some(candidate_text)
}

fn choose_trace(traces: &[MicroCandidateTrace]) -> Option<&MicroCandidateTrace> {
    let mut sorted: Vec<&MicroCandidateTrace> = traces
        .iter()
        .filter(|trace| trace.confidence >= 0.55)
        .collect();
    sorted.sort_by(|left, right| {
        right
            .confidence
            .total_cmp(&left.confidence)
            .then_with(|| {
                right
                    .engine_score
                    .unwrap_or(f64::NEG_INFINITY)
                    .total_cmp(&left.engine_score.unwrap_or(f64::NEG_INFINITY))
            })
            .then_with(|| left.source.cmp(&right.source))
    });
    let best = *sorted.first()?;
    if let Some(second) = sorted
        .iter()
        .find(|trace| trace.candidate != best.candidate)
    {
        if best.confidence - second.confidence < 0.02 {
            if safe_engine_score_tiebreak(best, second) {
                return Some(best);
            }
            return None;
        }
    }
    Some(best)
}

fn safe_engine_score_tiebreak(best: &MicroCandidateTrace, second: &MicroCandidateTrace) -> bool {
    if !matches!(best.action, MicroAction::TypoFix)
        || !matches!(second.action, MicroAction::TypoFix)
    {
        return false;
    }
    if best.source.contains("extra")
        || best.source.contains("cleanup")
        || best.source.contains("split")
        || best.source.contains("glued")
        || best.source.contains("prefix")
    {
        return false;
    }
    let Some(best_score) = best.engine_score else {
        return false;
    };
    let Some(second_score) = second.engine_score else {
        return false;
    };
    if best_score - second_score < 0.8 {
        return false;
    }
    best.candidate
        .split_whitespace()
        .last()
        .is_some_and(is_probably_completed_natural_word)
}

fn consensus_confidence(scores: &[MicroScore]) -> f32 {
    if scores.is_empty() {
        return 0.0;
    }
    if scores
        .iter()
        .any(|score| score.confidence <= 0.10 && score.reason_code.ends_with("_veto"))
    {
        return 0.0;
    }
    let min = scores
        .iter()
        .map(|score| score.confidence)
        .fold(1.0f32, f32::min);
    let max = scores
        .iter()
        .map(|score| score.confidence)
        .fold(0.0f32, f32::max);
    if max - min > 0.78 {
        return 0.0;
    }
    scores.iter().map(|score| score.confidence).sum::<f32>() / scores.len() as f32
}

fn run_mesh_relaxation(
    ctx: &MicroContext,
    candidate: &CorrectionCandidate,
    base_confidence: f32,
) -> (f32, Vec<MeshTickTrace>) {
    if base_confidence <= 0.0 {
        return (
            0.0,
            vec![MeshTickTrace {
                tick: 0,
                confidence: 0.0,
                board: mesh_board(ctx, candidate),
                reason_code: "mesh_base_veto",
            }],
        );
    }

    let board = mesh_board(ctx, candidate);
    let mut confidence = base_confidence;
    let mut ticks = Vec::with_capacity(3);
    for tick in 1..=3 {
        let (delta, reason_code) = mesh_delta(&board, candidate);
        confidence = (confidence + delta).clamp(0.0, 1.0);
        ticks.push(MeshTickTrace {
            tick,
            confidence,
            board: board.clone(),
            reason_code,
        });
        if confidence <= 0.0 {
            break;
        }
    }
    (confidence, ticks)
}

fn mesh_board(ctx: &MicroContext, candidate: &CorrectionCandidate) -> MeshBoard {
    let features = extract_features(ctx, candidate);
    let layout_energy = if candidate.source.contains("layout")
        || matches!(
            candidate.action,
            MicroAction::LayoutEnToRu | MicroAction::LayoutRuToEn
        ) {
        if candidate_preserves_sentence_shape(ctx, candidate) {
            0.78
        } else {
            0.52
        }
    } else {
        0.24
    };
    let is_pure_space_repair = strong_space_boundary_repair(&ctx.original_tail, candidate);
    let morphology_energy = if is_pure_space_repair
        || matches!(candidate.action, MicroAction::SplitGlue)
            && preserves_word_boundary_shape(&ctx.original_tail, &candidate.text)
    {
        0.82
    } else if features.edit_distance <= 0.22 {
        0.62
    } else {
        0.42
    };
    let space_energy = if is_pure_space_repair {
        0.86
    } else if features.whitespace_preserved {
        if removes_word_boundary(&ctx.original_tail, &candidate.text) {
            0.0
        } else {
            0.74
        }
    } else {
        0.18
    };
    let technical_token_risk = looks_like_safe_ascii_token(&ctx.original_tail)
        && !matches!(candidate.action, MicroAction::LayoutEnToRu);
    let technical_risk = if features.cli_like || technical_token_risk {
        0.68
    } else {
        0.16
    };
    let undo_risk = if risky_single_prefix_deletion(&ctx.original_tail, candidate) {
        0.95
    } else if features.edit_distance > 0.55 {
        0.58
    } else {
        0.22
    };
    let coherence = ((layout_energy + morphology_energy + space_energy) / 3.0f32
        - (technical_risk + undo_risk) / 4.0f32)
        .clamp(0.0f32, 1.0f32);
    MeshBoard {
        layout_energy,
        morphology_energy,
        space_energy,
        technical_risk,
        undo_risk,
        coherence,
    }
}

fn mesh_delta(board: &MeshBoard, candidate: &CorrectionCandidate) -> (f32, &'static str) {
    if board.undo_risk >= 0.90 {
        return (-0.45, "mesh_undo_risk");
    }
    if board.space_energy <= 0.05 {
        return (-0.35, "mesh_space_veto");
    }
    if board.technical_risk >= 0.65 && !matches!(candidate.action, MicroAction::Protect) {
        return (-0.12, "mesh_technical_guard");
    }
    if board.coherence >= 0.58 {
        return (0.035, "mesh_coherent_peak");
    }
    if board.coherence <= 0.26 {
        return (-0.06, "mesh_incoherent");
    }
    (0.0, "mesh_stable")
}

fn default_experts() -> &'static [&'static dyn MicroExpert] {
    &DEFAULT_EXPERTS
}

struct LayoutSignal16kStub;
struct ProtectedToken16kStub;
struct CliGuard16kStub;
struct ContextTail32kStub;
struct UserMemory64kStub;
struct TrainedLayoutSignal64k;
struct SentenceMesh64kStub;
struct LayoutWriter64kStub;

static LAYOUT_SIGNAL_16K_STUB: LayoutSignal16kStub = LayoutSignal16kStub;
static PROTECTED_TOKEN_16K_STUB: ProtectedToken16kStub = ProtectedToken16kStub;
static CLI_GUARD_16K_STUB: CliGuard16kStub = CliGuard16kStub;
static CONTEXT_TAIL_32K_STUB: ContextTail32kStub = ContextTail32kStub;
static USER_MEMORY_64K_STUB: UserMemory64kStub = UserMemory64kStub;
static TRAINED_LAYOUT_SIGNAL_64K: TrainedLayoutSignal64k = TrainedLayoutSignal64k;
static SENTENCE_MESH_64K_STUB: SentenceMesh64kStub = SentenceMesh64kStub;
static LAYOUT_WRITER_64K_STUB: LayoutWriter64kStub = LayoutWriter64kStub;
static DEFAULT_EXPERTS: [&'static dyn MicroExpert; 7] = [
    &LAYOUT_SIGNAL_16K_STUB,
    &PROTECTED_TOKEN_16K_STUB,
    &CLI_GUARD_16K_STUB,
    &CONTEXT_TAIL_32K_STUB,
    &USER_MEMORY_64K_STUB,
    &TRAINED_LAYOUT_SIGNAL_64K,
    &SENTENCE_MESH_64K_STUB,
];
static DEFAULT_WRITERS: [&'static dyn MicroWriter; 1] = [&LAYOUT_WRITER_64K_STUB];

fn default_writers() -> &'static [&'static dyn MicroWriter] {
    &DEFAULT_WRITERS
}

impl MicroExpert for LayoutSignal16kStub {
    fn name(&self) -> &'static str {
        "layout_signal_16k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Layout)
    }

    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if matches!(
            candidate.action,
            MicroAction::LayoutRuToEn | MicroAction::LayoutEnToRu
        ) && convert(&ctx.original_tail, detect_direction(&ctx.original_tail)) == candidate.text
        {
            return score(self.name(), 0.97, "exact_layout");
        }
        if candidate.source.contains("layout") {
            return score(self.name(), 0.82, "layout_source");
        }
        score(self.name(), 0.50, "neutral")
    }
}

impl MicroExpert for ProtectedToken16kStub {
    fn name(&self) -> &'static str {
        "protected_token_16k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Guard)
    }

    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if matches!(candidate.action, MicroAction::Protect | MicroAction::Keep) {
            return score(self.name(), 0.92, "keep_or_protect");
        }
        if matches!(candidate.action, MicroAction::LayoutEnToRu)
            && has_known_ascii_or_protected_token(&ctx.original_tail)
            && has_cyrillic(&candidate.text)
        {
            if layout_candidate_mutates_known_ascii_token(&ctx.original_tail, &candidate.text) {
                return score(self.name(), 0.0, "known_ascii_token_mutation_veto");
            }
            if layout_candidate_is_short_russian_function(&candidate.text) {
                return score(self.name(), 0.58, "short_ru_function_layout");
            }
            return score(self.name(), 0.0, "known_ascii_layout_veto");
        }
        if !matches!(candidate.action, MicroAction::LayoutEnToRu)
            && looks_like_safe_ascii_token(&ctx.original_tail)
            && has_cyrillic(&candidate.text)
        {
            return score(self.name(), 0.0, "protected_ascii_veto");
        }
        score(self.name(), 0.58, "no_protection_signal")
    }
}

fn layout_candidate_is_short_russian_function(text: &str) -> bool {
    text.split_whitespace().any(|token| {
        let (_, word, _) = split_word_punctuation(token);
        is_short_russian_function_candidate(word)
    })
}

fn layout_candidate_mutates_known_ascii_token(original: &str, candidate: &str) -> bool {
    original
        .split_whitespace()
        .zip(candidate.split_whitespace())
        .any(|(original_token, candidate_token)| {
            let (_, original_word, _) = split_word_punctuation(original_token);
            if !is_known_ascii_like_or_protected_token(original_word) {
                return false;
            }
            let (_, candidate_word, _) = split_word_punctuation(candidate_token);
            if is_short_russian_function_candidate(candidate_word) {
                return false;
            }
            !candidate_word.eq_ignore_ascii_case(original_word)
        })
}

fn is_short_russian_function_candidate(word: &str) -> bool {
    let lower = word.to_lowercase();
    let len = lower.chars().filter(|ch| is_cyrillic_char(*ch)).count();
    !lower.is_empty()
        && len <= 3
        && (is_ru_short_function_word(&lower)
            || is_ru_short_pronoun(&lower)
            || is_common_ru_word(&lower))
}

impl MicroExpert for CliGuard16kStub {
    fn name(&self) -> &'static str {
        "cli_guard_16k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Guard)
    }

    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if contains_cli_flag(&ctx.original_tail) && candidate.text != ctx.original_tail {
            return score(self.name(), 0.0, "cli_flag_veto");
        }
        if ctx
            .previous_tokens
            .iter()
            .any(|token| matches!(token.as_str(), "git" | "cd" | "ssh" | "sudo"))
        {
            return score(self.name(), 0.62, "cli_context");
        }
        score(self.name(), 0.56, "neutral")
    }
}

impl MicroExpert for ContextTail32kStub {
    fn name(&self) -> &'static str {
        "context_tail_32k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Language)
    }

    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if creates_mixed_script_token(&candidate.text) {
            return score(self.name(), 0.0, "mixed_script_veto");
        }
        if strong_space_boundary_repair(&ctx.original_tail, candidate) {
            return score(self.name(), 0.82, "space_boundary_repair");
        }
        if matches!(candidate.action, MicroAction::SplitGlue)
            && removes_word_boundary(&ctx.original_tail, &candidate.text)
        {
            return score(self.name(), 0.0, "boundary_removal_veto");
        }
        if normalized_edit_distance(&ctx.original_tail, &candidate.text) <= 0.34 {
            return score(self.name(), 0.72, "small_edit");
        }
        score(self.name(), 0.52, "weak_context")
    }
}

impl MicroExpert for UserMemory64kStub {
    fn name(&self) -> &'static str {
        "user_memory_64k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::User)
    }

    fn score(&self, _ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if candidate.source.contains("personal") {
            return score(self.name(), 0.96, "personal_rule");
        }
        if candidate.source.contains("exact") {
            return score(self.name(), 0.90, "exact_rule");
        }
        score(self.name(), 0.55, "no_memory_signal")
    }
}

impl MicroExpert for TrainedLayoutSignal64k {
    fn name(&self) -> &'static str {
        TRAINED_LAYOUT_SIGNAL_EXPERT_ID
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Layout)
    }

    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if !matches!(
            candidate.action,
            MicroAction::LayoutRuToEn | MicroAction::LayoutEnToRu
        ) && !candidate.source.contains("layout")
        {
            return score(self.name(), 0.56, "trained_layout_out_of_domain");
        }
        let cell = trained_layout_signal_cell();
        trained_layout_signal_score(ctx, candidate, cell)
    }
}

fn trained_layout_signal_score(
    ctx: &MicroContext,
    candidate: &CorrectionCandidate,
    cell: &Expert64Cell,
) -> MicroScore {
    if !matches!(
        candidate.action,
        MicroAction::LayoutRuToEn | MicroAction::LayoutEnToRu
    ) && !candidate.source.contains("layout")
    {
        return score(
            TRAINED_LAYOUT_SIGNAL_EXPERT_ID,
            0.56,
            "trained_layout_out_of_domain",
        );
    }
    if cell.weights.iter().all(|weight| *weight == 0) {
        return score(
            TRAINED_LAYOUT_SIGNAL_EXPERT_ID,
            0.56,
            "trained_layout_no_packet",
        );
    }
    let raw = cell.score(&candidate_features(ctx, candidate));
    let confidence = 0.42 + raw * 0.38;
    score(
        TRAINED_LAYOUT_SIGNAL_EXPERT_ID,
        confidence,
        "trained_layout_signal",
    )
}

fn trained_layout_signal_cell() -> &'static Expert64Cell {
    static CELL: OnceLock<Expert64Cell> = OnceLock::new();
    CELL.get_or_init(|| {
        std::env::var_os("LAY_NANDA_LAYOUT_SIGNAL_64K")
            .map(PathBuf::from)
            .or_else(|| Some(default_trained_layout_signal_path()))
            .and_then(|path| Expert64Cell::read(path).ok())
            .filter(|cell| {
                cell.expert_id == TRAINED_LAYOUT_SIGNAL_EXPERT_ID
                    && cell.version == EXPERT64_VERSION
                    && cell.schema_hash == EXPERT64_SCHEMA_HASH
                    && cell.weights.len() == EXPERT64_WEIGHT_BYTES
            })
            .unwrap_or_else(|| Expert64Cell::neutral(TRAINED_LAYOUT_SIGNAL_EXPERT_ID))
    })
}

impl MicroExpert for SentenceMesh64kStub {
    fn name(&self) -> &'static str {
        "sentence_mesh_64k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Mesh)
    }

    fn score(&self, ctx: &MicroContext, candidate: &CorrectionCandidate) -> MicroScore {
        if creates_mixed_script_token(&candidate.text) {
            return score(self.name(), 0.0, "mesh_mixed_script_veto");
        }
        if strong_space_boundary_repair(&ctx.original_tail, candidate) {
            return score(self.name(), 0.88, "mesh_space_boundary_repair");
        }
        if matches!(candidate.action, MicroAction::SplitGlue)
            && preserves_word_boundary_shape(&ctx.original_tail, &candidate.text)
        {
            return score(self.name(), 0.88, "mesh_phrase_repair");
        }
        if removes_word_boundary(&ctx.original_tail, &candidate.text) {
            return score(self.name(), 0.0, "mesh_boundary_removal_veto");
        }
        if risky_single_prefix_deletion(&ctx.original_tail, candidate) {
            return score(self.name(), 0.0, "mesh_detached_prefix_veto");
        }
        if candidate.source.contains("layout") && candidate_preserves_sentence_shape(ctx, candidate)
        {
            return score(self.name(), 0.76, "mesh_layout_sentence_shape");
        }
        if normalized_edit_distance(&ctx.original_tail, &candidate.text) <= 0.22 {
            return score(self.name(), 0.72, "mesh_small_phrase_edit");
        }
        score(self.name(), 0.57, "mesh_neutral")
    }
}

impl MicroWriter for LayoutWriter64kStub {
    fn name(&self) -> &'static str {
        "layout_writer_64k_stub"
    }

    fn profile(&self) -> Expert64Profile {
        Expert64Profile::heuristic(self.name(), ExpertDomain::Layout)
    }

    fn write_candidates(&self, ctx: &MicroContext) -> Vec<GeneratedCandidate> {
        let original = ctx.original_tail.as_str();
        if original.trim().is_empty()
            || contains_cli_flag(original)
            || original.split_whitespace().count() != 1
        {
            return Vec::new();
        }
        let converted = convert(original, detect_direction(original));
        if converted == original || creates_mixed_script_token(&converted) {
            return Vec::new();
        }
        let action = if has_cyrillic(original) && converted.is_ascii() {
            let (_, original_word, _) = split_word_punctuation(original);
            let (_, converted_word, _) = split_word_punctuation(&converted);
            if looks_like_cyrillic_acronym(original_word) {
                return Vec::new();
            }
            if recognize_token(original_word).is_known_russian_plain_word() {
                return Vec::new();
            }
            if converted_word.chars().count() <= 2 {
                return Vec::new();
            }
            let converted_identity = recognize_token(converted_word);
            if !converted_identity.is_known_ascii_or_protected_token() {
                return Vec::new();
            }
            MicroAction::LayoutRuToEn
        } else if original.is_ascii() && has_cyrillic(&converted) {
            let (_, original_word, _) = split_word_punctuation(original);
            if is_known_ascii_like_or_protected_token(original_word) {
                return Vec::new();
            }
            let (_, converted_word, _) = split_word_punctuation(&converted);
            let long_exact_layout_word = converted_word.chars().count() >= 7;
            if !long_exact_layout_word
                && !is_known_russian_word_or_form(&converted_word.to_lowercase())
            {
                return Vec::new();
            }
            MicroAction::LayoutEnToRu
        } else {
            return Vec::new();
        };
        vec![GeneratedCandidate {
            action,
            text: converted,
            source: match action {
                MicroAction::LayoutRuToEn => "nanda_writer_layout_ru_to_en_64k",
                MicroAction::LayoutEnToRu => "nanda_writer_layout_en_to_ru_64k",
                _ => "nanda_writer_layout_64k",
            },
            reason_code: "keyboard_layout_candidate",
        }]
    }
}

fn score(expert: &'static str, confidence: f32, reason_code: &'static str) -> MicroScore {
    MicroScore {
        expert,
        confidence,
        reason_code,
    }
}

fn previous_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .rev()
        .skip(1)
        .take(4)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn contains_cli_flag(text: &str) -> bool {
    text.split_whitespace()
        .any(|token| token.starts_with('-') && token.len() > 1)
}

fn looks_like_safe_ascii_token(text: &str) -> bool {
    let token = text.trim();
    token.len() <= 4 && token.is_ascii() && token.chars().any(|ch| ch.is_ascii_alphabetic())
}

fn has_known_ascii_or_protected_token(text: &str) -> bool {
    text.split_whitespace().any(|token| {
        let (_, word, _) = split_word_punctuation(token);
        is_known_ascii_like_or_protected_token(word)
    })
}

fn is_known_ascii_like_or_protected_token(token: &str) -> bool {
    let identity = recognize_token(token);
    identity.core.is_ascii()
        && identity.core.chars().any(|ch| ch.is_ascii_alphabetic())
        && (identity.known_en || identity.technical || identity.protected)
}

fn creates_mixed_script_token(text: &str) -> bool {
    text.split_whitespace()
        .any(|token| has_cyrillic(token) && has_latin(token))
}

fn looks_like_cyrillic_acronym(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|ch| is_cyrillic_char(*ch)).collect();
    (2..=6).contains(&letters.len())
        && letters.len() == word.chars().count()
        && letters.iter().all(|ch| ch.is_uppercase())
}

fn candidate_features(ctx: &MicroContext, candidate: &CorrectionCandidate) -> Vec<String> {
    let mut features = Vec::new();
    push_common_features(
        &mut features,
        &ctx.original_tail,
        &candidate.text,
        &candidate.source,
        action_name(candidate.action),
    );
    features
}

fn row_training_features(row: &Expert64TrainingRow) -> Vec<String> {
    let mut features = Vec::new();
    push_common_features(
        &mut features,
        &row.original,
        &row.candidate,
        &row.operation,
        &row.operation,
    );
    features
}

fn push_common_features(
    features: &mut Vec<String>,
    original: &str,
    candidate: &str,
    source: &str,
    action: &str,
) {
    features.push("bias".to_string());
    features.push(format!("op:{action}"));
    features.push(format!("source:{}", coarse_source(source)));
    features.push(format!("source_id:{source}"));
    features.push(format!("orig_script:{:?}", script_mix(original)));
    features.push(format!("cand_script:{:?}", script_mix(candidate)));
    push_text_profile(features, "orig", original);
    push_text_profile(features, "cand", candidate);
    push_relation_profile(features, original, candidate);
    features.push(format!(
        "len_delta:{}",
        bucket_i32(candidate.chars().count() as i32 - original.chars().count() as i32)
    ));
    features.push(format!(
        "edit:{}",
        bucket_f32(normalized_edit_distance(original, candidate) as f32)
    ));
    features.push(format!(
        "words:{}->{}",
        original.split_whitespace().count().min(4),
        candidate.split_whitespace().count().min(4)
    ));
    features.push(format!(
        "space:{}",
        original.ends_with(char::is_whitespace) == candidate.ends_with(char::is_whitespace)
    ));
    features.push(format!(
        "compact:{}",
        without_whitespace(original) == without_whitespace(candidate)
    ));
    features.push(format!(
        "edit_span:{}",
        bucket_i32(common_replacement_span(original, candidate) as i32)
    ));
    features.push(format!(
        "layout_exact:{}",
        convert(original, detect_direction(original)) == candidate
    ));
    features.push(format!(
        "reverse_layout_exact:{}",
        convert(candidate, detect_direction(candidate)) == original
    ));
    features.push(format!(
        "layout_edit:{}",
        bucket_f32(normalized_edit_distance(
            &convert(original, detect_direction(original)),
            candidate
        ) as f32)
    ));
    if has_cyrillic(original) && candidate.is_ascii() {
        features.push("dir:cyr_to_ascii".to_string());
    } else if original.is_ascii() && has_cyrillic(candidate) {
        features.push("dir:ascii_to_cyr".to_string());
    }
    for token in original.split_whitespace().take(4) {
        features.push(format!("orig_tok:{}", token_shape(token)));
    }
    for token in candidate.split_whitespace().take(4) {
        features.push(format!("cand_tok:{}", token_shape(token)));
    }
    for gram in char_ngrams(original, 3).into_iter().take(64) {
        features.push(format!("orig_ng:{gram}"));
    }
    for gram in char_ngrams(candidate, 3).into_iter().take(64) {
        features.push(format!("cand_ng:{gram}"));
    }
    for gram in char_ngrams(candidate, 3).into_iter().take(64) {
        features.push(format!("delta_plus:{gram}"));
    }
    for gram in char_ngrams(original, 3).into_iter().take(64) {
        features.push(format!("delta_minus:{gram}"));
    }
}

fn push_text_profile(features: &mut Vec<String>, prefix: &str, text: &str) {
    let chars: Vec<char> = text.chars().collect();
    let tokens: Vec<&str> = text.split_whitespace().collect();
    features.push(format!("{prefix}:chars:{}", bucket_i32(chars.len() as i32)));
    features.push(format!("{prefix}:tokens:{}", tokens.len().min(6)));
    features.push(format!(
        "{prefix}:has_space:{}",
        text.contains(char::is_whitespace)
    ));
    features.push(format!(
        "{prefix}:leading_space:{}",
        text.starts_with(char::is_whitespace)
    ));
    features.push(format!(
        "{prefix}:trailing_space:{}",
        text.ends_with(char::is_whitespace)
    ));
    features.push(format!(
        "{prefix}:upper:{}",
        bucket_i32(count_upper(text) as i32)
    ));
    features.push(format!(
        "{prefix}:digits:{}",
        bucket_i32(count_digits(text) as i32)
    ));
    features.push(format!(
        "{prefix}:punct:{}",
        bucket_i32(count_punctuation(text) as i32)
    ));

    let mut known_ru = 0usize;
    let mut known_en = 0usize;
    let mut protected = 0usize;
    let mut technical = 0usize;
    let mut cli = 0usize;
    let mut natural = 0usize;
    let mut script_seq = String::new();
    let mut language_seq = String::new();
    let mut kind_seq = String::new();

    for token in tokens.iter().take(6) {
        let identity = recognize_token(token);
        known_ru += usize::from(identity.known_ru);
        known_en += usize::from(identity.known_en);
        protected += usize::from(identity.protected);
        technical += usize::from(identity.technical);
        cli += usize::from(matches!(identity.kind, WordKind::CliOption));
        natural += usize::from(is_probably_completed_natural_word(token));
        push_short_code(
            &mut script_seq,
            format!("{:?}", identity.script).chars().next(),
        );
        push_short_code(
            &mut language_seq,
            format!("{:?}", identity.language).chars().next(),
        );
        push_short_code(&mut kind_seq, format!("{:?}", identity.kind).chars().next());
    }

    features.push(format!("{prefix}:known_ru:{}", known_ru.min(6)));
    features.push(format!("{prefix}:known_en:{}", known_en.min(6)));
    features.push(format!("{prefix}:protected:{}", protected.min(6)));
    features.push(format!("{prefix}:technical:{}", technical.min(6)));
    features.push(format!("{prefix}:cli:{}", cli.min(6)));
    features.push(format!("{prefix}:natural:{}", natural.min(6)));
    features.push(format!("{prefix}:script_seq:{script_seq}"));
    features.push(format!("{prefix}:lang_seq:{language_seq}"));
    features.push(format!("{prefix}:kind_seq:{kind_seq}"));

    if let Some(first) = tokens.first() {
        features.push(format!("{prefix}:first_shape:{}", token_shape(first)));
    }
    if let Some(last) = tokens.last() {
        features.push(format!("{prefix}:last_shape:{}", token_shape(last)));
        let (_, core, _) = split_word_punctuation(last);
        features.push(format!(
            "{prefix}:last_core_len:{}",
            bucket_i32(core.chars().count() as i32)
        ));
    }
}

fn push_relation_profile(features: &mut Vec<String>, original: &str, candidate: &str) {
    let original_tokens: Vec<&str> = original.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    features.push(format!(
        "token_delta:{}",
        bucket_i32(candidate_tokens.len() as i32 - original_tokens.len() as i32)
    ));
    features.push(format!(
        "space_count_delta:{}",
        bucket_i32(count_spaces(candidate) as i32 - count_spaces(original) as i32)
    ));
    features.push(format!(
        "punct_delta:{}",
        bucket_i32(count_punctuation(candidate) as i32 - count_punctuation(original) as i32)
    ));
    features.push(format!(
        "upper_delta:{}",
        bucket_i32(count_upper(candidate) as i32 - count_upper(original) as i32)
    ));
    features.push(format!(
        "digit_delta:{}",
        bucket_i32(count_digits(candidate) as i32 - count_digits(original) as i32)
    ));
    features.push(format!(
        "same_first_char:{}",
        first_char(original) == first_char(candidate)
    ));
    features.push(format!(
        "same_last_char:{}",
        last_non_space_char(original) == last_non_space_char(candidate)
    ));
    features.push(format!(
        "last_token_risky_layout:{}",
        last_core_pair(original, candidate)
            .is_some_and(|(left, right)| is_plain_layout_autocorrect_risky(left, right))
    ));
}

fn char_ngrams(text: &str, max_n: usize) -> Vec<String> {
    let chars: Vec<char> = format!("^{text}$").chars().collect();
    let mut out = Vec::new();
    for n in 1..=max_n {
        if chars.len() < n {
            continue;
        }
        for window in chars.windows(n) {
            out.push(window.iter().collect());
        }
    }
    out
}

fn action_name(action: MicroAction) -> &'static str {
    match action {
        MicroAction::Keep => "keep",
        MicroAction::LayoutRuToEn | MicroAction::LayoutEnToRu => "layout",
        MicroAction::TypoFix => "typo",
        MicroAction::SplitGlue => "split_glue",
        MicroAction::Protect => "protect",
    }
}

fn coarse_source(source: &str) -> &'static str {
    if source.contains("layout") {
        "layout"
    } else if source.contains("split") || source.contains("glued") || source.contains("prefix") {
        "space"
    } else if source.contains("exact") || source.contains("personal") {
        "exact"
    } else if source.contains("protect") {
        "protect"
    } else {
        "other"
    }
}

fn token_shape(token: &str) -> String {
    let mut out = String::new();
    for ch in token.chars().take(12) {
        let kind = if is_cyrillic_char(ch) {
            if ch.is_uppercase() {
                'Ж'
            } else {
                'ж'
            }
        } else if ch.is_ascii_alphabetic() {
            if ch.is_ascii_uppercase() {
                'A'
            } else {
                'a'
            }
        } else if ch.is_ascii_digit() {
            '0'
        } else if ch.is_whitespace() {
            '_'
        } else {
            '.'
        };
        if !out.ends_with(kind) {
            out.push(kind);
        }
    }
    out
}

fn push_short_code(out: &mut String, code: Option<char>) {
    let Some(code) = code else {
        return;
    };
    if !out.ends_with(code) {
        out.push(code);
    }
}

fn count_upper(text: &str) -> usize {
    text.chars().filter(|ch| ch.is_uppercase()).count()
}

fn count_digits(text: &str) -> usize {
    text.chars().filter(|ch| ch.is_ascii_digit()).count()
}

fn count_spaces(text: &str) -> usize {
    text.chars().filter(|ch| ch.is_whitespace()).count()
}

fn count_punctuation(text: &str) -> usize {
    text.chars()
        .filter(|ch| !ch.is_alphanumeric() && !ch.is_whitespace())
        .count()
}

fn first_char(text: &str) -> Option<char> {
    text.chars().next()
}

fn last_non_space_char(text: &str) -> Option<char> {
    text.chars().rev().find(|ch| !ch.is_whitespace())
}

fn last_core_pair<'a>(left: &'a str, right: &'a str) -> Option<(&'a str, &'a str)> {
    let left_token = left.split_whitespace().last()?;
    let right_token = right.split_whitespace().last()?;
    let (_, left_core, _) = split_word_punctuation(left_token);
    let (_, right_core, _) = split_word_punctuation(right_token);
    if left_core.is_empty() || right_core.is_empty() {
        None
    } else {
        Some((left_core, right_core))
    }
}

fn bucket_i32(value: i32) -> i32 {
    value.clamp(-8, 8)
}

fn bucket_f32(value: f32) -> i32 {
    (value * 10.0).round().clamp(0.0, 10.0) as i32
}

fn trained_score_i16(weights: &[i16], features: &[String]) -> f32 {
    if weights.is_empty() {
        return 0.5;
    }
    let sum: i32 = features
        .iter()
        .map(|feature| {
            let idx = feature_index(feature, weights.len());
            weights[idx] as i32
        })
        .sum();
    sigmoid(sum as f32 / TRAINED_SCORE_SCALE)
}

fn feature_index(feature: &str, modulo: usize) -> usize {
    if modulo == 0 {
        return 0;
    }
    (fnv1a64(feature.as_bytes()) as usize) % modulo
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value.clamp(-32.0, 32.0)).exp())
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn dedup_generated(generated: Vec<GeneratedCandidate>) -> Vec<GeneratedCandidate> {
    let mut deduped = Vec::new();
    for candidate in generated {
        if !deduped
            .iter()
            .any(|existing: &GeneratedCandidate| existing.text == candidate.text)
        {
            deduped.push(candidate);
        }
    }
    deduped
}

fn removes_word_boundary(original: &str, candidate: &str) -> bool {
    original.split_whitespace().count() > candidate.split_whitespace().count()
}

fn strong_space_boundary_repair(original: &str, candidate: &CorrectionCandidate) -> bool {
    matches!(candidate.action, MicroAction::SplitGlue)
        && candidate.source == ids::SPLIT_WORD_PAIR
        && without_whitespace(original) == without_whitespace(&candidate.text)
}

fn preserves_word_boundary_shape(original: &str, candidate: &str) -> bool {
    original.split_whitespace().count() == candidate.split_whitespace().count()
        && original.ends_with(char::is_whitespace) == candidate.ends_with(char::is_whitespace)
}

fn candidate_preserves_sentence_shape(ctx: &MicroContext, candidate: &CorrectionCandidate) -> bool {
    preserves_word_boundary_shape(&ctx.original_tail, &candidate.text)
        && !creates_mixed_script_token(&candidate.text)
}

fn risky_single_prefix_deletion(original: &str, candidate: &CorrectionCandidate) -> bool {
    if !candidate.source.contains("extra") && !candidate.source.contains("cleanup") {
        return false;
    }
    let original = original.trim();
    let replacement = candidate.text.trim();
    let mut chars = original.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !matches!(first, 'й' | 'Й' | 'ы' | 'Ы' | 'в' | 'В') {
        return false;
    }
    let rest: String = chars.collect();
    if rest == replacement && looks_like_safe_detached_prefix_cleanup(first, replacement) {
        return false;
    }
    rest == replacement
        && original.chars().count() >= 6
        && is_known_russian_word_or_form(&replacement.to_lowercase())
}

fn looks_like_safe_detached_prefix_cleanup(first: char, replacement: &str) -> bool {
    if !matches!(first, 'ы' | 'Ы') {
        return false;
    }
    let lower = replacement.to_lowercase();
    lower.ends_with("ить")
        || lower.ends_with("ать")
        || lower.ends_with("ять")
        || lower.ends_with("еть")
        || lower.ends_with("уть")
        || lower.ends_with("ти")
}

fn bounded_len(text: &str) -> u16 {
    text.chars().count().min(u16::MAX as usize) as u16
}

fn script_mix(text: &str) -> ScriptMix {
    let token = text.trim();
    if token.is_empty() {
        return ScriptMix::Empty;
    }
    let has_ascii_alpha = token.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_cyr = has_cyrillic(token);
    let has_lat = has_latin(token);
    if has_cyr && has_lat {
        ScriptMix::Mixed
    } else if has_cyr {
        ScriptMix::Cyrillic
    } else if has_ascii_alpha && token.is_ascii() {
        ScriptMix::Ascii
    } else {
        ScriptMix::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_rule_graph::ids;
    use std::collections::HashSet;

    #[test]
    fn microbrain_accepts_existing_layout_candidate() {
        let ctx = MicroContext::new("djn ");
        let candidate = CorrectionCandidate {
            action: MicroAction::LayoutEnToRu,
            text: "вот ".to_string(),
            source: ids::LAYOUT_EN_TO_RU.to_string(),
            engine_score: None,
        };
        let trace = decide(&ctx, &[candidate], &MicrobrainOptions::default());
        assert_eq!(trace.chosen.as_deref(), Some("вот "));
    }

    #[test]
    fn microbrain_can_ablate_expert() {
        let opts = MicrobrainOptions::with_disabled(&["cli_guard_16k_stub".to_string()]);
        assert!(!opts.expert_enabled("cli_guard_16k_stub"));
        assert!(opts.expert_enabled("layout_signal_16k_stub"));
    }

    #[test]
    fn microbrain_can_enable_only_selected_cells() {
        let opts = MicrobrainOptions::with_enabled_only(&["sentence_mesh_64k_stub".to_string()]);
        assert!(opts.expert_enabled("sentence_mesh_64k_stub"));
        assert!(!opts.expert_enabled("layout_signal_16k_stub"));
        assert!(!opts.expert_enabled("layout_writer_64k_stub"));
    }

    #[test]
    fn microbrain_rejects_cli_flag_mutation() {
        let ctx = MicroContext::new("git checkout -b ");
        let candidate = CorrectionCandidate {
            action: MicroAction::LayoutEnToRu,
            text: "пше сруслаг еи ".to_string(),
            source: ids::LAYOUT_EN_TO_RU.to_string(),
            engine_score: None,
        };
        let trace = decide(&ctx, &[candidate], &MicrobrainOptions::default());
        assert_eq!(trace.chosen, None);
    }

    #[test]
    fn microbrain_rejects_multiword_layout_that_mutates_known_ascii_token() {
        let ctx = MicroContext::new("in ljv ");
        let candidate = CorrectionCandidate {
            action: MicroAction::LayoutEnToRu,
            text: "шт дом ".to_string(),
            source: ids::LAYOUT_EN_TO_RU.to_string(),
            engine_score: None,
        };
        let trace = decide(&ctx, &[candidate], &MicrobrainOptions::default());
        assert_eq!(trace.chosen, None);
        let candidate = trace.candidates.first().expect("candidate trace");
        assert!(candidate
            .expert_scores
            .iter()
            .any(|score| score.reason_code == "known_ascii_token_mutation_veto"));
    }

    #[test]
    fn expert64_profiles_are_unique_and_cache_sized() {
        let profiles = default_expert_profiles();
        let mut names = HashSet::new();
        assert_eq!(profiles.len(), 7);
        for profile in profiles {
            assert_eq!(profile.magic, EXPERT64_MAGIC);
            assert_eq!(profile.state_budget_bytes, EXPERT64_STATE_BYTES);
            assert_eq!(profile.input_schema_id, "lay.microfeatures.v2");
            assert!(names.insert(profile.expert_id), "duplicate expert id");
        }
    }

    #[test]
    fn microfeatures_do_not_store_raw_candidate_text() {
        let ctx = MicroContext::new("цусрфе ");
        let candidate = CorrectionCandidate {
            action: MicroAction::LayoutRuToEn,
            text: "wechat ".to_string(),
            source: ids::LAYOUT_RU_TO_EN.to_string(),
            engine_score: None,
        };
        let features = extract_features(&ctx, &candidate);
        assert_eq!(features.original_script, ScriptMix::Cyrillic);
        assert_eq!(features.candidate_script, ScriptMix::Ascii);
        assert!(features.source_is_layout);
        assert!(features.whitespace_preserved);
    }

    #[test]
    fn training_features_include_multiple_feature_spaces() {
        let candidate = CorrectionCandidate {
            action: MicroAction::LayoutEnToRu,
            text: "html вот ".to_string(),
            source: ids::LAYOUT_EN_TO_RU.to_string(),
            engine_score: None,
        };
        let features = candidate_features(&MicroContext::new("html djn "), &candidate);
        assert!(features
            .iter()
            .any(|item| item.starts_with("orig:technical:")));
        assert!(features
            .iter()
            .any(|item| item.starts_with("cand:known_ru:")));
        assert!(features
            .iter()
            .any(|item| item.starts_with("space_count_delta:")));
        assert!(features.iter().any(|item| item.starts_with("edit_span:")));
        assert!(features.iter().any(|item| item.starts_with("layout_edit:")));
        assert!(features
            .iter()
            .any(|item| item.starts_with("last_token_risky_layout:")));
    }

    #[test]
    fn sentence_mesh_cell_is_active() {
        assert!(default_expert_names().contains(&"sentence_mesh_64k_stub"));
        assert!(default_expert_names().contains(&TRAINED_LAYOUT_SIGNAL_EXPERT_ID));
    }

    #[test]
    fn nanda_status_reports_cells_and_roles() {
        let status = nanda_status_text();
        assert!(status.contains("NANDA: 8 клеток"));
        assert!(status.contains("генератор раскладки"));
        assert!(status.contains("защита: защита технических слов"));
        assert!(status.contains("сетка согласования: сетка согласования фразы"));
    }

    #[test]
    fn sentence_mesh_rejects_risky_detached_prefix_deletion() {
        let ctx = MicroContext::new("ыприблизительные ");
        let candidate = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "приблизительные ".to_string(),
            source: ids::EXTRA_LETTERS.to_string(),
            engine_score: None,
        };
        let trace = decide(&ctx, &[candidate], &MicrobrainOptions::default());
        assert_eq!(trace.chosen, None);
        let candidate = trace.candidates.first().expect("candidate trace");
        assert!(candidate
            .expert_scores
            .iter()
            .any(|score| score.reason_code == "mesh_detached_prefix_veto"));
    }

    #[test]
    fn sentence_mesh_allows_safe_detached_prefix_cleanup() {
        let ctx = MicroContext::new("ыпроверить ");
        let candidate = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "проверить ".to_string(),
            source: ids::EXTRA_LETTERS.to_string(),
            engine_score: Some(86.0),
        };
        let trace = decide(&ctx, &[candidate], &MicrobrainOptions::default());
        assert_eq!(trace.chosen.as_deref(), Some("проверить "));
        let candidate = trace.candidates.first().expect("candidate trace");
        assert!(!candidate
            .expert_scores
            .iter()
            .any(|score| score.reason_code == "mesh_detached_prefix_veto"));
    }

    #[test]
    fn microbrain_uses_engine_score_only_for_safe_typo_tiebreaks() {
        let ctx = MicroContext::new("помагу ");
        let weak = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "помашу ".to_string(),
            source: ids::SINGLE_LETTER_SUBSTITUTION.to_string(),
            engine_score: Some(81.69),
        };
        let strong = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "помогу ".to_string(),
            source: ids::VOWEL_CONFUSION.to_string(),
            engine_score: Some(82.99),
        };
        let trace = decide(&ctx, &[weak, strong], &MicrobrainOptions::default());
        assert_eq!(trace.chosen.as_deref(), Some("помогу "));

        let ctx = MicroContext::new("таможе ");
        let unsafe_cleanup = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "тамое ".to_string(),
            source: ids::EXTRA_LETTERS.to_string(),
            engine_score: Some(85.28),
        };
        let alternative = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "таможн ".to_string(),
            source: ids::SINGLE_LETTER_SUBSTITUTION.to_string(),
            engine_score: Some(81.69),
        };
        let trace = decide(
            &ctx,
            &[unsafe_cleanup, alternative],
            &MicrobrainOptions::default(),
        );
        assert_eq!(trace.chosen, None);
    }

    #[test]
    fn sentence_mesh_allows_pure_space_boundary_repair_before_veto() {
        let ctx = MicroContext::new("я вно ");
        let candidate = CorrectionCandidate {
            action: MicroAction::SplitGlue,
            text: "явно ".to_string(),
            source: ids::SPLIT_WORD_PAIR.to_string(),
            engine_score: None,
        };
        let trace = decide(&ctx, &[candidate], &MicrobrainOptions::default());
        assert_eq!(trace.chosen.as_deref(), Some("явно "));
        let candidate = trace.candidates.first().expect("candidate trace");
        assert!(candidate
            .expert_scores
            .iter()
            .any(|score| score.reason_code == "mesh_space_boundary_repair"));
    }

    #[test]
    fn disabling_sentence_mesh_removes_mesh_ticks() {
        let ctx = MicroContext::new("djn ");
        let candidate = CorrectionCandidate {
            action: MicroAction::LayoutEnToRu,
            text: "вот ".to_string(),
            source: ids::LAYOUT_EN_TO_RU.to_string(),
            engine_score: None,
        };
        let trace = decide(
            &ctx,
            &[candidate],
            &MicrobrainOptions::with_disabled(&["sentence_mesh_64k_stub".to_string()]),
        );
        let candidate = trace.candidates.first().expect("candidate trace");
        assert!(candidate.mesh_ticks.is_empty());
        assert!(!candidate
            .expert_scores
            .iter()
            .any(|score| score.expert == "sentence_mesh_64k_stub"));
    }

    #[test]
    fn expert64_pool_scale_probe() {
        let counts = [1usize, 8, 64, 256, 1024, 2048, 16_536];
        for count in counts {
            let bytes = expert64_pool_bytes(count);
            eprintln!("expert64_pool count={count} bytes={bytes}");
            assert_eq!(bytes, count * EXPERT64_STATE_BYTES);
        }
        assert_eq!(expert64_pool_bytes(1024), 64 * 1024 * 1024);
        assert_eq!(expert64_pool_bytes(2048), 128 * 1024 * 1024);
        assert_eq!(expert64_pool_bytes(16_536), 1_083_703_296);
    }

    #[test]
    fn expert_cell_plan_follows_cpu_cache_geometry() {
        let t480 = CpuCacheProfile {
            l1d_bytes_per_core: 32 * 1024,
            l2_bytes_per_core: 256 * 1024,
            l3_bytes_shared: 8 * 1024 * 1024,
            cache_line_bytes: 64,
        };
        let plan = plan_expert_cells(t480);
        assert_eq!(plan.cell_bytes, 64 * 1024);
        assert_eq!(plan.active_experts, 16);
        assert_eq!(plan.warm_pool_experts, 64);
        assert_eq!(
            expert_pool_bytes(plan.warm_pool_experts, plan.cell_bytes),
            4 * 1024 * 1024
        );

        let bigger_l2 = CpuCacheProfile {
            l1d_bytes_per_core: 48 * 1024,
            l2_bytes_per_core: 1024 * 1024,
            l3_bytes_shared: 24 * 1024 * 1024,
            cache_line_bytes: 64,
        };
        let plan = plan_expert_cells(bigger_l2);
        assert_eq!(plan.cell_bytes, 256 * 1024);
        assert_eq!(plan.active_experts, 12);
        assert_eq!(plan.warm_pool_experts, 48);
    }

    #[test]
    fn expert64_packet_roundtrips_as_exact_64kb_cell() {
        let mut cell = Expert64Cell::neutral(TRAINED_LAYOUT_SIGNAL_EXPERT_ID);
        cell.weights[17] = 42;
        let bytes = cell.to_bytes();
        assert_eq!(bytes.len(), EXPERT64_STATE_BYTES);
        let decoded = Expert64Cell::from_bytes(&bytes).expect("decode packet");
        assert_eq!(decoded.expert_id, TRAINED_LAYOUT_SIGNAL_EXPERT_ID);
        assert_eq!(decoded.weights.len(), EXPERT64_WEIGHT_BYTES);
        assert_eq!(decoded.weights[17], 42);
    }

    #[test]
    fn trained_layout_signal_cell_learns_tiny_dataset() {
        let rows = [
            Expert64TrainingRow {
                group_id: "layout".to_string(),
                original: "djn ".to_string(),
                candidate: "вот ".to_string(),
                operation: "layout".to_string(),
                label: true,
            },
            Expert64TrainingRow {
                group_id: "layout".to_string(),
                original: "djn ".to_string(),
                candidate: "djn ".to_string(),
                operation: "keep".to_string(),
                label: false,
            },
            Expert64TrainingRow {
                group_id: "acronym".to_string(),
                original: "ИНН ".to_string(),
                candidate: "BYY ".to_string(),
                operation: "layout".to_string(),
                label: false,
            },
            Expert64TrainingRow {
                group_id: "acronym".to_string(),
                original: "ИНН ".to_string(),
                candidate: "ИНН ".to_string(),
                operation: "keep".to_string(),
                label: true,
            },
        ];
        let (cell, report) = train_expert64_layout_signal(&rows, 12);
        assert_eq!(cell.weights.len(), EXPERT64_WEIGHT_BYTES);
        assert_eq!(cell.to_bytes().len(), EXPERT64_STATE_BYTES);
        assert!(cell.weights.iter().any(|weight| *weight != 0));
        assert!(report.accuracy >= 0.66);
    }

    #[test]
    fn trained_layout_signal_is_neutral_outside_layout_domain() {
        let ctx = MicroContext::new("помагу ");
        let candidate = CorrectionCandidate {
            action: MicroAction::TypoFix,
            text: "помогу ".to_string(),
            source: ids::MISSING_LETTER.to_string(),
            engine_score: None,
        };
        let score = TRAINED_LAYOUT_SIGNAL_64K.score(&ctx, &candidate);
        assert_eq!(score.confidence, 0.56);
        assert_eq!(score.reason_code, "trained_layout_out_of_domain");
    }
}
