use crate::lexer::PackedToken;
use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use derive_more::Display;
use kodept_core::code_point::CodePoint;
use peg::str::LineCol;
use peg::{Parse, ParseElem, ParseLiteral, ParseSlice, RuleResult};

#[derive(Display, Copy, Clone, Debug)]
#[display("{line}:{col}")]
pub struct Position {
    line: usize,
    col: u32,
    length: u32,
    offset: u32,
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
            col: value.column as u32,
            length: 1,
            offset: value.offset as u32,
        }
    }
}

impl<'t> Parse for PackedTokenStream<'t> {
    type PositionRepr = Position;

    #[inline(always)]
    fn start(&self) -> usize {
        self.first().map_or(0, |it| it.point.offset) as usize
    }

    #[inline(always)]
    fn is_eof(&self, pos: usize) -> bool {
        pos >= self.len()
    }

    #[inline(always)]
    fn position_repr(&self, pos: usize) -> Self::PositionRepr {
        let (before, point) = match self.split_at(pos) {
            (a, [b, ..]) => (a, b.point),
            (a @ [.., last], []) => (a, last.point),
            ([], []) => panic!("Cannot slice empty stream"),
        };
        let line = before
            .iter()
            .filter(|it| matches!(it.token, PackedToken::Newline))
            .count()
            + 1;
        let col = before
            .iter()
            .rev()
            .take_while(|it| !matches!(it.token, PackedToken::Newline))
            .map(|it| it.point.length)
            .sum::<u32>()
            + 1;

        Position {
            line,
            col,
            length: point.length,
            offset: point.offset,
        }
    }
}

impl<'input> ParseElem<'input> for PackedTokenStream<'input> {
    type Element = PackedTokenMatch;

    #[inline(always)]
    fn parse_elem(&'input self, pos: usize) -> RuleResult<Self::Element> {
        let slice = &self[pos..];
        match slice.first() {
            None => RuleResult::Failed,
            Some(x) => RuleResult::Matched(pos + 1, *x),
        }
    }
}

impl<'input> ParseLiteral for PackedTokenStream<'input> {
    #[inline(always)]
    fn parse_string_literal(&self, pos: usize, literal: &str) -> RuleResult<()> {
        let Some(token) = PackedToken::from_name(literal) else {
            unreachable!("Bug in grammar. Any literal used should be convertible to token.")
        };

        match (self.get(pos), token) {
            (Some(a), b) if a.token == b => RuleResult::Matched(pos + 1, ()),
            _ => RuleResult::Failed,
        }
    }
}

impl<'input> ParseSlice<'input> for PackedTokenStream<'input> {
    type Slice = PackedTokenStream<'input>;

    #[inline(always)]
    fn parse_slice(&'input self, p1: usize, p2: usize) -> Self::Slice {
        self.sub_stream(p1..p2)
    }
}
