use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::{select, Either};
use futures::{SinkExt, StreamExt};
use litho_import::import;
use smol_str::SmolStr;
use tokio::sync::Mutex;
use tokio::time::sleep_until;

pub struct Import {
    interval: Duration,
    result: Arc<Mutex<Option<Result<SmolStr, String>>>>,
    sender: Sender<Duration>,
}

impl Import {
    pub fn new(url: String, interval: Duration, refresh: Sender<()>) -> (Import, ImportWorker) {
        let (sender, receiver) = channel(1024);
        let import = Import {
            interval,
            result: Arc::new(Mutex::new(None)),
            sender,
        };
        let worker = ImportWorker::new(
            url,
            interval,
            Arc::downgrade(&import.result),
            receiver,
            refresh,
        );
        (import, worker)
    }

    pub async fn result(&self) -> impl Deref<Target = Option<Result<SmolStr, String>>> + '_ {
        self.result.lock().await
    }

    pub async fn update(&mut self, interval: Duration) {
        if self.interval != interval {
            self.interval = interval;
            let _ = self.sender.send(interval).await;
        }
    }
}

pub struct ImportWorker {
    url: String,
    interval: Duration,
    last_updated: Option<Instant>,
    result: Weak<Mutex<Option<Result<SmolStr, String>>>>,
    receiver: Receiver<Duration>,
    refresh: Sender<()>,
}

impl ImportWorker {
    pub fn new(
        url: String,
        interval: Duration,
        result: Weak<Mutex<Option<Result<SmolStr, String>>>>,
        receiver: Receiver<Duration>,
        refresh: Sender<()>,
    ) -> ImportWorker {
        ImportWorker {
            url,
            interval,
            last_updated: None,
            result,
            receiver,
            refresh,
        }
    }

    pub fn next_refresh(&self) -> Instant {
        match self.last_updated {
            Some(last_updated) => last_updated + self.interval,
            None => Instant::now(),
        }
    }

    pub async fn work(mut self) {
        loop {
            let result = select(
                Box::pin(sleep_until(self.next_refresh().into())),
                self.receiver.next(),
            )
            .await;

            match result {
                Either::Left(_) => {
                    let result = import::<SmolStr>(&self.url).await;
                    self.last_updated = Some(Instant::now());

                    let Some(mutex) = self.result.upgrade() else {
                        return
                    };
                    mutex.lock().await.replace(result);
                    let _ = self.refresh.send(()).await;
                }
                Either::Right((Some(interval), _)) => {
                    self.interval = interval;
                }
                Either::Right((None, _)) => return,
            }
        }
    }
}
