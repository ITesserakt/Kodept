use crate::per_file::ast_shenanigans::lint::{IntoReadonlySystem, Lint, LintDescriptor};
use crate::source::collection::Reporter;
use bevy_ecs::prelude::*;
use kodept_ast::properties::Lexeme;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::Located;
use kodept_report::message::Diagnostic;
use kodept_report::prelude::{IntoSpannedReportMessage, Severity};

pub(super) struct SingleModuleWithBracketsLint;

struct SuspiciousStructure(Entity);

impl Lint for SingleModuleWithBracketsLint {
    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("single_module_with_brackets")
    }

    fn lint() -> impl IntoReadonlySystem<(), (), ()> {
        IntoSystem::into_system(system)
    }
}

fn system(
    query: Query<(Entity, &Lexeme), With<FileDecl>>,
    syntax: Res<SyntaxResolver>,
    mut reporter: Reporter,
) {
    query.iter().for_each(|(id, lexeme)| {
        let Ok(node) = syntax.try_get::<kodept_rlt::prelude::File>(lexeme.0) else {
            reporter.report(SuspiciousStructure(id));
            return;
        };

        if let [kodept_rlt::prelude::Module::Ordinary { lbrace, rbrace, .. }] = node.0.as_ref() {
            reporter.report_ad_hoc(|| {
                Diagnostic::new(Severity::Warning)
                    .with_message("Consider replacing brackets with single `=>`")
                    .with_primary_label("replace with `=>`", lbrace.location())
                    .with_primary_label("remove", rbrace.location())
            });
        }
    })
}

impl IntoSpannedReportMessage for SuspiciousStructure {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Bug)
            .with_message(format!("Expected {} to point at File node", self.0))
    }
}
