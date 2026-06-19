use std::path::PathBuf;

use super::log;

pub(super) fn sanitize_user_replacements() {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = PathBuf::from(home).join(lay::typing_replacements::REPLACEMENTS_PATH);
    match lay::typing_replacements::sanitize_replacement_rules_path(&path) {
        Ok(0) => {}
        Ok(removed) => log(&format!(
            "► replacements: removed {removed} unsafe learned rule(s)"
        )),
        Err(e) => log(&format!("⚠ replacements sanitize failed: {e}")),
    }
}
