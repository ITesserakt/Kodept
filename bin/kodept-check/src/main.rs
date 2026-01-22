use clap::Parser;
use kodept_cli::prelude::{
    DiagnosticConfig, Extension, LexerChoice, LoadingConfig, ParserChoice, ParsingConfig,
    ReportsPlugin,
};
use kodept_frontend::engine::Engine;
use kodept_frontend::engine::reporter::CompilationFailed;
use kodept_frontend::engine::utils::Timings;
use kodept_systems::configs::Lexer;
use kodept_systems::global::prelude::{EachSubEnginePhase, FinishPhase, LoadAllSourcesPhase};
use kodept_systems::loader::{Loader, LoadingError};
use kodept_systems::per_file::inject_common_resources_phase;
use kodept_systems::per_file::prelude::{
    AstNormalizationPhase, AstPassesPhase, BuildAstPhase, ParseSourcePhase,
};
use kodept_systems::source::collection::SourceView;
use std::io::{Read, stdin};

#[derive(Debug, Parser)]
struct Cli {
    /// Measure duration of different stages
    #[arg(short = 't', long, action)]
    timings: bool,

    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Diagnostic options")]
    diagnostic_config: DiagnosticConfig,
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

fn main() -> Result<(), CompilationFailed> {
    let cli = Cli::parse();
    let mut engine = Engine::new();

    if cli.timings {
        engine.init_resource::<Timings>();
    }

    engine.add_plugin(ReportsPlugin {
        config: &cli.diagnostic_config,
    });

    engine
        .install(LoadAllSourcesPhase {
            config: Convert(cli.loading_config),
        })
        .install(inject_common_resources_phase())
        .install(EachSubEnginePhase::new(move |engine| {
            let source = engine.resource::<SourceView>();
            let is_ascii = source.contents().is_ascii();
            engine.insert_resource(match cli.parsing_config.lexer {
                LexerChoice::Peg => Lexer::Peg,
                LexerChoice::ASCII if is_ascii => Lexer::Ascii,
                LexerChoice::ASCII => panic!("Cannot use ascii lexer for non-ascii input"),
                LexerChoice::Auto if is_ascii => Lexer::Ascii,
                LexerChoice::Auto => Lexer::Peg,
            });

            engine.insert_resource(match cli.parsing_config.parser {
                ParserChoice::Peg => kodept_systems::configs::Parser::Peg,
                ParserChoice::Auto => kodept_systems::configs::Parser::Peg,
            });

            engine
                .install(ParseSourcePhase)
                .install(BuildAstPhase)
                .install(AstNormalizationPhase)
                .install(AstPassesPhase);
        }))
        .install(FinishPhase);

    engine.run()
}
