use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::time::Instant;

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::{select, Either};
use futures::{SinkExt, StreamExt};
use litho_import::import;
use litho_types::Import;
use smol_str::SmolStr;
use tokio::sync::Mutex;
use tokio::time::sleep_until;

pub struct ImportTracker {
    import: Import,
    result: Arc<Mutex<Option<Result<SmolStr, String>>>>,
    sender: Sender<Import>,
}

impl ImportTracker {
    pub fn new(url: String, import: Import, refresh: Sender<()>) -> (ImportTracker, ImportWorker) {
        let (sender, receiver) = channel(1024);
        let tracker = ImportTracker {
            import: import.clone(),
            result: Arc::new(Mutex::new(None)),
            sender,
        };
        let worker = ImportWorker::new(
            url,
            import,
            Arc::downgrade(&tracker.result),
            receiver,
            refresh,
        );
        (tracker, worker)
    }

    pub async fn result(&self) -> impl Deref<Target = Option<Result<SmolStr, String>>> + '_ {
        self.result.lock().await
    }

    pub async fn update(&mut self, import: Import) {
        if self.import != import {
            self.import = import.clone();
            let _ = self.sender.send(import).await;
        }
    }
}

pub struct ImportWorker {
    url: String,
    import: Import,
    last_updated: Option<Instant>,
    result: Weak<Mutex<Option<Result<SmolStr, String>>>>,
    receiver: Receiver<Import>,
    refresh: Sender<()>,
}

impl ImportWorker {
    pub fn new(
        url: String,
        import: Import,
        result: Weak<Mutex<Option<Result<SmolStr, String>>>>,
        receiver: Receiver<Import>,
        refresh: Sender<()>,
    ) -> ImportWorker {
        ImportWorker {
            url,
            import,
            last_updated: None,
            result,
            receiver,
            refresh,
        }
    }

    pub fn next_refresh(&self) -> Instant {
        match self.last_updated {
            Some(last_updated) => last_updated + self.import.refresh,
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
                    let result = import::<SmolStr>(
                        &self.url,
                        self.import
                            .headers
                            .iter()
                            .flat_map(|header| {
                                Some((
                                    header.name.clone().try_into().ok()?,
                                    header.value.clone().try_into().ok()?,
                                ))
                            })
                            .collect(),
                    )
                    .await;
                    self.last_updated = Some(Instant::now());

                    let Some(mutex) = self.result.upgrade() else {
                        return
                    };
                    mutex.lock().await.replace(result);
                    let _ = self.refresh.send(()).await;
                }
                Either::Right((Some(import), _)) => {
                    self.import = import;
                }
                Either::Right((None, _)) => return,
            }
        }
    }
}
