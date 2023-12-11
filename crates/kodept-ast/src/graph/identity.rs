#![allow(clippy::needless_lifetimes)]

use crate::graph::utils::{FromOptVec, OptVec};
use std::any::type_name;

#[repr(transparent)]
pub struct Identity<T>(pub T);

impl<T> FromOptVec for Identity<T> {
    type Ref<'a> = &'a T where Self::T: 'a;
    type Mut<'a> = &'a mut T where Self::T: 'a;
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

    fn unwrap_mut<'a>(value: OptVec<&'a mut Self::T>) -> Self::Mut<'a> {
        match value {
            OptVec::Single(x) => x,
            _ => panic!(
                "Container must has only one child <{}>, but has {}",
                type_name::<T>(),
                value.len()
            ),
        }
    }
}
