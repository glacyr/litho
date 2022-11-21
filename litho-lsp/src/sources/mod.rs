use lsp_types::Url;

pub trait SourceRoot {
    type Error;

    fn walk(&self, url: &Url) -> Result<Vec<Url>, Self::Error>;
    fn read(&self, url: &Url) -> Result<String, Self::Error>;
}

impl SourceRoot for () {
    type Error = ();

    fn walk(&self, _url: &Url) -> Result<Vec<Url>, Self::Error> {
        Ok(vec![])
    }

    fn read(&self, _url: &Url) -> Result<String, Self::Error> {
        Err(())
    }
}

#[cfg(feature = "fs")]
pub mod fs;
