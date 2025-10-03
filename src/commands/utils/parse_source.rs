use crate::cli::configs::{LexerImpl, ParserImpl, ParsingConfig};
use kodept::report::Reports;
use kodept::source::collection::SourceView;
use kodept_frontend::prelude::ExtractReports;
use kodept_frontend::Execution;
use kodept_parse::common::{ErrorAdapter, RLTProducer};
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_parse::token_stream::PackedTokenStream;
use kodept_parse::tokenizer::{EagerTokenizer, Tok, TokCtor};
use kodept_report::message::{Diagnostic, Severity};
use kodept_report::traits::{IntoSpannedReportMessage, MessageBehaviour};
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;
use std::fmt::{Display, Write};

struct Wrapper<T>(T);

impl<A: Display> IntoSpannedReportMessage for Wrapper<ParseError<A>> {
    type Message = Diagnostic;

    fn behaviour(&self) -> kodept_report::prelude::MessageBehaviour {
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

pub fn get_rlt(config: &ParsingConfig, source: &SourceView, reports: &Reports) -> Execution<RLT> {
    let lexing_backend = config.get_lexing_backend(source.contents());
    let input = source.contents();

    let tokens = match lexing_backend {
        LexerImpl::Peg(x) => EagerTokenizer::new(input, x)
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0)),
        LexerImpl::ASCII(x) => EagerTokenizer::new(input, x)
            .try_into_vec()
            .map_err(|e| match e { })
    }
    .map_err(|e: ParseErrors<&str>| e.into_iter().map(Wrapper))
    .extract_reports(*source.id, reports)?;
    let stream = PackedTokenStream::new(&tokens);

    let parsing_backend = config.get_parsing_backend();
    match parsing_backend {
        ParserImpl::Peg(x) => x.parse_stream(&stream).map_err(|e| e.adapt(stream, 0)),
        #[cfg(feature = "nom")]
        ParserImpl::Nom(x) => x.parse_stream(&stream).map_err(|e| e.adapt(stream, 0)),
    }
    .map_err(|e| e.into_iter().map(Wrapper))
    .extract_reports(*source.id, reports)
}

fn expected_to_string(mut expected: Vec<Cow<'static, str>>) -> Cow<'static, str> {
    let Some(last_expected) = expected.pop() else {
        return Cow::Borrowed("");
    };

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
