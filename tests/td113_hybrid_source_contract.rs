use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(relative: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(relative)).expect("runtime source")
}

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing section start {start:?}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing section end {end:?}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn sha256(path: &Path) -> String {
    let bytes = std::fs::read(path).expect("protected artifact bytes");
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn td113_live_adapters_share_the_hybrid_mode_mapper() {
    let core = read("src/correction_core.rs");
    let mapper = section(
        &core,
        "pub const fn live_correction_mode",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
    );
    let mapper = without_whitespace(mapper);
    assert!(mapper.contains(
        "ifnanda_autocorrect{CorrectionMode::DeterministicAndNanda}else{CorrectionMode::DeterministicOnly}"
    ));

    let sources = read("src/correction_core/candidate_sources.rs");
    let source_mode = section(&sources, "impl L2CandidateSource", "fn push_candidates");
    let source_mode = without_whitespace(source_mode);
    assert!(source_mode
        .contains("constDETERMINISTIC_AND_NANDA:[Self;2]=[Self::Deterministic,Self::Nanda]"));
    assert!(source_mode
        .contains("CorrectionMode::DeterministicAndNanda=>&Self::DETERMINISTIC_AND_NANDA"));

    let ime = read("src/ime_correction.rs");
    let ime_mapping = section(&ime, "fn correction_mode(self", "#[cfg(test)]");
    assert!(ime_mapping.contains("crate::correction_core::live_correction_mode"));
    assert!(!ime_mapping.contains("CorrectionMode::NandaOnly"));

    let cache = read("src/nanda_wave/l2_field/cache.rs");
    assert!(!cache.contains("L11SeedKey"));
    assert!(!cache.contains("l11_seed_ready"));
    assert!(!cache.contains("get_or_query_l11_seeds"));

    let bridge = read("src/nanda_wave/l2_field/bridge.rs");
    let l11_readout = section(
        &bridge,
        "fn live_l11_contour_results",
        "fn merge_contour_seeds",
    );
    assert!(l11_readout.contains("request_l11_seed_surfaces"));
    assert!(!l11_readout.contains("get_or_query_l11_seeds"));

    let daemon = read("src/bin/lay_daemon/typing_assist_runtime/decoder/gate.rs");
    let daemon_mapping = section(
        &daemon,
        "fn decode_input_gate_tail",
        "fn build_input_gate_decoded_tail",
    );
    assert!(daemon_mapping.contains("lay::correction_core::live_correction_mode"));
    assert!(!daemon_mapping.contains("NandaOnly"));

    let cli = read("src/main.rs");
    let live_cli = section(&cli, "fn print_correction_core_explanation", "fn convert");
    assert!(live_cli.contains("correction_core::live_correction_mode"));
    assert!(!live_cli.contains("NandaOnly"));

    let diagnostic_cli = section(
        &cli,
        "fn print_nanda_explanation",
        "fn print_correction_core_explanation",
    );
    assert!(diagnostic_cli.contains("correction_core::CorrectionMode::NandaOnly"));
    assert!(!diagnostic_cli.contains("live_correction_mode"));
}

#[test]
fn td113_runtime_authority_contains_no_fixture_or_source_id_shortcut() {
    let mut runtime_sections = vec![
        read("src/correction_core.rs"),
        read("src/correction_core/candidate_sources.rs"),
        read("src/main.rs"),
        read("src/nanda_wave/l2_field/bridge.rs"),
        read("src/typing_transition/decision/admission.rs"),
        read("src/typing_transition/proposal_admission.rs"),
        read("src/typing_transition/proposal_admission/structural_guards.rs"),
        read("src/typing_transition/proposal_admission/surface_support.rs"),
        read("src/bin/lay_daemon/typing_assist_runtime/decoder/gate.rs"),
    ];
    let ime = read("src/ime_correction.rs");
    runtime_sections.push(
        ime.split_once("#[cfg(test)]\nmod tests")
            .expect("IME runtime/test boundary")
            .0
            .to_string(),
    );

    for forbidden in [
        "плозо",
        "обьяснить",
        "объяснить",
        "верменно",
        "текст е",
        "т ыпочитай",
        "протколах",
        "пр отколах",
        "split_word_pair",
        "moved_prefix_pair",
        "TD113_ALLOW_UNVERIFIED_BOUNDARY",
        "TD113_BOUNDARY_RANK_BONUS",
        "TD113_IGNORE_EXACT_NEGATIVE",
    ] {
        assert!(
            runtime_sections
                .iter()
                .all(|source| !source.contains(forbidden)),
            "runtime authority contains forbidden TD-113 shortcut {forbidden:?}"
        );
    }

    let core = read("src/correction_core.rs");
    let resolver = section(
        &core,
        "fn resolve_text_correction_observed_internal",
        "pub fn correction_gate_stats_json",
    );
    assert_eq!(
        resolver.matches("L2CandidateLattice::with_options").count(),
        1
    );
    assert_eq!(
        resolver
            .matches("into_observed_resolution_with_peak_context")
            .count(),
        1
    );
    assert!(!resolver.contains("resolve_text_correction("));
}

#[test]
fn td113_unsuperseded_protected_artifacts_match_the_v4_preflight_baseline() {
    let manifest_path =
        Path::new(ROOT).join("tech_debt/evidence/td113-implementation-preflight-v4.json");
    let manifest_dir = manifest_path.parent().expect("manifest directory");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).expect("V4 preflight manifest"))
            .expect("valid V4 preflight manifest");
    let baseline_checks = manifest["baseline_checks"]
        .as_array()
        .expect("baseline_checks array");

    for protected in manifest["preserved_artifacts"]
        .as_array()
        .expect("preserved_artifacts array")
    {
        assert_eq!(protected["policy"].as_str(), Some("byte_identical"));
        let baseline_id = protected["baseline_check_id"]
            .as_str()
            .expect("baseline check id");
        let baseline = baseline_checks
            .iter()
            .find(|check| check["id"].as_str() == Some(baseline_id))
            .unwrap_or_else(|| panic!("baseline check {baseline_id:?}"));

        // TD-115 deliberately changed this former TD-113 byte boundary to scope
        // Space mutation to the current token.  The immutable TD-113 preflight
        // remains historical evidence; its other protected artifacts must stay
        // byte-identical, while this successor-owned file is covered by the
        // dedicated IME Space boundary contract.
        if baseline_id == "space-prefetch" {
            assert_eq!(
                baseline["path"].as_str(),
                Some("../../src/bin/lay_ibus_engine/space_autocorrect_prefetch.rs")
            );
            continue;
        }

        if baseline_id == "ime-mutation" {
            const COMPOSITION_COMMIT_PATH: &str = "src/bin/lay_ibus_engine/composition_commit.rs";
            const TD120_COMPOSITION_SHA256: &str =
                "bfee6bdabc148de1cccf44558d176e3d78c101f42b8912e4ffa25b9de131f713";
            const COMPOSITION_COMMIT_MODE: &str = "0664";
            assert_eq!(
                baseline["path"].as_str(),
                Some("../../src/bin/lay_ibus_engine/composition_commit.rs")
            );
            let binding_path = Path::new(ROOT)
                .join("tech_debt/evidence/td120-composition-mutation-successor.json");
            let binding: serde_json::Value = serde_json::from_slice(
                &std::fs::read(&binding_path).expect("TD-120 successor binding"),
            )
            .expect("valid TD-120 successor binding");
            let predecessor = &binding["predecessor"];
            let successor = &binding["successor"];
            assert_eq!(
                predecessor["manifest"].as_str(),
                Some("tech_debt/evidence/td113-implementation-preflight-v4.json")
            );
            assert_eq!(predecessor["baseline_id"].as_str(), Some(baseline_id));
            assert_eq!(predecessor["path"].as_str(), Some(COMPOSITION_COMMIT_PATH));
            assert_eq!(
                predecessor["sha256"].as_str(),
                baseline["expect"]["sha256"].as_str()
            );
            assert_eq!(
                predecessor["mode"].as_str(),
                baseline["expect"]["mode"].as_str()
            );
            assert_eq!(
                successor["task"].as_str(),
                Some("tech_debt/120-scope-autocorrect-suppression-to-word-lifetime.md")
            );
            assert_eq!(
                successor["review"].as_str(),
                Some("tech_debt/evidence/td120-code-review-pass2.md")
            );
            assert_eq!(successor["review_score"].as_str(), Some("8/10"));
            assert_eq!(successor["path"].as_str(), Some(COMPOSITION_COMMIT_PATH));
            assert_eq!(successor["sha256"].as_str(), Some(TD120_COMPOSITION_SHA256));
            assert_eq!(successor["mode"].as_str(), Some(COMPOSITION_COMMIT_MODE));

            let td121_binding_path = Path::new(ROOT)
                .join("tech_debt/evidence/td121-composition-mutation-successor.json");
            let td121_binding: serde_json::Value = serde_json::from_slice(
                &std::fs::read(&td121_binding_path).expect("TD-121 successor binding"),
            )
            .expect("valid TD-121 successor binding");
            assert_eq!(
                td121_binding["schema"].as_str(),
                Some("lay.tech-debt.successor-binding.v1")
            );
            assert_eq!(td121_binding["task"].as_str(), Some("TD-121"));
            assert_eq!(
                td121_binding["status"].as_str(),
                Some("ACCEPTED"),
                "TD-121 must not admit an unreviewed proposed successor"
            );
            assert_eq!(
                td121_binding["checkpoint"]["state"].as_str(),
                Some("frozen-at-checkpoint")
            );
            let td121_predecessor = &td121_binding["predecessor"];
            assert_eq!(td121_predecessor["task"].as_str(), Some("TD-120"));
            assert_eq!(
                td121_predecessor["binding"].as_str(),
                Some("tech_debt/evidence/td120-composition-mutation-successor.json")
            );
            assert_eq!(
                td121_predecessor["path"].as_str(),
                Some(COMPOSITION_COMMIT_PATH)
            );
            assert_eq!(
                td121_predecessor["sha256"].as_str(),
                Some(TD120_COMPOSITION_SHA256)
            );
            assert_eq!(
                td121_predecessor["mode"].as_str(),
                Some(COMPOSITION_COMMIT_MODE)
            );

            let td121_successor = &td121_binding["successor"];
            assert_eq!(
                td121_successor["task"].as_str(),
                Some("tech_debt/121-preserve-word-across-ime-layout-handoff.md")
            );
            assert_eq!(
                td121_successor["path"].as_str(),
                Some(COMPOSITION_COMMIT_PATH)
            );
            assert_eq!(
                td121_successor["mode"].as_str(),
                Some(COMPOSITION_COMMIT_MODE)
            );
            let successor_path = Path::new(ROOT).join(
                td121_successor["path"]
                    .as_str()
                    .expect("TD-121 successor source path"),
            );
            // The first-word repair deliberately extends only explicit suffix
            // append. Keep TD-121 frozen and verify its reviewed successor.
            assert_eq!(
                td121_successor["sha256"].as_str(),
                Some("33739ccb18d07fd7206f4b98e605f5e45ec5fe3041e6033f4f70190397f01d5f")
            );
            let suffix_binding: serde_json::Value = serde_json::from_str(&read(
                "tech_debt/evidence/ime-first-word-suffix-composition-successor.json",
            ))
            .expect("valid first-word suffix successor binding");
            assert_eq!(
                suffix_binding["schema"].as_str(),
                Some("lay.tech-debt.successor-binding.v1")
            );
            assert_eq!(suffix_binding["status"].as_str(), Some("ACCEPTED"));
            assert_eq!(
                suffix_binding["runtime_authority_changed"].as_bool(),
                Some(false)
            );
            let suffix_predecessor = &suffix_binding["predecessor"];
            assert_eq!(
                suffix_predecessor["binding"].as_str(),
                Some("tech_debt/evidence/td121-composition-mutation-successor.json")
            );
            assert_eq!(
                suffix_predecessor["binding_sha256"].as_str(),
                Some(sha256(&td121_binding_path).as_str())
            );
            for key in ["path", "sha256", "mode"] {
                assert_eq!(suffix_predecessor[key], td121_successor[key]);
            }
            let suffix_successor = &suffix_binding["successor"];
            assert_eq!(suffix_successor["path"], td121_successor["path"]);
            assert_eq!(suffix_successor["mode"], td121_successor["mode"]);
            assert_eq!(
                sha256(&successor_path),
                suffix_successor["sha256"]
                    .as_str()
                    .expect("first-word suffix successor sha256")
            );
            let suffix_review = &suffix_binding["review"];
            assert_eq!(suffix_review["state"].as_str(), Some("PASS"));
            assert_eq!(suffix_review["score"].as_str(), Some("9/10"));
            assert_eq!(suffix_review["high_findings"].as_u64(), Some(0));
            assert_eq!(suffix_review["medium_findings"].as_u64(), Some(0));
            assert_eq!(
                suffix_review["report"].as_str(),
                Some("tech_debt/evidence/ime-first-word-suffix-source-review.md")
            );
            assert_eq!(
                suffix_review["report_sha256"].as_str(),
                Some(
                    sha256(
                        &Path::new(ROOT).join(
                            suffix_review["report"]
                                .as_str()
                                .expect("source review path"),
                        )
                    )
                    .as_str()
                )
            );
            assert_eq!(
                std::fs::metadata(&successor_path)
                    .expect("TD-121 successor metadata")
                    .permissions()
                    .mode()
                    & 0o7777,
                u32::from_str_radix(
                    td121_successor["mode"]
                        .as_str()
                        .expect("TD-121 successor mode"),
                    8,
                )
                .expect("octal TD-121 successor mode")
            );
            let review = &td121_binding["review"];
            assert_eq!(review["state"].as_str(), Some("PASS"));
            let review_report = review["report"]
                .as_str()
                .expect("TD-121 independent review report");
            assert!(!review_report.trim().is_empty());
            assert!(
                review_report.starts_with("tech_debt/evidence/"),
                "TD-121 independent review report must be a repository evidence path"
            );
            assert!(
                Path::new(ROOT).join(review_report).is_file(),
                "TD-121 independent review report must exist: {review_report}"
            );
            let review_score = review["score"]
                .as_str()
                .and_then(|score| score.strip_suffix("/10"))
                .and_then(|score| score.parse::<u8>().ok())
                .expect("TD-121 review score in N/10 form");
            assert!(
                review_score >= 8,
                "TD-121 review score must be at least 8/10"
            );
            assert_eq!(review["high_findings"].as_u64(), Some(0));
            assert_eq!(review["medium_findings"].as_u64(), Some(0));

            let readiness = &td121_binding["readiness"];
            assert_eq!(
                readiness["state"].as_str(),
                Some("PASS"),
                "TD-121 must not admit pending functional readiness"
            );
            let unknown_start =
                &readiness["functional_tests"]["unknown_start_active_composition_refusal"];
            assert_eq!(unknown_start["state"].as_str(), Some("PASS"));
            assert!(!unknown_start["test"]
                .as_str()
                .expect("UnknownStart functional test reference")
                .trim()
                .is_empty());
            let known_start =
                &readiness["functional_tests"]["known_start_complete_word_acceptance"];
            assert_eq!(known_start["state"].as_str(), Some("PASS"));
            assert!(!known_start["test"]
                .as_str()
                .expect("KnownStart functional test reference")
                .trim()
                .is_empty());
            continue;
        }

        let path = manifest_dir.join(baseline["path"].as_str().expect("protected baseline path"));
        let path: PathBuf = path
            .canonicalize()
            .unwrap_or_else(|error| panic!("protected path {path:?}: {error}"));
        let expected_sha = baseline["expect"]["sha256"]
            .as_str()
            .expect("protected baseline sha256");
        let expected_mode = u32::from_str_radix(
            baseline["expect"]["mode"]
                .as_str()
                .expect("protected baseline mode"),
            8,
        )
        .expect("octal protected mode");

        assert_eq!(sha256(&path), expected_sha, "protected bytes: {path:?}");
        assert_eq!(
            std::fs::metadata(&path)
                .expect("protected metadata")
                .permissions()
                .mode()
                & 0o7777,
            expected_mode,
            "protected mode: {path:?}"
        );
    }
}

