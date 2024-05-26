use derive_more::From;

use kodept_ast::{AbstFnDecl, Appl, BodyFnDecl, Exprs, IfExpr, InitVar, Lambda, Lit, node_sub_enum, Ref, TyParam, Type, TypeEnum, VarDecl};
use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_ast::traits::AsEnum;
use kodept_inference::r#type::{MonomorphicType, Tuple};

use crate::node_family::Errors::Undefined;
use crate::scope::{ScopeError, ScopeSearch};

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

#[derive(From)]
pub enum Errors {
    #[from(ignore)]
    TooComplex,
    #[from(ignore)]
    Undefined(String),
    Scope(ScopeError),
}

pub(crate) fn convert(
    ty: &Type,
    scope: ScopeSearch,
    ast: &SyntaxTree,
    token: &PermTkn,
) -> Result<MonomorphicType, Errors> {
    return match ty.as_enum() {
        TypeEnum::TyName(constant) => scope
            .ty(&constant.name)
            .ok_or(Undefined(constant.name.clone())),
        TypeEnum::Tuple(tuple) => {
            let types: Result<Vec<_>, _> = tuple
                .types(ast, token)
                .into_iter()
                .map(|it| match convert(it, scope.clone(), ast, token) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(e),
                })
                .collect();
            let types = types?;
            Ok(MonomorphicType::Tuple(Tuple::new(types)).into())
        }
    };
}
