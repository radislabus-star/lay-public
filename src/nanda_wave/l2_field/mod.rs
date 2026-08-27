pub(crate) mod bridge;
mod cache;
mod compact_format;
mod compiler;
mod compositional;
mod compositional_format;
mod compositional_proof;
mod context;
mod contextual_compositional_proof;
mod format;
mod model;
mod package_bytes;
mod productive;
mod productive_format;
mod productive_proof;
mod productive_v1;
mod proof;
mod runtime;
mod runtime_storage;
mod teacher;
mod v13_typed_peak;

pub(crate) use bridge::{
    canonical_ime_candidates_observed, canonical_text_candidates, canonical_text_readout_observed,
    canonical_text_readout_observed_with_frame, cold_probe_surfaces,
};
pub(crate) use runtime::CanonicalFieldTelemetry;
pub(crate) use runtime::L2FieldAuthority;
pub(crate) use runtime::L2FieldAvailability;

pub const CANONICAL_L2_LEMMA_FRONTIER: usize = 256;
pub const CANONICAL_L2_ACTIVE_LEMMA_LIMIT: usize = 256;
pub const CANONICAL_L2_FEATURE_LIMIT: usize = 16;
pub const CANONICAL_L2_FORM_LIMIT: usize = 32;
pub const CANONICAL_L2_ATOM_RELATION_LIMIT: usize = 196_608;
pub const CANONICAL_L2_PRODUCTIVE_LEMMA_LIMIT: usize = 32;
pub const CANONICAL_L2_PRODUCTIVE_FORM_LIMIT: usize = 8;

const DEFAULT_L2_MODEL_DIR_SUFFIX: &str = ".local/share/lay/nanda_wave/l2";
const DEFAULT_L2_PACKAGE_NAME: &str = "LAY-L2-RU-FULL-v13.bin";
const DEFAULT_PRODUCTIVE_L2_PACKAGE_NAME: &str = "LAY-L2-RU-PRODUCTIVE-v1.bin";
const DEFAULT_PRODUCTIVE_L2_V1_PACKAGE_NAME: &str = "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m";
const DEFAULT_EXACT_V13_SIDECAR_NAME: &str = "LAY-L2-RU-FULL-v13.dafsa";

fn productive_sidecar_state() -> &'static std::sync::RwLock<
    Option<std::sync::Arc<productive_format::CompactProductiveMorphologyView>>,
> {
    static STATE: std::sync::OnceLock<
        std::sync::RwLock<
            Option<std::sync::Arc<productive_format::CompactProductiveMorphologyView>>,
        >,
    > = std::sync::OnceLock::new();
    STATE.get_or_init(|| std::sync::RwLock::new(None))
}

type ProductiveV1LoadResult =
    Result<std::sync::Arc<productive_v1::PackagedProductiveRuntimeV1>, String>;

fn load_cached_generation<T: Clone, E: Clone>(
    state: &std::sync::Mutex<Option<Result<T, E>>>,
    loader: impl FnOnce() -> Result<T, E>,
) -> Result<(Result<T, E>, bool), &'static str> {
    let mut state = state.lock().map_err(|_| "runtime lock poisoned")?;
    if let Some(cached) = state.as_ref() {
        return Ok((cached.clone(), false));
    }
    let loaded = loader();
    *state = Some(loaded.clone());
    Ok((loaded, true))
}

fn productive_v1_state() -> &'static std::sync::Mutex<Option<ProductiveV1LoadResult>> {
    static STATE: std::sync::OnceLock<std::sync::Mutex<Option<ProductiveV1LoadResult>>> =
        std::sync::OnceLock::new();
    STATE.get_or_init(|| std::sync::Mutex::new(None))
}

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

pub fn discover_installed_productive_l2_sidecar() -> std::io::Result<Option<std::path::PathBuf>> {
    if let Some(explicit) = std::env::var_os("LAY_L2_PRODUCTIVE_PACKAGE") {
        let path = std::path::PathBuf::from(explicit);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "LAY_L2_PRODUCTIVE_PACKAGE points to a missing package: {}",
                path.display()
            ),
        ));
    }
    let path = default_l2_model_dir().join(DEFAULT_PRODUCTIVE_L2_PACKAGE_NAME);
    Ok(path.is_file().then_some(path))
}

pub fn discover_installed_productive_l2_v1_package() -> std::io::Result<Option<std::path::PathBuf>>
{
    if let Some(explicit) = std::env::var_os("LAY_L2_PRODUCTIVE_V1_PACKAGE") {
        let path = std::path::PathBuf::from(explicit);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "LAY_L2_PRODUCTIVE_V1_PACKAGE points to a missing package: {}",
                path.display()
            ),
        ));
    }
    let path = default_l2_model_dir().join(DEFAULT_PRODUCTIVE_L2_V1_PACKAGE_NAME);
    Ok(path.is_file().then_some(path))
}

