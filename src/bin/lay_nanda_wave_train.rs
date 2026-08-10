use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lay::nanda_wave::llmwave;
use lay::nanda_wave::packet::{write_learned_packet, LearnedPacketEntry};
use lay::nanda_wave::L2PhaseTrainingEntry;
use lay::{lexicon, russian_lexicon};
use serde::Deserialize;

#[path = "lay_nanda_wave_train/l3_online.rs"]
mod l3_online;

#[allow(dead_code)]
#[path = "../nanda_wave/lexical_phase/format.rs"]
mod lexical_phase_format;
#[path = "../lexical_surface_atoms.rs"]
mod lexical_surface_atoms;
#[path = "../stable_hash.rs"]
mod stable_hash;

#[allow(dead_code)]
mod lexical_phase_compiler {
    include!("../nanda_wave/lexical_phase/compiler.rs");
}

use lexical_phase_format as format;
include!("lay_nanda_wave_train/lexical_phase_compile.rs");

const DEFAULT_DATASET: &str = "data/nanda_training/generated_cases.tsv";
const RECENT_ACTIONS: &str = ".local/share/lay/recent_actions.jsonl";
const CORRECTIONS_LOG: &str = ".local/share/lay/corrections.jsonl";
const USAGE_EVENTS: &str = ".local/share/lay/nanda_wave/word_usage_events.jsonl";
const DEFAULT_L11_TRAINING_SURFACES_PER_WORD: usize = 2;

