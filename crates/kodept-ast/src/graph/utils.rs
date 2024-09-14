#![allow(clippy::needless_lifetimes)]

use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use smallvec::SmallVec;

use crate::graph::any_node::AnyNode;
use crate::graph::nodes::{NodeCell, PermTkn};
use crate::graph::Identifiable;
use kodept_core::{ConvertibleToMut, ConvertibleToRef};

#[repr(transparent)]
pub struct TypedNodeCell<'a, T> {
    node: &'a NodeCell,
    _phantom: PhantomData<T>,
}

pub type OptVec<T> = SmallVec<[T; 1]>;

impl<'a, T> TypedNodeCell<'a, T> {
    pub fn new(node: &'a NodeCell) -> Self {
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
    fn unwrap_mut<'a>(value: OptVec<&'a NodeCell>) -> Self::Mut<'a>;
}

impl<T: Debug> FromOptVec for Option<T> {
    type Ref<'a> = Option<&'a T> where T: 'a;
    type Mut<'a> = Option<TypedNodeCell<'a, T>>;
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

    fn unwrap_mut<'a>(value: OptVec<&'a NodeCell>) -> Self::Mut<'a> {
        match value.split_first() {
            None => None,
            Some((x, [])) => Some(TypedNodeCell::new(x)),
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
    type Mut<'a> = Vec<TypedNodeCell<'a, T>>;
    type T = T;

    fn unwrap<'a>(value: OptVec<&'a Self::T>) -> Self::Ref<'a> {
        value.to_vec()
    }

    fn unwrap_mut<'a>(value: OptVec<&'a NodeCell>) -> Self::Mut<'a> {
        value.into_iter().map(|x| TypedNodeCell::new(x)).collect()
    }
}

impl<'a, T> TypedNodeCell<'a, T>
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

impl<'a, T> TypedNodeCell<'a, T>
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

impl<'a, T> Debug for TypedNodeCell<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedNodeCell")
            .field("node", self.node)
            .field("_phantom", &self._phantom)
            .finish()
    }
}
