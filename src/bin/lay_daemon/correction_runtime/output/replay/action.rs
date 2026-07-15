use lay::action_log::RecentActionGateTrace;
use lay::text_edit::{AuthorizedEdit, TextReplacement};

use super::super::super::super::log;
use super::super::context::ManualOutputCommon;

pub(super) fn manual_replay_action(
    ctx: &ManualOutputCommon<'_>,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<AuthorizedEdit> {
    let plan = TextReplacement {
        move_left: 0,
        backspaces: ctx.n_backspaces,
        insert: ctx.mapped_target.to_string(),
        move_right: 0,
    };
    let edit_action = lay::text_edit::plan_manual_edit(
        "manual-replay",
        1000,
        ctx.mapped_orig,
        ctx.mapped_target,
        plan,
        ctx.words_orig,
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::MANUAL_TEXT_REPLACE,
        input_gate,
    );
    let backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        edit_action,
    );
    let backend = backend_action.backend;
    let reason = backend_action.reason;
    if let Some(authorized_edit) = backend_action.into_authorized() {
        return Some(authorized_edit);
    }
    log(&format!(
        "⚠ manual replay blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
        reason,
        backend.as_str(),
        ctx.mapped_orig,
        ctx.mapped_target
    ));
    None
}
