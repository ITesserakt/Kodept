use crate::code_point::CodePoint;

pub mod span;

pub trait Located {
    fn location(&self) -> CodePoint;
}
