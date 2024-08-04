use cfg_if::cfg_if;

pub use enums::*;

pub mod enums;
pub mod traits;

cfg_if! {
    if #[cfg(all(feature = "peg", not(feature = "trace")))] {
        pub type DefaultLexer = PegLexer<{crate::TRACING_OPTION}>; 
    } else if #[cfg(feature = "pest")] {
        pub type DefaultLexer = PestLexer;
    } else if #[cfg(feature = "nom")] {
        pub type DefaultLexer = NomLexer;
    } else {
        compilation_error!("Either feature `peg` or `nom` or `pest` must be enabled for this crate")
    }
}

#[cfg(feature = "nom")]
pub type NomLexer = crate::nom::Lexer;
#[cfg(feature = "peg")]
pub type PegLexer<const TRACE: bool = false> = crate::peg::Lexer<TRACE>;
#[cfg(feature = "pest")]
pub type PestLexer = crate::pest::Lexer;

#[cfg(test)]
#[cfg(feature = "nom")]
#[cfg(not(all()))]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::fmt::Debug;

    use nom::Finish;
    use rstest::rstest;

    #[allow(unused_imports)]
    use crate::lexer::{Ignore::*, token, Token::Ignore};
    use crate::TokenizationResult;

    #[rstest]
    #[case::ignore_comment(token("// hello world!"), Ignore(Comment("// hello world!")), None)]
    #[case::ignore_comment_another_line(
        token("//hello world!\nthis is not comment"),
        Ignore(Comment("//hello world!")),
        Some("\nthis is not comment")
    )]
    #[case::ignore_multiline_comment(
        token("/* this is\nmultiline comment */"),
        Ignore(MultilineComment("/* this is\nmultiline comment */")),
        None
    )]
    #[case::ignore_multiline_comment_with_rest(
        token("/* this is\nmultiline comment */ this is not"),
        Ignore(MultilineComment("/* this is\nmultiline comment */")),
        Some(" this is not")
    )]
    #[case::ignore_newline(token("\n\n\n"), Ignore(Newline), Some("\n\n"))]
    #[case::ignore_whitespace(token("   \t"), Ignore(Whitespace), None)]
    fn test_parser<T: PartialEq + Debug>(
        #[case] result: TokenizationResult<T>,
        #[case] expected: T,
        #[case] expected_rest: Option<&'static str>,
    ) {
        let (rest, data) = result.finish().unwrap();

        assert_eq!(rest, expected_rest.unwrap_or(""));
        assert_eq!(data, expected);
    }
}
