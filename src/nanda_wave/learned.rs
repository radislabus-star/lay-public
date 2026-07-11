use std::env;
use std::path::PathBuf;

/// Legacy packet destination retained for offline migration tools.
/// Live candidate generation no longer performs exact `surface -> answer` reads.
pub fn default_memory_path() -> PathBuf {
    env::var_os("LAY_NANDA_WAVE_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share/lay/nanda_wave/learned_memory.cell32")
        })
}
