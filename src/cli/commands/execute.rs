use crate::cli::commands::build_rlt;
use crate::cli::traits::Command;
use clap::Parser;
use kodept::codespan_settings::CodespanSettings;
use kodept::macro_context::MacroContext;
use kodept::read_code_source::ReadCodeSource;
use kodept::steps::capabilities::BasicCapability;
use kodept::steps::common::Config;
use kodept_ast::graph::SyntaxTree;
use kodept_macros::error::report_collector::ReportCollector;
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

#[cfg(not(feature = "parallel"))]
impl Command for Execute {
    type Params = ();

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        _: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let (tree, accessor) = SyntaxTree::recursively_build(&rlt, &source);
        debug!("Produced AST with node count = {}", tree.node_count());
        let context = MacroContext::new(BasicCapability {
            file: source.path(),
            ast: tree,
            rlt: accessor,
            reporter: ReportCollector::new(),
        });
        let config = Config {
            recursion_depth: self.type_checking_recursion_depth,
        };

        kodept::steps::common::run_common_steps(context, &config).or_emit(settings, &source)?;
        Ok(())
    }
}

#[cfg(feature = "parallel")]
impl Command for Execute {
    type Params = ();

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        _: &mut Self::Params,
    ) -> Result<(), ErrorReported> {
        use kodept_ast::traits::parallel::Parallelize;

        let rlt = build_rlt(&source).or_emit(settings, &source)?;
        let parallel_rlt = Parallelize(rlt.0);
        let (tree, accessor) = SyntaxTree::parallel_recursively_build(&parallel_rlt, &source);
        debug!("Produced AST with node count = {}", tree.node_count());
        let context = MacroContext::new(BasicCapability {
            file: source.path(),
            ast: tree,
            rlt: accessor,
            reporter: ReportCollector::new(),
        });
        let config = Config {
            recursion_depth: self.type_checking_recursion_depth,
        };

        kodept::steps::common::run_common_steps(context, &config).or_emit(settings, &source)?;
        Ok(())
    }
}
