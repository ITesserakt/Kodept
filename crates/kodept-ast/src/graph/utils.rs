#![allow(clippy::needless_lifetimes)]

use crate::graph::Identity;

pub(crate) type OptVec<T> = Vec<T>;

pub struct OptionFamily;
pub struct VecFamily;
pub struct IdentityFamily;

pub trait ContainerFamily {
    type This<T>;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::This<T>;
}

pub trait HasContainerFamily {
    type Family: ContainerFamily;
    type T;
}

pub(crate) type ContainerT<C, T> = <C as ContainerFamily>::This<T>;
pub(crate) type ChangeT<From, To> =
    <<From as HasContainerFamily>::Family as ContainerFamily>::This<To>;
pub(crate) type WrapRef<'a, C> = ChangeT<C, &'a InnerT<C>>;
pub(crate) type FamilyT<Of> = <Of as HasContainerFamily>::Family;
pub(crate) type InnerT<C> = <C as HasContainerFamily>::T;

impl<T> HasContainerFamily for Option<T> {
    type Family = OptionFamily;
    type T = T;
}

impl<T> HasContainerFamily for Vec<T> {
    type Family = VecFamily;
    type T = T;
}

impl<T> HasContainerFamily for Identity<T> {
    type Family = IdentityFamily;
    type T = T;
}

impl ContainerFamily for OptionFamily {
    type This<T> = Option<T>;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::This<T> {
        let mut into_iter = iter.into_iter();
        match (into_iter.next(), into_iter.next()) {
            (None, _) => None,
            (Some(x), None) => Some(x),
            (Some(_), Some(_)) => unreachable!("Container must have at most one element")
        }
    }
}

impl ContainerFamily for VecFamily {
    type This<T> = Vec<T>;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::This<T> {
        Vec::from_iter(iter)
    }
}

impl ContainerFamily for IdentityFamily {
    type This<T> = T;

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> Self::This<T> {
        let mut into_iter = iter.into_iter();

        match (into_iter.next(), into_iter.next()) {
            (Some(x), None) => x,
            (None, _) | (Some(_), Some(_)) => unreachable!("Container must have exactly one element"),
        }
    }
}
