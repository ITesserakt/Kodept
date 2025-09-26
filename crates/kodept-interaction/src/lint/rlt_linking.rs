use crate::lint::{Lint, LintDescriptor};
use crate::report::Reporter;
use bevy_ecs::prelude::{Entity, IntoSystem, Query, Res};
use kodept_ast::properties::{Lexeme, Node};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_report::message::{Diagnostic, Severity};

pub struct RLTLinkLint;

impl Lint for RLTLinkLint {
    type Result = ();

    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("RLT_linking").run_on_each_pass()
    }

    fn lint() -> impl IntoSystem<(), Self::Result, ()> {
        IntoSystem::into_system(Self::check_system)
    }
}

impl RLTLinkLint {
    fn check_system(
        nodes: Query<(Entity, &Node, Option<&Lexeme>)>,
        syntax: Res<SyntaxResolver>,
        reporter: Reporter,
    ) {
        nodes.iter().for_each(|(entity, kind, lexeme)| {
            if lexeme.is_some_and(|it| syntax.try_get_unknown(it.0).is_some()) {
                return;
            }
            reporter.report_ad_hoc(|| {
                Diagnostic::new(Severity::Bug)
                    .with_message("Node is not linked with any other RLT nodes")
                    .with_note(format!("Entity: {}; kind: {}", entity, kind))
            });
        });
    }
}
