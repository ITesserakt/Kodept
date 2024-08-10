use std::ops::Range;

use derive_more::{Constructor, Display};

use crate::structure::Located;

#[derive(Constructor, Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
#[display("...{}:{}", offset, length)]
pub struct CodePoint {
    pub length: u32,
    pub offset: u32,
}

impl CodePoint {
    #[must_use]
    pub const fn single_point(offset: u32) -> Self {
        Self {
            length: 1,
            offset,
        }
    }

    pub const fn as_range(&self) -> Range<usize> {
        let offset = self.offset as usize;
        let length = self.length as usize;
        offset..offset + length
    }
}

impl Located for CodePoint {
    fn location(&self) -> CodePoint {
        *self
    }
}