#[derive(Debug, Clone)]
struct Learned {
    expected: String,
    operation: String,
    count: usize,
    conflicts: usize,
    live_count: usize,
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        print_usage();
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-V" | "--version"))
    {
        println!("lay-nanda-wave-train {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--watch-l3-context-online") {
        return l3_online::run(&args);
    }
    if let Some(reference) = arg_path(&args, "--compact-canonical-l2") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compact_canonical_l2_package(&reference, &output)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(compact) = arg_path(&args, "--prove-compact-canonical-l2") {
        let reference = arg_path(&args, "--reference").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--reference is required")
        })?;
        let report = lay::nanda_wave::prove_compact_canonical_l2_parity(&reference, &compact)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--canonical-l2-status") {
        println!(
            "{}",
            serde_json::to_string_pretty(&lay::nanda_wave::canonical_l2_status())
                .map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--compile-l4-cross-scene") {
        let input = arg_path(&args, "--input")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--input is required"))?;
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compile_l4_cross_scene_memory(&input, &output)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--prove-l4-cross-scene") {
        let russian = arg_path(&args, "--russian-words").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--russian-words is required")
        })?;
        let english = arg_path(&args, "--english-words").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--english-words is required")
        })?;
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::prove_l4_cross_scene_memory(&russian, &english, &output)?;
        if let Some(receipt) = arg_path(&args, "--receipt") {
            let mut bytes = serde_json::to_vec_pretty(&report).map_err(io::Error::other)?;
            bytes.push(b'\n');
            lay::private_file::write_private_bytes(&receipt, &bytes)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(package) = arg_path(&args, "--l4-cross-scene-status") {
        println!(
            "{}",
            serde_json::to_string_pretty(&lay::nanda_wave::l4_cross_scene_status_json(&package))
                .map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(cases) = arg_path(&args, "--prove-l3-sentence-context") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::build_and_prove_l3_sentence_context_memory(&cases, &output)?;
        if let Some(receipt) = arg_path(&args, "--receipt") {
            let mut bytes = serde_json::to_vec_pretty(&report).map_err(io::Error::other)?;
            bytes.push(b'\n');
            lay::private_file::write_private_bytes(&receipt, &bytes)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(l2_package) = arg_path(&args, "--prove-canonical-l2") {
        let l1_package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let morphology_corpus = arg_path(&args, "--morphology-corpus").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--morphology-corpus is required",
            )
        })?;
        let report = lay::nanda_wave::prove_canonical_l2_package(
            &l1_package,
            &l2_package,
            &morphology_corpus,
            arg_usize(&args, "--limit").unwrap_or(0),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(l2_package) = arg_path(&args, "--prove-compositional-l2") {
        let l1_package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let report = lay::nanda_wave::prove_compositional_l2_restoration(
            &l1_package,
            &l2_package,
            arg_usize(&args, "--heldout-per-class").unwrap_or(20_000),
            arg_usize(&args, "--workers").unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(1)
            }),
            arg_usize(&args, "--lemma-limit").unwrap_or(4),
            arg_usize(&args, "--form-limit").unwrap_or(16),
            arg_usize(&args, "--atom-relation-limit")
                .unwrap_or(lay::nanda_wave::CANONICAL_L2_ATOM_RELATION_LIMIT),
        )?;
        if let Some(receipt) = arg_path(&args, "--receipt") {
            let mut bytes = serde_json::to_vec_pretty(&report).map_err(io::Error::other)?;
            bytes.push(b'\n');
            lay::private_file::write_private_bytes(&receipt, &bytes)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(l2_package) = arg_path(&args, "--prove-contextual-compositional-l2") {
        let l1_package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let morphology_corpus = arg_path(&args, "--morphology-corpus").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--morphology-corpus is required",
            )
        })?;
        let report = lay::nanda_wave::prove_contextual_compositional_l2_restoration(
            &l1_package,
            &l2_package,
            &morphology_corpus,
            arg_usize(&args, "--heldout-per-class").unwrap_or(100),
            arg_usize(&args, "--workers").unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(1)
            }),
            arg_usize(&args, "--lemma-limit")
                .unwrap_or(lay::nanda_wave::CANONICAL_L2_LEMMA_FRONTIER),
            arg_usize(&args, "--active-lemma-limit")
                .unwrap_or(lay::nanda_wave::CANONICAL_L2_ACTIVE_LEMMA_LIMIT),
            arg_usize(&args, "--feature-limit")
                .unwrap_or(lay::nanda_wave::CANONICAL_L2_FEATURE_LIMIT),
            arg_usize(&args, "--form-limit").unwrap_or(lay::nanda_wave::CANONICAL_L2_FORM_LIMIT),
            arg_usize(&args, "--atom-relation-limit")
                .unwrap_or(lay::nanda_wave::CANONICAL_L2_ATOM_RELATION_LIMIT),
        )?;
        if let Some(receipt) = arg_path(&args, "--receipt") {
            let mut bytes = serde_json::to_vec_pretty(&report).map_err(io::Error::other)?;
            bytes.push(b'\n');
            lay::private_file::write_private_bytes(&receipt, &bytes)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(l2_package) = arg_path(&args, "--query-canonical-l2") {
        let l1_package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let context = arg_string(&args, "--context")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--context is required"))?;
        let seeds = arg_string(&args, "--seeds")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--seeds is required"))?
            .split(',')
            .map(str::trim)
            .filter(|surface| !surface.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let report = lay::nanda_wave::query_canonical_l2_package(
            &l1_package,
            &l2_package,
            &context,
            &seeds,
            arg_usize(&args, "--limit").unwrap_or(16),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(morphology_corpus) = arg_path(&args, "--compile-canonical-l2") {
        let l1_package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compile_canonical_l2_package(
            &l1_package,
            &morphology_corpus,
            &output,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(morphology_corpus) = arg_path(&args, "--export-l2-unseeded-l11-corpus") {
        let l1_package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::export_unseeded_l11_seed_corpus(
            &l1_package,
            &morphology_corpus,
            &output,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(manifest) = arg_path(&args, "--prove-l11-composite") {
        let base_corpus = arg_path(&args, "--base-corpus").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--base-corpus is required")
        })?;
        let delta_corpus = arg_path(&args, "--delta-corpus").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--delta-corpus is required")
        })?;
        let report = lay::nanda_wave::prove_l1_lexical_grokking_composite(
            &base_corpus,
            &delta_corpus,
            &manifest,
            arg_usize(&args, "--heldout-per-class").unwrap_or(20_000),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(package) = arg_path(&args, "--analyze-l1-forward-compression") {
        let report = lay::nanda_wave::analyze_l1_forward_compression(&package)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(package) = arg_path(&args, "--build-l1-v8") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let atoms_per_shard = arg_usize(&args, "--atoms-per-shard").unwrap_or(32);
        let report = lay::nanda_wave::build_lazy_v8_package_with_shard_size(
            &package,
            &output,
            atoms_per_shard,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--export-l1-latency-surfaces") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::export_l1_fixed_latency_surfaces(
            &corpus,
            &output,
            arg_usize(&args, "--max-words").unwrap_or(0),
            arg_usize(&args, "--heldout-per-class").unwrap_or(20_000),
            arg_usize(&args, "--samples").unwrap_or(512),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(package) = arg_path(&args, "--bench-l1-diverse-restoration") {
        let surfaces = arg_path(&args, "--surfaces")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--surfaces is required"))?;
        let report = lay::nanda_wave::benchmark_l1_diverse_restoration(
            &package,
            &surfaces,
            arg_usize(&args, "--limit").unwrap_or(64),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--init-l11-composite") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let base = arg_path(&args, "--base")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--base is required"))?;
        let report = lay::nanda_wave::initialize_l11_composite_manifest(&manifest, &base)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--admit-l11-delta") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let delta = arg_path(&args, "--delta")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--delta is required"))?;
        let proof_receipt = arg_path(&args, "--proof-receipt").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--proof-receipt is required for delta admission",
            )
        })?;
        let report = lay::nanda_wave::admit_l11_delta(
            &manifest,
            &delta,
            &proof_receipt,
            arg_string(&args, "--scope").as_deref(),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--admit-l11-tombstone") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let surface = arg_string(&args, "--surface")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--surface is required"))?;
        let proof_receipt = arg_path(&args, "--proof-receipt").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--proof-receipt is required for tombstone admission",
            )
        })?;
        let report = lay::nanda_wave::admit_l11_tombstone(
            &manifest,
            &surface,
            &proof_receipt,
            arg_string(&args, "--scope").as_deref(),
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(l1_package) = arg_path(&args, "--bench-l1-lexical-grokking") {
        let surface = arg_string(&args, "--surface")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--surface is required"))?;
        let iterations = arg_usize(&args, "--iterations").unwrap_or(1_000);
        let limit = arg_usize(&args, "--limit").unwrap_or(64);
        let report = lay::nanda_wave::benchmark_l1_lexical_grokking(
            &l1_package,
            &surface,
            iterations,
            limit,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(package) = arg_path(&args, "--query-l1-lexical-grokking") {
        let surface = arg_string(&args, "--surface")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--surface is required"))?;
        let limit = arg_usize(&args, "--limit").unwrap_or(8);
        let report = lay::nanda_wave::query_l1_lexical_grokking(&package, &surface, limit)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(package) = arg_path(&args, "--compact-l1-depth0-package") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compact_depth0_package(&package, &output)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--crystallize-l1-lexical-grokking") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_words = arg_usize(&args, "--max-words").unwrap_or(0);
        let heldout_per_class = arg_usize(&args, "--heldout-per-class").unwrap_or(20_000);
        let training_surfaces_per_word = arg_usize(&args, "--training-surfaces-per-word")
            .unwrap_or(DEFAULT_L11_TRAINING_SURFACES_PER_WORD);
        let training_surface_policy = arg_string(&args, "--training-surface-policy")
            .unwrap_or_else(|| "legacy-alphabetical".to_string());
        let training_surface_policy =
            lay::nanda_wave::ScaleTrainingSurfacePolicy::parse(&training_surface_policy)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
        let maximum_rss_mib = arg_usize(&args, "--max-rss-mib").unwrap_or(24 * 1024);
        let report = lay::nanda_wave::crystallize_l1_lexical_grokking_with_surface_policy(
            &corpus,
            &output,
            max_words,
            heldout_per_class,
            training_surfaces_per_word,
            maximum_rss_mib,
            training_surface_policy,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l1-lexical-grokking-scale-package") {
        let package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let max_words = arg_usize(&args, "--max-words").unwrap_or(0);
        let heldout_per_class = arg_usize(&args, "--heldout-per-class").unwrap_or(20_000);
        let training_surfaces_per_word = arg_usize(&args, "--training-surfaces-per-word")
            .unwrap_or(DEFAULT_L11_TRAINING_SURFACES_PER_WORD);
        let terminal_start = arg_usize(&args, "--terminal-start").unwrap_or_default();
        let terminal_count = arg_usize(&args, "--terminal-count").unwrap_or_default();
        let report = if terminal_start != 0 || terminal_count != 0 {
            lay::nanda_wave::prove_l1_lexical_grokking_scale_package_range(
                &corpus,
                &package,
                max_words,
                terminal_start,
                terminal_count,
                heldout_per_class,
                training_surfaces_per_word,
            )?
        } else {
            lay::nanda_wave::prove_l1_lexical_grokking_scale_package(
                &corpus,
                &package,
                max_words,
                heldout_per_class,
                training_surfaces_per_word,
            )?
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l1-lexical-grokking-package") {
        let package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let max_words = arg_usize(&args, "--max-words").unwrap_or(10_000);
        let report =
            lay::nanda_wave::prove_l1_lexical_grokking_package(&corpus, &package, max_words)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l1-lexical-grokking") {
        let output = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_words = arg_usize(&args, "--max-words").unwrap_or(10_000);
        let report = if args
            .iter()
            .any(|arg| arg == "--l1-complete-forward-postings")
        {
            lay::nanda_wave::prove_l1_lexical_grokking_complete_postings(
                &corpus, &output, max_words,
            )?
        } else {
            lay::nanda_wave::prove_l1_lexical_grokking(&corpus, &output, max_words)?
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--merge-l3-context-phase-shards")
    {
        let inputs = arg_paths(&args, "--input");
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        if inputs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one --input is required",
            ));
        }
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(1);
        let report =
            lay::nanda_wave::merge_l3_context_phase_shards(&inputs, &out, min_surface_support)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--init-l3-context-composite") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let base = arg_path(&args, "--base")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--base is required"))?;
        let report = lay::nanda_wave::initialize_l3_context_composite_manifest(&manifest, &base)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--admit-l3-context-delta") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let delta = arg_path(&args, "--delta")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--delta is required"))?;
        let proof_receipt = arg_path(&args, "--proof-receipt").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--proof-receipt is required for delta admission",
            )
        })?;
        let full_proof_receipt = arg_path(&args, "--full-proof-receipt");
        let scope = arg_string(&args, "--scope");
        let report = if let Some(full_proof_receipt) = full_proof_receipt {
            lay::nanda_wave::admit_l3_context_delta_with_full_proof(
                &manifest,
                &delta,
                &proof_receipt,
                &full_proof_receipt,
                scope.as_deref(),
            )?
        } else {
            lay::nanda_wave::admit_l3_context_delta(
                &manifest,
                &delta,
                Some(&proof_receipt),
                scope.as_deref(),
            )?
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--prove-l3-context-delta") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let delta = arg_path(&args, "--delta")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--delta is required"))?;
        let cases = arg_path(&args, "--cases")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--cases is required"))?;
        let receipt = arg_path(&args, "--out-receipt").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--out-receipt is required")
        })?;
        let report =
            lay::nanda_wave::prove_l3_context_delta_targeted(&manifest, &delta, &cases, &receipt)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        if report.get("verdict").and_then(serde_json::Value::as_str) != Some("PASS") {
            return Err(io::Error::other("targeted L3 delta proof did not pass"));
        }
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--prove-l3-sentence-context-delta")
    {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let delta = arg_path(&args, "--delta")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--delta is required"))?;
        let cases = arg_path(&args, "--cases")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--cases is required"))?;
        let receipt = arg_path(&args, "--out-receipt").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--out-receipt is required")
        })?;
        let report = lay::nanda_wave::prove_l3_sentence_context_delta_targeted(
            &manifest, &delta, &cases, &receipt,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        if report.get("verdict").and_then(serde_json::Value::as_str) != Some("PASS") {
            return Err(io::Error::other(
                "targeted L3 sentence delta proof did not pass",
            ));
        }
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--compact-l3-context-composite")
    {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compact_l3_context_composite(&manifest, &out)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--snapshot-l3-context-composite")
    {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::snapshot_l3_context_composite(&manifest, &out)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--snapshot-l3-context-candidate")
    {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let delta = arg_path(&args, "--delta")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--delta is required"))?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report =
            lay::nanda_wave::snapshot_l3_context_composite_with_delta(&manifest, &delta, &out)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--reload-l3-context-composite")
    {
        let report = lay::nanda_wave::reload_l3_context_composite()?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--l3-context-phase-status") {
        let memory = arg_path(&args, "--memory");
        println!(
            "{}",
            serde_json::to_string_pretty(&lay::nanda_wave::l3_context_phase_status_json(
                memory.as_deref(),
            ))
            .map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l3-context-phase-package") {
        let package = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let surface_evidence = arg_path(&args, "--surface-evidence");
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(2);
        let report = lay::nanda_wave::prove_l3_context_phase_package(
            &corpus,
            &package,
            max_fragments,
            min_profile_support,
            surface_evidence.as_deref(),
            min_surface_support,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l3-context-phase-delta-full") {
        let baseline = arg_path(&args, "--baseline-memory").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--baseline-memory is required")
        })?;
        let candidate = arg_path(&args, "--memory")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--memory is required"))?;
        let surface_evidence = arg_path(&args, "--surface-evidence").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--surface-evidence is required",
            )
        })?;
        let receipt = arg_path(&args, "--out-receipt").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--out-receipt is required")
        })?;
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(2);
        let report = lay::nanda_wave::prove_l3_context_phase_delta_full(
            &corpus,
            &baseline,
            &candidate,
            &surface_evidence,
            max_fragments,
            min_surface_support,
            &receipt,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        if report.get("verdict").and_then(serde_json::Value::as_str) != Some("PASS") {
            return Err(io::Error::other(
                "full L3 delta differential proof did not pass",
            ));
        }
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l3-context-composite-delta-full") {
        let manifest = arg_path(&args, "--manifest")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--manifest is required"))?;
        let delta = arg_path(&args, "--delta")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--delta is required"))?;
        let surface_evidence = arg_path(&args, "--surface-evidence").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--surface-evidence is required",
            )
        })?;
        let receipt = arg_path(&args, "--out-receipt").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--out-receipt is required")
        })?;
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(2);
        let report = lay::nanda_wave::prove_l3_context_composite_delta_full(
            &corpus,
            &manifest,
            &delta,
            &surface_evidence,
            max_fragments,
            min_surface_support,
            &receipt,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        if report.get("verdict").and_then(serde_json::Value::as_str) != Some("PASS") {
            return Err(io::Error::other(
                "full L3 composite delta differential proof did not pass",
            ));
        }
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--build-and-prove-l3-context-phase") {
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(2);
        let surface_evidence = arg_path(&args, "--surface-evidence").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--build-and-prove-l3-context-phase requires --surface-evidence CORRECTIONS.jsonl",
            )
        })?;
        let report =
            lay::nanda_wave::build_and_prove_l3_context_phase_memory_with_surface_evidence(
                &corpus,
                &surface_evidence,
                &out,
                max_fragments,
                min_profile_support,
                min_surface_support,
            )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l3-context-phase") {
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let report = lay::nanda_wave::prove_l3_context_phase_memory(
            &corpus,
            max_fragments,
            min_profile_support,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--compile-l3-context-delta") {
        let manifest = arg_path(&args, "--manifest").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--compile-l3-context-delta requires --manifest RUNTIME.json",
            )
        })?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(2);
        let pairwise_only = args.iter().any(|arg| arg == "--pairwise-only-delta");
        let surface_evidence = arg_path(&args, "--surface-evidence").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--compile-l3-context-delta requires --surface-evidence CORRECTIONS.jsonl",
            )
        })?;
        let report = lay::nanda_wave::compile_l3_context_delta_for_manifest(
            &manifest,
            &corpus,
            &surface_evidence,
            &out,
            min_profile_support,
            min_surface_support,
            pairwise_only,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--compile-l3-context-phase") {
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let min_surface_support = arg_u32(&args, "--min-surface-support").unwrap_or(2);
        let surface_evidence = arg_path(&args, "--surface-evidence").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "--compile-l3-context-phase requires --surface-evidence CORRECTIONS.jsonl",
            )
        })?;
        let report = lay::nanda_wave::compile_l3_context_phase_memory_with_surface_evidence(
            &corpus,
            &surface_evidence,
            &out,
            max_fragments,
            min_profile_support,
            min_surface_support,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--compile-l3-context-feedback-overlay")
    {
        let base = arg_path(&args, "--base")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--base is required"))?;
        let usage_events = arg_path(&args, "--usage-events").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--usage-events is required")
        })?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compile_l3_context_feedback_overlay_memory(
            &base,
            &usage_events,
            &out,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--build-l3-context-feedback-corpus")
    {
        let usage_events = arg_path(&args, "--usage-events").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--usage-events is required")
        })?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_repeat_per_phrase = arg_usize(&args, "--max-repeat-per-phrase").unwrap_or(4);
        let report = lay::nanda_wave::build_l3_context_feedback_corpus(
            &usage_events,
            &out,
            max_repeat_per_phrase,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--build-l2-lexical-feedback-corpus")
    {
        let usage_events = arg_path(&args, "--usage-events").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--usage-events is required")
        })?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_repeat_per_phrase = arg_usize(&args, "--max-repeat-per-phrase").unwrap_or(4);
        let max_repeat_per_word = arg_usize(&args, "--max-repeat-per-word").unwrap_or(4);
        let report = lay::nanda_wave::build_l2_lexical_feedback_corpus(
            &usage_events,
            &out,
            max_repeat_per_phrase,
            max_repeat_per_word,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--export-generated-russian-forms")
    {
        return run_export_generated_russian_forms(&args);
    }
    if args.iter().any(|arg| arg == "--compile-lexical-phase") {
        return run_lexical_phase_compile(&args);
    }
    if args.iter().any(|arg| arg == "--l2-surface-status") {
        print_l2_surface_status();
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--llmwave-corpus") {
        let out = arg_path(&args, "--llmwave-out")
            .or_else(|| arg_path(&args, "--out"))
            .or_else(llmwave::default_memory_path)
            .expect("default llmwave memory path");
        train_llmwave_corpus(&corpus, &out)?;
        return Ok(());
    }
    let dataset = arg_path(&args, "--dataset").unwrap_or_else(|| PathBuf::from(DEFAULT_DATASET));
    let out =
        arg_path(&args, "--out").unwrap_or_else(lay::nanda_wave::learned::default_memory_path);
    let phase_out = arg_path(&args, "--phase-out")
        .unwrap_or_else(lay::nanda_wave::default_l2_candidate_phase_memory_path);
    let pack_live = args.iter().any(|arg| arg == "--pack-live");
    let include_live_actions = pack_live || args.iter().any(|arg| arg == "--include-live-actions");
    let include_user_corrections =
        pack_live || args.iter().any(|arg| arg == "--include-user-corrections");
    let phase_only = args.iter().any(|arg| arg == "--phase-only");
    if phase_only {
        let phase_entries =
            phase_training_entries(&dataset, include_live_actions, include_user_corrections)?;
        write_phase_memory(&phase_out, phase_entries)?;
        println!("phase_out: {}", phase_out.display());
        return Ok(());
    }
    let mut learned = learn(&dataset)?;
    let live_report = if include_live_actions || include_user_corrections {
        add_live_learning(&mut learned, include_user_corrections)?
    } else {
        LiveLearningReport::default()
    };
    let phase_entries =
        phase_training_entries(&dataset, include_live_actions, include_user_corrections)?;
    write_memory(&out, &learned)?;
    write_phase_memory(&phase_out, phase_entries)?;
    print_summary(&dataset, &out, &phase_out, &learned, &live_report);
    Ok(())
}

