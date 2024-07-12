pub use enums::*;

pub mod enums;
pub mod traits;
#[cfg(feature = "nom")]
mod grammar;

#[cfg(feature = "nom")]
pub(crate) use grammar::token;

#[cfg(test)]
#[cfg(feature = "nom")]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::fmt::Debug;

    use nom::Finish;
    use rstest::rstest;

    #[allow(unused_imports)]
    use crate::lexer::{ignore, Ignore};
    use crate::TokenizationResult;

    #[rstest]
    #[case::ignore_comment(ignore("// hello world!"), Ignore::Comment("// hello world!"), None)]
    #[case::ignore_comment_another_line(
        ignore("//hello world!\nthis is not comment"),
        Ignore::Comment("//hello world!"),
        Some("\nthis is not comment")
    )]
    #[case::ignore_multiline_comment(
        ignore("/* this is\nmultiline comment */"),
        Ignore::MultilineComment("/* this is\nmultiline comment */"),
        None
    )]
    #[case::ignore_multiline_comment_with_rest(
        ignore("/* this is\nmultiline comment */ this is not"),
        Ignore::MultilineComment("/* this is\nmultiline comment */"),
        Some(" this is not")
    )]
    #[case::ignore_newline(ignore("\n\n\n"), Ignore::Newline, Some("\n\n"))]
    #[case::ignore_whitespace(ignore("   \t"), Ignore::Whitespace, None)]
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
