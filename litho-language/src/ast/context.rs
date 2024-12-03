use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use arbitrary::Arbitrary;

pub trait ContextValue<'a>: Borrow<str> + Clone {
    type Shared<T>: Deref<Target = T> + AsPtr + Clone
    where
        T: 'a;
    type List<T>: Deref<Target = [T]> + Extend<T>
    where
        T: 'a;
}

pub trait Lives<T> {}

impl<A, B> Lives<A> for B {}

pub trait Iter<T> {
    type Iter<'a>: Iterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    fn iter(&self) -> Self::Iter<'_>;
}

impl<T> Iter<T> for Vec<T> {
    type Iter<'a> = std::slice::Iter<'a, T> where Self: 'a, T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}

pub trait Context<'a, T>
where
    T: ContextValue<'a>,
{
    fn shared<U>(&self, value: U) -> Shared<'a, T, U>
    where
        U: 'a;

    fn list<U>(&self) -> List<'a, T, U>
    where
        U: Clone + 'a;
}

pub struct List<'a, T, U>(T::List<U>)
where
    T: ContextValue<'a>,
    U: Clone + 'a;

impl<'a, T, U> List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Clone,
{
    pub fn new(value: T::List<U>) -> List<'a, T, U> {
        List(value)
    }
}

impl<'a, T, U> Extend<U> for List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Clone,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<'a, T, U> Deref for List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Clone,
{
    type Target = [U];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a, T, U> Clone for List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Clone,
{
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<'a, 'b, T, U> IntoIterator for &'b List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Clone,
{
    type Item = &'b U;
    type IntoIter = std::slice::Iter<'b, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'arbitrary, T, U> Arbitrary<'arbitrary> for List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Arbitrary<'arbitrary> + Clone,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'arbitrary>) -> arbitrary::Result<Self> {
        todo!()
    }
}

impl<'a, T, U> Debug for List<'a, T, U>
where
    T: ContextValue<'a>,
    U: Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

pub struct Shared<'a, T, U>(T::Shared<U>)
where
    T: ContextValue<'a>,
    U: 'a;

impl<'a, T, U> Shared<'a, T, U>
where
    T: ContextValue<'a>,
{
    pub fn new(value: T::Shared<U>) -> Shared<'a, T, U> {
        Shared(value)
    }
}

impl<'a, T, U> AsRef<U> for Shared<'a, T, U>
where
    T: ContextValue<'a>,
{
    fn as_ref(&self) -> &U {
        &*self.0
    }
}

impl<'a, 'arbitrary, T, U> Arbitrary<'arbitrary> for Shared<'a, T, U>
where
    T: ContextValue<'a>,
    U: Arbitrary<'arbitrary>,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'arbitrary>) -> arbitrary::Result<Self> {
        todo!()
        // Ok(Shared(T::Shared::from(Arbitrary::arbitrary(u)?)))
    }
}

impl<'a, T, U> Clone for Shared<'a, T, U>
where
    T: ContextValue<'a>,
{
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<'a, T, U> Debug for Shared<'a, T, U>
where
    T: ContextValue<'a>,
    U: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a, T, U> Deref for Shared<'a, T, U>
where
    T: ContextValue<'a>,
{
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a, T, U> From<U> for Shared<'a, T, U>
where
    T: ContextValue<'a>,
    T::Shared<U>: From<U>,
{
    fn from(value: U) -> Self {
        Shared(value.into())
    }
}

impl<'a> ContextValue<'a> for &'a str {
    type Shared<T> = BumpBox<'a, T>
    where
        T: 'a;

    type List<T> = bumpalo::collections::Vec<'a, T>
    where
        T: 'a;
}

pub struct BumpBox<'a, T>(pub *const T, pub PhantomData<&'a ()>);

impl<'a, T> Clone for BumpBox<'a, T> {
    fn clone(&self) -> Self {
        BumpBox(self.0, self.1)
    }
}

impl<'a, T> AsPtr for BumpBox<'a, T> {
    fn as_ptr(&self) -> usize {
        self.0 as usize
    }
}

impl<'a, T> Deref for BumpBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

pub trait AsPtr {
    fn as_ptr(&self) -> usize;
}

impl<T> AsPtr for &T {
    fn as_ptr(&self) -> usize {
        *self as *const T as usize
    }
}

impl<T> AsPtr for Rc<T> {
    fn as_ptr(&self) -> usize {
        Rc::as_ptr(self) as usize
    }
}

impl<T> AsPtr for Arc<T> {
    fn as_ptr(&self) -> usize {
        Arc::as_ptr(self) as usize
    }
}

impl<'a, T, U> AsPtr for Shared<'a, T, U>
where
    T: ContextValue<'a>,
{
    fn as_ptr(&self) -> usize {
        self.0.as_ptr()
    }
}
