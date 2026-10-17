use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, TryLockError};
use std::thread::JoinHandle;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) enum DeviceKind {
    Keyboard,
    Pointer,
}

pub(super) type DeviceKey = (DeviceKind, PathBuf);
pub(super) type Listeners = HashMap<DeviceKey, JoinHandle<()>>;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum OutputState {
    Ready,
    Recovered,
    Skipped,
}

/// Availability only: never retries an edit or replaces an existing device.
pub(super) fn recover_output<T>(
    shared: &Mutex<Option<T>>,
    detect_only: bool,
    shutdown: &AtomicBool,
    create: impl FnOnce() -> io::Result<T>,
) -> io::Result<OutputState> {
    if detect_only || shutdown.load(Ordering::Acquire) {
        return Ok(OutputState::Skipped);
    }
    let mut slot = match shared.try_lock() {
        Ok(slot) => slot,
        Err(TryLockError::WouldBlock) => return Ok(OutputState::Skipped),
        Err(TryLockError::Poisoned(error)) => error.into_inner(),
    };
    if slot.is_some() {
        return Ok(OutputState::Ready);
    }
    if shutdown.load(Ordering::Acquire) {
        return Ok(OutputState::Skipped);
    }
    let device = create()?;
    if shutdown.load(Ordering::Acquire) {
        return Ok(OutputState::Skipped);
    }
    *slot = Some(device);
    Ok(OutputState::Recovered)
}

pub(super) fn discover_paths(
    explicit: Option<&Path>,
    keyboards: impl FnOnce() -> io::Result<Vec<PathBuf>>,
    pointers: impl FnOnce() -> io::Result<Vec<PathBuf>>,
) -> (io::Result<Vec<PathBuf>>, io::Result<Vec<PathBuf>>) {
    let keyboard_paths = match explicit {
        Some(path) => Ok(vec![path.to_path_buf()]),
        None => keyboards(),
    };
    // A missing keyboard must not prevent pointer discovery.
    (keyboard_paths, pointers())
}

/// Discovery misses retain live readers. Only finished threads may be replaced.
pub(super) fn refresh_listeners(
    listeners: &mut Listeners,
    desired: &[DeviceKey],
    shutdown: &AtomicBool,
    mut spawn: impl FnMut(&DeviceKey) -> io::Result<JoinHandle<()>>,
) -> Vec<(DeviceKey, io::Result<()>)> {
    let finished: Vec<_> = listeners
        .iter()
        .filter(|(_, handle)| handle.is_finished())
        .map(|(key, _)| key.clone())
        .collect();
    for key in finished {
        if let Some(handle) = listeners.remove(&key) {
            if handle.join().is_err() {
                eprintln!("lay: input listener terminated unexpectedly: {key:?}");
            }
        }
    }
    let mut attempts = Vec::new();
    for key in desired {
        if shutdown.load(Ordering::Acquire) {
            break;
        }
        if listeners.contains_key(key) {
            continue;
        }
        let outcome = match spawn(key) {
            Ok(handle) => {
                listeners.insert(key.clone(), handle);
                Ok(())
            }
            Err(error) => Err(error),
        };
        attempts.push((key.clone(), outcome));
    }
    attempts
}

#[cfg(test)]
#[path = "tests/device_access.rs"]
mod tests;
