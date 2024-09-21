use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::static_assert_size;
use kodept_core::structure::Located;

use crate::lexer::PackedToken;

#[derive(Debug, Clone, Copy, PartialEq, Constructor)]
pub struct PackedTokenMatch {
    pub token: PackedToken,
    pub point: CodePoint
}

static_assert_size!(PackedTokenMatch, 12);

impl Located for PackedTokenMatch {
    fn location(&self) -> CodePoint {
        self.point
    }
}
