use derive_more::From;
use tracing::debug;

use kodept_ast::BodyFnDecl;
use kodept_ast::graph::ChangeSet;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::structure::{Located, rlt};
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Environment;
use kodept_inference::r#type::PolymorphicType;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::Macro;
use kodept_macros::traits::Context;

use crate::convert_model::ExtractName;
use crate::scope::{ScopeError, ScopeTree};
use crate::type_checker::InferError::Unknown;
use crate::Witness;

pub struct CannotInfer(AlgorithmWError);

pub struct TypeInfo<'a> {
    name: &'a str,
    ty: &'a PolymorphicType,
}

impl From<TypeInfo<'_>> for ReportMessage {
    fn from(value: TypeInfo<'_>) -> Self {
        Self::new(
            Severity::Note,
            "TC001",
            format!(
                "Type of function `{}` inferred to: {}",
                value.name, value.ty
            ),
        )
    }
}

impl From<CannotInfer> for ReportMessage {
    fn from(value: CannotInfer) -> Self {
        Self::new(Severity::Warning, "TC002", value.0.to_string())
    }
}

pub struct TypeChecker<'a> {
    pub(crate) symbols: &'a ScopeTree,
    constraints: Environment,
    evidence: Witness,
}

#[derive(From, Debug)]
pub enum InferError {
    AlgorithmW(AlgorithmWError),
    Scope(ScopeError),
    Unknown,
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbols: &'a ScopeTree, evidence: Witness) -> Self {
        Self {
            symbols,
            constraints: Default::default(),
            evidence,
        }
    }

    pub fn into_inner(self) -> Vec<Environment> {
        vec![self.constraints]
    }
}

impl From<InferError> for ReportMessage {
    fn from(value: InferError) -> Self {
        match value {
            InferError::AlgorithmW(x) => Self::new(Severity::Error, "TI001", x.to_string()),
            InferError::Scope(x) => x.into(),
            Unknown => Self::new(Severity::Bug, "TI004", "Bug in implementation".to_string()),
        }
    }
}

impl Macro for TypeChecker<'_> {
    type Error = InferError;
    type Node = BodyFnDecl;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();
        let Some(tree) = context.tree().upgrade() else {
            return Execution::Skipped;
        };
        if matches!(side, VisitSide::Leaf | VisitSide::Exiting) {
            let model = self.to_model(&tree, node.token(), &*node, self.evidence)?;
            debug!(
                "Built compatible model for function {}: {}",
                node.name, model
            );
            let assumptions = &mut self.constraints;
            let fn_location = context
                .access(&*node)
                .map_or(vec![], |it: &rlt::BodiedFunction| vec![it.id.location()]);
            match model.infer(&assumptions) {
                Ok(ty) => {
                    context.add_report(
                        fn_location,
                        TypeInfo {
                            name: &node.name,
                            ty: &ty,
                        },
                    );
                    assumptions.push(node.extract_name(&*tree, node.token()), ty);
                }
                Err(e) => context.add_report(fn_location, CannotInfer(e)),
            }
        }

        Execution::Completed(ChangeSet::new())
    }
}
