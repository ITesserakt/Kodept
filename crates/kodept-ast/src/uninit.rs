use tracing::warn;
use crate::graph::{Identifiable, NodeId};
use crate::rlt_accessor::RLTFamily;

pub struct Uninit<'rlt, T> {
    value: T,
    rlt_ref: Option<RLTFamily<'rlt>>
}

impl<T> Uninit<'static, T> {
    pub fn new(value: T) -> Self {
        Self { value, rlt_ref: None }
    }
}

impl<'rlt, T> Uninit<'rlt, T> {
    pub fn unwrap(self, id: NodeId<T>) -> (T, Option<RLTFamily<'rlt>>)
    where
        T: Identifiable,
    {
        self.value.set_id(id);
        if self.rlt_ref.is_none() {
            warn!("No rlt linked with node {id}")
        }
        (self.value, self.rlt_ref)
    }
    
    #[allow(unsafe_code)]
    #[allow(private_bounds)]
    pub unsafe fn unwrap_unchecked(self, id: NodeId<T>) -> (T, Option<RLTFamily<'rlt>>)
    where 
        T: Identifiable
    {
        self.value.set_id(id);
        (self.value, self.rlt_ref)
    }

    pub fn with_rlt<R>(self, rlt_node: R) -> Uninit<'rlt, T>
    where 
        R: Into<RLTFamily<'rlt>>,
        R: 'rlt
    {
        Self {
            value: self.value,
            rlt_ref: Some(rlt_node.into())
        }
    }
    
    pub fn use_value<R>(&mut self, mut f: impl FnMut(&mut T) -> R) -> R {
        f(&mut self.value)
    }
    
    pub fn map<R>(self, f: impl FnOnce(T) -> R) -> Uninit<'rlt, R> {
        Uninit {
            value: f(self.value),
            rlt_ref: self.rlt_ref,
        }
    }

    #[inline]
    pub fn map_into<U>(self) -> Uninit<'rlt, U>
    where
        T: Into<U>,
    {
        Uninit {
            value: self.value.into(),
            rlt_ref: self.rlt_ref
        }
    }
}
