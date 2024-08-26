use slotmap::Key;
use slotmap::{
    basic::{Iter as BasicIter, SlotMap},
    secondary::{Iter as SecondaryIter, SecondaryMap},
    sparse_secondary::{Iter as SparseIter, SparseSecondaryMap},
};
use std::ops::{Index, IndexMut};

pub trait SlotMapContainer: Index<Self::Key, Output = Self::Data> + IndexMut<Self::Key, Output = Self::Data> {
    type Key: Key;
    type Data;
    type Iter<'a>: Iterator<Item = (Self::Key, &'a Self::Data)>
    where
        Self: 'a;

    fn get(&self, index: Self::Key) -> Option<&Self::Data>;
    fn get_mut(&mut self, index: Self::Key) -> Option<&mut Self::Data>;
    fn iter(&self) -> Self::Iter<'_>;
    fn len(&self) -> usize;
    fn remove(&mut self, key: Self::Key) -> Option<Self::Data>;
    fn contains_key(&self, key: Self::Key) -> bool;
}

macro_rules! impl_container {
    ($t:ident {iter: $iter:ident}) => {
        impl<K: Key, V> SlotMapContainer for $t<K, V> {
            type Key = K;
            type Data = V;
            type Iter<'a> = $iter<'a, K, V> where K: 'a, V: 'a;

            fn get(&self, index: Self::Key) -> Option<&Self::Data> {
                $t::get(self, index)
            }

            fn get_mut(&mut self, index: Self::Key) -> Option<&mut Self::Data> {
                $t::get_mut(self, index)
            }

            fn len(&self) -> usize {
                $t::len(self)
            }

            fn iter(&self) -> Self::Iter<'_> {
                $t::iter(self)
            }

            fn remove(&mut self, key: Self::Key) -> Option<Self::Data> {
                $t::remove(self, key)
            }

            fn contains_key(&self, key: Self::Key) -> bool {
                $t::contains_key(self, key)
            }
        }
    };
}

impl_container!(SlotMap {
    iter: BasicIter
});
impl_container!(SecondaryMap {
    iter: SecondaryIter
});
impl_container!(SparseSecondaryMap {
    iter: SparseIter
});
