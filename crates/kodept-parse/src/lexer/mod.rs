pub use enums::*;

pub mod enums;
pub mod traits;

pub type NomLexer = crate::nom::Lexer;
pub type PegLexer<const TRACE: bool> = crate::peg::Lexer<TRACE>;
pub type PestLexer = crate::pest::Lexer;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::common::TokenProducer;
    use crate::lexer::{PackedToken, PackedToken::*, PegLexer};
    use rstest::rstest;

    #[rstest]
    #[case::ignore_comment("// hello world!", Comment, None)]
    #[case::ignore_comment_another_line(
        "//hello world!\nthis is not comment",
        Comment,
        Some("\nthis is not comment")
    )]
    #[case::ignore_multiline_comment("/* this is\nmultiline comment */", MultilineComment, None)]
    #[case::ignore_multiline_comment_with_rest(
        "/* this is\nmultiline comment */ this is not",
        MultilineComment,
        Some(" this is not")
    )]
    #[case::ignore_newline("\n\n\n", Newline, Some("\n\n"))]
    #[case::ignore_whitespace("   \t", Whitespace, None)]
    fn test_parser(
        #[case] input: &'static str,
        #[case] expected: PackedToken,
        #[case] expected_rest: Option<&'static str>,
    ) {
        let data = PegLexer::<true>::new().parse_string(input, 0).unwrap();
        let rest = &input[data.point.length as usize..];

        assert_eq!(expected, data.token);
        assert_eq!(rest, expected_rest.unwrap_or(""));
    }
}
