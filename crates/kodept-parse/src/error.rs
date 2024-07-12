use crate::lexer::Token;
use crate::token_stream::TokenStream;
use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::rlt::RLT;
use std::borrow::Cow;

#[derive(Debug, Constructor)]
pub struct ErrorLocation {
    pub in_stream: usize,
    pub in_code: CodePoint,
}

#[derive(Debug, Constructor)]
pub struct ParseError<'t> {
    pub expected: Vec<Cow<'static, str>>,
    pub actual: Token<'t>,
    pub location: ErrorLocation,
}

#[inline(always)]
pub fn parse_from_top(stream: TokenStream) -> Result<RLT, Vec<ParseError>> {
    #[cfg(feature = "peg")]
    return peg::implementation(stream);
    #[cfg(all(not(feature = "peg"), feature = "nom"))]
    return default::implementation(stream);
    #[cfg(not(any(feature = "nom", feature = "peg")))]
    compile_error!("Either feature `peg` or `nom` must be enabled for this crate")
}

fn point_pos(stream: TokenStream, point: CodePoint) -> usize {
    stream
        .slice
        .iter()
        .position(|it| it.span.point == point)
        .unwrap()
}

#[cfg(feature = "peg")]
mod peg {
    use crate::error::{point_pos, ErrorLocation, ParseError};
    use crate::token_stream::TokenStream;
    use kodept_core::structure::rlt::RLT;
    use std::borrow::Cow;

    pub fn implementation(stream: TokenStream) -> Result<RLT, Vec<ParseError>> {
        let error = match crate::grammar::parser::kodept(&stream) {
            Ok(x) => return Ok(x),
            Err(x) => x,
        };
        let expected = error.expected.tokens().map(Cow::Borrowed).collect();
        let pos = point_pos(stream, error.location.into());
        let actual = stream.slice[pos].token;
        let location = ErrorLocation::new(pos, error.location.into());

        Err(vec![ParseError::new(expected, actual, location)])
    }
}

#[cfg(feature = "nom")]
pub mod default {
    use crate::error::{point_pos, ErrorLocation, ParseError};
    use crate::lexer::Token;
    use crate::parser::error::TokenVerificationError;
    use crate::parser::file::grammar;
    use crate::token_stream::TokenStream;
    use derive_more::Constructor;
    use itertools::Itertools;
    use kodept_core::structure::rlt::RLT;
    use nom_supreme::error::{BaseErrorKind, Expectation};
    use nom_supreme::final_parser::final_parser;
    use std::borrow::Cow;
    use std::collections::VecDeque;

    fn take_first_token(stream: TokenStream) -> Token {
        stream
            .slice
            .iter()
            .next()
            .map_or(Token::Unknown, |it| it.token)
    }

    #[derive(Debug, Constructor)]
    struct BaseError<'t, 's> {
        location: TokenStream<'t>,
        kind: BaseErrorKind<&'s str, TokenVerificationError>,
    }

    impl<'t, 's> BaseError<'t, 's> {
        pub fn into_expected(self) -> Cow<'static, str> {
            match self.kind {
                BaseErrorKind::Expected(Expectation::Something) => Cow::Borrowed("something"),
                BaseErrorKind::Expected(expectation) => Cow::Owned(expectation.to_string()),
                BaseErrorKind::Kind(kind) => Cow::Owned(kind.description().to_string()),
                BaseErrorKind::External(ext) => Cow::Borrowed(ext.expected),
            }
        }
    }

    pub fn implementation(stream: TokenStream) -> Result<RLT, Vec<ParseError>> {
        let error: crate::ParseError = match final_parser(grammar)(stream) {
            Ok(x) => return Ok(RLT(x)),
            Err(x) => x,
        };

        let mut current_errors = VecDeque::from([error]);
        let mut base_errors = vec![];
        loop {
            match current_errors.pop_front() {
                Some(crate::ParseError::Base { location, kind }) => {
                    base_errors.push(BaseError::new(location, kind))
                }
                Some(crate::ParseError::Stack { base, .. }) => current_errors.push_back(*base),
                Some(crate::ParseError::Alt(vec)) => current_errors.extend(vec),
                None => break,
            }
        }

        let parse_errors = base_errors
            .into_iter()
            .chunk_by(|it| it.location)
            .into_iter()
            .map(|(key, group)| {
                let point = key.slice[0].span.point;
                let pos = point_pos(stream, point);
                let error_loc = ErrorLocation::new(pos, point);
                let actual = take_first_token(key);
                let expected = group.map(|it| it.into_expected()).collect();
                ParseError::new(expected, actual, error_loc)
            })
            .collect();
        Err(parse_errors)
    }
}
