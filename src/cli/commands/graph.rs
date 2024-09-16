use crate::cli::commands::{build_rlt, get_output_file};
use crate::cli::traits::Command;
use clap::Parser;
use kodept::codespan_settings::CodespanSettings;
use kodept::source_files::SourceView;
use kodept_ast::graph::SyntaxTree;
use kodept_macros::error::traits::{ResultTEExt, ResultTRExt};
use kodept_macros::error::ErrorReported;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct Graph;

impl Command for Graph {
    type Params = PathBuf;

    fn exec_for_source(
        &self,
        source: SourceView,
        settings: &mut CodespanSettings,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let (tree, _) = SyntaxTree::recursively_build(&rlt, &*source);
        let mut output_file =
            get_output_file(&source, params).or_emit(settings, &source, *source.id)?;

        write!(output_file, "{}", tree.export_dot(&[])).or_emit(settings, &source, *source.id)?;
        Ok(())
    }
}
