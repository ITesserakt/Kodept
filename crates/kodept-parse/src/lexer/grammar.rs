use std::borrow::Cow;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, take_while};
use nom::character::complete::{anychar, char, digit0, digit1, not_line_ending, one_of};
use nom::combinator::{map, opt, recognize, value};
use nom::error::context;
use nom::multi::{many1, many_till};
use nom::Parser;
use nom::sequence::{delimited, tuple};
use nom_supreme::ParserExt;
use nom_supreme::tag::complete::{tag, tag_no_case};
use crate::lexer::{BitOperator, ComparisonOperator, Identifier, Ignore, Keyword, Literal, LogicOperator, MathOperator, Operator, Symbol, Token};
use crate::{Span, TokenizationResult};
use crate::error::ParseErrors;
use crate::token_match::TokenMatch;

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

pub fn soft_literal_token(literal: Span) -> impl Parser<Span, Span, crate::TokenizationError> {
    identifier
        .map(|it| match it {
            Identifier::Type(x) | Identifier::Identifier(x) => x,
        })
        .verify(move |&it| it == literal)
}

pub fn ignore(input: Span) -> TokenizationResult<Ignore> {
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
            value(Ignore::Newline, one_of("\r\n")),
            multiline_comment,
        )),
    )(input)
}

pub fn keyword(input: Span) -> TokenizationResult<Keyword> {
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

pub fn symbol(input: Span) -> TokenizationResult<Symbol> {
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

pub fn identifier(input: Span) -> TokenizationResult<Identifier> {
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

pub fn literal(input: Span) -> TokenizationResult<Literal> {
    fn number_parser<'a>(
        prefix: &'static str,
        alphabet: &'static str,
    ) -> impl Parser<Span<'a>, &'a str, crate::TokenizationError<'a>> {
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

pub fn operator(input: Span) -> TokenizationResult<Operator> {
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

fn token(input: Span) -> TokenizationResult<Token> {
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

pub fn parse_token<'t>(input: &'t str, all_input: &'t str) -> Result<TokenMatch<'t>, ParseErrors<&'t str>> {
    use kodept_core::structure::span::Span;
    use kodept_core::code_point::CodePoint;
    use nom::Err::{Error, Failure};
    use nom::Err::Incomplete;

    let (rest, token) = match token(input) {
        Ok(tok) => tok,
        Err(Error(e) | Failure(e)) => return Err(ParseErrors::from((e, all_input))),
        Err(Incomplete(_)) => ("", Token::Unknown)
    };
    let matched_length = input.len() - rest.len();
    Ok(TokenMatch::new(token, Span::new(CodePoint::new(matched_length, 0))))
}