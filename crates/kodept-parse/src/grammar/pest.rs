use faster_pest::*;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;
use crate::error::{ParseErrors};
use crate::lexer::{BitOperator, ComparisonOperator, Identifier, Ignore, Keyword, Literal, LogicOperator, MathOperator, Symbol, Token};
use crate::lexer::Operator::{Bit, Comparison, Dot, Flow, Logic, Math};
use crate::token_match::TokenMatch;

#[derive(Parser)]
#[grammar = "crates/kodept-parse/src/grammar/kodept.pest"]
struct Grammar;

pub fn parse_token(input: &str) -> Result<TokenMatch, ParseErrors<String>> {
    let token = Grammar::parse(Rule::token, input).map_err(|e| e.into_pest(input))?;
    let ident = token.into_iter().next().unwrap().into_inner().next().unwrap();
    Ok(parse_token_from_ident(ident))
}

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
        _ => Token::Unknown,
    };

    let length = span.end() - span.start();
    TokenMatch::new(token, Span::new(CodePoint::new(length, span.start())))
}

pub struct Tokenizer<'i> {
    pairs: Pairs2<'i, Ident<'i>>,
}

impl<'i> Tokenizer<'i> {
    pub fn try_new(input: &'i str) -> Result<Self, ParseErrors<String>> {
        let mut pairs =
            Grammar::parse(Rule::tokens, input).map_err(|e| e.into_pest(input))?;
        Ok(Self {
            pairs: pairs.next().unwrap().into_inner(),
        })
    }

    pub fn new(input: &'i str) -> Self {
        Self::try_new(input).unwrap()
    }

    pub fn into_vec(self) -> Vec<TokenMatch<'i>> {
        let mut vec: Vec<_> = self.collect();
        vec.shrink_to_fit();
        vec
    }
}

impl<'i> Iterator for Tokenizer<'i> {
    type Item = TokenMatch<'i>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pairs.by_ref()
            .filter(|it| matches!(it.as_rule(), Rule::token))
            .flat_map(|it| it.into_inner())
            .map(parse_token_from_ident)
            .next()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Ignore, Literal, LogicOperator, Operator, Token};
    use crate::lexer::Identifier::{Identifier, Type};
    use crate::grammar::pest::{Tokenizer};

    #[test]
    fn test_example() {
        let input = "Hello world! 1234";
        let output = Tokenizer::new(input);
        let tokens = output.map(|it| it.token).collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier(Type("Hello")),
                Token::Ignore(Ignore::Whitespace),
                Token::Identifier(Identifier("world")),
                Token::Operator(Operator::Logic(LogicOperator::NotLogic)),
                Token::Ignore(Ignore::Whitespace),
                Token::Literal(Literal::Floating("1234"))
            ]
        )
    }
}