pub fn discover_installed_exact_v13_sidecar() -> std::io::Result<Option<std::path::PathBuf>> {
    if let Some(explicit) = std::env::var_os("LAY_L2_V13_DAFSA") {
        let path = std::path::PathBuf::from(explicit);
        if path.is_file() {
            return Ok(Some(path));
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "LAY_L2_V13_DAFSA points to a missing sidecar: {}",
                path.display()
            ),
        ));
    }
    let path = default_l2_model_dir().join(DEFAULT_EXACT_V13_SIDECAR_NAME);
    Ok(path.is_file().then_some(path))
}

type ExactV13LoadResult =
    Result<Option<std::sync::Arc<v13_typed_peak::ExactV13Generation>>, String>;

fn exact_v13_state() -> &'static std::sync::OnceLock<ExactV13LoadResult> {
    static STATE: std::sync::OnceLock<ExactV13LoadResult> = std::sync::OnceLock::new();
    &STATE
}

fn installed_exact_v13(canonical_index: &runtime::StandaloneL2Field) -> ExactV13LoadResult {
    exact_v13_state()
        .get_or_init(|| load_exact_v13(canonical_index))
        .clone()
}

fn load_exact_v13(canonical_index: &runtime::StandaloneL2Field) -> ExactV13LoadResult {
    let Some(sidecar_path) =
        discover_installed_exact_v13_sidecar().map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };
    let canonical_path = discover_installed_l2_package()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "canonical L2 package is not installed".to_string())?;
    let canonical_bytes = std::fs::metadata(&canonical_path)
        .map_err(|error| format!("{}: {error}", canonical_path.display()))?
        .len();
    let canonical_sha256 = sha256_file(&canonical_path)?;
    let (forms, _, _, bindings, _, _) = canonical_index.package_counts();
    v13_typed_peak::ExactV13Generation::load(
        &sidecar_path,
        canonical_sha256,
        canonical_bytes,
        forms,
        bindings,
    )
    .map(std::sync::Arc::new)
    .map(Some)
}

fn installed_productive_l2_v1(
) -> Result<std::sync::Arc<productive_v1::PackagedProductiveRuntimeV1>, String> {
    // A package admission failure is a stable generation result, not a reason
    // to re-read and re-hash every model file on each keystroke. Explicit
    // reload replaces this cached result after package installation or repair.
    let (loaded, loaded_now) = load_cached_generation(productive_v1_state(), load_productive_l2_v1)
        .map_err(|_| "productive V1 runtime lock poisoned")?;
    if loaded_now && loaded.is_ok() {
        clear_productive_runtime_dependents();
    }
    loaded
}

fn reload_productive_l2_v1_inner(
) -> Result<std::sync::Arc<productive_v1::PackagedProductiveRuntimeV1>, String> {
    let loaded = load_productive_l2_v1();
    *productive_v1_state()
        .lock()
        .map_err(|_| "productive V1 runtime lock poisoned")? = Some(loaded.clone());
    clear_productive_runtime_dependents();
    loaded
}

#[cfg(test)]
mod productive_runtime_cache_tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use super::load_cached_generation;

    #[test]
    fn failed_admission_is_cached_until_explicit_reload() {
        let state = Mutex::new(None);
        let attempts = AtomicUsize::new(0);
        let load = || {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err::<usize, _>("fingerprint mismatch".to_string())
        };

        let (first, first_loaded) = load_cached_generation(&state, load).expect("first load");
        let (second, second_loaded) = load_cached_generation(&state, load).expect("cached load");

        assert_eq!(first, Err("fingerprint mismatch".to_string()));
        assert_eq!(second, first);
        assert!(first_loaded);
        assert!(!second_loaded);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}

fn load_productive_l2_v1(
) -> Result<std::sync::Arc<productive_v1::PackagedProductiveRuntimeV1>, String> {
    let productive_path = discover_installed_productive_l2_v1_package()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "productive L2 V1 package is not installed".to_string())?;
    let l11_path = crate::nanda_wave::discover_installed_l11_package()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "L1.1 package is not installed".to_string())?
        .artifact_path;
    let canonical_l2_path = discover_installed_l2_package()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "canonical L2 package is not installed".to_string())?;
    let l11_sha256 = sha256_file(&l11_path)?;
    let canonical_l2_sha256 = sha256_file(&canonical_l2_path)?;
    let runtime = std::sync::Arc::new(
        productive_v1::PackagedProductiveRuntimeV1::load_with_semantic_transducer(
            &productive_path,
            l11_sha256,
            canonical_l2_sha256,
        )?,
    );
    Ok(runtime)
}

