use crate::lexer::{Ignore::*, Token::Ignore};
use crate::token_match::TokenMatch;
use crate::token_stream::TokenStream;
use derive_more::Display;
use kodept_core::code_point::CodePoint;
use peg::{Parse, ParseElem, ParseLiteral, ParseSlice, RuleResult};
use tracing::error;
#[cfg(not(feature = "trace"))]
use crate::tokenizer::Tokenizer as Tokenizer;
#[cfg(feature = "trace")]
use crate::tokenizer::SimpleTokenizer as Tokenizer;

#[derive(Display, Copy, Clone, Debug)]
#[display(fmt = "{line}:{col}")]
pub struct Position {
    line: usize,
    col: usize,
    length: usize,
    offset: usize,
}

impl From<Position> for CodePoint {
    fn from(value: Position) -> Self {
        CodePoint::new(value.length, value.offset)
    }
}

impl<'t> Parse for TokenStream<'t> {
    type PositionRepr = Position;

    #[inline]
    fn start(&self) -> usize {
        self.slice.first().map_or(0, |it| it.span.point.offset)
    }

    #[inline]
    fn is_eof(&self, pos: usize) -> bool {
        pos >= self.len()
    }

    #[inline]
    fn position_repr(&self, pos: usize) -> Self::PositionRepr {
        let (before, point) = match self.slice.split_at(pos) {
            (a, [b, ..]) => (a, b.span.point),
            (a@[.., last], []) => (a, last.span.point),
            ([], []) => panic!("Cannot slice empty stream")
        };
        let line = before
            .iter()
            .filter(|it| matches!(it.token, Ignore(Newline)))
            .count()
            + 1;
        let col = before
            .iter()
            .rev()
            .take_while(|it| !matches!(it.token, Ignore(Newline)))
            .map(|it| it.span.point.length)
            .sum::<usize>() + 1;

        Position {
            line,
            col,
            length: point.length,
            offset: point.offset,
        }
    }
}

impl<'input> ParseElem<'input> for TokenStream<'input> {
    type Element = TokenMatch<'input>;

    #[inline]
    fn parse_elem(&'input self, pos: usize) -> RuleResult<Self::Element> {
        let slice = &self.slice[pos..];
        match slice
            .iter()
            .enumerate()
            .find(|(_, it)| !it.token.is_ignored())
        {
            None => RuleResult::Failed,
            Some((idx, token)) => RuleResult::Matched(pos + 1 + idx, *token),
        }
    }
}

impl<'input> ParseLiteral for TokenStream<'input> {
    #[inline]
    fn parse_string_literal(&self, pos: usize, literal: &str) -> RuleResult<()> {
        let Ok(tokenizer) = Tokenizer::try_new(literal) else {
            error!("Cannot parse given literal: {literal}");
            return RuleResult::Failed;
        };
        let tokens = tokenizer.into_vec();
        let l = tokens.len();

        let Some(slice) = self.slice.get(pos..pos + l) else {
            return RuleResult::Failed;
        };

        if TokenStream::new(slice)
            .token_iter()
            .zip(TokenStream::new(&tokens).token_iter())
            .all(|(a, b)| a == b)
        {
            RuleResult::Matched(pos + l, ())
        } else {
            RuleResult::Failed
        }
    }
}

impl<'input> ParseSlice<'input> for TokenStream<'input> {
    type Slice = TokenStream<'input>;

    fn parse_slice(&'input self, p1: usize, p2: usize) -> Self::Slice {
        TokenStream::new(&self.slice[p1..p2])
    }
}
