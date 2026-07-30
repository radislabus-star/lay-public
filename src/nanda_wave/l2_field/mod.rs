pub(crate) mod bridge;
mod compiler;
mod context;
mod format;
mod model;
mod proof;
mod runtime;
mod teacher;

pub(crate) use bridge::{cold_probe_surfaces, shadow_text_candidates, shadow_text_readout};
pub(crate) use runtime::L2FieldAuthority;

const DEFAULT_L2_MODEL_DIR_SUFFIX: &str = ".local/share/lay/nanda_wave/l2";
const DEFAULT_L2_PACKAGE_NAME: &str = "LAY-L2-RU-FULL-v7.bin";

pub fn default_l2_model_dir() -> std::path::PathBuf {
    if let Some(explicit) = std::env::var_os("LAY_L2_MODEL_DIR") {
        return explicit.into();
    }
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .map(|home| home.join(DEFAULT_L2_MODEL_DIR_SUFFIX))
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_L2_MODEL_DIR_SUFFIX))
}

pub fn discover_installed_l2_package() -> std::io::Result<Option<std::path::PathBuf>> {
    if let Some(explicit) = std::env::var_os("LAY_L2_PACKAGE") {
        let path = std::path::PathBuf::from(explicit);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "LAY_L2_PACKAGE points to a missing package: {}",
                path.display()
            ),
        ));
    }
    let path = default_l2_model_dir().join(DEFAULT_L2_PACKAGE_NAME);
    Ok(path.is_file().then_some(path))
}

fn installed_l2_field() -> Result<&'static runtime::StandaloneL2Field, &'static str> {
    static FIELD: std::sync::OnceLock<Result<runtime::StandaloneL2Field, String>> =
        std::sync::OnceLock::new();
    FIELD
        .get_or_init(|| {
            let path = discover_installed_l2_package()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "canonical L2 package is not installed".to_string())?;
            runtime::StandaloneL2Field::load(&path)
        })
        .as_ref()
        .map_err(String::as_str)
}

pub fn canonical_l2_status() -> serde_json::Value {
    let package = discover_installed_l2_package()
        .ok()
        .flatten()
        .map(|path| path.display().to_string());
    match installed_l2_field() {
        Ok(field) => {
            let (forms, l1_bound_forms, lemmas, bindings, competition_edges, decoder_bytes) =
                field.package_counts();
            serde_json::json!({
                "status": "ready",
                "package": package,
                "l1_package_fingerprint": field.l1_package_fingerprint(),
                "forms": forms,
                "l1_bound_forms": l1_bound_forms,
                "l2_materialized_forms": forms.saturating_sub(l1_bound_forms),
                "decoder_bytes": decoder_bytes,
                "lemmas": lemmas,
                "morph_bindings": bindings,
                "competition_edges": competition_edges,
            })
        }
        Err(error) => serde_json::json!({
            "status": "unavailable",
            "package": package,
            "message": error,
        }),
    }
}

pub fn prove_canonical_l2_package(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    limit: usize,
) -> std::io::Result<serde_json::Value> {
    proof::prove_package(
        l1_package_path,
        l2_package_path,
        morphology_corpus_path,
        limit,
    )
}