fn clear_productive_runtime_dependents() {
    cache::clear();
    bridge::clear_prepared_field_cache();
    crate::nanda_wave::candidate_gate::clear_live_completion_cache();
}

pub fn candidate_material_generation() -> u64 {
    cache::stats().generation
}

pub fn reload_productive_l2_v1() -> serde_json::Value {
    match reload_productive_l2_v1_inner() {
        Ok(runtime) => serde_json::json!({
            "status": "reloaded_live_owner",
            "package_bytes": runtime.package_bytes(),
            "recovery_package_bytes": runtime.anchor_recovery_package_bytes(),
            "resident_cache_bytes": runtime.resident_cache_bytes(),
            "mmap_backed": runtime.mmap_backed(),
            "package_sha256": hex_sha256(runtime.package_sha256()),
            "runtime_authority_changed": true,
        }),
        Err(error) => serde_json::json!({
            "status": "unavailable",
            "message": error,
            "runtime_authority_changed": false,
        }),
    }
}

pub fn productive_l2_v1_status() -> serde_json::Value {
    let package = discover_installed_productive_l2_v1_package()
        .ok()
        .flatten()
        .map(|path| path.display().to_string());
    match installed_productive_l2_v1() {
        Ok(runtime) => serde_json::json!({
            "status": "ready_live_owner",
            "package": package,
            "package_bytes": runtime.package_bytes(),
            "recovery_package_bytes": runtime.anchor_recovery_package_bytes(),
            "recovery_paths": runtime.anchor_recovery_path_count(),
            "resident_cache_bytes": runtime.resident_cache_bytes(),
            "mmap_backed": runtime.mmap_backed(),
            "package_sha256": hex_sha256(runtime.package_sha256()),
            "route": "L1.1 bounded lattice -> Productive V90 L2 -> common L3 -> DecisionCore -> verifier",
            "runtime_authority_changed": true,
        }),
        Err(error) => serde_json::json!({
            "status": "unavailable",
            "package": package,
            "message": error,
            "runtime_authority_changed": false,
        }),
    }
}

fn installed_productive_l2_sidecar(
) -> Result<std::sync::Arc<productive_format::CompactProductiveMorphologyView>, String> {
    if let Some(view) = productive_sidecar_state()
        .read()
        .map_err(|_| "productive sidecar lock poisoned")?
        .clone()
    {
        return Ok(view);
    }
    reload_productive_l2_sidecar_inner()
}

fn reload_productive_l2_sidecar_inner(
) -> Result<std::sync::Arc<productive_format::CompactProductiveMorphologyView>, String> {
    let path = discover_installed_productive_l2_sidecar()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "productive L2 sidecar is not installed".to_string())?;
    let view = std::sync::Arc::new(productive_format::CompactProductiveMorphologyView::load(
        &path,
    )?);
    if let Ok(field) = installed_l2_field() {
        if view.l2_fingerprint() != field.l1_package_fingerprint() {
            return Err(
                "productive sidecar L1.1 fingerprint does not match canonical L2".to_string(),
            );
        }
    }
    *productive_sidecar_state()
        .write()
        .map_err(|_| "productive sidecar lock poisoned")? = Some(view.clone());
    cache::clear();
    bridge::clear_prepared_field_cache();
    Ok(view)
}

