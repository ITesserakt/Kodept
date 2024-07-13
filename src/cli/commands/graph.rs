use std::path::PathBuf;
use clap::Parser;
use kodept::codespan_settings::CodespanSettings;
use kodept::read_code_source::ReadCodeSource;
use kodept_ast::ast_builder::ASTBuilder;
use crate::cli::commands::{build_rlt, get_output_file};
use crate::cli::traits::{Command};
use std::io::Write;
use kodept_macros::error::ErrorReported;
use kodept_macros::error::traits::ResultTRExt;
use kodept_macros::error::traits::ResultTEExt;

#[derive(Parser, Debug, Clone)]
pub struct Graph;

impl Command for Graph {
    type Params = PathBuf;

    fn exec_for_source(&self, source: ReadCodeSource, settings: &mut CodespanSettings, params: &mut Self::Params) -> Result<(), ErrorReported> {
        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let (tree, _) = ASTBuilder.recursive_build(&rlt.0, &source);
        let mut output_file = get_output_file(&source, params).or_emit(settings, &source, source.path())?;
        
        write!(output_file, "{}", tree.export_dot(&[])).or_emit(settings, &source, source.path())?;
        Ok(())
    }
}