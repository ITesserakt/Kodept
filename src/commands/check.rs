use crate::cli::configs::{LoadingConfig, ParsingConfig};
use clap::Parser;
use kodept_frontend::engine::utils::Timings;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::global::prelude::{EachSubEnginePhase, FinishPhase, LoadAllSourcesPhase};
use kodept_systems::per_file::inject_common_resources_phase;
use kodept_systems::per_file::prelude::{BuildAstPhase, ParseSourcePhase};
use kodept_systems::source::collection::SourceView;

#[derive(Debug, Parser)]
pub struct Check {
    /// Measure duration of different stages
    #[arg(short, long, action, default_value_t = false)]
    timings: bool,
    /// Enable some lints during analysis that are disabled by default
    #[arg(long)]
    enabled_lints: Vec<String>,

    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
}

impl Plugin for Check {
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
                engine.insert_resource(self.parsing_config.get_parsing_backend());
                let source = engine.resource::<SourceView>();
                let lexing_backend = self.parsing_config.get_lexing_backend(source.contents());
                engine.insert_resource(lexing_backend);

                engine.install(ParseSourcePhase).install(BuildAstPhase);
            }))
            .install(FinishPhase);
    }
}
