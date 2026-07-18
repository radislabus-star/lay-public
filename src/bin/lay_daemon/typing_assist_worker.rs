use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex};

use lay::word_buffer::WordBuffer;

use super::{prepare_typing_assist_after_space, TypingAssistCorrection};

struct WorkRequest {
    id: u64,
    buffer: WordBuffer,
}

struct WorkResult {
    id: u64,
    correction: Option<TypingAssistCorrection>,
}

pub(super) enum WorkerPoll {
    Pending,
    Completed(Option<Box<TypingAssistCorrection>>),
}

pub(super) struct TypingAssistWorker {
    request_wake: SyncSender<()>,
    pending_request: Arc<Mutex<Option<WorkRequest>>>,
    results: Receiver<WorkResult>,
    latest_requested_id: Arc<AtomicU64>,
    next_id: u64,
}

impl TypingAssistWorker {
    pub(super) fn new() -> Self {
        Self::with_prepare(prepare_typing_assist_after_space)
    }

    fn with_prepare(prepare: fn(&WordBuffer) -> Option<TypingAssistCorrection>) -> Self {
        let (request_wake_tx, request_wake_rx) = mpsc::sync_channel::<()>(1);
        let pending_request = Arc::new(Mutex::new(None::<WorkRequest>));
        let worker_pending_request = Arc::clone(&pending_request);
        let (result_tx, result_rx) = mpsc::channel::<WorkResult>();
        let latest_requested_id = Arc::new(AtomicU64::new(0));
        let worker_latest_requested_id = Arc::clone(&latest_requested_id);
        std::thread::Builder::new()
            .name("lay-boundary-decision".to_string())
            .spawn(move || {
                while request_wake_rx.recv().is_ok() {
                    // A single slot bounds memory and collapses queued boundaries.
                    let Some(request) = worker_pending_request
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .take()
                    else {
                        continue;
                    };
                    if request.id != worker_latest_requested_id.load(Ordering::Acquire) {
                        continue;
                    }
                    let correction = prepare(&request.buffer);
                    // Work that lost its revision race must never reach the executor.
                    if request.id != worker_latest_requested_id.load(Ordering::Acquire) {
                        continue;
                    }
                    if result_tx
                        .send(WorkResult {
                            id: request.id,
                            correction,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .expect("spawn typing-assist boundary worker");
        Self {
            request_wake: request_wake_tx,
            pending_request,
            results: result_rx,
            latest_requested_id,
            next_id: 1,
        }
    }

    pub(super) fn submit(&mut self, buffer: &WordBuffer) -> Option<u64> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        self.latest_requested_id.store(id, Ordering::Release);
        *self
            .pending_request
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(WorkRequest {
            id,
            buffer: buffer.clone(),
        });
        let _ = self.request_wake.try_send(());
        Some(id)
    }

    pub(super) fn poll(&self, expected_id: u64) -> WorkerPoll {
        loop {
            match self.results.try_recv() {
                Ok(result) if result.id == expected_id => {
                    return WorkerPoll::Completed(result.correction.map(Box::new));
                }
                Ok(_) => continue,
                Err(TryRecvError::Empty) => return WorkerPoll::Pending,
                Err(TryRecvError::Disconnected) => return WorkerPoll::Completed(None),
            }
        }
    }
}

#[cfg(test)]
#[path = "typing_assist_worker_tests.rs"]
mod tests;
