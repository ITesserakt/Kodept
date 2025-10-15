use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::phases::build_ast::BuildAstPhase;
use crate::phases::each_sub_engine::EachSubEnginePhase;
use crate::phases::load_all_sources::LoadAllSourcesPhase;
use crate::phases::parse_source::ParseSourcePhase;
use clap::Parser;
use kodept_frontend::engine::{Engine, Plugin};

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
        engine
            .install(LoadAllSourcesPhase {
                config: self.loading_config,
            })
            .install(EachSubEnginePhase::new(move |engine| {
                engine
                    .install(ParseSourcePhase {
                        config: self.parsing_config.clone(),
                    })
                    .install(BuildAstPhase);
            }));
    }
}
