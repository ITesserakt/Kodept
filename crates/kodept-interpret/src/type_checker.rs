use std::rc::Rc;

use derive_more::From;

use kodept_ast::BodyFnDecl;
use kodept_ast::graph::ChangeSet;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_core::structure::{Located, rlt};
use kodept_inference::algorithm_u::AlgorithmUError;
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::assumption::Assumptions;
use kodept_inference::Environment;
use kodept_inference::language::{Language, var};
use kodept_inference::r#type::PolymorphicType;
use kodept_macros::error::report::{ReportMessage, Severity};
use kodept_macros::Macro;
use kodept_macros::traits::Context;

use crate::scope::{ScopeError, ScopeTree};
use crate::type_checker::InferError::Unknown;
use crate::Witness;

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

pub struct TypeChecker<'a> {
    pub(crate) symbols: &'a ScopeTree,
    env: Environment,
    constraints: Assumptions,
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
            env: Default::default(),
            constraints: Default::default(),
            evidence,
        }
    }

    pub fn into_inner(self) -> Vec<Assumptions> {
        vec![self.constraints]
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
                Self::new(Severity::Error, "TI001", format!("`{name}` is not defined"))
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

impl Macro for TypeChecker<'_> {
    type Error = InferError;
    type Node = BodyFnDecl;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (fnc, side) = guard.allow_all();
        let Some(tree) = context.tree().upgrade() else {
            return Execution::Skipped;
        };

        if matches!(side, VisitSide::Leaf | VisitSide::Exiting) {
            let model = Rc::new(self.to_model(&tree, fnc.token(), &*fnc, self.evidence)?);
            let mut assumptions = self.constraints.clone();
            Language::infer_with_env(model.clone(), &mut assumptions, &mut self.env)?;
            let self_type = assumptions.get(&model).expect("No assumption found");
            self.constraints.push(Rc::new(var(&fnc.name).into()), Rc::new(self_type.clone()));
            context.add_report(
                context
                    .access(&*fnc)
                    .map_or(vec![], |it: &rlt::BodiedFunction| vec![it.id.location()]),
                TypeInfo {
                    name: &fnc.name,
                    ty: self_type,
                },
            );
        }

        Execution::Completed(ChangeSet::new())
    }
}
