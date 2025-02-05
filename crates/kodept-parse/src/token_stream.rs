use crate::token_match::PackedTokenMatch;
use derive_more::Constructor;
use kodept_core::code_point::CodePoint;
use kodept_core::static_assert_size;
use kodept_core::structure::Located;
use std::fmt::Debug;
use std::ops::{Deref, Index, Range, RangeBounds};

#[derive(Clone, Debug, PartialEq, Copy, Constructor)]
pub struct PackedTokenStream<'t> {
    slice: &'t [PackedTokenMatch],
}

static_assert_size!(PackedTokenStream<'static>, 16);

impl<'t> PackedTokenStream<'t> {
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    pub fn sub_stream<B: RangeBounds<usize>>(&self, range: B) -> PackedTokenStream<'t>
    where 
        [PackedTokenMatch]: Index<B, Output = [PackedTokenMatch]>,
    {
        Self::new(&self.slice[range])
    }

    pub fn into_single(self) -> PackedTokenMatch {
        match self.slice {
            [x] => *x,
            _ => unreachable!("Token stream with 1 element can be coerced to match"),
        }
    }

    /// Original implementation: subslice_range from std lib
    pub fn sub_stream_range(&self, suffix: PackedTokenStream) -> Option<Range<usize>> {
        let self_start = self.slice.as_ptr() as usize;
        let subslice_start = suffix.slice.as_ptr() as usize;

        let byte_start = subslice_start.wrapping_sub(self_start);

        if byte_start % size_of::<PackedTokenMatch>() != 0 {
            return None;
        }

        let start = byte_start / size_of::<PackedTokenMatch>();
        let end = start.wrapping_add(suffix.len());

        if start <= self.len() && end <= self.len() {
            Some(start..end)
        } else {
            None
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

#[cfg(test)]
mod tests {
    use crate::lexer::PackedToken;
    use crate::token_match::PackedTokenMatch;
    use crate::token_stream::PackedTokenStream;
    use kodept_core::code_point::CodePoint;

    #[test]
    fn test_sub_streams() {
        let storage = &[
            PackedTokenMatch::new(
                PackedToken::Abstract,
                CodePoint::new(1, 0),
            ),
            PackedTokenMatch::new(
                PackedToken::With,
                CodePoint::new(1, 1),
            )
        ];
        let stream = PackedTokenStream::new(storage);
        let suffix = stream.sub_stream(1..);
        let empty_suffix = stream.sub_stream(2..);
        
        assert_eq!(stream.sub_stream_range(suffix), Some(1..2));
        assert_eq!(stream.sub_stream_range(empty_suffix), Some(2..2));
    }
}
