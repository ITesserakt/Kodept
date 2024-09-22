use std::fmt::{Display, Formatter};

use crate::lexer::traits::ToRepresentation;
use derive_more::{From, TryInto};
#[cfg(feature = "enum-iter")]
use enum_iterator::Sequence;
use kodept_core::static_assert_size;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ignore<'t> {
    Comment(&'t str),
    MultilineComment(&'t str),
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
    Identifier(&'t str),
    Type(&'t str),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal<'t> {
    Binary(&'t str),
    Octal(&'t str),
    Hex(&'t str),
    Floating(&'t str),
    Char(&'t str),
    String(&'t str),
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

#[derive(Debug, PartialEq, Clone, From, Copy, TryInto)]
pub enum Token<'t> {
    Ignore(Ignore<'t>),
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(Identifier<'t>),
    Literal(Literal<'t>),
    Operator(Operator),
    Unknown,
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum PackedToken {
    Comment,
    MultilineComment,
    Newline,
    Whitespace,
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
    Identifier,
    Type,
    Binary,
    Octal,
    Hex,
    Floating,
    Char,
    String,
    Dot,
    Flow,
    Plus,
    Sub,
    Div,
    Mod,
    Pow,
    Times,
    Equals,
    Equiv,
    NotEquiv,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
    Spaceship,
    OrLogic,
    AndLogic,
    NotLogic,
    OrBit,
    AndBit,
    XorBit,
    NotBit,
    #[default]
    Unknown,
}

static_assert_size!(Token<'static>, 32);
static_assert_size!(PackedToken, 1);

impl PackedToken {
    pub fn is_ignored(&self) -> bool {
        match self {
            PackedToken::Comment => true,
            PackedToken::MultilineComment => true,
            PackedToken::Newline => true,
            PackedToken::Whitespace => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            PackedToken::Comma => true,
            PackedToken::Semicolon => true,
            PackedToken::LBrace => true,
            PackedToken::RBrace => true,
            PackedToken::LBracket => true,
            PackedToken::RBracket => true,
            PackedToken::LParen => true,
            PackedToken::RParen => true,
            PackedToken::TypeGap => true,
            PackedToken::DoubleColon => true,
            PackedToken::Colon => true,
            _ => false,
        }
    }

    pub fn is_keyword(&self) -> bool {
        match self {
            PackedToken::Fun => true,
            PackedToken::Val => true,
            PackedToken::Var => true,
            PackedToken::If => true,
            PackedToken::Elif => true,
            PackedToken::Else => true,
            PackedToken::Match => true,
            PackedToken::While => true,
            PackedToken::Module => true,
            PackedToken::Extend => true,
            PackedToken::Lambda => true,
            PackedToken::Abstract => true,
            PackedToken::Trait => true,
            PackedToken::Struct => true,
            PackedToken::Class => true,
            PackedToken::Enum => true,
            PackedToken::Foreign => true,
            PackedToken::TypeAlias => true,
            PackedToken::With => true,
            PackedToken::Return => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            PackedToken::Dot => true,
            PackedToken::Flow => true,
            PackedToken::Plus => true,
            PackedToken::Sub => true,
            PackedToken::Div => true,
            PackedToken::Mod => true,
            PackedToken::Pow => true,
            PackedToken::Times => true,
            PackedToken::Equals => true,
            PackedToken::Equiv => true,
            PackedToken::NotEquiv => true,
            PackedToken::Less => true,
            PackedToken::LessEquals => true,
            PackedToken::Greater => true,
            PackedToken::GreaterEquals => true,
            PackedToken::Spaceship => true,
            PackedToken::OrLogic => true,
            PackedToken::AndLogic => true,
            PackedToken::NotLogic => true,
            PackedToken::OrBit => true,
            PackedToken::AndBit => true,
            PackedToken::XorBit => true,
            PackedToken::NotBit => true,
            _ => false,
        }
    }
}

impl Token<'_> {
    pub fn is_ignored(&self) -> bool {
        matches!(self, Token::Ignore(_))
    }
}

impl From<Token<'_>> for PackedToken {
    fn from(value: Token<'_>) -> Self {
        match value {
            Token::Ignore(x) => match x {
                Ignore::Comment(_) => PackedToken::Comment,
                Ignore::MultilineComment(_) => PackedToken::MultilineComment,
                Ignore::Newline => PackedToken::Newline,
                Ignore::Whitespace => PackedToken::Whitespace,
            },
            Token::Keyword(x) => match x {
                Keyword::Fun => PackedToken::Fun,
                Keyword::Val => PackedToken::Val,
                Keyword::Var => PackedToken::Var,
                Keyword::If => PackedToken::If,
                Keyword::Elif => PackedToken::Elif,
                Keyword::Else => PackedToken::Else,
                Keyword::Match => PackedToken::Match,
                Keyword::While => PackedToken::While,
                Keyword::Module => PackedToken::Module,
                Keyword::Extend => PackedToken::Extend,
                Keyword::Lambda => PackedToken::Lambda,
                Keyword::Abstract => PackedToken::Abstract,
                Keyword::Trait => PackedToken::Trait,
                Keyword::Struct => PackedToken::Struct,
                Keyword::Class => PackedToken::Class,
                Keyword::Enum => PackedToken::Enum,
                Keyword::Foreign => PackedToken::Foreign,
                Keyword::TypeAlias => PackedToken::TypeAlias,
                Keyword::With => PackedToken::With,
                Keyword::Return => PackedToken::Return,
            },
            Token::Symbol(x) => match x {
                Symbol::Comma => PackedToken::Comma,
                Symbol::Semicolon => PackedToken::Semicolon,
                Symbol::LBrace => PackedToken::LBrace,
                Symbol::RBrace => PackedToken::RBrace,
                Symbol::LBracket => PackedToken::LBracket,
                Symbol::RBracket => PackedToken::RBracket,
                Symbol::LParen => PackedToken::LParen,
                Symbol::RParen => PackedToken::RParen,
                Symbol::TypeGap => PackedToken::TypeGap,
                Symbol::DoubleColon => PackedToken::DoubleColon,
                Symbol::Colon => PackedToken::Colon,
            },
            Token::Identifier(Identifier::Identifier(_)) => PackedToken::Identifier,
            Token::Identifier(Identifier::Type(_)) => PackedToken::Type,
            Token::Literal(x) => match x {
                Literal::Binary(_) => PackedToken::Binary,
                Literal::Octal(_) => PackedToken::Octal,
                Literal::Hex(_) => PackedToken::Hex,
                Literal::Floating(_) => PackedToken::Floating,
                Literal::Char(_) => PackedToken::Char,
                Literal::String(_) => PackedToken::String,
            },
            Token::Operator(x) => match x {
                Operator::Dot => PackedToken::Dot,
                Operator::Flow => PackedToken::Flow,
                Operator::Math(x) => match x {
                    MathOperator::Plus => PackedToken::Plus,
                    MathOperator::Sub => PackedToken::Sub,
                    MathOperator::Div => PackedToken::Div,
                    MathOperator::Mod => PackedToken::Mod,
                    MathOperator::Pow => PackedToken::Pow,
                    MathOperator::Times => PackedToken::Times,
                },
                Operator::Comparison(x) => match x {
                    ComparisonOperator::Equals => PackedToken::Equals,
                    ComparisonOperator::Equiv => PackedToken::Equiv,
                    ComparisonOperator::NotEquiv => PackedToken::NotEquiv,
                    ComparisonOperator::Less => PackedToken::Less,
                    ComparisonOperator::LessEquals => PackedToken::LessEquals,
                    ComparisonOperator::Greater => PackedToken::Greater,
                    ComparisonOperator::GreaterEquals => PackedToken::GreaterEquals,
                    ComparisonOperator::Spaceship => PackedToken::Spaceship,
                },
                Operator::Logic(x) => match x {
                    LogicOperator::OrLogic => PackedToken::OrLogic,
                    LogicOperator::AndLogic => PackedToken::AndLogic,
                    LogicOperator::NotLogic => PackedToken::NotLogic,
                },
                Operator::Bit(x) => match x {
                    BitOperator::OrBit => PackedToken::OrBit,
                    BitOperator::AndBit => PackedToken::AndBit,
                    BitOperator::XorBit => PackedToken::XorBit,
                    BitOperator::NotBit => PackedToken::NotBit,
                },
            },
            Token::Unknown => PackedToken::Unknown,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

impl Display for PackedToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.representation())
    }
}
