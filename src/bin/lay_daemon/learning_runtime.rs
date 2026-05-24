use lay::typing_assist::REPLACEMENTS_PATH;
use lay::word_buffer::UserLearningCorrection;

use super::{active_learning_log, log};

#[path = "learning_runtime/log_file.rs"]
mod log_file;
#[path = "learning_runtime/promotion.rs"]
mod promotion;

#[cfg(test)]
pub(super) use log_file::keep_last_jsonl_lines;
pub(super) use log_file::{
    append_learning_log_to_path, append_user_correction_learning_log_to_path,
};
pub(super) use promotion::{promote_user_correction_if_repeated, LearningPromotion};

const LEARN_LOG_PATH: &str = ".local/share/lay/corrections.jsonl";
const LEARN_CANDIDATES_PATH: &str = ".local/share/lay/learning_candidates.json";
const LEARN_LOG_MAX_BYTES: u64 = 1024 * 1024;
const LEARN_LOG_KEEP_LINES: usize = 3000;
const LEARN_PROMOTION_THRESHOLD: u32 = 2;

pub(super) fn append_learning_log(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
) {
    if !active_learning_log() {
        return;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = std::path::PathBuf::from(home).join(LEARN_LOG_PATH);
    append_learning_log_to_path(&path, kind, from, to, replace_words, words);
}

pub(super) fn append_user_correction_learning_log(correction: &UserLearningCorrection) {
    if !active_learning_log() {
        return;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let home = std::path::PathBuf::from(home);
    let path = home.join(LEARN_LOG_PATH);
    append_user_correction_learning_log_to_path(&path, correction);
    match promote_user_correction_if_repeated(
        &home.join(LEARN_CANDIDATES_PATH),
        &home.join(REPLACEMENTS_PATH),
        correction,
    ) {
        LearningPromotion::Promoted { from, to } => {
            runtime_log(&format!("  learn: promoted exact rule {from:?} -> {to:?}"));
        }
        LearningPromotion::Recorded { count, from, to } => {
            runtime_log(&format!(
                "  learn: candidate {from:?} -> {to:?}, count={count}/{LEARN_PROMOTION_THRESHOLD}"
            ));
        }
        LearningPromotion::Skipped => {}
    }
}

fn runtime_log(message: &str) {
    log(message);
}
