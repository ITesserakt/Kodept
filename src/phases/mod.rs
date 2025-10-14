pub mod load_all_sources {
    use crate::cli::configs::LoadingConfig;
    use bevy_ecs::prelude::*;
    use kodept::loader::{Loader, LoadingError};
    use kodept::source::{SourcesLoadingError, load_each_source};
    use kodept_frontend::Either;
    use kodept_frontend::engine::{Engine, Phase, SubEngine};
    use kodept_frontend::prelude::CollectedSources;
    use std::sync::Arc;
    use kodept::source::collection::SystemExt;

    #[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct LoadAllSourcesPhaseSystems;

    pub struct LoadAllSourcesPhase {
        pub config: LoadingConfig,
    }

    impl Phase for LoadAllSourcesPhase {
        type Set = LoadAllSourcesPhaseSystems;

        fn build(self, engine: &mut Engine) {
            engine.add_systems(
                system
                    .with_input(self.config)
                    .report_errors()
                    .in_set(LoadAllSourcesPhaseSystems),
            );
        }
    }

    fn system(
        InMut(config): InMut<LoadingConfig>,
        mut commands: Commands,
    ) -> Result<(), Either<LoadingError, SourcesLoadingError>> {
        let loader = Loader::try_from(&*config).map_err(Either::Left)?;
        let sources = load_each_source(loader).map_err(Either::Right)?;
        let sources = Arc::new(sources);
        let views = sources.collect();

        commands.insert_resource(CollectedSources { inner: sources });

        commands.spawn_batch(views.into_iter().map(|it| (it, SubEngine::new())));

        Ok(())
    }
}

pub mod each_sub_engine {
    use bevy_ecs::prelude::*;
    use kodept::source::collection::SourceView;
    use kodept_frontend::engine::{Engine, Phase, SubEngine};
    use std::fmt::{Debug, Formatter};
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;

    #[derive(SystemSet)]
    pub struct EachSubEnginePhaseSystems<F>(PhantomData<fn() -> F>);

    pub struct EachSubEnginePhase<F>(F);

    impl<F> EachSubEnginePhase<F> {
        pub fn new(configuration: F) -> Self
        where
            F: Fn(&mut Engine) + Send + Sync + 'static,
        {
            EachSubEnginePhase(configuration)
        }
    }

    impl<F: 'static> Phase for EachSubEnginePhase<F>
    where
        F: Fn(&mut Engine) + Send + Sync,
    {
        type Set = EachSubEnginePhaseSystems<F>;

        fn build(self, engine: &mut Engine) {
            engine.add_systems(
                system
                    .with_input(self.0)
                    .in_set(EachSubEnginePhaseSystems::<F>::default()),
            )
        }
    }

    fn system<F>(
        InMut(configuration): InMut<F>,
        mut sub_engines: Query<(&SourceView, &mut SubEngine)>,
    ) where
        F: Fn(&mut Engine) + Send + Sync + 'static,
    {
        sub_engines.par_iter_mut().for_each(|(source, mut engine)| {
            // Update source
            engine.insert_resource(source.clone());
            // Configure engine
            configuration(&mut *engine);
            // Run it!
            engine.run();
        });
    }

    impl<F> Debug for EachSubEnginePhaseSystems<F> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("EachSubEnginePhaseSystems").finish()
        }
    }

    impl<F> Clone for EachSubEnginePhaseSystems<F> {
        fn clone(&self) -> Self {
            Self(PhantomData)
        }
    }

    impl<F> Copy for EachSubEnginePhaseSystems<F> {}

    impl<F> PartialEq for EachSubEnginePhaseSystems<F> {
        fn eq(&self, _: &Self) -> bool {
            true
        }
    }

    impl<F> Hash for EachSubEnginePhaseSystems<F> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    impl<F> Default for EachSubEnginePhaseSystems<F> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<F> Eq for EachSubEnginePhaseSystems<F> {}
}

pub mod parse_source {
    use crate::cli::configs::{LexerImpl, ParserImpl, ParsingConfig};
    use bevy_ecs::prelude::*;
    use kodept::source::collection::{SourceView, SystemExt};
    use kodept_ast::resource::rlt::SyntaxResolver;
    use kodept_frontend::Either;
    use kodept_frontend::engine::{Engine, Phase};
    use kodept_parse::common::{ErrorAdapter, RLTProducer};
    use kodept_parse::error::{ParseError, ParseErrors};
    use kodept_parse::lexer::PackedToken;
    use kodept_parse::token_stream::PackedTokenStream;
    use kodept_parse::tokenizer::{EagerTokenizer, Tok, TokCtor};
    use kodept_report::prelude::*;
    use std::borrow::Cow;
    use std::fmt::Display;

    #[derive(Debug, SystemSet, Copy, Clone, PartialEq, Eq, Hash, Default)]
    pub struct ParseSourcePhaseLabel;

    pub struct ParseSourcePhase {
        pub config: ParsingConfig,
    }

    impl Phase for ParseSourcePhase {
        type Set = ParseSourcePhaseLabel;

        fn build(self, engine: &mut Engine) {
            engine.add_systems(
                system
                    .with_input(self.config)
                    .report_errors()
                    .in_set(ParseSourcePhaseLabel),
            )
        }
    }

