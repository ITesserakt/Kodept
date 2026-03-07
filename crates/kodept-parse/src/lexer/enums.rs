use std::fmt::{Display, Formatter};

use crate::lexer::traits::ToRepresentation;
#[cfg(feature = "enum-iter")]
use enum_iterator::Sequence;

#[derive(Debug, PartialEq, Copy, Clone, Default)]
#[cfg_attr(feature = "enum-iter", derive(Sequence))]
pub enum Token {
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

const _: () = assert!(size_of::<Token>() - 1 == 0);

#[allow(clippy::match_like_matches_macro)]
impl Token {
    pub fn is_ignored(&self) -> bool {
        match self {
            Token::Comment => true,
            Token::MultilineComment => true,
            Token::Newline => true,
            Token::Whitespace => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            Token::Comma => true,
            Token::Semicolon => true,
            Token::LBrace => true,
            Token::RBrace => true,
            Token::LBracket => true,
            Token::RBracket => true,
            Token::LParen => true,
            Token::RParen => true,
            Token::TypeGap => true,
            Token::DoubleColon => true,
            Token::Colon => true,
            _ => false,
        }
    }

    pub fn is_keyword(&self) -> bool {
        match self {
            Token::Fun => true,
            Token::Val => true,
            Token::Var => true,
            Token::If => true,
            Token::Elif => true,
            Token::Else => true,
            Token::Match => true,
            Token::While => true,
            Token::Module => true,
            Token::Extend => true,
            Token::Abstract => true,
            Token::Trait => true,
            Token::Struct => true,
            Token::Class => true,
            Token::Enum => true,
            Token::Foreign => true,
            Token::TypeAlias => true,
            Token::With => true,
            Token::Return => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            Token::Dot => true,
            Token::Flow => true,
            Token::Plus => true,
            Token::Sub => true,
            Token::Div => true,
            Token::Mod => true,
            Token::Pow => true,
            Token::Times => true,
            Token::Equals => true,
            Token::Equiv => true,
            Token::NotEquiv => true,
            Token::Less => true,
            Token::LessEquals => true,
            Token::Greater => true,
            Token::GreaterEquals => true,
            Token::Spaceship => true,
            Token::OrLogic => true,
            Token::AndLogic => true,
            Token::NotLogic => true,
            Token::OrBit => true,
            Token::AndBit => true,
            Token::XorBit => true,
            Token::NotBit => true,
            _ => false,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.representation())
    }
}
