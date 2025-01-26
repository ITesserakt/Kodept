use crate::lint::{Lint, LintDescriptor};
use crate::report::Reporter;
use crate::{done, skip, Result};
use bevy_ecs::prelude::{Entity, IntoSystem, Res, Single};
use bevy_ecs::query::With;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::Located;
use kodept_report::error::report::{Label, Severity};
use kodept_report::error::Diagnostic;
use kodept_rlt::prelude::{File, Module};
use std::convert::Infallible;

pub struct SingleModuleWithBrackets;

impl Lint for SingleModuleWithBrackets {
    type Error = Infallible;

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("single_module_with_brackets")
    }

    fn lint() -> impl IntoSystem<(), Result<Self::Error>, ()> {
        IntoSystem::into_system(
            |query: Option<Single<Entity, With<FileDecl>>>,
             syntax: Res<SyntaxResolver>,
             reporter: Reporter| {
                let Some(root) = query else { return skip() };
                let Ok(node): std::result::Result<&File, _> = syntax.get(*root) else {
                    return skip();
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
