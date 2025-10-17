use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::commands::inspect::export_ast::ExportAstPhase;
use crate::commands::inspect::export_rlt::ExportRltPhase;
use crate::phases::build_ast::BuildAstPhase;
use crate::phases::each_sub_engine::EachSubEnginePhase;
use crate::phases::load_all_sources::LoadAllSourcesPhase;
use crate::phases::parse_source::ParseSourcePhase;
use clap::Parser;
use kodept_frontend::engine::utils::{InjectResourcesPhase, Timings};
use kodept_frontend::engine::{Engine, Plugin};
use crate::commands::inject_common_resources;

#[derive(Parser, Debug, Clone)]
pub struct Inspect {
    /// Measure duration of different stages
    #[arg(short = 't', long, action)]
    timings: bool,
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

impl Plugin for Inspect {
    fn build(self, engine: &mut Engine) {
        if self.timings {
            engine.init_resource::<Timings>();
        }

        engine
            .install(LoadAllSourcesPhase {
                config: self.loading_config,
            })
            .install(InjectResourcesPhase::new(inject_common_resources))
            .install(EachSubEnginePhase::new(move |engine| {
                if !self.export_rlt && !self.export_rlt {
                    return;
                }

                let mut sources = engine.install(ParseSourcePhase {
                    config: self.parsing_config.clone(),
                });

                if self.export_rlt {
                    sources.install(ExportRltPhase);
                }

                if self.export_ast {
                    sources.install(BuildAstPhase).install(ExportAstPhase);
                }
            }));
    }
}

mod export_rlt {
    use crate::cli::primary::OutputConfig;
    use bevy_ecs::prelude::*;
    use derive_more::{Display, Error, From};
    use kodept::source::collection::{Reporter, SourceView};
    use kodept::utils::ReportSystemEx;
    use kodept_ast::resource::rlt::SyntaxResolver;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::PhaseEngine;
    use kodept_report::prelude::{Diagnostic, Severity};
    use std::fs::File;

    define_phase!(
        pub phase ExportRltPhase[ExportRltPhaseLabel];

        fn build (self, engine: &mut PhaseEngine<Self>) {
            engine.add_systems(
                system.report_errors(),
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
        config: Res<OutputConfig>,
        source: Res<SourceView>,
        syntax: Res<SyntaxResolver>,
        mut reporter: Reporter,
    ) -> Result<(), Error> {
        let output_filepath = config.get_path_for_source(source.path(), "rlt.json")?;
        let output_file = File::create(&output_filepath)?;
        serde_json::to_writer(output_file, syntax.root().0)?;

        reporter.report_ad_hoc(|| {
            Diagnostic::new(Severity::Note).with_message(format!(
                "Successfully exported RLT to {}",
                output_filepath.display()
            ))
        });

        Ok(())
    }
}

mod export_ast {
    use crate::cli::primary::OutputConfig;
    use bevy_ecs::prelude::*;
    use kodept::source::collection::{Reporter, SourceView};
    use kodept::utils::ReportSystemEx;
    use kodept_ast::syntax_tree::prelude::AST;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::PhaseEngine;
    use kodept_report::prelude::{Diagnostic, Severity};
    use std::fs::File;

    define_phase!(
        pub phase ExportAstPhase[ExportAstPhaseLabel];

        fn build(self, engine: &mut PhaseEngine<Self>) {
            engine.add_systems(
                system.report_errors()
            )
        }
    );

    fn system(
        config: Res<OutputConfig>,
        source: Res<SourceView>,
        mut reporter: Reporter,
        world: &World,
    ) -> Result<(), std::io::Error> {
        let output_filepath = config.get_path_for_source(source.path(), "puml")?;
        let mut output_file = File::create(&output_filepath)?;
        AST::export_dot_in(world, &mut output_file)
            .expect("Some components did not registered still")?;

        reporter.report_ad_hoc(|| {
            Diagnostic::new(Severity::Note).with_message(format!(
                "Successfully exported AST to {}",
                output_filepath.display()
            ))
        });

        Ok(())
    }
}
