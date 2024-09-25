#![allow(clippy::needless_lifetimes)]

use std::iter::{once, Once};
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
pub struct Identity<T>(pub T);

impl<T> IntoIterator for Identity<T> {
    type Item = T;
    type IntoIter = Once<T>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.0)
    }
}

impl<T> Deref for Identity<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Identity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

