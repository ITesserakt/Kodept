use crate::configs::{Lexer, Parser};
use crate::source::collection::SourceView;
use crate::utils::ReportSystemEx;
use bevy_ecs::prelude::*;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_frontend::engine::{Phase, PhaseEngine};
use kodept_frontend::Either;
use kodept_parse::common::{ErrorAdapter, RLTProducer};
use kodept_parse::error::ParseError;
use kodept_parse::lexer::{ASCIILexer, PackedToken, PegLexer};
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::PackedTokenStream;
use kodept_parse::tokenizer::{EagerTokenizer, Tok, TokCtor};
use kodept_report::prelude::*;
use std::borrow::Cow;
use std::fmt::Display;

#[derive(Debug, SystemSet, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct ParseSourcePhaseLabel;

pub struct ParseSourcePhase;

impl Phase for ParseSourcePhase {
    type Set = ParseSourcePhaseLabel;

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(system.extract_reports())
    }
}

fn system(
    source: Res<SourceView>,
    lexer: Res<Lexer>,
    parser: Res<Parser>,
    mut commands: Commands,
) -> Result<
    (),
    Either<
        impl Iterator<Item = Wrapper<ParseError<String>>> + use<>,
        impl Iterator<Item = Wrapper<ParseError<PackedToken>>> + use<>,
    >,
> {
    let input = source.contents();

    let tokens = match &*lexer {
        Lexer::Peg => EagerTokenizer::new(input, PegLexer::<false>::new())
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0))
            .map_err(|e| e.into_iter().map(Wrapper))
            .map_err(Either::Left)?,
        Lexer::PegWithTracing => EagerTokenizer::new(input, PegLexer::<true>::new())
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0))
            .map_err(|e| e.into_iter().map(Wrapper))
            .map_err(Either::Left)?,
        Lexer::Ascii => match EagerTokenizer::new(input, ASCIILexer::new()).try_into_vec() {
            Ok(x) => x,
            Err(e) => match e {},
        },
    };

    let stream = PackedTokenStream::new(&tokens);

    let rlt = match &*parser {
        Parser::Peg => PegParser::new()
            .parse_stream(&stream)
            .map_err(|e| e.adapt(stream, 0))
            .map_err(|e| e.into_iter().map(Wrapper))
            .map_err(Either::Right)?,
    };

    commands.insert_resource(SyntaxResolver::build(rlt));

    Ok(())
}

struct Wrapper<T>(T);

impl<A: Display> IntoSpannedReportMessage for Wrapper<ParseError<A>> {
    type Message = Diagnostic;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Error while parsing")
    }

    fn into_message(self) -> Self::Message {
        let (expected, actual, location, hints) = match self.0 {
            ParseError::ExpectedInstead {
                expected,
                actual,
                location,
                hints,
            } => (expected, Some(actual), location, hints),
            ParseError::ExpectedNotEOF {
                expected,
                location,
                hints,
            } => (expected, None, location, hints),
        };

        let diagnostic = if expected.is_empty() {
            let actual = actual
                .map(|it| Cow::Owned(it.to_string()))
                .unwrap_or(Cow::Borrowed("EOF"));

            Diagnostic::new(Severity::Error)
                .with_message(format!("Unexpected {actual}"))
                .with_primary_label("here", location.in_code)
        } else if let Some(actual) = actual {
            let exp_msg = expected_to_string(expected);

            Diagnostic::new(Severity::Error)
                .with_message(format!("Expected {exp_msg}, got {actual}"))
                .with_primary_label("here", location.in_code)
        } else {
            let exp_msg = expected_to_string(expected);

            Diagnostic::new(Severity::Error)
                .with_message(format!("Expected {exp_msg} after, got EOF"))
                .with_primary_label("here", location.in_code)
        };

        hints
            .into_iter()
            .fold(diagnostic, |acc, next| acc.with_note(next))
    }
}

fn expected_to_string(mut expected: Vec<Cow<'static, str>>) -> Cow<'static, str> {
    let Some(last_expected) = expected.pop() else {
        return Cow::Borrowed("");
    };

    use std::fmt::Write;
    if let Some(last) = expected.pop() {
        let mut result = String::new();
        for item in expected {
            _ = write!(result, "{item}, ");
        }
        _ = write!(result, "{last} or {last_expected}");
        result.into()
    } else {
        last_expected
    }
}
