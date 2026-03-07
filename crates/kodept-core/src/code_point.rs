use std::ops::{Add, Range};

use crate::structure::Located;
use derive_more::{Constructor, Display};

#[derive(Constructor, Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
#[display("...{offset}:{length}")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodePoint {
    pub length: u32,
    pub offset: u32,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
#[display("{start}..{end}", start = offset, end = offset + length)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    pub length: u32,
    pub offset: u32,
}

const _: () = {
    assert!(size_of::<CodePoint>() - 8 == 0);
    assert!(size_of::<Span>() - 8 == 0);
};

impl CodePoint {
    #[must_use]
    #[inline(always)]
    pub const fn single_point(offset: u32) -> Self {
        Self { length: 1, offset }
    }

    #[inline(always)]
    pub const fn as_range(&self) -> Range<usize> {
        let offset = self.offset as usize;
        let length = self.length as usize;
        offset..offset + length
    }
}

impl Span {
    #[inline(always)]
    pub const fn with(self, other: Self) -> Self {
        let [min, max] = if other.offset < self.offset {
            [other, self]
        } else {
            [self, other]
        };

        Self {
            offset: min.offset,
            length: max.offset + max.length - min.offset,
        }
    }

    #[inline(always)]
    pub const fn with_opt(self, other: Option<Self>) -> Self {
        match other {
            None => self,
            Some(other) => self.with(other),
        }
    }

    #[inline(always)]
    pub const fn as_range(self) -> Range<usize> {
        self.offset as usize..(self.offset + self.length) as usize
    }
}

impl Located for CodePoint {
    #[inline(always)]
    fn location(&self) -> CodePoint {
        *self
    }
}

impl Add for Span {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        self.with(rhs)
    }
}

impl Add<Option<Self>> for Span {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Option<Self>) -> Self::Output {
        self.with_opt(rhs)
    }
}

impl Add for CodePoint {
    type Output = Span;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Span::from(self) + Span::from(rhs)
    }
}

impl Add<Span> for CodePoint {
    type Output = Span;

    #[inline(always)]
    fn add(self, rhs: Span) -> Self::Output {
        Span::from(self) + rhs
    }
}

impl Add<CodePoint> for Span {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: CodePoint) -> Self::Output {
        self + Span::from(rhs)
    }
}

impl Add<Option<Self>> for CodePoint {
    type Output = Span;

    #[inline(always)]
    fn add(self, rhs: Option<Self>) -> Self::Output {
        Span::from(self) + rhs.map(Span::from)
    }
}

impl Add<Option<CodePoint>> for Span {
    type Output = Span;

    #[inline(always)]
    fn add(self, rhs: Option<CodePoint>) -> Self::Output {
        self + rhs.map(Span::from)
    }
}

impl Add<Option<Span>> for CodePoint {
    type Output = Span;

    #[inline(always)]
    fn add(self, rhs: Option<Span>) -> Self::Output {
        Span::from(self) + rhs
    }
}

impl From<CodePoint> for Span {
    #[inline(always)]
    fn from(value: CodePoint) -> Self {
        Self {
            length: value.length,
            offset: value.offset,
        }
    }
}

#[cfg(feature = "arbitrary")]
const _: () = {
    use crate::code_point::CodePoint;
    use proptest::prelude::{Arbitrary, Strategy};
    use proptest::strategy::Map;
    use std::ops::Range;

    impl Arbitrary for CodePoint {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            (0..100u32, 0..1000u32).prop_map(|it| CodePoint::new(it.0, it.1))
        }

        type Strategy = Map<(Range<u32>, Range<u32>), fn((u32, u32)) -> CodePoint>;
    }
};
