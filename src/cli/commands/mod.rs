use crate::cli::commands::execute::Execute;
use crate::cli::commands::graph::Graph;
use crate::cli::commands::inspect::InspectParser;
use crate::cli::common::Kodept;
use crate::cli::traits::Command;
use clap::Subcommand;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
use kodept::codespan_settings::CodespanSettings;
use kodept::read_code_source::ReadCodeSource;
use kodept_core::file_relative::CodePath;
use kodept_core::structure::rlt::RLT;
use kodept_macros::error::ErrorReported;
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_parse::parser::default_parse_from_top;
use kodept_parse::token_match::TokenMatch;
use kodept_parse::token_stream::TokenStream;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tracing::info;

mod execute;
mod graph;
mod inspect;

#[cfg(feature = "parallel")]
const SWITCH_TO_PARALLEL_THRESHOLD: usize = 20 * 1024 * 1024; // 20 MB

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
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        match self {
            Commands::Graph(x) => x.exec_for_source(source, settings, &mut params.output),
            Commands::InspectParser(x) => x.exec_for_source(source, settings, &mut params.output),
            Commands::Execute(x) => x.exec_for_source(source, settings, &mut ()),
        }
    }
}

fn to_diagnostic<A: Display>(error: ParseError<A>) -> Diagnostic<()> {
    let exp_msg = error.expected.into_iter().join(" or ");

    Diagnostic::error()
        .with_code("SE001")
        .with_message(format!("Expected {}, got \"{}\"", exp_msg, error.actual))
        .with_labels(vec![Label::primary((), error.location.in_code.as_range())])
}

fn tokenize(source: &ReadCodeSource) -> Result<Vec<TokenMatch>, ParseErrors<&str>> {
    use kodept_parse::tokenizer::*;
    
    #[cfg(feature = "parallel")]
    {
        if source.contents().len() > SWITCH_TO_PARALLEL_THRESHOLD {
            return ParallelTokenizer::default(source.contents()).try_collect_adapted();
        }
    }
    EagerTokenizer::default(source.contents()).try_collect_adapted()
}

fn build_rlt(source: &ReadCodeSource) -> Result<RLT, Vec<Diagnostic<()>>> {
    let tokens =
        tokenize(source).map_err(|es| es.into_iter().map(to_diagnostic).collect::<Vec<_>>())?;
    info!("Tokens length = {}", tokens.len());
    let token_stream = TokenStream::new(&tokens);
    let result = default_parse_from_top(token_stream)
        .map_err(|es| es.into_iter().map(to_diagnostic).collect::<Vec<_>>())?;
    Ok(result)
}

fn get_output_file(source: &ReadCodeSource, output_path: &Path) -> std::io::Result<File> {
    let filename = match source.path() {
        CodePath::ToFile(file) => file
            .with_extension("kd.dot")
            .file_name()
            .expect("Source should be a file")
            .to_os_string(),
        CodePath::ToMemory(name) => PathBuf::from(name).with_extension("kd.dot").into(),
    };
    ensure_path_exists(output_path)?;
    File::create(output_path.join(filename))
}

fn ensure_path_exists(path: &Path) -> std::io::Result<()> {
    match create_dir_all(path) {
        Err(e) if e.kind() != ErrorKind::AlreadyExists => Err(e)?,
        _ => Ok(()),
    }
}
