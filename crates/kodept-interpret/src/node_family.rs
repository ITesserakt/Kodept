use std::collections::HashMap;
use std::rc::Rc;

use derive_more::{From, Into};

use kodept_ast::{
    AbstractFunctionDeclaration, Application, BodiedFunctionDeclaration, ExpressionBlock, Identifier,
    IfExpression, InitializedVariable, Lambda, Literal, Reference, Type, TypedParameter,
    Variable, wrapper,
};
use kodept_ast::graph::{GenericASTNode, GhostToken, NodeId, NodeUnion, SyntaxTree};
use kodept_ast::Identifier::TypeReference;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::{Language, var};
use kodept_inference::r#type::{MonomorphicType, PolymorphicType, Tuple, Union};
use kodept_macros::error::report::{ReportMessage, Severity};

use crate::node_family::Errors::Undefined;
use crate::scope::{ScopeError, ScopeSearch, ScopeTree};
use crate::type_checker::InferError;

pub struct Store {
    models_cache: HashMap<NodeId<GenericASTNode>, Rc<Language>>,
    types_cache: HashMap<NodeId<GenericASTNode>, Rc<PolymorphicType>>,
    constraints_cache: HashMap<NodeId<GenericASTNode>, Assumptions>,
}

pub struct Context<'a> {
    scopes: &'a ScopeTree,
    ast: &'a SyntaxTree,
    token: &'a GhostToken,
}

pub trait NodeWithType {
    fn to_model(&self, context: Context) -> Result<Rc<Language>, ScopeError>;
    fn type_of(&self, context: Context) -> Result<PolymorphicType, InferError>;
    fn constraints(&self, context: Context) -> Result<Assumptions, ScopeError>;
}

wrapper! {
    #[derive(From, Into, PartialEq, Debug)]
    wrapper TypeDerivableNode {
        function(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x),
        expression_block(ExpressionBlock) = GenericASTNode::ExpressionBlock(x) => Some(x),
        init_var(InitializedVariable) = GenericASTNode::InitializedVariable(x) => Some(x),
        lambda(Lambda) = GenericASTNode::Lambda(x) => Some(x),
        application(Application) = GenericASTNode::Application(x) => Some(x),
        if_expr(IfExpression) = GenericASTNode::If(x) => Some(x),
        reference(Reference) = GenericASTNode::Reference(x@Reference{ ident: Identifier::Reference { .. }, .. }) => Some(x),

        literal(Literal) = x if Literal::contains(x) => x.try_into().ok(),
    }
}

wrapper! {
    #[derive(From, Into)]
    wrapper TypeRestrictedNode {
        typed_parameter(TypedParameter) = GenericASTNode::TypedParameter(x) => Some(x),
        function(AbstractFunctionDeclaration) = GenericASTNode::AbstractFunction(x) => Some(x),
        variable(Variable) = GenericASTNode::Variable(x) => Some(x),
        reference(Reference) = GenericASTNode::Reference(x@Reference { ident: TypeReference { .. }, .. }) => Some(x),

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
        token: &GhostToken,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors> {
        <Self as HasRestrictedType>::type_of(self, ast, token, scopes)
    }
}

trait HasRestrictedType {
    fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &GhostToken,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors>;
}

impl HasRestrictedType for TypeRestrictedNode {
    fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &GhostToken,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, Errors> {
        if let Some(node) = self.as_typed_parameter() {
            return node.type_of(ast, token, scopes);
        }
        let mut a0 = Assumptions::empty();
        let scope = scopes.lookup(self, ast, token)?;
        if let Some(node) = self.as_variable() {
            if let Some(ty) = node.assigned_type(ast, token) {
                let model = var(&node.name).into();
                a0.push(Rc::new(model), Rc::new(convert(ty, scope, ast, token)?));
            }
        } else if let Some(node) = self.as_function() {
            let assumptions: Result<Vec<_>, _> = node
                .parameters(ast, token)
                .into_iter()
                .map(|it| it.type_of(ast, token, scopes))
                .collect();
            a0 = assumptions?
                .into_iter()
                .fold(a0, |acc, next| acc.merge(next));
            // TODO: add full lambda type
        } else if let Some(Reference {
            ident: TypeReference { name },
            ..
        }) = self.as_reference()
        {
            let ty = scope.ty(name).ok_or(Undefined(name.clone()))?;
            a0.push(Rc::new(var(name).into()), Rc::new(ty));
        }
        Ok(a0)
    }
}

impl HasRestrictedType for TypedParameter {
    fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &GhostToken,
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
    token: &GhostToken,
) -> Result<PolymorphicType, Errors> {
    if let Some(constant) = ty.as_type_name() {
        scope
            .ty(&constant.name)
            .ok_or(Undefined(constant.name.clone()))
    } else if let Some(tuple) = ty.as_tuple() {
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
    } else if let Some(union) = ty.as_union() {
        let types: Result<Vec<_>, _> = union
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
    } else {
        unreachable!()
    }
}
