use std::collections::HashMap;
use std::future::Future;
use std::sync::Weak;
use std::time::{Duration, Instant};

use dbounce::debounced;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::select_all;
use futures::stream::FuturesUnordered;
use futures::{SinkExt, StreamExt};
use litho_import::import;
use smol_str::SmolStr;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::sleep_until;

use super::Workspace;

pub enum Message {
    Register(Weak<Mutex<Workspace>>),
    Update {
        imports: HashMap<String, Duration>,
        immediately: bool,
    },
}

#[derive(Default)]
pub struct Import {
    interval: Duration,
    result: Option<Result<SmolStr, String>>,
    last_updated: Option<Instant>,
}

impl Import {
    pub fn next_refresh(&self) -> Instant {
        match self.last_updated {
            Some(last_updated) => last_updated + self.interval,
            None => Instant::now(),
        }
    }

    pub async fn refresh(&mut self, url: &str) {
        sleep_until(self.next_refresh().into()).await;
        self.result = Some(import(url).await);
        self.last_updated = Some(Instant::now());
    }
}

#[derive(Default)]
pub struct ImportState {
    workspace: Option<Weak<Mutex<Workspace>>>,
    imports: HashMap<String, Import>,
}

impl ImportState {
    pub fn update(&mut self, imports: HashMap<String, Duration>) {
        self.imports.retain(|url, _| imports.contains_key(url));

        for (url, interval) in imports.into_iter() {
            self.imports.entry(url).or_default().interval = interval;
        }
    }

    pub async fn refresh(&mut self) {
        if self.imports.is_empty() {
            return std::future::pending().await;
        }

        select_all(
            self.imports
                .iter_mut()
                .map(|(url, import)| Box::pin(import.refresh(url))),
        )
        .await;

        if let Some(workspace) = self
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.upgrade())
        {
            let imports = self
                .imports
                .iter()
                .flat_map(|(url, import)| match import.result.as_ref() {
                    Some(result) => Some((url.clone(), result.clone())),
                    None => None,
                })
                .collect();

            workspace.lock().await.update_imports(imports).await;
        }
    }
}

pub struct ImportWorker(Receiver<Message>);

impl ImportWorker {
    pub async fn work(mut self) {
        let mut state = ImportState::default();

        let (mut sender, receiver) = channel::<HashMap<String, Duration>>(16);
        let mut debounced_receiver = debounced(receiver, Duration::from_secs(1));

        loop {
            select! {
                _ = state.refresh() => {},
                imports = debounced_receiver.next() => {
                    if let Some(imports) = imports {
                        state.update(imports);
                    }
                }
                message = self.0.next() => {
                    let Some(message) = message else { return };

                    match message {
                        Message::Register(ws) => state.workspace = Some(ws),
                        Message::Update { imports, immediately } => match immediately {
                            true => state.update(imports),
                            false => {let _ = sender.send(imports).await;}
                        }
                    }
                }
            }
        }
    }
}

pub struct ImportQueue(Sender<ImportWorker>);

impl ImportQueue {
    pub fn new() -> (ImportQueue, impl Future) {
        let (sender, receiver) = channel::<ImportWorker>(1024);

        let routine = async move {
            let mut unordered = FuturesUnordered::new();
            let mut receiver = receiver;

            loop {
                select! {
                    _ = unordered.next() => {},
                    worker = receiver.next() => {
                        match worker {
                            Some(worker) => unordered.push(worker.work()),
                            None => break,
                        }
                    }
                }
            }

            unordered.collect::<Vec<_>>().await;
        };

        (ImportQueue(sender), routine)
    }

    /// Creates a new importer and adds it to the queue.
    pub fn importer(&mut self) -> Importer {
        let (sender, receiver) = channel::<Message>(1024);

        self.0.try_send(ImportWorker(receiver)).unwrap();

        Importer(sender)
    }
}

pub struct Importer(Sender<Message>);

impl Importer {
    pub async fn register(&mut self, workspace: Weak<Mutex<Workspace>>) {
        let _ = self.0.send(Message::Register(workspace)).await;
    }

    pub async fn update(&mut self, imports: &HashMap<String, Duration>) {
        let _ = self
            .0
            .send(Message::Update {
                imports: imports.clone(),
                immediately: false,
            })
            .await;
    }
}
