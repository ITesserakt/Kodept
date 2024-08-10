use derive_more::Constructor;
use faster_pest::*;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;

use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::Operator::*;
use crate::lexer::*;
use crate::token_match::TokenMatch;

#[derive(Parser)]
#[grammar = "crates/kodept-parse/src/pest/kodept.pest"]
struct Grammar;

fn parse_token_from_ident<'i>(input: Pair2<'i, Ident<'i>>) -> TokenMatch<'i> {
    let span = input.as_span();

    let token = match input.as_rule() {
        Rule::ignore => Token::Ignore({
            let input = input.into_inner().next().unwrap();
            match input.as_rule() {
                Rule::comment => Ignore::Comment(input.as_str()),
                Rule::multiline_comment => Ignore::MultilineComment(input.as_str()),
                Rule::newline => Ignore::Newline,
                Rule::whitespace => Ignore::Whitespace,
                _ => unreachable!(),
            }
        }),
        Rule::keyword => Token::Keyword(match input.as_str() {
            "fun" => Keyword::Fun,
            "val" => Keyword::Val,
            "var" => Keyword::Var,
            "match" => Keyword::Match,
            "while" => Keyword::While,
            "module" => Keyword::Module,
            "extend" => Keyword::Extend,
            "return" => Keyword::Return,
            "\\" => Keyword::Lambda,
            "if" => Keyword::If,
            "elif" => Keyword::Elif,
            "else" => Keyword::Else,
            "abstract" => Keyword::Abstract,
            "trait" => Keyword::Trait,
            "struct" => Keyword::Struct,
            "class" => Keyword::Class,
            "enum" => Keyword::Enum,
            "foreign" => Keyword::Foreign,
            "type" => Keyword::TypeAlias,
            "with" => Keyword::With,
            _ => unreachable!(),
        }),
        Rule::symbol => Token::Symbol(match input.as_str() {
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
            _ => unreachable!(),
        }),
        Rule::identifier => Token::Identifier(match input.as_str().as_bytes() {
            [b'_', b'a'..=b'z', ..] => Identifier::Identifier(input.as_str()),
            [b'a'..=b'z', ..] => Identifier::Identifier(input.as_str()),
            [b'_', b'A'..=b'Z', ..] => Identifier::Type(input.as_str()),
            [b'A'..=b'Z', ..] => Identifier::Type(input.as_str()),
            _ => unreachable!(),
        }),
        Rule::literal => Token::Literal({
            let input = input.into_inner().next().unwrap();
            match input.as_rule() {
                Rule::bin_lit => Literal::Binary(input.as_str()),
                Rule::oct_lit => Literal::Octal(input.as_str()),
                Rule::hex_lit => Literal::Hex(input.as_str()),
                Rule::flt_lit => Literal::Floating(input.as_str()),
                Rule::chr_lit => Literal::Char(input.as_str().trim_matches('\'')),
                Rule::str_lit => Literal::String(input.as_str().trim_matches('"')),
                _ => unreachable!(),
            }
        }),
        Rule::operator => Token::Operator(match input.as_str() {
            "." => Dot,
            "=>" => Flow,
            "+" => Math(MathOperator::Plus),
            "-" => Math(MathOperator::Sub),
            "**" => Math(MathOperator::Pow),
            "*" => Math(MathOperator::Times),
            "/" => Math(MathOperator::Div),
            "%" => Math(MathOperator::Mod),
            "<=>" => Comparison(ComparisonOperator::Spaceship),
            "==" => Comparison(ComparisonOperator::Equiv),
            "=" => Comparison(ComparisonOperator::Equals),
            "!=" => Comparison(ComparisonOperator::NotEquiv),
            ">=" => Comparison(ComparisonOperator::GreaterEquals),
            ">" => Comparison(ComparisonOperator::Greater),
            "<=" => Comparison(ComparisonOperator::LessEquals),
            "<" => Comparison(ComparisonOperator::Less),
            "||" => Logic(LogicOperator::OrLogic),
            "&&" => Logic(LogicOperator::AndLogic),
            "!" => Logic(LogicOperator::NotLogic),
            "|" => Bit(BitOperator::OrBit),
            "&" => Bit(BitOperator::AndBit),
            "^" => Bit(BitOperator::XorBit),
            "~" => Bit(BitOperator::NotBit),
            _ => unreachable!(),
        }),
        x => panic!("Unknown rule encountered: {x:?}"),
    };

    let length = span.end() - span.start();
    TokenMatch::new(token, Span::new(CodePoint::new(length as u32, span.start() as u32)))
}

#[derive(Constructor, Debug, Clone, Copy)]
pub struct Lexer;

impl TokenProducer for Lexer {
    type Error<'t> = pest::error::Error<Rule>;

    #[inline]
    fn parse_token<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<TokenMatch<'t>, Self::Error<'t>> {
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

    fn parse_tokens<'t>(&self, input: &'t str) -> Result<Vec<TokenMatch<'t>>, Self::Error<'t>> {
        let tokens = Grammar::parse(Rule::tokens, input).map_err(|e| e.into_pest(input))?;
        let idents = tokens.into_iter().next().unwrap().into_inner();
        Ok(idents
            .map(|it| it.into_inner().next().unwrap())
            .map(parse_token_from_ident)
            .collect())
    }
}
