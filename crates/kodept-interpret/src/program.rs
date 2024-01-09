use std::convert::Infallible;

use kodept_ast::graph::GenericASTNode;
use kodept_ast::visitor::visit_side::VisitGuard;
use kodept_ast::visitor::TraversingResult;
use kodept_core::Named;
use kodept_macros::analyzer::Analyzer;
use kodept_macros::traits::Context;

use crate::scope::ScopedSymbolTable;

pub struct SemanticAnalyzer {
    current_scope: ScopedSymbolTable,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self {
            current_scope: ScopedSymbolTable::new("global", None),
        }
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Named for SemanticAnalyzer {}

impl Analyzer for SemanticAnalyzer {
    type Error = Infallible;
    type Node = GenericASTNode;

    fn analyze<'c, C: Context<'c>>(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut C,
    ) -> TraversingResult<Self::Error> {
        let (node, side) = guard.allow_all();

        Ok(())
    }
}
