use crate::lint::{Lint, LintDescriptor};
use crate::report::Reporter;
use bevy_ecs::prelude::{Entity, IntoSystem, Res};
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use kodept_ast::properties::Lexeme;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::Located;
use kodept_report::message::{Diagnostic, Severity};
use kodept_report::prelude::{IntoSpannedReportMessage, ReportMessage};
use kodept_rlt::prelude::{File, Module};

pub struct SingleModuleWithBrackets;

pub struct SuspiciousStructure(Entity);

impl IntoSpannedReportMessage for SuspiciousStructure {
    type Message = ReportMessage;

    fn into_message(self) -> Self::Message {
        ReportMessage::new(
            Severity::Warning,
            format!("Expected {} to point at File node", self.0),
        )
    }
}

impl Lint for SingleModuleWithBrackets {
    type Result = Result<(), SuspiciousStructure>;

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("single_module_with_brackets")
    }

    fn lint() -> impl IntoSystem<(), Self::Result, ()> {
        IntoSystem::into_system(
            |query: Query<(&Lexeme, Entity), With<FileDecl>>,
             syntax: Res<SyntaxResolver>,
             reporter: Reporter| {
                query.into_iter().try_for_each(|(lexeme, entity)| {
                    let Ok(node) = syntax.try_get::<File>(lexeme.0) else {
                        return Err(SuspiciousStructure(entity));
                    };

                    if let [Module::Ordinary { lbrace, rbrace, .. }] = node.0.as_ref() {
                        reporter.report_ad_hoc(|| {
                            Diagnostic::new(Severity::Warning)
                                .with_message("Consider replacing brackets with single `=>`")
                                .with_primary_label("replace with `=>`", lbrace.location())
                                .with_primary_label("remove", rbrace.location())
                        });
                    }
                    Ok(())
                })
            },
        )
    }
}
