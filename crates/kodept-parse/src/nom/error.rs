use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};
use crate::lexer::PackedToken;
use crate::nom::TokenVerificationError;
use crate::token_stream::PackedTokenStream;
use derive_more::Constructor;
use itertools::Itertools;
use kodept_core::code_point::CodePoint;
use nom::Offset;
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation, GenericErrorTree, StackContext};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::iter::repeat;

trait ExpectedError {
    fn expected(&self) -> Cow<'static, str>;
}

#[derive(Debug, Clone, PartialEq)]
struct Context<O> {
    prefix: O,
    item_name: StackContext<&'static str>,
}

#[derive(Debug, Constructor)]
struct BaseError<'s, O, E> {
    location: O,
    kind: BaseErrorKind<&'s str, E>,
    context: Vec<Context<O>>,
}

impl<'s, O, E> BaseError<'s, O, E>
where
    E: ExpectedError,
{
    fn into_expected(self) -> Cow<'static, str> {
        match self.kind {
            BaseErrorKind::Expected(Expectation::Something) => Cow::Borrowed("anything"),
            BaseErrorKind::Expected(expectation) => Cow::Owned(expectation.to_string()),
            BaseErrorKind::Kind(kind) => Cow::Owned(kind.description().to_string()),
            BaseErrorKind::External(ext) => ext.expected(),
        }
    }
}

impl ExpectedError for TokenVerificationError {
    fn expected(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.expected)
    }
}

impl<T: ?Sized + ToString> ExpectedError for Box<T> {
    fn expected(&self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}

impl<O> From<(O, StackContext<&'static str>)> for Context<O> {
    fn from(value: (O, StackContext<&'static str>)) -> Self {
        Self {
            prefix: value.0,
            item_name: value.1,
        }
    }
}

fn flatten_error_tree<'a, I, E>(
    tree: GenericErrorTree<I, &'a str, &'static str, E>,
) -> Vec<BaseError<'a, I, E>>
where
    I: Clone,
{
    let mut current_errors = VecDeque::from([(tree, vec![])]);
    let mut base_errors = vec![];

    loop {
        match current_errors.pop_front() {
            None => break,
            Some((GenericErrorTree::Base { location, kind }, context)) => {
                base_errors.push(BaseError::new(location, kind, context))
            }
            Some((GenericErrorTree::Stack { base, contexts }, context)) => current_errors
                .push_back((
                    *base,
                    context
                        .into_iter()
                        .chain(contexts.into_iter().map_into())
                        .collect(),
                )),
            Some((GenericErrorTree::Alt(es), context)) => {
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
    I: Copy + PartialEq + Offset,
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

impl<'a, A> ErrorAdapter<A, &'a str> for ErrorTree<&'a str>
where
    &'a str: Original<A>,
    A: From<&'a str>,
{
    fn adapt(self, original_input: &'a str, _: usize) -> ParseErrors<A> {
        let base_errors = flatten_error_tree(self);
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

impl<'t> ErrorAdapter<PackedToken, PackedTokenStream<'t>> for super::parser::ParseError<'t> {
    fn adapt(self, original_input: PackedTokenStream<'t>, _: usize) -> ParseErrors<PackedToken> {
        let base_errors = flatten_error_tree(self);
        convert_base_errors(original_input, base_errors, |original, suffix| {
            let position = original.offset(&suffix);
            match suffix.first() {
                None => (
                    None,
                    ErrorLocation::new(
                        position,
                        original
                            .last()
                            .map(|it| it.point)
                            .unwrap_or(CodePoint::single_point(0)),
                    ),
                ),
                Some(first) => (Some(first.token), ErrorLocation::new(position, first.point)),
            }
        })
    }
}
