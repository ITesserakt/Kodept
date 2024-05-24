#![allow(clippy::needless_lifetimes)]

use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use smallvec::SmallVec;

use kodept_core::{ConvertibleToMut, ConvertibleToRef};

use crate::graph::nodes::Inaccessible;
use crate::graph::{AnyNode, Identifiable, PermTkn};

#[repr(transparent)]
pub struct RefMut<'a, T> {
    node: &'a Inaccessible,
    _phantom: PhantomData<T>,
}

pub type OptVec<T> = SmallVec<[T; 1]>;

impl<'a, T> RefMut<'a, T> {
    pub fn new(node: &'a Inaccessible) -> Self {
        Self {
            node,
            _phantom: Default::default(),
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
    fn unwrap_mut<'a>(value: OptVec<&'a Inaccessible>) -> Self::Mut<'a>;
}

impl<T: Debug> FromOptVec for Option<T> {
    type Ref<'a> = Option<&'a T> where T: 'a;
    type Mut<'a> = Option<RefMut<'a, T>>;
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

    fn unwrap_mut<'a>(value: OptVec<&'a Inaccessible>) -> Self::Mut<'a> {
        match value.split_first() {
            None => None,
            Some((x, [])) => Some(RefMut::new(x)),
            Some((_, x)) => panic!(
                "Container must has no more then one child <{}>, but has {:?}",
                type_name::<T>(),
                x
            ),
        }
    }
}

impl<T> FromOptVec for Vec<T> {
    type Ref<'a> = Vec<&'a T> where Self::T: 'a;
    type Mut<'a> = Vec<RefMut<'a, T>>;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        value.to_vec()
    }

    fn unwrap_mut<'a>(value: OptVec<&'a Inaccessible>) -> Self::Mut<'a> {
        value.into_iter().map(|x| RefMut::new(x)).collect()
    }
}

impl<'a, T> RefMut<'a, T>
where
    AnyNode: ConvertibleToMut<T>,
{
    pub fn borrow_mut<'b>(&'b self, token: &'a mut PermTkn) -> &'a mut T {
        let read = self.node.rw(token);
        let disc = read.describe();
        let id = read.get_id();
        match read.try_as_mut() {
            None => panic!(
                "Node [{}] has wrong type: expected `{}`, actual `{}`",
                id,
                type_name::<T>(),
                disc
            ),
            Some(x) => x,
        }
    }
}

impl<'a, T> RefMut<'a, T>
where
    AnyNode: ConvertibleToRef<T>,
{
    pub fn borrow(&self, token: &'a PermTkn) -> &T {
        let read = self.node.ro(token);
        match read.try_as_ref() {
            None => panic!(
                "Node [{}] has wrong type: expected `{}`, actual `{}`",
                read.get_id(),
                type_name::<T>(),
                read.describe()
            ),
            Some(x) => x,
        }
    }
}

impl<'a, T> Debug for RefMut<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefMut")
            .field("node", self.node)
            .field("_phantom", &self._phantom)
            .finish()
    }
}
