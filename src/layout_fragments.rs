//! Optional RU/EN layout evidence from overlapping byte fragments.
//!
//! This module has no field, IME, config, training or output authority. Its result
//! is consumed by existing automatic candidate preparation and common admission.

mod model;
#[cfg(test)]
mod tests;

use std::sync::OnceLock;

use crate::dict::{convert, Direction};
use model::Model;

const PACKAGE: &[u8] = include_bytes!("../data/layout_fragments/model.bin");
static MODEL: OnceLock<Option<Model<'static>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Unavailable = 0,
    Normal = 1,
    Known = 2,
    Special = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Score {
    pub total: i32,
    pub status: Status,
    pub matches: u32,
}

impl Score {
    const fn unavailable() -> Self {
        Self {
            total: 0,
            status: Status::Unavailable,
            matches: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Russian,
    English,
    Ambiguous,
    Special,
    Unavailable,
}

/// Reported selector order. Special results carry no inferred mutation meaning.
pub fn select(russian: Score, english: Score) -> Selection {
    if russian.status == Status::Special || english.status == Status::Special {
        return Selection::Special;
    }
    if russian.status == Status::Unavailable || english.status == Status::Unavailable {
        return Selection::Unavailable;
    }
    if english.status == Status::Known {
        return Selection::English;
    }
    if russian.status == Status::Known {
        return Selection::Russian;
    }
    match russian.total.cmp(&english.total) {
        std::cmp::Ordering::Equal if russian.total == 0 => Selection::Ambiguous,
        std::cmp::Ordering::Less => Selection::English,
        _ => Selection::Russian,
    }
}

#[derive(Debug)]
pub struct Analysis {
    pub russian: String,
    pub english: String,
    pub russian_score: Score,
    pub english_score: Score,
    pub selection: Selection,
}

impl Analysis {
    pub fn replacement(&self, original: &str) -> Option<String> {
        let selected = match self.selection {
            Selection::Russian => &self.russian,
            Selection::English => &self.english,
            _ => return None,
        };
        (selected != original).then(|| selected.clone())
    }
}

/// Same physical keys, two language interpretations; no spelling repair.
pub fn analyze(token: &str, boundary: Boundary) -> Option<Analysis> {
    let is_russian = |c: char| matches!(c, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё');
    if token.is_empty() || token.chars().count() > 32 {
        return None;
    }
    let (russian, english) = if token.chars().all(is_russian) {
        (token.to_string(), convert(token, Direction::Ru2Us))
    } else if token.is_ascii() {
        let projected = convert(token, Direction::Us2Ru);
        if !projected.chars().all(is_russian) {
            return None;
        }
        (projected, token.to_string())
    } else {
        return None;
    };
    let model = MODEL.get_or_init(|| Model::parse(PACKAGE)).as_ref()?;
    let russian_score = model.score(&russian, boundary);
    let english_score = model.score(&english, boundary);
    Some(Analysis {
        selection: select(russian_score, english_score),
        russian,
        english,
        russian_score,
        english_score,
    })
}

pub fn model_bytes() -> usize {
    PACKAGE.len()
}
