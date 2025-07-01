use crate::done;
use crate::lint::{Lint, LintDescriptor};
use crate::report::Reporter;
use crate::scope::storage::Scope;
use bevy_ecs::prelude::{Added, IntoSystem, NameOrEntity, Populated, Query};
use kodept_ast::properties::SourceSpan;
use kodept_report::message::{Diagnostic, Severity};
use kodept_report::prelude::Label;
use std::convert::Infallible;

pub struct DebugScopesLint;

impl Lint for DebugScopesLint {
    type Error = Infallible;

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("debug_scopes").disabled_by_default()
    }

    fn lint() -> impl IntoSystem<(), crate::Result<Self::Error>, ()> {
        IntoSystem::into_system(
            |scopes: Populated<(&Scope, NameOrEntity), Added<Scope>>,
             spans: Query<&SourceSpan>,
             reporter: Reporter| {
                for (scope, name) in scopes.iter() {
                    reporter.report_ad_hoc(|| {
                        Diagnostic::new(Severity::Note)
                            .with_message(format!("Added new scope `{}`", name))
                            .with_label(Label::primary(
                                "scope coverage",
                                spans.get(scope.start_from.entity()).unwrap().0,
                            ))
                            .with_note(format!("{:?}", scope))
                    })
                }

                done()
            },
        )
    }
}