fn print_usage() {
    println!(
        "usage: lay-nanda-wave-train [explicit training, proof, status, or composite command]\n\
         \nRun with no arguments only to compile the legacy default training package.\n\
         Common safe inspection commands:\n\
           --canonical-l2-status\n\
           --compact-canonical-l2 REFERENCE.bin --out COMPACT.bin\n\
           --prove-compact-canonical-l2 COMPACT.bin --reference REFERENCE.bin\n\
           --prove-compositional-l2 L2.bin --memory L1.bin [--heldout-per-class N] [--workers N] [--lemma-limit N] [--form-limit N] [--atom-relation-limit N] [--receipt PATH]\n\
           --prove-contextual-compositional-l2 L2.bin --memory L1.bin --morphology-corpus CORPUS.tsv [--heldout-per-class N] [--workers N] [--lemma-limit N] [--active-lemma-limit N] [--feature-limit N] [--form-limit N] [--atom-relation-limit N] [--receipt PATH]\n\
           --l3-context-phase-status [--memory PATH]\n\
           --l4-cross-scene-status PATH\n\
           --compile-l4-cross-scene --input EVENTS.jsonl --out PACKAGE.bin\n\
           --prove-l4-cross-scene --russian-words RU --english-words EN --out PACKAGE.bin\n\
           --reload-l3-context-composite\n\
           --version"
    );
}

