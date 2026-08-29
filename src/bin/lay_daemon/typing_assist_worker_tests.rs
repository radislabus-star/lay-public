use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::*;

fn slow_no_correction(_: &WordBuffer) -> Option<TypingAssistCorrection> {
    std::thread::sleep(Duration::from_millis(40));
    None
}

static SLOW_PREPARE_STARTED: AtomicBool = AtomicBool::new(false);

fn slow_started_no_correction(_: &WordBuffer) -> Option<TypingAssistCorrection> {
    SLOW_PREPARE_STARTED.store(true, Ordering::Release);
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

#[test]
fn submit_returns_pending_while_boundary_decision_runs() {
    let mut worker = TypingAssistWorker::with_prepare(slow_no_correction);
    let request_id = worker.submit(&WordBuffer::new()).expect("submitted");

    assert!(matches!(worker.poll(request_id), WorkerPoll::Pending));
    std::thread::sleep(Duration::from_millis(60));
    assert!(matches!(
        worker.poll(request_id),
        WorkerPoll::Completed(None)
    ));
}

#[test]
fn busy_worker_keeps_the_latest_boundary_snapshot() {
    SLOW_PREPARE_STARTED.store(false, Ordering::Release);
    let mut worker = TypingAssistWorker::with_prepare(slow_started_no_correction);
    worker.submit(&WordBuffer::new()).expect("first submitted");

    let wait_started = Instant::now();
    while !SLOW_PREPARE_STARTED.load(Ordering::Acquire) {
        assert!(wait_started.elapsed() < Duration::from_secs(1));
        std::thread::yield_now();
    }

    worker.submit(&WordBuffer::new()).expect("second submitted");
    let latest_id = worker
        .submit(&WordBuffer::new())
        .expect("latest snapshot must replace queued stale work");

    std::thread::sleep(Duration::from_millis(100));
    assert!(matches!(
        worker.poll(latest_id),
        WorkerPoll::Completed(None)
    ));
}
