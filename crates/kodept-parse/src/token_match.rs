use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;

use crate::lexer::Token;

#[derive(Debug, Clone, Copy, PartialEq, Constructor)]
pub struct TokenMatch {
    pub token: Token,
    pub point: CodePoint,
}

const _: () = assert!(size_of::<TokenMatch>() - 12 == 0);

impl Located for TokenMatch {
    fn location(&self) -> CodePoint {
        self.point
    }
}
