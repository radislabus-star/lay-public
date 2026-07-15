use super::LayConfig;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const DEBUG_ACTION_LOG: u8 = 1 << 0;
const USAGE_LEARNING: u8 = 1 << 1;

static FLAGS: AtomicU8 = AtomicU8::new(0);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn publish_runtime_config(config: &LayConfig) {
    let mut flags = 0u8;
    if config.debug_action_log {
        flags |= DEBUG_ACTION_LOG;
    }
    if config.learning_log || config.nanda_precognition || config.nanda_autocorrect {
        flags |= USAGE_LEARNING;
    }
    FLAGS.store(flags, Ordering::Release);
    INITIALIZED.store(true, Ordering::Release);
}

fn current() -> u8 {
    if !INITIALIZED.load(Ordering::Acquire) {
        let _ = LayConfig::load();
    }
    FLAGS.load(Ordering::Acquire)
}

pub fn runtime_debug_action_log() -> bool {
    current() & DEBUG_ACTION_LOG != 0
}

pub fn runtime_usage_learning_enabled() -> bool {
    current() & USAGE_LEARNING != 0
}
