use derive_more::From;
use kodept_ast::graph::ChangeSet;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_ast::BodiedFunctionDeclaration;
use kodept_inference::algorithm_u::AlgorithmUError;
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Assumptions;
use kodept_inference::Environment;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::traits::Context;
use kodept_macros::Macro;
use tracing::debug;

use crate::scope::{ScopeError, ScopeTree};

pub struct TypeChecker {
    pub(crate) symbols: ScopeTree,
    env: Environment,
}

#[derive(From, Debug)]
pub enum InferError {
    AlgorithmW(AlgorithmWError),
    Scope(ScopeError),
    Unknown,
}

impl TypeChecker {
    pub fn new(symbols: ScopeTree) -> Self {
        Self {
            symbols,
            env: Default::default(),
        }
    }
}

impl From<InferError> for ReportMessage {
    fn from(value: InferError) -> Self {
        match value {
            InferError::AlgorithmW(AlgorithmWError::AlgorithmU(AlgorithmUError::CannotUnify {
                from,
                to,
            })) => Self::new(
                Severity::Error,
                "TI002",
                format!("Expected to have type `{from}`, but have type `{to}``"),
            ),
            InferError::AlgorithmW(AlgorithmWError::UnknownVar(name)) => {
                Self::new(Severity::Bug, "TI001", format!("`{name}` is not defined"))
            }
            InferError::AlgorithmW(AlgorithmWError::AlgorithmU(
                AlgorithmUError::InfiniteType { type_var, with },
            )) => Self::new(
                Severity::Error,
                "TI003",
                format!("Infinite type detected: `{type_var}` ~ `{with}`"),
            ),
            InferError::Scope(x) => x.into(),
            InferError::Unknown => {
                Self::new(Severity::Bug, "TI004", "Bug in implementation".to_string())
            }
        }
    }
}

impl Macro for TypeChecker {
    type Error = InferError;
    type Node = BodiedFunctionDeclaration;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        if !matches!(side, VisitSide::Exiting | VisitSide::Leaf) {
            return Execution::Skipped;
        }
        let Some(tree) = context.tree().upgrade() else {
            return Execution::Skipped;
        };

        let model = self.to_model(&tree, node.token(), &*node)?;
        let mut assumptions = Assumptions::empty();
        let ty = model.infer_with_env(&mut assumptions, &mut self.env)?;
        debug!("{} => {model} => {ty}", node.name);

        Execution::Completed(ChangeSet::new())
    }
}
