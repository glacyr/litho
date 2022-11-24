use std::future::Future;
use std::marker::PhantomData;

use futures::channel::mpsc::Receiver;
use futures::StreamExt;
use lsp_types::{Diagnostic, Url};

use super::Importer;

use crate::WorkspaceUpdate;

pub struct Coordinator<D, F> {
    receiver: Receiver<WorkspaceUpdate>,
    importer: Importer,
    diagnostics: D,
    marker: PhantomData<F>,
}

impl<D, F> Coordinator<D, F>
where
    D: Fn(Url, Vec<Diagnostic>, Option<i32>) -> F,
    F: Future<Output = ()>,
{
    pub fn new(
        receiver: Receiver<WorkspaceUpdate>,
        importer: Importer,
        diagnostics: D,
    ) -> Coordinator<D, F> {
        Coordinator {
            receiver,
            importer,
            diagnostics,
            marker: PhantomData,
        }
    }

    pub async fn work(mut self) {
        while let Some(update) = self.receiver.next().await {
            match update {
                WorkspaceUpdate::Diagnostics {
                    url,
                    diagnostics,
                    version,
                } => {
                    (self.diagnostics)(url, diagnostics, version).await;
                }
                WorkspaceUpdate::Imports(imports) => self.importer.update(imports).await,
            }
        }
    }
}
