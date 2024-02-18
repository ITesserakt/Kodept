use kodept_ast::graph::GenericASTNode;
use kodept_ast::visitor::visit_side::VisitGuard;
use kodept_ast::visitor::TraversingResult;
use kodept_core::code_point::CodePoint;

use crate::traits::{Context, UnrecoverableError};

pub trait Analyzer {
    type Error: Into<UnrecoverableError>;
    type Node: TryFrom<GenericASTNode>;

    fn analyze(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> TraversingResult<Self::Error>;
}

pub trait AccessExt {
    fn or_unknown(self) -> Vec<CodePoint>;
}

impl AccessExt for Option<CodePoint> {
    fn or_unknown(self) -> Vec<CodePoint> {
        self.map_or(vec![], |it| vec![it])
    }
}

impl AccessExt for Option<Vec<CodePoint>> {
    fn or_unknown(self) -> Vec<CodePoint> {
        self.unwrap_or_default()
    }
}
