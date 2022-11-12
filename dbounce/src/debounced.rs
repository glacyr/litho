use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::{FutureExt, Stream, StreamExt};

use super::{delayed, Delayed};

pub struct Debounced<S>
where
    S: Stream,
{
    stream: S,
    delay: Duration,
    pending: Option<Delayed<S::Item>>,
}

impl<S> Stream for Debounced<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(next) = self.stream.poll_next_unpin(cx) {
            match next {
                Some(next) => self.pending = Some(delayed(next, self.delay)),
                None => {
                    if self.pending.is_none() {
                        return Poll::Ready(None);
                    }
                }
            }
        }

        match self.pending.as_mut() {
            Some(pending) => match pending.poll_unpin(cx) {
                Poll::Ready(value) => {
                    let _ = self.pending.take();
                    Poll::Ready(Some(value))
                }
                Poll::Pending => Poll::Pending,
            },
            None => Poll::Pending,
        }
    }
}

pub fn debounced<S>(stream: S, delay: Duration) -> Debounced<S>
where
    S: Stream,
{
    Debounced {
        stream,
        delay,
        pending: None,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::channel::mpsc::channel;
    use futures::future::join;
    use futures::{SinkExt, StreamExt};
    use tokio::time::sleep;

    use super::debounced;

    #[tokio::test]
    async fn test_debounce() {
        let (mut sender, receiver) = channel(1024);
        let mut receiver = debounced(receiver, Duration::from_millis(100));

        join(
            async move {
                for i in 0..10 {
                    let _ = sleep(Duration::from_millis(200)).await;
                    let _ = sender.send(i).await;
                }

                eprintln!("Sender ended");
            },
            async move {
                while let Some(value) = receiver.next().await {
                    println!("Got a value: {:#?}", value);
                }

                println!("Debounce ended.");
            },
        )
        .await;
    }
}
