use crate::source::collection::SourceView;
use crate::utils::{LogSystemEx, ReportSystemEx};
use bevy_ecs::prelude::*;
use derive_more::From;
use kodept_ast::arity::Plural;
use kodept_ast::experimental::FromSyntax;
use kodept_ast::prelude::NodeId;
use kodept_ast::properties::Root;
use kodept_ast::relationship::ContainedBy;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::experimental::GenericSpawnContext;
use kodept_ast_nodes::Error;
use kodept_ast_nodes::{Module, Modules};
use kodept_core::structure::CodeHolder;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, Severity};
use std::borrow::Cow;

define_phase!(
    pub phase BuildAstPhase[BuildAstPhaseLabel];

    fn build (self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(system.extract_reports().trace_completion());
    }
);

fn system(
    source: Res<SourceView>,
    syntax: Res<SyntaxResolver>,
    mut commands: Commands,
) -> Result<(), Wrapper> {
    let code_holder = source.map(|it| Cow::Owned(it.to_string()));

    let (root, _) = syntax.root();
    let root_id = commands
        .spawn((
            Root {
                associated_file: source.describe(),
            },
            Modules,
        ))
        .id();

    for module in &root.0 {
        let module_id: NodeId<Module> =
            Module::from_syntax(module, GenericSpawnContext::new(&mut commands), code_holder)?;
        commands
            .entity(root_id)
            .add_one_related::<ContainedBy<(), Plural>>(module_id.entity());
    }

    Ok(())
}

#[derive(Debug, From)]
struct Wrapper(Error);

impl IntoSpannedReportMessage for Wrapper {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        match self.0 {
            Error::NoQuotesInLiteral(point) => Diagnostic::new(Severity::Bug)
                .with_message("String or char literals must contain quotes")
                .with_primary_label("no quotes", point),
            Error::WrongLiteralLength(point, len) => Diagnostic::new(Severity::Bug)
                .with_message(format!("Literal must have length at least `{}`", len))
                .with_primary_label("wrong length", point),
            Error::CannotParseFloat(point, e) => Diagnostic::new(Severity::Bug)
                .with_message(format!("Cannot parse floating literal: {}", e))
                .with_primary_label("cannot parse floating literal", point),
            Error::CannotParseInt(point, e) => Diagnostic::new(Severity::Bug)
                .with_message(format!("Cannot parse integer literal: {}", e))
                .with_primary_label("cannot parse integer literal", point),
            Error::Unsupported(span) => Diagnostic::new(Severity::Bug)
                .with_message("Syntax is unsupported")
                .with_primary_label("unsupported", span),
            Error::UnexpectedStatement(span) => Diagnostic::new(Severity::Error)
                .with_message("Statement in this position is unexpected")
                .with_primary_label("expected expression", span)
                .with_note("Try wrapping this statement in block"),
            Error::UnexpectedExpression(span) => Diagnostic::new(Severity::Error)
                .with_message("Expression in this position is unexpected")
                .with_primary_label("expected statement", span)
                .with_note("Try calling or linking this expression"),
        }
    }
}
