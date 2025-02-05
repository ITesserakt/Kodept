use derive_more::Constructor;
use nom::Err::{Error, Failure, Incomplete};
use nom::Parser;

use kodept_core::code_point::CodePoint;

use crate::common::TokenProducer;
use crate::lexer::Token;
use crate::nom::TError;
use crate::token_match::PackedTokenMatch;

pub(crate) const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub(crate) const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

mod grammar {
    use nom::branch::alt;
    use nom::bytes::complete::{is_a, is_not, take_while};
    use nom::bytes::{tag, tag_no_case};
    use nom::character::complete::{anychar, char, not_line_ending, one_of};
    use nom::combinator::{cut, map, not, opt, recognize, value, verify};
    use nom::error::context;
    use nom::multi::{many1, many_till};
    use nom::number::recognize_float;
    use nom::sequence::{delimited, preceded};
    use nom::Parser;

    use crate::lexer::{
        BitOperator, ComparisonOperator, Identifier, Ignore, Keyword, Literal, LogicOperator,
        MathOperator, Operator, Symbol, Token,
    };
    use crate::nom::lexer::{LOWER_ALPHABET, UPPER_ALPHABET};
    use crate::nom::TParser;

    macro_rules! include_literal {
        (exact $tag:literal => $token:expr) => {
            value($token, exact_literal_token(tag($tag)))
        };
        ($tag:literal => $token:expr) => {
            value($token, tag($tag))
        };
        (soft $tag:literal => $token:expr) => {
            value($token, soft_literal_token($tag))
        };
    }

    macro_rules! include_literals {
    {$($($specifier:ident)* $tag:literal => $token:expr,)+} => {
        ($(include_literal!($($specifier)* $tag => $token),)+)
    };
}

    fn soft_literal_token<'t, 's>(literal: &'s str) -> impl TParser<'t> + 's
    where
        't: 's
    {
        let name_extract = map(identifier(), |it| match it {
            Identifier::Identifier(x) => x,
            Identifier::Type(x) => x,
        });

        verify(name_extract, move |it: &str| it == literal)
    }

    fn ignore<'t>() -> impl TParser<'t, Ignore<'t>> {
        let comment = map(
            recognize(preceded(tag("//"), cut(not_line_ending))),
            Ignore::Comment,
        );
        let multiline_comment = map(
            recognize(preceded(tag("/*"), cut(many_till(anychar, tag("*/"))))),
            Ignore::MultilineComment,
        );

        context(
            "ignore",
            alt((
                comment,
                value(Ignore::Whitespace, many1(is_a(" \t"))),
                value(Ignore::Newline, one_of("\r\n")),
                multiline_comment,
            )),
        )
    }

    fn keyword<'t>() -> impl TParser<'t, Keyword> {
        context(
            "keyword",
            alt(include_literals! {
                "fun" => Keyword::Fun,
                "val" => Keyword::Val,
                "var" => Keyword::Var,
                soft "if" => Keyword::If,
                soft "elif" => Keyword::Elif,
                soft "else" => Keyword::Else,
                "match" => Keyword::Match,
                "while" => Keyword::While,
                "module" => Keyword::Module,
                "extend" => Keyword::Extend,
                "return" => Keyword::Return,
                "\\" => Keyword::Lambda,
                soft "abstract" => Keyword::Abstract,
                soft "trait" => Keyword::Trait,
                soft "struct" => Keyword::Struct,
                soft "class" => Keyword::Class,
                soft "enum" => Keyword::Enum,
                soft "foreign" => Keyword::Foreign,
                soft "type" => Keyword::TypeAlias,
                soft "with" => Keyword::With,
            }),
        )
    }

    fn symbol<'t>() -> impl TParser<'t, Symbol> {
        context(
            "symbol",
            alt(include_literals! {
                "," => Symbol::Comma,
                ";" => Symbol::Semicolon,
                "{" => Symbol::LBrace,
                "}" => Symbol::RBrace,
                "[" => Symbol::LBracket,
                "]" => Symbol::RBracket,
                "(" => Symbol::LParen,
                ")" => Symbol::RParen,
                "_" => Symbol::TypeGap,
                "::" => Symbol::DoubleColon,
                ":" => Symbol::Colon,
            }),
        )
    }

