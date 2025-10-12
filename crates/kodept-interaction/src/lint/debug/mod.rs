use crate::lint::{IntoReadonlySystem, Lint, LintDescriptor};
use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::typing::Typed;
use bevy_ecs::prelude::{Added, IntoSystem, NameOrEntity, Populated, Query};
use kodept_ast::properties::SourceSpan;
use kodept_report::message::{Diagnostic, Severity};

pub struct DebugScopesLint;

impl Lint for DebugScopesLint {
    type Result = ();

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("debug_scopes")
            .disabled_by_default()
            .run_on_each_pass()
    }

    fn lint() -> impl IntoReadonlySystem<Self::Result> {
        IntoSystem::into_system(
            |scopes: Populated<(&Scope, NameOrEntity), Added<Scope>>,
             spans: Query<&SourceSpan>,
             reporter: Reporter| {
                for (scope, name) in scopes.iter() {
                    reporter.report_ad_hoc(|| {
                        Diagnostic::new(Severity::Note)
                            .with_message(format!("Added new scope `{}`", name))
                            .with_primary_label(
                                "scope coverage",
                                spans.get(scope.start_from.entity()).unwrap().0,
                            )
                            .with_note(format!("{:?}", scope))
                    })
                }
            },
        )
    }
}

pub struct DebugTypingLint;

impl Lint for DebugTypingLint {
    type Result = ();

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("debug_typing")
            .disabled_by_default()
            .run_on_each_pass()
    }

    fn lint() -> impl IntoReadonlySystem<Self::Result> {
        IntoSystem::into_system(
            |query: Populated<(&SourceSpan, &Typed), Added<Typed>>, reporter: Reporter| {
                for (span, ty) in query {
                    reporter.report_ad_hoc(|| {
                        Diagnostic::new(Severity::Note)
                            .with_message("Type resolved")
                            .with_primary_label(format!("{}", ty.0), span.0)
                    })
                }
            },
        )
    }
}
