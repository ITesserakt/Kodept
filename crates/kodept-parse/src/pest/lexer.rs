use derive_more::Constructor;
use faster_pest::*;

use kodept_core::code_point::CodePoint;

use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::*;
use crate::token_match::{PackedTokenMatch};

#[derive(Parser)]
#[grammar = "crates/kodept-parse/src/pest/kodept.pest"]
struct Grammar;

fn parse_token_from_ident<'i>(input: Pair2<'i, Ident<'i>>) -> PackedTokenMatch {
    let span = input.as_span();

    let token = match input.as_rule() {
        Rule::ignore => match input.into_inner().next().unwrap().as_rule() {
            Rule::comment => PackedToken::Comment,
            Rule::multiline_comment => PackedToken::MultilineComment,
            Rule::newline => PackedToken::Newline,
            Rule::whitespace => PackedToken::Whitespace,
            _ => unreachable!(),
        },
        Rule::keyword => PackedToken::from_name(input.as_str())
            .expect("Found a keyword not defined in PackedToken"),
        Rule::symbol => PackedToken::from_name(input.as_str())
            .expect("Found a symbol not defined in PackedToken"),
        Rule::identifier => match input.as_str().as_bytes() {
            [b'_', b'a'..=b'z', ..] => PackedToken::Identifier,
            [b'a'..=b'z', ..] => PackedToken::Identifier,
            [b'_', b'A'..=b'Z', ..] => PackedToken::Type,
            [b'A'..=b'Z', ..] => PackedToken::Type,
            _ => unreachable!(),
        },
        Rule::literal => match input.into_inner().next().unwrap().as_rule() {
            Rule::bin_lit => PackedToken::Binary,
            Rule::oct_lit => PackedToken::Octal,
            Rule::hex_lit => PackedToken::Hex,
            Rule::flt_lit => PackedToken::Floating,
            Rule::chr_lit => PackedToken::Char,
            Rule::str_lit => PackedToken::String,
            _ => unreachable!(),
        },
        Rule::operator => match input.as_str() {
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
            _ => unreachable!(),
        },
        x => panic!("Unknown rule encountered: {x:?}"),
    };

    let length = span.end() - span.start();
    PackedTokenMatch::new(token, CodePoint::new(length as u32, span.start() as u32))
}

#[derive(Constructor, Debug, Clone, Copy)]
pub struct Lexer;

impl TokenProducer for Lexer {
    type Error<'t> = pest::error::Error<Rule>;

    #[inline]
    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<PackedTokenMatch, Self::Error<'t>> {
        let input = &whole_input[position..];
        let token = Grammar::parse(Rule::token, input).map_err(|e| e.into_pest(input))?;
        let ident = token
            .into_iter()
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap();
        Ok(parse_token_from_ident(ident))
    }
}

impl EagerTokensProducer for Lexer {
    type Error<'t> = pest::error::Error<Rule>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>> {
        let tokens = Grammar::parse(Rule::tokens, input).map_err(|e| e.into_pest(input))?;
        let idents = tokens.into_iter().next().unwrap().into_inner();
        Ok(idents
            .map(|it| it.into_inner().next().unwrap())
            .map(parse_token_from_ident)
            .collect())
    }
}
