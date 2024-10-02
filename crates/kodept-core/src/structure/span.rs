use derive_more::Constructor;

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
}
