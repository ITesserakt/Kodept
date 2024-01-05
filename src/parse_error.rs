use codespan_reporting::diagnostic::{Diagnostic, Label};
use extend::ext;
use nom_supreme::error::{BaseErrorKind, Expectation, StackContext};
use nom_supreme::error::GenericErrorTree::{Alt, Base, Stack};

use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_parse::lexer::Token;
use kodept_parse::ParseError;
use kodept_parse::parser::error::TokenVerificationError;
use kodept_parse::token_stream::TokenStream;

#[ext]
impl<'s> TokenStream<'s> {
    fn location(&self) -> Option<CodePoint> {
        self.iter().next().map(|it| it.span.location())
    }

    fn as_labels(&self) -> Vec<Label<()>> {
        self.location()
            .map_or(vec![], |it| vec![Label::primary((), it.as_range())])
    }
}

fn add_notes(
    diagnostic: Diagnostic<()>,
    context: Vec<(TokenStream, StackContext<&str>)>,
) -> Diagnostic<()> {
    diagnostic.with_notes(
        context
            .into_iter()
            .filter_map(|it| match it {
                (_, StackContext::Context(c)) => Some(format!("at `{}`", c)),
                _ => None,
            })
            .collect(),
    )
}

fn base_to_report(
    needle: TokenStream,
    kind: BaseErrorKind<&str, TokenVerificationError>,
    context: Vec<(TokenStream, StackContext<&str>)>,
) -> Vec<Diagnostic<()>> {
    match kind {
        BaseErrorKind::Expected(Expectation::Something) => vec![Diagnostic::error()
            .with_code("SE001")
            .with_message("Expected something, got EOF")],
        BaseErrorKind::Expected(expectation) => {
            vec![Diagnostic::error()
                .with_code("SE003")
                .with_message(format!(
                    "Expected `{}`, got `{:?}`",
                    expectation,
                    needle.iter().next().map_or(Token::Unknown, |it| it.token)
                ))
                .with_labels(needle.as_labels())]
        }
        BaseErrorKind::Kind(kind) => {
            vec![Diagnostic::error()
                .with_code("SE004")
                .with_message(format!(
                    "Expected `{}`, got `{:?}`",
                    kind.description(),
                    needle.iter().next().map_or(Token::Unknown, |it| it.token)
                ))
                .with_labels(needle.as_labels())]
        }
        BaseErrorKind::External(x) => vec![Diagnostic::error()
            .with_code("SE002")
            .with_message(format!(
                "Expected `{}`, got `{:?}`",
                x.expected,
                needle.iter().next().map_or(Token::Unknown, |it| it.token)
            ))
            .with_labels(needle.as_labels())],
    }
    .into_iter()
    .map(|it| add_notes(it, context.clone()))
    .collect()
}

pub trait Reportable {
    fn to_diagnostics(self) -> Vec<Diagnostic<()>>;
}

fn to_diagnostics_with_context(
    error: ParseError,
    context: Vec<(TokenStream, StackContext<&str>)>,
) -> Vec<Diagnostic<()>> {
    match error {
        Base { location, kind } => base_to_report(location, kind, context),
        Stack { base, contexts } => to_diagnostics_with_context(*base, contexts),
        Alt(vec) => vec.into_iter().flat_map(|it| it.to_diagnostics()).collect(),
    }
}

impl<'s> Reportable for ParseError<'s> {
    fn to_diagnostics(self) -> Vec<Diagnostic<()>> {
        to_diagnostics_with_context(self, vec![])
    }
}
