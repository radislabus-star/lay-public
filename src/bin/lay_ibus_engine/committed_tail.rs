use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommittedTailPlan {
    backspaces: u32,
    replacement: String,
    original_token: String,
}

impl LayIbusEngine {
    pub(super) async fn accept_stuck_tail(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.tail_buffer.trim().is_empty() {
            return Ok(false);
        }
        let mut text = self.selected_visible_completion_suffix();
        if text.is_empty() {
            return Ok(false);
        }
        if with_space {
            text.push(' ');
        }
        self.clear_preedit(emitter).await?;
        Self::commit_text(emitter, make_ibus_text(text.clone()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.sync_tail_after_stuck_completion(&text);
        Ok(true)
    }

    fn sync_tail_after_stuck_completion(&mut self, text: &str) {
        for ch in text.chars() {
            self.push_tail_char(ch);
        }
    }

    pub(super) async fn toggle_committed_tail(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let Some((backspaces, mut replacement)) = self.committed_tail_toggle_replacement() else {
            return Ok(false);
        };
        let trailing_ws = self.tail_trailing_whitespace_chars();
        for _ in 0..trailing_ws {
            replacement.push(' ');
        }
        let backspaces = backspaces.saturating_add(trailing_ws as u32);
        let handled = self
            .replace_committed_tail(emitter, backspaces, replacement.clone())
            .await?;
        if handled {
            self.sync_layout_after_manual_toggle(&replacement);
            self.trace_key("double_shift_committed_tail", 0, 0, true, None);
        }
        Ok(handled)
    }

    fn committed_tail_toggle_replacement(&self) -> Option<(u32, String)> {
        let token = self.last_tail_token_text();
        if token.is_empty() {
            return None;
        }

        if let Some(recovered) = self.recover_missing_initial_layout_toggle(&token) {
            return Some(recovered);
        }

        let converted = self.double_shift_replacement(&token);
        (converted != token).then(|| (token.chars().count() as u32, converted))
    }

    fn recover_missing_initial_layout_toggle(&self, token: &str) -> Option<(u32, String)> {
        if token.chars().count() < 5 || !token.chars().all(char::is_alphabetic) {
            return None;
        }

        let normal = self.double_shift_replacement(token);
        let mut best: Option<(f32, String, String)> = None;
        for prefix in missing_initial_prefixes(token) {
            let candidate = format!("{prefix}{token}");
            let replacement = self.double_shift_replacement(&candidate);
            if replacement.is_empty() || replacement == candidate || replacement == normal {
                continue;
            }
            let score = replacement_quality_score(&replacement);
            if score < 0.98 {
                continue;
            }
            if !known_layout_recovery_replacement(&replacement) {
                continue;
            }
            let better_than_best = match best.as_ref() {
                Some((best_score, _, _)) => score > *best_score,
                None => true,
            };
            if better_than_best {
                best = Some((score, candidate, replacement));
            }
        }

        let (_, candidate, replacement) = best?;
        Some((candidate.chars().count() as u32, replacement))
    }

    pub(super) async fn autocorrect_committed_tail_space(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let started_at = Instant::now();
        let Some(plan) = self.committed_tail_boundary_replacement(true) else {
            return Ok(false);
        };
        let handled = self
            .replace_committed_tail(emitter, plan.backspaces, plan.replacement.clone())
            .await?;
        if handled {
            self.sync_layout_after_committed_text(&plan.replacement);
            lay::action_log::record_action(
                "ime-typing-assist",
                &format!("{} ", plan.original_token),
                &plan.replacement,
                1,
                1,
                started_at.elapsed().as_millis(),
                true,
            );
        }
        Ok(handled)
    }

    pub(super) async fn autocorrect_committed_tail_enter(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let Some(plan) = self.committed_tail_boundary_replacement(false) else {
            return Ok(false);
        };
        let handled = self
            .replace_committed_tail(emitter, plan.backspaces, plan.replacement.clone())
            .await?;
        if handled {
            self.sync_layout_after_committed_text(&plan.replacement);
        }
        Ok(handled)
    }

    fn committed_tail_boundary_replacement(
        &self,
        include_separator: bool,
    ) -> Option<CommittedTailPlan> {
        let token = self.last_tail_token_text();
        if token.is_empty() {
            return None;
        }
        if !self.committed_tail_visible_token_matches(&token) {
            return None;
        }
        let original = format!("{token} ");
        let replacement = self.autocorrect_committed_tail_text(&original)?;
        if replacement == original {
            return None;
        }
        let replacement = if include_separator {
            replacement
        } else {
            replacement
                .trim_end_matches(char::is_whitespace)
                .to_string()
        };
        Some(CommittedTailPlan {
            backspaces: token.chars().count() as u32,
            replacement,
            original_token: token,
        })
    }
}

fn missing_initial_prefixes(token: &str) -> impl Iterator<Item = char> {
    let ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    let prefixes: &'static str = if ascii && token.chars().all(|ch| ch.is_ascii_lowercase()) {
        "abcdefghijklmnopqrstuvwxyz"
    } else if ascii && token.chars().all(|ch| ch.is_ascii_uppercase()) {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    } else if ascii {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    } else {
        "абвгдеёжзийклмнопрстуфхцчшщъыьэюяАБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ"
    };
    prefixes.chars()
}

fn replacement_quality_score(replacement: &str) -> f32 {
    let has_cyrillic = replacement
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    let has_ascii = replacement.chars().any(|ch| ch.is_ascii_alphabetic());
    if has_cyrillic && !has_ascii {
        lay::quality::score(replacement, "ru")
    } else if has_ascii && !has_cyrillic {
        lay::quality::score(replacement, "en")
    } else {
        0.0
    }
}

fn known_layout_recovery_replacement(replacement: &str) -> bool {
    let word = replacement.trim().to_lowercase();
    !word.is_empty()
        && word
            .chars()
            .all(|ch| ('а'..='я').contains(&ch) || ch == 'ё')
        && (lay::lexicon::is_common_ru_word(&word)
            || lay::russian_lexicon::is_known_russian_word_or_form(&word))
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
    use crate::text::make_ibus_text;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                auto_replace: true,
                typing_assist: true,
                auto_switch_layout: true,
                correction_safety: "experimental".to_string(),
                nanda_autocorrect: true,
                nanda_precognition: true,
                ..LayConfig::default()
            },
        )
    }

    #[test]
    fn stuck_completion_appends_suffix_to_tail_memory() {
        let mut engine = engine();
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }

        engine.sync_tail_after_stuck_completion("ерка ");

        assert_eq!(engine.tail_buffer, "проверка ");
        assert_eq!(engine.preedit_fast.token(), "");
    }

    #[test]
    fn enter_boundary_uses_completed_tail_autocorrect_without_inserting_space() {
        let mut engine = engine();
        for ch in "fвтозамена".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.committed_tail_boundary_replacement(false),
            Some(super::CommittedTailPlan {
                backspaces: 10,
                replacement: "автозамена".to_string(),
                original_token: "fвтозамена".to_string()
            })
        );
    }

    #[test]
    fn space_boundary_repairs_duplicate_latin_prefix_before_russian_word() {
        let mut engine = engine();
        for ch in "fавтозамена".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.committed_tail_boundary_replacement(true),
            Some(super::CommittedTailPlan {
                backspaces: 11,
                replacement: "автозамена ".to_string(),
                original_token: "fавтозамена".to_string()
            })
        );
    }

    #[test]
    fn committed_tail_boundary_rejects_suffix_when_visible_token_has_prefix() {
        let mut engine = engine();
        for ch in "jn".chars() {
            engine.push_tail_char(ch);
        }
        let visible = make_ibus_text("don".to_string());
        engine.update_surrounding_text(&visible, 3, 3);

        assert_eq!(engine.committed_tail_boundary_replacement(true), None);
    }

    #[test]
    fn committed_tail_boundary_allows_matching_visible_token() {
        let mut engine = engine();
        for ch in "ghbdtn".chars() {
            engine.push_tail_char(ch);
        }
        let visible = make_ibus_text("ghbdtn".to_string());
        engine.update_surrounding_text(&visible, 6, 6);

        assert_eq!(
            engine.committed_tail_boundary_replacement(true),
            Some(super::CommittedTailPlan {
                backspaces: 6,
                replacement: "привет ".to_string(),
                original_token: "ghbdtn".to_string()
            })
        );
    }

    #[test]
    fn double_shift_recovers_missing_initial_ascii_layout_letter() {
        let mut engine = engine();
        for ch in "hbdtn".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.committed_tail_toggle_replacement(),
            Some((6, "привет".to_string()))
        );
    }

    #[test]
    fn double_shift_recovers_missing_initial_autozamena_letter() {
        let mut engine = engine();
        for ch in "dnjpfvtyf".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.committed_tail_toggle_replacement(),
            Some((10, "автозамена".to_string()))
        );
    }

    #[test]
    fn double_shift_does_not_recover_missing_initial_for_short_ascii_tail() {
        let mut engine = engine();
        for ch in "в ima".chars() {
            engine.push_tail_char(ch);
        }

        let (backspaces, replacement) = engine
            .committed_tail_toggle_replacement()
            .expect("plain double shift still toggles the short tail");

        assert_eq!(backspaces, 3);
        assert_ne!(replacement, "dime");
    }
}
