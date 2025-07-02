use derive_more::Constructor;
use nom::multi::fold;
use nom::Err::{Error, Failure, Incomplete};
use nom::{IResult, Offset, Parser};

use kodept_core::code_point::CodePoint;

use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::PackedToken;
use crate::nom::{TError, TParser};
use crate::token_match::PackedTokenMatch;

pub(crate) const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub(crate) const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

mod grammar {
    use nom::branch::alt;
    use nom::bytes::complete::{is_a, is_not, take_while};
    use nom::bytes::{tag, tag_no_case};
    use nom::character::complete::{anychar, char, not_line_ending, one_of};
    use nom::combinator::{cut, not, opt, recognize, value};
    use nom::error::context;
    use nom::multi::{many1, many_till};
    use nom::number::recognize_float;
    use nom::sequence::{delimited, preceded};
    use nom::Parser;

    use crate::lexer::PackedToken;
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
            value($token, tag($tag))
        };
    }

    macro_rules! include_literals {
        {$($($specifier:ident)* $tag:literal => $token:expr,)+} => {
            ($(include_literal!($($specifier)* $tag => $token),)+)
        };
    }

    fn ignore<'t>() -> impl TParser<'t, PackedToken> {
        context(
            "ignore",
            alt((
                value(PackedToken::Whitespace, many1(is_a(" \t"))),
                value(PackedToken::Newline, one_of("\r\n")),
            )),
        )
    }

    fn ignore_unpopular<'t>() -> impl TParser<'t, PackedToken> {
        let comment = value(
            PackedToken::Comment,
            recognize(preceded(tag("//"), cut(not_line_ending))),
        );
        let multiline_comment = value(
            PackedToken::MultilineComment,
            recognize(preceded(tag("/*"), cut(many_till(anychar, tag("*/"))))),
        );
        context("ignore", alt((comment, multiline_comment)))
    }

    fn keyword<'t>() -> impl TParser<'t, PackedToken> {
        context(
            "keyword",
            alt(include_literals! {
                "fun" => PackedToken::Fun,
                "val" => PackedToken::Val,
                "var" => PackedToken::Var,
                soft "if" => PackedToken::If,
                soft "elif" => PackedToken::Elif,
                soft "else" => PackedToken::Else,
                "match" => PackedToken::Match,
                "while" => PackedToken::While,
                "module" => PackedToken::Module,
                "extend" => PackedToken::Extend,
                "return" => PackedToken::Return,
                "\\" => PackedToken::Lambda,
                soft "abstract" => PackedToken::Abstract,
                soft "trait" => PackedToken::Trait,
                soft "struct" => PackedToken::Struct,
                soft "class" => PackedToken::Class,
                soft "enum" => PackedToken::Enum,
                soft "foreign" => PackedToken::Foreign,
                soft "type" => PackedToken::TypeAlias,
                soft "with" => PackedToken::With,
            }),
        )
    }

    fn symbol<'t>() -> impl TParser<'t, PackedToken> {
        context(
            "symbol",
            alt(include_literals! {
                "," => PackedToken::Comma,
                ";" => PackedToken::Semicolon,
                "{" => PackedToken::LBrace,
                "}" => PackedToken::RBrace,
                "[" => PackedToken::LBracket,
                "]" => PackedToken::RBracket,
                "(" => PackedToken::LParen,
                ")" => PackedToken::RParen,
                "_" => PackedToken::TypeGap,
                "::" => PackedToken::DoubleColon,
                ":" => PackedToken::Colon,
            }),
        )
    }

    fn identifier<'t>() -> impl TParser<'t, PackedToken> {
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
                value(PackedToken::Identifier, identifier_parser(LOWER_ALPHABET)),
                value(PackedToken::Type, identifier_parser(UPPER_ALPHABET)),
            )),
        )
    }

    fn literal<'t>() -> impl TParser<'t, PackedToken> {
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
                value(PackedToken::Binary, binary),
                value(PackedToken::Octal, octal),
                value(PackedToken::Hex, hex),
                value(PackedToken::Floating, floating),
                value(PackedToken::Char, char_p),
                value(PackedToken::String, string),
            )),
        )
    }

    fn operator<'t>() -> impl TParser<'t, PackedToken> {
        let branches = alt((
            alt(include_literals!(
                "." => PackedToken::Dot,
                "=>" => PackedToken::Flow,
                "+" => PackedToken::Plus,
                "-" => PackedToken::Sub,
                "**" => PackedToken::Pow,
                "*" => PackedToken::Times,
                "/" => PackedToken::Div,
                "%" => PackedToken::Mod,
                "<=>" => PackedToken::Spaceship,
                "==" => PackedToken::Equiv,
                "=" => PackedToken::Equals,
            )),
            alt(include_literals!(
                "!=" => PackedToken::NotEquiv,
                ">=" => PackedToken::GreaterEquals,
                ">" => PackedToken::Greater,
                "<=" => PackedToken::LessEquals,
                "<" => PackedToken::Less,
                "||" => PackedToken::OrLogic,
                "&&" => PackedToken::AndLogic,
                "!" => PackedToken::NotLogic,
                "|" => PackedToken::OrBit,
                "&" => PackedToken::AndBit,
                "^" => PackedToken::XorBit,
                "~" => PackedToken::NotBit,
            )),
        ));
        context("operator", branches)
    }

    pub(crate) fn token<'t>() -> impl TParser<'t, PackedToken> {
        let branches = alt((
            ignore(),
            identifier(),
            symbol(),
            literal(),
            keyword(),
            operator(),
            ignore_unpopular(),
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
            Err(Incomplete(_)) => ("", PackedToken::Unknown),
        };
        let matched_length = input.len() - rest.len();
        Ok(PackedTokenMatch::new(
            token.into(),
            CodePoint::new(matched_length as u32, 0),
        ))
    }
}

impl EagerTokensProducer for Lexer {
    type Error<'t> = TError<'t>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>> {
        fn token_parser(input: &str) -> IResult<&'_ str, (PackedToken, u32), TError<'_>> {
            let (rest, token) = grammar::token().parse(input)?;
            let length = input.offset(rest) as u32;
            Ok((rest, (token, length)))
        }

        fn parser<'t>() -> impl TParser<'t, Vec<PackedTokenMatch>> {
            let mut offset = 0;
            fold(
                0..,
                token_parser,
                || vec![],
                move |mut acc, (token, length)| {
                    offset += length;
                    acc.push(PackedTokenMatch::new(token, CodePoint::new(length, offset)));
                    acc
                },
            )
        }

        match parser().parse_complete(input) {
            Ok((_, x)) => Ok(x),
            Err(Error(e) | Failure(e)) => Err(e),
            Err(Incomplete(_)) => Ok(vec![PackedTokenMatch::new(
                PackedToken::Unknown,
                CodePoint::new(input.len() as u32, 0),
            )]),
        }
    }
}
