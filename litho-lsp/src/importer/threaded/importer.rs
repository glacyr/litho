use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use dbounce::debounced;
use futures::channel::{mpsc, oneshot};
use futures::future::join;
use futures::select;
use futures::{SinkExt, StreamExt};

use super::ImporterState;

use crate::{Importer, ImporterCallback};

pub struct Update {
    imports: HashMap<String, Duration>,
    immediately: bool,
}

pub struct ThreadedImporter(
    Option<oneshot::Sender<ImporterCallback>>,
    mpsc::Sender<Update>,
);

impl ThreadedImporter {
    pub fn new() -> (ThreadedImporter, ImporterWorker) {
        let oneshot = oneshot::channel();
        let mpsc = mpsc::channel(1024);
        (
            ThreadedImporter(Some(oneshot.0), mpsc.0),
            ImporterWorker(oneshot.1, mpsc.1),
        )
    }
}

impl Importer for ThreadedImporter {
    fn register<'a>(
        &'a mut self,
        callback: ImporterCallback,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> {
        Box::pin(async move {
            let Some(sender) = self.0.take() else {
            return
        };

            let _ = sender.send(callback);
        })
    }

    fn update<'a>(
        &'a mut self,
        imports: &'a HashMap<String, Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> {
        Box::pin(async move {
            let _ = self
                .1
                .send(Update {
                    imports: imports.clone(),
                    immediately: false,
                })
                .await;
        })
    }
}

pub struct ImporterWorker(oneshot::Receiver<ImporterCallback>, mpsc::Receiver<Update>);

impl ImporterWorker {
    pub async fn work(mut self) {
        let Ok(callback) = self.0.await else {
            return
        };

        let (mut state, state_worker) = ImporterState::new(callback);

        let (mut sender, receiver) = mpsc::channel::<HashMap<String, Duration>>(16);
        let mut debounced_receiver = debounced(receiver, Duration::from_secs(1)).fuse();

        join(state_worker.work(), async move {
            loop {
                select! {
                    imports = debounced_receiver.next() => {
                        if let Some(imports) = imports {
                            state.update(imports).await;
                        }
                    }
                    update = self.1.next() => {
                        let Some(update) = update else { return };

                        match update.immediately {
                            true => state.update(update.imports).await,
                            false => {
                                let _ = sender.send(update.imports).await;
                            }
                        }
                    }
                }
            }
        })
        .await;
    }
}
