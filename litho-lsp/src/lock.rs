use std::future::Future;
use std::ops::DerefMut;
use std::pin::Pin;

pub trait Lock<T>: Send + Sync {
    type Guard<'a>: DerefMut<Target = T>
    where
        Self: 'a,
        T: 'a;

    fn lock(&self) -> Pin<Box<dyn Future<Output = Self::Guard<'_>> + Send + '_>>;
}

mod fut {
    use std::future::Future;
    use std::pin::Pin;

    use futures::lock::{Mutex, MutexGuard};

    use super::Lock;

    impl<T> Lock<T> for Mutex<T>
    where
        T: Send,
    {
        type Guard<'a> = MutexGuard<'a, T> where T: 'a;

        fn lock(&self) -> Pin<Box<dyn Future<Output = Self::Guard<'_>> + Send + '_>> {
            Box::pin(self.lock())
        }
    }
}

// mod sync {
//     use std::future::{ready, Future};
//     use std::pin::Pin;
//     use std::sync::{Mutex, MutexGuard};

//     use super::Lock;

//     impl<T> Lock<T> for Mutex<T>
//     where
//         T: Send,
//     {
//         type Guard<'a> = MutexGuard<'a, T>
//         where
//             T: 'a;

//         fn lock(&self) -> Pin<Box<dyn Future<Output = Self::Guard<'_>> + '_>> {
//             Box::pin(ready(self.lock().unwrap()))
//         }
//     }
// }

#[cfg(feature = "threaded")]
mod threaded {
    use std::future::Future;
    use std::pin::Pin;

    use tokio::sync::{Mutex, MutexGuard};

    use super::Lock;

    impl<T> Lock<T> for Mutex<T>
    where
        T: Send,
    {
        type Guard<'a> = MutexGuard<'a, T>
        where
            T: 'a;

        fn lock(&self) -> Pin<Box<dyn Future<Output = Self::Guard<'_>> + Send + '_>> {
            Box::pin(self.lock())
        }
    }
}
