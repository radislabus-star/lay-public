//! Runtime sizing profile for NANDA/Expert64 experiments.
//!
//! This module only reports the recommended cell budget. It does not change
//! correction decisions by itself.

use std::fs;
use std::path::Path;

pub const EXPERT_CELL_KIB: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NandaCpuProfile {
    pub l1d_kib_per_core: Option<usize>,
    pub l2_kib_per_core: Option<usize>,
    pub l3_kib_total: Option<usize>,
    pub active_cells: usize,
    pub warm_cells: usize,
}

impl NandaCpuProfile {
    pub fn detect() -> Self {
        let caches = CpuCaches::detect();
        Self::from_caches(
            caches.l1d_kib_per_core,
            caches.l2_kib_per_core,
            caches.l3_kib_total,
        )
    }

    pub fn from_caches(
        l1d_kib_per_core: Option<usize>,
        l2_kib_per_core: Option<usize>,
        l3_kib_total: Option<usize>,
    ) -> Self {
        let active_cells = recommended_active_cells(l2_kib_per_core);
        let warm_cells = recommended_warm_cells(l3_kib_total);
        Self {
            l1d_kib_per_core,
            l2_kib_per_core,
            l3_kib_total,
            active_cells,
            warm_cells,
        }
    }

    pub fn compact_text(&self) -> String {
        format!(
            "Клетка: {} КБ · активно {} · запас {} · L2 {} · L3 {}",
            EXPERT_CELL_KIB,
            self.active_cells,
            self.warm_cells,
            fmt_cache(self.l2_kib_per_core),
            fmt_cache(self.l3_kib_total)
        )
    }
}

#[derive(Debug, Default)]
struct CpuCaches {
    l1d_kib_per_core: Option<usize>,
    l2_kib_per_core: Option<usize>,
    l3_kib_total: Option<usize>,
}

impl CpuCaches {
    fn detect() -> Self {
        let root = Path::new("/sys/devices/system/cpu/cpu0/cache");
        let mut caches = Self::default();
        let Ok(entries) = fs::read_dir(root) else {
            return caches;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(level) = fs::read_to_string(path.join("level")) else {
                continue;
            };
            let Ok(cache_type) = fs::read_to_string(path.join("type")) else {
                continue;
            };
            let Some(size_kib) = read_cache_size_kib(&path.join("size")) else {
                continue;
            };

            match (level.trim(), cache_type.trim()) {
                ("1", "Data") => caches.l1d_kib_per_core = Some(size_kib),
                ("2", _) => caches.l2_kib_per_core = Some(size_kib),
                ("3", _) => caches.l3_kib_total = Some(size_kib),
                _ => {}
            }
        }

        caches
    }
}

fn recommended_active_cells(l2_kib_per_core: Option<usize>) -> usize {
    match l2_kib_per_core.unwrap_or(256) {
        0..=255 => 4,
        256..=511 => 8,
        512..=1023 => 16,
        _ => 32,
    }
}

fn recommended_warm_cells(l3_kib_total: Option<usize>) -> usize {
    let l3_cells = l3_kib_total.unwrap_or(4096) / EXPERT_CELL_KIB;
    l3_cells.clamp(32, 256)
}

fn read_cache_size_kib(path: &Path) -> Option<usize> {
    let text = fs::read_to_string(path).ok()?;
    parse_cache_size_kib(text.trim())
}

fn parse_cache_size_kib(text: &str) -> Option<usize> {
    let number = text
        .trim_end_matches(['K', 'k', 'M', 'm'])
        .parse::<usize>()
        .ok()?;
    if text.ends_with(['M', 'm']) {
        Some(number * 1024)
    } else {
        Some(number)
    }
}

fn fmt_cache(kib: Option<usize>) -> String {
    match kib {
        Some(value) if value >= 1024 && value % 1024 == 0 => format!("{} МБ", value / 1024),
        Some(value) => format!("{value} КБ"),
        None => "неизвестно".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t480_like_profile_uses_small_hot_set_and_l3_warm_pool() {
        let profile = NandaCpuProfile::from_caches(Some(32), Some(256), Some(8192));

        assert_eq!(profile.active_cells, 8);
        assert_eq!(profile.warm_cells, 128);
        assert_eq!(
            profile.compact_text(),
            "Клетка: 64 КБ · активно 8 · запас 128 · L2 256 КБ · L3 8 МБ"
        );
    }

    #[test]
    fn larger_l2_allows_larger_active_set() {
        let profile = NandaCpuProfile::from_caches(Some(64), Some(2048), Some(32768));

        assert_eq!(profile.active_cells, 32);
        assert_eq!(profile.warm_cells, 256);
    }

    #[test]
    fn parses_sysfs_cache_size_units() {
        assert_eq!(parse_cache_size_kib("256K"), Some(256));
        assert_eq!(parse_cache_size_kib("8M"), Some(8192));
    }
}
