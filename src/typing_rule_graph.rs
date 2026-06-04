//! Registry of typing-assist rules.
//!
//! The pipeline owns ordering and arbitration. Individual rule modules own
//! language logic. This registry is the only place that maps a stable rule id to
//! its family and execution function.

#[path = "typing_rule_graph/builders.rs"]
mod builders;
#[path = "typing_rule_graph/definitions.rs"]
mod definitions;
#[path = "typing_rule_graph/ids.rs"]
pub(crate) mod ids;
#[path = "typing_rule_graph/priorities.rs"]
pub(crate) mod priorities;
#[path = "typing_rule_graph/registry.rs"]
mod registry;
#[path = "typing_rule_graph/rules.rs"]
pub(crate) mod rules;
#[path = "typing_rule_graph/types.rs"]
mod types;
#[path = "typing_rule_graph/weights.rs"]
mod weights;

pub(crate) use registry::{
    find_typing_rule, typing_rule_definitions, typing_rule_enabled_without_auto_replace,
    typing_rule_family, typing_rule_required_safety,
};
pub(crate) use types::TypingRuleContext;
pub(crate) use types::TypingRuleRequiredSafety;
pub(crate) use weights::{typing_rule_candidate_is_safe, typing_rule_family_weight};