#[test]
fn td113_glued_phrase_reuses_the_single_generation_aware_material_cache() {
    let memo = read("src/ru_typo/memo.rs");
    assert!(memo.contains("const WORD_MATERIAL_CACHE_CAPACITY: usize = 512;"));
    assert_eq!(
        memo.matches("static CACHE: OnceLock<Mutex<WordMaterialCache>>")
            .count(),
        1,
        "TD-113 must extend the existing bounded cache instead of adding another owner"
    );
    assert!(
        memo.contains("GluedPhrase"),
        "glued-phrase material must have a typed cache key"
    );
    assert!(memo.contains("candidate_material_generation()"));

    let ru_typo = read("src/ru_typo.rs");
    assert!(ru_typo.contains("pub(crate) use memo::{memoized_text, WordMaterialKind};"));

    let glued = read("src/phrase_reader/glued_phrase.rs");
    let wrapper = section(
        &glued,
        "pub fn correct_glued_russian_phrase(word: &str) -> Option<String> {",
        "fn correct_glued_russian_phrase_uncached",
    );
    assert!(wrapper.contains("crate::ru_typo::memoized_text"));
    assert!(wrapper.contains("crate::ru_typo::WordMaterialKind::GluedPhrase"));
    assert!(wrapper.contains("correct_glued_russian_phrase_uncached"));

    for forbidden in [
        "CorrectionDecision",
        "CandidateDecision",
        "CorrectionResolution",
        "AuthorizationDecision",
        "MutationReceipt",
    ] {
        assert!(
            !memo.contains(forbidden),
            "material cache must not cache final decision or authority type {forbidden}"
        );
    }
}

