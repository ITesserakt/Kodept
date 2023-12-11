#![allow(clippy::needless_lifetimes)]

use std::any::type_name;

pub enum OptVec<T> {
    Empty,
    Single(T),
    Vector(Vec<T>),
}

impl<A> FromIterator<A> for OptVec<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut this = OptVec::Empty;
        let mut iter = iter.into_iter();
        match iter.next() {
            None => return this,
            Some(x) => this = OptVec::Single(x),
        };
        match (iter.next(), this) {
            (None, x) => x,
            (Some(x), OptVec::Single(y)) => {
                let mut vec = vec![y, x];
                vec.extend(iter);
                OptVec::Vector(vec)
            }
            _ => unreachable!(),
        }
    }
}

impl<T> OptVec<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OptVec::Empty => vec![],
            OptVec::Single(x) => vec![x],
            OptVec::Vector(x) => x,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            OptVec::Empty => 0,
            OptVec::Single(_) => 1,
            OptVec::Vector(x) => x.len(),
        }
    }
}

pub(crate) trait FromOptVec {
    type Ref<'a>
    where
        Self::T: 'a;
    type Mut<'a>
    where
        Self::T: 'a;
    type T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a>;
    fn unwrap_mut<'a>(value: OptVec<&'a mut Self::T>) -> Self::Mut<'a>;
}

impl<T> FromOptVec for Option<T> {
    type Ref<'a>= Option<&'a T> where T: 'a;
    type Mut<'a> = Option<&'a mut T> where T: 'a;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        match value {
            OptVec::Empty => None,
            OptVec::Single(x) => Some(x),
            OptVec::Vector(x) => panic!(
                "Container must has no more then one child <{}>, but has {}",
                type_name::<T>(),
                x.len()
            ),
        }
    }

    fn unwrap_mut<'a>(value: OptVec<&'a mut Self::T>) -> Self::Mut<'a> {
        match value {
            OptVec::Empty => None,
            OptVec::Single(x) => Some(x),
            OptVec::Vector(x) => panic!(
                "Container must has no more then one child <{}>, but has {}",
                type_name::<T>(),
                x.len()
            ),
        }
    }
}

impl<T> FromOptVec for Vec<T> {
    type Ref<'a> = Vec<&'a T> where Self::T: 'a;
    type Mut<'a> = Vec<&'a mut T> where Self::T: 'a;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        value.into_vec()
    }

    fn unwrap_mut<'a>(value: OptVec<&'a mut Self::T>) -> Self::Mut<'a> {
        value.into_vec()
    }
}
