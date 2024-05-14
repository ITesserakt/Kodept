#![allow(clippy::needless_lifetimes)]

use std::any::type_name;
use std::fmt::Debug;
use std::iter::{Once, once};
use std::ops::{Deref, DerefMut};

use crate::graph::nodes::Inaccessible;
use crate::graph::utils::{FromOptVec, OptVec, RefMut};

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

impl<T: Debug> FromOptVec for Identity<T> {
    type Ref<'a> = &'a T where Self::T: 'a;
    type Mut<'a> = RefMut<'a, T>;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        match value.as_slice() {
            [x] => x,
            _ => panic!(
                "Container must has only one child <{}>, but has {:?}",
                type_name::<T>(),
                value
            ),
        }
    }

    fn unwrap_mut<'a>(value: OptVec<&'a Inaccessible>) -> Self::Mut<'a> {
        match value.as_slice() {
            [x] => RefMut::new(x),
            _ => panic!(
                "Container must has only one child <{}>, but has {:?}",
                type_name::<T>(),
                value
            ),
        }
    }
}
