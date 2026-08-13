pub(super) const DEFAULT_BIRTH_ATOMS_PER_CHANNEL: usize = 4;
const MAX_BIRTH_ATOMS_PER_CHANNEL: usize = 32;
pub(super) const DEFAULT_BIRTH_POSTING_BUDGET: usize = 131_072;
const MAX_BIRTH_POSTING_BUDGET: usize = 131_072;
const DEFAULT_REVERSE_CACHE_MIB: usize = 16;
const DEFAULT_FIRST_TOUCH_PROFILE_WORDS: usize = 4_096;
const MAX_FIRST_TOUCH_PROFILE_WORDS: usize = 16_384;

pub(super) const FIRST_TOUCH_TRANSIENT_RESERVE_MIB: usize = 4;

pub(super) fn readout_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("LAY_L11_READOUT_TRACE").is_some())
}

pub(super) fn readout_trace_terminal() -> Option<u32> {
    static VALUE: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("LAY_L11_READOUT_TRACE_TERMINAL")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
    })
}

pub(super) fn birth_atoms_per_channel() -> usize {
    static VALUE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("LAY_L11_BIRTH_ATOMS_PER_CHANNEL")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_BIRTH_ATOMS_PER_CHANNEL)
            .clamp(1, MAX_BIRTH_ATOMS_PER_CHANNEL)
    })
}

pub(super) fn birth_posting_budget() -> usize {
    static VALUE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        std::env::var("LAY_L11_BIRTH_POSTING_BUDGET")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_BIRTH_POSTING_BUDGET)
            .clamp(1, MAX_BIRTH_POSTING_BUDGET)
    })
}

pub(super) fn reverse_cache_bytes() -> usize {
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        std::env::var("LAY_L11_V8_REVERSE_CACHE_MIB")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_REVERSE_CACHE_MIB)
            .min(128)
            .saturating_mul(1024 * 1024)
    })
}

pub(super) fn first_touch_profile_word_count() -> usize {
    static WORDS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *WORDS.get_or_init(|| {
        std::env::var("LAY_L11_V8_WARM_PROFILE_WORDS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_FIRST_TOUCH_PROFILE_WORDS)
            .clamp(1, MAX_FIRST_TOUCH_PROFILE_WORDS)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_preserve_the_pinned_runtime_contract() {
        if [
            "LAY_L11_BIRTH_ATOMS_PER_CHANNEL",
            "LAY_L11_BIRTH_POSTING_BUDGET",
            "LAY_L11_V8_REVERSE_CACHE_MIB",
            "LAY_L11_V8_WARM_PROFILE_WORDS",
        ]
        .into_iter()
        .all(|name| std::env::var_os(name).is_none())
        {
            assert_eq!(birth_atoms_per_channel(), 4);
            assert_eq!(birth_posting_budget(), 131_072);
            assert_eq!(reverse_cache_bytes(), 16 * 1024 * 1024);
            assert_eq!(first_touch_profile_word_count(), 4_096);
        }
    }
}
