use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::Token;
use crate::lexer::{BitOperator, ComparisonOperator, LogicOperator, MathOperator};
use crate::lexer::{Identifier, Ignore, Keyword, Literal, Operator, Symbol};
use crate::token_match::TokenMatch;
use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;
use peg::error::ParseError;
use peg::str::LineCol;

peg::parser! {grammar grammar() for str {
    rule newline() = "\n" / "\r\n" / "\r"

    rule comment() -> Ignore<'input> =
        i:$("//" (!newline() [_])* &newline()?) { Ignore::Comment(i) }
    rule multiline_comment() -> Ignore<'input> =
        i:$( "/*" (!"*/" [_])* "*/") { Ignore::MultilineComment(i) }
    rule whitespace() -> Ignore<'input> =
        ("\t" / " ") { Ignore::Whitespace }
    rule ignore() -> Ignore<'input> = quiet!{i:(
        comment()                                  /
        multiline_comment()                        /
        whitespace()+       { Ignore::Whitespace } /
        newline()           { Ignore::Newline }
    ) { i }}

    rule keyword() -> Keyword =
        "fun"       { Keyword::Fun }       /
        "val"       { Keyword::Val }       /
        "var"       { Keyword::Var }       /
        "match"     { Keyword::Match }     /
        "while"     { Keyword::While }     /
        "module"    { Keyword::Module }    /
        "extend"    { Keyword::Extend }    /
        "return"    { Keyword::Return }    /
        "\\"        { Keyword::Lambda }    /
        "if"        { Keyword::If }        /
        "elif"      { Keyword::Elif }      /
        "else"      { Keyword::Else }      /
        "abstract"  { Keyword::Abstract }  /
        "trait"     { Keyword::Trait }     /
        "struct"    { Keyword::Struct }    /
        "class"     { Keyword::Class }     /
        "enum"      { Keyword::Enum }      /
        "foreign"   { Keyword::Foreign }   /
        "type"      { Keyword::TypeAlias } /
        "with"      { Keyword::With }

    rule symbol() -> Symbol =
        ","  { Symbol::Comma }       /
        ";"  { Symbol::Semicolon }   /
        "{"  { Symbol::LBrace }      /
        "}"  { Symbol::RBrace }      /
        "["  { Symbol::LBracket }    /
        "]"  { Symbol::RBracket }    /
        "("  { Symbol::LParen }      /
        ")"  { Symbol::RParen }      /
        "_"  { Symbol::TypeGap }     /
        "::" { Symbol::DoubleColon } /
        ":"  { Symbol::Colon }

    rule type_() -> Identifier<'input> = i:$(
        "_"*
        (quiet!{[cl if cl.is_uppercase()]} / expected!("uppercase letter"))
        ("_" / (quiet!{[c if c.is_alphanumeric()]} / expected!("letter")))*
    ) { Identifier::Type(i) }

    rule reference() -> Identifier<'input> = i:$(
        "_"*
        (quiet!{[cl if cl.is_lowercase()]} / expected!("lowercase letter"))
        ("_" / (quiet!{[c if c.is_alphanumeric()]} / expected!("letter")))*
    ) { Identifier::Identifier(i) }

    rule identifier() -> Identifier<'input> = reference() / type_()

    rule number<T>(prefix_lower: char, prefix_upper: char, digits: rule<T>) -> &'input str = i:$(
        "0" [c if c == prefix_lower || c == prefix_upper] (
            ([^'0' | '_'] (digits() {  } / "_")+) /
            digits() {  }
        )
    ) { i }

    rule bin_lit() -> Literal<'input> =
        i:number('b', 'B', <['0'..='1']>) { Literal::Binary(i) }
    rule oct_lit() -> Literal<'input> =
        i:number('c', 'C', <['0'..='7']>) { Literal::Octal(i) }
    rule hex_lit() -> Literal<'input> =
        i:number('x', 'X', <['0'..='9' | 'a'..='f' | 'A'..='F']>) { Literal::Hex(i) }

    rule sign() = ['+' | '-']
    rule floating_lit() =
        ['0'..='9']+ ("." ['0'..='9']*)? / "." ['0'..='9']+
    rule e_notation() =
        ['e' | 'E'] sign()? ['0'..='9']+

    rule literal() -> Literal<'input> =
        bin_lit()                                                                        /
        oct_lit()                                                                        /
        hex_lit()                                                                        /
        i:$(sign()? whitespace()* floating_lit() e_notation()?) { Literal::Floating(i) } /
        "'" i:$(!"'" [_]) "'"                                   { Literal::Char(i) }     /
        "\"" i:$((!"\"" [_])*) "\""                             { Literal::String(i) }

    rule operator() -> Operator =
        "."   { Operator::Dot }                                            /
        "=>"  { Operator::Flow }                                           /
        "+"   { Operator::Math(MathOperator::Plus) }                       /
        "-"   { Operator::Math(MathOperator::Sub) }                        /
        "**"  { Operator::Math(MathOperator::Pow) }                        /
        "*"   { Operator::Math(MathOperator::Times) }                      /
        "/"   { Operator::Math(MathOperator::Div) }                        /
        "%"   { Operator::Math(MathOperator::Mod) }                        /
        "<=>" { Operator::Comparison(ComparisonOperator::Spaceship) }      /
        "=="  { Operator::Comparison(ComparisonOperator::Equiv) }          /
        "="   { Operator::Comparison(ComparisonOperator::Equals) }         /
        "!="  { Operator::Comparison(ComparisonOperator::NotEquiv) }       /
        ">="  { Operator::Comparison(ComparisonOperator::GreaterEquals) }  /
        ">"   { Operator::Comparison(ComparisonOperator::Greater) }        /
        "<="  { Operator::Comparison(ComparisonOperator::LessEquals) }     /
        "<"   { Operator::Comparison(ComparisonOperator::Less) }           /
        "||"  { Operator::Logic(LogicOperator::OrLogic) }                  /
        "&&"  { Operator::Logic(LogicOperator::AndLogic) }                 /
        "!"   { Operator::Logic(LogicOperator::NotLogic) }                 /
        "|"   { Operator::Bit(BitOperator::OrBit) }                        /
        "&"   { Operator::Bit(BitOperator::AndBit) }                       /
        "^"   { Operator::Bit(BitOperator::XorBit) }                       /
        "~"   { Operator::Bit(BitOperator::NotBit) }

    rule token_() -> Token<'input> =
        i:ignore()     { Token::Ignore(i) }     /
        i:keyword()    { Token::Keyword(i) }    /
        i:symbol()     { Token::Symbol(i) }     /
        i:identifier() { Token::Identifier(i) } /
        i:operator()   { Token::Operator(i) }   /
        i:literal()    { Token::Literal(i) }

    rule token_match() -> TokenMatch<'input> =
        start:position!() t:token_() end:position!() {
            let length = end - start;
            TokenMatch::new(t, Span::new(CodePoint::new(length as u32, start as u32)))
        }

    rule traced<T>(e: rule<T>) -> T =
        &(input:$([_]*) {
            #[cfg(feature = "trace")]
            println!("[PEG_INPUT_START]\n{}\n[PEG_TRACE_START]", input);
        })
        e:e()? {?
            #[cfg(feature = "trace")]
            println!("[PEG_TRACE_STOP]");
            e.ok_or("")
        }

    rule tokens_() -> Vec<TokenMatch<'input>> = i:token_match()* ![_] { i }

    pub rule tokens() -> Vec<TokenMatch<'input>> = traced(<tokens_()>)
    #[no_eof]
    pub rule token() -> TokenMatch<'input> = traced(<token_match()>)
}}

