use crate::cli::configs::{LoadingConfig, ParsingConfig};
use clap::Parser;
use kodept_frontend::engine::utils::{Timings};
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::configs::OutputDirectory;
use kodept_systems::global::prelude::{EachSubEnginePhase, FinishPhase, LoadAllSourcesPhase};
use kodept_systems::per_file::inject_common_resources_phase;
use kodept_systems::per_file::prelude::{BuildAstPhase, ExportAstPhase, ExportRltPhase, ParseSourcePhase};
use kodept_systems::source::collection::SourceView;
use crate::cli::primary::OutputConfig;

#[derive(Parser, Debug)]
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
                config: self.loading_config,
            })
            .install(inject_common_resources_phase())
            .install(EachSubEnginePhase::new(move |engine| {
                if !self.export_rlt && !self.export_ast {
                    return;
                }

                engine.insert_resource(OutputDirectory::new(&self.output_config.output));
                engine.insert_resource(self.parsing_config.get_parsing_backend());
                let source = engine.resource::<SourceView>();
                let lexing_backend = self.parsing_config.get_lexing_backend(source.contents());
                engine.insert_resource(lexing_backend);

                let mut sources = engine.install(ParseSourcePhase);

                if self.export_rlt {
                    sources.install(ExportRltPhase);
                }

                if self.export_ast {
                    sources.install(BuildAstPhase).install(ExportAstPhase);
                }
            }))
            .install(FinishPhase);
    }
}
