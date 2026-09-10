use std::future::poll_fn;
use std::pin::Pin;
use std::task::Poll;

use ordered_stream::{Join, OrderedStream, PollResult};

/// The only admitted merge primitive for streams sharing one zbus connection.
/// `Sequence` stays opaque inside the `OrderedStream` contract.
pub(crate) fn join_ordered_streams<A, B>(left: A, right: B) -> Join<A, B>
where
    A: OrderedStream,
    B: OrderedStream<Data = A::Data, Ordering = A::Ordering>,
{
    ordered_stream::join(left, right)
}

/// Drain one item from an already joined observer stream. `NoneBefore` is not
/// valid for an unbounded poll and therefore terminates the observer fail-closed.
pub(crate) async fn next_ordered_message<S>(
    mut stream: Pin<&mut S>,
) -> Option<zbus::Result<zbus::Message>>
where
    S: OrderedStream<Ordering = zbus::message::Sequence, Data = zbus::Result<zbus::Message>>
        + ?Sized,
{
    poll_fn(|cx| match stream.as_mut().poll_next_before(cx, None) {
        Poll::Ready(PollResult::Item { data, .. }) => Poll::Ready(Some(data)),
        Poll::Ready(PollResult::Terminated | PollResult::NoneBefore) => Poll::Ready(None),
        Poll::Pending => Poll::Pending,
    })
    .await
}
