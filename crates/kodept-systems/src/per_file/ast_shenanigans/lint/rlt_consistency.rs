use crate::lint::{IntoReadonlySystem, Lint, LintDescriptor};
use crate::source::collection::Reporter;
use bevy_ecs::prelude::*;
use kodept_ast::properties::{Lexeme, Node};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_report::prelude::{Diagnostic, Severity};

pub(super) struct RLTConsistencyLint;

impl Lint for RLTConsistencyLint {
    fn descriptor() -> LintDescriptor {
        LintDescriptor::new("check_rlt_linking_consistency")
    }

    fn lint() -> impl IntoReadonlySystem<(), (), ()> {
        IntoSystem::into_system(system)
    }
}

fn system(
    nodes: Populated<(Entity, &Node, Option<&Lexeme>), Changed<Lexeme>>,
    syntax: If<Res<SyntaxResolver>>,
    mut reporter: Reporter,
) {
    // TODO: Use `par_iter`
    nodes.iter().for_each(|(entity, node, lexeme)| {
        if lexeme.is_some_and(|it| syntax.try_get_unknown(it.0).is_some()) {
            return;
        }
        reporter.report_ad_hoc(|| {
            Diagnostic::new(Severity::Bug)
                .with_message("AST node is not linked with any RLT node")
                .with_note(format!("Entity: {}, kind: {}", entity, node.kind))
        })
    });
}
