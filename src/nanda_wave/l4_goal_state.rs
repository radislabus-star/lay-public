use super::llmwave::{self, LlmWaveMemory};
use super::options::WaveOptions;
use super::signal::LayerTrace;

pub const L4_GOAL_STATE_CELL: &str = "L4GoalStateCell32";

pub fn derive_l4_goal_state_trace(original: &str, options: &WaveOptions) -> Option<LayerTrace> {
    if !options.llmwave_shadow() {
        return None;
    }
    let memory = llmwave::load_default_memory();
    Some(goal_state_trace_with_memory(original, &memory))
}

fn goal_state_trace_with_memory(original: &str, memory: &LlmWaveMemory) -> LayerTrace {
    let tokens = llmwave::tokenize(original);
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
            "state=READY context_tokens={} next={:?} score={:.3} support={}",
            tokens.len(),
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
}