#[derive(Constructor, Debug, Copy, Clone)]
pub struct Lexer<const TRACE: bool>;

#[allow(dead_code)]
enum GagContainer {
    Empty,
    #[cfg(feature = "trace")]
    Full(gag::Gag),
}

impl GagContainer {
    #[must_use]
    fn enable<const E: bool>() -> Self {
        #[cfg(feature = "trace")] {
            return if !E {
                Self::Full(gag::Gag::stdout().expect("Cannot suppress stdout"))
            } else {
                Self::Empty
            }
        }
        #[cfg(not(feature = "trace"))]
        Self::Empty
    }
}

impl<const TRACE: bool> TokenProducer for Lexer<TRACE> {
    type Error<'t> = ParseError<LineCol>;

    fn parse_string<'t>(
        &self,
        whole_input: &'t str,
        position: usize,
    ) -> Result<TokenMatch<'t>, Self::Error<'t>> {
        let input = &whole_input[position..];
        let _gag = GagContainer::enable::<TRACE>();
        grammar::token(input)
    }
}

impl<const TRACE: bool> EagerTokensProducer for Lexer<TRACE> {
    type Error<'t> = ParseError<LineCol>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<TokenMatch<'t>>, Self::Error<'t>> {
        let _gag = GagContainer::enable::<TRACE>();
        grammar::tokens(input)
    }
}