fn arg_usize(args: &[String], name: &str) -> Option<usize> {
    args.windows(2)
        .find(|window| window[0] == name)
        .and_then(|window| window[1].parse().ok())
}

fn arg_u32(args: &[String], name: &str) -> Option<u32> {
    args.windows(2)
        .find(|window| window[0] == name)
        .and_then(|window| window[1].parse().ok())
}

fn print_l2_surface_status() {
    let status = serde_json::to_value(lay::nanda_wave::l2::l2_surface_memory_status())
        .expect("L2 surface status must serialize");
    println!("l2_surface_status:");
    for key in [
        "active_source_target",
        "source_words",
        "l1_centers",
        "l1_postings",
        "l2_word_centers",
        "grapheme_nodes",
        "grapheme_arcs",
        "decoder_states",
        "decoder_arcs",
        "training_surfaces",
        "artifact_bytes",
        "artifact_mmap_backed",
        "raw_word_table",
        "generated_forms_loaded",
        "generated_forms_words",
    ] {
        println!("  {key}: {}", status[key]);
    }
    let phase = lay::nanda_wave::l2_transition_phase_report_json(None);
    println!(
        "l2_transition_phase: loaded={} profiles={} hot_bytes={}",
        phase["loaded"].as_bool().unwrap_or(false),
        phase["profile_count"].as_u64().unwrap_or(0),
        phase["hot_bytes"].as_u64().unwrap_or(0)
    );
}

