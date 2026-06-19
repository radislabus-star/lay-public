use super::super::super::{active_nanda_trace, active_nanda_trace_text};

pub(crate) fn record_nanda_trace_if_enabled(original: &str, replacement: &str) {
    if !active_nanda_trace() {
        return;
    }
    let trace = lay::nanda_wave::run_wave_trace(original);
    lay::nanda_wave::journal::record_trace_with_text_policy(
        "runtime:typing-assist",
        "typing-assist",
        &trace,
        Some(replacement),
        active_nanda_trace_text(),
    );
}
