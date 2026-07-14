use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};

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
    requests: SyncSender<WorkRequest>,
    results: Receiver<WorkResult>,
    next_id: u64,
}

impl TypingAssistWorker {
    pub(super) fn new() -> Self {
        Self::with_prepare(prepare_typing_assist_after_space)
    }

    fn with_prepare(prepare: fn(&WordBuffer) -> Option<TypingAssistCorrection>) -> Self {
        let (request_tx, request_rx) = mpsc::sync_channel::<WorkRequest>(1);
        let (result_tx, result_rx) = mpsc::channel::<WorkResult>();
        std::thread::Builder::new()
            .name("lay-boundary-decision".to_string())
            .spawn(move || {
                while let Ok(request) = request_rx.recv() {
                    let correction = prepare(&request.buffer);
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
            requests: request_tx,
            results: result_rx,
            next_id: 1,
        }
    }

    pub(super) fn submit(&mut self, buffer: &WordBuffer) -> Option<u64> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        match self.requests.try_send(WorkRequest {
            id,
            buffer: buffer.clone(),
        }) {
            Ok(()) => Some(id),
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => None,
        }
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
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    fn slow_no_correction(_: &WordBuffer) -> Option<TypingAssistCorrection> {
        std::thread::sleep(Duration::from_millis(40));
        None
    }

    #[test]
    fn submit_never_runs_boundary_decision_on_key_thread() {
        let mut worker = TypingAssistWorker::with_prepare(slow_no_correction);
        let started = Instant::now();
        let request_id = worker.submit(&WordBuffer::new()).expect("submitted");

        assert!(started.elapsed() < Duration::from_millis(10));
        assert!(matches!(worker.poll(request_id), WorkerPoll::Pending));
        std::thread::sleep(Duration::from_millis(60));
        assert!(matches!(
            worker.poll(request_id),
            WorkerPoll::Completed(None)
        ));
    }
}
