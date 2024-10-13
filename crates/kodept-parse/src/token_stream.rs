use crate::token_match::PackedTokenMatch;
use kodept_core::code_point::CodePoint;
use kodept_core::static_assert_size;
use kodept_core::structure::Located;
use nom::{InputIter, InputLength, InputTake, Needed, Offset, Slice, UnspecializedInput};
use std::fmt::Debug;
use std::ops::{Deref, Range, RangeTo};

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct PackedTokenStream<'t> {
    slice: &'t [PackedTokenMatch],
}

static_assert_size!(PackedTokenStream<'static>, 16);

impl<'t> PackedTokenStream<'t> {
    pub fn new(slice: &'t [PackedTokenMatch]) -> Self {
        Self {
            slice,
        }
    }

    pub fn len(&self) -> usize {
        self.slice.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    pub fn sub_stream(&self, range: Range<usize>) -> PackedTokenStream {
        Self::new(&self.slice[range])
    }

    pub fn into_single(self) -> PackedTokenMatch {
        match self.slice {
            [x] => *x,
            _ => unreachable!("Token stream with 1 element can be coerced to match"),
        }
    }
}

impl Located for PackedTokenStream<'_> {
    fn location(&self) -> CodePoint {
        let len = self.slice.iter().map(|it| it.point.length).sum();

        match self.slice {
            [x, ..] => CodePoint::new(len, x.point.offset),
            [] => CodePoint::new(0, 0),
        }
    }
}

impl<'t> Deref for PackedTokenStream<'t> {
    type Target = &'t [PackedTokenMatch];

    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<'t> InputIter for PackedTokenStream<'t> {
    type Item = PackedTokenMatch;
    type Iter = impl Iterator<Item = (usize, PackedTokenMatch)>;
    type IterElem = impl Iterator<Item = PackedTokenMatch>;

    fn iter_indices(&self) -> Self::Iter {
        self.slice.iter().enumerate().map(|it| (it.0, *it.1))
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.slice.iter().cloned()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.slice.iter().position(|&it| predicate(it))
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.slice.len()))
        }
    }
}

impl<'t> InputTake for PackedTokenStream<'t> {
    fn take(&self, count: usize) -> Self {
        Self {
            slice: &self[..count],
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (first, second) = self.slice.split_at(count);
        (
            Self {
                slice: second,
                ..*self
            },
            Self {
                slice: first,
                ..*self
            },
        )
    }
}

impl<'t> InputLength for PackedTokenStream<'t> {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl UnspecializedInput for PackedTokenStream<'_> {}

impl Slice<RangeTo<usize>> for PackedTokenStream<'_> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        Self {
            slice: &self[range],
            ..*self
        }
    }
}

impl Offset for PackedTokenStream<'_> {
    fn offset(&self, second: &Self) -> usize {
        let fst = self.as_ptr();
        let snd = second.as_ptr();

        snd as usize - fst as usize
    }
}