pub fn query_canonical_l2_package(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    context: &str,
    seed_surfaces: &[String],
    limit: usize,
) -> std::io::Result<serde_json::Value> {
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let field = runtime::StandaloneL2Field::load(l2_package_path).map_err(std::io::Error::other)?;
    if field.l1_package_fingerprint() != l1.corpus_fingerprint() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "L2 package was compiled for a different L1.1 corpus fingerprint",
        ));
    }
    let resolved_seeds = seed_surfaces
        .iter()
        .filter_map(|surface| Some((surface, l1.terminal_for_exact_surface(surface)?)))
        .collect::<Vec<_>>();
    let seeds = resolved_seeds
        .iter()
        .map(|(_, terminal_id)| runtime::L2LexicalSeed {
            terminal_id: Some(*terminal_id),
            surface: None,
            evidence_milli: 1_000,
        })
        .collect::<Vec<_>>();
    let readout = field.readout(context, &seeds, limit.max(1));
    let verdict = match &readout.verdict {
        runtime::L2LocalVerdict::Winner { form_ref } => serde_json::json!({
            "kind": "winner",
            "form_refs": [form_ref],
        }),
        runtime::L2LocalVerdict::Tied { form_refs } => serde_json::json!({
            "kind": "tied",
            "form_refs": form_refs,
        }),
        runtime::L2LocalVerdict::Abstain => serde_json::json!({
            "kind": "abstain",
            "form_refs": [],
        }),
    };
    Ok(serde_json::json!({
        "kind": "canonical_l2_query",
        "l1_package": l1_package_path,
        "l2_package": l2_package_path,
        "context": context,
        "requested_seed_surfaces": seed_surfaces,
        "resolved_seeds": resolved_seeds.iter().map(|(surface, terminal_id)| serde_json::json!({
            "surface": surface,
            "terminal_id": terminal_id,
        })).collect::<Vec<_>>(),
        "verdict": verdict,
        "context_mode_id": readout.context_mode_id,
        "candidates": readout.candidates.iter().map(|candidate| serde_json::json!({
            "form_ref": candidate.form_ref,
            "surface": candidate.surface,
            "l1_terminal_id": candidate.l1_terminal_id,
            "l1_evidence_milli": candidate.l1_evidence_milli,
            "slot_phase_milli": candidate.slot_phase_milli,
            "neighbor_pressure": candidate.neighbor_pressure,
            "competition_pressure": candidate.competition_pressure,
            "explicit_competition_pressure": candidate.explicit_competition_pressure,
            "local_score": candidate.local_score,
            "lemma_ids": candidate.lemma_ids,
            "feature_masks": candidate.feature_masks,
        })).collect::<Vec<_>>(),
        "runtime_authority_changed": false,
    }))
}

pub fn export_unseeded_l11_seed_corpus(
    l1_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    output_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let source = std::fs::read_to_string(morphology_corpus_path)?;
    let corpus = teacher::L2TeacherCorpus::parse_tsv(&source).map_err(std::io::Error::other)?;
    let unique_surfaces = corpus
        .forms
        .iter()
        .map(|form| form.surface.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let (terminal_map, resolver_workers) = resolve_l1_terminals(&l1, &unique_surfaces);
    let mut unseeded_lemmas = 0_usize;
    let mut seed_surfaces = std::collections::BTreeSet::<String>::new();
    let mut start = 0_usize;
    while start < corpus.forms.len() {
        let lemma = corpus.forms[start].lemma.as_str();
        let mut end = start + 1;
        while end < corpus.forms.len() && corpus.forms[end].lemma == lemma {
            end += 1;
        }
        let forms = &corpus.forms[start..end];
        if !forms
            .iter()
            .any(|form| terminal_map.contains_key(&form.surface))
        {
            unseeded_lemmas += 1;
            let seed = forms
                .iter()
                .min_by_key(|form| {
                    (
                        form.surface != form.lemma,
                        form.surface.chars().count(),
                        form.surface.as_str(),
                    )
                })
                .expect("non-empty lemma form group");
            seed_surfaces.insert(seed.surface.clone());
        }
        start = end;
    }
    let mut bytes = seed_surfaces
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n")
        .into_bytes();
    if !bytes.is_empty() {
        bytes.push(b'\n');
    }
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = output_path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temporary, &bytes)?;
    std::fs::rename(&temporary, output_path)?;
    Ok(serde_json::json!({
        "kind": "l2_unseeded_l11_seed_corpus_export",
        "l1_package": l1_package_path,
        "l1_corpus_fingerprint": l1.corpus_fingerprint(),
        "morphology_corpus": morphology_corpus_path,
        "source_forms": corpus.forms.len(),
        "source_unique_surfaces": unique_surfaces.len(),
        "source_lemmas": corpus.forms.iter().map(|form| form.lemma.as_str()).collect::<std::collections::BTreeSet<_>>().len(),
        "l1_bound_surfaces": terminal_map.len(),
        "unseeded_lemmas": unseeded_lemmas,
        "delta_seed_surfaces": seed_surfaces.len(),
        "resolver_workers": resolver_workers,
        "output": output_path,
        "output_bytes": bytes.len(),
        "runtime_authority_changed": false,
    }))
}

