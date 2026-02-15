use std::fmt::{Display, Formatter};

use crate::lexer::traits::ToRepresentation;
#[cfg(feature = "enum-iter")]
use enum_iterator::Sequence;
use kodept_core::static_assert_size;

pub type Token = PackedToken;

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

static_assert_size!(PackedToken, 1);

#[allow(clippy::match_like_matches_macro)]
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

impl Display for PackedToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.representation())
    }
}
