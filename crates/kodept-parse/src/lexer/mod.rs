use nom::{
    branch::alt,
    bytes::complete::take_while,
    bytes::complete::{is_a, is_not},
    character::complete::{anychar, char, digit0, digit1, not_line_ending, one_of},
    combinator::{map, opt, recognize, value},
    error::context,
    multi::{many1, many_till},
    sequence::{delimited, tuple},
    Parser,
};
use nom_supreme::tag::complete::{tag, tag_no_case};
use nom_supreme::ParserExt;

pub use enums::*;

use crate::Span;
use crate::{TokenizationError, TokenizationResult};

pub mod enums;
pub mod traits;

pub const LOWER_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPER_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

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

pub fn exact_literal_token<'t, O, P: Parser<&'t str, O, TokenizationError<'t>>>(
    mut parser: P,
) -> impl FnMut(&'t str) -> TokenizationResult<O> {
    move |input| parser.parse(input)
}

pub fn soft_literal_token(literal: Span) -> impl Parser<Span, Span, TokenizationError> {
    identifier
        .map(|it| match it {
            Identifier::Type(x) | Identifier::Identifier(x) => x,
        })
        .verify(move |&it| it == literal)
}

fn ignore(input: Span) -> TokenizationResult<Ignore> {
    let comment = not_line_ending
        .cut()
        .preceded_by(tag("//"))
        .recognize()
        .map(Ignore::Comment);
    let multiline_comment = tag("/*")
        .precedes(many_till(anychar, tag("*/")).cut())
        .recognize()
        .map(Ignore::MultilineComment);

    context(
        "ignore",
        alt((
            comment,
            value(Ignore::Whitespace, many1(is_a(" \t"))),
            value(Ignore::Newline, many1(is_a("\n\r"))),
            multiline_comment,
        )),
    )(input)
}

fn keyword(input: Span) -> TokenizationResult<Keyword> {
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
    )(input)
}

fn symbol(input: Span) -> TokenizationResult<Symbol> {
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
    )(input)
}

fn identifier(input: Span) -> TokenizationResult<Identifier> {
    let identifier_parser = |alphabet| {
        recognize(tuple((
            tag("_").opt(),
            one_of(alphabet),
            take_while(|it: char| it == '_' || it.is_alphanumeric()),
        )))
    };

    context(
        "identifier",
        alt((
            map(identifier_parser(LOWER_ALPHABET), Identifier::Identifier),
            map(identifier_parser(UPPER_ALPHABET), Identifier::Type),
        )),
    )(input)
}

fn literal(input: Span) -> TokenizationResult<Literal> {
    fn number_parser<'a>(
        prefix: &'static str,
        alphabet: &'static str,
    ) -> impl Parser<Span<'a>, &'a str, TokenizationError<'a>> {
        tag_no_case(prefix)
            .precedes(alt((
                one_of("0_")
                    .not()
                    .precedes(many1(one_of(alphabet).or(char('_'))))
                    .recognize(),
                one_of(alphabet).recognize(),
            )))
            .recognize()
    }

    let binary = number_parser("0b", "01");
    let octal = number_parser("0c", "01234567");
    let hex = number_parser("0x", "0123456789ABCDEFabcdef");
    let floating = recognize(tuple((
        opt(one_of("-+")),
        alt((
            tuple((digit1, opt(tuple((char('.'), digit0))))).recognize(),
            tuple((char('.'), digit1)).recognize(),
        )),
        opt(tuple((tag_no_case("e"), opt(one_of("-+")), digit1))),
    )));
    let char_p = delimited(char('\''), anychar.recognize(), char('\''));
    let string = delimited(char('"'), is_not(r#"""#).opt(), char('"'));

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
    )(input)
}

fn operator(input: Span) -> TokenizationResult<Operator> {
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
    )(input)
}

pub(crate) fn token(input: Span) -> TokenizationResult<Token> {
    context(
        "lexer",
        alt((
            map(ignore, Token::Ignore),
            map(keyword, Token::Keyword),
            map(symbol, Token::Symbol),
            map(identifier, Token::Identifier),
            map(literal, Token::Literal),
            map(operator, Token::Operator),
        )),
    )(input)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::fmt::Debug;

    use nom::Finish;
    use rstest::rstest;

    #[allow(unused_imports)]
    use crate::lexer::{ignore, Ignore};
    use crate::TokenizationResult;

    #[rstest]
    #[case::ignore_comment(ignore("// hello world!"), Ignore::Comment("// hello world!"), None)]
    #[case::ignore_comment_another_line(
        ignore("//hello world!\nthis is not comment"),
        Ignore::Comment("//hello world!"),
        Some("\nthis is not comment")
    )]
    #[case::ignore_multiline_comment(
        ignore("/* this is\nmultiline comment */"),
        Ignore::MultilineComment("/* this is\nmultiline comment */"),
        None
    )]
    #[case::ignore_multiline_comment_with_rest(
        ignore("/* this is\nmultiline comment */ this is not"),
        Ignore::MultilineComment("/* this is\nmultiline comment */"),
        Some(" this is not")
    )]
    #[case::ignore_newline(ignore("\n\n\n"), Ignore::Newline, None)]
    #[case::ignore_whitespace(ignore("   \t"), Ignore::Whitespace, None)]
    fn test_parser<T: PartialEq + Debug>(
        #[case] result: TokenizationResult<T>,
        #[case] expected: T,
        #[case] expected_rest: Option<&'static str>,
    ) {
        let (rest, data) = result.finish().unwrap();

        assert_eq!(rest, expected_rest.unwrap_or(""));
        assert_eq!(data, expected);
    }
}
