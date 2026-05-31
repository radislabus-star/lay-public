//! Shared parser for plain line-based data files.
//!
//! Data files in this project use the same lightweight convention: trim each
//! line, skip empty lines, and skip comments starting with `#`.

pub(crate) fn data_lines(data: &str) -> impl Iterator<Item = &str> {
    data.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}
