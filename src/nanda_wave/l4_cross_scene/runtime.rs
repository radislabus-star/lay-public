use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use super::encoder::encode_scene_for_version;
use super::format::read_package;
use super::model::{
    L4CrossSceneDisposition, L4CrossSceneInput, L4CrossScenePackage, L4CrossSceneReadout,
    L4CrossSceneRecommendation,
};
use crate::nanda_wave::phase_field::max_coherence;
use crate::text_metrics::score_to_milli;

const REFRESH_INTERVAL_MS: u64 = 1_000;
static LAST_REFRESH_MS: AtomicU64 = AtomicU64::new(0);
static SHADOW_PACKAGE: OnceLock<RwLock<ShadowPackageState>> = OnceLock::new();

#[derive(Default)]
struct ShadowPackageState {
    stamp: Option<PackageStamp>,
    package: Option<Arc<L4CrossScenePackage>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PackageStamp {
    path: PathBuf,
    bytes: u64,
    modified_ns: u128,
}

pub(crate) fn shadow_readout(input: L4CrossSceneInput<'_>) -> L4CrossSceneReadout {
    refresh_if_due(false);
    let package = shadow_state()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .package
        .clone();
    package
        .as_deref()
        .map(|package| readout(package, input))
        .unwrap_or_default()
}

pub(crate) fn reload_shadow_package() -> bool {
    refresh_if_due(true)
}

pub(crate) fn readout(
    package: &L4CrossScenePackage,
    input: L4CrossSceneInput<'_>,
) -> L4CrossSceneReadout {
    let Some(profile_key) = profile_key_for_package(package, input.profile) else {
        return L4CrossSceneReadout {
            package_loaded: true,
            ..L4CrossSceneReadout::default()
        };
    };
    let Some(encoded) = encode_scene_for_version(input, package.encoder_version) else {
        return L4CrossSceneReadout {
            package_loaded: true,
            ..L4CrossSceneReadout::default()
        };
    };
    let Some(profile) = package
        .profiles
        .binary_search_by_key(&profile_key, |profile| profile.key)
        .ok()
        .map(|index| &package.profiles[index])
    else {
        return L4CrossSceneReadout {
            package_loaded: true,
            ..L4CrossSceneReadout::default()
        };
    };

    let positive = max_coherence(&encoded.vector, &profile.positive).unwrap_or_default();
    let negative = max_coherence(&encoded.vector, &profile.negative).unwrap_or_default();
    let hard_negative = max_coherence(&encoded.vector, &profile.hard_negative).unwrap_or_default();
    let ambiguity = max_coherence(&encoded.vector, &profile.ambiguity).unwrap_or_default();
    let destructive = negative.max(hard_negative);
    let profile_margin = positive - destructive;
    let profile_threshold = profile.threshold_micro as f32 / 1_000_000.0;

    let pair = pair_readout(package, &encoded.vector, input, profile_key);
    let pair_blocks_support = pair.present && pair.margin < -pair.threshold;
    let pair_blocks_repel = pair.present && pair.margin > pair.threshold;
    let ambiguity_peak = ambiguity.max(pair.ambiguity);
    let evidence_peak = positive.max(destructive).max(pair.evidence_peak);
    let ambiguity_close = ambiguity_peak > 0.0
        && (ambiguity_peak + profile_threshold * 0.5 >= evidence_peak
            || profile_margin.abs() <= profile_threshold);

    let disposition = if ambiguity_close {
        L4CrossSceneDisposition::Ambiguous
    } else if profile_margin > profile_threshold && !pair_blocks_support {
        L4CrossSceneDisposition::Supported
    } else if profile_margin < -profile_threshold && !pair_blocks_repel {
        L4CrossSceneDisposition::Repelled
    } else if pair.present && pair.margin > pair.threshold && positive > 0.0 {
        L4CrossSceneDisposition::Supported
    } else if pair.present && pair.margin < -pair.threshold && destructive > 0.0 {
        L4CrossSceneDisposition::Repelled
    } else {
        L4CrossSceneDisposition::Unknown
    };
    let recommendation = if disposition == L4CrossSceneDisposition::Supported {
        L4CrossSceneRecommendation::SuggestOnly
    } else {
        L4CrossSceneRecommendation::Keep
    };
    L4CrossSceneReadout {
        package_loaded: true,
        profile_present: true,
        disposition,
        recommendation,
        margin_milli: score_to_milli(profile_margin),
        threshold_milli: score_to_milli(profile_threshold.max(pair.threshold)),
        positive_milli: score_to_milli(positive),
        negative_milli: score_to_milli(negative),
        hard_negative_milli: score_to_milli(hard_negative),
        ambiguity_milli: score_to_milli(ambiguity_peak),
        pair_margin_milli: score_to_milli(pair.margin),
        positive_centers: profile.positive.len().min(u8::MAX as usize) as u8,
        negative_centers: profile.negative.len().min(u8::MAX as usize) as u8,
        hard_negative_centers: profile.hard_negative.len().min(u8::MAX as usize) as u8,
        ambiguity_centers: profile.ambiguity.len().min(u8::MAX as usize) as u8,
    }
}

fn profile_key_for_package(
    package: &L4CrossScenePackage,
    key: super::model::L4CrossSceneProfileKey,
) -> Option<super::model::L4CrossSceneProfileKey> {
    match (package.encoder_version, package.encoder_hash) {
        (super::V1_ENCODER_VERSION, super::V1_ENCODER_HASH) => Some(key.legacy_v1()),
        (super::ENCODER_VERSION, super::ENCODER_HASH) => Some(key),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PairReadout {
    present: bool,
    margin: f32,
    threshold: f32,
    ambiguity: f32,
    evidence_peak: f32,
}

fn pair_readout(
    package: &L4CrossScenePackage,
    vector: &[crate::nanda_wave::phase_field::PhaseCell],
    input: L4CrossSceneInput<'_>,
    profile_key: super::model::L4CrossSceneProfileKey,
) -> PairReadout {
    if input.candidate_relation_id == input.keep_relation_id {
        return PairReadout::default();
    }
    let low = input.candidate_relation_id.min(input.keep_relation_id);
    let high = input.candidate_relation_id.max(input.keep_relation_id);
    let key = (profile_key, low, high);
    let Some(pair) = package
        .pair_profiles
        .binary_search_by_key(&key, |pair| {
            (pair.key, pair.low_relation, pair.high_relation)
        })
        .ok()
        .map(|index| &package.pair_profiles[index])
    else {
        return PairReadout::default();
    };
    let low_score = max_coherence(vector, &pair.low_wins).unwrap_or_default();
    let high_score = max_coherence(vector, &pair.high_wins).unwrap_or_default();
    let hard_low = max_coherence(vector, &pair.hard_low_wins).unwrap_or_default();
    let hard_high = max_coherence(vector, &pair.hard_high_wins).unwrap_or_default();
    let low_score = low_score.max(hard_low);
    let high_score = high_score.max(hard_high);
    let candidate_is_low = input.candidate_relation_id == low;
    let margin = if candidate_is_low {
        low_score - high_score
    } else {
        high_score - low_score
    };
    PairReadout {
        present: true,
        margin,
        threshold: pair.threshold_micro as f32 / 1_000_000.0,
        ambiguity: max_coherence(vector, &pair.ambiguity).unwrap_or_default(),
        evidence_peak: low_score.max(high_score),
    }
}

fn default_package_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_L4_CROSS_SCENE_MEMORY") {
        return Some(PathBuf::from(path));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/share/lay/nanda_wave/l4_cross_scene_v1.bin"))
}

fn shadow_state() -> &'static RwLock<ShadowPackageState> {
    SHADOW_PACKAGE.get_or_init(|| RwLock::new(ShadowPackageState::default()))
}

fn refresh_if_due(force: bool) -> bool {
    let now = now_millis();
    if !force {
        let previous = LAST_REFRESH_MS.load(Ordering::Relaxed);
        if now.saturating_sub(previous) < REFRESH_INTERVAL_MS {
            return false;
        }
        if LAST_REFRESH_MS
            .compare_exchange(previous, now, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return false;
        }
    } else {
        LAST_REFRESH_MS.store(now, Ordering::Release);
    }

    let path = default_package_path();
    let stamp = path.as_deref().and_then(package_stamp);
    {
        let state = shadow_state()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.stamp == stamp {
            return false;
        }
    }
    let package = path
        .as_deref()
        .filter(|path| path.is_file())
        .and_then(|path| read_package(path).ok())
        .map(Arc::new);
    let mut state = shadow_state()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.stamp = stamp;
    state.package = package;
    true
}

fn package_stamp(path: &Path) -> Option<PackageStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified_ns = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some(PackageStamp {
        path: path.to_path_buf(),
        bytes: metadata.len(),
        modified_ns,
    })
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l4_cross_scene::compiler::{
        compile_observations, CrossSceneCompileConfig,
    };
    use crate::nanda_wave::l4_cross_scene::encoder::{
        candidate_relation_id, context_signal_from_text, keep_relation_id,
        relation_class_from_context,
    };
    use crate::nanda_wave::l4_cross_scene::model::{
        L4CrossSceneL2Signal, L4CrossSceneObservation, L4CrossSceneProfileKey,
    };
    use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};
    use crate::typing_memory::{
        LayoutProjectionDirection, LayoutProjectionScope, TypingMemoryOutcome,
    };

    fn observation(
        context: &[&str],
        outcome: TypingMemoryOutcome,
        receipt_id: u64,
    ) -> L4CrossSceneObservation {
        let context = context
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let from = "ghbdtn";
        let to = "привет";
        let relation = TransitionRelationAtoms::for_operator(
            from,
            to,
            TransitionOperatorKind::LayoutProjection,
        );
        let identity =
            crate::typing_memory::TypingTransitionIdentity::observed(from, to, "replacement");
        let sentence_language =
            crate::typing_scene::SentenceLanguageEvidence::script_only(&context, to);
        L4CrossSceneObservation {
            receipt_id,
            complete_chain: true,
            profile: L4CrossSceneProfileKey::new(
                TransitionOperatorKind::LayoutProjection,
                Some(LayoutProjectionDirection::EnToRu),
                Some(LayoutProjectionScope::CurrentToken),
            )
            .with_scene(identity.scene, sentence_language),
            context: context.clone(),
            from_text: from.to_string(),
            to_text: to.to_string(),
            relation_atoms: relation.atoms().to_vec(),
            candidate_relation_id: candidate_relation_id(relation.atoms()),
            keep_relation_id: keep_relation_id(),
            l3_relation_class: relation_class_from_context(&context, to),
            context_signal: context_signal_from_text(&context, to),
            l2_signal: L4CrossSceneL2Signal::Support,
            sentence_language,
            scene_symbols: identity.scene.known_symbols(),
            outcome,
        }
    }

    #[test]
    fn supported_cross_scene_transfer_never_grants_apply_authority() {
        let observations = vec![
            observation(&["мы", "пишем"], TypingMemoryOutcome::ConfirmedPositive, 1),
            observation(
                &["они", "читают"],
                TypingMemoryOutcome::ConfirmedPositive,
                2,
            ),
        ];
        let query = observation(&["вы", "видите"], TypingMemoryOutcome::Censored, 3);
        let (package, _) = compile_observations(&observations, CrossSceneCompileConfig::default());
        let result = readout(&package, query.input());

        assert_eq!(result.disposition, L4CrossSceneDisposition::Supported);
        assert_eq!(
            result.recommendation,
            L4CrossSceneRecommendation::SuggestOnly
        );
        assert!(!result.recommendation.automatic_apply());
    }
}