#[test]
fn td113_proposal_only_substitution_has_an_explicit_non_apply_cap() {
    let sources = read("src/correction_core/candidate_sources.rs");
    let producer = section(
        &sources,
        "fn proposal_only_substitution_competitor",
        "fn repeated_prefix_composite_word",
    );

    assert!(producer.contains("CandidateGateAction::SuggestOnly"));
    assert!(producer.contains("proposal_only_substitution_competitor"));
    assert!(!producer.contains("admit_candidate_proposal"));
    assert!(!producer.contains("CandidateGateAction::Eligible"));
}

#[test]
fn td113_boundary_material_reuses_the_bounded_l2_readout_cache_owner() {
    let l2 = read("src/nanda_wave/l2.rs");
    let boundary = section(
        &l2,
        "pub fn ime_l2_boundary_candidates",
        "pub(crate) fn ime_l2_boundary_evidence",
    );
    assert!(boundary.contains("cached_boundary_candidates"));
    assert!(boundary.contains("ime_l2_boundary_candidates_uncached"));

    let readout = read("src/nanda_wave/l2/ime_readout.rs");
    assert!(readout.contains("const BOUNDARY_READOUT_CACHE_CAPACITY: usize = 128;"));
    assert_eq!(
        readout
            .matches("static READOUT_MATERIAL_CACHE: OnceLock<Mutex<ReadoutMaterialCache>>")
            .count(),
        1,
        "near-surface and boundary material must share the existing L2 readout cache owner"
    );
    let cache = section(
        &readout,
        "struct BoundaryReadoutCacheEntry",
        "enum LexicalReadoutMode",
    );
    for required in [
        "generation: u64",
        "context_prefix: String",
        "token: String",
        "limit: usize",
        "Vec<L2ImeWordCandidate>",
        "cached_boundary_candidates_with_generation",
        "candidate_material_generation",
    ] {
        assert!(
            cache.contains(required),
            "missing boundary cache contract {required:?}"
        );
    }
    assert!(
        cache.matches("if generation_now() != generation").count() >= 3,
        "boundary cache must revalidate the material generation before every return/publish boundary"
    );
    for forbidden in [
        "CorrectionDecision",
        "CandidateGate",
        "AuthorizationDecision",
        "EditPlan",
        "MutationReceipt",
    ] {
        assert!(
            !cache.contains(forbidden),
            "boundary material cache must not retain authority type {forbidden}"
        );
    }

    let canonical_cache = read("src/nanda_wave/l2_field/cache.rs");
    assert!(canonical_cache.contains("boundary candidates, L3/L4 evidence and final"));
    assert!(!canonical_cache.contains("BoundaryReadoutCacheEntry"));
}

