use super::super::super::{active_nanda_trace, active_nanda_wave_options};

pub(crate) fn record_nanda_trace_if_enabled(original: &str, replacement: &str) {
    if !active_nanda_trace() {
        return;
    }
    let original = original.to_string();
    let replacement = replacement.to_string();
    let options = active_nanda_wave_options();
    let include_text = active_nanda_trace();
    std::thread::spawn(move || {
        lay::typing_cpu::TypingCpu::record_typing_assist_trace(
            &original,
            &replacement,
            &options,
            include_text,
        );
    });
}
