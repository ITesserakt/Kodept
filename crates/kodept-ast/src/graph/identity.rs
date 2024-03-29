#![allow(clippy::needless_lifetimes)]

use crate::graph::nodes::Owned;
use std::any::type_name;

use crate::graph::utils::{FromOptVec, OptVec, RefMut};

#[repr(transparent)]
pub struct Identity<T>(pub T);

impl<T> FromOptVec for Identity<T> {
    type Ref<'a> = &'a T where Self::T: 'a;
    type Mut<'a> = RefMut<'a, T>;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        match value {
            OptVec::Single(x) => x,
            _ => panic!(
                "Container must has only one child <{}>, but has {}",
                type_name::<T>(),
                value.len()
            ),
        }
    }

    fn unwrap_mut<'a>(value: OptVec<&'a Owned>) -> Self::Mut<'a> {
        match value {
            OptVec::Single(x) => RefMut::new(x),
            _ => panic!(
                "Container must has only one child <{}>, but has {}",
                type_name::<T>(),
                value.len()
            ),
        }
    }
}
