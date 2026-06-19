use super::signal::WordCandidate;

pub const PATTERN_WAVE_CELL: &str = "PatternWaveCell32";
const SLOT_COUNT: u16 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternWaveVerdict {
    Boost,
    Veto,
    None,
}

impl PatternWaveVerdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Boost => "boost",
            Self::Veto => "veto",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternWaveReport {
    pub class: &'static str,
    pub resonance: f32,
    pub verdict: PatternWaveVerdict,
    pub slots: [u16; 4],
}

impl PatternWaveReport {
    pub fn boost(&self) -> f32 {
        match self.verdict {
            PatternWaveVerdict::Boost => (self.resonance * 0.08).clamp(0.0, 0.08),
            PatternWaveVerdict::Veto | PatternWaveVerdict::None => 0.0,
        }
    }

    pub fn vetoes(&self) -> bool {
        self.verdict == PatternWaveVerdict::Veto && self.resonance >= 0.70
    }

    pub fn summary(&self) -> String {
        format!(
            "class={} verdict={} resonance={:.3} slots=[{},{},{},{}]",
            self.class,
            self.verdict.as_str(),
            self.resonance,
            self.slots[0],
            self.slots[1],
            self.slots[2],
            self.slots[3]
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Ru,
    En,
    Technical,
    Mixed,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatternBlocks<'a> {
    left: &'a str,
    focus_original: &'a str,
    focus_candidate: &'a str,
    right: &'a str,
}

pub fn evaluate_pattern_wave(original: &str, candidate: &WordCandidate) -> PatternWaveReport {
    let blocks = pattern_blocks(original, &candidate.text);
    let class = pattern_class(candidate, &blocks);
    let resonance = pattern_resonance(candidate, class, &blocks);
    let verdict = pattern_verdict(candidate, class, resonance);
    PatternWaveReport {
        class,
        resonance,
        verdict,
        slots: [
            wave_slot("B0", blocks.left, class),
            wave_slot("B1", blocks.focus_original, class),
            wave_slot("B2", blocks.right, class),
            wave_slot("C", blocks.focus_candidate, class),
        ],
    }
}

fn pattern_blocks<'a>(original: &'a str, candidate: &'a str) -> PatternBlocks<'a> {
    let original_tokens = original.split_whitespace().collect::<Vec<_>>();
    let candidate_tokens = candidate.split_whitespace().collect::<Vec<_>>();
    if original_tokens.is_empty() {
        return PatternBlocks {
            left: "",
            focus_original: "",
            focus_candidate: "",
            right: "",
        };
    }
    let focus_idx = original_tokens
        .iter()
        .zip(candidate_tokens.iter())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| original_tokens.len().saturating_sub(1));
    PatternBlocks {
        left: focus_idx
            .checked_sub(1)
            .and_then(|idx| original_tokens.get(idx))
            .copied()
            .unwrap_or(""),
        focus_original: original_tokens.get(focus_idx).copied().unwrap_or(""),
        focus_candidate: candidate_tokens.get(focus_idx).copied().unwrap_or(""),
        right: original_tokens.get(focus_idx + 1).copied().unwrap_or(""),
    }
}

fn pattern_class(candidate: &WordCandidate, blocks: &PatternBlocks<'_>) -> &'static str {
    if candidate.source == "TechTokenCell32"
        || candidate.source == "LayoutWordCell32" && has_guarded_layout_shape(blocks)
    {
        return "technical_shape";
    }
    if candidate.source == "LayoutWordCell32" {
        return "safe_layout_shape";
    }
    if candidate.source == "BoundaryCell32" || candidate.source == "PhraseMemoryCell32" {
        return "space_glue_shape";
    }
    if candidate.source == super::context_wave::SEMANTIC_WORD_SOURCE
        || candidate.source == "CommonRuFixCell32"
        || candidate.source == "PhraseCell32"
        || candidate.source == "GrammarCell32"
    {
        return if token_kind(blocks.focus_original) == TokenKind::Ru {
            "semantic_typo_shape"
        } else {
            "semantic_phrase_shape"
        };
    }
    "unknown_shape"
}

fn pattern_resonance(
    candidate: &WordCandidate,
    class: &'static str,
    blocks: &PatternBlocks<'_>,
) -> f32 {
    let local = local_match_energy(blocks);
    let source = source_energy(class);
    let candidate_signal = (candidate.energy - candidate.risk).clamp(0.0, 1.0);
    (local * 0.35 + source * 0.30 + candidate_signal * 0.35).clamp(0.0, 1.0)
}

fn pattern_verdict(
    candidate: &WordCandidate,
    class: &'static str,
    resonance: f32,
) -> PatternWaveVerdict {
    if class == "technical_shape" && candidate.source == "LayoutWordCell32" {
        return PatternWaveVerdict::Veto;
    }
    if matches!(
        class,
        "space_glue_shape" | "semantic_typo_shape" | "semantic_phrase_shape"
    ) && resonance >= 0.50
    {
        return PatternWaveVerdict::Boost;
    }
    PatternWaveVerdict::None
}

fn local_match_energy(blocks: &PatternBlocks<'_>) -> f32 {
    let left = token_kind(blocks.left);
    let original = token_kind(blocks.focus_original);
    let candidate = token_kind(blocks.focus_candidate);
    let right = token_kind(blocks.right);
    let mut energy: f32 = 0.35;
    if candidate != TokenKind::Other && candidate != TokenKind::Mixed {
        energy += 0.20;
    }
    if original != candidate {
        energy += 0.18;
    }
    if left != TokenKind::Other || right != TokenKind::Other {
        energy += 0.12;
    }
    if left == TokenKind::Technical || right == TokenKind::Technical {
        energy += 0.10;
    }
    energy.clamp(0.0, 1.0)
}

fn source_energy(class: &str) -> f32 {
    match class {
        "safe_layout_shape" => 0.78,
        "semantic_typo_shape" => 0.74,
        "semantic_phrase_shape" => 0.70,
        "space_glue_shape" => 0.76,
        "technical_shape" => 0.82,
        _ => 0.40,
    }
}

fn token_kind(token: &str) -> TokenKind {
    let clean = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    if clean.is_empty() {
        return TokenKind::Other;
    }
    let has_ru = clean.chars().any(is_cyrillic);
    let has_en = clean.chars().any(|ch| ch.is_ascii_alphabetic());
    if has_ru && has_en {
        return TokenKind::Mixed;
    }
    if has_ru {
        return TokenKind::Ru;
    }
    if has_en {
        if crate::lexicon::is_common_en_technical_word(&clean.to_ascii_lowercase()) {
            TokenKind::Technical
        } else {
            TokenKind::En
        }
    } else {
        TokenKind::Other
    }
}

fn has_guarded_layout_shape(blocks: &PatternBlocks<'_>) -> bool {
    let left = blocks
        .left
        .trim_matches(|ch: char| ch.is_ascii_punctuation())
        .to_ascii_lowercase();
    crate::lexicon::is_common_en_guard_prefix(&left)
        || blocks.focus_original.starts_with('-')
        || blocks.focus_original.contains('/')
        || blocks.focus_original.contains('=')
        || blocks.focus_original.contains("://")
}

fn is_cyrillic(ch: char) -> bool {
    ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch) || ch == 'ё' || ch == 'Ё'
}

fn wave_slot(block: &str, value: &str, class: &str) -> u16 {
    let mut hash = 0x811c_9dc5u32;
    for byte in block
        .bytes()
        .chain([0xff])
        .chain(class.bytes())
        .chain([0xfe])
        .chain(value.bytes())
    {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    (hash % u32::from(SLOT_COUNT)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(text: &str, source: &'static str) -> WordCandidate {
        WordCandidate {
            text: text.to_string(),
            source,
            energy: 0.82,
            risk: 0.12,
            support: Vec::new(),
        }
    }

    #[test]
    fn pattern_wave_classifies_safe_layout_shape_without_owning_it() {
        let report = evaluate_pattern_wave(
            "пишу djn дальше ",
            &candidate("пишу вот дальше", "LayoutWordCell32"),
        );
        assert_eq!(report.class, "safe_layout_shape");
        assert_eq!(report.verdict, PatternWaveVerdict::None);
        assert!(report.slots.iter().all(|slot| *slot < SLOT_COUNT));
    }

    #[test]
    fn pattern_wave_boosts_semantic_typo_shape() {
        let report = evaluate_pattern_wave(
            "это невидные ",
            &candidate(
                "это невалидные",
                super::super::context_wave::SEMANTIC_WORD_SOURCE,
            ),
        );
        assert_eq!(report.class, "semantic_typo_shape");
        assert_eq!(report.verdict, PatternWaveVerdict::Boost);
    }

    #[test]
    fn pattern_wave_vetoes_layout_inside_technical_shape() {
        let report = evaluate_pattern_wave("git djn ", &candidate("git вот", "LayoutWordCell32"));
        assert_eq!(report.class, "technical_shape");
        assert_eq!(report.verdict, PatternWaveVerdict::Veto);
    }
}