pub fn reload_productive_l2_sidecar() -> serde_json::Value {
    match reload_productive_l2_sidecar_inner() {
        Ok(view) => serde_json::json!({
            "status": "reloaded",
            "backing_bytes": view.backing_bytes(),
            "mmap_backed": view.mmap_backed(),
            "l2_package_fingerprint": view.l2_fingerprint(),
            "runtime_authority_changed": false,
        }),
        Err(error) => serde_json::json!({
            "status": "unavailable",
            "message": error,
            "runtime_authority_changed": false,
        }),
    }
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

pub(crate) fn surfaces_share_morphology_identity(left: &str, right: &str) -> bool {
    let Ok(field) = installed_l2_field() else {
        return false;
    };
    let (Some(left_ref), Some(right_ref)) = (
        field.form_ref_for_surface(left),
        field.form_ref_for_surface(right),
    ) else {
        return false;
    };
    let left_lemmas = field
        .imported_binding_identities_for_form(left_ref)
        .into_iter()
        .map(|(lemma_id, _)| lemma_id)
        .collect::<std::collections::BTreeSet<_>>();
    field
        .imported_binding_identities_for_form(right_ref)
        .into_iter()
        .any(|(lemma_id, _)| left_lemmas.contains(&lemma_id))
}

pub(crate) fn morphology_slot_identities_for_surface(
    surface: &str,
) -> Vec<crate::correction_core::MorphologySlotIdentity> {
    let Ok(field) = installed_l2_field() else {
        return Vec::new();
    };
    let Some(form_ref) = field.form_ref_for_surface(surface) else {
        return Vec::new();
    };
    let mut identities = field
        .imported_binding_identities_for_form(form_ref)
        .into_iter()
        .map(
            |(lemma_id, feature_mask)| crate::correction_core::MorphologySlotIdentity {
                domain: crate::correction_core::MorphologySlotIdentityDomain::CanonicalFeature,
                lemma_id,
                slot_id: feature_mask,
            },
        )
        .collect::<Vec<_>>();
    identities.sort_unstable();
    identities.dedup();
    identities
}

pub(crate) fn canonical_form_contains_surface(surface: &str) -> bool {
    installed_l2_field()
        .ok()
        .and_then(|field| field.form_ref_for_surface(surface))
        .is_some()
}

pub(crate) fn warm_up_installed_l2_field() {
    // Loading and indexing the standalone package can take hundreds of
    // milliseconds. Keep that first touch on the existing background IME
    // warmup thread instead of charging it to the user's first Space.
    let _ = preload_installed_l2_field();
}

pub(in crate::nanda_wave::l2_field) fn preload_installed_l2_field() -> Result<(), String> {
    let canonical_index = installed_l2_field().map_err(str::to_string)?;
    installed_productive_l2_v1()?;
    let _ = installed_exact_v13(canonical_index);
    Ok(())
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
            let (package_storage, package_backing_bytes) = field.package_storage();
            serde_json::json!({
                "status": "ready",
                "package": package,
                "package_storage": package_storage,
                "package_backing_bytes": package_backing_bytes,
                "package_mmap_backed": field.package_mmap_backed(),
                "compositional_index_source": field.compositional_index_source(),
                "compositional_index_resident_bytes": field.compositional_index_bytes(),
                "compositional_index_view_bytes": field.compositional_index_view_bytes(),
                "compositional_limits": {
                    "lemma_frontier": CANONICAL_L2_LEMMA_FRONTIER,
                    "active_lemma_limit": CANONICAL_L2_ACTIVE_LEMMA_LIMIT,
                    "features_per_lemma": CANONICAL_L2_FEATURE_LIMIT,
                    "form_lattice": CANONICAL_L2_FORM_LIMIT,
                    "atom_relations": CANONICAL_L2_ATOM_RELATION_LIMIT,
                },
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

pub fn exact_v13_status() -> serde_json::Value {
    let sidecar = discover_installed_exact_v13_sidecar()
        .ok()
        .flatten()
        .map(|path| path.display().to_string());
    let canonical_index = match installed_l2_field() {
        Ok(field) => field,
        Err(error) => {
            return serde_json::json!({
                "status": "unavailable",
                "sidecar": sidecar,
                "message": error,
                "runtime_authority_changed": false,
            });
        }
    };
    match installed_exact_v13(canonical_index) {
        Ok(Some(generation)) => serde_json::json!({
            "status": "ready_immutable_owner",
            "sidecar": sidecar,
            "sidecar_bytes": generation.sidecar_bytes(),
            "sidecar_sha256": hex_sha256(generation.sidecar_sha256()),
            "typed_payload_bytes": generation.typed_payload_bytes(),
            "lifetime": "process",
            "reload": "process_restart_required",
            "runtime_authority_changed": false,
        }),
        Ok(None) => serde_json::json!({
            "status": "not_installed",
            "sidecar": sidecar,
            "runtime_authority_changed": false,
        }),
        Err(error) => serde_json::json!({
            "status": "unavailable",
            "sidecar": sidecar,
            "message": error,
            "runtime_authority_changed": false,
        }),
    }
}

pub fn compile_exact_v13_sidecar(
    canonical_l2_package: &std::path::Path,
    output: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    v13_typed_peak::compile_exact_sidecar_file(canonical_l2_package, output)
}

pub fn query_exact_v13_sidecar(
    canonical_l2_package: &std::path::Path,
    sidecar: &std::path::Path,
    observed: &str,
) -> std::io::Result<serde_json::Value> {
    v13_typed_peak::query_exact_sidecar_file(canonical_l2_package, sidecar, observed)
}

pub fn compile_productive_l2_sidecar(
    l2_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    output_path: &std::path::Path,
    minimum_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    if minimum_profile_support == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "productive sidecar profile support must be greater than zero",
        ));
    }
    let started = std::time::Instant::now();
    let field = runtime::StandaloneL2Field::load(l2_package_path).map_err(std::io::Error::other)?;
    let load_us = started.elapsed().as_micros() as u64;
    let training_started = std::time::Instant::now();
    let mut index = field
        .train_productive_morphology(|_| true, minimum_profile_support)
        .map_err(std::io::Error::other)?;
    index.train_context_slots_from_corpus(
        morphology_corpus_path,
        &std::collections::BTreeSet::new(),
    )?;
    let training_us = training_started.elapsed().as_micros() as u64;
    let report = index.report().clone();
    let encoding_started = std::time::Instant::now();
    let (bytes, stats) = productive_format::encode_index(&index, field.l1_package_fingerprint())
        .map_err(std::io::Error::other)?;
    let encoded_sha256 = sha256_hex(&bytes);
    let view = productive_format::CompactProductiveMorphologyView::from_bytes(bytes.clone())
        .map_err(std::io::Error::other)?;
    if view.l2_fingerprint() != field.l1_package_fingerprint()
        || view.report() != report
        || view.backing_bytes() != bytes.len()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "productive sidecar round-trip changed metadata",
        ));
    }
    let encoding_us = encoding_started.elapsed().as_micros() as u64;
    write_atomic(output_path, &bytes)?;
    Ok(serde_json::json!({
        "kind": "canonical_l2_productive_morphology_sidecar_v2",
        "verdict": "PASS_format_roundtrip",
        "l2_package": l2_package_path,
        "l2_package_fingerprint": field.l1_package_fingerprint(),
        "morphology_corpus": morphology_corpus_path,
        "minimum_profile_support": minimum_profile_support,
        "output": output_path,
        "output_bytes": bytes.len(),
        "output_sha256": encoded_sha256,
        "format_version": stats.version,
        "rules": stats.rules,
        "target_features": stats.target_features,
        "context_slots": stats.context_slots,
        "known_contexts": stats.known_contexts,
        "context_pairs": stats.context_pairs,
        "payload_bytes": stats.payload_bytes,
        "observed_lemmas": report.observed_lemmas,
        "admitted_lemmas": report.admitted_lemmas,
        "observed_transforms": report.observed_transforms,
        "admitted_profiles": report.admitted_profiles,
        "rejected_low_support_profiles": report.rejected_low_support_profiles,
        "observed_context_rows": report.observed_context_rows,
        "admitted_context_rows": report.admitted_context_rows,
        "rejected_context_rows": report.rejected_context_rows,
        "observed_competitor_rows": report.observed_competitor_rows,
        "observed_competitor_surfaces": report.observed_competitor_surfaces,
        "same_lemma_competitor_surfaces": report.same_lemma_competitor_surfaces,
        "admitted_pair_observations": report.admitted_pair_observations,
        "load_us": load_us,
        "training_us": training_us,
        "encoding_us": encoding_us,
        "peak_rss_kib": proc_status_kib("VmHWM:"),
        "runtime_authority_changed": false,
        "not_tested": [
            "generated-form parity against the trained index",
            "fixed restoration quality",
            "live L2/L3 integration",
            "daemon and IBus latency",
        ],
    }))
}

