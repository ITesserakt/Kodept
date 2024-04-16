use crate::node_family::TypeRestrictedNode;
use derive_more::From;
use kodept_ast::graph::{ChangeSet, GenericASTNode};
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_ast::BodiedFunctionDeclaration;
use kodept_core::ConvertibleToRef;
use kodept_inference::algorithm_u::AlgorithmUError;
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Language;
use kodept_inference::Environment;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::traits::Context;
use kodept_macros::Macro;
use std::ops::Deref;
use std::rc::Rc;
use tracing::debug;

use crate::scope::{ScopeError, ScopeTree};
use crate::type_checker::InferError::Unknown;

pub struct TypeChecker {
    pub(crate) symbols: Rc<ScopeTree>,
    env: Environment,
    constraints: Vec<Assumptions>,
}

#[derive(From, Debug)]
pub enum InferError {
    AlgorithmW(AlgorithmWError),
    Scope(ScopeError),
    Unknown,
}

impl TypeChecker {
    pub fn new(symbols: Rc<ScopeTree>) -> Self {
        Self {
            symbols,
            env: Default::default(),
            constraints: Default::default(),
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
            Unknown => Self::new(Severity::Bug, "TI004", "Bug in implementation".to_string()),
        }
    }
}

impl Macro for TypeChecker {
    type Error = InferError;
    type Node = GenericASTNode;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        let Some(tree) = context.tree().upgrade() else {
            return Execution::Skipped;
        };
        if let Some(restricted) = node.deref().try_as_ref() {
            if matches!(side, VisitSide::Leaf | VisitSide::Entering) {
                let restricted: &TypeRestrictedNode = restricted;
                let current_a = self.constraints.pop().unwrap_or_default();
                let current_a = current_a.merge(
                    restricted
                        .type_of(&tree, node.token(), &self.symbols)
                        .map_err(|_| Unknown)?,
                );
                self.constraints.push(current_a);
            }
        } else if let Some(fnc) = node.deref().try_as_ref() {
            if side == VisitSide::Entering {
                self.constraints.push(Assumptions::empty());
            }
            if matches!(side, VisitSide::Leaf | VisitSide::Exiting) {
                let fnc: &BodiedFunctionDeclaration = fnc;
                let model = Rc::new(self.to_model(&tree, node.token(), fnc)?);
                let mut assumptions = if side == VisitSide::Leaf {
                    Assumptions::empty()
                } else {
                    self.constraints.pop().unwrap_or_default()
                };
                Language::infer_with_env(model.clone(), &mut assumptions, &mut self.env)?;
                debug!("{assumptions}\n");
            }
        } else {
            return Execution::Skipped;
        }

        Execution::Completed(ChangeSet::new())
    }
}
