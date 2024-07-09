use std::fmt::{Display, Formatter};

use derive_more::From;
#[cfg(feature = "enum-iter")]
use enum_iterator::Sequence;

use crate::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ignore<'t> {
    Comment(Span<'t>),
    MultilineComment(Span<'t>),
    Newline,
    Whitespace,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum Keyword {
    Fun,
    Val,
    Var,
    If,
    Elif,
    Else,
    Match,
    While,
    Module,
    Extend,
    Lambda,
    Abstract,
    Trait,
    Struct,
    Class,
    Enum,
    Foreign,
    TypeAlias,
    With,
    Return,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum Symbol {
    Comma,
    Semicolon,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    TypeGap,
    DoubleColon,
    Colon,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Identifier<'t> {
    Identifier(Span<'t>),
    Type(Span<'t>),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal<'t> {
    Binary(Span<'t>),
    Octal(Span<'t>),
    Hex(Span<'t>),
    Floating(Span<'t>),
    Char(Span<'t>),
    String(Span<'t>),
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum MathOperator {
    Plus,
    Sub,
    Div,
    Mod,
    Pow,
    Times,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum ComparisonOperator {
    Equals,
    Equiv,
    NotEquiv,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
    Spaceship,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum LogicOperator {
    OrLogic,
    AndLogic,
    NotLogic,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum BitOperator {
    OrBit,
    AndBit,
    XorBit,
    NotBit,
}

#[derive(Debug, PartialEq, Clone, From, Copy)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum Operator {
    Dot,
    Flow,
    Math(MathOperator),
    Comparison(ComparisonOperator),
    Logic(LogicOperator),
    Bit(BitOperator),
}

#[derive(Debug, PartialEq, Clone, From, Copy)]
pub enum Token<'t> {
    Ignore(Ignore<'t>),
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(Identifier<'t>),
    Literal(Literal<'t>),
    Operator(Operator),
    Unknown,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ignore(x) => match x {
                Ignore::Comment(x) => write!(f, "{x}"),
                Ignore::MultilineComment(x) => write!(f, "{x}"),
                Ignore::Newline => write!(f, "<newline>"),
                Ignore::Whitespace => write!(f, " "),
            },
            Token::Unknown => write!(f, "<unknown>"),
            _ => write!(f, "{self:?}"),
        }
    }
}