fn train_llmwave_corpus(corpus: &Path, out: &Path) -> io::Result<()> {
    let text = fs::read_to_string(corpus)?;
    let memory = llmwave::LlmWaveMemory::from_text(&text);
    llmwave::write_memory_packet(out, &memory)?;
    let bytes = fs::metadata(out).map(|meta| meta.len()).unwrap_or_default();
    println!(
        "llmwave_corpus_train: input={} output={} records={} vocabulary={} bytes={} record_bytes={}",
        corpus.display(),
        out.display(),
        memory.len(),
        memory.vocabulary_len(),
        bytes,
        llmwave::LLMWAVE_RECORD_BYTES
    );
    Ok(())
}

fn learn(path: &Path) -> io::Result<BTreeMap<String, Learned>> {
    let text = fs::read_to_string(path)?;
    let mut map = BTreeMap::<String, Learned>::new();
    for (idx, line) in text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 8 || cols[5] != "1" || cols[2] == cols[3] {
            continue;
        }
        let original = cols[2].trim_end();
        let expected = cols[3].trim_end();
        if original == expected {
            continue;
        }
        let entry = map.entry(original.to_string()).or_insert_with(|| Learned {
            expected: expected.to_string(),
            operation: cols[4].to_string(),
            count: 0,
            conflicts: 0,
            live_count: 0,
        });
        if entry.expected == expected {
            entry.count += 1;
        } else {
            entry.conflicts += 1;
        }
    }
    map.retain(|_, item| item.count > 0 && item.conflicts == 0);
    Ok(map)
}

