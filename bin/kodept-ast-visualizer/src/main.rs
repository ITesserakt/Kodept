#[cfg(feature = "graphviz")]
mod graphviz;
mod phase;
mod plugins;

use crate::phase::ExportAstPhase;
use crate::plugins::Plugins;
use bevy_ecs::prelude::{Entity, Event, InMut, Res};
use clap::Parser;
use kodept_ast::relationship::RelationshipMetadata;
use kodept_cli::prelude::{
    DiagnosticConfig, Extension, LexerChoice, LoadingConfig, OutputConfig,
    ParserChoice, ParsingConfig,
};
use kodept_frontend::engine::reporter::StopEngine;
use kodept_frontend::engine::utils::{InjectResourcesPhase, Timings};
use kodept_frontend::engine::{Engine, SubEngine};
use kodept_systems::configs::{Lexer, OutputDirectory};
use kodept_systems::global::prelude::{EachSubEnginePhase, FinishPhase, LoadAllSourcesPhase};
use kodept_systems::loader::{Loader, LoadingError};
use kodept_systems::per_file::prelude::{BuildAstPhase, ParseSourcePhase};
use kodept_systems::source::collection::SourceView;
use std::io::{stdin, Read};
use kodept_ast::resource::reflection::DebugRegistry;

#[derive(Debug, Parser)]
struct Cli {
    /// Measure duration of different stages
    #[arg(short = 't', long, action)]
    timings: bool,

    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Output options")]
    output_config: OutputConfig,
    #[command(flatten, next_help_heading = "Diagnostic options")]
    diagnostic_config: DiagnosticConfig,

    #[cfg(feature = "graphviz")]
    #[command(flatten, next_help_heading = "Drawing options")]
    draw_config: graphviz::Config,
}

#[derive(Debug, Event)]
enum ExportControlEvent {
    Start,
    Finish,
    Root(Entity),
    Inner {
        parent_id: Entity,
        metadata: RelationshipMetadata,
        this_id: Entity,
    },
}

struct Convert(LoadingConfig);

impl TryFrom<&Convert> for Loader {
    type Error = LoadingError;

    fn try_from(value: &Convert) -> Result<Self, Self::Error> {
        if value.0.read_stdin {
            let mut stdin_input = String::new();
            stdin().read_to_string(&mut stdin_input)?;
            Ok(Loader::from_single_snippet(stdin_input))
        } else {
            let builder = Loader::file();
            let builder = match &value.0.extension {
                Extension::Any => builder.with_any_source_extension(),
                Extension::Specified(ext) => builder.with_extension(ext),
            };
            let builder = match value.0.input.first() {
                None => builder,
                Some(x) => builder.with_starting_path(x),
            };
            builder.build()
        }
    }
}

fn main() -> Result<(), StopEngine> {
    let cli_args = Cli::parse();
    let mut engine = Engine::new();

    engine.add_plugin(Plugins { config: &cli_args });

    if cli_args.timings {
        engine.init_resource::<Timings>();
    }

    engine
        .install(LoadAllSourcesPhase {
            config: Convert(cli_args.loading_config),
        })
        .install(InjectResourcesPhase::new(|
            InMut(engine): InMut<SubEngine>,
            timings: Option<Res<Timings>>,
            report_settings: Option<Res<kodept_frontend::engine::reporter::Settings>>,
            debug_registry: Option<Res<DebugRegistry>>
        | {
            if timings.is_some() {
                engine.init_resource::<Timings>();
            }
            if let Some(settings) = report_settings {
                engine.insert_resource(settings.as_ref().clone());
            }
            if let Some(registry) = debug_registry {
                engine.insert_resource(registry.as_ref().clone());
            }
        }))
        .install(EachSubEnginePhase::new(move |engine| {
            engine.insert_resource(OutputDirectory::new(&cli_args.output_config.output));

            let source = engine.resource::<SourceView>();
            let is_ascii = source.contents().is_ascii();
            engine.insert_resource(match cli_args.parsing_config.lexer {
                LexerChoice::Peg => Lexer::Peg,
                LexerChoice::ASCII if is_ascii => Lexer::Ascii,
                LexerChoice::ASCII => panic!("Cannot use ascii lexer for non-ascii input"),
                LexerChoice::Auto if is_ascii => Lexer::Ascii,
                LexerChoice::Auto => Lexer::Peg,
            });

            engine.insert_resource(match cli_args.parsing_config.parser {
                ParserChoice::Peg => kodept_systems::configs::Parser::Peg,
                ParserChoice::Auto => kodept_systems::configs::Parser::Peg,
            });

            #[cfg(feature = "graphviz")]
            {
                engine.add_plugin(graphviz::GraphvizPlugin);
                engine.insert_resource(cli_args.draw_config.clone());
            }

            engine
                .install(ParseSourcePhase)
                .install(BuildAstPhase)
                .install(ExportAstPhase);
        }))
        .install(FinishPhase);

    engine.run()
}
