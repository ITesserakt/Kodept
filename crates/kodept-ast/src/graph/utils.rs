#![allow(clippy::needless_lifetimes)]

use std::any::type_name;
use std::fmt::Debug;

pub(crate) type OptVec<T> = Vec<T>;

pub trait FromOptVec {
    type Ref<'a>
    where
        Self::T: 'a;
    type Mut<'a>
    where Self::T: 'a;
    type T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a>;
    fn unwrap_mut<'a>(value: OptVec<&'a mut Self::T>) -> Self::Mut<'a>;
}

impl<T: Debug> FromOptVec for Option<T> {
    type Ref<'a> = Option<&'a T> where T: 'a;
    type Mut<'a> = Option<&'a mut T> where T: 'a;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        match value.split_first() {
            None => None,
            Some((x, [])) => Some(x),
            Some((_, x)) => panic!(
                "Container must has no more then one child <{}>, but has {:?}",
                type_name::<T>(),
                x
            ),
        }
    }

    fn unwrap_mut<'a>(mut value: OptVec<&'a mut Self::T>) -> Self::Mut<'a> {
        if value.len() <= 1 {
            value.pop()
        } else {
            panic!(
                "Container must has no more then one child <{}>, but has {:?}",
                type_name::<T>(),
                value
            )
        }
    }
}

impl<T> FromOptVec for Vec<T> {
    type Ref<'a> = Vec<&'a T> where Self::T: 'a;
    type Mut<'a> = Vec<&'a mut T> where Self::T: 'a; 
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        value.to_vec()
    }

    fn unwrap_mut<'a>(value: OptVec<&'a mut Self::T>) -> Self::Mut<'a> {
        value
    }
}
