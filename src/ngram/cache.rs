use super::{CharNgramModel, Lang};
use std::io;

const CACHE_VERSION: u32 = 1;
const RU_CACHE_PATH: &str = ".cache/lay/ngram_ru_v1.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedModel {
    version: u32,
    model: CharNgramModel,
}

pub fn default_ru_cache_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(RU_CACHE_PATH))
}

pub fn load_ru_cache(path: &std::path::Path) -> io::Result<CharNgramModel> {
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
    Ok(cached.model)
}

pub fn save_ru_cache(path: &std::path::Path, model: &CharNgramModel) -> io::Result<u64> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cached = CachedModel {
        version: CACHE_VERSION,
        model: model.clone(),
    };
    let text = serde_json::to_string(&cached).map_err(io::Error::other)?;
    std::fs::write(path, text)?;
    Ok(std::fs::metadata(path)?.len())
}
