use std::ops::Deref;
use itertools::Itertools;
use nonempty_collections::{nev, NEVec};

use crate::scope::ScopeSearch;
use crate::type_checker::InferError;
use kodept_ast::graph::SyntaxTree;
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_ast::traits::{AsEnum, Identifiable};
use kodept_ast::*;
use kodept_core::structure::Located;
use kodept_inference::algorithm_w::AlgorithmWError::UnknownVar;
use kodept_inference::language::var;
use kodept_inference::r#type::{fun, unit_type, MonomorphicType, Tuple};
use kodept_macros::error::traits::SpannedError;
use kodept_macros::execution::Execution;
use kodept_macros::execution::Execution::{Completed, Skipped};
use Identifier::TypeReference;

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    pub enum TypeDerivableNode {
        Function(BodyFnDecl),
        ExpressionBlock(Exprs),
        InitVar(InitVar),
        Lambda(Lambda),
        Application(Appl),
        IfExpr(IfExpr),
        Reference(Ref),
        Literal(forward Lit)
    }
}

node_sub_enum! {
    pub enum TypeRestrictedNode {
        TypedParameter(TyParam),
        Function(AbstFnDecl),
        Variable(VarDecl),
        Reference(Ref)
    }
}

impl TypeRestrictedNode {
    pub fn type_of(
        &self,
        scopes: &ScopeSearch,
        tree: &SyntaxTree,
        rlt: &RLTAccessor,
    ) -> Execution<SpannedError<InferError>, MonomorphicType> {
        match self.as_enum() {
            TypeRestrictedNodeEnum::Reference(Ref {
                ident: TypeReference { name },
                ..
            }) => {
                let location = rlt.get_unknown(self.get_id()).unwrap().location();
                let ty = scopes.ty(name).ok_or(SpannedError::new(
                    InferError::AlgorithmW(UnknownVar(nev![var(name.deref())])),
                    location,
                ))?;
                Completed(ty)
            }
            TypeRestrictedNodeEnum::Variable(node) => {
                if let Some(ty) = node.assigned_type(tree) {
                    Completed(convert(ty, scopes, tree, rlt)?)
                } else {
                    Skipped
                }
            }
            TypeRestrictedNodeEnum::TypedParameter(x) => {
                Completed(convert(x.parameter_type(tree), scopes, tree, rlt)?)
            }
            TypeRestrictedNodeEnum::Function(node) => {
                let params = node.parameters(tree);
                let return_type = node.return_type(tree);

                let ret = match return_type {
                    None => unit_type(),
                    Some(ty) => convert(ty, scopes, tree, rlt)?,
                };
                let ps = params
                    .into_iter()
                    .map(|it| convert(it.parameter_type(tree), scopes, tree, rlt))
                    .try_collect()?;
                match NEVec::from_vec(ps) {
                    None => Completed(ret),
                    Some(ps) => Completed(fun(ps, ret)),
                }
            }
            _ => Skipped,
        }
    }
}

pub(crate) fn convert(
    ty: &Type,
    scope: &ScopeSearch,
    ast: &SyntaxTree,
    rlt: &RLTAccessor,
) -> Result<MonomorphicType, SpannedError<InferError>> {
    match ty.as_enum() {
        TypeEnum::TyName(constant) => {
            let location = rlt.get_unknown(ty.get_id()).unwrap().location();
            scope.ty(&constant.name).ok_or(SpannedError::new(
                InferError::AlgorithmW(UnknownVar(nev![var(constant.name.deref())])),
                location,
            ))
        }
        TypeEnum::Tuple(tuple) => {
            let types: Result<Vec<_>, _> = tuple
                .types(ast)
                .into_iter()
                .map(|it| match convert(it, scope, ast, rlt) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(e),
                })
                .collect();
            let types = types?;
            Ok(MonomorphicType::Tuple(Tuple::new(types)))
        }
    }
}
