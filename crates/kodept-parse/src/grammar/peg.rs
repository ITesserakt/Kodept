use std::iter::FusedIterator;
use crate::lexer::*;
use crate::token_match::TokenMatch;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;
use peg::error::ParseError;
use peg::str::LineCol;

peg::parser! {grammar grammar() for str {
    rule newline() = "\n" / "\r\n" / "\r"

    rule comment() -> Ignore<'input> =
        i:$("//" (!newline() [_]) newline()) { Ignore::Comment(i) }
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
        i:literal()    { Token::Literal(i) }    /
        i:operator()   { Token::Operator(i) }

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

    rule tokens_() -> Vec<TokenMatch<'input>> =
        i:(start:position!() t:token_() end:position!() { (start, t, end) })* ![_]
    {
        i.into_iter().map(|(start, token, end)| {
            let length = end - start;
            TokenMatch::new(token, Span::new(CodePoint::new(length, start)))
        }).collect()
    }

    pub rule tokens() -> Vec<TokenMatch<'input>> = traced(<tokens_()>)
    pub rule token() -> Token<'input> = traced(<token_()>)
}}

pub struct Tokenizer<'t, const TRACE: bool> {
    tokens: Vec<TokenMatch<'t>>,
    pos: usize,
}

impl<'t, const TRACE: bool> Tokenizer<'t, TRACE> {
    #[cfg(feature = "trace")]
    pub fn try_new(input: &'t str) -> Result<Self, ParseError<LineCol>> {
        use gag::Gag;
        let mut _gag = None;
        if !TRACE {
            _gag = Some(Gag::stdout().expect("Cannot silence stdout"))
        }
        Ok(Self {
            tokens: grammar::tokens(input)?,
            pos: 0,
        })
    }

    #[cfg(not(feature = "trace"))]
    pub fn try_new(input: &'t str) -> Result<Self, ParseError<LineCol>> {
        Ok(Self {
            tokens: grammar::tokens(input)?,
            pos: 0,
        })
    }
    
    pub fn new(input: &'t str) -> Self {
        Self::try_new(input).unwrap()
    }

    pub fn into_vec(mut self) -> Vec<TokenMatch<'t>> {
        self.tokens.shrink_to_fit();
        self.tokens
    }
}

impl<'t, const TRACE: bool> Iterator for Tokenizer<'t, TRACE> {
    type Item = TokenMatch<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            self.pos += 1;
            Some(self.tokens[self.pos - 1])
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.tokens[self.pos..].len();
        (len, Some(len))
    }
}

impl<'t, const TRACE: bool> FusedIterator for Tokenizer<'t, TRACE> {}
impl<'t, const TRACE: bool> ExactSizeIterator for Tokenizer<'t, TRACE> {}

#[cfg(test)]
mod tests {
    use super::Tokenizer;

    #[test]
    fn test_iter_size() {
        let input = "\n\n\n";
        let mut tokenizer: Tokenizer<false> = Tokenizer::new(input);
        assert_eq!((3, Some(3)), tokenizer.size_hint());
        let _ = tokenizer.next();
        assert_eq!(2, tokenizer.len());
    }
}
