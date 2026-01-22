use crate::per_file::ast_shenanigans::lint::{IntoReadonlySystem, Lint, LintDescriptor};
use crate::source::collection::Reporter;
use bevy_ecs::prelude::*;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_core::structure::Located;
use kodept_report::message::Diagnostic;
use kodept_report::prelude::Severity;

pub(super) struct SingleModuleWithBracketsLint;

impl Lint for SingleModuleWithBracketsLint {
    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("single_module_with_brackets")
    }

    fn lint() -> impl IntoReadonlySystem<(), (), ()> {
        IntoSystem::into_system(system)
    }
}

fn system(syntax: Res<SyntaxResolver>, mut reporter: Reporter) {
    let node = syntax.root().0;

    if let [kodept_rlt::prelude::Module::Ordinary { lbrace, rbrace, .. }] = node.0.as_ref() {
        reporter.report_ad_hoc(|| {
            Diagnostic::new(Severity::Warning)
                .with_message("Consider replacing brackets with a single `=>`")
                .with_primary_label("replace with `=>`", lbrace.location())
                .with_primary_label("remove", rbrace.location())
        });
    }
}
