use std::borrow::Cow;
use crate::cli::commands::execute::Execute;
use crate::cli::commands::graph::Graph;
use crate::cli::commands::inspect::InspectParser;
use crate::cli::common::Kodept;
use crate::cli::traits::{Command};
use clap::Subcommand;
use itertools::Itertools;
use kodept::read_code_source::ReadCodeSource;
use kodept::source_files::SourceView;
use kodept_core::structure::rlt::RLT;
use kodept_macros::error::report::{Label, Severity};
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::DrainReports;
use kodept_macros::error::{Diagnostic, ErrorReported};
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_parse::parser::{parse_from_top, LaLRPop, PegParser};
use kodept_parse::token_match::TokenMatch;
use kodept_parse::token_stream::TokenStream;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::Path;
use tracing::debug;

mod execute;
mod graph;
mod inspect;

#[cfg(feature = "parallel")]
const SWITCH_TO_PARALLEL_THRESHOLD: usize = 20 * 1024; // 20 KB

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Output AST in .dot format
    Graph(Graph),
    /// Output parsing process files
    InspectParser(InspectParser),
    /// Run type checker
    Execute(Execute),
}

impl Command for Commands {
    type Params = Kodept;

    fn exec_for_source(
        &self,
        source: SourceView,
        collector: &mut ReportCollector,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        match self {
            Commands::Graph(x) => x.exec_for_source(source, collector, &mut params.output),
            Commands::InspectParser(x) => x.exec_for_source(source, collector, &mut params.output),
            Commands::Execute(x) => x.exec_for_source(source, collector, &mut ()),
        }
    }
}

fn to_diagnostic<A: Display>(error: ParseError<A>) -> Diagnostic {
    let exp_msg = if error.expected.is_empty() {
        Cow::Borrowed("???")
    } else {
        error.expected.into_iter().join(" or ").into()
    };

    Diagnostic::new(Severity::Error)
        .with_message(format!("Expected {}, got \"{}\"", exp_msg, error.actual))
        .with_label(Label::primary("here", error.location.in_code))
}

fn tokenize(source: &ReadCodeSource) -> Result<Vec<TokenMatch>, ParseErrors<&str>> {
    use kodept_parse::lexer::*;
    use kodept_parse::tokenizer::*;

    #[cfg(feature = "parallel")]
    {
        let backend: PegLexer<false> = PegLexer::new();
        if source.contents().len() > SWITCH_TO_PARALLEL_THRESHOLD {
            debug!(
                backend = std::any::type_name_of_val(&backend),
                "Using parallel tokenizer"
            );
            return ParallelTokenizer::new(source.contents(), backend).try_collect_adapted();
        }
    }
    let backend = PestLexer::new();
    debug!(
        backend = std::any::type_name_of_val(&backend),
        "Using sequential tokenizer"
    );
    EagerTokenizer::new(source.contents(), backend).try_collect_adapted()
}

fn build_rlt(source: &SourceView, collector: &mut ReportCollector) -> Option<RLT> {
    let tokens = tokenize(source)
        .map_err(|es| es.into_iter().map(to_diagnostic))
        .drain(*source.id, collector)?;
    
    debug!(length = tokens.len(), "Produced token stream");
    let token_stream = TokenStream::new(&tokens);
    let result = parse_from_top(token_stream, LaLRPop::new())
        .map_err(|es| es.into_iter().map(to_diagnostic))
        .drain(*source.id, collector)?;
    
    debug!("Produced RLT with modules count {}", result.0 .0.len());
    Some(result)
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