#[derive(Debug, Default)]
struct LiveLearningReport {
    read: usize,
    accepted: usize,
    skipped: usize,
    user_skipped: usize,
}

#[derive(Debug, Deserialize)]
struct LiveAction {
    #[serde(default)]
    kind: String,
    #[serde(default, rename = "from")]
    from_text: String,
    #[serde(default, rename = "to")]
    to_text: String,
    #[serde(default)]
    safety_allow_apply: Option<bool>,
    #[serde(default)]
    lay_from: String,
    #[serde(default)]
    lay_to: String,
    #[serde(default)]
    user_target: String,
}

fn add_live_learning(
    learned: &mut BTreeMap<String, Learned>,
    include_user_corrections: bool,
) -> io::Result<LiveLearningReport> {
    let mut report = LiveLearningReport::default();
    for path in live_paths() {
        add_live_file(learned, &path, include_user_corrections, &mut report)?;
    }
    learned.retain(|_, item| item.count > 0 && item.conflicts == 0);
    Ok(report)
}

fn add_live_file(
    learned: &mut BTreeMap<String, Learned>,
    path: &Path,
    include_user_corrections: bool,
    report: &mut LiveLearningReport,
) -> io::Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        report.read += 1;
        let Ok(action) = serde_json::from_str::<LiveAction>(line) else {
            report.skipped += 1;
            continue;
        };
        if !is_learnable_live_kind(&action.kind, include_user_corrections) {
            if action.kind == "user-correction" {
                report.user_skipped += 1;
            } else {
                report.skipped += 1;
            }
            continue;
        }
        let Some((from, to)) = normalized_live_pair(&action.from_text, &action.to_text) else {
            report.skipped += 1;
            continue;
        };
        let operation = operation_from_live_kind(&action.kind, &from, &to).to_string();
        let entry = learned.entry(from).or_insert_with(|| Learned {
            expected: to.clone(),
            operation,
            count: 0,
            conflicts: 0,
            live_count: 0,
        });
        if entry.expected == to {
            entry.count += 1;
            entry.live_count += 1;
            report.accepted += 1;
        } else {
            entry.conflicts += 1;
            report.skipped += 1;
        }
    }
    Ok(())
}

fn is_learnable_live_kind(kind: &str, include_user_corrections: bool) -> bool {
    matches!(
        kind,
        "typing-assist" | "ime-typing-assist" | "layout-replay" | "smart-text"
    ) || (include_user_corrections && kind == "user-correction")
}

fn normalized_live_pair(from: &str, to: &str) -> Option<(String, String)> {
    let from = from.trim_end();
    let to = to.trim_end();
    if from.is_empty()
        || to.is_empty()
        || from == to
        || from.chars().count() > 96
        || to.chars().count() > 96
        || from.chars().any(char::is_control)
        || to.chars().any(char::is_control)
        || from.split_whitespace().count().max(1) > 6
        || to.split_whitespace().count().max(1) > 6
    {
        return None;
    }
    Some((from.to_string(), to.to_string()))
}