pub fn compile_canonical_l2_package(
    l1_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    output_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let l1_fingerprint = l1.corpus_fingerprint();
    let l1_terminal_count = u64::from(l1.terminal_count());
    let source = std::fs::read_to_string(morphology_corpus_path)?;
    let corpus = teacher::L2TeacherCorpus::parse_tsv(&source).map_err(std::io::Error::other)?;
    let unique_surfaces = corpus
        .forms
        .iter()
        .map(|form| form.surface.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let reusable_terminals = std::fs::read(output_path)
        .ok()
        .and_then(|bytes| format::decode_package(&bytes).ok())
        .filter(|package| {
            package.l1_package_fingerprint == l1_fingerprint
                && package.form_refs.len() == unique_surfaces.len()
        })
        .map(|package| {
            unique_surfaces
                .iter()
                .cloned()
                .zip(
                    package
                        .form_refs
                        .into_iter()
                        .map(|form| form.l1_terminal_id),
                )
                .filter(|(_, terminal_id)| *terminal_id != model::NO_L1_TERMINAL)
                .collect::<std::collections::BTreeMap<_, _>>()
        });
    let terminal_map = if let Some(terminals) = reusable_terminals.as_ref() {
        terminals.clone()
    } else {
        resolve_l1_terminals(&l1, &unique_surfaces).0
    };
    let resolver_workers = if reusable_terminals.is_some() {
        0
    } else {
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .min(unique_surfaces.len().max(1))
    };
    let (package, report) = compiler::compile_l2_package(&corpus, l1_fingerprint, |surface| {
        terminal_map.get(surface).copied()
    })
    .map_err(std::io::Error::other)?;
    let bytes = format::encode_package(&package).map_err(std::io::Error::other)?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = output_path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temporary, &bytes)?;
    std::fs::rename(&temporary, output_path)?;
    Ok(serde_json::json!({
        "kind": "canonical_l2_compile",
        "l1_package": l1_package_path,
        "l1_corpus_fingerprint": l1_fingerprint,
        "l1_terminal_count": l1_terminal_count,
        "terminal_map_reused": reusable_terminals.is_some(),
        "terminal_resolver_workers": if reusable_terminals.is_some() { 0 } else { resolver_workers },
        "morphology_corpus": morphology_corpus_path,
        "output": output_path,
        "package_bytes": bytes.len(),
        "source_bindings": report.source_forms,
        "source_forms": report.source_forms,
        "source_unique_surfaces": report.source_unique_surfaces,
        "source_lemmas": report.source_lemmas,
        "admitted_forms": report.admitted_forms,
        "l1_bound_forms": report.l1_bound_forms,
        "l2_materialized_forms": report.admitted_forms.saturating_sub(report.l1_bound_forms),
        "missing_unique_surfaces": report.missing_l1_forms,
        "missing_l1_forms": report.missing_l1_forms,
        "unseeded_lemmas": report.unseeded_lemmas,
        "decoder_bytes": package.decoder_bytes.len(),
        "lemma_centers": report.lemma_centers,
        "morph_bindings": report.morph_bindings,
        "context_modes": report.context_modes,
        "slot_centers": report.slot_centers,
        "neighbor_couplings": report.neighbor_couplings,
        "competition_edges": report.competition_edges,
        "train_scenes": report.train_scenes,
        "heldout_scenes": report.heldout_scenes,
        "runtime_authority_changed": false,
    }))
}

fn resolve_l1_terminals(
    l1: &crate::nanda_wave::L1RestorationHost,
    unique_surfaces: &std::collections::BTreeSet<String>,
) -> (std::collections::BTreeMap<String, u32>, usize) {
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(unique_surfaces.len().max(1));
    let surfaces = unique_surfaces.iter().cloned().collect::<Vec<_>>();
    let chunk_size = surfaces.len().div_ceil(workers);
    let terminals = std::thread::scope(|scope| {
        let handles = surfaces
            .chunks(chunk_size.max(1))
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .filter_map(|surface| {
                            Some((surface.clone(), l1.terminal_for_exact_surface(surface)?))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("L2 terminal resolver worker"))
            .collect::<std::collections::BTreeMap<_, _>>()
    });
    (terminals, workers)
}
