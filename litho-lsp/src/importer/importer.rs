use std::sync::Weak;
use std::time::Duration;

use dbounce::debounced;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::join;
use futures::lock::Mutex;
use futures::select;
use futures::{SinkExt, StreamExt};

use super::ImporterState;

use crate::{Imports, Workspace};

pub struct Update {
    imports: Imports,
    immediately: bool,
}

pub struct Importer(Sender<Update>);

impl Importer {
    pub fn new(workspace: Weak<Mutex<Workspace>>) -> (Importer, ImporterWorker) {
        let mpsc = channel(1024);

        (Importer(mpsc.0), ImporterWorker(workspace, mpsc.1))
    }

    pub async fn update(&mut self, imports: Imports) -> () {
        let _ = self
            .0
            .send(Update {
                imports,
                immediately: false,
            })
            .await;
    }
}

pub struct ImporterWorker(Weak<Mutex<Workspace>>, Receiver<Update>);

impl ImporterWorker {
    pub async fn work(mut self) {
        let (mut state, state_worker) = ImporterState::new(self.0);

        let (mut sender, receiver) = channel::<Imports>(16);
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
