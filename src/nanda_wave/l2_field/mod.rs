pub(crate) mod bridge;
mod compiler;
mod context;
mod format;
mod model;
mod proof;
mod runtime;
mod teacher;

pub(crate) use bridge::{cold_probe_surfaces, shadow_text_candidates};

const DEFAULT_L2_MODEL_DIR_SUFFIX: &str = ".local/share/lay/nanda_wave/l2";
const DEFAULT_L2_PACKAGE_NAME: &str = "LAY-L2-RU-FULL-v4.bin";

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
            let (forms, lemmas, bindings, competition_edges) = field.package_counts();
            serde_json::json!({
                "status": "ready",
                "package": package,
                "l1_package_fingerprint": field.l1_package_fingerprint(),
                "forms": forms,
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

pub fn compile_canonical_l2_package(
    l1_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    output_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let l1_header = crate::nanda_wave::inspect_l1_package_header(l1_package_path)?;
    let l1_fingerprint = l1_header
        .get("corpus_fingerprint")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| std::io::Error::other("L1.1 header omitted corpus fingerprint"))?;
    let l1_terminal_count = l1_header
        .get("terminal_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
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
                .collect::<std::collections::BTreeMap<_, _>>()
        });
    let resolver_workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(unique_surfaces.len().max(1));
    let terminal_map = if let Some(terminals) = reusable_terminals.as_ref() {
        terminals.clone()
    } else {
        let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
        let surfaces = unique_surfaces.into_iter().collect::<Vec<_>>();
        let chunk_size = surfaces.len().div_ceil(resolver_workers);
        std::thread::scope(|scope| {
            let handles = surfaces
                .chunks(chunk_size.max(1))
                .map(|chunk| {
                    let l1 = &l1;
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
        })
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
        "admitted_forms": report.admitted_forms,
        "missing_unique_surfaces": report.missing_l1_forms,
        "missing_l1_forms": report.missing_l1_forms,
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
