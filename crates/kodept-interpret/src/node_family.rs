use std::rc::Rc;

use derive_more::{From, Into};

use kodept_ast::graph::{GenericASTNode, NodeUnion, PermTkn, SyntaxTree};
use kodept_ast::Identifier::TypeReference;
use kodept_ast::{
    wrapper, AbstractFunctionDeclaration, Application, BodiedFunctionDeclaration, ExpressionBlock,
    ForceInto, Identifier, IfExpression, InitializedVariable, Lambda, Literal, Reference, Type,
    TypeEnum, TypedParameter, Variable,
};
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::var;
use kodept_inference::r#type::{MonomorphicType, PolymorphicType, Tuple, Union};
use kodept_macros::error::report::{ReportMessage, Severity};

use crate::node_family::Errors::Undefined;
use crate::scope::{ScopeError, ScopeSearch, ScopeTree};

wrapper! {
    #[derive(From, Into, PartialEq, Debug)]
    pub wrapper TypeDerivableNode {
        function(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => x.into(),
        expression_block(ExpressionBlock) = GenericASTNode::ExpressionBlock(x) => x.into(),
        init_var(InitializedVariable) = GenericASTNode::InitializedVariable(x) => x.into(),
        lambda(Lambda) = GenericASTNode::Lambda(x) => x.into(),
        application(Application) = GenericASTNode::Application(x) => x.into(),
        if_expr(IfExpression) = GenericASTNode::If(x) => x.into(),
        reference(Reference) = GenericASTNode::Reference(x@Reference{ ident: Identifier::Reference { .. }, .. }) => x.into(),

        literal(Literal) = x if Literal::contains(x) => x.force_into::<Literal>().into(),
    }
}

wrapper! {
    #[derive(From, Into)]
    pub wrapper TypeRestrictedNode {
        typed_parameter(TypedParameter) = GenericASTNode::TypedParameter(x) => x.into(),
        function(AbstractFunctionDeclaration) = GenericASTNode::AbstractFunction(x) => x.into(),
        variable(Variable) = GenericASTNode::Variable(x) => x.into(),
        reference(Reference) = GenericASTNode::Reference(x@Reference { ident: TypeReference { .. }, .. }) => x.into(),

        // Literals are not suitable here because they don't have name
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
            TypeRestrictedNodeEnum::Variable(node) => match node.assigned_type(ast, token) {
                Some(ty) => {
                    let model = var(&node.name).into();
                    a0.push(Rc::new(model), Rc::new(convert(ty, scope, ast, token)?));
                }
                _ => {}
            },
            TypeRestrictedNodeEnum::Reference(Reference {
                ident: TypeReference { name },
                ..
            }) => {
                let ty = scope.ty(name).ok_or(Undefined(name.clone()))?;
                a0.push(Rc::new(var(name).into()), Rc::new(ty));
            }
            _ => {}
        };

        return Ok(a0);
    }
}

impl HasRestrictedType for TypedParameter {
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
        a0.push(Rc::new(model), Rc::new(convert(ty, scope, ast, token)?));
        Ok(a0)
    }
}

fn convert(
    ty: &Type,
    scope: ScopeSearch,
    ast: &SyntaxTree,
    token: &PermTkn,
) -> Result<PolymorphicType, Errors> {
    return match ty.as_enum() {
        TypeEnum::TypeName(constant) => scope
            .ty(&constant.name)
            .ok_or(Undefined(constant.name.clone())),
        TypeEnum::Tuple(tuple) => {
            let types: Result<Vec<_>, _> = tuple
                .types(ast, token)
                .into_iter()
                .map(|it| match convert(it, scope.clone(), ast, token) {
                    Ok(PolymorphicType::Monomorphic(x)) => Ok(x),
                    Err(e) => Err(e),
                    _ => Err(Errors::TooComplex),
                })
                .collect();
            let types = types?;
            Ok(MonomorphicType::Tuple(Tuple::new(types)).into())
        }
        TypeEnum::Union(tuple) => {
            let types: Result<Vec<_>, _> = tuple
                .types(ast, token)
                .into_iter()
                .map(|it| match convert(it, scope.clone(), ast, token) {
                    Ok(PolymorphicType::Monomorphic(x)) => Ok(x),
                    Err(e) => Err(e),
                    _ => Err(Errors::TooComplex),
                })
                .collect();
            let types = types?;
            Ok(MonomorphicType::Union(Union::new(types)).into())
        }
    };
}
