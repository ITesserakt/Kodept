use itertools::Itertools;
use nonempty_collections::{nev, NEVec};

use Identifier::TypeReference;
use kodept_ast::*;
use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_ast::traits::AsEnum;
use kodept_ast::utils::Execution;
use kodept_ast::utils::Execution::{Completed, Skipped};
use kodept_inference::algorithm_w::AlgorithmWError::UnknownVar;
use kodept_inference::language::var;
use kodept_inference::r#type::{fun, MonomorphicType, Tuple, unit_type};

use crate::scope::ScopeSearch;
use crate::type_checker::InferError;

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
        token: &PermTkn,
    ) -> Execution<InferError, MonomorphicType> {
        match self.as_enum() {
            TypeRestrictedNodeEnum::Reference(Ref {
                ident: TypeReference { name },
                ..
            }) => {
                let ty = scopes.ty(name).ok_or(UnknownVar(nev![var(name)]))?;
                Completed(ty)
            }
            TypeRestrictedNodeEnum::Variable(node) => {
                if let Some(ty) = node.assigned_type(tree, token) {
                    Completed(convert(ty, scopes, tree, token)?)
                } else {
                    Skipped
                }
            }
            TypeRestrictedNodeEnum::TypedParameter(x) => {
                Completed(convert(x.parameter_type(tree, token), scopes, tree, token)?)
            }
            TypeRestrictedNodeEnum::Function(node) => {
                let params = node.parameters(tree, token);
                let return_type = node.return_type(tree, token);

                let ret = match return_type {
                    None => unit_type(),
                    Some(ty) => convert(ty, scopes, tree, token)?,
                };
                let ps = params
                    .into_iter()
                    .map(|it| convert(it.parameter_type(tree, token), scopes, tree, token))
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
    token: &PermTkn,
) -> Result<MonomorphicType, InferError> {
    return match ty.as_enum() {
        TypeEnum::TyName(constant) => {
            scope
                .ty(&constant.name)
                .ok_or(InferError::AlgorithmW(UnknownVar(nev![
                    var(&constant.name)
                ])))
        }
        TypeEnum::Tuple(tuple) => {
            let types: Result<Vec<_>, _> = tuple
                .types(ast, token)
                .into_iter()
                .map(|it| match convert(it, scope, ast, token) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(e),
                })
                .collect();
            let types = types?;
            Ok(MonomorphicType::Tuple(Tuple::new(types)))
        }
    };
}
