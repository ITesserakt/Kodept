use std::rc::Rc;

use derive_more::{From};

use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_ast::Identifier::TypeReference;
use kodept_ast::{AbstFnDecl, Appl, BodyFnDecl, Exprs, IfExpr, InitVar, Lambda, Lit, Ref, Type, TypeEnum, TyParam, VarDecl, node_sub_enum};
use kodept_ast::traits::AsEnum;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::var;
use kodept_inference::r#type::{MonomorphicType, PolymorphicType, Tuple};
use kodept_macros::error::report::{ReportMessage, Severity};

use crate::node_family::Errors::Undefined;
use crate::scope::{ScopeError, ScopeSearch, ScopeTree};

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

impl From<Errors> for ReportMessage {
    fn from(value: Errors) -> Self {
        match value {
            Errors::TooComplex => {
                Self::new(Severity::Bug, "TI006", "Still in development".to_string())
            }
            Undefined(reference) => Self::new(
                Severity::Error,
                "TI007",
                format!("Undefined reference: {reference}"),
            ),
            Errors::Scope(e) => e.into(),
        }
    }
}

impl TypeRestrictedNode {
    pub fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &PermTkn,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors> {
        <Self as HasRestrictedType>::type_of(self, ast, token, scopes)
    }
}

trait HasRestrictedType {
    fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &PermTkn,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors>;
}

impl HasRestrictedType for TypeRestrictedNode {
    fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &PermTkn,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors> {
        let mut a0 = Assumptions::empty();
        let scope = scopes.lookup(self, ast, token)?;

        match self.as_enum() {
            TypeRestrictedNodeEnum::TypedParameter(x) => return x.type_of(ast, token, scopes),
            TypeRestrictedNodeEnum::Function(node) => {
                let assumptions: Result<Vec<_>, _> = node
                    .parameters(ast, token)
                    .into_iter()
                    .map(|it| it.type_of(ast, token, scopes))
                    .collect();
                a0 = assumptions?
                    .into_iter()
                    .fold(a0, |acc, next| acc.merge(next));
                // TODO: add full lambda type
            }
            TypeRestrictedNodeEnum::Variable(node) => {
                if let Some(ty) = node.assigned_type(ast, token) {
                    let model = var(&node.name).into();
                    a0.push(Rc::new(model), Rc::new(convert(ty, scope, ast, token)?.into()));
                }
            }
            TypeRestrictedNodeEnum::Reference(Ref {
                ident: TypeReference { name },
                ..
            }) => {
                let ty = scope.ty(name).ok_or(Undefined(name.clone()))?;
                a0.push(Rc::new(var(name).into()), Rc::new(ty.into()));
            }
            _ => {}
        };

        Ok(a0)
    }
}

impl HasRestrictedType for TyParam {
    fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &PermTkn,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors> {
        let mut a0 = Assumptions::empty();
        let scope = scopes.lookup(self, ast, token)?;
        let ty = self.parameter_type(ast, token);
        let model = var(&self.name).into();
        a0.push(Rc::new(model), Rc::new(convert(ty, scope, ast, token)?.into()));
        Ok(a0)
    }
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
