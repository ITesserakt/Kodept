#![allow(clippy::needless_lifetimes)]

use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::{replace, take};

use crate::graph::{GenericASTNode, GhostToken};
use derive_more::IsVariant;

use crate::graph::nodes::Owned;

pub struct RefMut<'a, T> {
    node: &'a Owned,
    _phantom: PhantomData<T>,
}

#[derive(Default, IsVariant)]
pub enum OptVec<T> {
    #[default]
    Empty,
    Single(T),
    Vector(Vec<T>),
}

#[derive(Default)]
enum OptVecIter<'a, T> {
    #[default]
    Empty,
    Single(&'a T),
    Vector(std::slice::Iter<'a, T>),
}

impl<'a, T> RefMut<'a, T> {
    pub fn new(node: &'a Owned) -> Self {
        Self {
            node,
            _phantom: Default::default(),
        }
    }
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

    pub fn push(&mut self, item: T) {
        match self {
            OptVec::Empty => *self = OptVec::Single(item),
            OptVec::Single(_) => {
                let OptVec::Single(x) = replace(self, OptVec::Empty) else {
                    return;
                };
                *self = OptVec::Vector(vec![x, item])
            }
            OptVec::Vector(vec) => vec.push(item),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        match self {
            OptVec::Empty => OptVecIter::Empty,
            OptVec::Single(t) => OptVecIter::Single(t),
            OptVec::Vector(x) => OptVecIter::Vector(x.iter()),
        }
    }
}

pub trait FromOptVec {
    type Ref<'a>
    where
        Self::T: 'a;
    type Mut<'a>;
    type T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a>;
    fn unwrap_mut<'a>(value: OptVec<&'a Owned>) -> Self::Mut<'a>;
}

impl<T> FromOptVec for Option<T> {
    type Ref<'a> = Option<&'a T> where T: 'a;
    type Mut<'a> = Option<RefMut<'a, T>>;
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

    fn unwrap_mut<'a>(value: OptVec<&'a Owned>) -> Self::Mut<'a> {
        match value {
            OptVec::Empty => None,
            OptVec::Single(x) => Some(RefMut::new(x)),
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
    type Mut<'a> = Vec<RefMut<'a, T>>;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        value.into_vec()
    }

    fn unwrap_mut<'a>(value: OptVec<&'a Owned>) -> Self::Mut<'a> {
        value.iter().map(|x| RefMut::new(x)).collect()
    }
}

impl<'a, T> Iterator for OptVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OptVecIter::Empty => None,
            OptVecIter::Single(_) => {
                let OptVecIter::Single(t) = take(self) else {
                    unreachable!()
                };
                Some(t)
            }
            OptVecIter::Vector(iter) => iter.next(),
        }
    }
}

impl<'a, T> RefMut<'a, T>
where
    for<'b> &'b mut T: TryFrom<&'b mut GenericASTNode>,
    for<'b> <&'b mut GenericASTNode as TryInto<&'b mut T>>::Error: Debug,
{
    pub fn borrow_mut(&self, token: &'a mut GhostToken) -> &mut T {
        self.node.rw(token).try_into().expect("Node has wrong type")
    }
}

impl<'a, T> RefMut<'a, T>
where
    for<'b> &'b T: TryFrom<&'b GenericASTNode>,
    for<'b> <&'b GenericASTNode as TryInto<&'b T>>::Error: Debug,
{
    pub fn borrow(&self, token: &'a GhostToken) -> &T {
        self.node.ro(token).try_into().expect("Node has wrong type")
    }
}
