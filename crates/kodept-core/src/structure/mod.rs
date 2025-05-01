use crate::code_point::{CodePoint, Span};

pub use span::CodeHolder;

pub mod span {
    use crate::code_point::CodePoint;
    use crate::structure::Located;
    use std::marker::PhantomData;

    pub trait CodeHolder: Send + Sync + Sized + Copy {
        type Str;

        fn get_chunk(self, at: CodePoint) -> Self::Str;

        fn get_chunk_located<L: Located>(self, for_item: &L) -> Self::Str {
            self.get_chunk(for_item.location())
        }

        fn map<T, F>(self, func: F) -> MappingCodeHolder<Self, T, F>
        where
            F: FnOnce(Self::Str) -> T,
        {
            MappingCodeHolder {
                func,
                inner: self,
                _phantom: Default::default(),
            }
        }
    }

    #[derive(Debug)]
    pub struct MappingCodeHolder<C, T, F = fn(<C as CodeHolder>::Str) -> T>
    where
        C: CodeHolder,
    {
        func: F,
        inner: C,
        _phantom: PhantomData<T>,
    }

    impl<C, T, F> CodeHolder for MappingCodeHolder<C, T, F>
    where
        C: CodeHolder,
        F: Copy + Send + Sync + FnOnce(C::Str) -> T,
        T: Send + Sync,
    {
        type Str = T;

        #[inline(always)]
        fn get_chunk(self, at: CodePoint) -> Self::Str {
            (self.func)(self.inner.get_chunk(at))
        }
    }

    impl<C, T, F> Clone for MappingCodeHolder<C, T, F>
    where
        C: CodeHolder,
        F: Copy,
    {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<C, T, F> Copy for MappingCodeHolder<C, T, F>
    where
        C: CodeHolder,
        F: Copy,
    {
    }
}

pub trait Located {
    fn location(&self) -> CodePoint;
}

pub trait SpanBounds {
    fn bounds(&self) -> Span;
}

