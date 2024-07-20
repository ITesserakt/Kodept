use derive_more::Display;
use EitherOrBoth::{Both, Left, Right};
use itertools::{EitherOrBoth, Itertools};
use peg::str::LineCol;
use peg::{Parse, ParseElem, ParseLiteral, ParseSlice, RuleResult};

use kodept_core::code_point::CodePoint;

use crate::lexer::{Ignore::*, Token::Ignore};
use crate::token_match::TokenMatch;
use crate::token_stream::TokenStream;
use crate::tokenizer::LazyTokenizer;

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

impl From<LineCol> for Position {
    fn from(value: LineCol) -> Self {
        Position {
            line: value.line,
            col: value.column,
            length: 1,
            offset: value.offset,
        }
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
            (a @ [.., last], []) => (a, last.span.point),
            ([], []) => panic!("Cannot slice empty stream"),
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
            .sum::<usize>()
            + 1;

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

    #[inline(always)]
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
    #[inline(always)]
    fn parse_string_literal(&self, pos: usize, literal: &str) -> RuleResult<()> {
        let tokenizer = LazyTokenizer::new(literal);

        let mut length = 0;
        for pair in self.slice[pos..].iter().zip_longest(tokenizer) {
            match pair {
                Both(a, b) if a.token != b.token => return RuleResult::Failed,
                Both(_, _) => length += 1,
                Right(_) => return RuleResult::Failed,
                Left(_) => break,
            }
        }
        RuleResult::Matched(pos + length, ())
    }
}

impl<'input> ParseSlice<'input> for TokenStream<'input> {
    type Slice = TokenStream<'input>;

    #[inline(always)]
    fn parse_slice(&'input self, p1: usize, p2: usize) -> Self::Slice {
        TokenStream::new(&self.slice[p1..p2])
    }
}
