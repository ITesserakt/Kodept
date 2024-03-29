use std::fs::{create_dir_all, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use clap::{Parser, Subcommand};
use codespan_reporting::diagnostic::Diagnostic;
use extend::ext;
use nom_supreme::final_parser::final_parser;
use rayon::prelude::ParallelIterator;

use kodept::codespan_settings::CodespanSettings;
use kodept::macro_context::{DefaultContext, ErrorReported};
use kodept::parse_error::Reportable;
use kodept::read_code_source::ReadCodeSource;
use kodept::traversing::Traversable;
use kodept::{codespan_settings::ReportExt, top_parser};
use kodept_ast::ast_builder::ASTBuilder;
use kodept_core::file_relative::CodePath;
use kodept_core::structure::rlt::RLT;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::Tokenizer;
use kodept_parse::ParseError;

use crate::stage::PredefinedTraverseSet;
use crate::WideError;

#[derive(Parser, Debug)]
pub struct Graph;

#[derive(Debug)]
pub struct Execute;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Output AST in .dot format
    Graph(Graph),
}

#[ext]
impl<T> Result<T, Vec<Diagnostic<()>>> {
    fn or_emit_diagnostics(
        self,
        settings: &mut CodespanSettings,
        source: &ReadCodeSource,
    ) -> Result<T, ErrorReported> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                e.into_iter()
                    .try_for_each(|it| it.emit(settings, source))
                    .expect("Cannot emit diagnostics");
                Err(ErrorReported)
            }
        }
    }
}

impl Commands {
    #[allow(clippy::let_and_return)]
    fn build_rlt(source: &ReadCodeSource) -> Result<RLT, Vec<Diagnostic<()>>> {
        let tokenizer = Tokenizer::new(source.contents());
        let tokens = tokenizer.into_vec();
        let token_stream = TokenStream::new(&tokens);
        let result =
            final_parser(top_parser)(token_stream).map_err(|e: ParseError| e.to_diagnostics());
        result
    }
}

impl Execute {
    pub fn exec(
        self,
        sources: impl ParallelIterator<Item = ReadCodeSource>,
        settings: CodespanSettings,
    ) -> Result<(), WideError> {
        sources.try_for_each_with(settings, |settings, source| {
            self.exec_for_source(source, settings)
        })
    }

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
    ) -> Result<(), WideError> {
        let rlt = Commands::build_rlt(&source)
            .or_emit_diagnostics(settings, &source)?
            .0;
        let (tree, accessor) = ASTBuilder.recursive_build(&rlt, &source);
        let context = DefaultContext::new(
            source.with_filename(|_| ReportCollector::new()),
            accessor,
            Rc::new(tree.build()),
        );
        let mut set = PredefinedTraverseSet::default();
        let context = set.traverse(context).or_else(|(errors, context)| {
            errors
                .into_iter()
                .map(|it| it.unwrap_report())
                .try_for_each(|it| it.emit(settings, &source))?;
            Result::<_, WideError>::Ok(context)
        })?;
        context.emit_diagnostics(settings, &source);

        Ok(())
    }
}

impl Graph {
    pub fn exec(
        sources: impl ParallelIterator<Item = ReadCodeSource>,
        settings: CodespanSettings,
        output_path: PathBuf,
    ) -> Result<(), WideError> {
        sources.try_for_each_with(settings, |settings, source| {
            Graph::exec_for_source(source, settings, &output_path)
        })
    }

    fn exec_for_source(
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        output_path: &Path,
    ) -> Result<(), WideError> {
        let rlt = Commands::build_rlt(&source).or_emit_diagnostics(settings, &source)?;
        let (tree, _) = ASTBuilder.recursive_build(&rlt.0, &source);
        let mut output_file = Self::get_output_file(&source, output_path)?;

        write!(output_file, "{}", tree.export_dot(&[]))?;
        Ok(())
    }

    fn get_output_file(source: &ReadCodeSource, output_path: &Path) -> Result<File, WideError> {
        let filename = match source.path() {
            CodePath::ToFile(file) => file
                .with_extension("kd.dot")
                .file_name()
                .expect("Source should be a file")
                .to_os_string(),
            CodePath::ToMemory(name) => PathBuf::from(name).with_extension("kd.dot").into(),
        };
        match create_dir_all(output_path) {
            Err(e) if e.kind() != ErrorKind::AlreadyExists => Err(e)?,
            _ => {}
        }
        Ok(File::create(output_path.join(filename))?)
    }
}