    fn system(
        InMut(config): InMut<ParsingConfig>,
        source: Res<SourceView>,
        mut commands: Commands,
    ) -> Result<
        (),
        Either<
            impl Iterator<Item = Wrapper<ParseError<String>>> + use<>,
            impl Iterator<Item = Wrapper<ParseError<PackedToken>>> + use<>,
        >,
    > {
        let input = source.contents();
        let lexer = config.get_lexing_backend(input);

        let tokens = match lexer {
            LexerImpl::Peg(x) => EagerTokenizer::new(input, x)
                .try_into_vec()
                .map_err(|e| e.adapt(input, 0)),
            LexerImpl::ASCII(x) => EagerTokenizer::new(input, x)
                .try_into_vec()
                .map_err(|e| match e {}),
        }
        .map_err(|e: ParseErrors<String>| e.into_iter().map(Wrapper))
        .map_err(Either::Left)?;
        let stream = PackedTokenStream::new(&tokens);

        let parser = config.get_parsing_backend();
        let rlt = match parser {
            ParserImpl::Peg(x) => x.parse_stream(&stream).map_err(|e| e.adapt(stream, 0)),
        }
        .map_err(|e| e.into_iter().map(Wrapper))
        .map_err(Either::Right)?;

        commands.insert_resource(SyntaxResolver::build(rlt));

        Ok(())
    }

    struct Wrapper<T>(T);

    impl<A: Display> IntoSpannedReportMessage for Wrapper<ParseError<A>> {
        type Message = Diagnostic;

        fn behaviour(&self) -> MessageBehaviour {
            MessageBehaviour::fail_fast("Error while parsing")
        }

        fn into_message(self) -> Self::Message {
            let (expected, actual, location, hints) = match self.0 {
                ParseError::ExpectedInstead {
                    expected,
                    actual,
                    location,
                    hints,
                } => (expected, Some(actual), location, hints),
                ParseError::ExpectedNotEOF {
                    expected,
                    location,
                    hints,
                } => (expected, None, location, hints),
            };

            let diagnostic = if expected.is_empty() {
                let actual = actual
                    .map(|it| Cow::Owned(it.to_string()))
                    .unwrap_or(Cow::Borrowed("EOF"));

                Diagnostic::new(Severity::Error)
                    .with_message(format!("Unexpected {actual}"))
                    .with_primary_label("here", location.in_code)
            } else if let Some(actual) = actual {
                let exp_msg = expected_to_string(expected);

                Diagnostic::new(Severity::Error)
                    .with_message(format!("Expected {exp_msg}, got {actual}"))
                    .with_primary_label("here", location.in_code)
            } else {
                let exp_msg = expected_to_string(expected);

                Diagnostic::new(Severity::Error)
                    .with_message(format!("Expected {exp_msg} after, got EOF"))
                    .with_primary_label("here", location.in_code)
            };

            hints
                .into_iter()
                .fold(diagnostic, |acc, next| acc.with_note(next))
        }
    }

    fn expected_to_string(mut expected: Vec<Cow<'static, str>>) -> Cow<'static, str> {
        let Some(last_expected) = expected.pop() else {
            return Cow::Borrowed("");
        };

        use std::fmt::Write;
        if let Some(last) = expected.pop() {
            let mut result = String::new();
            for item in expected {
                _ = write!(result, "{item}, ");
            }
            _ = write!(result, "{last} or {last_expected}");
            result.into()
        } else {
            last_expected
        }
    }
}

pub mod build_ast {
    use bevy_ecs::prelude::*;
    use derive_more::From;
    use kodept::source::collection::{SourceView, SystemExt};
    use kodept_ast::prelude::FromSyntax;
    use kodept_ast::resource::rlt::SyntaxResolver;
    use kodept_ast_nodes::Error;
    use kodept_ast_nodes::file::FileDecl;
    use kodept_core::structure::CodeHolder;
    use kodept_frontend::define_phase;
    use kodept_frontend::engine::Engine;
    use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, Severity};
    use std::borrow::Cow;

    define_phase!(
        pub phase BuildAstPhase[BuildAstPhaseLabel];

        fn build (self, engine: &mut Engine) {
            engine.add_systems(
                system.report_errors().in_set(BuildAstPhaseLabel)
            );
        }
    );

    fn system(
        source: Res<SourceView>,
        syntax: Res<SyntaxResolver>,
        mut commands: Commands,
    ) -> Result<(), Wrapper> {
        let code_holder = source.map(|it| Cow::Owned(it.to_string()));

        let (root, root_id) = syntax.root();
        let whole_bundle = FileDecl::from_syntax(root, code_holder)?;

        let mut entity = commands.spawn((
            whole_bundle,
            kodept_ast::properties::Root {
                associated_file: source.describe(),
            },
        ));
        entity.insert(kodept_ast::properties::Lexeme(root_id));

        Ok(())
    }

    #[derive(Debug, From)]
    struct Wrapper(Error);

    impl IntoSpannedReportMessage for Wrapper {
        type Message = Diagnostic;

        fn into_message(self) -> Self::Message {
            let diagnostic = Diagnostic::new(Severity::Bug);
            match self.0 {
                Error::NoQuotesInLiteral(point) => diagnostic
                    .with_message("String or char literals must contain quotes")
                    .with_primary_label("no quotes", point),
                Error::WrongLiteralLength(point, len) => diagnostic
                    .with_message(format!("Literal must have length at least `{}`", len))
                    .with_primary_label("wrong length", point),
                Error::CannotParseFloat(point, e) => diagnostic
                    .with_message(format!("Cannot parse floating literal: {}", e))
                    .with_primary_label("cannot parse floating literal", point),
                Error::CannotParseInt(point, e) => diagnostic
                    .with_message(format!("Cannot parse integer literal: {}", e))
                    .with_primary_label("cannot parse integer literal", point),
            }
        }
    }
}
