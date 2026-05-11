//! Shared correction result contract.
//!
//! Desktop adapters should decide how to execute this result locally: replay
//! physical keys, replace a minimal text range, or keep the original text.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Correction {
    ReplayAll,
    InsertText(String),
}

impl Correction {
    pub fn is_insert_text(&self) -> bool {
        matches!(self, Self::InsertText(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_insert_text_check() {
        assert!(Correction::InsertText("Double".to_string()).is_insert_text());
        assert!(!Correction::ReplayAll.is_insert_text());
    }
}
