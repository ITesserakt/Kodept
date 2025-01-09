use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "String"))]
#[cfg_attr(feature = "serde", serde(from = "Cow<str>"))]
pub struct SharedStr {
    implementation: Box<dyn Stringy>,
}

#[allow(unsafe_code)]
pub unsafe trait Stringy {
    fn as_ref(&self) -> &str;
    fn clone(&self) -> SharedStr;
}

impl SharedStr {
    pub fn new<T: Stringy + 'static>(value: T) -> Self {
        Self {
            implementation: Box::new(value),
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for SharedStr {}
#[allow(unsafe_code)]
unsafe impl Sync for SharedStr {}

impl Deref for SharedStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.implementation.as_ref().as_ref()
    }
}

impl From<SharedStr> for String {
    fn from(val: SharedStr) -> Self {
        val.deref().to_string()
    }
}

impl From<Cow<'static, str>> for SharedStr {
    fn from(value: Cow<'static, str>) -> Self {
        struct Helper(Cow<'static, str>);

        #[allow(unsafe_code)]
        unsafe impl Stringy for Helper {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }

            fn clone(&self) -> SharedStr {
                SharedStr::new(Self(self.0.clone()))
            }
        }

        Self::new(Helper(value))
    }
}

impl Clone for SharedStr {
    fn clone(&self) -> Self {
        self.implementation.clone()
    }
}

impl PartialEq for SharedStr {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl PartialOrd for SharedStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for SharedStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Debug for SharedStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl Ord for SharedStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref().cmp(other.deref())
    }
}

impl Eq for SharedStr {}
