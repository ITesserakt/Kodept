use kodept::report::Reports;
use kodept::source::collection::SourceView;
use kodept_ast::syntax_tree::prelude::{SourceCode, AST};
use kodept_ast_nodes::file::FileDecl;
use kodept_ast_nodes::Error;
use kodept_core::file_name::FileDescriptor;
use kodept_core::structure::span::CodeHolder;
use kodept_frontend::prelude::ExtractReports;
use kodept_frontend::Execution;
use kodept_report::{
    prelude::{Diagnostic, Severity},
    traits::IntoSpannedReportMessage,
};
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;

struct Wrapper(Error);

impl IntoSpannedReportMessage for Wrapper {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        let diagnostic = Diagnostic::new(Severity::Bug);
        match self.0 {
            Error::NoQuotesInLiteral(point) => diagnostic
                .with_message("String or char literals must contain quotes")
                .with_primary_label("no quotes", point),
            Error::WrongLiteralLength(point, len) => diagnostic
                .with_message(format!("Literal must have length at least `{}`", len))
                .with_primary_label("wrong length", point),
            Error::CannotParseFloat(point, e) => diagnostic
                .with_message(format!("Cannot parse floating literal: {}", e))
                .with_primary_label("cannot parse floating literal", point),
            Error::CannotParseInt(point, e) => diagnostic
                .with_message(format!("Cannot parse integer literal: {}", e))
                .with_primary_label("cannot parse integer literal", point),
        }
    }
}

#[cfg(feature = "interning")]
pub fn build_ast(source: &SourceView, rlt: RLT, reports: &Reports) -> Execution<AST> {
    let code_holder =
        kodept_interning::InterningCodeHolder::new(&**source).map(|it| Cow::Borrowed(it.0));
    let ast = AST::recursively_build::<FileDecl>(
        rlt,
        SourceCode::new(
            code_holder,
            FileDescriptor::new(source.path().clone(), *source.id),
        ),
    )
    .map_err(Wrapper)
    .extract_reports(*source.id, reports)?;
    let metrics = kodept_interning::metrics::InterningMetrics::gather();
    let (saved_value, saved_suffix) = metrics.memory_save();
    tracing::debug!(
        ?metrics,
        "Interning saved {:.2}{}",
        saved_value,
        saved_suffix
    );
    Execution::Continue(ast)
}

#[cfg(not(feature = "interning"))]
pub fn build_ast(source: &SourceView, rlt: RLT, reports: &Reports) -> Execution<AST> {
    let code_holder = source.map(|it| Cow::Owned(it.to_string()));
    AST::recursively_build::<FileDecl>(
        rlt,
        SourceCode::new(
            code_holder,
            FileDescriptor::new(source.path().clone(), *source.id),
        ),
    )
    .map_err(Wrapper)
    .extract_reports(*source.id, reports)
}
