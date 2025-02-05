use crate::token_match::PackedTokenMatch;
use crate::token_stream::PackedTokenStream;
use nom::{Input, Needed, Offset};
use std::iter::Enumerate;

pub struct PackedTokenStreamIter<'t> {
    slice_iter: std::slice::Iter<'t, PackedTokenMatch>,
}

impl<'t> Input for PackedTokenStream<'t> {
    type Item = PackedTokenMatch;
    type Iter = PackedTokenStreamIter<'t>;
    type IterIndices = Enumerate<PackedTokenStreamIter<'t>>;

    #[inline(always)]
    fn input_len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn take(&self, index: usize) -> Self {
        Self::new(&self[..index])
    }

    #[inline(always)]
    fn take_from(&self, index: usize) -> Self {
        Self::new(&self[index..])
    }

    #[inline(always)]
    fn take_split(&self, index: usize) -> (Self, Self) {
        let (first, second) = self.split_at(index);
        (Self::new(first), Self::new(second))
    }

    #[inline(always)]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.iter().position(|&it| predicate(it))
    }

    #[inline(always)]
    fn iter_elements(&self) -> Self::Iter {
        PackedTokenStreamIter {
            slice_iter: self.iter(),
        }
    }

    #[inline(always)]
    fn iter_indices(&self) -> Self::IterIndices {
        PackedTokenStreamIter {
            slice_iter: self.iter(),
        }
        .enumerate()
    }

    #[inline(always)]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.len()))
        }
    }
}

impl Iterator for PackedTokenStreamIter<'_> {
    type Item = PackedTokenMatch;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.slice_iter.next().copied()
    }
}

impl Offset for PackedTokenStream<'_> {
    #[inline(always)]
    fn offset(&self, second: &Self) -> usize {
        self.sub_stream_range(*second)
            .expect("The second argument must be a suffix of the first one")
            .start
    }
}
