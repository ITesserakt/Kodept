use crate::lint::{Lint, LintDescriptor};
use crate::report::Reporter;
use crate::{done, fail, Result};
use bevy_ecs::prelude::{Entity, IntoSystem, Res, Single};
use bevy_ecs::query::With;
use kodept_ast::properties::Lexeme;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::Located;
use kodept_diagnostic_macros::Diagnostic;
use kodept_report::message::{Diagnostic, Label, Severity};
use kodept_rlt::prelude::{File, Module};

pub struct SingleModuleWithBrackets;

#[derive(Diagnostic)]
#[severity("warning")]
#[message("Expected {entity} to point at `File` node")]
pub struct SuspiciousStructure {
    entity: Entity,
}

impl Lint for SingleModuleWithBrackets {
    type Error = SuspiciousStructure;

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("single_module_with_brackets")
    }

    fn lint() -> impl IntoSystem<(), Result<Self::Error>, ()> {
        IntoSystem::into_system(
            |query: Single<(&Lexeme, Entity), With<FileDecl>>,
             syntax: Res<SyntaxResolver>,
             reporter: Reporter| {
                let Ok(node) = syntax.try_get::<File>(query.0 .0) else {
                    return fail(SuspiciousStructure { entity: query.1 });
                };

                if let [Module::Ordinary { lbrace, rbrace, .. }] = node.0.as_ref() {
                    reporter.report_ad_hoc(|| {
                        Diagnostic::new(Severity::Warning)
                            .with_message("Consider replacing brackets with single `=>`")
                            .with_label(Label::primary("replace with `=>`", lbrace.location()))
                            .with_label(Label::primary("remove", rbrace.location()))
                    });
                }
                done()
            },
        )
    }
}
