use clap::Parser;
use kodept_cli::prelude::{LexerChoice, LoadingConfig, OutputConfig, ParserChoice, ParsingConfig};
use kodept_frontend::engine::utils::Timings;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::configs::{Lexer, OutputDirectory, Parser as ParserImpl};
use kodept_systems::global::prelude::{EachSubEnginePhase, FinishPhase, LoadAllSourcesPhase};
use kodept_systems::per_file::inject_common_resources_phase;
use kodept_systems::per_file::prelude::{AstPassesPhase, BuildAstPhase, ParseSourcePhase};
use kodept_systems::source::collection::SourceView;
use crate::commands::Convert;

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
    #[command(flatten, next_help_heading = "Output options")]
    pub output_config: OutputConfig,
}

impl Plugin for Check {
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
                engine.insert_resource(OutputDirectory::new(&self.output_config.output));

                let source = engine.resource::<SourceView>();
                let lexer = match self.parsing_config.lexer {
                    LexerChoice::Auto if source.contents().is_ascii() => Lexer::Ascii,
                    LexerChoice::Auto => Lexer::Peg,
                    LexerChoice::Peg => Lexer::Peg,
                    LexerChoice::ASCII if source.contents().is_ascii() => Lexer::Ascii,
                    LexerChoice::ASCII => panic!("Cannot use ascii lexer on non-ascii input"),
                };
                engine.insert_resource(lexer);

                engine.insert_resource(match self.parsing_config.parser {
                    ParserChoice::Peg => ParserImpl::Peg,
                    ParserChoice::Auto => ParserImpl::Peg,
                });

                engine
                    .install(ParseSourcePhase)
                    .install(BuildAstPhase)
                    .install(AstPassesPhase);
            }))
            .install(FinishPhase);
    }
}