fn operation_from_live_kind(kind: &str, from: &str, to: &str) -> &'static str {
    if kind == "layout-replay" || scripts_look_layout_like(from, to) {
        "layout"
    } else if from.split_whitespace().count() != to.split_whitespace().count() {
        "split"
    } else {
        "typo"
    }
}

fn scripts_look_layout_like(from: &str, to: &str) -> bool {
    let from_ascii = from.chars().any(|ch| ch.is_ascii_alphabetic());
    let from_cyr = from
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    let to_ascii = to.chars().any(|ch| ch.is_ascii_alphabetic());
    let to_cyr = to
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    (from_ascii && to_cyr) || (from_cyr && to_ascii)
}

fn live_paths() -> Vec<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    vec![
        home.join(RECENT_ACTIONS),
        home.join(CORRECTIONS_LOG),
        home.join(USAGE_EVENTS),
    ]
}

fn write_memory(path: &Path, learned: &BTreeMap<String, Learned>) -> io::Result<()> {
    let entries = learned
        .iter()
        .map(|(original, item)| LearnedPacketEntry {
            original: original.clone(),
            expected: item.expected.clone(),
            operation: item.operation.clone(),
            count: item.count,
        })
        .collect::<Vec<_>>();
    let report = write_learned_packet(path, &entries)?;
    println!(
        "cell32_packet: bytes={} encoded={} skipped={}",
        lay::nanda_wave::CELL32_BYTES,
        report.encoded,
        report.skipped
    );
    Ok(())
}

fn write_phase_memory(path: &Path, entries: Vec<L2PhaseTrainingEntry>) -> io::Result<()> {
    let bytes = lay::nanda_wave::write_l2_candidate_phase_memory_labeled(path, entries)?;
    println!("l2_candidate_phase_packet: bytes={bytes}");
    let report = lay::nanda_wave::l2_transition_phase_report_json(Some(path));
    println!(
        "l2_transition_phase_profiles: profiles={} raw_words_stored={}",
        report["profile_count"].as_u64().unwrap_or(0),
        report["raw_words_stored"].as_bool().unwrap_or(true)
    );
    Ok(())
}

fn phase_training_entries(
    dataset: &Path,
    include_live_actions: bool,
    include_user_corrections: bool,
) -> io::Result<Vec<L2PhaseTrainingEntry>> {
    let text = fs::read_to_string(dataset)?;
    let rows = text
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols = line.split('\t').collect::<Vec<_>>();
            (cols.len() >= 8 && cols[2] != cols[3]).then(|| {
                (
                    cols[0].to_string(),
                    cols[2].trim_end().to_string(),
                    cols[3].trim_end().to_string(),
                    cols[4].to_string(),
                    cols[5] == "1",
                )
            })
        })
        .collect::<Vec<_>>();
    let group_operators = rows
        .iter()
        .filter(|row| row.4)
        .map(|row| {
            (
                row.0.clone(),
                lay::nanda_wave::infer_l2_transition_operator(&row.1, &row.2, &row.3).to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut entries = rows
        .into_iter()
        .filter_map(|(group, original, candidate, operation, accepted)| {
            let operation = group_operators.get(&group).cloned().unwrap_or(operation);
            (!original.is_empty() && !candidate.is_empty()).then_some(L2PhaseTrainingEntry {
                original,
                candidate,
                operation,
                accepted,
                count: 1,
            })
        })
        .collect::<Vec<_>>();

    if include_live_actions || include_user_corrections {
        for path in live_paths() {
            append_live_phase_entries(&mut entries, &path, include_user_corrections)?;
        }
    }
    Ok(entries)
}

fn append_live_phase_entries(
    entries: &mut Vec<L2PhaseTrainingEntry>,
    path: &Path,
    include_user_corrections: bool,
) -> io::Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(action) = serde_json::from_str::<LiveAction>(line) else {
            continue;
        };
        if action.kind == "layout-replay" {
            push_phase_entry(
                entries,
                &action.from_text,
                &action.to_text,
                &action.kind,
                true,
            );
        } else if action.kind == "candidate_before_apply"
            && action.safety_allow_apply == Some(false)
        {
            push_phase_entry(
                entries,
                &action.from_text,
                &action.to_text,
                &action.kind,
                false,
            );
        } else if include_user_corrections && action.kind == "user-correction" {
            push_causal_user_phase_entries(entries, &action);
        }
    }
    Ok(())
}

fn push_causal_user_phase_entries(entries: &mut Vec<L2PhaseTrainingEntry>, action: &LiveAction) {
    // A generic user-correction record can describe later typing at another
    // caret position. Train only an observed local chain: raw input -> Lay
    // proposal -> immediate user replacement of that exact proposal.
    let Some((original, automatic)) = normalized_live_pair(&action.lay_from, &action.lay_to) else {
        return;
    };
    let full_target = if action.user_target.trim().is_empty() {
        let Some(target) = lay::word_buffer::reconstruct_user_correction_target(
            &action.lay_to,
            &action.from_text,
            &action.to_text,
        ) else {
            return;
        };
        target
    } else {
        action.user_target.clone()
    };
    let Some((_, target)) = normalized_live_pair(&automatic, &full_target) else {
        return;
    };
    if original.split_whitespace().count() != 1
        || automatic.split_whitespace().count() != 1
        || target.split_whitespace().count() != 1
        || automatic == target
    {
        return;
    }
    let operator =
        lay::nanda_wave::infer_l2_transition_operator(&original, &target, "user-correction");
    let automatic_operator =
        lay::nanda_wave::infer_l2_transition_operator(&original, &automatic, "user-correction");
    if !matches!(
        operator,
        "adjacent_transposition"
            | "missing_letter_repair"
            | "repeated_letter_repair"
            | "extra_letter_repair"
            | "letter_substitution"
    ) || automatic_operator != operator
    {
        return;
    }
    entries.push(L2PhaseTrainingEntry {
        original: original.clone(),
        candidate: target,
        operation: operator.to_string(),
        accepted: true,
        count: 1,
    });
    entries.push(L2PhaseTrainingEntry {
        original,
        candidate: automatic,
        operation: operator.to_string(),
        accepted: false,
        count: 1,
    });
}

