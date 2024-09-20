use crate::lexer::Token;
use crate::token_match::TokenMatch;
use crate::token_stream::TokenStream;
use std::convert::Infallible;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct CompatIter<'t>(std::slice::Iter<'t, TokenMatch<'t>>);

impl<'t> CompatIter<'t> {
    pub fn new(stream: TokenStream<'t>) -> Self {
        Self(stream.slice.into_iter())
    }
}

impl<'t> Iterator for CompatIter<'t> {
    type Item = Spanned<Token<'t>, usize, Infallible>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let mut token_match = self.0.next()?;
        while token_match.token.is_ignored() {
            token_match = self.0.next()?;
        }
        
        let range = token_match.span.point.as_range();
        Some(Ok((range.start, token_match.token, range.end)))
    }
}
