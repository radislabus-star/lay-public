//! Chrome source-read coordination; no snapshot installation or text output.
use super::{NativeResult, Observation, SurroundingTextSnapshot};
use std::future::Future;
use std::time::{Duration, Instant};

/// Await the actual owned append within one deadline, without publishing it.
pub(super) async fn read_with_owned_append_resampling<R, F, P>(
    deadline: Instant,
    mut read: R,
    is_pending: P,
    resample_changed: bool,
) -> NativeResult<Observation>
where
    R: FnMut(Instant, bool) -> F,
    F: Future<Output = NativeResult<Observation>>,
    P: Fn(&SurroundingTextSnapshot) -> bool,
{
    let mut first = true;
    loop {
        if Instant::now() >= deadline {
            return Err("native_deadline");
        }
        let observation = read(deadline, first).await;
        if Instant::now() >= deadline {
            return Err("native_deadline");
        }
        let resample = match &observation {
            Ok(observation) => is_pending(&observation.snapshot),
            Err("native_document_changed") => first && resample_changed,
            Err(_) => false,
        };
        if !resample {
            return observation;
        }
        crate::trace::record(
            serde_json::json!({
                "kind": "ibus_native_document", "stage": "owned_append_resample",
                "reason": observation.as_ref().err().copied().unwrap_or("older_owned_prefix"),
            })
            .to_string(),
        );
        first = false;
        // Yield even if the transport has an immediately ready old response.
        // Both the next actual read and this yield keep the original deadline.
        async_io::Timer::at((Instant::now() + Duration::from_millis(1)).min(deadline)).await;
    }
}