fn push_phase_entry(
    entries: &mut Vec<L2PhaseTrainingEntry>,
    from: &str,
    to: &str,
    kind: &str,
    accepted: bool,
) {
    let Some((original, candidate)) = normalized_live_pair(from, to) else {
        return;
    };
    entries.push(L2PhaseTrainingEntry {
        operation: operation_from_live_kind(kind, &original, &candidate).to_string(),
        original,
        candidate,
        accepted,
        count: 1,
    });
}

fn print_summary(
    dataset: &Path,
    out: &Path,
    phase_out: &Path,
    learned: &BTreeMap<String, Learned>,
    live_report: &LiveLearningReport,
) {
    let mut by_operation = BTreeMap::<&str, usize>::new();
    let mut live_entries = 0usize;
    for item in learned.values() {
        *by_operation.entry(&item.operation).or_default() += 1;
        if item.live_count > 0 {
            live_entries += 1;
        }
    }
    println!("dataset: {}", dataset.display());
    println!("out: {}", out.display());
    println!("phase_out: {}", phase_out.display());
    println!("learned_corrections: {}", learned.len());
    if live_report.read > 0 {
        println!(
            "live_actions: read={} accepted={} skipped={} user_skipped={} live_entries={}",
            live_report.read,
            live_report.accepted,
            live_report.skipped,
            live_report.user_skipped,
            live_entries
        );
    }
    for (operation, count) in by_operation {
        println!("  {operation}: {count}");
    }
}

fn arg_path(args: &[String], name: &str) -> Option<PathBuf> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| PathBuf::from(&pair[1])))
}

fn arg_string(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| pair[1].clone()))
}

fn arg_paths(args: &[String], name: &str) -> Vec<PathBuf> {
    args.windows(2)
        .filter(|window| window[0] == name)
        .map(|window| PathBuf::from(&window[1]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_typing_action_is_learnable() {
        let pair = normalized_live_pair("fавтозамена ", "автозамена ").unwrap();
        assert_eq!(pair, ("fавтозамена".to_string(), "автозамена".to_string()));
        assert!(is_learnable_live_kind("ime-typing-assist", false));
        assert_eq!(
            operation_from_live_kind("ime-typing-assist", &pair.0, &pair.1),
            "layout"
        );
    }

    #[test]
    fn user_corrections_are_opt_in_for_training() {
        assert!(!is_learnable_live_kind("user-correction", false));
        assert!(is_learnable_live_kind("user-correction", true));
    }

    #[test]
    fn causal_user_correction_compiles_a_positive_and_candidate_specific_anti() {
        let action = LiveAction {
            kind: "user-correction".to_string(),
            from_text: "провекра ".to_string(),
            to_text: "проверка ".to_string(),
            safety_allow_apply: None,
            lay_from: "провека ".to_string(),
            lay_to: "провекра ".to_string(),
            user_target: "проверка ".to_string(),
        };
        let mut entries = Vec::new();

        push_causal_user_phase_entries(&mut entries, &action);

        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .any(|entry| entry.accepted && entry.candidate == "проверка"));
        assert!(entries
            .iter()
            .any(|entry| !entry.accepted && entry.candidate == "провекра"));
    }

    #[test]
    fn unrelated_later_user_text_is_not_a_phase_label() {
        let action = LiveAction {
            kind: "user-correction".to_string(),
            from_text: "потом ".to_string(),
            to_text: "предложение ".to_string(),
            safety_allow_apply: None,
            lay_from: "птом ".to_string(),
            lay_to: "потом ".to_string(),
            user_target: "".to_string(),
        };
        let mut entries = Vec::new();

        push_causal_user_phase_entries(&mut entries, &action);

        assert!(entries.is_empty());
    }

    #[test]
    fn llmwave_corpus_training_writes_phrase_memory_packet() {
        let dir =
            std::env::temp_dir().join(format!("lay-llmwave-corpus-train-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let corpus = dir.join("book.txt");
        let out = dir.join("phrase_memory.llmw.bin");
        fs::write(
            &corpus,
            "на улице опять идёт дождь\nя хочу проверить автозамену\n",
        )
        .unwrap();

        train_llmwave_corpus(&corpus, &out).unwrap();

        let memory = llmwave::read_memory_packet(&out).unwrap();
        assert!(!memory.is_empty());
        assert!(memory.vocabulary_len() >= 6);
        let _ = fs::remove_dir_all(&dir);
    }
}
