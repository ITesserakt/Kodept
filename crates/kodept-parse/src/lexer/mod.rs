use cfg_if::cfg_if;

pub use enums::*;

pub mod enums;
pub mod traits;

cfg_if! {
	if #[cfg(feature = "pest")] {
        pub type DefaultLexer = PestLexer;
    } else if #[cfg(all(feature = "peg", not(feature = "trace")))] {
        pub type DefaultLexer = PegLexer<false>;
    } else if #[cfg(feature = "nom")] {
        pub type DefaultLexer = NomLexer;
    } else {
        compilation_error!("Either feature `peg` or `nom` or `pest` must be enabled for this crate")
    }
}

#[cfg(feature = "nom")]
pub type NomLexer = crate::nom::Lexer;
#[cfg(feature = "peg")]
pub type PegLexer<const TRACE: bool> = crate::peg::Lexer<TRACE>;
#[cfg(feature = "pest")]
pub type PestLexer = crate::pest::Lexer;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::common::TokenProducer;
    use crate::lexer::{DefaultLexer, Ignore::*, Token, Token::*};
    use rstest::rstest;
    use std::fmt::Debug;

    #[rstest]
    #[case::ignore_comment("// hello world!", Comment("// hello world!"), None)]
    // #[case::ignore_comment_another_line(
    //     "//hello world!\nthis is not comment",
    //     Comment("//hello world!"),
    //     Some("\nthis is not comment")
    // )]
    #[case::ignore_multiline_comment(
        "/* this is\nmultiline comment */",
        MultilineComment("/* this is\nmultiline comment */"),
        None
    )]
    #[case::ignore_multiline_comment_with_rest(
        "/* this is\nmultiline comment */ this is not",
        MultilineComment("/* this is\nmultiline comment */"),
        Some(" this is not")
    )]
    #[case::ignore_newline("\n\n\n", Ignore(Newline), Some("\n\n"))]
    #[case::ignore_whitespace("   \t", Ignore(Whitespace), None)]
    fn test_parser<T: PartialEq + Debug + Into<Token<'static>>>(
        #[case] input: &'static str,
        #[case] expected: T,
        #[case] expected_rest: Option<&'static str>,
    ) {
        let data = DefaultLexer::new().parse_string(input, 0).unwrap();
        let rest = &input[data.span.point.length as usize..];

        assert_eq!(data.token, expected.into());
        assert_eq!(rest, expected_rest.unwrap_or(""));
    }
}
