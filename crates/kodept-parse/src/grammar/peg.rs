use crate::lexer::*;
use crate::token_match::TokenMatch;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;

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
        newline()+          { Ignore::Newline }
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
            [^'0'] digits() (digits() {  } / "_")* digits() /
            digits() {  }
        )
    ) { i }

    rule literal() -> Literal<'input> =
        i:number('b', 'B', <['0'..'1']>)                           { Literal::Binary(i) }  /
        i:number('o', 'O', <['0'..'7']>)                           { Literal::Octal(i) }   /
        i:number('x', 'X', <['0'..='9' | 'a'..='f' | 'A'..='F']>)  { Literal::Hex(i) }     /
        i:$(['+' | '-']? whitespace()* (
            ['0'..='9']+ ("." ['0'..='9']*)? /
            "." ['0'..='9']+
        ) (['e' | 'E'] ['+' | '-']? ['0'..='9']+)?)                { Literal::Floating(i) } /
        "'" i:$(!"'" [_]) "'"                                      { Literal::Char(i) }     /
        "\"" i:$((!"\"" [_])*) "\""                                { Literal::String(i) }

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

    // TODO: proper error handling
    rule token() -> Token<'input> =
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
        i:(start:position!() t:token() end:position!() { (start, t, end) })* ![_]
    {
        i.into_iter().map(|(start, token, end)| {
            let length = end - start;
            TokenMatch::new(token, Span::new(CodePoint::new(length, start)))
        }).collect()
    }

    pub rule tokens() -> Vec<TokenMatch<'input>> = traced(<tokens_()>)
}}

pub struct Tokenizer<'t> {
    tokens: Vec<TokenMatch<'t>>,
    pos: usize,
}

impl<'t> Tokenizer<'t> {
    pub fn new(input: &'t str) -> Self {
        Self {
            tokens: grammar::tokens(input).unwrap(),
            pos: 0,
        }
    }

    pub fn into_vec(mut self) -> Vec<TokenMatch<'t>> {
        self.tokens.shrink_to_fit();
        self.tokens
    }
}

impl<'t> Iterator for Tokenizer<'t> {
    type Item = TokenMatch<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            self.pos += 1;
            Some(self.tokens[self.pos - 1])
        }
    }
}
