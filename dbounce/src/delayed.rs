use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::FutureExt;
use tokio::time::{sleep, Sleep};

pub struct Delayed<T> {
    sleep: Pin<Box<Sleep>>,
    value: Option<T>,
}

impl<T> Unpin for Delayed<T> {}

impl<T> Future for Delayed<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.sleep.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(self.value.take().unwrap()),
        }
    }
}

pub fn delayed<T>(value: T, duration: Duration) -> Delayed<T> {
    Delayed {
        sleep: Box::pin(sleep(duration)),
        value: Some(value),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::delayed;

    #[tokio::test]
    async fn test_delay() {
        assert_eq!(delayed(42, Duration::from_secs(1)).await, 42);
    }
}
