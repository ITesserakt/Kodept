use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::Token;
use crate::token_match::TokenMatch;
use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use peg::error::ParseError;
use peg::str::LineCol;

peg::parser! {grammar grammar() for str {
    rule newline() = "\n" / "\r\n" / "\r"

    rule comment() -> Token =
        "//" (!newline() [_])* &newline()? { Token::Comment }
    rule multiline_comment() -> Token =
        "/*" (!"*/" [_])* "*/" { Token::MultilineComment }
    rule whitespace() -> Token =
        ("\t" / " ") { Token::Whitespace }
    rule ignore() -> Token = quiet!{i:(
        comment()                                  /
        multiline_comment()                        /
        whitespace()+       { Token::Whitespace } /
        newline()           { Token::Newline }
    ) { i }}

    rule letter() = [c if c.is_alphanumeric()] / "_"
    rule keyword() -> Token =
        "fun" !letter()        { Token::Fun }       /
        "val" !letter()        { Token::Val }       /
        "var" !letter()        { Token::Var }       /
        "match" !letter()      { Token::Match }     /
        "while" !letter()      { Token::While }     /
        "module" !letter()     { Token::Module }    /
        "extend" !letter()     { Token::Extend }    /
        "return" !letter()     { Token::Return }    /
        "if" !letter()         { Token::If }        /
        "elif" !letter()       { Token::Elif }      /
        "else" !letter()       { Token::Else }      /
        "abstract" !letter()   { Token::Abstract }  /
        "trait" !letter()      { Token::Trait }     /
        "struct" !letter()     { Token::Struct }    /
        "class" !letter()      { Token::Class }     /
        "enum" !letter()       { Token::Enum }      /
        "foreign" !letter()    { Token::Foreign }   /
        "type" !letter()       { Token::TypeAlias } /
        "with" !letter()       { Token::With }

    rule symbol() -> Token =
        ","  { Token::Comma }       /
        ";"  { Token::Semicolon }   /
        "{"  { Token::LBrace }      /
        "}"  { Token::RBrace }      /
        "["  { Token::LBracket }    /
        "]"  { Token::RBracket }    /
        "("  { Token::LParen }      /
        ")"  { Token::RParen }      /
        "_"  { Token::TypeGap }     /
        "::" { Token::DoubleColon } /
        ":"  { Token::Colon }

    rule type_() -> Token = (
        "_"*
        (quiet!{[cl if cl.is_uppercase()]} / expected!("uppercase letter"))
        ("_" / (quiet!{[c if c.is_alphanumeric()]} / expected!("letter")))*
    ) { Token::Type }

    rule reference() -> Token = (
        "_"*
        (quiet!{[cl if cl.is_lowercase()]} / expected!("lowercase letter"))
        ("_" / (quiet!{[c if c.is_alphanumeric()]} / expected!("letter")))*
    ) { Token::Identifier }

    rule identifier() -> Token = reference() / type_()

    rule number<T>(prefix_lower: char, prefix_upper: char, digits: rule<T>) -> () = (
        "0" [c if c == prefix_lower || c == prefix_upper] (
            ((!("0" / "_") digits() { }) (digits() {  } / "_")*) /
            digits() {  }
        )
    )

    rule bin_lit() -> Token =
        i:number('b', 'B', <['0'..='1']>) { Token::Binary }
    rule oct_lit() -> Token =
        i:number('c', 'C', <['0'..='7']>) { Token::Octal }
    rule hex_lit() -> Token =
        i:number('x', 'X', <['0'..='9' | 'a'..='f' | 'A'..='F']>) { Token::Hex }

    rule sign() = ['+' | '-']
    rule floating_lit() =
        ['0'..='9']+ ("." ['0'..='9']*)? / "." ['0'..='9']+
    rule e_notation() =
        ['e' | 'E'] sign()? ['0'..='9']+

    rule literal() -> Token =
        bin_lit()                                                                    /
        oct_lit()                                                                    /
        hex_lit()                                                                    /
        sign()? floating_lit() e_notation()?               { Token::Floating } /
        "'" i:$(!"'" [_]) "'"                              { Token::Char }     /
        "\"" i:$((!"\"" [_])*) "\""                        { Token::String }

    rule operator() -> Token =
        "."   { Token::Dot }           /
        "=>"  { Token::Flow }          /
        "+"   { Token::Plus }          /
        "-"   { Token::Sub }           /
        "**"  { Token::Pow }           /
        "*"   { Token::Times }         /
        "/"   { Token::Div }           /
        "%"   { Token::Mod }           /
        "<=>" { Token::Spaceship }     /
        "=="  { Token::Equiv }         /
        "="   { Token::Equals }        /
        "!="  { Token::NotEquiv }      /
        ">="  { Token::GreaterEquals } /
        ">"   { Token::Greater }       /
        "<="  { Token::LessEquals }    /
        "<"   { Token::Less }          /
        "||"  { Token::OrLogic }       /
        "&&"  { Token::AndLogic }      /
        "!"   { Token::NotLogic }      /
        "|"   { Token::OrBit }         /
        "&"   { Token::AndBit }        /
        "^"   { Token::XorBit }        /
        "~"   { Token::NotBit }

    rule token_() -> Token =
        ignore()     /
        keyword()    /
        symbol()     /
        identifier() /
        literal()    /
        operator()

    rule token_match() -> TokenMatch =
        start:position!() t:token_() end:position!() {
            let length = end - start;
            TokenMatch::new(t, CodePoint::new(length as u32, start as u32))
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

    rule tokens_() -> Vec<TokenMatch> = i:token_match()* ![_] { i }

    pub rule tokens() -> Vec<TokenMatch> = traced(<tokens_()>)
    #[no_eof]
    pub rule token() -> TokenMatch = traced(<token_match()>)
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
        #[cfg(feature = "trace")]
        {
            return if !E {
                Self::Full(gag::Gag::stdout().expect("Cannot suppress stdout"))
            } else {
                Self::Empty
            };
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
    ) -> Result<TokenMatch, Self::Error<'t>> {
        let input = &whole_input[position..];
        let _gag = GagContainer::enable::<TRACE>();
        grammar::token(input)
    }
}

impl<const TRACE: bool> EagerTokensProducer for Lexer<TRACE> {
    type Error<'t> = ParseError<LineCol>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<TokenMatch>, Self::Error<'t>> {
        let _gag = GagContainer::enable::<TRACE>();
        grammar::tokens(input)
    }
}
