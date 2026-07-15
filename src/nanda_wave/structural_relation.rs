use super::signal::WordCandidate;
use crate::candidate_contract::CandidateOrigin;

pub const STRUCTURAL_RELATION_CELL: &str = "StructuralRelationCell32";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralVerdict {
    Boost,
    Veto,
    Watch,
}

impl StructuralVerdict {
    fn as_str(self) -> &'static str {
        match self {
            Self::Boost => "boost",
            Self::Veto => "veto",
            Self::Watch => "watch",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuralRelationReport {
    pub route: &'static str,
    pub relation: &'static str,
    pub coherence: f32,
    pub verdict: StructuralVerdict,
}

impl StructuralRelationReport {
    pub fn boost(&self) -> f32 {
        match self.verdict {
            StructuralVerdict::Boost => (self.coherence * 0.06).clamp(0.0, 0.06),
            StructuralVerdict::Veto | StructuralVerdict::Watch => 0.0,
        }
    }

    pub fn vetoes(&self) -> bool {
        self.verdict == StructuralVerdict::Veto
            && (self.coherence >= 0.62 || self.relation == "short_token_choice")
    }

    pub fn summary(&self) -> String {
        format!(
            "route={} relation={} verdict={} coherence={:.3}",
            self.route,
            self.relation,
            self.verdict.as_str(),
            self.coherence
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenRole {
    RuWord,
    EnWord,
    Technical,
    Mixed,
    Boundary,
    Other,
}

pub fn evaluate_structural_relation(
    original: &str,
    candidate: &WordCandidate,
) -> StructuralRelationReport {
    let roles = relation_roles(original, &candidate.text);
    let route = route_for(&roles);
    let relation = relation_for(candidate, &roles);
    let coherence = relation_coherence(candidate, route, relation, &roles);
    let verdict = relation_verdict(route, relation, coherence, &roles);
    StructuralRelationReport {
        route,
        relation,
        coherence,
        verdict,
    }
}

#[derive(Debug, Clone)]
struct RelationRoles {
    left_token: String,
    original_token: String,
    left: TokenRole,
    original: TokenRole,
    candidate: TokenRole,
    right: TokenRole,
}

fn relation_roles(original: &str, candidate: &str) -> RelationRoles {
    let original_tokens = original.split_whitespace().collect::<Vec<_>>();
    let candidate_tokens = candidate.split_whitespace().collect::<Vec<_>>();
    let focus_idx = original_tokens
        .iter()
        .zip(candidate_tokens.iter())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| original_tokens.len().saturating_sub(1));
    let left_index = focus_idx.checked_sub(1);
    RelationRoles {
        left_token: left_index
            .and_then(|idx| original_tokens.get(idx))
            .copied()
            .unwrap_or("")
            .to_string(),
        left: left_index
            .and_then(|idx| original_tokens.get(idx))
            .copied()
            .map(token_role)
            .unwrap_or(TokenRole::Other),
        original: original_tokens
            .get(focus_idx)
            .copied()
            .map(token_role)
            .unwrap_or(TokenRole::Other),
        original_token: original_tokens
            .get(focus_idx)
            .copied()
            .unwrap_or("")
            .to_string(),
        candidate: candidate_tokens
            .get(focus_idx)
            .copied()
            .map(token_role)
            .unwrap_or(TokenRole::Other),
        right: original_tokens
            .get(focus_idx + 1)
            .copied()
            .map(token_role)
            .unwrap_or(TokenRole::Other),
    }
}

fn route_for(roles: &RelationRoles) -> &'static str {
    if is_guard_prefix(&roles.left_token) {
        return "guarded_technical_context";
    }
    if roles.left == TokenRole::Technical || roles.right == TokenRole::Technical {
        return "technical_context";
    }
    if roles.original == TokenRole::Mixed || roles.candidate == TokenRole::Mixed {
        return "mixed_script";
    }
    if roles.original == TokenRole::Boundary || roles.candidate == TokenRole::Boundary {
        return "boundary";
    }
    if roles.left == TokenRole::RuWord || roles.right == TokenRole::RuWord {
        return "russian_phrase";
    }
    "local_word"
}

fn relation_for(candidate: &WordCandidate, roles: &RelationRoles) -> &'static str {
    match candidate.origin {
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
            if roles.original_token.chars().count() <= 1 =>
        {
            "short_token_choice"
        }
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo => "layout_binding",
        CandidateOrigin::Boundary => "space_boundary",
        CandidateOrigin::L3Context => "semantic_repair",
        CandidateOrigin::Technical => "technical_keep",
        _ if roles.original != roles.candidate => "role_change",
        _ => "identity",
    }
}

fn relation_coherence(
    candidate: &WordCandidate,
    route: &str,
    relation: &str,
    roles: &RelationRoles,
) -> f32 {
    let signal = (candidate.energy - candidate.risk).clamp(0.0, 1.0);
    let route_energy = match route {
        "technical_context" => 0.82,
        "russian_phrase" => 0.74,
        "boundary" => 0.78,
        "mixed_script" => 0.62,
        _ => 0.54,
    };
    let relation_energy = match relation {
        "layout_binding" if roles.candidate == TokenRole::RuWord => 0.82,
        "space_boundary" => 0.80,
        "semantic_repair" => 0.76,
        "grammar_agreement" => 0.70,
        "technical_keep" => 0.86,
        _ => 0.52,
    };
    (signal * 0.42 + route_energy * 0.30 + relation_energy * 0.28).clamp(0.0, 1.0)
}

fn relation_verdict(
    route: &str,
    relation: &str,
    coherence: f32,
    roles: &RelationRoles,
) -> StructuralVerdict {
    if route == "guarded_technical_context"
        && relation == "layout_binding"
        && roles.candidate == TokenRole::RuWord
    {
        return StructuralVerdict::Veto;
    }
    if route == "technical_context"
        && matches!(relation, "role_change" | "short_token_choice")
        && roles.candidate == TokenRole::RuWord
    {
        return StructuralVerdict::Veto;
    }
    if route == "local_word"
        && matches!(relation, "role_change" | "short_token_choice")
        && roles.candidate == TokenRole::RuWord
        && looks_like_latin_single_letter_label(&roles.original_token)
    {
        return StructuralVerdict::Veto;
    }
    if relation == "layout_binding" && roles.candidate == TokenRole::Mixed {
        return StructuralVerdict::Watch;
    }
    if matches!(
        relation,
        "layout_binding" | "space_boundary" | "semantic_repair" | "grammar_agreement"
    ) && coherence >= 0.54
    {
        return StructuralVerdict::Boost;
    }
    StructuralVerdict::Watch
}

fn looks_like_latin_single_letter_label(token: &str) -> bool {
    let clean = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    clean.chars().count() == 1
        && clean.chars().all(|ch| ch.is_ascii_alphabetic())
        && clean.chars().all(|ch| ch.is_ascii_uppercase())
}

fn token_role(token: &str) -> TokenRole {
    let clean = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
    if clean.is_empty() {
        return TokenRole::Boundary;
    }
    let has_ru = clean.chars().any(is_cyrillic);
    let has_en = clean.chars().any(|ch| ch.is_ascii_alphabetic());
    if has_ru && has_en {
        return TokenRole::Mixed;
    }
    if has_ru {
        return TokenRole::RuWord;
    }
    if has_en {
        if crate::lexicon::is_common_en_technical_word(&clean.to_ascii_lowercase()) {
            TokenRole::Technical
        } else {
            TokenRole::EnWord
        }
    } else {
        TokenRole::Other
    }
}

fn is_guard_prefix(token: &str) -> bool {
    crate::lexicon::is_common_en_guard_prefix(
        &token
            .trim_matches(|ch: char| ch.is_ascii_punctuation())
            .to_ascii_lowercase(),
    )
}

fn is_cyrillic(ch: char) -> bool {
    ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch) || ch == 'ё' || ch == 'Ё'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(text: &str, source: &'static str) -> WordCandidate {
        WordCandidate {
            text: text.to_string(),
            origin: if source == "PhraseCell32" {
                crate::candidate_contract::CandidateOrigin::L3Context
            } else {
                crate::candidate_contract::CandidateOrigin::Layout
            },
            source,
            energy: 0.78,
            risk: 0.18,
            support: Vec::new(),
        }
    }

    #[test]
    fn structural_relation_boosts_safe_layout_binding() {
        let report = evaluate_structural_relation(
            "пишу djn дальше ",
            &candidate("пишу вот дальше", "LayoutWordCell32"),
        );
        assert_eq!(report.route, "russian_phrase");
        assert_eq!(report.relation, "layout_binding");
        assert_eq!(report.verdict, StructuralVerdict::Boost);
        assert!(report.boost() > 0.0);
    }

    #[test]
    fn structural_relation_vetoes_technical_route_splice() {
        let report =
            evaluate_structural_relation("git djn ", &candidate("git вот", "LayoutWordCell32"));
        assert_eq!(report.route, "guarded_technical_context");
        assert_eq!(report.verdict, StructuralVerdict::Veto);
        assert!(report.vetoes());
    }

    #[test]
    fn structural_relation_allows_mixed_technical_sentence() {
        let report = evaluate_structural_relation(
            "html djn api ",
            &candidate("html вот api", "LayoutWordCell32"),
        );
        assert_eq!(report.route, "technical_context");
        assert_ne!(report.verdict, StructuralVerdict::Veto);
    }

    #[test]
    fn structural_relation_vetoes_short_token_inside_technical_context() {
        let report = evaluate_structural_relation(
            "HTML b tag ",
            &candidate("HTML и tag", "ShortTokenCell32"),
        );

        assert_eq!(report.route, "technical_context");
        assert_eq!(report.verdict, StructuralVerdict::Veto);
    }

    #[test]
    fn structural_relation_vetoes_uppercase_latin_label() {
        let report =
            evaluate_structural_relation("vitamin B ", &candidate("vitamin И", "PhraseCell32"));

        assert_eq!(report.route, "local_word");
        assert_eq!(report.verdict, StructuralVerdict::Veto);
    }
}
