//! Example demonstrating the usage of kodept-diagnostic-macros
#![allow(dead_code)]

use std::borrow::Cow;

use kodept_core::code_point::{CodePoint, Span};
use kodept_report::{FileId, report::Report};
use kodept_report_macros::Report;

// Simple diagnostic with primary and secondary labels
#[derive(Report, Debug)]
#[severity("Error")]
#[note("Remove or rename previous symbol")]
struct DuplicatedSymbolError {
    #[primary_label("symbol already defined")]
    current_def: Span,
    #[secondary_label("previous declaration")]
    previous_def: Span,
}

// Diagnostic with multiple notes and labels
#[derive(Report, Debug)]
#[severity("Error")]
#[note("Unresolved reference: {}", self.ref_name)]
#[note("{}", self.suggestion)]
struct UnresolvedReferenceError {
    #[primary_label("cannot resolve reference")]
    ref_span: Span,
    ref_name: String,
    suggestion: String,
    #[secondary_label("available symbols")]
    scope_span: Span,
}

// Diagnostic with custom message fields
#[derive(Report, Debug)]
#[severity("Error")]
#[code(1234)]
struct TypeMismatchError {
    #[primary_label("expected type")]
    expected_span: Span,
    #[secondary_label("found type")]
    found_span: Span,
    expected_type: String,
    found_type: String,
    context: String,
}

// Diagnostic with optional fields
#[derive(Report, Debug)]
#[severity("Error")]
#[message("{message}")]
#[note("{}", self.hint_text)]
struct InvalidSyntaxError {
    #[primary_label("syntax error")]
    error_span: Span,
    message: String,
    #[secondary_label("hint")]
    hint_span: Span,
    hint_text: Cow<'static, str>,
}

fn main() {
    let span1 = Span::from(CodePoint::new(15, 20));
    let span2 = Span::from(CodePoint::new(20, 25));
    let file_id = FileId::generate();

    let duplicated_error = DuplicatedSymbolError {
        current_def: span1,
        previous_def: span2,
    };

    let unresolved_error = UnresolvedReferenceError {
        ref_span: span1,
        ref_name: "undefined_var".to_string(),
        suggestion: "did you mean 'defined_var'?".to_string(),
        scope_span: span2,
    };

    let type_error = TypeMismatchError {
        expected_span: span1,
        found_span: span2,
        expected_type: "i32".to_string(),
        found_type: "String".to_string(),
        context: "in function call".to_string(),
    };

    let syntax_error = InvalidSyntaxError {
        error_span: span1,
        message: "unexpected token".to_string(),
        hint_span: span2,
        hint_text: "missing semicolon".into(),
    };

    println!("{:?}", Report::from_message(file_id, duplicated_error));
    println!();
    println!("{:?}", Report::from_message(file_id, unresolved_error));
    println!();
    println!("{:?}", Report::from_message(file_id, type_error));
    println!();
    println!("{:?}", Report::from_message(file_id, syntax_error));
}