    fn identifier<'t>() -> impl TParser<'t, Identifier<'t>> {
        let identifier_parser = |alphabet| {
            recognize((
                opt(tag("_")),
                one_of(alphabet),
                take_while(|it: char| it == '_' || it.is_alphabetic()),
            ))
        };

        context(
            "identifier",
            alt((
                map(identifier_parser(LOWER_ALPHABET), Identifier::Identifier),
                map(identifier_parser(UPPER_ALPHABET), Identifier::Type),
            )),
        )
    }

    fn literal<'t>() -> impl TParser<'t, Literal<'t>> {
        fn number_parser<'a>(prefix: &'static str, alphabet: &'static str) -> impl TParser<'a> {
            recognize(preceded(
                tag_no_case(prefix),
                alt((
                    recognize(preceded(
                        not(one_of("_0")),
                        many1(one_of(alphabet).or(char('_'))),
                    )),
                    recognize(one_of(alphabet)),
                )),
            ))
        }

        let binary = number_parser("0b", "01");
        let octal = number_parser("0c", "01234567");
        let hex = number_parser("0x", "0123456789ABCDEFabcdef");
        let floating = recognize_float();
        let char_p = delimited(char('\''), recognize(anychar), char('\''));
        let string = delimited(char('"'), opt(is_not(r#"""#)), char('"'));

        context(
            "literal",
            alt((
                map(binary, Literal::Binary),
                map(octal, Literal::Octal),
                map(hex, Literal::Hex),
                map(floating, Literal::Floating),
                map(char_p, Literal::Char),
                map(string, |it| Literal::String(it.unwrap_or_default())),
            )),
        )
    }

    fn operator<'t>() -> impl TParser<'t, Operator> {
        context(
            "operator",
            alt((
                alt(include_literals! {
                    "." => Operator::Dot,
                    "=>" => Operator::Flow,
                }),
                map(
                    alt(include_literals! {
                    "+" => MathOperator::Plus,
                    "-" => MathOperator::Sub,
                    "**" => MathOperator::Pow,
                    "*" => MathOperator::Times,
                    "/" => MathOperator::Div,
                    "%" => MathOperator::Mod,
                    }),
                    Operator::Math,
                ),
                map(
                    alt(include_literals! {
                        "<=>" => ComparisonOperator::Spaceship,
                        "==" => ComparisonOperator::Equiv,
                        "=" => ComparisonOperator::Equals,
                        "!=" => ComparisonOperator::NotEquiv,
                        ">=" => ComparisonOperator::GreaterEquals,
                        ">" => ComparisonOperator::Greater,
                        "<=" => ComparisonOperator::LessEquals,
                        "<" => ComparisonOperator::Less,
                    }),
                    Operator::Comparison,
                ),
                map(
                    alt(include_literals! {
                        "||" => LogicOperator::OrLogic,
                        "&&" => LogicOperator::AndLogic,
                        "!" => LogicOperator::NotLogic,
                    }),
                    Operator::Logic,
                ),
                map(
                    alt(include_literals! {
                        "|" => BitOperator::OrBit,
                        "&" => BitOperator::AndBit,
                        "^" => BitOperator::XorBit,
                        "~" => BitOperator::NotBit,
                    }),
                    Operator::Bit,
                ),
            )),
        )
    }

    pub(crate) fn token<'t>() -> impl TParser<'t, Token<'t>> {
        let branches = alt((
            map(ignore(), Token::Ignore),
            map(keyword(), Token::Keyword),
            map(symbol(), Token::Symbol),
            map(identifier(), Token::Identifier),
            map(literal(), Token::Literal),
            map(operator(), Token::Operator),
        ));
        context("token", branches)
    }
}

#[derive(Constructor, Debug, Copy, Clone)]
pub struct Lexer;

impl TokenProducer for Lexer {
    type Error<'t> = TError<'t>;

    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<PackedTokenMatch, Self::Error<'t>> {
        let input = &whole_input[position..];
        let (rest, token) = match grammar::token().parse(input) {
            Ok(x) => x,
            Err(Error(e) | Failure(e)) => return Err(e),
            Err(Incomplete(_)) => ("", Token::Unknown),
        };
        let matched_length = input.len() - rest.len();
        Ok(PackedTokenMatch::new(
            token.into(),
            CodePoint::new(matched_length as u32, 0),
        ))
    }
}
