use super::llmwave::{self, LlmWaveMemory};
use super::options::WaveOptions;
use super::signal::LayerTrace;

pub const L4_GOAL_STATE_CELL: &str = "L4GoalStateCell32";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4LanguageScene {
    Russian,
    English,
    Mixed,
    Technical,
    Unknown,
}

impl L4LanguageScene {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Russian => "ru",
            Self::English => "en",
            Self::Mixed => "mixed",
            Self::Technical => "technical",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4EditIntent {
    Typing,
    Command,
    Quote,
    Code,
}

impl L4EditIntent {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Typing => "typing",
            Self::Command => "command",
            Self::Quote => "quote",
            Self::Code => "code",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4AllowedAction {
    Suggest,
    Wait,
    Block,
}

impl L4AllowedAction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Suggest => "suggest",
            Self::Wait => "wait",
            Self::Block => "block",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct L4SceneState {
    pub(crate) language_scene: L4LanguageScene,
    pub(crate) edit_intent: L4EditIntent,
    pub(crate) allowed_action: L4AllowedAction,
    pub(crate) confidence: f32,
    pub(crate) context_tokens: usize,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L4SceneStateInput<'a> {
    pub(crate) context_prefix: &'a str,
    pub(crate) current_word: &'a str,
    pub(crate) candidate_count: usize,
}

pub(crate) fn derive_l4_scene_state(input: L4SceneStateInput<'_>) -> L4SceneState {
    let context_tokens = llmwave::tokenize(input.context_prefix).len();
    let letters = LetterProfile::from_parts(input.context_prefix, input.current_word);
    let technical = looks_technical(input.context_prefix) || looks_technical(input.current_word);
    let language_scene = if technical {
        L4LanguageScene::Technical
    } else if letters.cyrillic > 0 && letters.latin > 0 {
        L4LanguageScene::Mixed
    } else if letters.cyrillic > 0 {
        L4LanguageScene::Russian
    } else if letters.latin > 0 {
        L4LanguageScene::English
    } else {
        L4LanguageScene::Unknown
    };

    let edit_intent = if is_command_like(input.context_prefix) {
        L4EditIntent::Command
    } else if quote_balance_open(input.context_prefix) {
        L4EditIntent::Quote
    } else if technical {
        L4EditIntent::Code
    } else {
        L4EditIntent::Typing
    };

    let current_len = input.current_word.chars().count();
    let (allowed_action, reason) = match (language_scene, edit_intent) {
        (L4LanguageScene::Unknown, _) => (L4AllowedAction::Block, "no_letter_scene"),
        (_, L4EditIntent::Code) if current_len < 4 => (L4AllowedAction::Wait, "technical_short"),
        (L4LanguageScene::English, _) if input.candidate_count == 0 => {
            (L4AllowedAction::Wait, "english_without_candidate")
        }
        (L4LanguageScene::Mixed, _) if current_len < 3 => (L4AllowedAction::Wait, "mixed_short"),
        _ => (L4AllowedAction::Suggest, "scene_allows_suggestion"),
    };

    let confidence = scene_confidence(
        language_scene,
        edit_intent,
        current_len,
        input.candidate_count,
    );
    L4SceneState {
        language_scene,
        edit_intent,
        allowed_action,
        confidence,
        context_tokens,
        reason,
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct LetterProfile {
    cyrillic: usize,
    latin: usize,
}

impl LetterProfile {
    fn from_parts(context: &str, current_word: &str) -> Self {
        let mut profile = Self::default();
        for ch in context.chars().chain(current_word.chars()) {
            if crate::keyboard::is_cyrillic_letter(ch) {
                profile.cyrillic += 1;
            } else if ch.is_ascii_alphabetic() {
                profile.latin += 1;
            }
        }
        profile
    }
}

fn looks_technical(text: &str) -> bool {
    text.chars()
        .any(|ch| matches!(ch, '/' | '\\' | '_' | '@' | ':' | '=' | '$' | '#'))
        || text.split_whitespace().any(|token| {
            token.contains('.')
                || token.contains('-')
                || token.chars().any(|ch| ch.is_ascii_digit())
        })
}

fn is_command_like(context: &str) -> bool {
    context
        .trim_start()
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '/' | ':' | '$' | '>'))
}

fn quote_balance_open(context: &str) -> bool {
    let single = context.chars().filter(|ch| *ch == '\'').count();
    let double = context.chars().filter(|ch| *ch == '"').count();
    single % 2 == 1 || double % 2 == 1
}

fn scene_confidence(
    language_scene: L4LanguageScene,
    edit_intent: L4EditIntent,
    current_len: usize,
    candidate_count: usize,
) -> f32 {
    let mut confidence: f32 = match language_scene {
        L4LanguageScene::Russian => 0.62,
        L4LanguageScene::English => 0.54,
        L4LanguageScene::Mixed => 0.58,
        L4LanguageScene::Technical => 0.66,
        L4LanguageScene::Unknown => 0.35,
    };
    if edit_intent != L4EditIntent::Typing {
        confidence += 0.08;
    }
    if current_len >= 4 {
        confidence += 0.08;
    }
    if candidate_count > 0 {
        confidence += 0.06;
    }
    confidence.clamp(0.0, 0.92)
}

pub fn derive_l4_goal_state_trace(original: &str, options: &WaveOptions) -> Option<LayerTrace> {
    if !options.llmwave_shadow() {
        return None;
    }
    let memory = llmwave::load_default_memory();
    Some(goal_state_trace_with_memory(original, &memory))
}

fn goal_state_trace_with_memory(original: &str, memory: &LlmWaveMemory) -> LayerTrace {
    let tokens = llmwave::tokenize(original);
    let scene = derive_l4_scene_state(L4SceneStateInput {
        context_prefix: original,
        current_word: tokens.last().map(String::as_str).unwrap_or_default(),
        candidate_count: 0,
    });
    if memory.is_empty() {
        return LayerTrace {
            name: L4_GOAL_STATE_CELL,
            summary: format!("state=WATCH-no-memory context_tokens={}", tokens.len()),
        };
    }
    if tokens.len() < 2 {
        return LayerTrace {
            name: L4_GOAL_STATE_CELL,
            summary: format!("state=WATCH-short-context context_tokens={}", tokens.len()),
        };
    }

    let predictions = memory.predict_phrase(&tokens.join(" "), 1, 4);
    let Some(top) = predictions.first() else {
        return LayerTrace {
            name: L4_GOAL_STATE_CELL,
            summary: format!(
                "state=WATCH-no-continuation context_tokens={}",
                tokens.len()
            ),
        };
    };
    let next = top
        .tokens
        .get(tokens.len())
        .map(String::as_str)
        .unwrap_or_default();
    LayerTrace {
        name: L4_GOAL_STATE_CELL,
        summary: format!(
            "state=READY context_tokens={} scene={} intent={} action={} next={:?} score={:.3} support={}",
            tokens.len(),
            scene.language_scene.as_str(),
            scene.edit_intent.as_str(),
            scene.allowed_action.as_str(),
            next,
            top.score,
            top.support
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l4_goal_state_reads_corpus_memory_without_applying_text() {
        let memory = LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nвечером на улице опять идёт дождь",
        );
        let trace = goal_state_trace_with_memory("на улице опять идёт", &memory);

        assert_eq!(trace.name, L4_GOAL_STATE_CELL);
        assert!(trace.summary.contains("state=READY"));
        assert!(trace.summary.contains("next=\"дождь\""));
    }

    #[test]
    fn l4_scene_waits_on_short_technical_context() {
        let state = derive_l4_scene_state(L4SceneStateInput {
            context_prefix: "file.rs ",
            current_word: "пр",
            candidate_count: 4,
        });

        assert_eq!(state.language_scene, L4LanguageScene::Technical);
        assert_eq!(state.allowed_action, L4AllowedAction::Wait);
    }

    #[test]
    fn l4_scene_suggests_russian_typing_context() {
        let state = derive_l4_scene_state(L4SceneStateInput {
            context_prefix: "на улице опять ",
            current_word: "ид",
            candidate_count: 2,
        });

        assert_eq!(state.language_scene, L4LanguageScene::Russian);
        assert_eq!(state.edit_intent, L4EditIntent::Typing);
        assert_eq!(state.allowed_action, L4AllowedAction::Suggest);
    }
}
