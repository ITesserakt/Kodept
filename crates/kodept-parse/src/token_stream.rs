use crate::lexer::traits::ToRepresentation;
use crate::lexer::{Identifier, Ignore, Literal, Token};
use crate::token_match::{PackedTokenMatch, TokenMatch};
use kodept_core::code_point::CodePoint;
use kodept_core::static_assert_size;
use kodept_core::structure::Located;
use nom::{InputIter, InputLength, InputTake, Needed, Offset, Slice, UnspecializedInput};
use nom_supreme::final_parser::RecreateContext;
use std::fmt::{Debug, Display, Formatter};
use std::iter::FusedIterator;
use std::ops::{Deref, Range, RangeTo};

#[deprecated]
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct TokenStream<'t, 's> {
    pub(crate) slice: &'s [TokenMatch<'t>],
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct PackedTokenStream<'t> {
    slice: &'t [PackedTokenMatch],
    pub(crate) original_input: &'t str,
}

static_assert_size!(TokenStream<'static, 'static>, 16);
static_assert_size!(PackedTokenStream<'static>, 32);

impl<'t> PackedTokenStream<'t> {
    pub fn new(slice: &'t [PackedTokenMatch], original_input: &'t str) -> Self {
        Self {
            slice,
            original_input,
        }
    }

    pub fn len(&self) -> usize {
        self.slice.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    pub fn sub_stream(&self, range: Range<usize>) -> Self {
        Self::new(&self.slice[range], self.original_input)
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
        let len = self.slice.into_iter().map(|it| it.point.length).sum();

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

impl Display for PackedTokenStream<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let location = self.location();
        write!(f, "{}", &self.original_input[location.as_range()])
    }
}

impl<'t, 's> TokenStream<'t, 's> {
    #[must_use]
    #[inline]
    pub fn iter(&self) -> TokenStreamIterator {
        TokenStreamIterator {
            stream: *self,
            position: 0,
        }
    }

    #[must_use]
    pub const fn new(slice: &'s [TokenMatch<'t>]) -> Self {
        Self { slice }
    }

    #[must_use]
    pub fn into_token_match(self) -> Option<TokenMatch<'t>> {
        match self.slice {
            [x] => Some(*x),
            _ => None,
        }
    }

    pub fn token_iter(&self) -> impl Iterator<Item = &Token> {
        self.slice.iter().map(|it| &it.token)
    }

    pub fn len(&self) -> usize {
        self.slice.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    pub fn iter_indices(&self) -> TokenStreamIndices {
        TokenStreamIndices {
            stream: *self,
            position: 0,
        }
    }

    pub fn iter_elements(&self) -> TokenStreamIterator {
        TokenStreamIterator {
            stream: *self,
            position: 0,
        }
    }
}

pub struct TokenStreamIterator<'t, 's> {
    stream: TokenStream<'t, 's>,
    position: usize,
}

pub struct TokenStreamIndices<'t, 's> {
    stream: TokenStream<'t, 's>,
    position: usize,
}

impl<'t, 's> Iterator for TokenStreamIterator<'t, 's> {
    type Item = TokenMatch<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position > self.stream.slice.len() {
            return None;
        }

        let token = self.stream.slice.get(self.position);
        self.position += 1;
        token.cloned()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.slice.iter().size_hint()
    }
}

impl FusedIterator for TokenStreamIterator<'_, '_> {}

impl<'t, 's> Iterator for TokenStreamIndices<'t, 's> {
    type Item = (usize, TokenMatch<'t>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position > self.stream.slice.len() {
            return None;
        }

        let token = self.stream.slice.get(self.position);
        self.position += 1;
        token.cloned().map(|it| (self.position - 1, it))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.slice.iter().size_hint()
    }
}

impl FusedIterator for TokenStreamIndices<'_, '_> {}

impl<'t, 's> InputIter for TokenStream<'t, 's> {
    type Item = TokenMatch<'t>;
    type Iter = TokenStreamIndices<'t, 's>;
    type IterElem = TokenStreamIterator<'t, 's>;

    fn iter_indices(&self) -> Self::Iter {
        TokenStreamIndices {
            stream: *self,
            position: 0,
        }
    }

    fn iter_elements(&self) -> Self::IterElem {
        TokenStreamIterator {
            stream: *self,
            position: 0,
        }
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.slice.iter().position(|it| predicate(*it))
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.slice.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.slice.len()))
        }
    }
}

impl UnspecializedInput for TokenStream<'_, '_> {}

impl InputLength for TokenStream<'_, '_> {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl InputTake for TokenStream<'_, '_> {
    fn take(&self, count: usize) -> Self {
        TokenStream {
            slice: &self.slice[..count],
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (first, second) = self.slice.split_at(count);
        (Self { slice: second }, Self { slice: first })
    }
}

impl<'t, 's> RecreateContext<TokenStream<'t, 's>> for CodePoint {
    fn recreate_context(original_input: TokenStream<'t, 's>, tail: TokenStream<'t, 's>) -> Self {
        if let Some((head, _)) = tail.slice.split_first() {
            head.span.point
        } else if let Some((last, _)) = original_input.slice.split_last() {
            last.span.point
        } else {
            CodePoint::single_point(0)
        }
    }
}

impl Display for TokenStream<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Some(offset) = self.slice.first().map(|it| it.span.point.offset) else {
            return write!(f, "");
        };
        let size = self.slice.last().expect("Unreachable").span.point.offset - offset + 1;
        // FIXME: wrong size
        let mut output = " ".repeat((size * 4) as usize);
        for token_match in self.iter() {
            let index = (token_match.span.point.offset - offset) as usize;
            let len = token_match.span.point.length as usize;

            output.replace_range(
                index..index + len,
                match token_match.token {
                    Token::Ignore(Ignore::Newline) => "\n",
                    Token::Ignore(Ignore::Whitespace) => "",
                    Token::Ignore(Ignore::MultilineComment(x)) => x,
                    Token::Ignore(Ignore::Comment(x)) => x,
                    Token::Keyword(x) => x.representation(),
                    Token::Symbol(x) => x.representation(),
                    Token::Identifier(x) => match x {
                        Identifier::Identifier(x) => x,
                        Identifier::Type(x) => x,
                    },
                    Token::Literal(x) => match x {
                        Literal::Binary(x) => x,
                        Literal::Octal(x) => x,
                        Literal::Hex(x) => x,
                        Literal::Floating(x) => x,
                        Literal::Char(x) => x,
                        Literal::String(x) => x,
                    },
                    Token::Operator(x) => x.representation(),
                    Token::Unknown => "?",
                },
            )
        }
        write!(f, "{}", output)
    }
}

impl Located for TokenStream<'_, '_> {
    fn location(&self) -> CodePoint {
        let len = self.slice.iter().map(|it| it.span.point.length).sum();

        match self.slice {
            [x, ..] => CodePoint::new(len, x.span.point.offset),
            [] => CodePoint::new(0, 0),
        }
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
            original_input: self.original_input,
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
