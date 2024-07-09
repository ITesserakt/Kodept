use std::borrow::Cow;

use derive_more::Constructor;

use crate::code_point::CodePoint;
use crate::structure::Located;

#[derive(Constructor, Debug, Clone, PartialEq, Copy)]
pub struct Span {
    pub point: CodePoint,
}

impl Located for Span {
    fn location(&self) -> CodePoint {
        self.point
    }
}

pub trait CodeHolder {
    fn get_chunk(&self, at: CodePoint) -> Cow<str>;

    fn get_chunk_located<L: Located>(&self, for_item: &L) -> Cow<str> {
        self.get_chunk(for_item.location())
    }
}
