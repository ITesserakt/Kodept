use bevy_ecs::prelude::*;
use derive_more::From;
use kodept::source::collection::SourceView;
use kodept::utils::{ReportSystemEx};
use kodept_ast::prelude::FromSyntax;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast_nodes::Error;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::CodeHolder;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, Severity};
use std::borrow::Cow;

define_phase!(
    pub phase BuildAstPhase[BuildAstPhaseLabel];

    fn build (self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(
            system.report_errors()
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
