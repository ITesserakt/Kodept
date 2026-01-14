use crate::commands::Convert;
use clap::Parser;
use kodept_cli::prelude::{LoadingConfig, OutputConfig, ParsingConfig};
use kodept_frontend::engine::utils::Timings;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::configs::OutputDirectory;
use kodept_systems::global::prelude::{EachSubEnginePhase, FinishPhase, LoadAllSourcesPhase};
use kodept_systems::per_file::inject_common_resources_phase;
use kodept_systems::per_file::prelude::{ExportRltPhase, ParseSourcePhase};

#[derive(Parser, Debug)]
pub struct Inspect {
    /// Measure duration of different stages
    #[arg(short = 't', long, action)]
    timings: bool,
    /// Export raw lexeme tree in .json format into a file
    #[arg(short = 'r', long, action)]
    export_rlt: bool,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Output options")]
    pub output_config: OutputConfig,
}

impl Plugin for Inspect {
    fn build(self, engine: &mut Engine) {
        if self.timings {
            engine.init_resource::<Timings>();
        }

        engine
            .install(LoadAllSourcesPhase {
                config: Convert(self.loading_config),
            })
            .install(inject_common_resources_phase())
            .install(EachSubEnginePhase::new(move |engine| {
                if !self.export_rlt {
                    return;
                }

                engine.insert_resource(OutputDirectory::new(&self.output_config.output));

                let mut sources = engine.install(ParseSourcePhase);

                if self.export_rlt {
                    sources.install(ExportRltPhase);
                }
            }))
            .install(FinishPhase);
    }
}
