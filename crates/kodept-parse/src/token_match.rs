use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::static_assert_size;
use kodept_core::structure::span::Span;

use crate::lexer::{PackedToken, Token};
use crate::lexer::Identifier::{Identifier, Type};
use crate::lexer::Ignore::{Comment, MultilineComment, Newline, Whitespace};
use crate::lexer::Keyword::*;
use crate::lexer::Symbol::*;
use crate::lexer::Literal::*;
use crate::lexer::Operator::*;
use crate::lexer::MathOperator::*;
use crate::lexer::ComparisonOperator::*;
use crate::lexer::BitOperator::*;
use crate::lexer::LogicOperator::*;

#[derive(Debug, Clone, PartialEq, Constructor, Copy)]
pub struct TokenMatch<'t> {
    pub token: Token<'t>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Constructor)]
pub struct PackedTokenMatch {
    pub token: PackedToken,
    pub point: CodePoint
}

static_assert_size!(TokenMatch<'static>, 40);
static_assert_size!(PackedTokenMatch, 12);

impl<'t> From<(PackedTokenMatch, &'t str)> for TokenMatch<'t> {
    #[inline(always)]
    fn from(value: (PackedTokenMatch, &'t str)) -> Self {
        let input = &value.1[value.0.point.as_range()];
        
        let token = match value.0.token {
            PackedToken::Comment => Token::Ignore(Comment(input)),
            PackedToken::MultilineComment => Token::Ignore(MultilineComment(input)),
            PackedToken::Newline => Token::Ignore(Newline),
            PackedToken::Whitespace => Token::Ignore(Whitespace),
            PackedToken::Fun => Token::Keyword(Fun),
            PackedToken::Val => Token::Keyword(Val),
            PackedToken::Var => Token::Keyword(Var),
            PackedToken::If => Token::Keyword(If),
            PackedToken::Elif => Token::Keyword(Elif),
            PackedToken::Else => Token::Keyword(Else),
            PackedToken::Match => Token::Keyword(Match),
            PackedToken::While => Token::Keyword(While),
            PackedToken::Module => Token::Keyword(Module),
            PackedToken::Extend => Token::Keyword(Extend),
            PackedToken::Lambda => Token::Keyword(Lambda),
            PackedToken::Abstract => Token::Keyword(Abstract),
            PackedToken::Trait => Token::Keyword(Trait),
            PackedToken::Struct => Token::Keyword(Struct),
            PackedToken::Class => Token::Keyword(Class),
            PackedToken::Enum => Token::Keyword(Enum),
            PackedToken::Foreign => Token::Keyword(Foreign),
            PackedToken::TypeAlias => Token::Keyword(TypeAlias),
            PackedToken::With => Token::Keyword(With),
            PackedToken::Return => Token::Keyword(Return),
            PackedToken::Comma => Token::Symbol(Comma),
            PackedToken::Semicolon => Token::Symbol(Semicolon),
            PackedToken::LBrace => Token::Symbol(LBrace),
            PackedToken::RBrace => Token::Symbol(RBrace),
            PackedToken::LBracket => Token::Symbol(LBracket),
            PackedToken::RBracket => Token::Symbol(RBracket),
            PackedToken::LParen => Token::Symbol(LParen),
            PackedToken::RParen => Token::Symbol(RParen),
            PackedToken::TypeGap => Token::Symbol(TypeGap),
            PackedToken::DoubleColon => Token::Symbol(DoubleColon),
            PackedToken::Colon => Token::Symbol(Colon),
            PackedToken::Identifier => Token::Identifier(Identifier(input)),
            PackedToken::Type => Token::Identifier(Type(input)),
            PackedToken::Binary => Token::Literal(Binary(input)),
            PackedToken::Octal => Token::Literal(Octal(input)),
            PackedToken::Hex => Token::Literal(Hex(input)),
            PackedToken::Floating => Token::Literal(Floating(input)),
            PackedToken::Char => Token::Literal(Char(input)),
            PackedToken::String => Token::Literal(String(input)),
            PackedToken::Dot => Token::Operator(Dot),
            PackedToken::Flow => Token::Operator(Flow),
            PackedToken::Plus => Token::Operator(Math(Plus)),
            PackedToken::Sub => Token::Operator(Math(Sub)),
            PackedToken::Div => Token::Operator(Math(Div)),
            PackedToken::Mod => Token::Operator(Math(Mod)),
            PackedToken::Pow => Token::Operator(Math(Pow)),
            PackedToken::Times => Token::Operator(Math(Times)),
            PackedToken::Equals => Token::Operator(Comparison(Equals)),
            PackedToken::Equiv => Token::Operator(Comparison(Equiv)),
            PackedToken::NotEquiv => Token::Operator(Comparison(NotEquiv)),
            PackedToken::Less => Token::Operator(Comparison(Less)),
            PackedToken::LessEquals => Token::Operator(Comparison(LessEquals)),
            PackedToken::Greater => Token::Operator(Comparison(Greater)),
            PackedToken::GreaterEquals => Token::Operator(Comparison(GreaterEquals)),
            PackedToken::Spaceship => Token::Operator(Comparison(Spaceship)),
            PackedToken::OrLogic => Token::Operator(Logic(OrLogic)),
            PackedToken::AndLogic => Token::Operator(Logic(AndLogic)),
            PackedToken::NotLogic => Token::Operator(Logic(NotLogic)),
            PackedToken::OrBit => Token::Operator(Bit(OrBit)),
            PackedToken::AndBit => Token::Operator(Bit(AndBit)),
            PackedToken::XorBit => Token::Operator(Bit(XorBit)),
            PackedToken::NotBit => Token::Operator(Bit(NotBit)),
            PackedToken::Unknown => Token::Unknown
        };
        TokenMatch::new(token, Span::new(value.0.point))
    }
}

impl From<TokenMatch<'_>> for PackedTokenMatch {
    fn from(value: TokenMatch<'_>) -> Self {
        PackedTokenMatch::new(value.token.into(), value.span.point)
    }
}