#[test]
fn td113_boundary_competitor_veto_is_complete_and_generation_independent() {
    let adapter = read("src/nanda_wave/l2/tail_scan_adapter.rs");
    let structural_evidence = section(
        &adapter,
        "pub(super) fn boundary_split_target_has_structural_evidence",
        "fn independent_boundary_center",
    );
    let structural_evidence = without_whitespace(structural_evidence);
    for required in [
        "left_function&&independent_boundary_center(right)",
        "right_function&&independent_boundary_center(left)",
        "||short_left_field",
        "||two_content_centers",
    ] {
        assert!(
            structural_evidence.contains(required),
            "missing target-bound structural evidence term {required:?}"
        );
    }
    assert!(!structural_evidence.contains("has_clean_single_damerau_edit_candidate"));
    assert!(!structural_evidence.contains("has_competing_known_single_token_repair"));

    let admission = read("src/typing_transition/decision/admission.rs");
    let competitor = section(
        &admission,
        "fn exact_boundary_split_has_clean_single_token_repair",
        "fn verified_zero_loss_boundary_evidence",
    );
    assert!(competitor.contains("has_clean_single_damerau_edit_candidate"));
    for forbidden in [
        "boundary_fuzzy_candidates",
        "l2_center_near_surfaces",
        "candidate_material_generation",
        ".take(",
        "truncate(",
    ] {
        assert!(
            !competitor.contains(forbidden),
            "whole-word competitor veto depends on incomplete material {forbidden:?}"
        );
    }

    let generator = read("src/russian_typo_candidates.rs");
    let frontier = section(
        &generator,
        "pub(crate) fn any_single_damerau_edit_candidate",
        "pub(crate) fn repeated_run_deletion_candidates",
    );
    assert!(frontier.contains("for removed in 0..chars.len()"));
    assert!(frontier.contains("transposed.swap(left, left + 1)"));
    assert!(frontier.contains("for replacement in RU_ALPHABET"));
    assert!(frontier.contains("for inserted in RU_ALPHABET"));
    assert!(!frontier.contains("candidate_material_generation"));
    assert!(!frontier.contains(".take("));
    assert!(!frontier.contains("truncate("));

    let authority = section(
        &admission,
        "let lexical_boundary_split_authority =",
        "let verified_boundary_transition =",
    );
    assert!(authority.contains("exact_current_token_function_word_split"));
    assert!(authority.contains("verified_l2_boundary_target_grounding"));
    assert!(authority.contains("exact_boundary_split_has_clean_single_token_repair"));
    assert!(authority.contains("context_state_support"));
    assert!(authority.contains("signals.l3_pairwise_certified"));
    assert!(authority.contains("exact_positive_transition"));
    assert!(!admission.contains("verified_function_word_boundary_dominates"));

    let readout = read("src/nanda_wave/l2/ime_readout.rs");
    let cached_boundary = section(
        &readout,
        "fn cached_boundary_candidates_with_generation",
        "enum LexicalReadoutMode",
    );
    let cached_boundary = without_whitespace(cached_boundary);
    assert!(cached_boundary.contains(
        "ifgeneration_now()!=generation{returnVec::new();}returncandidates.as_ref().clone();"
    ));
}
