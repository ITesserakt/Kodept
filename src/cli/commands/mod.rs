use crate::cli::commands::execute::Execute;
use crate::cli::commands::graph::Graph;
use crate::cli::commands::inspect::InspectParser;
use crate::cli::traits::CommandWithSources;
use clap::Subcommand;
use itertools::Itertools;
use kodept::codespan_settings::{ConsumeCollector, ProvideCollector, Reports};
use kodept::read_code_source::ReadCodeSource;
use kodept::source_files::GlobalReports;
use kodept_macros::error::report::{Label, Severity};
use kodept_macros::error::{Diagnostic, ErrorReported};
use kodept_parse::error::{ParseError, ParseErrors};
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
    pub fn execute(self, output: PathBuf, mut reports: Reports) -> Result<(), ErrorReported> {
        match self {
            Commands::Graph(x) => {
                let sources = reports
                    .provide_collector(&GlobalReports, |collector| x.build_sources(collector))
                    .map(Arc::new)
                    .ok_or(ErrorReported::new())?;
                let result = x
                    .exec(sources.clone(), &mut reports, output)
                    .ok_or(ErrorReported::new());
                reports.consume(&*sources);
                result
            }
            Commands::InspectParser(x) => {
                let sources = reports
                    .provide_collector(&GlobalReports, |collector| x.build_sources(collector))
                    .map(Arc::new)
                    .ok_or(ErrorReported::new())?;
                let result = x
                    .exec(sources.clone(), &mut reports, output)
                    .ok_or(ErrorReported::new());
                reports.consume(&*sources);
                result
            }
            Commands::Execute(x) => {
                let sources = reports
                    .provide_collector(&GlobalReports, |collector| x.build_sources(collector))
                    .map(Arc::new)
                    .ok_or(ErrorReported::new())?;
                let result = x
                    .exec(sources.clone(), &mut reports, output)
                    .ok_or(ErrorReported::new());
                reports.consume(&*sources);
                result
            }
        }
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
    
    if expected.is_empty() {
        last_expected
    } else {
        format!("{} or {}", expected.into_iter().join(", "), last_expected).into()
    }
}

fn to_diagnostics<A: Display>(errors: ParseErrors<A>) -> Vec<Diagnostic> {
    errors.into_iter().map(to_diagnostic).collect()
}

fn get_output_file(source: &ReadCodeSource, output_path: &Path) -> std::io::Result<File> {
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