pub fn audit_productive_anchor_recovery_v1(
    axis_schema_path: &std::path::Path,
    work_root: &std::path::Path,
    scratch_root: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    productive_v1::audit_productive_anchor_recovery_v1(axis_schema_path, work_root, scratch_root)
        .map_err(std::io::Error::other)
}

pub fn estimate_productive_semantic_transducer_v1(
    productive_package_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    productive_v1::estimate_productive_semantic_transducer_v1(productive_package_path)
        .map_err(std::io::Error::other)
}

#[allow(clippy::too_many_arguments)]
pub fn estimate_productive_semantic_transducer_heldout_v1(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    productive_package_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_dir: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> std::io::Result<serde_json::Value> {
    productive_v1::estimate_productive_semantic_transducer_heldout_v1(
        l1_package_path,
        l2_package_path,
        productive_package_path,
        axis_schema_path,
        work_dir,
        heldout_per_class,
        requested_workers,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compile_productive_paradigm_field_v1(
    l11_package_path: &std::path::Path,
    canonical_l2_path: &std::path::Path,
    corpus_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_root: &std::path::Path,
    output_path: &std::path::Path,
    expected_corpus_sha256: &str,
    expected_corpus_bytes: u64,
    workers: usize,
) -> std::io::Result<serde_json::Value> {
    let expected_corpus_sha256 = parse_sha256_hex(expected_corpus_sha256)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut progress = |stage: &str| eprintln!("productive_v1 stage={stage}");
    productive_v1::compile_productive_paradigm_field_v1(
        &productive_v1::ProductiveOrchestratorConfigV1 {
            l11_package_path: l11_package_path.to_path_buf(),
            canonical_l2_path: canonical_l2_path.to_path_buf(),
            corpus_path: corpus_path.to_path_buf(),
            axis_schema_path: axis_schema_path.to_path_buf(),
            work_root: work_root.to_path_buf(),
            output_path: output_path.to_path_buf(),
            expected_corpus_sha256,
            expected_corpus_bytes,
            workers,
            shared_support_recovery: false,
        },
        &mut progress,
    )
    .map_err(std::io::Error::other)
}

#[allow(clippy::too_many_arguments)]
pub fn resume_productive_paradigm_field_v1(
    l11_package_path: &std::path::Path,
    canonical_l2_path: &std::path::Path,
    corpus_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_root: &std::path::Path,
    output_path: &std::path::Path,
    expected_corpus_sha256: &str,
    expected_corpus_bytes: u64,
    workers: usize,
) -> std::io::Result<serde_json::Value> {
    let expected_corpus_sha256 = parse_sha256_hex(expected_corpus_sha256)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut progress = |stage: &str| eprintln!("productive_v1 stage={stage}");
    productive_v1::resume_productive_paradigm_field_v1(
        &productive_v1::ProductiveOrchestratorConfigV1 {
            l11_package_path: l11_package_path.to_path_buf(),
            canonical_l2_path: canonical_l2_path.to_path_buf(),
            corpus_path: corpus_path.to_path_buf(),
            axis_schema_path: axis_schema_path.to_path_buf(),
            work_root: work_root.to_path_buf(),
            output_path: output_path.to_path_buf(),
            expected_corpus_sha256,
            expected_corpus_bytes,
            workers,
            shared_support_recovery: false,
        },
        &mut progress,
    )
    .map_err(std::io::Error::other)
}

#[allow(clippy::too_many_arguments)]
pub fn resume_productive_paradigm_field_v1_shared_support(
    l11_package_path: &std::path::Path,
    canonical_l2_path: &std::path::Path,
    corpus_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_root: &std::path::Path,
    output_path: &std::path::Path,
    expected_corpus_sha256: &str,
    expected_corpus_bytes: u64,
    workers: usize,
) -> std::io::Result<serde_json::Value> {
    let expected_corpus_sha256 = parse_sha256_hex(expected_corpus_sha256)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut progress = |stage: &str| eprintln!("productive_v1 stage={stage}");
    productive_v1::resume_productive_paradigm_field_v1(
        &productive_v1::ProductiveOrchestratorConfigV1 {
            l11_package_path: l11_package_path.to_path_buf(),
            canonical_l2_path: canonical_l2_path.to_path_buf(),
            corpus_path: corpus_path.to_path_buf(),
            axis_schema_path: axis_schema_path.to_path_buf(),
            work_root: work_root.to_path_buf(),
            output_path: output_path.to_path_buf(),
            expected_corpus_sha256,
            expected_corpus_bytes,
            workers,
            shared_support_recovery: true,
        },
        &mut progress,
    )
    .map_err(std::io::Error::other)
}

#[allow(clippy::too_many_arguments)]
pub fn reinduce_productive_paradigm_field_v1(
    l11_package_path: &std::path::Path,
    canonical_l2_path: &std::path::Path,
    corpus_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_root: &std::path::Path,
    output_path: &std::path::Path,
    expected_corpus_sha256: &str,
    expected_corpus_bytes: u64,
    workers: usize,
) -> std::io::Result<serde_json::Value> {
    let expected_corpus_sha256 = parse_sha256_hex(expected_corpus_sha256)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut progress = |stage: &str| eprintln!("productive_v1 stage={stage}");
    productive_v1::reinduce_productive_paradigm_field_v1(
        &productive_v1::ProductiveOrchestratorConfigV1 {
            l11_package_path: l11_package_path.to_path_buf(),
            canonical_l2_path: canonical_l2_path.to_path_buf(),
            corpus_path: corpus_path.to_path_buf(),
            axis_schema_path: axis_schema_path.to_path_buf(),
            work_root: work_root.to_path_buf(),
            output_path: output_path.to_path_buf(),
            expected_corpus_sha256,
            expected_corpus_bytes,
            workers,
            shared_support_recovery: false,
        },
        &mut progress,
    )
    .map_err(std::io::Error::other)
}

pub fn compact_canonical_l2_package(
    reference_path: &std::path::Path,
    output_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let reference_bytes = std::fs::read(reference_path)?;
    let reference_sha256 = sha256_hex(&reference_bytes);
    let package = format::decode_package(&reference_bytes).map_err(std::io::Error::other)?;
    let (compact_bytes, stats) =
        compact_format::encode_package(&package).map_err(std::io::Error::other)?;
    let decoded = compact_format::decode_package(&compact_bytes).map_err(std::io::Error::other)?;
    if decoded != package {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "compact L2 round-trip changed the canonical package",
        ));
    }
    let compact_sha256 = sha256_hex(&compact_bytes);
    write_atomic(output_path, &compact_bytes)?;
    Ok(serde_json::json!({
        "kind": "canonical_l2_compact_format",
        "verdict": "PASS_format_roundtrip",
        "reference": reference_path,
        "reference_bytes": reference_bytes.len(),
        "reference_sha256": reference_sha256,
        "output": output_path,
        "compact_bytes": compact_bytes.len(),
        "compact_format_version": stats.version,
        "compact_sha256": compact_sha256,
        "compression_ratio": compact_bytes.len() as f64 / reference_bytes.len().max(1) as f64,
        "saved_bytes": reference_bytes.len().saturating_sub(compact_bytes.len()),
        "decoder_block_forms": compact_format::DECODER_BLOCK_FORMS,
        "forms": package.form_refs.len(),
        "lemmas": package.lemma_centers.len(),
        "morph_bindings": package.morph_bindings.len(),
        "feature_dictionary_entries": stats.feature_dictionary_entries,
        "sections": {
            "header_bytes": stats.header_bytes,
            "form_ref_bytes": stats.form_ref_bytes,
            "decoder_offset_bytes": stats.decoder_offset_bytes,
            "decoder_payload_bytes": stats.decoder_payload_bytes,
            "feature_dictionary_bytes": stats.feature_dictionary_bytes,
            "lemma_center_bytes": stats.lemma_center_bytes,
            "morph_binding_bytes": stats.morph_binding_bytes,
            "context_mode_bytes": stats.context_mode_bytes,
            "slot_center_bytes": stats.slot_center_bytes,
            "neighbor_coupling_bytes": stats.neighbor_coupling_bytes,
            "competition_edge_bytes": stats.competition_edge_bytes,
            "calibration_bytes": stats.calibration_bytes,
            "lemma_wave_range_bytes": stats.lemma_wave_range_bytes,
            "surface_wave_code_bytes": stats.surface_wave_code_bytes,
            "wave_band_offset_bytes": stats.wave_band_offset_bytes,
            "wave_band_posting_bytes": stats.wave_band_posting_bytes,
            "atom_key_bytes": stats.atom_key_bytes,
            "atom_offset_bytes": stats.atom_offset_bytes,
            "atom_posting_bytes": stats.atom_posting_bytes,
        },
        "decoder_blocks": stats.decoder_blocks,
        "lemma_wave_ranges": stats.lemma_wave_ranges,
        "surface_wave_codes": stats.surface_wave_codes,
        "wave_band_offsets": stats.wave_band_offsets,
        "wave_band_postings": stats.wave_band_postings,
        "atom_keys": stats.atom_keys,
        "atom_offsets": stats.atom_offsets,
        "atom_posting_bytes": stats.atom_postings,
        "exact_package_roundtrip": true,
        "runtime_authority_changed": false,
    }))
}

pub fn prove_compact_canonical_l2_parity(
    reference_path: &std::path::Path,
    compact_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let reference_bytes = std::fs::read(reference_path)?;
    let compact_bytes = std::fs::read(compact_path)?;
    let reference = format::decode_package(&reference_bytes).map_err(std::io::Error::other)?;
    let compact = compact_format::decode_package(&compact_bytes).map_err(std::io::Error::other)?;
    let reencoded_reference = format::encode_package(&compact).map_err(std::io::Error::other)?;
    let section_parity = serde_json::json!({
        "l1_package_fingerprint": reference.l1_package_fingerprint == compact.l1_package_fingerprint,
        "form_refs": reference.form_refs == compact.form_refs,
        "decoder_bytes": reference.decoder_bytes == compact.decoder_bytes,
        "lemma_centers": reference.lemma_centers == compact.lemma_centers,
        "morph_bindings": reference.morph_bindings == compact.morph_bindings,
        "context_modes": reference.context_modes == compact.context_modes,
        "slot_centers": reference.slot_centers == compact.slot_centers,
        "neighbor_couplings": reference.neighbor_couplings == compact.neighbor_couplings,
        "competition_edges": reference.competition_edges == compact.competition_edges,
        "calibration": reference.calibration == compact.calibration,
    });
    let exact_package_roundtrip = reference == compact;
    let reference_bytes_equal_after_decode = reference_bytes == reencoded_reference;
    if !exact_package_roundtrip || !reference_bytes_equal_after_decode {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "compact L2 parity failed: package={exact_package_roundtrip} reference_bytes={reference_bytes_equal_after_decode} sections={section_parity}"
            ),
        ));
    }
    Ok(serde_json::json!({
        "kind": "canonical_l2_compact_parity",
        "verdict": "PASS_exact_parity",
        "reference": reference_path,
        "reference_bytes": reference_bytes.len(),
        "reference_sha256": sha256_hex(&reference_bytes),
        "compact": compact_path,
        "compact_bytes": compact_bytes.len(),
        "compact_sha256": sha256_hex(&compact_bytes),
        "compression_ratio": compact_bytes.len() as f64 / reference_bytes.len().max(1) as f64,
        "forms": reference.form_refs.len(),
        "lemmas": reference.lemma_centers.len(),
        "morph_bindings": reference.morph_bindings.len(),
        "section_parity": section_parity,
        "exact_package_roundtrip": exact_package_roundtrip,
        "reference_bytes_equal_after_decode": reference_bytes_equal_after_decode,
        "runtime_authority_changed": false,
        "quality_proof_run": false,
    }))
}

fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(temporary, path)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;

    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha256_file(path: &std::path::Path) -> Result<[u8; 32], String> {
    use sha2::Digest;

    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = sha2::Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|error| error.to_string())?;
    Ok(hasher.finalize().into())
}

fn hex_sha256(value: [u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_sha256_hex(value: &str) -> Result<[u8; 32], &'static str> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("expected corpus SHA-256 must contain exactly 64 hexadecimal digits");
    }
    let mut output = [0_u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "expected corpus SHA-256 is invalid")?;
    }
    Ok(output)
}

fn proc_status_kib(prefix: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix(prefix)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .unwrap_or_default()
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

pub fn prove_compositional_l2_restoration(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
    lemma_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> std::io::Result<serde_json::Value> {
    compositional_proof::prove_package(
        l1_package_path,
        l2_package_path,
        heldout_per_class,
        requested_workers,
        lemma_limit,
        form_limit,
        atom_relation_limit,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prove_contextual_compositional_l2_restoration(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> std::io::Result<serde_json::Value> {
    contextual_compositional_proof::prove_package(
        l1_package_path,
        l2_package_path,
        morphology_corpus_path,
        heldout_per_class,
        requested_workers,
        broad_lemma_limit,
        active_lemma_limit,
        feature_limit,
        form_limit,
        atom_relation_limit,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prove_productive_l2_restoration(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
    minimum_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    productive_proof::prove_package(
        l1_package_path,
        l2_package_path,
        morphology_corpus_path,
        heldout_per_class,
        requested_workers,
        broad_lemma_limit,
        active_lemma_limit,
        feature_limit,
        form_limit,
        atom_relation_limit,
        minimum_profile_support,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prove_productive_l2_sidecar(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    productive_sidecar_path: &std::path::Path,
    morphology_corpus_path: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> std::io::Result<serde_json::Value> {
    productive_proof::prove_compact_sidecar(
        l1_package_path,
        l2_package_path,
        productive_sidecar_path,
        morphology_corpus_path,
        heldout_per_class,
        requested_workers,
        broad_lemma_limit,
        active_lemma_limit,
        feature_limit,
        form_limit,
        atom_relation_limit,
    )
}

pub fn prove_productive_paradigm_field_v1(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    productive_package_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_dir: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> std::io::Result<serde_json::Value> {
    productive_v1::prove_productive_paradigm_field_v1(
        l1_package_path,
        l2_package_path,
        productive_package_path,
        axis_schema_path,
        work_dir,
        heldout_per_class,
        requested_workers,
    )
}

pub fn prove_productive_paradigm_field_v1_semantic(
    l1_package_path: &std::path::Path,
    l2_package_path: &std::path::Path,
    productive_package_path: &std::path::Path,
    axis_schema_path: &std::path::Path,
    work_dir: &std::path::Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> std::io::Result<serde_json::Value> {
    productive_v1::prove_productive_paradigm_field_v1_semantic(
        l1_package_path,
        l2_package_path,
        productive_package_path,
        axis_schema_path,
        work_dir,
        heldout_per_class,
        requested_workers,
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
            origin: runtime::L2LexicalSeedOrigin::GroundedL11,
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

pub fn query_live_canonical_l2(
    original: &str,
    repeat: usize,
    productive_lemma_limit: usize,
) -> std::io::Result<serde_json::Value> {
    if original.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "live L2 query text must not be empty",
        ));
    }
    if repeat == 0 || repeat > 10_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "live L2 query repeat must be in 1..=10000",
        ));
    }
    if productive_lemma_limit == 0 || productive_lemma_limit > CANONICAL_L2_ACTIVE_LEMMA_LIMIT {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("productive lemma limit must be in 1..={CANONICAL_L2_ACTIVE_LEMMA_LIMIT}"),
        ));
    }
    Ok(bridge::query_live_canonical_l2(
        original,
        repeat,
        productive_lemma_limit,
    ))
}

pub fn query_live_productive_v90(
    original: &str,
    repeat: usize,
) -> std::io::Result<serde_json::Value> {
    if original.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "live Productive V90 query text must not be empty",
        ));
    }
    if repeat == 0 || repeat > 10_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "live Productive V90 query repeat must be in 1..=10000",
        ));
    }
    Ok(bridge::query_live_productive_v90(original, repeat))
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
