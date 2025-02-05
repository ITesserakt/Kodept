use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};
use crate::lexer::PackedToken;
use crate::nom::parser::{PError, PErrorContext, PErrorKind};
use crate::nom::{TError, TokenVerificationError, VerboseErrorKind};
use crate::token_stream::PackedTokenStream;
use derive_more::Constructor;
use itertools::Itertools;
use kodept_core::code_point::CodePoint;
use nom::Offset;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::iter::repeat;

pub(super) trait ExpectedError {
    fn expected(&self) -> Cow<'static, str>;
}

#[derive(Debug, Clone, PartialEq)]
struct Context<O> {
    prefix: O,
    item_name: PErrorContext,
}

#[derive(Debug, Constructor)]
struct BaseError<O, E> {
    location: O,
    kind: PErrorKind<E>,
    context: Vec<Context<O>>,
}

impl<O, E> BaseError<O, E>
where
    E: ExpectedError,
{
    fn into_expected(self) -> Cow<'static, str> {
        match self.kind {
            PErrorKind::Char(value) => Cow::Owned(value.to_string()),
            PErrorKind::Nom(kind) => Cow::Owned(kind.description().to_string()),
            PErrorKind::External(ext) => ext.expected(),
        }
    }
}

impl ExpectedError for TokenVerificationError {
    fn expected(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.expected)
    }
}

impl ExpectedError for () {
    fn expected(&self) -> Cow<'static, str> {
        Cow::Borrowed("anything")
    }
}

impl<T: ?Sized + ToString> ExpectedError for Box<T> {
    fn expected(&self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}

impl<O> From<(O, PErrorContext)> for Context<O> {
    fn from(value: (O, PErrorContext)) -> Self {
        Self {
            prefix: value.0,
            item_name: value.1,
        }
    }
}

fn flatten_error_tree<E>(tree: PError<E>) -> Vec<BaseError<PackedTokenStream, E>> {
    let mut current_errors = VecDeque::from([(tree, vec![])]);
    let mut base_errors = vec![];

    loop {
        match current_errors.pop_front() {
            None => break,
            Some((
                PError::Base {
                    location: input,
                    kind,
                },
                context,
            )) => base_errors.push(BaseError::new(input, kind, context)),
            Some((PError::Stack { base, contexts }, context)) => current_errors.push_back((
                *base,
                context
                    .into_iter()
                    .chain(contexts.into_iter().map_into())
                    .collect(),
            )),
            Some((PError::Alt(es), context)) => {
                current_errors.extend(es.into_iter().zip(repeat(context)))
            }
        }
    }

    base_errors
}

fn convert_base_errors<A, I, E>(
    original_input: I,
    errors: Vec<BaseError<I, E>>,
    mut f: impl FnMut(I, I) -> (Option<A>, ErrorLocation),
) -> ParseErrors<A>
where
    I: Copy + PartialEq,
    E: ExpectedError,
{
    let parse_errors = errors
        .into_iter()
        .chunk_by(|it| (it.location, it.context.clone()))
        .into_iter()
        .map(|((suffix, ctx), group)| {
            let (actual, location) = f(original_input, suffix);
            let expected = group.map(|it| it.into_expected()).collect();

            let error = match actual {
                None => ParseError::unexpected_eof(expected, location),
                Some(actual) => ParseError::expected(expected, actual, location),
            };
            ctx.into_iter().fold(error, |acc, next| {
                acc.with_hints(next.item_name.to_string().into())
            })
        })
        .collect();
    ParseErrors::new(parse_errors)
}

impl<'a, A> ErrorAdapter<A, &'a str> for TError<'a>
where
    &'a str: Original<A>,
    A: From<&'a str>,
{
    fn adapt(self, original_input: &'a str, _: usize) -> ParseErrors<A> {
        let base_errors = self
            .errors
            .into_iter()
            .map(|(location, kind)| {
                let (kind, contexts) = match kind {
                    VerboseErrorKind::Context(x) => (
                        PErrorKind::External(()),
                        vec![Context {
                            prefix: location,
                            item_name: PErrorContext::Context(x),
                        }],
                    ),
                    VerboseErrorKind::Char(value) => (PErrorKind::Char(value), vec![]),
                    VerboseErrorKind::Nom(kind) => (PErrorKind::Nom(kind), vec![]),
                };
                BaseError::new(location, kind, contexts)
            })
            .collect();
        convert_base_errors(original_input, base_errors, |original, suffix| {
            let actual = suffix.get(0..1).map(A::from);
            let suffix_offset = original.offset(suffix);
            (
                actual,
                ErrorLocation::new(suffix_offset, CodePoint::single_point(suffix_offset as u32)),
            )
        })
    }
}

impl<'t, E: ExpectedError> ErrorAdapter<PackedToken, PackedTokenStream<'t>> for PError<'t, E> {
    fn adapt(self, original_input: PackedTokenStream<'t>, _: usize) -> ParseErrors<PackedToken> {
        let base_errors = flatten_error_tree(self);
        convert_base_errors(original_input, base_errors, |original, suffix| {
            let position = original.sub_stream_range(suffix);
            match (position, &*suffix) {
                (Some(range), []) if range.is_empty() => (
                    None,
                    ErrorLocation::new(
                        original.len() - 1,
                        original
                            .last()
                            .map(|it| it.point)
                            .unwrap_or(CodePoint::single_point(0)),
                    ),
                ),
                (Some(range), [first, ..]) => (
                    Some(first.token),
                    ErrorLocation::new(range.start, first.point),
                ),
                _ => unreachable!(),
            }
        })
    }
}
