use derive_more::Constructor;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use crate::code_point::CodePoint;
use crate::structure::Located;

#[repr(transparent)]
#[derive(Constructor, Debug, Clone, PartialEq, Copy)]
pub struct Span {
    pub point: CodePoint,
}

impl Located for Span {
    fn location(&self) -> CodePoint {
        self.point
    }
}

pub trait CodeHolder: Send + Sync + Sized + Copy {
    type Str;

    fn get_chunk(self, at: CodePoint) -> Self::Str;

    fn get_chunk_located<L: Located>(self, for_item: &L) -> Self::Str {
        self.get_chunk(for_item.location())
    }

    fn map<F, R>(self, f: F) -> impl CodeHolder<Str = R>
    where
        R: Send + Sync,
        F: Fn(Self::Str) -> R + Copy + Send + Sync
    {
        MappingCodeHolder::new(self, f)
    }
}

pub struct MappingCodeHolder<C, F, R>(C, F, PhantomData<R>);

impl<C, F, R> MappingCodeHolder<C, F, R> {
    pub const fn new(inner: C, mapping: F) -> Self
    where
        C: CodeHolder,
        F: Fn(C::Str) -> R,
    {
        Self(inner, mapping, PhantomData)
    }
}

impl<C, F, R> CodeHolder for MappingCodeHolder<C, F, R>
where
    C: Copy + CodeHolder,
    F: Copy + Fn(C::Str) -> R + Send + Sync,
    R: Send + Sync,
{
    type Str = R;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        let chunk = self.0.get_chunk(at);
        self.1(chunk)
    }
}

impl<C, F, R> Debug for MappingCodeHolder<C, F, R>
where
    C: Debug,
    F: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MappingCodeHolder")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

impl<C, F, R> Clone for MappingCodeHolder<C, F, R>
where
    C: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            0: self.0.clone(),
            1: self.1.clone(),
            2: Default::default(),
        }
    }
}

impl<C, F, R> Copy for MappingCodeHolder<C, F, R>
where
    C: Copy,
    F: Copy,
{
}
