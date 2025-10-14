use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::inspect::export_ast::ExportAstPhase;
use crate::commands::inspect::export_rlt::ExportRltPhase;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::{Command, CommandV2};
use crate::phases::build_ast::BuildAstPhase;
use crate::phases::each_sub_engine::EachSubEnginePhase;
use crate::phases::load_all_sources::LoadAllSourcesPhase;
use crate::phases::parse_source::ParseSourcePhase;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept::source::collection::SourceView;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_core::code_point::CodePoint;
use kodept_frontend::Execution;
use kodept_frontend::engine::Engine;
use kodept_report::message::{Diagnostic, Severity};
use kodept_report::traits::ad_hoc_message;
use kodept_rlt::prelude::RLT;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use tracing::error;

#[derive(Parser, Debug, Clone)]
pub struct Inspect {
    /// Export raw lexeme tree in .json format into a file
    #[arg(short = 'r', long, action)]
    export_rlt: bool,
    /// Export abstract syntax tree in .dot format into a file
    #[arg(short = 'a', long, action)]
    export_ast: bool,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
}

impl CommandV2 for Inspect {
    fn build(self, engine: &mut Engine, config: OutputConfig) {
        engine
            .install(LoadAllSourcesPhase {
                config: self.loading_config,
            })
            .install(EachSubEnginePhase::new(move |engine| {
                if !self.export_rlt && !self.export_rlt {
                    return;
                }

                let mut sources = engine.install(ParseSourcePhase {
                    config: self.parsing_config.clone(),
                });

                if self.export_rlt {
                    sources.install(ExportRltPhase {
                        config: config.clone(),
                    });
                }

                if self.export_ast {
                    sources.install(BuildAstPhase).install(ExportAstPhase {
                        config: config.clone(),
                    });
                }
            }));
    }
}

impl Command for Inspect {
    fn exec(self, reports: GlobalReports, config: OutputConfig) -> ControlFlow<(), ()> {
        let (sources, reports) = get_all_sources(&self.loading_config, reports)?;
        for source in sources.collect() {
            let rlt = get_rlt(&self.parsing_config, &source, &reports)?;
            if self.export_rlt && export_rlt(&source, &config, &rlt).is_continue() {
                let message = ad_hoc_message(|| {
                    Diagnostic::new(Severity::Note)
                        .with_message("Source file parsed into a raw lexeme tree")
                        .with_primary_label("", CodePoint::single_point(0))
                });
                _ = reports.report(*source.id, message);
            }

            let mut ast = build_ast(&source, rlt, &reports)?;
            if self.export_ast && export_ast(&source, &config, &mut ast).is_continue() {
                let message = ad_hoc_message(|| {
                    Diagnostic::new(Severity::Note)
                        .with_message("Got abstract syntax tree of source file")
                        .with_primary_label("", CodePoint::single_point(0))
                });
                _ = reports.report(*source.id, message);
            }
        }
        Continue(())
    }
}

mod export_rlt {
    use crate::cli::primary::OutputConfig;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::InMut;
    use derive_more::{Display, Error, From};
    use kodept::source::collection::{SourceView, SystemExt};
    use kodept_ast::resource::rlt::SyntaxResolver;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::{Engine};

    define_phase!(
        pub phase ExportRltPhase[ExportRltPhaseLabel] {
            pub config: OutputConfig
        }
        fn build (self, engine: &mut Engine) {
            engine.add_systems(
                system
                    .with_input(self.config)
                    .report_errors()
                    .in_set(ExportRltPhaseLabel),
            );
        }
    );

    #[derive(Debug, Error, From, Display)]
    enum Error {
        #[display("Could not open file to output RLT: {_0}")]
        IO(std::io::Error),
        #[display("Could not serialize RLT into json: {_0}")]
        Serde(serde_json::Error),
    }

    fn system(
        InMut(config): InMut<OutputConfig>,
        source: Res<SourceView>,
        syntax: Res<SyntaxResolver>,
    ) -> Result<(), Error> {
        let output_file = config.open_file_for_source(source.path(), "rlt.json")?;
        serde_json::to_writer_pretty(output_file, syntax.root().0)?;

        Ok(())
    }
}

mod export_ast {
    use crate::cli::primary::OutputConfig;
    use bevy_ecs::prelude::*;
    use kodept::source::collection::{SourceView, SystemExt};
    use kodept_ast::syntax_tree::prelude::AST;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::Engine;

    define_phase!(
        pub phase ExportAstPhase[ExportAstPhaseLabel] {
            pub config: OutputConfig
        }

        fn build(self, engine: &mut Engine) {
            engine.add_systems(system
                .with_input(self.config)
                .report_errors()
                .in_set(ExportAstPhaseLabel)
            )
        }
    );

    fn system(
        InMut(config): InMut<OutputConfig>,
        source: Res<SourceView>,
        world: &World,
    ) -> Result<(), std::io::Error> {
        let mut output_file = config.open_file_for_source(source.path(), "puml")?;
        AST::export_dot_in(world, &mut output_file)?;
        Ok(())
    }
}

fn export_rlt(source: &SourceView, config: &OutputConfig, rlt: &RLT) -> Execution<()> {
    let output_file = match config.open_file_for_source(source.path(), "rlt.json") {
        Ok(x) => x,
        Err(e) => {
            error!("Could not open file to output RLT: {e}");
            return Break(());
        }
    };
    if let Err(e) = serde_json::to_writer_pretty(output_file, &rlt) {
        error!("Could not serialize RLT into json: {e}");
        return Break(());
    }
    Continue(())
}

fn export_ast(source: &SourceView, config: &OutputConfig, ast: &mut AST) -> Execution<()> {
    let mut output_file = match config.open_file_for_source(source.path(), "puml") {
        Ok(x) => x,
        Err(e) => {
            error!("Could not open file to output RLT: {e}");
            return Break(());
        }
    };
    if let Err(e) = ast.export_dot(&mut output_file) {
        error!("Could not export AST into .dot: {e}");
        return Break(());
    }
    Continue(())
}
