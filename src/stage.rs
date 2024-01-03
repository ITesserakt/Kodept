use crate::cli::Kodept;
use clap::Parser;
use kodept::codespan_settings::{CodespanSettings, ReportExt};
use kodept::macro_context::DefaultContext;
use kodept::parse_error::Reportable;
use kodept::read_code_source::ReadCodeSource;
use kodept::top_parser;
use kodept::traversing::{Traversable, TraverseSet};
use kodept_ast::ast_builder::ASTBuilder;
use kodept_core::code_source::CodeSource;
use kodept_core::file_relative::FileRelative;
use kodept_core::loader::{Loader, LoadingError};
use kodept_core::structure::rlt::RLT;
use kodept_macros::analyzers::ast_formatter::ASTFormatter;
use kodept_macros::analyzers::module_analyzer::ModuleUniquenessAnalyzer;
use kodept_macros::analyzers::variable_uniqueness::VariableUniquenessAnalyzer;
use kodept_macros::erased::ErasedAnalyzer;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::traits::{Context, UnrecoverableError};
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::Tokenizer;
use kodept_parse::ParseError;
use nom_supreme::final_parser::final_parser;
use std::io::stdout;
use std::rc::Rc;

pub struct PredefinedTraverseSet<'c, C: Context<'c>, E>(TraverseSet<'c, C, E>);

impl<'c, C: Context<'c>> Default for PredefinedTraverseSet<'c, C, UnrecoverableError> {
    fn default() -> Self {
        let mut set = TraverseSet::empty();
        set.add_independent(ASTFormatter::new(stdout()).erase());
        set.add_independent(ModuleUniquenessAnalyzer.erase());
        set.add_independent(VariableUniquenessAnalyzer.erase());
        Self(set)
    }
}

impl<'c, C: Context<'c>, E> Traversable<'c, C, E> for PredefinedTraverseSet<'c, C, E> {
    fn traverse(&self, context: C) -> Result<C, (E, C)> {
        self.0.traverse(context)
    }
}

pub struct Prepare;

impl Prepare {
    pub fn run(self) -> Result<(CodespanSettings, Vec<CodeSource>), LoadingError> {
        let cli_arguments = Kodept::parse();
        tracing_subscriber::fmt()
            .with_max_level(cli_arguments.level())
            .init();

        let settings = cli_arguments.diagnostic_config.into();
        let loader: Loader = cli_arguments.loading_config.try_into()?;
        Ok((settings, loader.into_sources()))
    }
}

pub struct Reading;

impl Reading {
    pub fn run(self, source: CodeSource) -> Result<ReadCodeSource, std::io::Error> {
        source.try_into()
    }
}

pub struct BuildingRLT;

impl BuildingRLT {
    pub fn run(self, source: &ReadCodeSource, settings: &mut CodespanSettings) -> Option<RLT> {
        let tokenizer = Tokenizer::new(source.contents());
        let tokens = tokenizer.into_vec();
        let stream = TokenStream::new(&tokens);
        let rlt = match final_parser::<_, _, _, ParseError>(top_parser)(stream) {
            Ok(x) => x,
            Err(e) => {
                e.to_diagnostics()
                    .into_iter()
                    .try_for_each(|it| it.emit(settings, source))
                    .expect("Cannot emit diagnostics");
                return None;
            }
        };
        Some(rlt)
    }
}

pub struct BuildingAST;

impl BuildingAST {
    pub fn run<'c>(self, source: &ReadCodeSource, rlt: &'c RLT) -> DefaultContext<'c> {
        let mut builder = ASTBuilder::default();
        let (ast, accessor) = builder.recursive_build(&rlt.0, source);
        DefaultContext::new(
            FileRelative {
                value: ReportCollector::default(),
                filepath: source.path(),
            },
            accessor,
            builder,
            Rc::new(ast),
        )
    }
}

pub struct Traversing;

impl Traversing {
    pub fn run<'c, C: Context<'c>, T: Traversable<'c, C, UnrecoverableError>>(
        self,
        set: &T,
        context: C,
        source: &ReadCodeSource,
        settings: &mut CodespanSettings,
    ) -> C {
        match set.traverse(context) {
            Ok(c) => c,
            Err((UnrecoverableError::Report(r), c)) => {
                r.emit(settings, source).expect("Cannot emit diagnostics");
                c
            }
            Err((_, c)) => c,
        }
    }
}

pub struct Emitting;

impl Emitting {
    pub fn run(
        self,
        context: DefaultContext,
        source: &ReadCodeSource,
        settings: &mut CodespanSettings,
    ) -> bool {
        let has_errors = context.has_errors();
        context.emit_diagnostics(settings, source);
        has_errors
    }
}
