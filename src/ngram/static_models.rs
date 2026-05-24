use super::cache::{default_ru_cache_path, load_ru_cache, save_ru_cache};
use super::sources::{build_en_model_from_sources, build_ru_model_from_sources};
use super::CharNgramModel;
use std::sync::OnceLock;

pub fn ru_score(text: &str) -> f64 {
    ru_model().score_text(text)
}

pub fn ru_candidate_margin(candidate: &str, baseline: &str) -> f64 {
    ru_model().margin(candidate, baseline)
}

pub fn ru_candidate_is_better(candidate: &str, baseline: &str, min_margin: f64) -> bool {
    ru_model().candidate_is_better(candidate, baseline, min_margin)
}

pub fn en_score(text: &str) -> f64 {
    en_model().score_text(text)
}

pub fn warm_up() {
    let _ = ru_model().vocab;
    let _ = en_model().vocab;
}

fn ru_model() -> &'static CharNgramModel {
    static MODEL: OnceLock<CharNgramModel> = OnceLock::new();
    MODEL.get_or_init(|| {
        if let Some(path) = default_ru_cache_path() {
            if let Ok(model) = load_ru_cache(&path) {
                return model;
            }
        }
        let model = build_ru_model_from_sources();
        if let Some(path) = default_ru_cache_path() {
            let _ = save_ru_cache(&path, &model);
        }
        model
    })
}

fn en_model() -> &'static CharNgramModel {
    static MODEL: OnceLock<CharNgramModel> = OnceLock::new();
    MODEL.get_or_init(build_en_model_from_sources)
}
