use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::Command;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept_ast::interaction::Interaction as Ctx;
use kodept_frontend::Execution;
use kodept_interaction::lint::module::SingleModuleWithBrackets;
use kodept_interaction::report::{ASTExt, FileDescriptor};
use kodept_interaction::Interaction;
use std::ops::ControlFlow::Continue;

#[derive(Debug, Parser)]
pub struct TypeCheck {
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
}

impl Command for TypeCheck {
    fn exec(self, reports: GlobalReports, _: OutputConfig) -> Execution<()> {
        let (sources, reports) = get_all_sources(&self.loading_config, reports)?;
        for source in sources.collect() {
            let rlt = get_rlt(&self.parsing_config, &source, &reports)?;
            let mut ast = build_ast(&source, rlt);

            ast.prepare_reporting(FileDescriptor {
                id: *source.id,
                file_name: source.path().clone(),
            });

            let mut lints_interaction = ast.interact();
            install_lints(&mut lints_interaction);
            lints_interaction.launch();

            ast.extract_reports(|it| reports.insert(it));
        }
        Continue(())
    }
}

fn install_lints(ctx: &mut Ctx) {
    SingleModuleWithBrackets::install(ctx);
}
