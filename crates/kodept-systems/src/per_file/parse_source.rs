use crate::configs::{Lexer, Parser};
use crate::utils::LogSystemEx;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::schedule::SystemSet;
use kodept_ecs::system::{Commands, Res};
use kodept_frontend::engine::reporter::Reporter;
use kodept_frontend::engine::{Phase, PhaseEngine};
use kodept_frontend::prelude::SourceView;
use kodept_parse::common::{ErrorAdapter, RLTProducer};
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_parse::lexer::{ASCIILexer, PegLexer};
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::{EagerTokenizer, Tok, TokCtor};
use kodept_report::prelude::*;
use std::borrow::Cow;
use std::fmt::{Debug, Display};

#[derive(Debug, SystemSet, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct ParseSourcePhaseLabel;

pub struct ParseSourcePhase;

impl Phase for ParseSourcePhase {
    type Set = ParseSourcePhaseLabel;

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(system.trace_completion())
    }
}

fn system(
    source: Res<SourceView>,
    lexer: Res<Lexer>,
    parser: Res<Parser>,
    mut commands: Commands,
    mut reporter: Reporter,
) {
    let input = source.contents();

    let tokens = match &*lexer {
        Lexer::Peg => EagerTokenizer::new(input, PegLexer::<false>::new())
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0))
            .map_err(|e: ParseErrors<&str>| e.into_iter().map(Wrapper)),
        Lexer::PegWithTracing => EagerTokenizer::new(input, PegLexer::<true>::new())
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0))
            .map_err(|e| e.into_iter().map(Wrapper)),
        Lexer::Ascii => match EagerTokenizer::new(input, ASCIILexer::new()).try_into_vec() {
            Ok(x) => Ok(x),
            Err(e) => match e {},
        },
    };
    let tokens = match tokens {
        Ok(x) => x,
        Err(errors) => {
            errors.for_each(|it| reporter.report(it));
            return;
        }
    };

    let stream = TokenStream::new(&tokens);

    let rlt = match &*parser {
        Parser::Peg => PegParser::new()
            .parse_stream(&stream)
            .map_err(|e| e.adapt(stream, 0))
            .map_err(|e| e.into_iter().map(Wrapper)),
    };
    let rlt = match rlt {
        Ok(x) => x,
        Err(errors) => {
            errors.for_each(|it| reporter.report(it));
            return;
        }
    };

    commands.insert_resource(SyntaxResolver::build(rlt));
}

struct Wrapper<T>(T);

impl<A: Display> IntoMessage for Wrapper<ParseError<A>> {
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
