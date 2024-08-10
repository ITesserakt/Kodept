use crate::common::TokenProducer;
use crate::lexer::{DefaultLexer, Ignore::*, Token::Ignore};
use crate::token_match::TokenMatch;
use crate::token_stream::TokenStream;
use derive_more::Display;
use kodept_core::code_point::CodePoint;
use peg::str::LineCol;
use peg::{Parse, ParseElem, ParseLiteral, ParseSlice, RuleResult};

#[derive(Display, Copy, Clone, Debug)]
#[display("{line}:{col}")]
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

    #[inline(always)]
    fn start(&self) -> usize {
        self.slice.first().map_or(0, |it| it.span.point.offset)
    }

    #[inline(always)]
    fn is_eof(&self, pos: usize) -> bool {
        pos >= self.len()
    }

    #[inline(always)]
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
        match slice.first() {
            None => RuleResult::Failed,
            Some(x) => RuleResult::Matched(pos + 1, *x),
        }
    }
}

impl<'input> ParseLiteral for TokenStream<'input> {
    #[inline(always)]
    fn parse_string_literal(&self, pos: usize, literal: &str) -> RuleResult<()> {
        let token_match = DefaultLexer::new()
            .parse_token(literal, 0)
            .expect("Unexpected token received in grammar");
        debug_assert_eq!(token_match.span.point.length, literal.len());

        match (self.slice.get(pos), token_match.token) {
            (Some(a), b) if a.token == b => RuleResult::Matched(pos + 1, ()),
            _ => RuleResult::Failed,
        }
    }
}

impl<'input> ParseSlice<'input> for TokenStream<'input> {
    type Slice = TokenStream<'input>;

    #[inline(always)]
    fn parse_slice(&'input self, p1: usize, p2: usize) -> Self::Slice {
        TokenStream::new(&self.slice[p1..p2])
    }
}
