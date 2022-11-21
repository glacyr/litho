use std::collections::HashMap;
use std::future::{ready, Future};
use std::pin::Pin;
use std::time::Duration;

use smol_str::SmolStr;

pub type ImporterCallback = Box<
    dyn FnMut(HashMap<String, Result<SmolStr, String>>) -> Pin<Box<dyn Future<Output = ()>>>
        + Send
        + Sync,
>;

pub trait Importer {
    fn register<'a>(
        &'a mut self,
        callback: ImporterCallback,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>;

    fn update<'a>(
        &'a mut self,
        imports: &'a HashMap<String, Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>;
}

#[cfg(feature = "threaded")]
pub mod threaded;

impl Importer for () {
    fn register<'a>(
        &'a mut self,
        _callback: ImporterCallback,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> {
        Box::pin(ready(()))
    }

    fn update<'a>(
        &'a mut self,
        _imports: &'a HashMap<String, Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> {
        Box::pin(ready(()))
    }
}
