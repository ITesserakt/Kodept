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
use kodept::source_files::SourceView;
use kodept_core::file_name::FileName;
use kodept_core::structure::rlt::RLT;
use kodept_macros::context::FileId;
use kodept_macros::error::ErrorReported;
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_parse::parser::{parse_from_top, PegParser};
use kodept_parse::token_match::TokenMatch;
use kodept_parse::token_stream::TokenStream;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
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

fn to_diagnostic<A: Display, FileId>(file_id: FileId, error: ParseError<A>) -> Diagnostic<FileId> {
    let exp_msg = error.expected.into_iter().join(" or ");

    Diagnostic::error()
        .with_code("SE001")
        .with_message(format!("Expected {}, got \"{}\"", exp_msg, error.actual))
        .with_labels(vec![Label::primary(
            file_id,
            error.location.in_code.as_range(),
        )])
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

fn build_rlt(source: &SourceView) -> Result<RLT, Vec<Diagnostic<FileId>>> {
    let tokens = tokenize(&*source).map_err(|es| {
        es.into_iter()
            .map(|it| to_diagnostic(*&*source.id, it))
            .collect::<Vec<_>>()
    })?;
    debug!(length = tokens.len(), "Produced token stream");
    let token_stream = TokenStream::new(&tokens);
    let result = parse_from_top(token_stream, PegParser::<false>::new()).map_err(|es| {
        es.into_iter()
            .map(|it| to_diagnostic(*&*source.id, it))
            .collect::<Vec<_>>()
    })?;
    debug!("Produced RLT with modules count {}", result.0 .0.len());
    Ok(result)
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
