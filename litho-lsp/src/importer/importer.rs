use std::collections::HashMap;
use std::sync::Weak;
use std::time::Duration;

use dbounce::debounced;
use futures::channel::{mpsc, oneshot};
use futures::future::join;
use futures::select;
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use super::ImporterState;

use crate::Workspace;

pub struct Update {
    imports: HashMap<String, Duration>,
    immediately: bool,
}

pub struct Importer(
    Option<oneshot::Sender<Weak<Mutex<Workspace>>>>,
    mpsc::Sender<Update>,
);

impl Importer {
    pub fn new() -> (Importer, ImporterWorker) {
        let oneshot = oneshot::channel();
        let mpsc = mpsc::channel(1024);
        (
            Importer(Some(oneshot.0), mpsc.0),
            ImporterWorker(oneshot.1, mpsc.1),
        )
    }

    pub async fn register(&mut self, workspace: Weak<Mutex<Workspace>>) {
        let Some(sender) = self.0.take() else {
            return
        };

        let _ = sender.send(workspace);
    }

    pub async fn update(&mut self, imports: &HashMap<String, Duration>) {
        let _ = self
            .1
            .send(Update {
                imports: imports.clone(),
                immediately: false,
            })
            .await;
    }
}

pub struct ImporterWorker(
    oneshot::Receiver<Weak<Mutex<Workspace>>>,
    mpsc::Receiver<Update>,
);

impl ImporterWorker {
    pub async fn work(mut self) {
        let Ok(workspace) = self.0.await else {
            return
        };

        let (mut state, state_worker) = ImporterState::new(workspace);

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
