use std::future::Future;

pub trait Importer {
    type Error: ToString;
    type Future<'a, T>: Future<Output = Result<T, Self::Error>> + 'a
    where
        Self: 'a;

    fn import<'a, T>(&'a self, path: &'a str) -> Self::Future<'a, T>;
}

pub async fn import<T>(_path: &str) -> Result<T, String>
where
    T: for<'a> From<&'a str>,
{
    Ok(T::from(""))
}
