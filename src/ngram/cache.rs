use super::{CharNgramModel, Lang};
use std::io;

const CACHE_VERSION: u32 = 2;
const RU_CACHE_PATH: &str = ".cache/lay/ngram_ru_v2.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedModel {
    version: u32,
    #[serde(default)]
    source_fingerprint: Option<String>,
    model: CharNgramModel,
}

pub fn default_ru_cache_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(RU_CACHE_PATH))
}

pub fn load_ru_cache(path: &std::path::Path) -> io::Result<CharNgramModel> {
    Ok(load_cached_model(path)?.model)
}

pub(super) fn load_ru_cache_for_source(
    path: &std::path::Path,
    expected_source_fingerprint: &str,
) -> io::Result<CharNgramModel> {
    let cached = load_cached_model(path)?;
    if cached.source_fingerprint.as_deref() != Some(expected_source_fingerprint) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ngram cache source fingerprint mismatch",
        ));
    }
    Ok(cached.model)
}

fn load_cached_model(path: &std::path::Path) -> io::Result<CachedModel> {
    let text = std::fs::read_to_string(path)?;
    let cached: CachedModel = serde_json::from_str(&text).map_err(io::Error::other)?;
    if cached.version != CACHE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported ngram cache version {}", cached.version),
        ));
    }
    if cached.model.lang != Lang::Ru {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ngram cache language is not RU",
        ));
    }
    Ok(cached)
}

pub fn save_ru_cache(path: &std::path::Path, model: &CharNgramModel) -> io::Result<u64> {
    save_cached_model(path, model, None)
}

pub(super) fn save_ru_cache_for_source(
    path: &std::path::Path,
    model: &CharNgramModel,
    source_fingerprint: &str,
) -> io::Result<u64> {
    save_cached_model(path, model, Some(source_fingerprint.to_string()))
}

fn save_cached_model(
    path: &std::path::Path,
    model: &CharNgramModel,
    source_fingerprint: Option<String>,
) -> io::Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cached = CachedModel {
        version: CACHE_VERSION,
        source_fingerprint,
        model: model.clone(),
    };
    let text = serde_json::to_string(&cached).map_err(io::Error::other)?;
    std::fs::write(path, text)?;
    Ok(std::fs::metadata(path)?.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "lay-ngram-cache-{name}-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("thread")
        ))
    }

    #[test]
    fn source_bound_cache_rejects_a_different_source() {
        let path = test_path("source");
        let model = CharNgramModel::train(Lang::Ru, ["пример", "проверка"]);
        save_ru_cache_for_source(&path, &model, "source-a").expect("save");
        assert!(load_ru_cache_for_source(&path, "source-a").is_ok());
        let error = load_ru_cache_for_source(&path, "source-b").expect_err("mismatch");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn explicit_cache_without_source_is_not_auto_runtime_authority() {
        let path = test_path("explicit");
        let model = CharNgramModel::train(Lang::Ru, ["пример", "проверка"]);
        save_ru_cache(&path, &model).expect("save");
        assert!(load_ru_cache(&path).is_ok());
        assert!(load_ru_cache_for_source(&path, "source-a").is_err());
        let _ = std::fs::remove_file(path);
    }
}
