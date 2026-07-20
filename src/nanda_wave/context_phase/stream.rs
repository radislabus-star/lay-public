use std::io::{self, Read};

use serde::Serialize;

pub(super) const MAX_FRAGMENT_TOKENS: usize = 64;
const MAX_FRAGMENT_BYTES: usize = 8 * 1024;
const READ_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub(super) struct FragmentStreamStats {
    pub(super) accepted_fragments: usize,
    pub(super) rejected_token_count: usize,
    pub(super) oversized_fragments: usize,
    pub(super) invalid_utf8_fragments: usize,
    pub(super) peak_fragment_bytes: usize,
}

/// Visits one tokenized fragment at a time. The reader and this adapter never
/// retain corpus text beyond one bounded fragment.
pub(super) fn visit_tokenized_fragments<R, F>(
    mut reader: R,
    max_fragments: usize,
    mut visitor: F,
) -> io::Result<FragmentStreamStats>
where
    R: Read,
    F: FnMut(usize, &[String]) -> io::Result<()>,
{
    let mut stats = FragmentStreamStats::default();
    let mut read_buffer = [0_u8; READ_BUFFER_BYTES];
    let mut fragment = Vec::with_capacity(512);
    let mut dropping_oversized = false;
    let mut done = false;

    loop {
        let read = reader.read(&mut read_buffer)?;
        if read == 0 {
            flush_fragment(
                &mut fragment,
                &mut dropping_oversized,
                max_fragments,
                &mut stats,
                &mut visitor,
            )?;
            break;
        }
        for &byte in &read_buffer[..read] {
            if is_fragment_boundary(byte) {
                flush_fragment(
                    &mut fragment,
                    &mut dropping_oversized,
                    max_fragments,
                    &mut stats,
                    &mut visitor,
                )?;
                if max_fragments > 0 && stats.accepted_fragments >= max_fragments {
                    done = true;
                    break;
                }
            } else if !dropping_oversized {
                if fragment.len() < MAX_FRAGMENT_BYTES {
                    fragment.push(byte);
                    stats.peak_fragment_bytes = stats.peak_fragment_bytes.max(fragment.len());
                } else {
                    fragment.clear();
                    dropping_oversized = true;
                }
            }
        }
        if done {
            break;
        }
    }
    Ok(stats)
}

fn flush_fragment<F>(
    fragment: &mut Vec<u8>,
    dropping_oversized: &mut bool,
    max_fragments: usize,
    stats: &mut FragmentStreamStats,
    visitor: &mut F,
) -> io::Result<()>
where
    F: FnMut(usize, &[String]) -> io::Result<()>,
{
    if *dropping_oversized {
        stats.oversized_fragments = stats.oversized_fragments.saturating_add(1);
        *dropping_oversized = false;
        fragment.clear();
        return Ok(());
    }
    if fragment.iter().all(u8::is_ascii_whitespace) {
        fragment.clear();
        return Ok(());
    }
    if max_fragments > 0 && stats.accepted_fragments >= max_fragments {
        fragment.clear();
        return Ok(());
    }
    let text = match std::str::from_utf8(fragment) {
        Ok(text) => text,
        Err(_) => {
            stats.invalid_utf8_fragments = stats.invalid_utf8_fragments.saturating_add(1);
            fragment.clear();
            return Ok(());
        }
    };
    let tokens = super::super::llmwave::tokenize(text);
    fragment.clear();
    if !(3..=MAX_FRAGMENT_TOKENS).contains(&tokens.len()) {
        stats.rejected_token_count = stats.rejected_token_count.saturating_add(1);
        return Ok(());
    }
    let ordinal = stats.accepted_fragments;
    visitor(ordinal, &tokens)?;
    stats.accepted_fragments = stats.accepted_fragments.saturating_add(1);
    Ok(())
}

fn is_fragment_boundary(byte: u8) -> bool {
    matches!(byte, b'\n' | b'\r' | b'.' | b'!' | b'?' | b';')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_never_retains_more_than_one_bounded_fragment() {
        let input = format!(
            "{}. один два три. четыре пять шесть.",
            "я".repeat(MAX_FRAGMENT_BYTES + 32)
        );
        let mut observed = Vec::new();
        let stats = visit_tokenized_fragments(input.as_bytes(), 0, |ordinal, tokens| {
            observed.push((ordinal, tokens.len()));
            Ok(())
        })
        .unwrap();

        assert_eq!(stats.oversized_fragments, 1);
        assert_eq!(stats.accepted_fragments, 2);
        assert!(stats.peak_fragment_bytes <= MAX_FRAGMENT_BYTES);
        assert_eq!(observed, vec![(0, 3), (1, 3)]);
    }

    #[test]
    fn max_fragments_counts_only_accepted_fragments() {
        let input = "один. один два три. четыре пять шесть. семь восемь девять.";
        let mut observed = 0;
        let stats = visit_tokenized_fragments(input.as_bytes(), 2, |_, _| {
            observed += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(observed, 2);
        assert_eq!(stats.accepted_fragments, 2);
        assert_eq!(stats.rejected_token_count, 1);
    }
}
