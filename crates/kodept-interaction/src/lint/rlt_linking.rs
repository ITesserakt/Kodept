use crate::lint::{Lint, LintDescriptor};
use crate::report::Reporter;
use crate::{done, Interacted};
use bevy_ecs::prelude::{Entity, IntoSystem, Query, Res, With};
use kodept_ast::properties::Node;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_report::error::report::Severity;
use kodept_report::error::Diagnostic;
use std::convert::Infallible;

pub struct RLTLinkLint;

impl Lint for RLTLinkLint {
    type Error = Infallible;

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("RLT_linking")
    }

    fn lint() -> impl IntoSystem<(), Interacted<Self::Error>, ()> {
        IntoSystem::into_system(Self::check_system)
    }
}

impl RLTLinkLint {
    fn check_system(
        nodes: Query<Entity, With<Node>>,
        syntax: Res<SyntaxResolver>,
        reporter: Reporter,
    ) -> Interacted<Infallible> {
        nodes.par_iter().for_each(|entity| {
            if syntax.get_unknown(entity).is_some() {
                return;
            }
            reporter.report_ad_hoc(|| {
                Diagnostic::new(Severity::Bug)
                    .with_message("Node is not linked with any RLT node")
                    .with_note(format!("Entity: {}", entity))
            });
        });
        done()
    }
}
