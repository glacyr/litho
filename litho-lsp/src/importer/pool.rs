use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::StreamExt;

use super::{Importer, ImporterWorker};

pub struct ImporterPool(Sender<ImporterWorker>);

impl ImporterPool {
    pub fn new() -> (ImporterPool, ImporterPoolWorker) {
        let (sender, receiver) = channel::<ImporterWorker>(1024);

        (ImporterPool(sender), ImporterPoolWorker(receiver))
    }

    pub fn importer(&mut self) -> Importer {
        let (importer, worker) = Importer::new();

        self.0.try_send(worker).unwrap();

        importer
    }
}

pub struct ImporterPoolWorker(Receiver<ImporterWorker>);

impl ImporterPoolWorker {
    pub async fn work(self) {
        self.0
            .for_each_concurrent(None, |worker| worker.work())
            .await
    }
}