use crate::cli::configs::{LexerImpl, ParserImpl, ParsingConfig};
use kodept::report::Reports;
use kodept::source::collection::SourceView;
use kodept_parse::common::{ErrorAdapter, RLTProducer};
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_parse::lexer::traits::ToRepresentation;
use kodept_parse::token_stream::PackedTokenStream;
use kodept_parse::tokenizer::{EagerTokenizer, LazyTokenizer, Tok, TokCtor};
use kodept_report::error::report::{Label, Severity};
use kodept_report::error::Diagnostic;
use kodept_report::FileId;
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;
use std::fmt::{Display, Write};
use std::ops::ControlFlow::{Break, Continue};
use kodept_frontend::Execution;

pub fn get_rlt(
    config: &ParsingConfig,
    source: &SourceView,
    reports: &Reports,
) -> Execution<RLT> {
    let lexing_backend = config.get_lexing_backend(source.contents().len());
    let input = source.contents();

    let tokens_result = match lexing_backend {
        LexerImpl::Peg(x) => EagerTokenizer::new(input, x)
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0)),
        LexerImpl::Nom(x) => LazyTokenizer::new(input, x)
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0)),
        LexerImpl::Pest(x) => EagerTokenizer::new(input, x)
            .try_into_vec()
            .map_err(|e| e.adapt(input, 0)),
    };
    let tokens = match tokens_result {
        Ok(x) => x,
        Err(e) => return Break(report_each(*source.id, reports, e)),
    };
    let stream = PackedTokenStream::new(&tokens);

    let parsing_backend = config.get_parsing_backend();
    let rlt_result = match parsing_backend {
        ParserImpl::Peg(x) => x.parse_stream(&stream).map_err(|e| e.adapt(stream, 0)),
        ParserImpl::Nom(x) => x.parse_stream(&stream).map_err(|e| e.adapt(stream, 0)),
    };
    match rlt_result {
        Ok(x) => Continue(x),
        Err(e) => Break(report_each(
            *source.id,
            reports,
            e.map(|t| t.representation()),
        )),
    }
}

fn report_each(file_id: FileId, reports: &Reports, errors: ParseErrors<&str>) {
    for error in errors {
        reports.report(file_id, to_diagnostic(error));
    }
}

fn to_diagnostic<A: Display>(error: ParseError<A>) -> Diagnostic {
    let (expected, actual, location, hints) = match error {
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
            .with_label(Label::primary("here", location.in_code))
    } else if let Some(actual) = actual {
        let exp_msg = expected_to_string(expected);

        Diagnostic::new(Severity::Error)
            .with_message(format!("Expected {exp_msg}, got {actual}"))
            .with_label(Label::primary("here", location.in_code))
    } else {
        let exp_msg = expected_to_string(expected);

        Diagnostic::new(Severity::Error)
            .with_message(format!("Expected {exp_msg} after, got EOF"))
            .with_label(Label::primary("here", location.in_code))
    };

    hints
        .into_iter()
        .fold(diagnostic, |acc, next| acc.with_note(next))
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
