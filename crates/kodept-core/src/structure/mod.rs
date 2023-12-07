use crate::code_point::CodePoint;

pub mod rlt;
pub mod span;

pub trait Located {
    fn location(&self) -> CodePoint;
}
