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
