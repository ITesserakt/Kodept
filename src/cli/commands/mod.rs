use crate::cli::commands::execute::Execute;
use crate::cli::commands::graph::Graph;
use crate::cli::commands::inspect::InspectParser;
use crate::cli::traits::CommandWithSources;
use clap::Subcommand;
use itertools::Itertools;
use kodept::report::GlobalReports;
use kodept::source::collection::SourceView;
use kodept_ast::Str;
use kodept_core::structure::span::CodeHolder;
use kodept_frontend::Execution;
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, Label, Severity};
use std::borrow::Cow;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod execute;
mod graph;
mod inspect;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Output AST in .dot format
    Graph(Graph),
    /// Output parsing process files
    InspectParser(InspectParser),
    /// Run type checker
    Execute(Execute),
}

impl Commands {
    pub fn execute(self, output: PathBuf, reports: GlobalReports) -> Execution<()> {
        let sources = Arc::new(match &self {
            Commands::Graph(x) => x.build_sources(&reports),
            Commands::InspectParser(x) => x.build_sources(&reports),
            Commands::Execute(x) => x.build_sources(&reports),
        }?);
        let reports = reports.upgrade(sources.clone());
        let sources = sources.collect();

        match self {
            Commands::Graph(x) => x.exec(sources, &reports, output),
            Commands::InspectParser(x) => x.exec(sources, &reports, output),
            Commands::Execute(x) => x.exec(sources, &reports, output),
        }
    }
}

pub struct ParseDiagnostic(Diagnostic);

impl IntoSpannedReportMessage for ParseDiagnostic {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        self.0
    }
}

fn to_diagnostic<A: Display>(error: ParseError<A>) -> ParseDiagnostic {
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
            .with_label(Label::primary("unexpected token", location.in_code))
    } else if let Some(actual) = actual {
        let exp_msg = expected_to_string(expected);

        Diagnostic::new(Severity::Error)
            .with_message(format!("Expected {exp_msg}, got {actual}"))
            .with_label(Label::primary("unexpected token", location.in_code))
    } else {
        let exp_msg = expected_to_string(expected);

        Diagnostic::new(Severity::Error)
            .with_message(format!("Expected {exp_msg}, got EOF"))
            .with_label(Label::primary("expected more", location.in_code))
    };

    let diag = hints
        .into_iter()
        .fold(diagnostic, |acc, next| acc.with_note(next));
    ParseDiagnostic(diag)
}

fn expected_to_string(mut expected: Vec<Cow<'static, str>>) -> Cow<'static, str> {
    let Some(last_expected) = expected.pop() else {
        return Cow::Borrowed("");
    };

    if expected.is_empty() {
        last_expected
    } else {
        format!("{} or {}", expected.into_iter().join(", "), last_expected).into()
    }
}

fn to_diagnostics<A: Display>(errors: ParseErrors<A>) -> Vec<ParseDiagnostic> {
    errors.into_iter().map(to_diagnostic).collect()
}

fn get_output_file(source: &SourceView, output_path: &Path) -> std::io::Result<File> {
    let name = source.path();
    let path = name.build_file_path().with_extension("kd.dot");
    let filename = path.file_name().unwrap();
    ensure_path_exists(output_path)?;
    File::create(output_path.join(filename))
}

fn ensure_path_exists(path: &Path) -> std::io::Result<()> {
    match create_dir_all(path) {
        Err(e) if e.kind() != ErrorKind::AlreadyExists => Err(e)?,
        _ => Ok(()),
    }
}

#[derive(Debug)]
struct PrintMetricsOnDrop<T>(T);

#[cfg(feature = "interning")]
impl<T> Drop for PrintMetricsOnDrop<T> {
    fn drop(&mut self) {
        use kodept_interning::metrics::InterningMetrics;
        use tracing::debug;
        
        let metrics = InterningMetrics::gather();
        debug!(?metrics);
    }
}

#[cfg(feature = "interning")]
fn get_code_holder(source: &SourceView) -> impl CodeHolder<Str = Str> + '_ {
    use kodept_interning::InterningCodeHolder;

    InterningCodeHolder::new(&**source).map(|it| Cow::Borrowed(it.0))
}

#[cfg(not(feature = "interning"))]
fn get_code_holder(source: &SourceView) -> impl CodeHolder<Str = Str> + '_ {
    source.map(|it| Cow::Owned(it.to_string()))
}
