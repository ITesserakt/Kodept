use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, KeyData};

use crate::NodeKey;

new_key_type! { pub(crate) struct CommonKey; }

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct Key<T> {
    key_data: KeyData,
    _phantom: PhantomData<T>,
}

impl CommonKey {
    pub fn to_index(&self) -> u64 {
        self.0.as_ffi()
    }

    pub fn from_index(index: u64) -> Self {
        KeyData::from_ffi(index).into()
    }
}

impl<T> From<CommonKey> for Key<T> {
    fn from(value: CommonKey) -> Self {
        Self {
            key_data: value.0,
            _phantom: Default::default(),
        }
    }
}

impl<T> Key<T> {
    pub fn cast<U: TryFrom<T>>(self) -> Key<U> {
        Key {
            _phantom: Default::default(),
            key_data: self.key_data,
        }
    }

    pub fn null() -> Self {
        <Self as slotmap::Key>::null()
    }
}

impl<T> From<KeyData> for Key<T> {
    fn from(value: KeyData) -> Self {
        Self {
            key_data: value,
            _phantom: Default::default(),
        }
    }
}

impl<T> Copy for Key<T> {}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for Key<T> {
    fn default() -> Self {
        Self {
            key_data: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T> Eq for Key<T> {}

impl<T> PartialEq<Self> for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key_data.eq(&other.key_data)
    }
}

impl<T> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key_data.cmp(&other.key_data)
    }
}

impl<T> PartialOrd<Self> for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key_data.hash(state)
    }
}

impl<T> Debug for Key<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key")
            .field("key_data", &self.key_data)
            .field("type", &self._phantom)
            .finish()
    }
}

impl<T> Display for Key<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:?}", self.key_data)
    }
}

unsafe impl<T> slotmap::Key for Key<T> {
    fn data(&self) -> KeyData {
        self.key_data
    }
}

impl<T> From<Key<T>> for NodeKey {
    fn from(value: Key<T>) -> Self {
        NodeKey(CommonKey(value.key_data))
    }
}
