//! Shared runtime configuration.
//!
//! The daemon, GNOME tray and future desktop frontends must use one config
//! schema. Keep the schema here instead of duplicating it in desktop-specific
//! adapters.

mod active;
mod defaults;
mod load;
mod pipeline;
mod types;

pub use crate::text_backend::TextBackendPreference;
pub use defaults::{default_typing_assist_pipeline, default_typing_assist_rules};
pub(crate) use pipeline::sort_typing_assist_pipeline;
pub use pipeline::{
    normalize_typing_assist_pipeline, typing_assist_pipeline_for_auto_replace,
    typing_assist_pipeline_for_policy,
};
pub use types::{CorrectionEngine, CorrectionSafety, LayConfig, TypingAssistRuleConfig};

pub const CONFIG_PATH: &str = ".config/lay/config.json";

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
