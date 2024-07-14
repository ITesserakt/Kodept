use cfg_if::cfg_if;
pub use enums::*;

pub mod enums;
pub mod traits;
#[cfg(feature = "nom")]
mod grammar;

#[cfg(feature = "nom")]
pub(crate) use grammar::*;
use crate::error::{ParseErrors};
use crate::token_match::TokenMatch;

#[inline]
pub fn parse_token<'t>(input: &'t str, all_input: &'t str) -> Result<TokenMatch<'t>, ParseErrors<&'t str>> {
    cfg_if! {
        if #[cfg(all(feature = "peg", not(feature = "trace")))] {
            let token = match crate::grammar::peg::token(input) {
                Ok(tok) => tok,
                Err(e) => return Err((e, all_input).into()),
            };
            Ok(token)
        }
        else if #[cfg(feature = "nom")] {
            use kodept_core::structure::span::Span;
            use kodept_core::code_point::CodePoint;
            use nom::Err::{Error, Failure};
            use nom::Err::Incomplete;
            
            let (rest, token) = match token(input) {
                Ok(tok) => tok,
                Err(Error(e) | Failure(e)) => return Err((e, all_input).into()),
                Err(Incomplete(_)) => ("", Token::Unknown)
            };
            let matched_length = input.len() - rest.len();
            Ok(TokenMatch::new(token, Span::new(CodePoint::new(matched_length, 0))))
        } else {
            compile_error!("Either feature `peg` or `nom` must be enabled for this crate")
        }
    }
}

#[cfg(test)]
#[cfg(feature = "nom")]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::fmt::Debug;

    use nom::Finish;
    use rstest::rstest;

    #[allow(unused_imports)]
    use crate::lexer::{token, Ignore::*, Token::Ignore};
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
