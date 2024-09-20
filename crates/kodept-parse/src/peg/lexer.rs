use crate::common::{EagerTokensProducer, TokenProducer};
use crate::lexer::PackedToken;
use crate::token_match::PackedTokenMatch;
use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use peg::error::ParseError;
use peg::str::LineCol;

peg::parser! {grammar grammar() for str {
    rule newline() = "\n" / "\r\n" / "\r"

    rule comment() -> PackedToken =
        "//" (!newline() [_])* &newline()? { PackedToken::Comment }
    rule multiline_comment() -> PackedToken =
        "/*" (!"*/" [_])* "*/" { PackedToken::MultilineComment }
    rule whitespace() -> PackedToken =
        ("\t" / " ") { PackedToken::Whitespace }
    rule ignore() -> PackedToken = quiet!{i:(
        comment()                                  /
        multiline_comment()                        /
        whitespace()+       { PackedToken::Whitespace } /
        newline()           { PackedToken::Newline }
    ) { i }}

    rule keyword() -> PackedToken =
        "fun"       { PackedToken::Fun }       /
        "val"       { PackedToken::Val }       /
        "var"       { PackedToken::Var }       /
        "match"     { PackedToken::Match }     /
        "while"     { PackedToken::While }     /
        "module"    { PackedToken::Module }    /
        "extend"    { PackedToken::Extend }    /
        "return"    { PackedToken::Return }    /
        "\\"        { PackedToken::Lambda }    /
        "if"        { PackedToken::If }        /
        "elif"      { PackedToken::Elif }      /
        "else"      { PackedToken::Else }      /
        "abstract"  { PackedToken::Abstract }  /
        "trait"     { PackedToken::Trait }     /
        "struct"    { PackedToken::Struct }    /
        "class"     { PackedToken::Class }     /
        "enum"      { PackedToken::Enum }      /
        "foreign"   { PackedToken::Foreign }   /
        "type"      { PackedToken::TypeAlias } /
        "with"      { PackedToken::With }

    rule symbol() -> PackedToken =
        ","  { PackedToken::Comma }       /
        ";"  { PackedToken::Semicolon }   /
        "{"  { PackedToken::LBrace }      /
        "}"  { PackedToken::RBrace }      /
        "["  { PackedToken::LBracket }    /
        "]"  { PackedToken::RBracket }    /
        "("  { PackedToken::LParen }      /
        ")"  { PackedToken::RParen }      /
        "_"  { PackedToken::TypeGap }     /
        "::" { PackedToken::DoubleColon } /
        ":"  { PackedToken::Colon }

    rule type_() -> PackedToken = (
        "_"*
        (quiet!{[cl if cl.is_uppercase()]} / expected!("uppercase letter"))
        ("_" / (quiet!{[c if c.is_alphanumeric()]} / expected!("letter")))*
    ) { PackedToken::Type }

    rule reference() -> PackedToken = (
        "_"*
        (quiet!{[cl if cl.is_lowercase()]} / expected!("lowercase letter"))
        ("_" / (quiet!{[c if c.is_alphanumeric()]} / expected!("letter")))*
    ) { PackedToken::Identifier }

    rule identifier() -> PackedToken = reference() / type_()

    rule number<T>(prefix_lower: char, prefix_upper: char, digits: rule<T>) -> () = (
        "0" [c if c == prefix_lower || c == prefix_upper] (
            ([^'0' | '_'] (digits() {  } / "_")+) /
            digits() {  }
        )
    )

    rule bin_lit() -> PackedToken =
        i:number('b', 'B', <['0'..='1']>) { PackedToken::Binary }
    rule oct_lit() -> PackedToken =
        i:number('c', 'C', <['0'..='7']>) { PackedToken::Octal }
    rule hex_lit() -> PackedToken =
        i:number('x', 'X', <['0'..='9' | 'a'..='f' | 'A'..='F']>) { PackedToken::Hex }

    rule sign() = ['+' | '-']
    rule floating_lit() =
        ['0'..='9']+ ("." ['0'..='9']*)? / "." ['0'..='9']+
    rule e_notation() =
        ['e' | 'E'] sign()? ['0'..='9']+

    rule literal() -> PackedToken =
        bin_lit()                                                                    /
        oct_lit()                                                                    /
        hex_lit()                                                                    /
        sign()? whitespace()* floating_lit() e_notation()? { PackedToken::Floating } /
        "'" i:$(!"'" [_]) "'"                              { PackedToken::Char }     /
        "\"" i:$((!"\"" [_])*) "\""                        { PackedToken::String }

    rule operator() -> PackedToken =
        "."   { PackedToken::Dot }           /
        "=>"  { PackedToken::Flow }          /
        "+"   { PackedToken::Plus }          /
        "-"   { PackedToken::Sub }           /
        "**"  { PackedToken::Pow }           /
        "*"   { PackedToken::Times }         /
        "/"   { PackedToken::Div }           /
        "%"   { PackedToken::Mod }           /
        "<=>" { PackedToken::Spaceship }     /
        "=="  { PackedToken::Equiv }         /
        "="   { PackedToken::Equals }        /
        "!="  { PackedToken::NotEquiv }      /
        ">="  { PackedToken::GreaterEquals } /
        ">"   { PackedToken::Greater }       /
        "<="  { PackedToken::LessEquals }    /
        "<"   { PackedToken::Less }          /
        "||"  { PackedToken::OrLogic }       /
        "&&"  { PackedToken::AndLogic }      /
        "!"   { PackedToken::NotLogic }      /
        "|"   { PackedToken::OrBit }         /
        "&"   { PackedToken::AndBit }        /
        "^"   { PackedToken::XorBit }        /
        "~"   { PackedToken::NotBit }

    rule token_() -> PackedToken =
        ignore()     /
        keyword()    /
        symbol()     /
        identifier() /
        operator()   /
        literal()

    rule token_match() -> PackedTokenMatch =
        start:position!() t:token_() end:position!() {
            let length = end - start;
            PackedTokenMatch::new(t, CodePoint::new(length as u32, start as u32))
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

    rule tokens_() -> Vec<PackedTokenMatch> = i:token_match()* ![_] { i }

    pub rule tokens() -> Vec<PackedTokenMatch> = traced(<tokens_()>)
    #[no_eof]
    pub rule token() -> PackedTokenMatch = traced(<token_match()>)
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
    ) -> Result<PackedTokenMatch, Self::Error<'t>> {
        let input = &whole_input[position..];
        let _gag = GagContainer::enable::<TRACE>();
        grammar::token(input)
    }
}

impl<const TRACE: bool> EagerTokensProducer for Lexer<TRACE> {
    type Error<'t> = ParseError<LineCol>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<PackedTokenMatch>, Self::Error<'t>> {
        let _gag = GagContainer::enable::<TRACE>();
        grammar::tokens(input)
    }
}
