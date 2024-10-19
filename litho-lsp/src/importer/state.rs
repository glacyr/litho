use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::join;
use futures::lock::Mutex;
use futures::{SinkExt, StreamExt};

use super::{ImportTracker, ImportWorker};

use crate::{Imports, Workspace};

pub struct ImporterState {
    imports: Arc<Mutex<HashMap<String, ImportTracker>>>,
    sender: Sender<ImportWorker>,
    refresh: Sender<()>,
}

impl ImporterState {
    pub fn new(workspace: Weak<Mutex<Workspace>>) -> (ImporterState, ImporterStateWorker) {
        let (sender, receiver) = channel(1024);
        let (refresh_sender, refresh_receiver) = channel(1024);

        let imports = Arc::new(Mutex::new(HashMap::new()));
        let weak_imports = Arc::downgrade(&imports);

        (
            ImporterState {
                imports,
                sender,
                refresh: refresh_sender,
            },
            ImporterStateWorker {
                imports: weak_imports,
                workspace,
                workers: receiver,
                refresh: refresh_receiver,
            },
        )
    }

    pub async fn update(&mut self, imports: Imports) {
        let mut self_imports = self.imports.lock().await;

        self_imports.retain(|url, _| imports.contains_key(url));

        for (url, import) in imports.into_iter() {
            match self_imports.entry(url.clone()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().update(import).await;
                }
                Entry::Vacant(entry) => {
                    entry.insert({
                        let (tracker, worker) =
                            ImportTracker::new(url, import, self.refresh.clone());
                        let _ = self.sender.send(worker).await;
                        tracker
                    });
                }
            }
        }

        let _ = self.refresh.send(()).await;
    }
}

pub struct ImporterStateWorker {
    imports: Weak<Mutex<HashMap<String, ImportTracker>>>,
    workspace: Weak<Mutex<Workspace>>,
    workers: Receiver<ImportWorker>,
    refresh: Receiver<()>,
}

impl ImporterStateWorker {
    pub async fn work(mut self) {
        join(
            self.workers
                .for_each_concurrent(None, |worker| worker.work()),
            async move {
                while let Some(_) = self.refresh.next().await {
                    let Some(imports) = self.imports.upgrade() else {
                        return;
                    };

                    let mut results = HashMap::new();

                    let imports = imports.lock().await;

                    for (url, import) in imports.iter() {
                        let Some(result) = &*import.result().await else {
                            continue;
                        };

                        results.insert(url.clone(), result.clone());
                    }

                    let Some(workspace) = self.workspace.upgrade() else {
                        return;
                    };

                    workspace.lock().await.update_imports(results).await;
                }
            },
        )
        .await;
    }
}
