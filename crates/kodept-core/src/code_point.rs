use crate::structure::Located;
use derive_more::{Constructor, Display};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::ops::Range;

#[derive(Constructor, Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
#[display(fmt = "...{}:{}", offset, length)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct CodePoint {
    pub length: usize,
    pub offset: usize,
}

impl CodePoint {
    #[must_use]
    pub const fn single_point(offset: usize) -> Self {
        Self { length: 1, offset }
    }

    pub const fn as_range(&self) -> Range<usize> {
        self.offset..self.offset + self.length
    }
}

impl Located for CodePoint {
    fn location(&self) -> CodePoint {
        *self
    }
}
