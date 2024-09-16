use crate::cli::commands::build_rlt;
use crate::cli::traits::Command;
use clap::Parser;
use kodept::codespan_settings::CodespanSettings;
use kodept::source_files::SourceView;
use kodept::steps::common::Config;
use kodept_ast::graph::SyntaxTree;
use kodept_macros::context::{Context, FileDescriptor};
use kodept_macros::error::traits::ResultTRExt;
use kodept_macros::error::ErrorReported;
use std::num::NonZeroU16;
use tracing::debug;

#[derive(Debug, Parser, Clone)]
pub struct Execute {
    /// Specifies maximum number of steps while type checking a function
    #[arg(default_value_t = NonZeroU16::new(256).unwrap(), long = "recursion_depth")]
    type_checking_recursion_depth: NonZeroU16,
}

impl Command for Execute {
    type Params = ();

    fn exec_for_source(
        &self,
        source: SourceView,
        settings: &mut CodespanSettings,
        _: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, &*source);
        debug!("Produced AST with node count = {}", tree.node_count());
        let context = Context {
            ast: tree,
            rlt: accessor,
            reports: Default::default(),
            current_file: FileDescriptor {
                name: source.path(),
                id: 0,
            },
        };
        let config = Config {
            recursion_depth: self.type_checking_recursion_depth,
        };

        kodept::steps::common::run_common_steps(context, &config).or_emit(settings, &source)?;
        Ok(())
    }
}
