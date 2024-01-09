use derive_more::From;
#[cfg(feature = "enum-iter")]
use enum_iterator::Sequence;
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::fmt::{Display, Formatter};

use crate::Span;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Ignore<'t> {
    Comment(Span<'t>),
    MultilineComment(Span<'t>),
    Newline,
    Whitespace,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
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
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
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

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Identifier<'t> {
    Identifier(Span<'t>),
    Type(Span<'t>),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Literal<'t> {
    Binary(Span<'t>),
    Octal(Span<'t>),
    Hex(Span<'t>),
    Floating(Span<'t>),
    Char(Span<'t>),
    String(Span<'t>),
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum MathOperator {
    Plus,
    Sub,
    Div,
    Mod,
    Pow,
    Times,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
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

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum LogicOperator {
    OrLogic,
    AndLogic,
    NotLogic,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum BitOperator {
    OrBit,
    AndBit,
    XorBit,
    NotBit,
}

#[derive(Debug, PartialEq, Clone, From)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Operator {
    Dot,
    Flow,
    Math(MathOperator),
    Comparison(ComparisonOperator),
    Logic(LogicOperator),
    Bit(BitOperator),
}

#[derive(Debug, PartialEq, Clone, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
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
