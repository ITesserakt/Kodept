use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::inspect::export_ast::ExportAstPhase;
use crate::commands::inspect::export_rlt::ExportRltPhase;
use crate::commands::{CommandV2};
use crate::phases::build_ast::BuildAstPhase;
use crate::phases::each_sub_engine::EachSubEnginePhase;
use crate::phases::load_all_sources::LoadAllSourcesPhase;
use crate::phases::parse_source::ParseSourcePhase;
use clap::Parser;
use kodept_frontend::engine::Engine;

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

mod export_rlt {
    use crate::cli::primary::OutputConfig;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::InMut;
    use derive_more::{Display, Error, From};
    use kodept::source::collection::{Reporter, SourceView, SystemExt};
    use kodept_ast::resource::rlt::SyntaxResolver;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::Engine;
    use kodept_report::prelude::{Diagnostic, Severity};
    use std::fs::File;

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
    use kodept::source::collection::{Reporter, SourceView, SystemExt};
    use kodept_ast::syntax_tree::prelude::AST;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::Engine;
    use kodept_report::prelude::{Diagnostic, Severity};
    use std::fs::File;

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
        mut reporter: Reporter,
        world: &World,
    ) -> Result<(), std::io::Error> {
        let output_filepath = config.get_path_for_source(source.path(), "puml")?;
        let mut output_file = File::create(&output_filepath)?;
        AST::export_dot_in(world, &mut output_file).expect("Some components did not registered still")?;

        reporter.report_ad_hoc(|| {
            Diagnostic::new(Severity::Note).with_message(format!(
                "Successfully exported AST to {}",
                output_filepath.display()
            ))
        });

        Ok(())
    }
}

